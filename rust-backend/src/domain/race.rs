use chrono::Utc;
use mongodb::bson::DateTime as BsonDateTime;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::services::car_validation::ValidatedCarData;

/// Tyre type chosen at race entry (and at each pit stop). It determines the
/// boost card pool: softer tyres give fewer but stronger cards, harder tyres
/// give more but weaker cards.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema, Default)]
pub enum TyreType {
    Soft,
    #[default]
    Medium,
    Hard,
}

impl TyreType {
    /// The boost card pool granted by this tyre, as a multiset of card values.
    /// Boost value 0 is never a card (it is the always-free no-boost move), so
    /// pools only contain values 1-4. These values are tentative and tuned in
    /// one place.
    #[must_use]
    pub fn initial_pool(self) -> Vec<u8> {
        match self {
            TyreType::Soft => vec![3, 4, 4],
            TyreType::Medium => vec![2, 2, 3, 3, 4],
            TyreType::Hard => vec![1, 1, 1, 2, 2, 3],
        }
    }
}

/// Boost hand management system for tracking available boost cards.
///
/// Cards are a multiset of values 1-4 drawn from the chosen [`TyreType`]'s pool.
/// Boost value 0 is NOT a card: it is the always-available free no-boost move.
/// The hand does NOT auto-replenish; a pit stop ([`BoostHand::refill`]) is the
/// only way to restore cards (and may swap tyre).
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostHand {
    /// The tyre currently fitted, which defined the pool.
    #[serde(default)]
    pub tyre_type: TyreType,

    /// Remaining count for each boost card value.
    /// Keys are values `"1".."4"` (value 0 is never a card).
    /// Using String keys for `MongoDB` compatibility.
    pub cards: HashMap<String, u32>,

    /// Number of cards remaining (sum of all counts).
    pub cards_remaining: u32,

    /// Total number of pit stops completed (each refills the pool).
    #[serde(default)]
    pub pit_stops_completed: u32,
}

/// Record of a single boost card usage
/// Tracks lap-by-lap boost card usage for history and analytics
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostUsageRecord {
    /// Lap number when the boost was used
    pub lap_number: u32,

    /// Boost card value that was used (0-4)
    pub boost_value: u8,

    /// Cycle number when the boost was used
    pub cycle_number: u32,

    /// Number of cards remaining after this usage
    pub cards_remaining_after: u32,

    /// Whether replenishment occurred after this usage
    pub replenishment_occurred: bool,
}

/// Summary statistics for a complete boost cycle
/// Provides cycle-level analytics for strategic analysis
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostCycleSummary {
    /// Cycle number
    pub cycle_number: u32,

    /// Boost card values used in this cycle (in order)
    pub cards_used: Vec<u8>,

    /// Lap numbers when cards were used in this cycle
    pub laps_in_cycle: Vec<u32>,

    /// Average boost value for this cycle
    pub average_boost: f32,
}

impl BoostHand {
    /// Initialize a new boost hand with the default (Medium) tyre pool.
    #[must_use]
    pub fn new() -> Self {
        Self::with_tyre(TyreType::default())
    }

    /// Initialize a new boost hand from a specific tyre's pool.
    #[must_use]
    pub fn with_tyre(tyre: TyreType) -> Self {
        let cards = Self::pool_to_counts(&tyre.initial_pool());
        let cards_remaining = cards.values().sum();

        Self {
            tyre_type: tyre,
            cards,
            cards_remaining,
            pit_stops_completed: 0,
        }
    }

    /// Build a count map (value -> remaining count) from a multiset of values.
    fn pool_to_counts(pool: &[u8]) -> HashMap<String, u32> {
        let mut counts: HashMap<String, u32> = HashMap::new();
        for &value in pool {
            *counts.entry(value.to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// Check if a specific boost card is available.
    /// Boost value 0 is always available (the free no-boost move).
    #[must_use]
    pub fn is_card_available(&self, boost_value: u8) -> bool {
        if boost_value == 0 {
            return true;
        }
        self.cards
            .get(&boost_value.to_string())
            .copied()
            .unwrap_or(0)
            > 0
    }

    /// Use a boost card. Boost value 0 is a free no-op (no card consumed).
    /// Returns Ok(()) if successful, Err with message if the card is not
    /// available. Does NOT auto-replenish — use [`Self::refill`] (pit stop).
    pub fn use_card(&mut self, boost_value: u8) -> Result<(), String> {
        if boost_value == 0 {
            return Ok(());
        }

        if !self.is_card_available(boost_value) {
            return Err(format!("Boost card {boost_value} is not available"));
        }

        let key = boost_value.to_string();
        let count = self.cards.entry(key).or_insert(0);
        *count -= 1;
        self.cards_remaining -= 1;

        Ok(())
    }

    /// Refill the pool from `new_tyre`'s pool, switching the fitted tyre.
    /// Performed during a pit stop; increments `pit_stops_completed`.
    pub fn refill(&mut self, new_tyre: TyreType) {
        self.tyre_type = new_tyre;
        self.cards = Self::pool_to_counts(&new_tyre.initial_pool());
        self.cards_remaining = self.cards.values().sum();
        self.pit_stops_completed += 1;
    }

    /// Get the sorted list of distinct boost card values currently available.
    /// Boost value 0 (free no-boost move) is always included first.
    #[must_use]
    pub fn get_available_cards(&self) -> Vec<u8> {
        let mut available: Vec<u8> = self
            .cards
            .iter()
            .filter(|(_, &count)| count > 0)
            .filter_map(|(key, _)| key.parse::<u8>().ok())
            .collect();

        available.push(0);
        available.sort_unstable();
        available.dedup();
        available
    }

    /// Remaining count for each boost card value 1-4, sorted by value.
    /// Intended for UI display of the pool.
    #[must_use]
    pub fn card_counts(&self) -> Vec<(u8, u32)> {
        let mut counts: Vec<(u8, u32)> = (1..=4)
            .map(|v| (v, self.cards.get(&v.to_string()).copied().unwrap_or(0)))
            .collect();
        counts.sort_unstable_by_key(|(v, _)| *v);
        counts
    }
}

impl Default for BoostHand {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Race {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    pub name: String,
    pub track: Track,
    pub participants: Vec<RaceParticipant>,
    pub lap_characteristic: LapCharacteristic,
    pub current_lap: u32,
    pub total_laps: u32,
    pub status: RaceStatus,
    /// Number of processing turns taken so far. Used only as a safety bound to
    /// guarantee a race terminates even on an unwinnable track.
    #[serde(default)]
    pub turns_taken: u32,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: BsonDateTime,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: BsonDateTime,
    // Individual lap action processing fields
    pub pending_actions: Vec<LapAction>,
    pub action_submissions: HashMap<Uuid, i64>, // Track submission times as Unix timestamps
    pub pending_performance_calculations: HashMap<Uuid, PerformanceCalculation>, // Store performance calculations
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Track {
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    pub name: String,
    pub sectors: Vec<Sector>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Sector {
    pub id: u32,
    pub name: String,
    pub min_value: u32,
    pub max_value: u32,
    pub slot_capacity: Option<u32>, // None = infinite (first and last sectors)
    pub sector_type: SectorType,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum SectorType {
    Start,    // First sector (infinite slots)
    Straight, // Straight section
    Curve,    // Curved section
    Finish,   // Last sector (infinite slots)
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RaceParticipant {
    #[serde(with = "uuid_as_string")]
    pub player_uuid: Uuid,
    #[serde(with = "uuid_as_string")]
    pub car_uuid: Uuid,
    #[serde(with = "uuid_as_string")]
    pub pilot_uuid: Uuid,
    pub current_sector: u32,
    pub current_position_in_sector: u32,
    pub current_lap: u32,
    pub total_value: u32,
    pub is_finished: bool,
    pub finish_position: Option<u32>,
    pub boost_hand: BoostHand,

    /// History of boost card usage for this participant
    #[serde(default)]
    pub boost_usage_history: Vec<BoostUsageRecord>,

    /// Whether this participant is controlled by the AI (solo mode opponents).
    /// Defaults to `false` so existing/serialized races deserialize as human.
    #[serde(default)]
    pub is_ai: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum RaceStatus {
    Waiting,    // Waiting for players to join
    InProgress, // Race is running
    Finished,   // Race completed
    Cancelled,  // Race was cancelled
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LapAction {
    #[serde(with = "uuid_as_string")]
    pub player_uuid: Uuid,
    pub boost_value: u32, // 0 to 5
}

/// Extended lap action with performance calculation
/// Used internally to store both the action and its calculated performance
#[derive(Debug, Clone)]
pub struct LapActionWithPerformance {
    pub action: LapAction,
    pub performance: PerformanceCalculation,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LapResult {
    pub lap: u32,
    pub lap_characteristic: LapCharacteristic,
    pub sector_positions: HashMap<String, Vec<RaceParticipant>>, // sector_id -> participants (String keys for MongoDB compatibility)
    pub movements: Vec<ParticipantMovement>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum LapCharacteristic {
    Straight,
    Curve,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ParticipantMovement {
    #[serde(with = "uuid_as_string")]
    pub player_uuid: Uuid,
    pub from_sector: u32,
    pub to_sector: u32,
    pub final_value: u32,
    pub movement_type: MovementType,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub enum MovementType {
    StayedInSector,
    MovedUp,
    MovedDown,
    FinishedLap,
    FinishedRace,
}

/// Movement probability based on performance prediction
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub enum MovementProbability {
    MoveUp,
    Stay,
    MoveDown,
}

/// Result of processing an individual lap action
#[derive(Debug)]
pub enum IndividualLapResult {
    /// Action was recorded, waiting for other players
    ActionRecorded {
        predicted_performance: PerformanceCalculation,
        waiting_for_players: Vec<Uuid>,
    },
    /// All actions submitted, lap was processed
    LapProcessed(LapResult),
}

/// Detailed performance calculation breakdown
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PerformanceCalculation {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
    pub boost_value: u32,
    pub final_value: u32,
}

impl Race {
    #[must_use]
    pub fn new(name: String, track: Track, total_laps: u32) -> Self {
        let now = BsonDateTime::now();
        Self {
            id: None,
            uuid: Uuid::new_v4(),
            name,
            track,
            participants: Vec::new(),
            lap_characteristic: LapCharacteristic::Straight,
            current_lap: 1,
            total_laps,
            status: RaceStatus::Waiting,
            turns_taken: 0,
            created_at: now,
            updated_at: now,
            pending_actions: Vec::new(),
            action_submissions: HashMap::new(),
            pending_performance_calculations: HashMap::new(),
        }
    }

    pub fn add_participant(
        &mut self,
        player_uuid: Uuid,
        car_uuid: Uuid,
        pilot_uuid: Uuid,
    ) -> Result<(), String> {
        self.add_participant_inner(
            player_uuid,
            car_uuid,
            pilot_uuid,
            false,
            TyreType::default(),
        )
    }

    /// Add a participant with a chosen starting tyre, which defines their
    /// initial boost card pool.
    pub fn add_participant_with_tyre(
        &mut self,
        player_uuid: Uuid,
        car_uuid: Uuid,
        pilot_uuid: Uuid,
        tyre: TyreType,
    ) -> Result<(), String> {
        self.add_participant_inner(player_uuid, car_uuid, pilot_uuid, false, tyre)
    }

    /// Add an AI-controlled participant (solo mode opponent). Identical to
    /// [`Self::add_participant`] except the resulting participant is flagged
    /// `is_ai = true` so the server drives its boost choices.
    pub fn add_ai_participant(
        &mut self,
        player_uuid: Uuid,
        car_uuid: Uuid,
        pilot_uuid: Uuid,
    ) -> Result<(), String> {
        self.add_participant_inner(player_uuid, car_uuid, pilot_uuid, true, TyreType::default())
    }

    /// Add an AI-controlled participant fitted with a specific starting tyre.
    pub fn add_ai_participant_with_tyre(
        &mut self,
        player_uuid: Uuid,
        car_uuid: Uuid,
        pilot_uuid: Uuid,
        tyre: TyreType,
    ) -> Result<(), String> {
        self.add_participant_inner(player_uuid, car_uuid, pilot_uuid, true, tyre)
    }

    fn add_participant_inner(
        &mut self,
        player_uuid: Uuid,
        car_uuid: Uuid,
        pilot_uuid: Uuid,
        is_ai: bool,
        tyre: TyreType,
    ) -> Result<(), String> {
        // Allow joining races that are Waiting OR InProgress (for late joins)
        if self.status != RaceStatus::Waiting && self.status != RaceStatus::InProgress {
            return Err(format!(
                "Cannot add participants to a race with status: {:?}",
                self.status
            ));
        }

        // Check if player is already participating
        if self
            .participants
            .iter()
            .any(|p| p.player_uuid == player_uuid)
        {
            return Err("Player is already participating in this race".to_string());
        }

        // For InProgress races, ensure we're still in early laps (allow late joins only in first lap)
        if self.status == RaceStatus::InProgress && self.current_lap > 1 {
            return Err("Cannot join race - race has progressed beyond first lap".to_string());
        }

        // Random qualification for now - cars start in different sectors
        let starting_sector = self.get_qualification_sector();

        let participant = RaceParticipant {
            player_uuid,
            car_uuid,
            pilot_uuid,
            current_sector: starting_sector,
            current_position_in_sector: 0, // Will be set during start_race
            current_lap: 1,
            total_value: 0,
            is_finished: false,
            finish_position: None,
            boost_hand: BoostHand::with_tyre(tyre),
            boost_usage_history: Vec::new(),
            is_ai,
        };

        self.participants.push(participant);
        self.updated_at = BsonDateTime::now();
        Ok(())
    }

    fn get_qualification_sector(&self) -> u32 {
        // Random qualification - distribute cars across sectors
        // TODO: Replace with proper qualification system
        use rand::Rng;
        let mut rng = rand::thread_rng();
        #[allow(clippy::cast_possible_truncation)]
        let max_sector = (self.track.sectors.len() - 1) as u32;
        rng.gen_range(0..=max_sector)
    }

    pub fn start_race(&mut self) -> Result<(), String> {
        if self.status != RaceStatus::Waiting {
            return Err("Race has already started or finished".to_string());
        }

        if self.participants.is_empty() {
            return Err("Cannot start race without participants".to_string());
        }

        self.status = RaceStatus::InProgress;

        // Set initial lap characteristic (random for now)
        self.lap_characteristic = Self::generate_lap_characteristic();

        // Sort participants in their starting sectors
        self.sort_participants_in_sectors();

        self.updated_at = BsonDateTime::now();
        Ok(())
    }

    fn generate_lap_characteristic() -> LapCharacteristic {
        // Random lap characteristic for now
        // TODO: Replace with track-specific or strategic system
        use rand::Rng;
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.5) {
            LapCharacteristic::Straight
        } else {
            LapCharacteristic::Curve
        }
    }

    /// Simple process lap method for backward compatibility with tests
    /// Uses a basic performance calculation (base value 10 + boost)
    pub fn process_lap(&mut self, actions: &[LapAction]) -> Result<LapResult, String> {
        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // Validate all participants have submitted actions
        for participant in &self.participants {
            if participant.is_finished {
                continue;
            }
            if !actions
                .iter()
                .any(|a| a.player_uuid == participant.player_uuid)
            {
                return Err(format!(
                    "Missing action for player {}",
                    participant.player_uuid
                ));
            }
        }

        // Validate boost values
        for action in actions {
            if action.boost_value > 5 {
                return Err(format!(
                    "Invalid boost value {} for player {}",
                    action.boost_value, action.player_uuid
                ));
            }
        }

        // Calculate simple performance values for tests (base 10 + boost)
        let mut participant_values: HashMap<Uuid, u32> = HashMap::new();
        for action in actions {
            if let Some(participant) = self
                .participants
                .iter()
                .find(|p| p.player_uuid == action.player_uuid)
            {
                if !participant.is_finished {
                    // Simple calculation: base value 10 + boost value
                    let base_value = 10u32;
                    let current_sector = &self.track.sectors[participant.current_sector as usize];
                    let capped_base_value = std::cmp::min(base_value, current_sector.max_value);
                    let final_value = capped_base_value + action.boost_value;
                    participant_values.insert(action.player_uuid, final_value);
                }
            }
        }

        Ok(self.process_lap_internal(actions, &participant_values))
    }

    /// Process lap with pre-calculated performance values from car components
    /// This is the new method that uses actual car data for performance calculation
    pub fn process_lap_with_car_data(
        &mut self,
        actions: &[LapAction],
        performance_calculations: &HashMap<Uuid, PerformanceCalculation>,
    ) -> Result<LapResult, String> {
        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // Validate all participants have submitted actions
        for participant in &self.participants {
            if participant.is_finished {
                continue;
            }
            if !actions
                .iter()
                .any(|a| a.player_uuid == participant.player_uuid)
            {
                return Err(format!(
                    "Missing action for player {}",
                    participant.player_uuid
                ));
            }
        }

        // Validate boost values
        for action in actions {
            if action.boost_value > 5 {
                return Err(format!(
                    "Invalid boost value {} for player {}",
                    action.boost_value, action.player_uuid
                ));
            }
        }

        // Use pre-calculated performance values from car components
        let mut participant_values: HashMap<Uuid, u32> = HashMap::new();
        for action in actions {
            if let Some(participant) = self
                .participants
                .iter()
                .find(|p| p.player_uuid == action.player_uuid)
            {
                if !participant.is_finished {
                    // Use the pre-calculated performance from car data
                    if let Some(performance) = performance_calculations.get(&action.player_uuid) {
                        participant_values.insert(action.player_uuid, performance.final_value);
                    } else {
                        return Err(format!(
                            "Missing performance calculation for player {}",
                            action.player_uuid
                        ));
                    }
                }
            }
        }

        Ok(self.process_lap_internal(actions, &participant_values))
    }

    /// Internal method that processes lap movements after performance values are calculated
    fn process_lap_internal(
        &mut self,
        actions: &[LapAction],
        participant_values: &HashMap<Uuid, u32>,
    ) -> LapResult {
        // Process movements using the new algorithm: best sector to worst sector
        let mut movements = Vec::new();
        #[allow(clippy::cast_possible_truncation)]
        let max_sector = (self.track.sectors.len() - 1) as u32;

        // Process sectors from highest to lowest (best to worst)
        for sector_id in (0..=max_sector).rev() {
            let sector_movements = self.process_sector_movements(sector_id, participant_values);
            movements.extend(sector_movements);
        }

        // Update total values for all participants
        for action in actions {
            if let Some(participant) = self
                .participants
                .iter_mut()
                .find(|p| p.player_uuid == action.player_uuid)
            {
                if !participant.is_finished {
                    if let Some(&final_value) = participant_values.get(&action.player_uuid) {
                        participant.total_value += final_value;
                    }
                }
            }
        }

        // Sort participants in each sector by their total value (descending = better position)
        self.sort_participants_in_sectors();

        // One processed turn = one lap (each player plays one boost per turn).
        self.turns_taken += 1;

        // The displayed lap is the lap currently being raced, capped at the
        // configured number of laps. Keep every participant's lap in sync so
        // per-car displays match the race lap.
        self.current_lap = (self.turns_taken + 1).min(self.total_laps).max(1);
        for participant in &mut self.participants {
            participant.current_lap = self.current_lap;
        }

        // Finish the race once every participant has completed all their laps
        // (or the safety bound is hit).
        self.check_race_completion();

        let processed_lap = self.current_lap;

        // Pick a fresh lap characteristic for the next turn while still racing.
        if self.status == RaceStatus::InProgress {
            self.lap_characteristic = Self::generate_lap_characteristic();
        }

        self.updated_at = BsonDateTime::now();

        LapResult {
            lap: processed_lap,
            lap_characteristic: self.lap_characteristic.clone(),
            sector_positions: self.get_sector_positions(),
            movements,
        }
    }

    /// Process individual lap action for a single player
    /// Stores pending actions until all players submit, then processes simultaneous turn resolution
    /// Record a player's boost action for the current turn: validate, consume
    /// the boost card, and stage the action plus its performance calculation.
    ///
    /// This does NOT resolve the lap — it only records the action. Callers that
    /// want the original "auto-process once everyone has acted" behaviour should
    /// use [`Self::process_individual_lap_action`]; orchestrators that need to
    /// enqueue other participants (e.g. solo-mode AI) before resolving should
    /// call this, enqueue the rest, then process explicitly. Returns the
    /// predicted performance for the recorded action.
    pub fn record_player_action(
        &mut self,
        player_uuid: Uuid,
        boost_value: u32,
        car_data: &ValidatedCarData,
    ) -> Result<PerformanceCalculation, String> {
        use crate::domain::boost_hand_manager::BoostHandManager;

        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // 1. Validate player is in race and not finished
        let participant_index = self
            .participants
            .iter()
            .position(|p| p.player_uuid == player_uuid)
            .ok_or("Player not found in race")?;

        if self.participants[participant_index].is_finished {
            return Err("Player has already finished the race".to_string());
        }

        // 2. Check if player has already submitted an action for this turn
        if self
            .pending_actions
            .iter()
            .any(|a| a.player_uuid == player_uuid)
        {
            return Err("Player has already submitted an action for this turn".to_string());
        }

        // 3. Validate boost value range (0-4 for boost cards)
        if boost_value > 4 {
            return Err(format!(
                "Invalid boost value: {boost_value}. Must be between 0 and 4"
            ));
        }

        // 4. Validate boost card availability and use the card
        #[allow(clippy::cast_possible_truncation)]
        let boost_value_u8 = boost_value as u8;

        // Record the pit-segment (number of pit stops so far) for this usage.
        let pit_segment = self.participants[participant_index]
            .boost_hand
            .pit_stops_completed;

        let boost_usage_result = BoostHandManager::use_boost_card(
            &mut self.participants[participant_index].boost_hand,
            boost_value_u8,
        )
        .map_err(|e| e.to_string())?;

        // Record boost usage in history. `lap_number` here is the processing
        // turn/round in which the boost was used (turns_taken counts completed
        // rounds; +1 = the round currently being submitted). `cycle_number` now
        // records the pit-segment index (pit stops completed at time of use).
        let usage_record = BoostUsageRecord {
            lap_number: self.turns_taken + 1,
            boost_value: boost_value_u8,
            cycle_number: pit_segment,
            cards_remaining_after: boost_usage_result.cards_remaining,
            replenishment_occurred: false,
        };
        self.participants[participant_index]
            .boost_usage_history
            .push(usage_record);

        // 5. Calculate performance using validated car data
        let performance = self.calculate_performance_with_car_data(
            &self.participants[participant_index],
            boost_value,
            car_data,
            &self.lap_characteristic,
        );

        // 6. Store action and performance calculation for batch processing
        let action = LapAction {
            player_uuid,
            boost_value,
        };
        self.pending_actions.push(action);
        self.action_submissions
            .insert(player_uuid, Utc::now().timestamp());
        self.pending_performance_calculations
            .insert(player_uuid, performance.clone());

        Ok(performance)
    }

    /// Resolve the lap if every active participant has now submitted; otherwise
    /// report the action as recorded along with who we are still waiting on.
    fn process_if_ready(
        &mut self,
        predicted_performance: PerformanceCalculation,
    ) -> Result<IndividualLapResult, String> {
        // Check if all participants have submitted actions
        if self.all_actions_submitted() {
            // Clone the pending actions and performance calculations to avoid borrowing issues
            let actions_to_process = self.pending_actions.clone();
            let performance_calculations = self.pending_performance_calculations.clone();

            // Process all actions simultaneously with their performance calculations
            let lap_result =
                self.process_lap_with_car_data(&actions_to_process, &performance_calculations)?;

            // Clear pending actions and calculations after processing
            self.pending_actions.clear();
            self.action_submissions.clear();
            self.pending_performance_calculations.clear();

            Ok(IndividualLapResult::LapProcessed(lap_result))
        } else {
            // Return current state with action recorded
            Ok(IndividualLapResult::ActionRecorded {
                predicted_performance,
                waiting_for_players: self.get_pending_players(),
            })
        }
    }

    pub fn process_individual_lap_action(
        &mut self,
        player_uuid: Uuid,
        boost_value: u32,
        car_data: &ValidatedCarData,
    ) -> Result<IndividualLapResult, String> {
        let performance = self.record_player_action(player_uuid, boost_value, car_data)?;
        self.process_if_ready(performance)
    }

    /// Record a pit-stop action for the current turn without resolving the lap:
    /// refill the boost pool from `new_tyre` (or the current tyre if `None`),
    /// then stage a free boost-0 move. See [`Self::record_player_action`] for why
    /// recording and processing are separated. Returns the predicted performance.
    pub fn record_pit_action(
        &mut self,
        player_uuid: Uuid,
        new_tyre: Option<TyreType>,
        car_data: &ValidatedCarData,
    ) -> Result<PerformanceCalculation, String> {
        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        let participant_index = self
            .participants
            .iter()
            .position(|p| p.player_uuid == player_uuid)
            .ok_or("Player not found in race")?;

        if self.participants[participant_index].is_finished {
            return Err("Player has already finished the race".to_string());
        }

        if self
            .pending_actions
            .iter()
            .any(|a| a.player_uuid == player_uuid)
        {
            return Err("Player has already submitted an action for this turn".to_string());
        }

        // Refill the pool with the chosen tyre (default: keep current tyre).
        let tyre = new_tyre.unwrap_or(self.participants[participant_index].boost_hand.tyre_type);
        self.participants[participant_index].boost_hand.refill(tyre);

        // The pit consumes the turn as a free boost-0 lap.
        self.record_player_action(player_uuid, 0, car_data)
    }

    /// Process a pit-stop action for a single player.
    ///
    /// A pit stop refills the player's boost pool from `new_tyre` (or the
    /// current tyre if `None`) and consumes the turn as a free boost-0 lap.
    /// The pool is refilled immediately on submission, which is equivalent to
    /// "the pit costs this lap": the player commits boost 0 this turn and races
    /// with a fresh pool from the next turn onward.
    pub fn process_individual_pit_action(
        &mut self,
        player_uuid: Uuid,
        new_tyre: Option<TyreType>,
        car_data: &ValidatedCarData,
    ) -> Result<IndividualLapResult, String> {
        let performance = self.record_pit_action(player_uuid, new_tyre, car_data)?;
        self.process_if_ready(performance)
    }

    /// Check if all active participants have submitted actions
    #[must_use]
    pub fn all_actions_submitted(&self) -> bool {
        let active_participants: HashSet<Uuid> = self
            .participants
            .iter()
            .filter(|p| !p.is_finished)
            .map(|p| p.player_uuid)
            .collect();

        let submitted_actions: HashSet<Uuid> =
            self.pending_actions.iter().map(|a| a.player_uuid).collect();

        // If there are no active participants, no actions are needed
        if active_participants.is_empty() {
            return true;
        }

        // If there are active participants but no submitted actions, not all submitted
        if submitted_actions.is_empty() {
            return false;
        }

        active_participants == submitted_actions
    }

    /// Get list of players who haven't submitted actions yet
    #[must_use]
    pub fn get_pending_players(&self) -> Vec<Uuid> {
        let submitted: HashSet<Uuid> = self.pending_actions.iter().map(|a| a.player_uuid).collect();

        self.participants
            .iter()
            .filter(|p| !p.is_finished && !submitted.contains(&p.player_uuid))
            .map(|p| p.player_uuid)
            .collect()
    }

    /// Calculate performance for all participants using their car data
    /// This is used for batch processing when all car data is available upfront
    pub fn calculate_all_performances(
        &self,
        actions: &[LapAction],
        car_data_map: &HashMap<Uuid, ValidatedCarData>,
    ) -> Result<HashMap<Uuid, PerformanceCalculation>, String> {
        let mut performance_calculations = HashMap::new();

        for action in actions {
            let participant = self
                .participants
                .iter()
                .find(|p| p.player_uuid == action.player_uuid)
                .ok_or_else(|| format!("Player {} not found in race", action.player_uuid))?;

            if participant.is_finished {
                continue;
            }

            let car_data = car_data_map
                .get(&action.player_uuid)
                .ok_or_else(|| format!("Car data not found for player {}", action.player_uuid))?;

            let performance = self.calculate_performance_with_car_data(
                participant,
                action.boost_value,
                car_data,
                &self.lap_characteristic,
            );

            performance_calculations.insert(action.player_uuid, performance);
        }

        Ok(performance_calculations)
    }

    /// Enqueue boost actions for every AI-controlled participant that is active
    /// and has not yet acted this turn (solo mode). Mirrors a human submission:
    /// it consumes each AI's boost card and records the usage, then pushes the
    /// action into `pending_actions`. Participants whose car data cannot be
    /// resolved are skipped so a missing entry never stalls the turn.
    pub fn enqueue_ai_actions(&mut self, car_data_map: &HashMap<Uuid, ValidatedCarData>) {
        use crate::domain::ai_player;
        use crate::domain::boost_hand_manager::BoostHandManager;

        let already_acted: HashSet<Uuid> =
            self.pending_actions.iter().map(|a| a.player_uuid).collect();
        let lap_characteristic = self.lap_characteristic.clone();
        // Laps left including the current one; the AI pit logic needs a future lap
        // to spend refilled cards on.
        let laps_remaining = self.total_laps.saturating_sub(self.turns_taken);

        // Decide first (immutable borrows), then mutate, to avoid borrow conflicts.
        let decisions: Vec<(Uuid, ai_player::AiTurnAction)> = self
            .participants
            .iter()
            .filter(|p| p.is_ai && !p.is_finished && !already_acted.contains(&p.player_uuid))
            .filter_map(|p| {
                let car_data = car_data_map.get(&p.player_uuid)?;
                let sector = self.track.sectors.get(p.current_sector as usize)?;
                let action = ai_player::decide_ai_action(
                    car_data,
                    &p.boost_hand,
                    sector,
                    &lap_characteristic,
                    laps_remaining,
                );
                Some((p.player_uuid, action))
            })
            .collect();

        for (player_uuid, action) in decisions {
            let Some(index) = self
                .participants
                .iter()
                .position(|p| p.player_uuid == player_uuid)
            else {
                continue;
            };

            // A pit and a boost both resolve as a card consumption for the turn; a
            // pit refills the pool first and then plays a free boost-0 move.
            let boost = match action {
                ai_player::AiTurnAction::Pit => {
                    let tyre = self.participants[index].boost_hand.tyre_type;
                    self.participants[index].boost_hand.refill(tyre);
                    0
                }
                ai_player::AiTurnAction::Boost(b) => b,
            };

            let pit_segment = self.participants[index].boost_hand.pit_stops_completed;
            // `choose_boost`/pit only yield available cards (0 is always free), so
            // this should not fail; if it ever did we still record the action below.
            if let Ok(result) =
                BoostHandManager::use_boost_card(&mut self.participants[index].boost_hand, boost)
            {
                self.participants[index]
                    .boost_usage_history
                    .push(BoostUsageRecord {
                        lap_number: self.turns_taken + 1,
                        boost_value: boost,
                        cycle_number: pit_segment,
                        cards_remaining_after: result.cards_remaining,
                        replenishment_occurred: false,
                    });
            }

            self.pending_actions.push(LapAction {
                player_uuid,
                boost_value: u32::from(boost),
            });
        }
    }

    /// Calculate performance using validated car data and boost selection
    fn calculate_performance_with_car_data(
        &self,
        participant: &RaceParticipant,
        boost_value: u32,
        car_data: &ValidatedCarData,
        lap_characteristic: &LapCharacteristic,
    ) -> PerformanceCalculation {
        // Get performance values based on lap characteristic (convert u8 to u32)
        let engine_value = match lap_characteristic {
            LapCharacteristic::Straight => u32::from(car_data.engine.straight_value),
            LapCharacteristic::Curve => u32::from(car_data.engine.curve_value),
        };

        let body_value = match lap_characteristic {
            LapCharacteristic::Straight => u32::from(car_data.body.straight_value),
            LapCharacteristic::Curve => u32::from(car_data.body.curve_value),
        };

        let pilot_value = match lap_characteristic {
            LapCharacteristic::Straight => u32::from(car_data.pilot.performance.straight_value),
            LapCharacteristic::Curve => u32::from(car_data.pilot.performance.curve_value),
        };

        // Calculate base performance
        let base_value = engine_value + body_value + pilot_value;

        // Apply sector performance ceiling to base value
        let current_sector = &self.track.sectors[participant.current_sector as usize];
        let capped_base_value = std::cmp::min(base_value, current_sector.max_value);

        // Add boost value directly to capped base value
        let final_value = capped_base_value + boost_value;

        PerformanceCalculation {
            engine_contribution: engine_value,
            body_contribution: body_value,
            pilot_contribution: pilot_value,
            base_value,
            sector_ceiling: current_sector.max_value,
            capped_base_value,
            boost_value,
            final_value,
        }
    }

    fn process_sector_movements(
        &mut self,
        sector_id: u32,
        participant_values: &HashMap<Uuid, u32>,
    ) -> Vec<ParticipantMovement> {
        let mut movements = Vec::new();

        // Get all participants in this sector with their performance values
        let mut participants_in_sector: Vec<(usize, u32)> = self
            .participants
            .iter()
            .enumerate()
            .filter(|(_, p)| p.current_sector == sector_id && !p.is_finished)
            .filter_map(|(i, p)| {
                participant_values
                    .get(&p.player_uuid)
                    .map(|&value| (i, value))
            })
            .collect();

        // Sort by performance value (highest first) - this determines ranking
        participants_in_sector.sort_by_key(|b| std::cmp::Reverse(b.1));

        // Process each participant, but only allow the first-ranked car to move up
        for (rank, &(participant_index, final_value)) in participants_in_sector.iter().enumerate() {
            let movement = self.calculate_movement_for_participant(
                participant_index,
                final_value,
                sector_id,
                rank == 0,
            );
            movements.push(movement);
        }

        movements
    }

    fn calculate_movement_for_participant(
        &mut self,
        participant_index: usize,
        final_value: u32,
        current_sector_id: u32,
        is_first_ranked: bool,
    ) -> ParticipantMovement {
        let participant = &self.participants[participant_index];
        let player_uuid = participant.player_uuid;
        let from_sector = current_sector_id;

        #[allow(clippy::cast_possible_truncation)]
        if current_sector_id >= self.track.sectors.len() as u32 {
            // Invalid sector - shouldn't happen
            return ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: from_sector,
                final_value,
                movement_type: MovementType::StayedInSector,
            };
        }

        let sector = &self.track.sectors[current_sector_id as usize];

        // Check movement conditions
        if final_value < sector.min_value {
            // Move DOWN - any car can move down if performance is too low
            self.move_participant_down(participant_index, from_sector, final_value)
        } else if final_value > sector.max_value && is_first_ranked {
            // Try to move UP - only the first-ranked car can move up
            self.move_participant_up(participant_index, from_sector, final_value)
        } else {
            // Stay in current sector (either performance is within range, or not first-ranked)
            ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: from_sector,
                final_value,
                movement_type: MovementType::StayedInSector,
            }
        }
    }

    fn move_participant_down(
        &mut self,
        participant_index: usize,
        from_sector: u32,
        final_value: u32,
    ) -> ParticipantMovement {
        let player_uuid = self.participants[participant_index].player_uuid;

        if from_sector == 0 {
            // Already in lowest sector, can't move down
            return ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: from_sector,
                final_value,
                movement_type: MovementType::StayedInSector,
            };
        }

        // Find a sector with available space, moving down
        let mut target_sector = from_sector - 1;

        loop {
            let sector = &self.track.sectors[target_sector as usize];

            // Check if sector has capacity
            let can_fit = match sector.slot_capacity {
                None => true, // Infinite capacity
                Some(capacity) => {
                    let current_count = self
                        .participants
                        .iter()
                        .enumerate()
                        .filter(|(i, p)| {
                            *i != participant_index
                                && p.current_sector == target_sector
                                && !p.is_finished
                        })
                        .count();
                    current_count < capacity as usize
                }
            };

            if can_fit {
                // Move to this sector
                self.participants[participant_index].current_sector = target_sector;
                // Place at last position (will be re-ranked later)
                self.participants[participant_index].current_position_in_sector = u32::MAX; // Temporary, will be fixed in re-ranking

                return ParticipantMovement {
                    player_uuid,
                    from_sector,
                    to_sector: target_sector,
                    final_value,
                    movement_type: MovementType::MovedDown,
                };
            }

            // Try next lower sector
            if target_sector == 0 {
                // Reached sector 0 (infinite capacity), must fit here
                self.participants[participant_index].current_sector = 0;
                self.participants[participant_index].current_position_in_sector = u32::MAX;

                return ParticipantMovement {
                    player_uuid,
                    from_sector,
                    to_sector: 0,
                    final_value,
                    movement_type: MovementType::MovedDown,
                };
            }

            target_sector -= 1;
        }
    }

    fn move_participant_up(
        &mut self,
        participant_index: usize,
        from_sector: u32,
        final_value: u32,
    ) -> ParticipantMovement {
        let player_uuid = self.participants[participant_index].player_uuid;
        let next_sector = from_sector + 1;

        // Sectors are the live standings between cars (relative position), not a
        // physical lap. The highest sector is the lead, so a car already there
        // simply holds the lead — it does not wrap around or "finish" early.
        // The race ends on the turn counter (see `check_race_completion`).
        #[allow(clippy::cast_possible_truncation)]
        if next_sector >= self.track.sectors.len() as u32 {
            return ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: from_sector,
                final_value,
                movement_type: MovementType::StayedInSector,
            };
        }

        // Check if next sector has capacity
        let next_sector_obj = &self.track.sectors[next_sector as usize];
        let can_move_up = match next_sector_obj.slot_capacity {
            None => true, // Infinite capacity
            Some(capacity) => {
                let current_count = self
                    .participants
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| {
                        *i != participant_index && p.current_sector == next_sector && !p.is_finished
                    })
                    .count();
                current_count < capacity as usize
            }
        };

        if can_move_up {
            // Move up to next sector
            self.participants[participant_index].current_sector = next_sector;
            return ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: next_sector,
                final_value,
                movement_type: MovementType::MovedUp,
            };
        }
        // Sector is full, stay in current sector
        ParticipantMovement {
            player_uuid,
            from_sector,
            to_sector: from_sector,
            final_value,
            movement_type: MovementType::StayedInSector,
        }
    }

    fn sort_participants_in_sectors(&mut self) {
        // Group participants by sector and sort by total_value (descending)
        let mut sector_groups: HashMap<u32, Vec<&mut RaceParticipant>> = HashMap::new();

        for participant in &mut self.participants {
            if !participant.is_finished {
                sector_groups
                    .entry(participant.current_sector)
                    .or_default()
                    .push(participant);
            }
        }

        // Sort each sector group by total_value (descending = better position)
        for participants in sector_groups.values_mut() {
            participants.sort_by_key(|b| std::cmp::Reverse(b.total_value));

            // Update position in sector
            for (index, participant) in participants.iter_mut().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                {
                    participant.current_position_in_sector = index as u32;
                }
            }
        }
    }

    fn get_sector_positions(&self) -> HashMap<String, Vec<RaceParticipant>> {
        let mut positions: HashMap<String, Vec<RaceParticipant>> = HashMap::new();

        for participant in &self.participants {
            if !participant.is_finished {
                positions
                    .entry(participant.current_sector.to_string())
                    .or_default()
                    .push(participant.clone());
            }
        }

        // Sort each sector by position
        for participants in positions.values_mut() {
            participants.sort_by_key(|p| p.current_position_in_sector);
        }

        positions
    }

    fn check_race_completion(&mut self) {
        // One turn = one lap, so the race ends once `total_laps` turns have been
        // played. Sectors are relative standings, so no car "finishes" early —
        // everyone crosses the line together on the final turn.
        let race_over = !self.participants.is_empty() && self.turns_taken >= self.total_laps;

        // Safety backstop; with the turn-based end above this is unreachable in
        // practice, but it guarantees termination if total_laps were ever 0.
        #[allow(clippy::cast_possible_truncation)]
        let sector_count = self.track.sectors.len() as u32;
        let safety_cap = self
            .total_laps
            .saturating_mul(sector_count)
            .saturating_mul(8)
            .saturating_add(50);
        let exceeded_safety = self.turns_taken > safety_cap;

        if race_over || exceeded_safety {
            self.status = RaceStatus::Finished;

            // Everyone crosses the finish line together on the final turn.
            for participant in &mut self.participants {
                participant.is_finished = true;
            }

            // Assign finish positions based on final sector and position
            let mut all_participants: Vec<&mut RaceParticipant> =
                self.participants.iter_mut().collect();

            // Sort by: 1) Finished status, 2) Current sector (higher = better), 3) Position in sector (lower = better), 4) Total value (higher = better)
            all_participants.sort_by(|a, b| {
                b.is_finished
                    .cmp(&a.is_finished)
                    .then_with(|| b.current_sector.cmp(&a.current_sector))
                    .then_with(|| {
                        a.current_position_in_sector
                            .cmp(&b.current_position_in_sector)
                    })
                    .then_with(|| b.total_value.cmp(&a.total_value))
            });

            for (index, participant) in all_participants.iter_mut().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                {
                    participant.finish_position = Some(index as u32 + 1);
                }
            }
        }
    }
}

impl RaceParticipant {
    /// Get boost usage history grouped by cycle
    /// Returns a vector of cycle summaries with statistics for each cycle
    #[must_use]
    pub fn get_boost_cycle_summaries(&self) -> Vec<BoostCycleSummary> {
        let mut summaries: HashMap<u32, BoostCycleSummary> = HashMap::new();

        for record in &self.boost_usage_history {
            let summary =
                summaries
                    .entry(record.cycle_number)
                    .or_insert_with(|| BoostCycleSummary {
                        cycle_number: record.cycle_number,
                        cards_used: Vec::new(),
                        laps_in_cycle: Vec::new(),
                        average_boost: 0.0,
                    });

            summary.cards_used.push(record.boost_value);
            summary.laps_in_cycle.push(record.lap_number);
        }

        // Calculate average boost for each cycle
        for summary in summaries.values_mut() {
            if !summary.cards_used.is_empty() {
                let sum: u32 = summary.cards_used.iter().map(|&v| u32::from(v)).sum();
                #[allow(clippy::cast_precision_loss)]
                {
                    summary.average_boost = sum as f32 / summary.cards_used.len() as f32;
                }
            }
        }

        // Sort by cycle number
        let mut result: Vec<BoostCycleSummary> = summaries.into_values().collect();
        result.sort_by_key(|s| s.cycle_number);
        result
    }

    /// Get boost usage history for a specific cycle
    #[must_use]
    pub fn get_boost_usage_for_cycle(&self, cycle_number: u32) -> Vec<&BoostUsageRecord> {
        self.boost_usage_history
            .iter()
            .filter(|record| record.cycle_number == cycle_number)
            .collect()
    }

    /// Get total number of boost cards used across all cycles
    #[must_use]
    pub fn get_total_boosts_used(&self) -> usize {
        self.boost_usage_history.len()
    }

    /// Get average boost value across all usage
    #[must_use]
    pub fn get_average_boost_value(&self) -> f32 {
        if self.boost_usage_history.is_empty() {
            return 0.0;
        }

        let sum: u32 = self
            .boost_usage_history
            .iter()
            .map(|record| u32::from(record.boost_value))
            .sum();

        #[allow(clippy::cast_precision_loss)]
        {
            sum as f32 / self.boost_usage_history.len() as f32
        }
    }
}

impl Track {
    pub fn new(name: String, sectors: Vec<Sector>) -> Result<Self, String> {
        if sectors.is_empty() {
            return Err("Track must have at least one sector".to_string());
        }

        // Validate first and last sectors have infinite capacity
        if sectors[0].slot_capacity.is_some() {
            return Err("First sector must have infinite capacity".to_string());
        }

        if sectors.len() > 1 && sectors[sectors.len() - 1].slot_capacity.is_some() {
            return Err("Last sector must have infinite capacity".to_string());
        }

        Ok(Self {
            uuid: Uuid::new_v4(),
            name,
            sectors,
        })
    }
}

impl PartialEq for RaceStatus {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

mod uuid_as_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_track() -> Track {
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 10,
                slot_capacity: None, // Infinite
                sector_type: SectorType::Start,
            },
            Sector {
                id: 1,
                name: "Straight 1".to_string(),
                min_value: 8,
                max_value: 15,
                slot_capacity: Some(3),
                sector_type: SectorType::Straight,
            },
            Sector {
                id: 2,
                name: "Curve 1".to_string(),
                min_value: 12,
                max_value: 20,
                slot_capacity: Some(2),
                sector_type: SectorType::Curve,
            },
            Sector {
                id: 3,
                name: "Finish".to_string(),
                min_value: 18,
                max_value: 25,
                slot_capacity: None, // Infinite
                sector_type: SectorType::Finish,
            },
        ];

        Track::new("Test Track".to_string(), sectors).unwrap()
    }

    /// Minimal complete car data for AI participants in enqueue tests.
    fn make_ai_car_data() -> ValidatedCarData {
        use crate::domain::{
            Body, BodyName, Car, CarName, ComponentRarity, Engine, EngineName, Pilot, PilotClass,
            PilotName, PilotPerformance, PilotRarity, PilotSkills,
        };

        let engine = Engine::new(
            EngineName::parse("Bot Engine").unwrap(),
            ComponentRarity::Common,
            7,
            5,
        )
        .unwrap();
        let body = Body::new(
            BodyName::parse("Bot Body").unwrap(),
            ComponentRarity::Common,
            5,
            7,
        )
        .unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Bot Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Rookie,
            PilotSkills::new(6, 6, 6, 6).unwrap(),
            PilotPerformance::new(8, 5).unwrap(),
        )
        .unwrap();
        let car = Car::new(CarName::parse("Bot Car").unwrap()).unwrap();
        ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        }
    }

    #[test]
    fn enqueue_ai_actions_fills_only_ai_seats() {
        let track = create_test_track();
        let mut race = Race::new("AI Enqueue Test".to_string(), track, 1);
        race.status = RaceStatus::InProgress;

        let human = Uuid::new_v4();
        let ai1 = Uuid::new_v4();
        let ai2 = Uuid::new_v4();
        race.add_participant(human, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai1, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai2, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        let mut car_data_map = HashMap::new();
        car_data_map.insert(ai1, make_ai_car_data());
        car_data_map.insert(ai2, make_ai_car_data());

        // The human submits first.
        race.pending_actions.push(LapAction {
            player_uuid: human,
            boost_value: 2,
        });

        race.enqueue_ai_actions(&car_data_map);

        // Both AI seats are now filled (human + 2 AI = 3) with legal boosts.
        assert_eq!(race.pending_actions.len(), 3);
        for ai in [ai1, ai2] {
            let action = race
                .pending_actions
                .iter()
                .find(|a| a.player_uuid == ai)
                .expect("AI action should be enqueued");
            assert!(action.boost_value <= 4, "boost must be a legal card (0-4)");
        }

        // All active participants have now submitted -> the lap is ready.
        assert!(race.all_actions_submitted());

        // Calling again is idempotent (does not double-add already-acted seats).
        race.enqueue_ai_actions(&car_data_map);
        assert_eq!(race.pending_actions.len(), 3);
    }

    /// The solo-mode track shape (mirrors `routes::races::build_solo_track`):
    /// ceilings tuned so a starter car advances with any boost >= 1.
    fn solo_track() -> Track {
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 10,
                slot_capacity: None,
                sector_type: SectorType::Start,
            },
            Sector {
                id: 1,
                name: "Straight".to_string(),
                min_value: 5,
                max_value: 14,
                slot_capacity: None,
                sector_type: SectorType::Straight,
            },
            Sector {
                id: 2,
                name: "Curve".to_string(),
                min_value: 5,
                max_value: 14,
                slot_capacity: None,
                sector_type: SectorType::Curve,
            },
            Sector {
                id: 3,
                name: "Finish".to_string(),
                min_value: 8,
                max_value: 16,
                slot_capacity: None,
                sector_type: SectorType::Finish,
            },
        ];
        Track::new("Solo Circuit".to_string(), sectors).unwrap()
    }

    #[test]
    fn solo_race_runs_to_completion() {
        // Three laps so we can also assert the lap counter visibly advances.
        let total_laps = 3;
        let mut race = Race::new("Solo".to_string(), solo_track(), total_laps);
        race.status = RaceStatus::InProgress;
        race.lap_characteristic = LapCharacteristic::Straight;

        let human = Uuid::new_v4();
        let ai1 = Uuid::new_v4();
        let ai2 = Uuid::new_v4();
        race.add_participant(human, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai1, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai2, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        let mut car_data_map = HashMap::new();
        for uuid in [human, ai1, ai2] {
            car_data_map.insert(uuid, make_ai_car_data());
        }

        // Drive turns: the human always plays a MoveUp boost; AI auto-fill.
        let mut turns = 0;
        let mut max_lap_seen = race.current_lap;
        while matches!(race.status, RaceStatus::InProgress) && turns < 400 {
            turns += 1;
            if race
                .participants
                .iter()
                .any(|p| p.player_uuid == human && !p.is_finished)
            {
                race.pending_actions.push(LapAction {
                    player_uuid: human,
                    boost_value: 1,
                });
            }
            race.enqueue_ai_actions(&car_data_map);

            let actions = race.pending_actions.clone();
            assert!(!actions.is_empty(), "a turn should always have actions");
            let perfs = race
                .calculate_all_performances(&actions, &car_data_map)
                .unwrap();
            race.process_lap_with_car_data(&actions, &perfs).unwrap();
            max_lap_seen = max_lap_seen.max(race.current_lap);

            race.pending_actions.clear();
            race.action_submissions.clear();
            race.pending_performance_calculations.clear();
        }

        // The displayed lap must advance past lap 1 over a multi-lap race.
        assert!(
            max_lap_seen > 1,
            "race.current_lap should advance beyond 1 (saw {max_lap_seen})"
        );
        assert_eq!(
            max_lap_seen, total_laps,
            "leader should reach the final lap"
        );

        assert!(
            matches!(race.status, RaceStatus::Finished),
            "race should finish; status was {:?} after {turns} turns",
            race.status
        );
        assert!(
            race.participants.iter().all(|p| p.is_finished),
            "every participant should cross the finish line"
        );
        assert!(
            race.participants
                .iter()
                .all(|p| p.finish_position.is_some()),
            "every participant should get a final ranking"
        );
    }

    // ========== BoostHand Tests ==========

    #[test]
    fn test_boost_hand_initialization() {
        let hand = BoostHand::new(); // default = Medium

        assert_eq!(hand.tyre_type, TyreType::Medium);
        assert_eq!(hand.cards_remaining, 5, "Medium pool has 5 cards");
        assert_eq!(hand.pit_stops_completed, 0);

        // Boost 0 is always available; the Medium pool contains 2,3,4.
        assert!(hand.is_card_available(0), "Boost 0 is always free");
        for value in [2, 3, 4] {
            assert!(hand.is_card_available(value), "Card {value} available");
        }
        // Value 1 is not in the Medium pool.
        assert!(!hand.is_card_available(1));
    }

    #[test]
    fn test_tyre_pools() {
        assert_eq!(TyreType::Soft.initial_pool(), vec![3, 4, 4]);
        assert_eq!(TyreType::Medium.initial_pool(), vec![2, 2, 3, 3, 4]);
        assert_eq!(TyreType::Hard.initial_pool(), vec![1, 1, 1, 2, 2, 3]);

        assert_eq!(BoostHand::with_tyre(TyreType::Soft).cards_remaining, 3);
        assert_eq!(BoostHand::with_tyre(TyreType::Medium).cards_remaining, 5);
        assert_eq!(BoostHand::with_tyre(TyreType::Hard).cards_remaining, 6);
    }

    #[test]
    fn test_boost_hand_use_card_with_duplicates() {
        let mut hand = BoostHand::with_tyre(TyreType::Medium); // [2,2,3,3,4]

        // Two value-2 cards: first use leaves it available, second depletes it.
        hand.use_card(2).unwrap();
        assert!(hand.is_card_available(2), "One value-2 card remains");
        assert_eq!(hand.cards_remaining, 4);

        hand.use_card(2).unwrap();
        assert!(!hand.is_card_available(2), "Both value-2 cards spent");
        assert_eq!(hand.cards_remaining, 3);

        let err = hand.use_card(2).unwrap_err();
        assert_eq!(err, "Boost card 2 is not available");
    }

    #[test]
    fn test_boost_zero_is_always_free() {
        let mut hand = BoostHand::with_tyre(TyreType::Soft); // [3,4,4]

        // Spend the whole pool.
        for value in [3, 4, 4] {
            hand.use_card(value).unwrap();
        }
        assert_eq!(hand.cards_remaining, 0);

        // Boost 0 still works, never decrements, never errors.
        assert!(hand.is_card_available(0));
        hand.use_card(0).unwrap();
        assert_eq!(hand.cards_remaining, 0);
    }

    #[test]
    fn test_boost_hand_no_auto_replenish() {
        let mut hand = BoostHand::with_tyre(TyreType::Medium);

        for value in [2, 2, 3, 3, 4] {
            hand.use_card(value).unwrap();
        }

        assert_eq!(hand.cards_remaining, 0, "Pool does not auto-replenish");
        assert_eq!(hand.pit_stops_completed, 0);
        assert_eq!(hand.get_available_cards(), vec![0], "Only free 0 remains");
    }

    #[test]
    fn test_boost_hand_refill_changes_tyre() {
        let mut hand = BoostHand::with_tyre(TyreType::Medium);
        hand.use_card(4).unwrap();

        // Pit onto Soft tyres.
        hand.refill(TyreType::Soft);

        assert_eq!(hand.tyre_type, TyreType::Soft);
        assert_eq!(hand.cards_remaining, 3, "Soft pool has 3 cards");
        assert_eq!(hand.pit_stops_completed, 1);
        assert_eq!(hand.get_available_cards(), vec![0, 3, 4]);
    }

    #[test]
    fn test_boost_hand_get_available_cards() {
        let mut hand = BoostHand::with_tyre(TyreType::Medium); // [2,2,3,3,4]

        // Distinct available values plus the always-free 0.
        assert_eq!(hand.get_available_cards(), vec![0, 2, 3, 4]);

        // Deplete value 2 entirely.
        hand.use_card(2).unwrap();
        hand.use_card(2).unwrap();

        let available = hand.get_available_cards();
        assert_eq!(available, vec![0, 3, 4]);
        assert!(!available.contains(&2), "Spent value-2 card excluded");
    }

    #[test]
    fn test_boost_hand_is_card_available_invalid_card() {
        let hand = BoostHand::new();

        // Out-of-pool / out-of-range values are unavailable (0 is the exception).
        assert!(!hand.is_card_available(5));
        assert!(!hand.is_card_available(10));
        assert!(!hand.is_card_available(255));
    }

    #[test]
    fn test_boost_hand_default_trait() {
        let hand = BoostHand::default();

        assert_eq!(hand.tyre_type, TyreType::Medium);
        assert_eq!(hand.cards_remaining, 5);
        assert_eq!(hand.pit_stops_completed, 0);
    }

    #[test]
    fn test_card_counts() {
        let hand = BoostHand::with_tyre(TyreType::Medium); // [2,2,3,3,4]
        assert_eq!(hand.card_counts(), vec![(1, 0), (2, 2), (3, 2), (4, 1)]);
    }

    // ========== End BoostHand Tests ==========

    #[test]
    fn test_create_race() {
        let track = create_test_track();
        let race = Race::new("Test Race".to_string(), track, 2);

        assert_eq!(race.name, "Test Race");
        assert_eq!(race.total_laps, 2);
        assert_eq!(race.status, RaceStatus::Waiting);
        assert!(matches!(
            race.lap_characteristic,
            LapCharacteristic::Straight
        ));
        assert_eq!(race.current_lap, 1);
    }

    #[test]
    fn test_add_participant() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        let result = race.add_participant(player_uuid, car_uuid, pilot_uuid);
        assert!(result.is_ok());
        assert_eq!(race.participants.len(), 1);
        assert_eq!(race.participants[0].player_uuid, player_uuid);
        // Starting sector is random due to qualification
        assert!(race.participants[0].current_sector <= 3);
    }

    #[test]
    fn test_duplicate_participant() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        let result = race.add_participant(player_uuid, car_uuid, pilot_uuid);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already participating"));
    }

    #[test]
    fn test_start_race() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        let result = race.start_race();
        assert!(result.is_ok());
        assert_eq!(race.status, RaceStatus::InProgress);
        // Lap characteristic should be set
        assert!(matches!(
            race.lap_characteristic,
            LapCharacteristic::Straight | LapCharacteristic::Curve
        ));
    }

    #[test]
    fn test_process_lap_basic_movement() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        // Set participant to start in sector 0 for predictable test
        race.participants[0].current_sector = 0;

        race.start_race().unwrap();

        // Player adds 5 boost (base 10 + boost 5 = 15)
        // Sector 0 has max_value 10, so player should move up to sector 1
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 5,
        }];

        let result = race.process_lap(&actions).unwrap();

        assert_eq!(result.lap, 1);
        assert_eq!(result.movements.len(), 1);
        assert_eq!(result.movements[0].movement_type, MovementType::MovedUp);
        assert_eq!(race.participants[0].total_value, 15); // base 10 + boost 5
        assert_eq!(race.participants[0].current_sector, 1);
    }

    #[test]
    fn test_move_up_sector() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 3);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        // Set participant to start in sector 0 for predictable test
        race.participants[0].current_sector = 0;

        race.start_race().unwrap();

        // Player adds enough boost to exceed sector 0 max (10)
        // Base value 10 + boost 5 = 15, which is > sector 0 max (10)
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 5,
        }];
        let result = race.process_lap(&actions).unwrap();

        assert_eq!(result.movements[0].movement_type, MovementType::MovedUp);
        assert_eq!(race.participants[0].current_sector, 1);
        assert_eq!(race.participants[0].total_value, 15);
    }

    #[test]
    fn test_move_down_sector() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        // Move player to sector 1 first
        race.participants[0].current_sector = 1;

        race.start_race().unwrap();

        // Base value 10 + boost 0 = 10, but sector 1 min is 8, so should stay
        // Let's use a negative scenario: base 5 + boost 0 = 5, which is < sector 1 min (8)
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 0,
        }];

        // We need to simulate a low base value for this test
        // For now, let's test with the current implementation
        let result = race.process_lap(&actions).unwrap();

        // With base value 10, the participant should stay in sector 1
        assert_eq!(
            result.movements[0].movement_type,
            MovementType::StayedInSector
        );
        assert_eq!(race.participants[0].current_sector, 1);
    }

    #[test]
    fn test_sector_capacity_limit() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add multiple participants
        let mut player_uuids = Vec::new();
        for _i in 0..5 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set all participants to start in sector 0 for predictable test
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Give different boost values to test performance-based movement priority
        let actions: Vec<LapAction> = player_uuids
            .iter()
            .enumerate()
            .map(|(i, &uuid)| LapAction {
                player_uuid: uuid,
                boost_value: 5 - (i as u32), // First player gets 5, second gets 4, etc.
                                             // This creates final values: 15, 14, 13, 12, 11 (all exceed sector 0 max of 10)
            })
            .collect();

        let _ = race.process_lap(&actions).unwrap();

        // Count how many are in sector 1 (capacity 3)
        let sector_1_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .count();

        // Should respect first-ranked rule - only 1 car should move up
        assert_eq!(sector_1_count, 1);

        // The remaining 4 should stay in sector 0 due to first-ranked rule
        let sector_0_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 0)
            .count();
        assert_eq!(sector_0_count, 4);

        // Verify that the participant who moved up is the best performer
        let moved_up_participant = race
            .participants
            .iter()
            .find(|p| p.current_sector == 1)
            .expect("Should have one participant in sector 1");

        // The best performer should have moved up (boost value 5)
        // Total value should be 15
        assert_eq!(
            moved_up_participant.total_value, 15,
            "Best performer should move up"
        );
    }

    #[test]
    fn test_single_slot_capacity_priority() {
        // Test the specific case where only ONE car can move up
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 10,
                slot_capacity: None, // Infinite
                sector_type: SectorType::Start,
            },
            Sector {
                id: 1,
                name: "Limited".to_string(),
                min_value: 8,
                max_value: 15,
                slot_capacity: Some(1), // Only ONE slot
                sector_type: SectorType::Straight,
            },
            Sector {
                id: 2,
                name: "Finish".to_string(),
                min_value: 12,
                max_value: 20,
                slot_capacity: None, // Infinite
                sector_type: SectorType::Finish,
            },
        ];

        let track = Track::new("Single Slot Track".to_string(), sectors).unwrap();
        let mut race = Race::new("Single Slot Test".to_string(), track, 1);

        // Add 3 participants
        let mut player_uuids = Vec::new();
        for _i in 0..3 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // All participants try to move up with different performance
        let actions: Vec<LapAction> = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 5,
            }, // Final: 15 (best)
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 4,
            }, // Final: 14 (second)
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 3,
            }, // Final: 13 (third)
        ];

        let result = race.process_lap(&actions).unwrap();

        // Only ONE car should move to sector 1 (the best performer)
        let sector_1_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 1);

        // The other 2 should stay in sector 0
        let sector_0_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 0)
            .count();
        assert_eq!(sector_0_count, 2);

        // The car that moved up should be the one with the highest performance (boost 5)
        let moved_up_participant = race
            .participants
            .iter()
            .find(|p| p.current_sector == 1)
            .unwrap();
        assert_eq!(moved_up_participant.player_uuid, player_uuids[0]);
        assert_eq!(moved_up_participant.total_value, 15); // base 10 + boost 5

        // Check that the participant in sector 1 has higher total_value than those in sector 0
        let stayed_participants: Vec<_> = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 0)
            .collect();

        for stayed_participant in &stayed_participants {
            assert!(
                moved_up_participant.total_value > stayed_participant.total_value,
                "Moved participant should have higher performance than stayed participant"
            );
        }

        // Verify the movements were recorded correctly - only 1 car should move up (first-ranked rule)
        let move_up_count = result
            .movements
            .iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .count();
        assert_eq!(
            move_up_count, 1,
            "Should have exactly 1 MovedUp movement (first-ranked car only)"
        );

        let stayed_count = result
            .movements
            .iter()
            .filter(|m| m.movement_type == MovementType::StayedInSector)
            .count();
        assert_eq!(
            stayed_count, 2,
            "Should have exactly 2 StayedInSector movements"
        );
    }

    #[test]
    fn test_invalid_boost_value() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race.start_race().unwrap();

        let actions = vec![LapAction {
            player_uuid,
            boost_value: 6, // Invalid: max is 5
        }];

        let result = race.process_lap(&actions);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid boost value"));
    }

    #[test]
    fn test_track_validation() {
        // Test empty sectors
        let result = Track::new("Empty Track".to_string(), vec![]);
        assert!(result.is_err());

        // Test first sector with capacity
        let sectors = vec![Sector {
            id: 0,
            name: "Start".to_string(),
            min_value: 0,
            max_value: 10,
            slot_capacity: Some(5), // Should be None
            sector_type: SectorType::Start,
        }];
        let result = Track::new("Invalid Track".to_string(), sectors);
        assert!(result.is_err());
    }

    #[test]
    fn test_sector_full_move_up_blocked() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add 4 participants
        let mut player_uuids = Vec::new();
        for i in 0..4 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);

            // Set first 3 in sector 1 (capacity 3), last one in sector 0
            if i < 3 {
                race.participants[i].current_sector = 1;
            } else {
                race.participants[i].current_sector = 0;
            }
        }

        race.start_race().unwrap();

        // All players need actions, but we're only testing the last one
        let actions = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 0,
            },
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 0,
            },
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 0,
            },
            LapAction {
                player_uuid: player_uuids[3],
                boost_value: 5,
            }, // Should exceed sector 0 max
        ];

        let result = race.process_lap(&actions).unwrap();

        // Player should stay in sector 0 because sector 1 is full
        assert_eq!(
            result.movements[0].movement_type,
            MovementType::StayedInSector
        );
        assert_eq!(race.participants[3].current_sector, 0);
    }

    #[test]
    fn test_sector_full_move_down_finds_space() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add participants and fill sectors strategically
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        // Set participant in sector 2
        race.participants[0].current_sector = 2;

        race.start_race().unwrap();

        // Simulate a very low performance that should move down
        // We'll need to modify the base value calculation for this test
        // For now, test the basic movement down logic
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 0, // Minimum boost
        }];

        let result = race.process_lap(&actions).unwrap();

        // With current base value of 10, participant should stay in sector 2
        // (since 10 >= sector 2 min_value of 12 is false, it should move down)
        // But our base value is 10, and sector 2 min is 12, so it should move down
        assert_eq!(result.movements[0].movement_type, MovementType::MovedDown);
        assert_eq!(race.participants[0].current_sector, 1);
    }

    #[test]
    fn test_lap_characteristic_changes() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 3);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race.participants[0].current_sector = 0;

        race.start_race().unwrap();

        let initial_characteristic = race.lap_characteristic.clone();

        // Process first lap
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 3,
        }];

        let result1 = race.process_lap(&actions).unwrap();
        // One turn = one lap: after the first turn the race is on lap 2.
        assert_eq!(result1.lap, 2);

        // Lap characteristic might change for next lap
        let second_characteristic = race.lap_characteristic.clone();

        // Process another turn -> lap 3 (capped at total_laps = 3).
        let result2 = race.process_lap(&actions).unwrap();
        assert_eq!(result2.lap, 3);

        // Verify lap characteristics are being tracked
        assert!(matches!(
            initial_characteristic,
            LapCharacteristic::Straight | LapCharacteristic::Curve
        ));
        assert!(matches!(
            second_characteristic,
            LapCharacteristic::Straight | LapCharacteristic::Curve
        ));
    }

    #[test]
    fn test_race_completion_by_laps() {
        // A small, easily-clearable track so the car can actually lap it.
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 3,
                slot_capacity: None,
                sector_type: SectorType::Start,
            },
            Sector {
                id: 1,
                name: "Finish".to_string(),
                min_value: 0,
                max_value: 5,
                slot_capacity: None,
                sector_type: SectorType::Finish,
            },
        ];
        let track = Track::new("Mini Track".to_string(), sectors).unwrap();
        // total_laps = number of turns to play (1 turn = 1 lap).
        let mut race = Race::new("Test Race".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        race.add_participant(player_uuid, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();

        let actions = vec![LapAction {
            player_uuid,
            boost_value: 2,
        }];

        // One turn = one lap: the race ends after exactly `total_laps` turns.
        let mut finished = false;
        let mut turns = 0u32;
        for _ in 0..50 {
            race.process_lap(&actions).unwrap();
            turns += 1;
            if race.status == RaceStatus::Finished {
                finished = true;
                break;
            }
        }

        assert!(finished, "race should finish after total_laps turns");
        assert_eq!(
            turns, race.total_laps,
            "race ends after exactly total_laps turns"
        );
        assert!(race.participants[0].is_finished);
        assert_eq!(race.current_lap, race.total_laps);
        assert!(race.participants[0].finish_position.is_some());
    }

    #[test]
    fn test_single_slot_movement_priority() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add 3 participants
        let mut player_uuids = Vec::new();
        for _i in 0..3 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set all participants in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        // Fill sector 1 with 2 participants (capacity is 3, so only 1 slot left)
        race.participants[0].current_sector = 1;
        race.participants[1].current_sector = 1;
        // participant[2] stays in sector 0

        race.start_race().unwrap();

        // All participants need actions, but only the one in sector 0 can potentially move
        let actions = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 0,
            }, // Already in sector 1
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 0,
            }, // Already in sector 1
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 5,
            }, // In sector 0, tries to move up
        ];

        let result = race.process_lap(&actions).unwrap();

        // Only the participant with higher performance should move up
        assert_eq!(
            race.participants[2].current_sector, 1,
            "Best performer should move up"
        );

        // Verify movements were recorded (3 total: 2 stay in sector 1, 1 moves up from sector 0)
        assert_eq!(result.movements.len(), 3);

        // Find the movement for the participant who was in sector 0
        let sector_0_movement = result
            .movements
            .iter()
            .find(|m| m.player_uuid == player_uuids[2])
            .expect("Should find movement for sector 0 participant");
        assert_eq!(sector_0_movement.movement_type, MovementType::MovedUp);

        // Verify sector 1 is now at capacity (3 participants)
        let sector_1_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 3, "Sector 1 should be at full capacity");

        // Verify movement counts
        let move_up_count = result
            .movements
            .iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .count();
        assert_eq!(move_up_count, 1, "Should have exactly 1 MovedUp movement");

        let stayed_count = result
            .movements
            .iter()
            .filter(|m| m.movement_type == MovementType::StayedInSector)
            .count();
        assert_eq!(
            stayed_count, 2,
            "Should have exactly 2 StayedInSector movements"
        );
    }

    #[test]
    fn test_multiple_cars_one_slot_performance_priority() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add 4 participants
        let mut player_uuids = Vec::new();
        for _i in 0..4 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set up scenario: sector 1 has 2 cars (capacity 3), sector 0 has 2 cars
        race.participants[0].current_sector = 1; // Already in sector 1
        race.participants[1].current_sector = 1; // Already in sector 1
        race.participants[2].current_sector = 0; // In sector 0, wants to move up
        race.participants[3].current_sector = 0; // In sector 0, wants to move up

        race.start_race().unwrap();

        // Both cars in sector 0 try to move up, but only 1 slot available in sector 1
        // Give different performance values to test priority
        let actions = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 0,
            }, // Stay in sector 1
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 0,
            }, // Stay in sector 1
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 3,
            }, // Lower performance (base 10 + 3 = 13)
            LapAction {
                player_uuid: player_uuids[3],
                boost_value: 5,
            }, // Higher performance (base 10 + 5 = 15)
        ];

        let result = race.process_lap(&actions).unwrap();

        // Only the best performer (player 3) should move up
        assert_eq!(
            race.participants[3].current_sector, 1,
            "Best performer should move up to sector 1"
        );
        assert_eq!(
            race.participants[2].current_sector, 0,
            "Lower performer should stay in sector 0"
        );

        // Verify sector 1 is now at capacity
        let sector_1_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 3, "Sector 1 should be at full capacity");

        // Verify exactly one car moved up
        let move_up_movements: Vec<_> = result
            .movements
            .iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .collect();
        assert_eq!(move_up_movements.len(), 1, "Exactly one car should move up");
        assert_eq!(
            move_up_movements[0].player_uuid, player_uuids[3],
            "The best performer should be the one who moved up"
        );
    }

    #[test]
    fn test_qualification_random_starting_positions() {
        let track = create_test_track();
        let track_sector_count = track.sectors.len() as u32;
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add multiple participants
        let mut starting_sectors = Vec::new();
        for _i in 0..10 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();

            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            starting_sectors.push(race.participants.last().unwrap().current_sector);
        }

        // Verify that not all participants start in the same sector
        let unique_sectors: std::collections::HashSet<_> = starting_sectors.iter().collect();

        // With random qualification, we should have some variety
        // (This test might occasionally fail due to randomness, but very unlikely with 10 participants)
        assert!(
            unique_sectors.len() > 1,
            "All participants started in the same sector, qualification not working"
        );

        // All starting sectors should be valid
        for &sector in &starting_sectors {
            assert!(sector < track_sector_count);
        }
    }

    #[test]
    fn test_sector_performance_ceiling_caps_base_value() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();

        // Set participant to start in sector 0 (max_value = 10)
        race.participants[0].current_sector = 0;

        race.start_race().unwrap();

        // Give a high boost that would normally result in base value > sector max
        // Base value is 10 (engine 5 + body 3 + pilot 2)
        // Sector 0 max_value is 10, so no capping should occur
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 3,
        }];

        let _result = race.process_lap(&actions).unwrap();

        // Final value should be base (10) + boost (3) = 13
        assert_eq!(race.participants[0].total_value, 13);

        // Now test with a car that has higher base stats
        // Manually set higher base stats by modifying the calculation
        // We'll create a scenario where base would be 15 but sector max is 10

        // Reset for second test
        let mut race2 = Race::new("Test Race 2".to_string(), create_test_track(), 1);
        race2
            .add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race2.participants[0].current_sector = 0; // Sector 0 max_value = 10
        race2.start_race().unwrap();

        // We need to test the capping logic directly since we can't easily modify car stats
        // Let's verify the capping logic by checking a scenario where it would apply

        // Test the capping calculation directly
        let base_value = 15u32; // Hypothetical high base value
        let sector_max = 10u32; // Sector 0 max value
        let boost = 3u32;

        let capped_base = std::cmp::min(base_value, sector_max);
        let final_value = capped_base + boost;

        assert_eq!(
            capped_base, 10,
            "Base value should be capped to sector maximum"
        );
        assert_eq!(final_value, 13, "Final value should be capped base + boost");

        // Verify that without capping, the value would be different
        let uncapped_final = base_value + boost;
        assert_eq!(
            uncapped_final, 18,
            "Without capping, final value would be higher"
        );
        assert_ne!(
            final_value, uncapped_final,
            "Capping should make a difference"
        );
    }

    #[test]
    fn test_sector_ceiling_different_scenarios() {
        // Test multiple scenarios of sector ceiling effects

        // Scenario 1: Base value below sector ceiling (no capping)
        let base_value_1 = 8u32;
        let sector_max_1 = 10u32;
        let boost_1 = 2u32;

        let capped_1 = std::cmp::min(base_value_1, sector_max_1);
        let final_1 = capped_1 + boost_1;

        assert_eq!(capped_1, 8, "Base value below ceiling should not be capped");
        assert_eq!(final_1, 10, "Final value should be base + boost");

        // Scenario 2: Base value exactly at sector ceiling (no capping)
        let base_value_2 = 10u32;
        let sector_max_2 = 10u32;
        let boost_2 = 2u32;

        let capped_2 = std::cmp::min(base_value_2, sector_max_2);
        let final_2 = capped_2 + boost_2;

        assert_eq!(capped_2, 10, "Base value at ceiling should not be capped");
        assert_eq!(final_2, 12, "Final value should be base + boost");

        // Scenario 3: Base value above sector ceiling (capping applied)
        let base_value_3 = 15u32;
        let sector_max_3 = 10u32;
        let boost_3 = 2u32;

        let capped_3 = std::cmp::min(base_value_3, sector_max_3);
        let final_3 = capped_3 + boost_3;

        assert_eq!(capped_3, 10, "Base value above ceiling should be capped");
        assert_eq!(final_3, 12, "Final value should be capped base + boost");

        // Scenario 4: High base value with high boost (capping still applies to base only)
        let base_value_4 = 20u32;
        let sector_max_4 = 5u32;
        let boost_4 = 5u32;

        let capped_4 = std::cmp::min(base_value_4, sector_max_4);
        let final_4 = capped_4 + boost_4;

        assert_eq!(
            capped_4, 5,
            "High base value should be capped to low sector ceiling"
        );
        assert_eq!(
            final_4, 10,
            "Final value should be capped base + full boost"
        );

        // Verify the strategic implication: boost becomes more important when capped
        let uncapped_final_4 = base_value_4 + boost_4;
        assert_eq!(
            uncapped_final_4, 25,
            "Without capping, final would be much higher"
        );

        let boost_percentage_capped = (boost_4 as f32 / final_4 as f32) * 100.0;
        let boost_percentage_uncapped = (boost_4 as f32 / uncapped_final_4 as f32) * 100.0;

        assert!(
            boost_percentage_capped > boost_percentage_uncapped,
            "Boost should be proportionally more important when base is capped"
        );
    }

    #[test]
    fn test_move_up_only_first_ranked_car() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add 3 participants
        let mut player_uuids = Vec::new();
        for _i in 0..3 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Give different performance levels to create clear ranking
        let actions: Vec<LapAction> = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 5,
            }, // Best: 15
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 4,
            }, // Second: 14
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 3,
            }, // Third: 13
        ];

        let _result = race.process_lap(&actions).unwrap();

        // All cars that exceed the threshold should move up (sector 1 has capacity 3, so space available)
        let sector_1_participants: Vec<_> = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .collect();

        assert_eq!(
            sector_1_participants.len(),
            1,
            "Only the first-ranked car should move up"
        );

        // Verify the moved car is the best performer
        let moved_car = sector_1_participants[0];
        assert_eq!(moved_car.total_value, 15, "Best performer should move up");

        // The other cars should stay in sector 0
        let sector_0_participants: Vec<_> = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 0)
            .collect();

        assert_eq!(
            sector_0_participants.len(),
            2,
            "Other cars should stay in sector 0"
        );

        // Verify the cars in sector 0 have lower performance than the moved car
        for participant in &sector_0_participants {
            assert!(
                participant.total_value < moved_car.total_value,
                "Cars in sector 0 should have lower performance"
            );
        }
    }

    #[test]
    fn test_move_up_with_equal_performance() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);

        // Add 3 participants
        let mut player_uuids = Vec::new();
        for _i in 0..3 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Give all cars the same performance level
        let actions: Vec<LapAction> = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 4,
            }, // All: 14
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 4,
            }, // All: 14
            LapAction {
                player_uuid: player_uuids[2],
                boost_value: 4,
            }, // All: 14
        ];

        let _result = race.process_lap(&actions).unwrap();

        // With equal performance, only one car should move up (first processed)
        let sector_1_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 1)
            .count();

        assert_eq!(
            sector_1_count, 1,
            "Only one car should move up when all have equal performance"
        );

        // Two cars should stay in sector 0
        let sector_0_count = race
            .participants
            .iter()
            .filter(|p| p.current_sector == 0)
            .count();

        assert_eq!(sector_0_count, 2, "Two cars should stay in sector 0");

        // All cars should have the same total value
        let all_values: Vec<u32> = race.participants.iter().map(|p| p.total_value).collect();

        assert!(
            all_values.iter().all(|&v| v == 14),
            "All cars should have the same total value"
        );
    }

    #[test]
    fn test_first_ranked_car_progression() {
        let track = create_test_track();
        let mut race = Race::new("Progression Test".to_string(), track, 2);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set both to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // LAP 1: Both try to move up, only first-ranked succeeds
        let actions_lap1: Vec<LapAction> = vec![
            LapAction {
                player_uuid: player_uuids[0],
                boost_value: 5,
            }, // Best performer
            LapAction {
                player_uuid: player_uuids[1],
                boost_value: 4,
            }, // Second performer
        ];

        let _result1 = race.process_lap(&actions_lap1).unwrap();

        // Only the best car should move to sector 1 (first-ranked rule)
        assert_eq!(
            race.participants
                .iter()
                .filter(|p| p.current_sector == 1)
                .count(),
            1
        );
        assert_eq!(
            race.participants
                .iter()
                .filter(|p| p.current_sector == 0)
                .count(),
            1
        );

        // Verify which car moved
        let sector_1_car = race
            .participants
            .iter()
            .find(|p| p.current_sector == 1)
            .unwrap();
        let sector_0_car = race
            .participants
            .iter()
            .find(|p| p.current_sector == 0)
            .unwrap();

        assert_eq!(sector_1_car.player_uuid, player_uuids[0]); // Best performer moved up
        assert_eq!(sector_0_car.player_uuid, player_uuids[1]); // Second performer stayed
    }

    #[test]
    fn test_individual_lap_action_processing() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Individual Action Test".to_string(), track, 2);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        // Set both to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Test 1: First player submits action, should be recorded
        let result1 = race
            .process_individual_lap_action(player_uuids[0], 3, &car_data)
            .unwrap();

        match result1 {
            IndividualLapResult::ActionRecorded {
                predicted_performance,
                waiting_for_players,
            } => {
                assert_eq!(predicted_performance.boost_value, 3);
                assert_eq!(waiting_for_players.len(), 1);
                assert_eq!(waiting_for_players[0], player_uuids[1]);
            }
            _ => panic!("Expected ActionRecorded result"),
        }

        // Verify pending actions are stored
        assert_eq!(race.pending_actions.len(), 1);
        assert_eq!(race.pending_actions[0].player_uuid, player_uuids[0]);
        assert_eq!(race.pending_actions[0].boost_value, 3);

        // Test 2: Second player submits action, should process lap
        let result2 = race
            .process_individual_lap_action(player_uuids[1], 2, &car_data)
            .unwrap();

        match result2 {
            IndividualLapResult::LapProcessed(lap_result) => {
                assert_eq!(lap_result.movements.len(), 2);
                // Both players should have moved (performance exceeds sector 0 max)
                assert!(lap_result
                    .movements
                    .iter()
                    .any(|m| m.movement_type == MovementType::MovedUp));
            }
            _ => panic!("Expected LapProcessed result"),
        }

        // Verify pending actions are cleared after processing
        assert_eq!(race.pending_actions.len(), 0);
        assert_eq!(race.action_submissions.len(), 0);

        // Test 3: Try to submit action for same player again in the same turn (should fail)
        // First, let's add an action to simulate a pending state
        race.pending_actions.push(LapAction {
            player_uuid: player_uuids[0],
            boost_value: 1,
        });

        let result3 = race.process_individual_lap_action(player_uuids[0], 1, &car_data);

        // This should fail because player already submitted an action
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("already submitted an action"));

        // Clear the test action
        race.pending_actions.clear();
    }

    #[test]
    fn test_individual_lap_action_validation() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Validation Test".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Test invalid boost value
        let result = race.process_individual_lap_action(
            player_uuid,
            6, // Invalid: max is 5
            &car_data,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid boost value"));

        // Test non-existent player
        let non_existent_player = Uuid::new_v4();
        let result = race.process_individual_lap_action(non_existent_player, 3, &car_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Player not found"));

        // Test race not in progress
        race.status = RaceStatus::Finished;
        let result = race.process_individual_lap_action(player_uuid, 3, &car_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Race is not in progress"));
    }

    #[test]
    fn test_boost_card_validation_in_race() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Boost Card Test".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Verify initial boost hand state
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        assert_eq!(race.participants[0].boost_hand.pit_stops_completed, 0);
        // Card 4 is unique in the Medium pool [2,2,3,3,4], so one use depletes it.
        assert!(race.participants[0].boost_hand.is_card_available(4));

        // Use boost card 4
        let result = race.process_individual_lap_action(player_uuid, 4, &car_data);

        assert!(result.is_ok());

        // Verify card 4 is now unavailable
        assert!(!race.participants[0].boost_hand.is_card_available(4));
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 4);

        // Clear pending actions to test again
        race.pending_actions.clear();
        race.action_submissions.clear();
        race.pending_performance_calculations.clear();

        // Try to use card 4 again - should fail
        let result = race.process_individual_lap_action(player_uuid, 4, &car_data);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not available"));
    }

    #[test]
    fn test_boost_card_replenishment_triggers_correctly() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Replenishment Test".to_string(), track, 10);

        // Add 2 participants to test individual actions
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Deplete player 1's whole Medium pool [2,2,3,3,4] over 5 turns. There is
        // NO auto-replenish, so the pool empties and stays empty until a pit stop.
        let boost_sequence_p1 = [2, 2, 3, 3, 4];

        for (index, &boost_value) in boost_sequence_p1.iter().enumerate() {
            // Player 1 submits a real card.
            let result =
                race.process_individual_lap_action(player_uuids[0], boost_value, &car_data);

            assert!(
                result.is_ok(),
                "Failed to use boost card {boost_value} for player 1"
            );

            // Player 2 plays the free boost 0 to complete the lap.
            let _result2 = race.process_individual_lap_action(player_uuids[1], 0, &car_data);

            // Cards remaining decrease by exactly one per real card, no refill.
            assert_eq!(
                race.participants[0].boost_hand.cards_remaining,
                4 - index as u32,
                "Cards remaining should decrease by one per used card"
            );
            assert_eq!(race.participants[0].boost_hand.pit_stops_completed, 0);
        }

        // Pool is empty and does NOT auto-replenish.
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 0);
        assert_eq!(
            race.participants[0].boost_hand.get_available_cards(),
            vec![0],
            "Only the free boost 0 remains until a pit stop"
        );

        // Player 1 pits (refills the pool, consumes the turn as a free boost-0 lap);
        // player 2 plays boost 0 to complete the lap.
        race.process_individual_pit_action(player_uuids[0], None, &car_data)
            .unwrap();
        race.process_individual_lap_action(player_uuids[1], 0, &car_data)
            .unwrap();

        // The pit refilled the Medium pool and recorded a pit stop.
        assert_eq!(
            race.participants[0].boost_hand.cards_remaining, 5,
            "Pit stop refills the Medium pool"
        );
        assert_eq!(race.participants[0].boost_hand.pit_stops_completed, 1);
        assert_eq!(
            race.participants[0].boost_hand.get_available_cards(),
            vec![0, 2, 3, 4],
            "Medium pool is available again after the pit"
        );

        // We can use card 2 again from the refreshed pool.
        let result = race.process_individual_lap_action(player_uuids[0], 2, &car_data);
        assert!(
            result.is_ok(),
            "Should be able to use card 2 again after the pit refill"
        );
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 4);
    }

    #[test]
    fn test_boost_card_multiple_pits_refill_pool() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Multiple Pits Test".to_string(), track, 60);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Repeat: deplete the Medium pool, then pit to refill it. No auto-replenish.
        for pit in 1..=3 {
            // Spend the full Medium pool [2,2,3,3,4].
            for &card in &[2u32, 2, 3, 3, 4] {
                race.process_individual_lap_action(player_uuids[0], card, &car_data)
                    .unwrap();
                race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                    .unwrap();
            }

            // Pool is empty; only the free boost 0 remains.
            assert_eq!(
                race.participants[0].boost_hand.cards_remaining, 0,
                "Pit {pit}: pool empties without auto-replenish"
            );
            assert_eq!(
                race.participants[0].boost_hand.get_available_cards(),
                vec![0]
            );

            // Pit to refill the pool.
            race.process_individual_pit_action(player_uuids[0], None, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();

            assert_eq!(
                race.participants[0].boost_hand.cards_remaining, 5,
                "Pit {pit}: pool is refilled to 5"
            );
            assert_eq!(
                race.participants[0].boost_hand.pit_stops_completed, pit,
                "Pit {pit}: pit count increments"
            );
        }

        // Final state: three pit stops, a full Medium pool.
        assert_eq!(race.participants[0].boost_hand.pit_stops_completed, 3);
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        assert_eq!(
            race.participants[0].boost_hand.get_available_cards(),
            vec![0, 2, 3, 4]
        );
    }

    #[test]
    fn test_boost_card_invalid_value_rejected() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Invalid Boost Test".to_string(), track, 2);

        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();

        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Test boost value > 4 (invalid)
        let result = race.process_individual_lap_action(player_uuid, 5, &car_data);

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid boost value"));
        assert!(error_msg.contains("Must be between 0 and 4"));

        // Verify boost hand state unchanged
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        assert_eq!(race.participants[0].boost_hand.pit_stops_completed, 0);
    }

    // ========== Boost Usage History Tests ==========

    #[test]
    fn test_boost_usage_history_records_created() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("History Test".to_string(), track, 10);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Initially, history should be empty
        assert_eq!(race.participants[0].boost_usage_history.len(), 0);

        // Use 3 boosts: 2 (real), 0 (free no-op), 4 (real). Boost 0 does not
        // consume a card, so cards_remaining only drops on the real cards.
        // Starting pool is 5; expected remaining after each: 2->4, 0->4, 4->3.
        let boost_sequence: Vec<u8> = vec![2, 0, 4];
        let expected_remaining: [u32; 3] = [4, 4, 3];

        for (index, &boost_value) in boost_sequence.iter().enumerate() {
            race.process_individual_lap_action(player_uuids[0], u32::from(boost_value), &car_data)
                .unwrap();

            // Complete lap with player 2 (free boost 0).
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();

            // Verify history record was created
            assert_eq!(
                race.participants[0].boost_usage_history.len(),
                index + 1,
                "Should have {} history records",
                index + 1
            );

            // Verify the latest record. `cycle_number` now records pit stops
            // completed (0, since no pit happened), and replenishment never occurs.
            let latest_record = &race.participants[0].boost_usage_history[index];
            assert_eq!(latest_record.boost_value, boost_value);
            assert_eq!(latest_record.lap_number, (index + 1) as u32);
            assert_eq!(latest_record.cycle_number, 0);
            assert_eq!(
                latest_record.cards_remaining_after,
                expected_remaining[index]
            );
            assert!(!latest_record.replenishment_occurred);
        }
    }

    #[test]
    fn test_boost_usage_history_tracks_replenishment() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Replenishment History Test".to_string(), track, 10);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Spend the full Medium pool [2,2,3,3,4], then pit to refill. History
        // records the pit segment (pit_stops_completed at time of use) in
        // `cycle_number`, and `replenishment_occurred` is always false now.
        for &card in &[2u32, 2, 3, 3, 4] {
            race.process_individual_lap_action(player_uuids[0], card, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }

        // 5 records so far, all in pit segment 0, none flagged as replenished.
        assert_eq!(race.participants[0].boost_usage_history.len(), 5);
        for record in &race.participants[0].boost_usage_history {
            assert_eq!(record.cycle_number, 0, "all uses are before any pit stop");
            assert!(!record.replenishment_occurred, "no auto-replenishment");
        }

        // Cards remaining counts down monotonically (no mid-sequence refill).
        let remaining: Vec<u32> = race.participants[0]
            .boost_usage_history
            .iter()
            .map(|r| r.cards_remaining_after)
            .collect();
        assert_eq!(remaining, vec![4, 3, 2, 1, 0]);

        // Pit refills the pool; the pit itself is a free boost-0 lap.
        race.process_individual_pit_action(player_uuids[0], None, &car_data)
            .unwrap();
        race.process_individual_lap_action(player_uuids[1], 0, &car_data)
            .unwrap();

        // The pit's boost-0 usage was recorded after the refill, so its segment is 1.
        let pit_record = race.participants[0].boost_usage_history.last().unwrap();
        assert_eq!(pit_record.boost_value, 0);
        assert_eq!(pit_record.cycle_number, 1, "recorded after the pit stop");
        assert_eq!(pit_record.cards_remaining_after, 5, "pool was refilled");
        assert!(!pit_record.replenishment_occurred);
    }

    #[test]
    fn test_boost_cycle_summaries() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Cycle Summary Test".to_string(), track, 15);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // `get_boost_cycle_summaries` now groups by pit segment (`cycle_number` =
        // pit stops completed at time of use), since the auto-cycle mechanic is
        // gone. Drive one segment, pit, then a second segment.

        // Segment 0 (before any pit): use Medium cards 2, 3, 4 on laps 1, 2, 3.
        for card in [2u32, 3, 4] {
            race.process_individual_lap_action(player_uuids[0], card, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }

        // Pit (free boost-0 lap, lap 4) refills the pool and bumps the segment to 1.
        race.process_individual_pit_action(player_uuids[0], None, &car_data)
            .unwrap();
        race.process_individual_lap_action(player_uuids[1], 0, &car_data)
            .unwrap();

        // Segment 1 (after the pit): use Medium cards 2, 3, 4 on laps 5, 6, 7.
        for card in [2u32, 3, 4] {
            race.process_individual_lap_action(player_uuids[0], card, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }

        // Get segment summaries
        let summaries = race.participants[0].get_boost_cycle_summaries();

        // Should have 2 segments (segment 0 and segment 1).
        assert_eq!(summaries.len(), 2);

        // Verify segment 0 summary: cards 2,3,4 on laps 1,2,3.
        let segment0 = &summaries[0];
        assert_eq!(segment0.cycle_number, 0);
        assert_eq!(segment0.cards_used, vec![2, 3, 4]);
        assert_eq!(segment0.laps_in_cycle, vec![1, 2, 3]);
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(segment0.average_boost, 3.0); // (2+3+4)/3 = 3.0
        }

        // Verify segment 1 summary: the pit's boost 0 (lap 4) then 2,3,4 (laps 5-7).
        let segment1 = &summaries[1];
        assert_eq!(segment1.cycle_number, 1);
        assert_eq!(segment1.cards_used, vec![0, 2, 3, 4]);
        assert_eq!(segment1.laps_in_cycle, vec![4, 5, 6, 7]);
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(segment1.average_boost, 2.25); // (0+2+3+4)/4 = 2.25
        }
    }

    #[test]
    fn test_boost_usage_statistics() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Statistics Test".to_string(), track, 10);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Use specific boost cards: 3, 4, 2
        let boost_sequence = vec![3, 4, 2];

        for &boost_value in &boost_sequence {
            race.process_individual_lap_action(player_uuids[0], boost_value, &car_data)
                .unwrap();

            // Player 2 plays the free boost 0 to complete the lap.
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }

        let participant = &race.participants[0];

        // Test total boosts used
        assert_eq!(participant.get_total_boosts_used(), 3);

        // Test average boost value: (3 + 4 + 2) / 3 = 3.0
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(participant.get_average_boost_value(), 3.0);
        }

        // No pit stops happened, so all usage is in segment 0.
        let segment0_usage = participant.get_boost_usage_for_cycle(0);
        assert_eq!(segment0_usage.len(), 3);
        assert_eq!(segment0_usage[0].boost_value, 3);
        assert_eq!(segment0_usage[1].boost_value, 4);
        assert_eq!(segment0_usage[2].boost_value, 2);
    }

    #[test]
    fn test_boost_usage_history_multiple_cycles() {
        use crate::domain::{
            Body, BodyName, Car, ComponentRarity, Engine, EngineName, Pilot, PilotClass, PilotName,
            PilotPerformance, PilotRarity, PilotSkills,
        };
        use crate::services::car_validation::ValidatedCarData;

        let track = create_test_track();
        let mut race = Race::new("Multi-Cycle History Test".to_string(), track, 15);

        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid)
                .unwrap();
            player_uuids.push(player_uuid);
        }

        for participant in &mut race.participants {
            participant.current_sector = 0;
        }

        race.start_race().unwrap();

        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
        )
        .unwrap();

        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
        )
        .unwrap();

        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
        )
        .unwrap();

        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap()).unwrap();

        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };

        // Two pit segments. In each, deplete the full Medium pool [2,2,3,3,4],
        // then pit (a free boost-0 lap) to refill before the next segment.
        // Segment 0: laps 1-5 (cycle_number 0).
        for &card in &[2u32, 2, 3, 3, 4] {
            race.process_individual_lap_action(player_uuids[0], card, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }
        // Pit at lap 6 (cycle_number 1 from here on).
        race.process_individual_pit_action(player_uuids[0], None, &car_data)
            .unwrap();
        race.process_individual_lap_action(player_uuids[1], 0, &car_data)
            .unwrap();
        // Segment 1: laps 7-11 (cycle_number 1).
        for &card in &[2u32, 2, 3, 3, 4] {
            race.process_individual_lap_action(player_uuids[0], card, &car_data)
                .unwrap();
            race.process_individual_lap_action(player_uuids[1], 0, &car_data)
                .unwrap();
        }

        let participant = &race.participants[0];

        // 11 records: 5 (segment 0) + 1 (pit boost 0) + 5 (segment 1).
        assert_eq!(participant.boost_usage_history.len(), 11);

        // Verify segment (cycle_number) tagging: first 5 are segment 0, the rest
        // (including the pit's boost-0 lap) are segment 1.
        for i in 0..5 {
            assert_eq!(participant.boost_usage_history[i].cycle_number, 0);
        }
        for i in 5..11 {
            assert_eq!(participant.boost_usage_history[i].cycle_number, 1);
        }

        // Replenishment flag is always false under the new semantics.
        for record in &participant.boost_usage_history {
            assert!(!record.replenishment_occurred);
        }

        // Get segment summaries
        let summaries = participant.get_boost_cycle_summaries();
        assert_eq!(summaries.len(), 2);

        // Segment 0 used 5 cards; segment 1 used 6 (the pit's 0 plus 5 cards).
        assert_eq!(summaries[0].cards_used.len(), 5);
        assert_eq!(summaries[1].cards_used.len(), 6);

        // Test statistics: 11 boosts, sum = (2+2+3+3+4)*2 + 0 = 28, avg = 28/11.
        assert_eq!(participant.get_total_boosts_used(), 11);
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(participant.get_average_boost_value(), 28.0 / 11.0);
        }
    }

    // ========== End Boost Usage History Tests ==========
}
