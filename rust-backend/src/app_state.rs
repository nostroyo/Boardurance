use std::sync::Arc;

use mongodb::Database;

use crate::repositories::{PlayerRepository, RaceRepository, SessionRepository};
use crate::services::{JwtService, SessionManager};

/// Application state that holds shared services.
///
/// Repositories are held as trait objects so the same route set can run
/// against either the in-memory `Mock*` repositories (local/test) or the
/// `Mongo*` repositories (prod/preprod) selected at startup — see
/// `startup.rs::run` and `configuration::DatabaseSettings::resolved_storage_backend`.
#[derive(Clone)]
pub struct AppState {
    pub player_repository: Arc<dyn PlayerRepository>,
    pub race_repository: Arc<dyn RaceRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub jwt_service: Arc<JwtService>,
    pub session_manager: Arc<SessionManager>,
    /// Raw Mongo handle, kept alongside the repository abstractions for the
    /// few call sites (e.g. `CarValidationService`) that query collections
    /// directly rather than through a `*Repository` trait. Cheap to clone
    /// (an `Arc`-backed handle internally).
    pub database: Database,
}

impl AppState {
    #[must_use]
    pub fn new(
        player_repository: Arc<dyn PlayerRepository>,
        race_repository: Arc<dyn RaceRepository>,
        session_repository: Arc<dyn SessionRepository>,
        jwt_service: Arc<JwtService>,
        session_manager: Arc<SessionManager>,
        database: Database,
    ) -> Self {
        Self {
            player_repository,
            race_repository,
            session_repository,
            jwt_service,
            session_manager,
            database,
        }
    }
}
