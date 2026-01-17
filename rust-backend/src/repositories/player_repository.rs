use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::domain::{Car, Pilot, Player, TeamName, WalletAddress};

#[async_trait]
pub trait PlayerRepository: Send + Sync {
    async fn create(&self, player: &Player) -> RepositoryResult<Player>;
    async fn find_all(&self) -> RepositoryResult<Vec<Player>>;
    async fn find_by_wallet_address(
        &self,
        wallet_address: &str,
    ) -> RepositoryResult<Option<Player>>;
    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>>;
    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>>;
    async fn update_team_name_by_wallet(
        &self,
        wallet_address: &str,
        team_name: TeamName,
    ) -> RepositoryResult<Option<Player>>;
    async fn update_team_name_by_uuid(
        &self,
        player_uuid: Uuid,
        team_name: TeamName,
    ) -> RepositoryResult<Option<Player>>;
    async fn update_wallet_address(
        &self,
        player_uuid: Uuid,
        wallet_address: WalletAddress,
    ) -> RepositoryResult<Option<Player>>;
    async fn delete_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<bool>;
    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool>;
    async fn add_car_by_wallet(
        &self,
        wallet_address: &str,
        car: Car,
    ) -> RepositoryResult<Option<Player>>;
    async fn add_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car: Car,
    ) -> RepositoryResult<Option<Player>>;
    async fn remove_car_by_wallet(
        &self,
        wallet_address: &str,
        car_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>>;
    async fn remove_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>>;
    async fn add_pilot_by_wallet(
        &self,
        wallet_address: &str,
        pilot: Pilot,
    ) -> RepositoryResult<Option<Player>>;
    async fn add_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot: Pilot,
    ) -> RepositoryResult<Option<Player>>;
    async fn remove_pilot_by_wallet(
        &self,
        wallet_address: &str,
        pilot_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>>;
    async fn remove_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>>;
    async fn set_cars_by_uuid(
        &self,
        player_uuid: Uuid,
        cars: Vec<Car>,
    ) -> RepositoryResult<Option<Player>>;
}
