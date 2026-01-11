use async_trait::async_trait;
use mongodb::{bson::doc, Database};
use uuid::Uuid;

use crate::domain::{LapAction, LapResult, Race, RaceStatus};
use crate::services::car_validation::ValidatedCarData;
use super::{RepositoryError, RepositoryResult};

#[async_trait]
pub trait RaceRepository: Send + Sync {
    async fn create(&self, race: &Race) -> RepositoryResult<Race>;
    async fn find_all(&self) -> RepositoryResult<Vec<Race>>;
    async fn find_by_uuid(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn find_by_pilot_uuid(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn join_race(&self, race_uuid: Uuid, pilot_uuid: Uuid, car_data: &ValidatedCarData) -> RepositoryResult<Option<Race>>;
    async fn process_turn_actions(&self, race_uuid: Uuid, pilot_uuid: Uuid, actions: Vec<LapAction>) -> RepositoryResult<Option<(LapResult, RaceStatus)>>;
    async fn submit_turn_action(&self, race_uuid: Uuid, pilot_uuid: Uuid, boost_value: u32) -> RepositoryResult<Option<Race>>;
    async fn update_race_status(&self, race_uuid: Uuid, status: RaceStatus) -> RepositoryResult<Option<Race>>;
    async fn get_races_by_status(&self, status: RaceStatus) -> RepositoryResult<Vec<Race>>;
}

pub struct MongoRaceRepository {
    database: Database,
}

impl MongoRaceRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl RaceRepository for MongoRaceRepository {
    async fn create(&self, race: &Race) -> RepositoryResult<Race> {
        let collection = self.database.collection::<Race>("races");
        let result = collection.insert_one(race, None).await?;
        
        let mut created_race = race.clone();
        created_race.id = result.inserted_id.as_object_id();
        Ok(created_race)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Race>> {
        let collection = self.database.collection::<Race>("races");
        let mut cursor = collection.find(None, None).await?;
        
        let mut races = Vec::new();
        while cursor.advance().await? {
            races.push(cursor.deserialize_current()?);
        }
        Ok(races)
    }

    async fn find_by_uuid(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        let filter = doc! { "uuid": race_uuid.to_string() };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn find_by_pilot_uuid(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        let filter = doc! {
            "pilots": {
                "$elemMatch": {
                    "uuid": pilot_uuid.to_string()
                }
            }
        };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        let filter = doc! {
            "pilots": {
                "$elemMatch": {
                    "uuid": pilot_uuid.to_string()
                }
            },
            "status": {
                "$in": ["WaitingForPlayers", "InProgress"]
            }
        };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn join_race(&self, race_uuid: Uuid, pilot_uuid: Uuid, car_data: &ValidatedCarData) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        
        // First, get the race to validate it exists and is joinable
        let filter = doc! { "uuid": race_uuid.to_string() };
        let race = collection.find_one(filter.clone(), None).await?;
        
        if let Some(mut race) = race {
            // Check if race is in correct status
            if !matches!(race.status, RaceStatus::WaitingForPlayers) {
                return Err(RepositoryError::Validation("Race is not accepting new players".to_string()));
            }
            
            // Check if pilot is already in race
            if race.pilots.iter().any(|p| p.uuid == pilot_uuid) {
                return Err(RepositoryError::Conflict("Pilot already in race".to_string()));
            }
            
            // Add pilot to race
            let pilot_with_car = crate::domain::PilotWithCar {
                uuid: pilot_uuid,
                car: car_data.car.clone(),
                current_lap: 0,
                current_sector: 0,
                total_time: 0,
                sector_times: Vec::new(),
                lap_times: Vec::new(),
                position: race.pilots.len() as u32 + 1,
                boost_hand: car_data.boost_hand.clone(),
                actions_submitted: false,
            };
            
            let update = doc! {
                "$push": {
                    "pilots": mongodb::bson::to_bson(&pilot_with_car)?
                }
            };
            
            collection.update_one(filter.clone(), update, None).await?;
            Ok(collection.find_one(filter, None).await?)
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn process_turn_actions(&self, race_uuid: Uuid, pilot_uuid: Uuid, actions: Vec<LapAction>) -> RepositoryResult<Option<(LapResult, RaceStatus)>> {
        let collection = self.database.collection::<Race>("races");
        
        // Get the race first
        let filter = doc! { "uuid": race_uuid.to_string() };
        let race = collection.find_one(filter.clone(), None).await?;
        
        if let Some(mut race) = race {
            // Find the pilot
            if let Some(pilot_index) = race.pilots.iter().position(|p| p.uuid == pilot_uuid) {
                // Process the actions (this would contain the game logic)
                // For now, we'll create a simple lap result
                let lap_result = LapResult {
                    lap_number: race.pilots[pilot_index].current_lap + 1,
                    sector_times: vec![30000, 32000, 28000], // Example times in milliseconds
                    total_lap_time: 90000,
                    position: race.pilots[pilot_index].position,
                    actions_used: actions,
                };
                
                // Update pilot's progress
                race.pilots[pilot_index].current_lap += 1;
                race.pilots[pilot_index].lap_times.push(lap_result.total_lap_time);
                race.pilots[pilot_index].total_time += lap_result.total_lap_time;
                
                // Check if race is finished
                let race_status = if race.pilots[pilot_index].current_lap >= race.total_laps {
                    RaceStatus::Finished
                } else {
                    race.status
                };
                
                // Update the race in database
                let update = doc! {
                    "$set": {
                        "pilots": mongodb::bson::to_bson(&race.pilots)?,
                        "status": mongodb::bson::to_bson(&race_status)?
                    }
                };
                
                collection.update_one(filter, update, None).await?;
                
                Ok(Some((lap_result, race_status)))
            } else {
                Err(RepositoryError::NotFound)
            }
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn submit_turn_action(&self, race_uuid: Uuid, pilot_uuid: Uuid, boost_value: u32) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        
        let filter = doc! { 
            "uuid": race_uuid.to_string(),
            "pilots.uuid": pilot_uuid.to_string()
        };
        
        let update = doc! {
            "$set": {
                "pilots.$.actions_submitted": true,
                "pilots.$.boost_used": boost_value
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        
        let race_filter = doc! { "uuid": race_uuid.to_string() };
        Ok(collection.find_one(race_filter, None).await?)
    }

    async fn update_race_status(&self, race_uuid: Uuid, status: RaceStatus) -> RepositoryResult<Option<Race>> {
        let collection = self.database.collection::<Race>("races");
        let filter = doc! { "uuid": race_uuid.to_string() };
        let update = doc! {
            "$set": {
                "status": mongodb::bson::to_bson(&status)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn get_races_by_status(&self, status: RaceStatus) -> RepositoryResult<Vec<Race>> {
        let collection = self.database.collection::<Race>("races");
        let filter = doc! { "status": mongodb::bson::to_bson(&status)? };
        let mut cursor = collection.find(filter, None).await?;
        
        let mut races = Vec::new();
        while cursor.advance().await? {
            races.push(cursor.deserialize_current()?);
        }
        Ok(races)
    }
}