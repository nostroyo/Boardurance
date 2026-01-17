pub mod player_repository;
pub mod race_repository;
pub mod session_repository;

pub mod mocks;

pub use player_repository::PlayerRepository;
pub use race_repository::RaceRepository;
pub use session_repository::SessionRepository;

pub use mocks::{MockPlayerRepository, MockRaceRepository, MockSessionRepository};

/// Common database error type
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found")]
    NotFound,
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
}

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;