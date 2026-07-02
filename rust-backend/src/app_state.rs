use std::sync::Arc;

use crate::repositories::{PlayerRepository, RaceRepository, SessionRepository};
use crate::services::{JwtService, SessionManager};

/// Application state that holds shared services.
///
/// Repositories are held as trait objects so the same route set can run
/// against either the in-memory `Mock*` repositories (local/test) or the
/// `Mongo*` repositories (prod/preprod) selected at startup — see
/// `startup.rs::run` and `configuration::DatabaseSettings::resolved_storage_backend`.
///
/// Deliberately has no raw `Database` handle: every data access goes through
/// one of the `*Repository` traits, so backend selection (mock vs Mongo) is
/// respected everywhere instead of being bypassable per call site.
#[derive(Clone)]
pub struct AppState {
    pub player_repository: Arc<dyn PlayerRepository>,
    pub race_repository: Arc<dyn RaceRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub jwt_service: Arc<JwtService>,
    pub session_manager: Arc<SessionManager>,
}

impl AppState {
    #[must_use]
    pub fn new(
        player_repository: Arc<dyn PlayerRepository>,
        race_repository: Arc<dyn RaceRepository>,
        session_repository: Arc<dyn SessionRepository>,
        jwt_service: Arc<JwtService>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            player_repository,
            race_repository,
            session_repository,
            jwt_service,
            session_manager,
        }
    }
}
