use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mongodb::{bson::doc, Database};
use uuid::Uuid;

use crate::services::session::Session;
use super::{RepositoryError, RepositoryResult};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> RepositoryResult<()>;
    async fn find_by_token(&self, token: &str) -> RepositoryResult<Option<Session>>;
    async fn deactivate(&self, token: &str) -> RepositoryResult<()>;
    async fn deactivate_all_for_user(&self, user_uuid: Uuid) -> RepositoryResult<()>;
    async fn cleanup_expired(&self, now: DateTime<Utc>) -> RepositoryResult<u64>;
    async fn count_active_for_user(&self, user_uuid: Uuid) -> RepositoryResult<usize>;
}

pub struct MongoSessionRepository {
    database: Database,
}

impl MongoSessionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl SessionRepository for MongoSessionRepository {
    async fn create(&self, session: &Session) -> RepositoryResult<()> {
        let collection = self.database.collection::<Session>("sessions");
        collection.insert_one(session, None).await?;
        Ok(())
    }

    async fn find_by_token(&self, token: &str) -> RepositoryResult<Option<Session>> {
        let collection = self.database.collection::<Session>("sessions");
        let filter = doc! {
            "token": token,
            "is_active": true,
            "expires_at": { "$gt": mongodb::bson::DateTime::now() }
        };
        Ok(collection.find_one(filter, None).await?)
    }

    async fn deactivate(&self, token: &str) -> RepositoryResult<()> {
        let collection = self.database.collection::<Session>("sessions");
        let filter = doc! { "token": token };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": mongodb::bson::DateTime::now()
            }
        };
        
        collection.update_one(filter, update, None).await?;
        Ok(())
    }

    async fn deactivate_all_for_user(&self, user_uuid: Uuid) -> RepositoryResult<()> {
        let collection = self.database.collection::<Session>("sessions");
        let filter = doc! {
            "user_uuid": user_uuid.to_string(),
            "is_active": true
        };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": mongodb::bson::DateTime::now()
            }
        };
        
        collection.update_many(filter, update, None).await?;
        Ok(())
    }

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> RepositoryResult<u64> {
        let collection = self.database.collection::<Session>("sessions");
        let filter = doc! { "expires_at": { "$lt": mongodb::bson::DateTime::from_chrono(now) } };
        let result = collection.delete_many(filter, None).await?;
        Ok(result.deleted_count)
    }

    async fn count_active_for_user(&self, user_uuid: Uuid) -> RepositoryResult<usize> {
        let collection = self.database.collection::<Session>("sessions");
        let filter = doc! {
            "user_uuid": user_uuid.to_string(),
            "is_active": true,
            "expires_at": { "$gt": mongodb::bson::DateTime::now() }
        };
        let count = collection.count_documents(filter, None).await?;
        Ok(count as usize)
    }
}