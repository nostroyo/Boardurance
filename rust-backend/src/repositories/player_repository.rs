use async_trait::async_trait;
use mongodb::{bson::doc, Database};
use uuid::Uuid;

use crate::domain::{Car, Pilot, Player, TeamName, WalletAddress};
use super::{RepositoryError, RepositoryResult};

#[async_trait]
pub trait PlayerRepository: Send + Sync {
    async fn create(&self, player: &Player) -> RepositoryResult<Player>;
    async fn find_all(&self) -> RepositoryResult<Vec<Player>>;
    async fn find_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<Option<Player>>;
    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>>;
    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn update_team_name_by_wallet(&self, wallet_address: &str, team_name: TeamName) -> RepositoryResult<Option<Player>>;
    async fn update_team_name_by_uuid(&self, player_uuid: Uuid, team_name: TeamName) -> RepositoryResult<Option<Player>>;
    async fn update_wallet_address(&self, player_uuid: Uuid, wallet_address: WalletAddress) -> RepositoryResult<Option<Player>>;
    async fn delete_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<bool>;
    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool>;
    async fn add_car_by_wallet(&self, wallet_address: &str, car: Car) -> RepositoryResult<Option<Player>>;
    async fn add_car_by_uuid(&self, player_uuid: Uuid, car: Car) -> RepositoryResult<Option<Player>>;
    async fn remove_car_by_wallet(&self, wallet_address: &str, car_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn remove_car_by_uuid(&self, player_uuid: Uuid, car_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn add_pilot_by_wallet(&self, wallet_address: &str, pilot: Pilot) -> RepositoryResult<Option<Player>>;
    async fn add_pilot_by_uuid(&self, player_uuid: Uuid, pilot: Pilot) -> RepositoryResult<Option<Player>>;
    async fn remove_pilot_by_wallet(&self, wallet_address: &str, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn remove_pilot_by_uuid(&self, player_uuid: Uuid, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn set_cars_by_uuid(&self, player_uuid: Uuid, cars: Vec<Car>) -> RepositoryResult<Option<Player>>;
}

pub struct MongoPlayerRepository {
    database: Database,
}

impl MongoPlayerRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl PlayerRepository for MongoPlayerRepository {
    async fn create(&self, player: &Player) -> RepositoryResult<Player> {
        let collection = self.database.collection::<Player>("players");
        let result = collection.insert_one(player, None).await?;
        
        let mut created_player = player.clone();
        created_player.id = result.inserted_id.as_object_id();
        Ok(created_player)
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Player>> {
        let collection = self.database.collection::<Player>("players");
        let mut cursor = collection.find(None, None).await?;
        
        let mut players = Vec::new();
        while cursor.advance().await? {
            players.push(cursor.deserialize_current()?);
        }
        Ok(players)
    }

    async fn find_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "email": email };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn update_team_name_by_wallet(&self, wallet_address: &str, team_name: TeamName) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let update = doc! {
            "$set": {
                "team_name": team_name.to_string()
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn update_team_name_by_uuid(&self, player_uuid: Uuid, team_name: TeamName) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$set": {
                "team_name": team_name.to_string()
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn update_wallet_address(&self, player_uuid: Uuid, wallet_address: WalletAddress) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$set": {
                "wallet_address": wallet_address.to_string()
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn delete_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<bool> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let result = collection.delete_one(filter, None).await?;
        Ok(result.deleted_count > 0)
    }

    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let result = collection.delete_one(filter, None).await?;
        Ok(result.deleted_count > 0)
    }

    async fn add_car_by_wallet(&self, wallet_address: &str, car: Car) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let update = doc! {
            "$push": {
                "cars": mongodb::bson::to_bson(&car)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn add_car_by_uuid(&self, player_uuid: Uuid, car: Car) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$push": {
                "cars": mongodb::bson::to_bson(&car)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn remove_car_by_wallet(&self, wallet_address: &str, car_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let update = doc! {
            "$pull": {
                "cars": { "uuid": car_uuid.to_string() }
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn remove_car_by_uuid(&self, player_uuid: Uuid, car_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$pull": {
                "cars": { "uuid": car_uuid.to_string() }
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn add_pilot_by_wallet(&self, wallet_address: &str, pilot: Pilot) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let update = doc! {
            "$push": {
                "pilots": mongodb::bson::to_bson(&pilot)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn add_pilot_by_uuid(&self, player_uuid: Uuid, pilot: Pilot) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$push": {
                "pilots": mongodb::bson::to_bson(&pilot)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn remove_pilot_by_wallet(&self, wallet_address: &str, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "wallet_address": wallet_address };
        let update = doc! {
            "$pull": {
                "pilots": { "uuid": pilot_uuid.to_string() }
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn remove_pilot_by_uuid(&self, player_uuid: Uuid, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$pull": {
                "pilots": { "uuid": pilot_uuid.to_string() }
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }

    async fn set_cars_by_uuid(&self, player_uuid: Uuid, cars: Vec<Car>) -> RepositoryResult<Option<Player>> {
        let collection = self.database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };
        let update = doc! {
            "$set": {
                "cars": mongodb::bson::to_bson(&cars)?
            }
        };
        
        collection.update_one(filter.clone(), update, None).await?;
        Ok(collection.find_one(filter, None).await?)
    }
}