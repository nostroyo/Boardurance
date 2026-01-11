use mongodb::Database;
use std::sync::Arc;

use crate::repositories::{MongoPlayerRepository, MongoRaceRepository, MongoSessionRepository};
use crate::services::{JwtService, SessionManager};

/// Application state that holds shared services
#[derive(Clone)]
pub struct AppState {
    pub database: Option<Database>,
    pub player_repository: Arc<MongoPlayerRepository>,
    pub race_repository: Arc<MongoRaceRepository>,
    pub session_repository: Arc<MongoSessionRepository>,
    pub jwt_service: Arc<JwtService>,
    pub session_manager: Arc<SessionManager<MongoSessionRepository>>,
}

impl AppState {
    #[must_use]
    pub fn new(
        database: Database,
        jwt_service: Arc<JwtService>,
        session_manager: Arc<SessionManager<MongoSessionRepository>>,
    ) -> Self {
        let player_repository = Arc::new(MongoPlayerRepository::new(database.clone()));
        let race_repository = Arc::new(MongoRaceRepository::new(database.clone()));
        let session_repository = Arc::new(MongoSessionRepository::new(database.clone()));

        Self {
            database: Some(database),
            player_repository,
            race_repository,
            session_repository,
            jwt_service,
            session_manager,
        }
    }
}
