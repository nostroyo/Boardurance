use async_trait::async_trait;
use uuid::Uuid;

use super::RepositoryResult;
use crate::domain::{LapAction, LapResult, Race, RaceStatus};
use crate::services::car_validation::ValidatedCarData;

#[async_trait]
pub trait RaceRepository: Send + Sync {
    async fn create(&self, race: &Race) -> RepositoryResult<Race>;
    async fn find_all(&self) -> RepositoryResult<Vec<Race>>;
    async fn find_by_uuid(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn find_by_pilot_uuid(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>>;
    async fn join_race(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        car_data: &ValidatedCarData,
    ) -> RepositoryResult<Option<Race>>;
    async fn process_turn_actions(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        actions: Vec<LapAction>,
    ) -> RepositoryResult<Option<(LapResult, RaceStatus)>>;
    async fn submit_turn_action(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        boost_value: u32,
    ) -> RepositoryResult<Option<Race>>;
    async fn update_race_status(
        &self,
        race_uuid: Uuid,
        status: RaceStatus,
    ) -> RepositoryResult<Option<Race>>;
    async fn get_races_by_status(&self, status: RaceStatus) -> RepositoryResult<Vec<Race>>;
}
