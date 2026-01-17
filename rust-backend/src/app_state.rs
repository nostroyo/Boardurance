use std::sync::Arc;

use crate::repositories::{PlayerRepository, RaceRepository, SessionRepository};
use crate::services::JwtService;

/// Application state that holds shared services
#[derive(Clone)]
pub struct AppState<P: PlayerRepository, R: RaceRepository, S: SessionRepository> {
    pub player_repository: Arc<P>,
    pub race_repository: Arc<R>,
    pub session_repository: Arc<S>,
    pub jwt_service: Arc<JwtService>,
}

impl<P: PlayerRepository, R: RaceRepository, S: SessionRepository> AppState<P, R, S> {
    #[must_use]
    pub fn new(
        player_repository: Arc<P>,
        race_repository: Arc<R>,
        session_repository: Arc<S>,
        jwt_service: Arc<JwtService>,
    ) -> Self {
        Self {
            player_repository,
            race_repository,
            session_repository,
            jwt_service,
        }
    }
}
