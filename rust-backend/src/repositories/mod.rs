pub mod player_repository;
pub mod race_repository;
pub mod session_repository;

#[cfg(test)]
pub mod mocks;

pub use player_repository::{PlayerRepository, MongoPlayerRepository};
pub use race_repository::{RaceRepository, MongoRaceRepository};
pub use session_repository::{SessionRepository, MongoSessionRepository};

#[cfg(test)]
pub use mocks::{MockPlayerRepository, MockRaceRepository, MockSessionRepository};

use async_trait::async_trait;
use mongodb::error::Error as MongoError;

/// Common database error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] MongoError),
    #[error("Entity not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;