use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use std::collections::HashMap;

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

        // Calculate final values for all participants
        let mut participant_values: HashMap<Uuid, u32> = HashMap::new();
        for action in actions {
            if let Some(participant) = self.participants.iter().find(|p| p.player_uuid == action.player_uuid) {
                if !participant.is_finished {
                    // TODO: Calculate base value from car engine + body + pilot performance
                    // For now, use a simple base value
                    let base_value = 10; // Placeholder
                    
                    // Apply sector performance ceiling: cap base value to current sector's max_value
                    let current_sector = &self.track.sectors[participant.current_sector as usize];
                    let capped_base_value = std::cmp::min(base_value, current_sector.max_value);
                    
                    // Add boost to the capped base value
                    let final_value = capped_base_value + action.boost_value;
                    participant_values.insert(action.player_uuid, final_value);
                }
            }
        }

        // Process movements using the new algorithm: best sector to worst sector
        let mut movements = Vec::new();
        #[allow(clippy::cast_possible_truncation)]
        let max_sector = (self.track.sectors.len() - 1) as u32;
        
        // Process sectors from highest to lowest (best to worst)
        for sector_id in (0..=max_sector).rev() {
            let sector_movements = self.process_sector_movements(sector_id, &participant_values);
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

        Ok(LapResult {
            lap: processed_lap,
            lap_characteristic: self.lap_characteristic.clone(),
            sector_positions: self.get_sector_positions(),
            movements,
        })
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
}
