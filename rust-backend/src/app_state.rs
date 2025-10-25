use mongodb::Database;
use std::sync::Arc;

use crate::services::{JwtService, SessionManager};

/// Shared application state containing all services
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub jwt_service: Arc<JwtService>,
    pub session_manager: Arc<SessionManager>,
}

impl AppState {
    pub fn new(
        database: Database,
        jwt_service: Arc<JwtService>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            database,
            jwt_service,
            session_manager,
        }
    }
}