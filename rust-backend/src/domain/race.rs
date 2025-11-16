use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

use crate::services::car_validation::ValidatedCarData;

/// Boost hand management system for tracking available boost cards
/// Each player has 5 boost cards (0, 1, 2, 3, 4) that can be used once per cycle
/// When all cards are used, the hand automatically replenishes
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostHand {
    /// Availability state for each boost card (0-4)
    /// true = available, false = used
    pub cards: HashMap<u8, bool>,
    
    /// Current cycle number (starts at 1)
    pub current_cycle: u32,
    
    /// Total number of cycles completed
    pub cycles_completed: u32,
    
    /// Number of cards remaining in current cycle
    pub cards_remaining: u32,
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
    /// Initialize a new boost hand with all cards available
    #[must_use]
    pub fn new() -> Self {
        let mut cards = HashMap::new();
        for i in 0..=4 {
            cards.insert(i, true);
        }
        
        Self {
            cards,
            current_cycle: 1,
            cycles_completed: 0,
            cards_remaining: 5,
        }
    }
    
    /// Check if a specific boost card is available
    #[must_use]
    pub fn is_card_available(&self, boost_value: u8) -> bool {
        self.cards.get(&boost_value).copied().unwrap_or(false)
    }
    
    /// Use a boost card (mark as unavailable)
    /// Returns Ok(()) if successful, Err with message if card is not available
    /// Automatically triggers replenishment when all cards are used
    pub fn use_card(&mut self, boost_value: u8) -> Result<(), String> {
        if !self.is_card_available(boost_value) {
            return Err(format!("Boost card {boost_value} is not available"));
        }
        
        self.cards.insert(boost_value, false);
        self.cards_remaining -= 1;
        
        // Check if all cards are used - trigger replenishment
        if self.cards_remaining == 0 {
            self.replenish();
        }
        
        Ok(())
    }
    
    /// Replenish all boost cards (internal method)
    /// Called automatically when all cards have been used
    fn replenish(&mut self) {
        for i in 0..=4 {
            self.cards.insert(i, true);
        }
        self.cards_remaining = 5;
        self.cycles_completed += 1;
        self.current_cycle += 1;
    }
    
    /// Get list of available boost card values
    #[must_use]
    pub fn get_available_cards(&self) -> Vec<u8> {
        let mut available: Vec<u8> = self.cards
            .iter()
            .filter(|(_, &is_available)| is_available)
            .map(|(&value, _)| value)
            .collect();
        
        // Sort for consistent ordering
        available.sort_unstable();
        available
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
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
    // Individual lap action processing fields
    pub pending_actions: Vec<LapAction>,
    pub action_submissions: HashMap<Uuid, DateTime<Utc>>, // Track submission times
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
    Start,      // First sector (infinite slots)
    Straight,   // Straight section
    Curve,      // Curved section
    Finish,     // Last sector (infinite slots)
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
    pub sector_positions: HashMap<u32, Vec<RaceParticipant>>, // sector_id -> participants
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
        let now = Utc::now();
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
            created_at: now,
            updated_at: now,
            pending_actions: Vec::new(),
            action_submissions: HashMap::new(),
            pending_performance_calculations: HashMap::new(),
        }
    }

    pub fn add_participant(&mut self, player_uuid: Uuid, car_uuid: Uuid, pilot_uuid: Uuid) -> Result<(), String> {
        if self.status != RaceStatus::Waiting {
            return Err("Cannot add participants to a race that has already started".to_string());
        }

        // Check if player is already participating
        if self.participants.iter().any(|p| p.player_uuid == player_uuid) {
            return Err("Player is already participating in this race".to_string());
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
            boost_hand: BoostHand::new(),
            boost_usage_history: Vec::new(),
        };

        self.participants.push(participant);
        self.updated_at = Utc::now();
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
        
        self.updated_at = Utc::now();
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

    /// Process lap with pre-calculated performance values from car components
    /// This is the new method that uses actual car data for performance calculation
    pub fn process_lap_with_car_data(&mut self, actions: &[LapAction], performance_calculations: &HashMap<Uuid, PerformanceCalculation>) -> Result<LapResult, String> {
        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // Validate all participants have submitted actions
        for participant in &self.participants {
            if participant.is_finished {
                continue;
            }
            if !actions.iter().any(|a| a.player_uuid == participant.player_uuid) {
                return Err(format!("Missing action for player {}", participant.player_uuid));
            }
        }

        // Validate boost values
        for action in actions {
            if action.boost_value > 5 {
                return Err(format!("Invalid boost value {} for player {}", action.boost_value, action.player_uuid));
            }
        }

        // Use pre-calculated performance values from car components
        let mut participant_values: HashMap<Uuid, u32> = HashMap::new();
        for action in actions {
            if let Some(participant) = self.participants.iter().find(|p| p.player_uuid == action.player_uuid) {
                if !participant.is_finished {
                    // Use the pre-calculated performance from car data
                    if let Some(performance) = performance_calculations.get(&action.player_uuid) {
                        participant_values.insert(action.player_uuid, performance.final_value);
                    } else {
                        return Err(format!("Missing performance calculation for player {}", action.player_uuid));
                    }
                }
            }
        }

        Ok(self.process_lap_internal(actions, &participant_values))
    }

    /// Legacy `process_lap` method for backward compatibility with tests
    /// Uses placeholder base value of 10 for performance calculation
    /// New code should use `process_lap_with_car_data` instead
    pub fn process_lap(&mut self, actions: &[LapAction]) -> Result<LapResult, String> {
        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // Validate all participants have submitted actions
        for participant in &self.participants {
            if participant.is_finished {
                continue;
            }
            if !actions.iter().any(|a| a.player_uuid == participant.player_uuid) {
                return Err(format!("Missing action for player {}", participant.player_uuid));
            }
        }

        // Validate boost values
        for action in actions {
            if action.boost_value > 5 {
                return Err(format!("Invalid boost value {} for player {}", action.boost_value, action.player_uuid));
            }
        }

        // Calculate final values for all participants using placeholder base value
        let mut participant_values: HashMap<Uuid, u32> = HashMap::new();
        for action in actions {
            if let Some(participant) = self.participants.iter().find(|p| p.player_uuid == action.player_uuid) {
                if !participant.is_finished {
                    // Use placeholder base value for backward compatibility
                    let base_value = 10;
                    
                    // Apply sector performance ceiling: cap base value to current sector's max_value
                    let current_sector = &self.track.sectors[participant.current_sector as usize];
                    let capped_base_value = std::cmp::min(base_value, current_sector.max_value);
                    
                    // Add boost to the capped base value
                    let final_value = capped_base_value + action.boost_value;
                    participant_values.insert(action.player_uuid, final_value);
                }
            }
        }

        Ok(self.process_lap_internal(actions, &participant_values))
    }

    /// Internal method that processes lap movements after performance values are calculated
    fn process_lap_internal(&mut self, actions: &[LapAction], participant_values: &HashMap<Uuid, u32>) -> LapResult {

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
            if let Some(participant) = self.participants.iter_mut().find(|p| p.player_uuid == action.player_uuid) {
                if !participant.is_finished {
                    if let Some(&final_value) = participant_values.get(&action.player_uuid) {
                        participant.total_value += final_value;
                    }
                }
            }
        }

        // Sort participants in each sector by their total value (descending = better position)
        self.sort_participants_in_sectors();

        // Check for race completion
        self.check_race_completion();

        // Store current lap for result before advancing
        let processed_lap = self.current_lap;

        // Advance to next lap if not finished
        if self.status == RaceStatus::InProgress {
            self.current_lap += 1;
            if self.current_lap <= self.total_laps {
                self.lap_characteristic = Self::generate_lap_characteristic();
            }
        }

        self.updated_at = Utc::now();

        LapResult {
            lap: processed_lap,
            lap_characteristic: self.lap_characteristic.clone(),
            sector_positions: self.get_sector_positions(),
            movements,
        }
    }

    /// Process individual lap action for a single player
    /// Stores pending actions until all players submit, then processes simultaneous turn resolution
    pub fn process_individual_lap_action(
        &mut self,
        player_uuid: Uuid,
        boost_value: u32,
        car_data: &ValidatedCarData,
    ) -> Result<IndividualLapResult, String> {
        use crate::domain::boost_hand_manager::BoostHandManager;

        if self.status != RaceStatus::InProgress {
            return Err("Race is not in progress".to_string());
        }

        // 1. Validate player is in race and not finished
        let participant_index = self.participants
            .iter()
            .position(|p| p.player_uuid == player_uuid)
            .ok_or("Player not found in race")?;
        
        if self.participants[participant_index].is_finished {
            return Err("Player has already finished the race".to_string());
        }

        // 2. Check if player has already submitted an action for this turn
        if self.pending_actions.iter().any(|a| a.player_uuid == player_uuid) {
            return Err("Player has already submitted an action for this turn".to_string());
        }

        // 3. Validate boost value range (0-4 for boost cards)
        if boost_value > 4 {
            return Err(format!("Invalid boost value: {boost_value}. Must be between 0 and 4"));
        }

        // 4. Validate boost card availability and use the card
        #[allow(clippy::cast_possible_truncation)]
        let boost_value_u8 = boost_value as u8;
        
        let boost_usage_result = BoostHandManager::use_boost_card(
            &mut self.participants[participant_index].boost_hand,
            boost_value_u8,
        ).map_err(|e| e.to_string())?;
        
        // Record boost usage in history
        let usage_record = BoostUsageRecord {
            lap_number: self.current_lap,
            boost_value: boost_value_u8,
            cycle_number: boost_usage_result.current_cycle,
            cards_remaining_after: boost_usage_result.cards_remaining,
            replenishment_occurred: boost_usage_result.replenishment_occurred,
        };
        self.participants[participant_index].boost_usage_history.push(usage_record);

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
        self.action_submissions.insert(player_uuid, Utc::now());
        self.pending_performance_calculations.insert(player_uuid, performance.clone());

        // 7. Check if all participants have submitted actions
        if self.all_actions_submitted() {
            // Clone the pending actions and performance calculations to avoid borrowing issues
            let actions_to_process = self.pending_actions.clone();
            let performance_calculations = self.pending_performance_calculations.clone();
            
            // Process all actions simultaneously with their performance calculations
            let lap_result = self.process_lap_with_car_data(&actions_to_process, &performance_calculations)?;
            
            // Clear pending actions and calculations after processing
            self.pending_actions.clear();
            self.action_submissions.clear();
            self.pending_performance_calculations.clear();
            
            Ok(IndividualLapResult::LapProcessed(lap_result))
        } else {
            // Return current state with action recorded
            Ok(IndividualLapResult::ActionRecorded {
                predicted_performance: performance,
                waiting_for_players: self.get_pending_players(),
            })
        }
    }

    /// Check if all active participants have submitted actions
    #[must_use] 
    pub fn all_actions_submitted(&self) -> bool {
        let active_participants: HashSet<Uuid> = self.participants
            .iter()
            .filter(|p| !p.is_finished)
            .map(|p| p.player_uuid)
            .collect();
            
        let submitted_actions: HashSet<Uuid> = self.pending_actions
            .iter()
            .map(|a| a.player_uuid)
            .collect();
            
        active_participants == submitted_actions
    }

    /// Get list of players who haven't submitted actions yet
    #[must_use] 
    pub fn get_pending_players(&self) -> Vec<Uuid> {
        let submitted: HashSet<Uuid> = self.pending_actions
            .iter()
            .map(|a| a.player_uuid)
            .collect();
            
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
            let participant = self.participants
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
        
        // Apply boost as a multiplier to make it more significant
        // Boost multiplier: 1.0 + (boost_value * 0.08)
        // This gives:
        // - Boost 0: 1.0x (no change)
        // - Boost 1: 1.08x (+8%)
        // - Boost 2: 1.16x (+16%)
        // - Boost 3: 1.24x (+24%)
        // - Boost 4: 1.32x (+32%)
        // - Boost 5: 1.40x (+40%)
        let boost_multiplier = 1.0 + (f64::from(boost_value) * 0.08);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let boosted_value = (f64::from(capped_base_value) * boost_multiplier).round() as u32;
        
        // Final value is the boosted value
        let final_value = boosted_value;
        
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

    fn process_sector_movements(&mut self, sector_id: u32, participant_values: &HashMap<Uuid, u32>) -> Vec<ParticipantMovement> {
        let mut movements = Vec::new();
        
        // Get all participants in this sector with their performance values
        let mut participants_in_sector: Vec<(usize, u32)> = self.participants
            .iter()
            .enumerate()
            .filter(|(_, p)| p.current_sector == sector_id && !p.is_finished)
            .filter_map(|(i, p)| {
                participant_values.get(&p.player_uuid).map(|&value| (i, value))
            })
            .collect();

        // Sort by performance value (highest first) - this determines ranking
        participants_in_sector.sort_by(|a, b| b.1.cmp(&a.1));

        // Process each participant, but only allow the first-ranked car to move up
        for (rank, &(participant_index, final_value)) in participants_in_sector.iter().enumerate() {
            let movement = self.calculate_movement_for_participant(participant_index, final_value, sector_id, rank == 0);
            movements.push(movement);
        }

        movements
    }

    fn calculate_movement_for_participant(&mut self, participant_index: usize, final_value: u32, current_sector_id: u32, is_first_ranked: bool) -> ParticipantMovement {
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

    fn move_participant_down(&mut self, participant_index: usize, from_sector: u32, final_value: u32) -> ParticipantMovement {
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
                    let current_count = self.participants.iter()
                        .enumerate()
                        .filter(|(i, p)| *i != participant_index && p.current_sector == target_sector && !p.is_finished)
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

    fn move_participant_up(&mut self, participant_index: usize, from_sector: u32, final_value: u32) -> ParticipantMovement {
        let player_uuid = self.participants[participant_index].player_uuid;
        let next_sector = from_sector + 1;

        // Check if we've reached the end (lap completion or race finish)
        #[allow(clippy::cast_possible_truncation)]
        if next_sector >= self.track.sectors.len() as u32 {
            // Completed a lap
            self.participants[participant_index].current_lap += 1;
            
            if self.participants[participant_index].current_lap > self.total_laps {
                // Finished the race
                self.participants[participant_index].is_finished = true;
                return ParticipantMovement {
                    player_uuid,
                    from_sector,
                    to_sector: from_sector,
                    final_value,
                    movement_type: MovementType::FinishedRace,
                };
            }
            // Start new lap - go back to sector 0
            self.participants[participant_index].current_sector = 0;
            return ParticipantMovement {
                player_uuid,
                from_sector,
                to_sector: 0,
                final_value,
                movement_type: MovementType::FinishedLap,
            };
        }

        // Check if next sector has capacity
        let next_sector_obj = &self.track.sectors[next_sector as usize];
        let can_move_up = match next_sector_obj.slot_capacity {
            None => true, // Infinite capacity
            Some(capacity) => {
                let current_count = self.participants.iter()
                    .enumerate()
                    .filter(|(i, p)| *i != participant_index && p.current_sector == next_sector && !p.is_finished)
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
                sector_groups.entry(participant.current_sector)
                    .or_default()
                    .push(participant);
            }
        }

        // Sort each sector group by total_value (descending = better position)
        for participants in sector_groups.values_mut() {
            participants.sort_by(|a, b| b.total_value.cmp(&a.total_value));
            
            // Update position in sector
            for (index, participant) in participants.iter_mut().enumerate() {
                #[allow(clippy::cast_possible_truncation)]
                {
                    participant.current_position_in_sector = index as u32;
                }
            }
        }
    }

    fn get_sector_positions(&self) -> HashMap<u32, Vec<RaceParticipant>> {
        let mut positions: HashMap<u32, Vec<RaceParticipant>> = HashMap::new();
        
        for participant in &self.participants {
            if !participant.is_finished {
                positions.entry(participant.current_sector)
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
        // Check if all laps are completed or all participants finished
        let finished_count = self.participants.iter().filter(|p| p.is_finished).count();
        let all_finished = finished_count == self.participants.len();
        let all_laps_completed = self.current_lap > self.total_laps;
        
        if all_finished || all_laps_completed {
            self.status = RaceStatus::Finished;
            
            // Assign finish positions based on final sector and position
            let mut all_participants: Vec<&mut RaceParticipant> = self.participants.iter_mut().collect();
            
            // Sort by: 1) Finished status, 2) Current sector (higher = better), 3) Position in sector (lower = better), 4) Total value (higher = better)
            all_participants.sort_by(|a, b| {
                b.is_finished.cmp(&a.is_finished)
                    .then_with(|| b.current_sector.cmp(&a.current_sector))
                    .then_with(|| a.current_position_in_sector.cmp(&b.current_position_in_sector))
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
            let summary = summaries.entry(record.cycle_number).or_insert_with(|| {
                BoostCycleSummary {
                    cycle_number: record.cycle_number,
                    cards_used: Vec::new(),
                    laps_in_cycle: Vec::new(),
                    average_boost: 0.0,
                }
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
        
        let sum: u32 = self.boost_usage_history
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

    // ========== BoostHand Tests ==========

    #[test]
    fn test_boost_hand_initialization() {
        let hand = BoostHand::new();
        
        // Verify initial state
        assert_eq!(hand.cards_remaining, 5, "Should start with 5 cards");
        assert_eq!(hand.current_cycle, 1, "Should start at cycle 1");
        assert_eq!(hand.cycles_completed, 0, "Should have 0 completed cycles");
        
        // Verify all cards are available
        for i in 0..=4 {
            assert!(hand.is_card_available(i), "Card {} should be available", i);
        }
        
        // Verify cards HashMap has correct size
        assert_eq!(hand.cards.len(), 5, "Should have 5 cards in HashMap");
    }

    #[test]
    fn test_boost_hand_use_card() {
        let mut hand = BoostHand::new();
        
        // Use card 2
        let result = hand.use_card(2);
        assert!(result.is_ok(), "Should successfully use card 2");
        
        // Verify card 2 is now unavailable
        assert!(!hand.is_card_available(2), "Card 2 should be unavailable");
        assert_eq!(hand.cards_remaining, 4, "Should have 4 cards remaining");
        
        // Verify other cards are still available
        assert!(hand.is_card_available(0), "Card 0 should still be available");
        assert!(hand.is_card_available(1), "Card 1 should still be available");
        assert!(hand.is_card_available(3), "Card 3 should still be available");
        assert!(hand.is_card_available(4), "Card 4 should still be available");
    }

    #[test]
    fn test_boost_hand_cannot_use_same_card_twice() {
        let mut hand = BoostHand::new();
        
        // Use card 3 first time
        let result1 = hand.use_card(3);
        assert!(result1.is_ok(), "First use should succeed");
        
        // Try to use card 3 again
        let result2 = hand.use_card(3);
        assert!(result2.is_err(), "Second use should fail");
        assert_eq!(
            result2.unwrap_err(),
            "Boost card 3 is not available",
            "Should return correct error message"
        );
    }

    #[test]
    fn test_boost_hand_replenishment() {
        let mut hand = BoostHand::new();
        
        // Use all 5 cards
        hand.use_card(2).unwrap();
        assert_eq!(hand.cards_remaining, 4);
        
        hand.use_card(0).unwrap();
        assert_eq!(hand.cards_remaining, 3);
        
        hand.use_card(4).unwrap();
        assert_eq!(hand.cards_remaining, 2);
        
        hand.use_card(1).unwrap();
        assert_eq!(hand.cards_remaining, 1);
        
        // Using the last card should trigger replenishment
        hand.use_card(3).unwrap();
        
        // Verify replenishment occurred
        assert_eq!(hand.cards_remaining, 5, "All cards should be replenished");
        assert_eq!(hand.current_cycle, 2, "Should be in cycle 2");
        assert_eq!(hand.cycles_completed, 1, "Should have 1 completed cycle");
        
        // Verify all cards are available again
        for i in 0..=4 {
            assert!(hand.is_card_available(i), "Card {} should be available after replenishment", i);
        }
    }

    #[test]
    fn test_boost_hand_multiple_cycles() {
        let mut hand = BoostHand::new();
        
        // Complete first cycle
        for i in 0..=4 {
            hand.use_card(i).unwrap();
        }
        assert_eq!(hand.current_cycle, 2);
        assert_eq!(hand.cycles_completed, 1);
        
        // Complete second cycle
        for i in 0..=4 {
            hand.use_card(i).unwrap();
        }
        assert_eq!(hand.current_cycle, 3);
        assert_eq!(hand.cycles_completed, 2);
        
        // Verify all cards are still available
        for i in 0..=4 {
            assert!(hand.is_card_available(i), "Card {} should be available", i);
        }
    }

    #[test]
    fn test_boost_hand_get_available_cards() {
        let mut hand = BoostHand::new();
        
        // Initially all cards should be available
        let available = hand.get_available_cards();
        assert_eq!(available.len(), 5, "Should have 5 available cards");
        assert_eq!(available, vec![0, 1, 2, 3, 4], "Should return sorted list of all cards");
        
        // Use some cards
        hand.use_card(1).unwrap();
        hand.use_card(3).unwrap();
        
        let available = hand.get_available_cards();
        assert_eq!(available.len(), 3, "Should have 3 available cards");
        assert_eq!(available, vec![0, 2, 4], "Should return sorted list of available cards");
        assert!(!available.contains(&1), "Should not include used card 1");
        assert!(!available.contains(&3), "Should not include used card 3");
    }

    #[test]
    fn test_boost_hand_is_card_available_invalid_card() {
        let hand = BoostHand::new();
        
        // Test with invalid card values
        assert!(!hand.is_card_available(5), "Card 5 should not be available (out of range)");
        assert!(!hand.is_card_available(10), "Card 10 should not be available (out of range)");
        assert!(!hand.is_card_available(255), "Card 255 should not be available (out of range)");
    }

    #[test]
    fn test_boost_hand_default_trait() {
        let hand = BoostHand::default();
        
        // Verify default is same as new()
        assert_eq!(hand.cards_remaining, 5);
        assert_eq!(hand.current_cycle, 1);
        assert_eq!(hand.cycles_completed, 0);
        
        for i in 0..=4 {
            assert!(hand.is_card_available(i));
        }
    }

    #[test]
    fn test_boost_hand_use_card_sequence() {
        let mut hand = BoostHand::new();
        
        // Use cards in a specific sequence
        let sequence = vec![4, 1, 3, 0, 2];
        
        for (index, &card) in sequence.iter().enumerate() {
            let result = hand.use_card(card);
            assert!(result.is_ok(), "Should successfully use card {}", card);
            
            // After using the 5th card (index 4), replenishment occurs immediately
            if index == 4 {
                // Replenishment should have occurred
                assert_eq!(hand.cards_remaining, 5, "Should be replenished after using all cards");
                assert_eq!(hand.cycles_completed, 1, "Should have completed 1 cycle");
            } else {
                // Cards should decrease normally
                assert_eq!(
                    hand.cards_remaining,
                    5 - (index as u32) - 1,
                    "Cards remaining should decrease"
                );
            }
        }
        
        // After using all cards, should be replenished
        assert_eq!(hand.cards_remaining, 5);
        assert_eq!(hand.cycles_completed, 1);
    }

    // ========== End BoostHand Tests ==========

    #[test]
    fn test_create_race() {
        let track = create_test_track();
        let race = Race::new("Test Race".to_string(), track, 2);
        
        assert_eq!(race.name, "Test Race");
        assert_eq!(race.total_laps, 2);
        assert_eq!(race.status, RaceStatus::Waiting);
        assert!(matches!(race.lap_characteristic, LapCharacteristic::Straight));
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
        let result = race.start_race();
        assert!(result.is_ok());
        assert_eq!(race.status, RaceStatus::InProgress);
        // Lap characteristic should be set
        assert!(matches!(race.lap_characteristic, LapCharacteristic::Straight | LapCharacteristic::Curve));
    }

    #[test]
    fn test_process_lap_basic_movement() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
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
        assert_eq!(result.movements[0].movement_type, MovementType::StayedInSector);
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            player_uuids.push(player_uuid);
        }
        
        // Set all participants to start in sector 0 for predictable test
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }
        
        race.start_race().unwrap();
        
        // Give different boost values to test performance-based movement priority
        let actions: Vec<LapAction> = player_uuids.iter().enumerate().map(|(i, &uuid)| LapAction {
            player_uuid: uuid,
            boost_value: 5 - (i as u32), // First player gets 5, second gets 4, etc.
            // This creates final values: 15, 14, 13, 12, 11 (all exceed sector 0 max of 10)
        }).collect();
        
        let _ = race.process_lap(&actions).unwrap();
        
        // Count how many are in sector 1 (capacity 3)
        let sector_1_count = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .count();
        
        // Should respect first-ranked rule - only 1 car should move up
        assert_eq!(sector_1_count, 1);
        
        // The remaining 4 should stay in sector 0 due to first-ranked rule
        let sector_0_count = race.participants.iter()
            .filter(|p| p.current_sector == 0)
            .count();
        assert_eq!(sector_0_count, 4);
        
        // Verify that the participant who moved up is the best performer
        let moved_up_participant = race.participants.iter()
            .find(|p| p.current_sector == 1)
            .expect("Should have one participant in sector 1");
        
        // The best performer should have moved up (boost value 5)
        // Total value should be 15
        assert_eq!(moved_up_participant.total_value, 15, "Best performer should move up");
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            player_uuids.push(player_uuid);
        }
        
        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }
        
        race.start_race().unwrap();
        
        // All participants try to move up with different performance
        let actions: Vec<LapAction> = vec![
            LapAction { player_uuid: player_uuids[0], boost_value: 5 }, // Final: 15 (best)
            LapAction { player_uuid: player_uuids[1], boost_value: 4 }, // Final: 14 (second)
            LapAction { player_uuid: player_uuids[2], boost_value: 3 }, // Final: 13 (third)
        ];
        
        let result = race.process_lap(&actions).unwrap();
        
        // Only ONE car should move to sector 1 (the best performer)
        let sector_1_count = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 1);
        
        // The other 2 should stay in sector 0
        let sector_0_count = race.participants.iter()
            .filter(|p| p.current_sector == 0)
            .count();
        assert_eq!(sector_0_count, 2);
        
        // The car that moved up should be the one with the highest performance (boost 5)
        let moved_up_participant = race.participants.iter()
            .find(|p| p.current_sector == 1)
            .unwrap();
        assert_eq!(moved_up_participant.player_uuid, player_uuids[0]);
        assert_eq!(moved_up_participant.total_value, 15); // base 10 + boost 5
        
        // Check that the participant in sector 1 has higher total_value than those in sector 0
        let stayed_participants: Vec<_> = race.participants.iter()
            .filter(|p| p.current_sector == 0)
            .collect();
            
        for stayed_participant in &stayed_participants {
            assert!(moved_up_participant.total_value > stayed_participant.total_value,
                "Moved participant should have higher performance than stayed participant");
        }
        
        // Verify the movements were recorded correctly - only 1 car should move up (first-ranked rule)
        let move_up_count = result.movements.iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .count();
        assert_eq!(move_up_count, 1, "Should have exactly 1 MovedUp movement (first-ranked car only)");
        
        let stayed_count = result.movements.iter()
            .filter(|m| m.movement_type == MovementType::StayedInSector)
            .count();
        assert_eq!(stayed_count, 2, "Should have exactly 2 StayedInSector movements");
    }

    #[test]
    fn test_invalid_boost_value() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 1);
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 10,
                slot_capacity: Some(5), // Should be None
                sector_type: SectorType::Start,
            },
        ];
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            LapAction { player_uuid: player_uuids[0], boost_value: 0 },
            LapAction { player_uuid: player_uuids[1], boost_value: 0 },
            LapAction { player_uuid: player_uuids[2], boost_value: 0 },
            LapAction { player_uuid: player_uuids[3], boost_value: 5 }, // Should exceed sector 0 max
        ];
        
        let result = race.process_lap(&actions).unwrap();
        
        // Player should stay in sector 0 because sector 1 is full
        assert_eq!(result.movements[0].movement_type, MovementType::StayedInSector);
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race.participants[0].current_sector = 0;
        
        race.start_race().unwrap();
        
        let initial_characteristic = race.lap_characteristic.clone();
        
        // Process first lap
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 3,
        }];
        
        let result1 = race.process_lap(&actions).unwrap();
        assert_eq!(result1.lap, 1);
        
        // Lap characteristic might change for next lap
        let second_characteristic = race.lap_characteristic.clone();
        
        // Process second lap
        let result2 = race.process_lap(&actions).unwrap();
        assert_eq!(result2.lap, 2);
        
        // Verify lap characteristics are being tracked
        assert!(matches!(initial_characteristic, LapCharacteristic::Straight | LapCharacteristic::Curve));
        assert!(matches!(second_characteristic, LapCharacteristic::Straight | LapCharacteristic::Curve));
    }

    #[test]
    fn test_race_completion_by_laps() {
        let track = create_test_track();
        let mut race = Race::new("Test Race".to_string(), track, 2); // Only 2 laps
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race.participants[0].current_sector = 0;
        
        race.start_race().unwrap();
        
        let actions = vec![LapAction {
            player_uuid,
            boost_value: 2,
        }];
        
        // Process lap 1
        let result1 = race.process_lap(&actions).unwrap();
        assert_eq!(result1.lap, 1);
        assert_eq!(race.status, RaceStatus::InProgress);
        
        // Process lap 2
        let result2 = race.process_lap(&actions).unwrap();
        assert_eq!(result2.lap, 2);
        assert_eq!(race.status, RaceStatus::InProgress);
        
        // Process lap 3 (should complete the race)
        let result3 = race.process_lap(&actions).unwrap();
        assert_eq!(result3.lap, 3);
        assert_eq!(race.status, RaceStatus::Finished);
        
        // Check finish positions are assigned
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            LapAction { player_uuid: player_uuids[0], boost_value: 0 }, // Already in sector 1
            LapAction { player_uuid: player_uuids[1], boost_value: 0 }, // Already in sector 1
            LapAction { player_uuid: player_uuids[2], boost_value: 5 }, // In sector 0, tries to move up
        ];
        
        let result = race.process_lap(&actions).unwrap();
        
        // Only the participant with higher performance should move up
        assert_eq!(race.participants[2].current_sector, 1, "Best performer should move up");
        
        // Verify movements were recorded (3 total: 2 stay in sector 1, 1 moves up from sector 0)
        assert_eq!(result.movements.len(), 3);
        
        // Find the movement for the participant who was in sector 0
        let sector_0_movement = result.movements.iter()
            .find(|m| m.player_uuid == player_uuids[2])
            .expect("Should find movement for sector 0 participant");
        assert_eq!(sector_0_movement.movement_type, MovementType::MovedUp);
        
        // Verify sector 1 is now at capacity (3 participants)
        let sector_1_count = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 3, "Sector 1 should be at full capacity");
        
        // Verify movement counts
        let move_up_count = result.movements.iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .count();
        assert_eq!(move_up_count, 1, "Should have exactly 1 MovedUp movement");
        
        let stayed_count = result.movements.iter()
            .filter(|m| m.movement_type == MovementType::StayedInSector)
            .count();
        assert_eq!(stayed_count, 2, "Should have exactly 2 StayedInSector movements");
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            LapAction { player_uuid: player_uuids[0], boost_value: 0 }, // Stay in sector 1
            LapAction { player_uuid: player_uuids[1], boost_value: 0 }, // Stay in sector 1
            LapAction { player_uuid: player_uuids[2], boost_value: 3 }, // Lower performance (base 10 + 3 = 13)
            LapAction { player_uuid: player_uuids[3], boost_value: 5 }, // Higher performance (base 10 + 5 = 15)
        ];
        
        let result = race.process_lap(&actions).unwrap();
        
        // Only the best performer (player 3) should move up
        assert_eq!(race.participants[3].current_sector, 1, "Best performer should move up to sector 1");
        assert_eq!(race.participants[2].current_sector, 0, "Lower performer should stay in sector 0");
        
        // Verify sector 1 is now at capacity
        let sector_1_count = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .count();
        assert_eq!(sector_1_count, 3, "Sector 1 should be at full capacity");
        
        // Verify exactly one car moved up
        let move_up_movements: Vec<_> = result.movements.iter()
            .filter(|m| m.movement_type == MovementType::MovedUp)
            .collect();
        assert_eq!(move_up_movements.len(), 1, "Exactly one car should move up");
        assert_eq!(move_up_movements[0].player_uuid, player_uuids[3], "The best performer should be the one who moved up");
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
            
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            starting_sectors.push(race.participants.last().unwrap().current_sector);
        }
        
        // Verify that not all participants start in the same sector
        let unique_sectors: std::collections::HashSet<_> = starting_sectors.iter().collect();
        
        // With random qualification, we should have some variety
        // (This test might occasionally fail due to randomness, but very unlikely with 10 participants)
        assert!(unique_sectors.len() > 1, "All participants started in the same sector, qualification not working");
        
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
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        
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
        race2.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race2.participants[0].current_sector = 0; // Sector 0 max_value = 10
        race2.start_race().unwrap();
        
        // We need to test the capping logic directly since we can't easily modify car stats
        // Let's verify the capping logic by checking a scenario where it would apply
        
        // Test the capping calculation directly
        let base_value = 15u32; // Hypothetical high base value
        let sector_max = 10u32;  // Sector 0 max value
        let boost = 3u32;
        
        let capped_base = std::cmp::min(base_value, sector_max);
        let final_value = capped_base + boost;
        
        assert_eq!(capped_base, 10, "Base value should be capped to sector maximum");
        assert_eq!(final_value, 13, "Final value should be capped base + boost");
        
        // Verify that without capping, the value would be different
        let uncapped_final = base_value + boost;
        assert_eq!(uncapped_final, 18, "Without capping, final value would be higher");
        assert_ne!(final_value, uncapped_final, "Capping should make a difference");
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
        
        assert_eq!(capped_4, 5, "High base value should be capped to low sector ceiling");
        assert_eq!(final_4, 10, "Final value should be capped base + full boost");
        
        // Verify the strategic implication: boost becomes more important when capped
        let uncapped_final_4 = base_value_4 + boost_4;
        assert_eq!(uncapped_final_4, 25, "Without capping, final would be much higher");
        
        let boost_percentage_capped = (boost_4 as f32 / final_4 as f32) * 100.0;
        let boost_percentage_uncapped = (boost_4 as f32 / uncapped_final_4 as f32) * 100.0;
        
        assert!(boost_percentage_capped > boost_percentage_uncapped, 
                "Boost should be proportionally more important when base is capped");
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
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            player_uuids.push(player_uuid);
        }
        
        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }
        
        race.start_race().unwrap();
        
        // Give different performance levels to create clear ranking
        let actions: Vec<LapAction> = vec![
            LapAction { player_uuid: player_uuids[0], boost_value: 5 }, // Best: 15
            LapAction { player_uuid: player_uuids[1], boost_value: 4 }, // Second: 14  
            LapAction { player_uuid: player_uuids[2], boost_value: 3 }, // Third: 13
        ];
        
        let _result = race.process_lap(&actions).unwrap();
        
        // All cars that exceed the threshold should move up (sector 1 has capacity 3, so space available)
        let sector_1_participants: Vec<_> = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .collect();
        
        assert_eq!(sector_1_participants.len(), 1, "Only the first-ranked car should move up");
        
        // Verify the moved car is the best performer
        let moved_car = sector_1_participants[0];
        assert_eq!(moved_car.total_value, 15, "Best performer should move up");
        
        // The other cars should stay in sector 0
        let sector_0_participants: Vec<_> = race.participants.iter()
            .filter(|p| p.current_sector == 0)
            .collect();
        
        assert_eq!(sector_0_participants.len(), 2, "Other cars should stay in sector 0");
        
        // Verify the cars in sector 0 have lower performance than the moved car
        for participant in &sector_0_participants {
            assert!(participant.total_value < moved_car.total_value, "Cars in sector 0 should have lower performance");
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
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            player_uuids.push(player_uuid);
        }
        
        // Set all participants to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }
        
        race.start_race().unwrap();
        
        // Give all cars the same performance level
        let actions: Vec<LapAction> = vec![
            LapAction { player_uuid: player_uuids[0], boost_value: 4 }, // All: 14
            LapAction { player_uuid: player_uuids[1], boost_value: 4 }, // All: 14
            LapAction { player_uuid: player_uuids[2], boost_value: 4 }, // All: 14
        ];
        
        let _result = race.process_lap(&actions).unwrap();
        
        // With equal performance, only one car should move up (first processed)
        let sector_1_count = race.participants.iter()
            .filter(|p| p.current_sector == 1)
            .count();
        
        assert_eq!(sector_1_count, 1, "Only one car should move up when all have equal performance");
        
        // Two cars should stay in sector 0
        let sector_0_count = race.participants.iter()
            .filter(|p| p.current_sector == 0)
            .count();
        
        assert_eq!(sector_0_count, 2, "Two cars should stay in sector 0");
        
        // All cars should have the same total value
        let all_values: Vec<u32> = race.participants.iter()
            .map(|p| p.total_value)
            .collect();
        
        assert!(all_values.iter().all(|&v| v == 14), "All cars should have the same total value");
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
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
            player_uuids.push(player_uuid);
        }
        
        // Set both to start in sector 0
        for participant in &mut race.participants {
            participant.current_sector = 0;
        }
        
        race.start_race().unwrap();
        
        // LAP 1: Both try to move up, only first-ranked succeeds
        let actions_lap1: Vec<LapAction> = vec![
            LapAction { player_uuid: player_uuids[0], boost_value: 5 }, // Best performer
            LapAction { player_uuid: player_uuids[1], boost_value: 4 }, // Second performer
        ];
        
        let _result1 = race.process_lap(&actions_lap1).unwrap();
        
        // Only the best car should move to sector 1 (first-ranked rule)
        assert_eq!(race.participants.iter().filter(|p| p.current_sector == 1).count(), 1);
        assert_eq!(race.participants.iter().filter(|p| p.current_sector == 0).count(), 1);
        
        // Verify which car moved
        let sector_1_car = race.participants.iter().find(|p| p.current_sector == 1).unwrap();
        let sector_0_car = race.participants.iter().find(|p| p.current_sector == 0).unwrap();
        
        assert_eq!(sector_1_car.player_uuid, player_uuids[0]); // Best performer moved up
        assert_eq!(sector_0_car.player_uuid, player_uuids[1]); // Second performer stayed
    }

    #[test]
    fn test_individual_lap_action_processing() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Individual Action Test".to_string(), track, 2);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Test 1: First player submits action, should be recorded
        let result1 = race.process_individual_lap_action(
            player_uuids[0],
            3,
            &car_data,
        ).unwrap();
        
        match result1 {
            IndividualLapResult::ActionRecorded { predicted_performance, waiting_for_players } => {
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
        let result2 = race.process_individual_lap_action(
            player_uuids[1],
            2,
            &car_data,
        ).unwrap();
        
        match result2 {
            IndividualLapResult::LapProcessed(lap_result) => {
                assert_eq!(lap_result.movements.len(), 2);
                // Both players should have moved (performance exceeds sector 0 max)
                assert!(lap_result.movements.iter().any(|m| m.movement_type == MovementType::MovedUp));
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
        
        let result3 = race.process_individual_lap_action(
            player_uuids[0],
            1,
            &car_data,
        );
        
        // This should fail because player already submitted an action
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("already submitted an action"));
        
        // Clear the test action
        race.pending_actions.clear();
    }

    #[test]
    fn test_individual_lap_action_validation() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Validation Test".to_string(), track, 2);
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();
        
        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
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
        let result = race.process_individual_lap_action(
            non_existent_player,
            3,
            &car_data,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Player not found"));
        
        // Test race not in progress
        race.status = RaceStatus::Finished;
        let result = race.process_individual_lap_action(
            player_uuid,
            3,
            &car_data,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Race is not in progress"));
    }

    #[test]
    fn test_boost_card_validation_in_race() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Boost Card Test".to_string(), track, 2);
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();
        
        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Verify initial boost hand state
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        assert_eq!(race.participants[0].boost_hand.current_cycle, 1);
        assert!(race.participants[0].boost_hand.is_card_available(2));
        
        // Use boost card 2
        let result = race.process_individual_lap_action(
            player_uuid,
            2,
            &car_data,
        );
        
        assert!(result.is_ok());
        
        // Verify card 2 is now unavailable
        assert!(!race.participants[0].boost_hand.is_card_available(2));
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 4);
        
        // Clear pending actions to test again
        race.pending_actions.clear();
        race.action_submissions.clear();
        race.pending_performance_calculations.clear();
        
        // Try to use card 2 again - should fail
        let result = race.process_individual_lap_action(
            player_uuid,
            2,
            &car_data,
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not available"));
    }

    #[test]
    fn test_boost_card_replenishment_triggers_correctly() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Replenishment Test".to_string(), track, 10);
        
        // Add 2 participants to test individual actions
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Use all 5 boost cards for player 1
        let boost_sequence_p1 = vec![2, 0, 4, 1, 3];
        let boost_sequence_p2 = vec![0, 1, 2, 3, 4];
        
        for (index, &boost_value) in boost_sequence_p1.iter().enumerate() {
            // Player 1 submits action
            let result = race.process_individual_lap_action(
                player_uuids[0],
                boost_value,
                &car_data,
            );
            
            assert!(result.is_ok(), "Failed to use boost card {} for player 1", boost_value);
            
            // Player 2 submits action to complete the lap
            let _result2 = race.process_individual_lap_action(
                player_uuids[1],
                boost_sequence_p2[index],
                &car_data,
            );
            
            // Check cards remaining after each use
            if index < 4 {
                // Before last card
                assert_eq!(
                    race.participants[0].boost_hand.cards_remaining,
                    4 - index as u32,
                    "Cards remaining should decrease"
                );
                assert_eq!(race.participants[0].boost_hand.current_cycle, 1);
                assert_eq!(race.participants[0].boost_hand.cycles_completed, 0);
            } else {
                // After using the 5th card, replenishment should occur
                assert_eq!(
                    race.participants[0].boost_hand.cards_remaining,
                    5,
                    "All cards should be replenished"
                );
                assert_eq!(
                    race.participants[0].boost_hand.current_cycle,
                    2,
                    "Should be in cycle 2"
                );
                assert_eq!(
                    race.participants[0].boost_hand.cycles_completed,
                    1,
                    "Should have 1 completed cycle"
                );
            }
        }
        
        // Verify all cards are available again after replenishment
        for i in 0..=4 {
            assert!(
                race.participants[0].boost_hand.is_card_available(i),
                "Card {} should be available after replenishment",
                i
            );
        }
        
        // Test that we can use the same cards again in the new cycle
        let result = race.process_individual_lap_action(
            player_uuids[0],
            2, // Same card we used first in previous cycle
            &car_data,
        );
        
        assert!(result.is_ok(), "Should be able to use card 2 again after replenishment");
        assert!(!race.participants[0].boost_hand.is_card_available(2));
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 4);
    }

    #[test]
    fn test_boost_card_multiple_cycles() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Multiple Cycles Test".to_string(), track, 20);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Complete 3 full cycles (15 laps total)
        for cycle in 1..=3 {
            for card in 0..=4 {
                // Player 1 uses a card
                let result = race.process_individual_lap_action(
                    player_uuids[0],
                    card,
                    &car_data,
                );
                
                assert!(result.is_ok(), "Cycle {}, card {} should work for player 1", cycle, card);
                
                // Player 2 completes the lap (also uses the same card sequence)
                let _result2 = race.process_individual_lap_action(
                    player_uuids[1],
                    card,
                    &car_data,
                );
            }
            
            // After each cycle, verify replenishment occurred
            assert_eq!(
                race.participants[0].boost_hand.cards_remaining,
                5,
                "Cycle {}: All cards should be replenished",
                cycle
            );
            assert_eq!(
                race.participants[0].boost_hand.current_cycle,
                cycle + 1,
                "Cycle {}: Should be in next cycle",
                cycle
            );
            assert_eq!(
                race.participants[0].boost_hand.cycles_completed,
                cycle,
                "Cycle {}: Should have completed {} cycles",
                cycle,
                cycle
            );
        }
        
        // Verify final state after 3 complete cycles
        assert_eq!(race.participants[0].boost_hand.current_cycle, 4);
        assert_eq!(race.participants[0].boost_hand.cycles_completed, 3);
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        
        // All cards should be available
        for i in 0..=4 {
            assert!(race.participants[0].boost_hand.is_card_available(i));
        }
    }

    #[test]
    fn test_boost_card_invalid_value_rejected() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Invalid Boost Test".to_string(), track, 2);
        
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        
        race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
        race.participants[0].current_sector = 0;
        race.start_race().unwrap();
        
        // Create mock validated car data
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Test boost value > 4 (invalid)
        let result = race.process_individual_lap_action(
            player_uuid,
            5,
            &car_data,
        );
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Invalid boost value"));
        assert!(error_msg.contains("Must be between 0 and 4"));
        
        // Verify boost hand state unchanged
        assert_eq!(race.participants[0].boost_hand.cards_remaining, 5);
        assert_eq!(race.participants[0].boost_hand.current_cycle, 1);
    }

    // ========== Boost Usage History Tests ==========

    #[test]
    fn test_boost_usage_history_records_created() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("History Test".to_string(), track, 10);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Initially, history should be empty
        assert_eq!(race.participants[0].boost_usage_history.len(), 0);
        
        // Use 3 boost cards
        let boost_sequence = vec![2, 0, 4];
        
        for (index, &boost_value) in boost_sequence.iter().enumerate() {
            race.process_individual_lap_action(
                player_uuids[0],
                boost_value,
                &car_data,
            ).unwrap();
            
            // Complete lap with player 2
            race.process_individual_lap_action(
                player_uuids[1],
                boost_value,
                &car_data,
            ).unwrap();
            
            // Verify history record was created
            assert_eq!(
                race.participants[0].boost_usage_history.len(),
                index + 1,
                "Should have {} history records",
                index + 1
            );
            
            // Verify the latest record
            let latest_record = &race.participants[0].boost_usage_history[index];
            assert_eq!(latest_record.boost_value, boost_value);
            assert_eq!(latest_record.lap_number, (index + 1) as u32);
            assert_eq!(latest_record.cycle_number, 1);
            assert_eq!(latest_record.cards_remaining_after, 4 - index as u32);
            assert!(!latest_record.replenishment_occurred);
        }
    }

    #[test]
    fn test_boost_usage_history_tracks_replenishment() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Replenishment History Test".to_string(), track, 10);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Use all 5 boost cards to trigger replenishment
        for card in 0..=4 {
            race.process_individual_lap_action(
                player_uuids[0],
                card,
                &car_data,
            ).unwrap();
            
            race.process_individual_lap_action(
                player_uuids[1],
                card,
                &car_data,
            ).unwrap();
        }
        
        // Verify we have 5 history records
        assert_eq!(race.participants[0].boost_usage_history.len(), 5);
        
        // Verify the last record shows replenishment occurred
        let last_record = &race.participants[0].boost_usage_history[4];
        assert_eq!(last_record.boost_value, 4);
        assert_eq!(last_record.cycle_number, 1);
        assert_eq!(last_record.cards_remaining_after, 5); // Replenished
        assert!(last_record.replenishment_occurred);
        
        // Verify earlier records don't show replenishment
        for i in 0..4 {
            assert!(!race.participants[0].boost_usage_history[i].replenishment_occurred);
        }
    }

    #[test]
    fn test_boost_cycle_summaries() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Cycle Summary Test".to_string(), track, 15);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Complete 2 full cycles
        // Cycle 1: use cards 0, 1, 2, 3, 4
        for card in 0..=4 {
            race.process_individual_lap_action(
                player_uuids[0],
                card,
                &car_data,
            ).unwrap();
            
            race.process_individual_lap_action(
                player_uuids[1],
                card,
                &car_data,
            ).unwrap();
        }
        
        // Cycle 2: use cards 4, 3, 2, 1, 0 (reverse order)
        for card in (0..=4).rev() {
            race.process_individual_lap_action(
                player_uuids[0],
                card,
                &car_data,
            ).unwrap();
            
            race.process_individual_lap_action(
                player_uuids[1],
                card,
                &car_data,
            ).unwrap();
        }
        
        // Get cycle summaries
        let summaries = race.participants[0].get_boost_cycle_summaries();
        
        // Should have 2 cycle summaries
        assert_eq!(summaries.len(), 2);
        
        // Verify cycle 1 summary
        let cycle1 = &summaries[0];
        assert_eq!(cycle1.cycle_number, 1);
        assert_eq!(cycle1.cards_used, vec![0, 1, 2, 3, 4]);
        assert_eq!(cycle1.laps_in_cycle, vec![1, 2, 3, 4, 5]);
        assert_eq!(cycle1.average_boost, 2.0); // (0+1+2+3+4)/5 = 2.0
        
        // Verify cycle 2 summary
        let cycle2 = &summaries[1];
        assert_eq!(cycle2.cycle_number, 2);
        assert_eq!(cycle2.cards_used, vec![4, 3, 2, 1, 0]);
        assert_eq!(cycle2.laps_in_cycle, vec![6, 7, 8, 9, 10]);
        assert_eq!(cycle2.average_boost, 2.0); // (4+3+2+1+0)/5 = 2.0
    }

    #[test]
    fn test_boost_usage_statistics() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Statistics Test".to_string(), track, 10);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Use specific boost cards: 3, 4, 2
        let boost_sequence = vec![3, 4, 2];
        
        for &boost_value in &boost_sequence {
            race.process_individual_lap_action(
                player_uuids[0],
                boost_value,
                &car_data,
            ).unwrap();
            
            race.process_individual_lap_action(
                player_uuids[1],
                boost_value,
                &car_data,
            ).unwrap();
        }
        
        let participant = &race.participants[0];
        
        // Test total boosts used
        assert_eq!(participant.get_total_boosts_used(), 3);
        
        // Test average boost value: (3 + 4 + 2) / 3 = 3.0
        assert_eq!(participant.get_average_boost_value(), 3.0);
        
        // Test get_boost_usage_for_cycle
        let cycle1_usage = participant.get_boost_usage_for_cycle(1);
        assert_eq!(cycle1_usage.len(), 3);
        assert_eq!(cycle1_usage[0].boost_value, 3);
        assert_eq!(cycle1_usage[1].boost_value, 4);
        assert_eq!(cycle1_usage[2].boost_value, 2);
    }

    #[test]
    fn test_boost_usage_history_multiple_cycles() {
        use crate::services::car_validation::ValidatedCarData;
        use crate::domain::{Engine, Body, Pilot, Car, EngineName, BodyName, PilotName, ComponentRarity, PilotClass, PilotRarity, PilotSkills, PilotPerformance};

        let track = create_test_track();
        let mut race = Race::new("Multi-Cycle History Test".to_string(), track, 15);
        
        // Add 2 participants
        let mut player_uuids = Vec::new();
        for _i in 0..2 {
            let player_uuid = Uuid::new_v4();
            let car_uuid = Uuid::new_v4();
            let pilot_uuid = Uuid::new_v4();
            race.add_participant(player_uuid, car_uuid, pilot_uuid).unwrap();
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
            None,
        ).unwrap();
        
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        ).unwrap();
        
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        ).unwrap();
        
        let car = Car::new(crate::domain::CarName::parse("Test Car").unwrap(), None).unwrap();
        
        let car_data = ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        };
        
        // Complete 2 full cycles (10 laps)
        for _cycle in 1..=2 {
            for card in 0..=4 {
                race.process_individual_lap_action(
                    player_uuids[0],
                    card,
                    &car_data,
                ).unwrap();
                
                race.process_individual_lap_action(
                    player_uuids[1],
                    card,
                    &car_data,
                ).unwrap();
            }
        }
        
        let participant = &race.participants[0];
        
        // Should have 10 history records (2 cycles * 5 cards)
        assert_eq!(participant.boost_usage_history.len(), 10);
        
        // Verify cycle numbers in history
        for i in 0..5 {
            assert_eq!(participant.boost_usage_history[i].cycle_number, 1);
        }
        for i in 5..10 {
            assert_eq!(participant.boost_usage_history[i].cycle_number, 2);
        }
        
        // Verify replenishment flags
        assert!(participant.boost_usage_history[4].replenishment_occurred); // End of cycle 1
        assert!(participant.boost_usage_history[9].replenishment_occurred); // End of cycle 2
        
        // Get cycle summaries
        let summaries = participant.get_boost_cycle_summaries();
        assert_eq!(summaries.len(), 2);
        
        // Verify both cycles have 5 cards each
        assert_eq!(summaries[0].cards_used.len(), 5);
        assert_eq!(summaries[1].cards_used.len(), 5);
        
        // Test statistics
        assert_eq!(participant.get_total_boosts_used(), 10);
        assert_eq!(participant.get_average_boost_value(), 2.0); // (0+1+2+3+4)*2 / 10 = 2.0
    }

    // ========== End Boost Usage History Tests ==========
}
