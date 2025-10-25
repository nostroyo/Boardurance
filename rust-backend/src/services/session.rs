use chrono::{DateTime, Duration, Utc};
use mongodb::{bson::oid::ObjectId, Database};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::Duration as StdDuration,
};
use uuid::Uuid;

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_sessions_per_user: usize,
    pub session_timeout: StdDuration,
    pub blacklist_cleanup_interval: StdDuration,
    pub cache_size_limit: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_user: 5,
            session_timeout: StdDuration::from_secs(24 * 60 * 60), // 24 hours
            blacklist_cleanup_interval: StdDuration::from_secs(60 * 60), // 1 hour
            cache_size_limit: 10000,
        }
    }
}

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_uuid: Uuid,
    pub token_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
}

/// Blacklisted token data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistedToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub token_id: String,
    pub user_uuid: Uuid,
    pub blacklisted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reason: String,
}

/// Session metadata for creation
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// In-memory cache for sessions and blacklisted tokens
#[derive(Debug)]
struct SessionCache {
    // token_id -> Session (LRU cache with size limit)
    sessions: HashMap<String, Session>,
    // token_id for quick blacklist lookup
    blacklisted_tokens: HashSet<String>,
    // user_uuid -> Vec<token_id> for user session tracking
    user_sessions: HashMap<Uuid, Vec<String>>,
}

impl SessionCache {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            blacklisted_tokens: HashSet::new(),
            user_sessions: HashMap::new(),
        }
    }
}

/// Session manager with MongoDB and in-memory caching
pub struct SessionManager {
    database: Arc<Database>,
    cache: Arc<RwLock<SessionCache>>,
    config: SessionConfig,
}

/// Session manager errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),
    #[error("Session not found")]
    SessionNotFound,
    #[error("Token is blacklisted")]
    TokenBlacklisted,
    #[error("Session expired")]
    SessionExpired,
    #[error("Too many sessions for user")]
    TooManySessions,
    #[error("Cache error: {0}")]
    Cache(String),
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(database: Arc<Database>, config: SessionConfig) -> Self {
        Self {
            database,
            cache: Arc::new(RwLock::new(SessionCache::new())),
            config,
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        user_uuid: Uuid,
        token_id: String,
        metadata: SessionMetadata,
    ) -> Result<(), SessionError> {
        let now = Utc::now();
        let expires_at = now + Duration::from_std(self.config.session_timeout)
            .map_err(|e| SessionError::Cache(e.to_string()))?;

        // Check if user has too many sessions
        let user_session_count = self.get_user_session_count(user_uuid).await?;
        if user_session_count >= self.config.max_sessions_per_user {
            return Err(SessionError::TooManySessions);
        }

        let session = Session {
            id: None,
            user_uuid,
            token_id: token_id.clone(),
            created_at: now,
            last_activity: now,
            expires_at,
            ip_address: metadata.ip_address,
            user_agent: metadata.user_agent,
            is_active: true,
        };

        // Store in database
        let collection = self.database.collection::<Session>("sessions");
        collection.insert_one(&session, None).await?;

        // Cache the session
        self.cache_session(session)?;

        Ok(())
    }

    /// Validate a session by token ID
    pub async fn validate_session(&self, token_id: &str) -> Result<bool, SessionError> {
        // Check blacklist first (cache)
        if self.is_token_blacklisted_cached(token_id) {
            return Err(SessionError::TokenBlacklisted);
        }

        // Check cache first
        if let Some(session) = self.get_session_from_cache(token_id) {
            if session.expires_at < Utc::now() {
                return Err(SessionError::SessionExpired);
            }
            if !session.is_active {
                return Err(SessionError::SessionNotFound);
            }
            return Ok(true);
        }

        // Check database
        let collection = self.database.collection::<Session>("sessions");
        let session = collection
            .find_one(
                mongodb::bson::doc! {
                    "token_id": token_id,
                    "is_active": true
                },
                None,
            )
            .await?;

        match session {
            Some(session) => {
                if session.expires_at < Utc::now() {
                    return Err(SessionError::SessionExpired);
                }
                
                // Cache the session
                self.cache_session(session)?;
                Ok(true)
            }
            None => Err(SessionError::SessionNotFound),
        }
    }

    /// Invalidate a session
    pub async fn invalidate_session(&self, token_id: &str, reason: &str) -> Result<(), SessionError> {
        // Add to blacklist
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // Keep blacklist for 30 days

        let blacklisted_token = BlacklistedToken {
            id: None,
            token_id: token_id.to_string(),
            user_uuid: Uuid::new_v4(), // We'll need to get this from the session
            blacklisted_at: now,
            expires_at,
            reason: reason.to_string(),
        };

        // Store in database
        let blacklist_collection = self.database.collection::<BlacklistedToken>("blacklisted_tokens");
        blacklist_collection.insert_one(&blacklisted_token, None).await?;

        // Deactivate session in database
        let session_collection = self.database.collection::<Session>("sessions");
        session_collection
            .update_one(
                mongodb::bson::doc! { "token_id": token_id },
                mongodb::bson::doc! { "$set": { "is_active": false } },
                None,
            )
            .await?;

        // Update cache
        self.blacklist_token_in_cache(token_id.to_string());
        self.remove_session_from_cache(token_id);

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_all_user_sessions(&self, user_uuid: Uuid, reason: &str) -> Result<(), SessionError> {
        // Get all active sessions for the user
        let collection = self.database.collection::<Session>("sessions");
        let mut cursor = collection
            .find(
                mongodb::bson::doc! {
                    "user_uuid": user_uuid.to_string(),
                    "is_active": true
                },
                None,
            )
            .await?;

        let mut token_ids = Vec::new();
        while cursor.advance().await? {
            let session: Session = cursor.deserialize_current()?;
            token_ids.push(session.token_id);
        }

        // Invalidate each session
        for token_id in token_ids {
            self.invalidate_session(&token_id, reason).await?;
        }

        Ok(())
    }

    /// Check if a token is blacklisted
    pub async fn is_token_blacklisted(&self, token_id: &str) -> Result<bool, SessionError> {
        // Check cache first
        if self.is_token_blacklisted_cached(token_id) {
            return Ok(true);
        }

        // Check database
        let collection = self.database.collection::<BlacklistedToken>("blacklisted_tokens");
        let blacklisted = collection
            .find_one(
                mongodb::bson::doc! {
                    "token_id": token_id,
                    "expires_at": { "$gt": mongodb::bson::DateTime::now() }
                },
                None,
            )
            .await?;

        let is_blacklisted = blacklisted.is_some();
        
        // Cache the result
        if is_blacklisted {
            self.blacklist_token_in_cache(token_id.to_string());
        }

        Ok(is_blacklisted)
    }

    /// Cleanup expired sessions and blacklisted tokens
    pub async fn cleanup_expired_sessions(&self) -> Result<usize, SessionError> {
        let now = mongodb::bson::DateTime::now();
        
        // Cleanup expired sessions
        let session_collection = self.database.collection::<Session>("sessions");
        let session_result = session_collection
            .delete_many(
                mongodb::bson::doc! { "expires_at": { "$lt": now } },
                None,
            )
            .await?;

        // Cleanup expired blacklisted tokens
        let blacklist_collection = self.database.collection::<BlacklistedToken>("blacklisted_tokens");
        let blacklist_result = blacklist_collection
            .delete_many(
                mongodb::bson::doc! { "expires_at": { "$lt": now } },
                None,
            )
            .await?;

        // Clear cache to force refresh
        self.clear_cache();

        Ok((session_result.deleted_count + blacklist_result.deleted_count) as usize)
    }

    // Private helper methods
    fn cache_session(&self, session: Session) -> Result<(), SessionError> {
        let mut cache = self.cache.write()
            .map_err(|e| SessionError::Cache(format!("Failed to acquire write lock: {}", e)))?;

        // Check cache size limit
        if cache.sessions.len() >= self.config.cache_size_limit {
            // Simple eviction: remove oldest entries
            let oldest_token = cache.sessions.keys().next().cloned();
            if let Some(token) = oldest_token {
                cache.sessions.remove(&token);
            }
        }

        // Add to session cache
        cache.sessions.insert(session.token_id.clone(), session.clone());

        // Add to user sessions tracking
        cache.user_sessions
            .entry(session.user_uuid)
            .or_insert_with(Vec::new)
            .push(session.token_id);

        Ok(())
    }

    fn get_session_from_cache(&self, token_id: &str) -> Option<Session> {
        let cache = self.cache.read().ok()?;
        cache.sessions.get(token_id).cloned()
    }

    fn remove_session_from_cache(&self, token_id: &str) {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(session) = cache.sessions.remove(token_id) {
                // Remove from user sessions tracking
                if let Some(user_sessions) = cache.user_sessions.get_mut(&session.user_uuid) {
                    user_sessions.retain(|id| id != token_id);
                    if user_sessions.is_empty() {
                        cache.user_sessions.remove(&session.user_uuid);
                    }
                }
            }
        }
    }

    fn blacklist_token_in_cache(&self, token_id: String) {
        if let Ok(mut cache) = self.cache.write() {
            cache.blacklisted_tokens.insert(token_id);
        }
    }

    fn is_token_blacklisted_cached(&self, token_id: &str) -> bool {
        self.cache.read()
            .map(|cache| cache.blacklisted_tokens.contains(token_id))
            .unwrap_or(false)
    }

    fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.sessions.clear();
            cache.blacklisted_tokens.clear();
            cache.user_sessions.clear();
        }
    }

    async fn get_user_session_count(&self, user_uuid: Uuid) -> Result<usize, SessionError> {
        let collection = self.database.collection::<Session>("sessions");
        let count = collection
            .count_documents(
                mongodb::bson::doc! {
                    "user_uuid": user_uuid.to_string(),
                    "is_active": true,
                    "expires_at": { "$gt": mongodb::bson::DateTime::now() }
                },
                None,
            )
            .await?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mongodb::Client;


    // Mock database for testing without requiring MongoDB connection
    async fn create_mock_database() -> Database {
        // Create a mock database that won't actually connect
        // For unit tests, we'll focus on testing the logic that doesn't require DB
        let client = Client::with_uri_str("mongodb://mock:27017").await.unwrap();
        client.database("mock_test")
    }

    #[test]
    fn session_config_default_works() {
        let config = SessionConfig::default();
        assert_eq!(config.max_sessions_per_user, 5);
        assert_eq!(config.session_timeout, StdDuration::from_secs(24 * 60 * 60));
        assert_eq!(config.blacklist_cleanup_interval, StdDuration::from_secs(60 * 60));
        assert_eq!(config.cache_size_limit, 10000);
    }

    #[test]
    fn session_creation_struct_works() {
        let user_uuid = Uuid::new_v4();
        let token_id = "test_token_123".to_string();
        let now = Utc::now();
        
        let session = Session {
            id: None,
            user_uuid,
            token_id: token_id.clone(),
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::hours(24),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            is_active: true,
        };

        assert_eq!(session.user_uuid, user_uuid);
        assert_eq!(session.token_id, token_id);
        assert!(session.is_active);
        assert!(session.expires_at > now);
    }

    #[test]
    fn blacklisted_token_creation_works() {
        let user_uuid = Uuid::new_v4();
        let token_id = "blacklisted_token".to_string();
        let now = Utc::now();
        
        let blacklisted_token = BlacklistedToken {
            id: None,
            token_id: token_id.clone(),
            user_uuid,
            blacklisted_at: now,
            expires_at: now + Duration::days(30),
            reason: "test_reason".to_string(),
        };

        assert_eq!(blacklisted_token.token_id, token_id);
        assert_eq!(blacklisted_token.user_uuid, user_uuid);
        assert_eq!(blacklisted_token.reason, "test_reason");
        assert!(blacklisted_token.expires_at > now);
    }

    #[test]
    fn session_metadata_creation_works() {
        let metadata = SessionMetadata {
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
        };

        assert_eq!(metadata.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(metadata.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[tokio::test]
    async fn session_manager_creation_works() {
        let db = Arc::new(create_mock_database().await);
        let config = SessionConfig::default();
        let session_manager = SessionManager::new(db, config);
        
        // Should not panic and should be created successfully
        assert_eq!(session_manager.config.max_sessions_per_user, 5);
    }

    #[tokio::test]
    async fn session_cache_operations_work() {
        let db = Arc::new(create_mock_database().await);
        let config = SessionConfig::default();
        let session_manager = SessionManager::new(db, config);
        
        let user_uuid = Uuid::new_v4();
        let token_id = "cache_test_token".to_string();
        let now = Utc::now();
        
        let session = Session {
            id: None,
            user_uuid,
            token_id: token_id.clone(),
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::hours(24),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            is_active: true,
        };

        // Test caching
        assert!(session_manager.cache_session(session.clone()).is_ok());
        
        // Test retrieval from cache
        let cached_session = session_manager.get_session_from_cache(&token_id);
        assert!(cached_session.is_some());
        assert_eq!(cached_session.unwrap().token_id, token_id);
        
        // Test blacklisting in cache
        session_manager.blacklist_token_in_cache(token_id.clone());
        assert!(session_manager.is_token_blacklisted_cached(&token_id));
        
        // Test removal from cache
        session_manager.remove_session_from_cache(&token_id);
        let cached_session_after_removal = session_manager.get_session_from_cache(&token_id);
        assert!(cached_session_after_removal.is_none());
    }

    #[test]
    fn session_error_types_work() {
        let db_error = SessionError::Database(mongodb::error::Error::custom("test error"));
        assert!(matches!(db_error, SessionError::Database(_)));
        
        let not_found_error = SessionError::SessionNotFound;
        assert!(matches!(not_found_error, SessionError::SessionNotFound));
        
        let blacklisted_error = SessionError::TokenBlacklisted;
        assert!(matches!(blacklisted_error, SessionError::TokenBlacklisted));
        
        let expired_error = SessionError::SessionExpired;
        assert!(matches!(expired_error, SessionError::SessionExpired));
        
        let too_many_error = SessionError::TooManySessions;
        assert!(matches!(too_many_error, SessionError::TooManySessions));
        
        let cache_error = SessionError::Cache("test cache error".to_string());
        assert!(matches!(cache_error, SessionError::Cache(_)));
    }

    // Integration tests that require MongoDB should be in a separate integration test file
    // For now, we'll focus on unit tests that test the logic without database dependencies
}