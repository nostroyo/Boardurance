use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::services::session::Session;
use super::{RepositoryResult};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> RepositoryResult<()>;
    async fn find_by_token(&self, token: &str) -> RepositoryResult<Option<Session>>;
    async fn deactivate(&self, token: &str) -> RepositoryResult<()>;
    async fn deactivate_all_for_user(&self, user_uuid: Uuid) -> RepositoryResult<()>;
    async fn cleanup_expired(&self, now: DateTime<Utc>) -> RepositoryResult<u64>;
    async fn count_active_for_user(&self, user_uuid: Uuid) -> RepositoryResult<usize>;
}