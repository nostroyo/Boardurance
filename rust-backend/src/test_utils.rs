//! Test utilities for creating mock-based test applications
//! This module provides infrastructure for testing without requiring a real MongoDB instance

use crate::repositories::{MockPlayerRepository, MockRaceRepository, MockSessionRepository};
use crate::services::{JwtConfig, JwtService, SessionConfig, SessionManager};
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Test application state using mock repositories
#[derive(Clone)]
pub struct TestAppState {
    pub player_repo: Arc<MockPlayerRepository>,
    pub race_repo: Arc<MockRaceRepository>,
    pub session_repo: Arc<MockSessionRepository>,
    pub jwt_service: Arc<JwtService>,
    pub session_manager: Arc<SessionManager<MockSessionRepository>>,
}

impl TestAppState {
    pub fn new() -> Self {
        let player_repo = Arc::new(MockPlayerRepository::new());
        let race_repo = Arc::new(MockRaceRepository::new());
        let session_repo = Arc::new(MockSessionRepository::new());

        // Initialize JWT service with test configuration
        let jwt_config = JwtConfig {
            secret: "test-secret-key-for-testing-only".to_string(),
            access_token_expiry: std::time::Duration::from_secs(30 * 60), // 30 minutes
            refresh_token_expiry: std::time::Duration::from_secs(30 * 24 * 60 * 60), // 30 days
            issuer: "racing-game-test".to_string(),
            audience: "racing-game-test-client".to_string(),
        };
        let jwt_service = Arc::new(JwtService::new(jwt_config));

        // Initialize session manager with mock repository
        let session_config = SessionConfig::default();
        let session_manager = Arc::new(SessionManager::new(
            session_repo.clone(),
            session_config,
        ));

        Self {
            player_repo,
            race_repo,
            session_repo,
            jwt_service,
            session_manager,
        }
    }

    /// Create test app state with pre-populated data
    pub fn with_test_data(
        players: Vec<crate::domain::Player>,
        races: Vec<crate::domain::Race>,
        sessions: Vec<crate::services::session::Session>,
    ) -> Self {
        let player_repo = Arc::new(MockPlayerRepository::with_players(players));
        let race_repo = Arc::new(MockRaceRepository::with_races(races));
        let session_repo = Arc::new(MockSessionRepository::with_sessions(sessions));

        let jwt_config = JwtConfig {
            secret: "test-secret-key-for-testing-only".to_string(),
            access_token_expiry: std::time::Duration::from_secs(30 * 60),
            refresh_token_expiry: std::time::Duration::from_secs(30 * 24 * 60 * 60),
            issuer: "racing-game-test".to_string(),
            audience: "racing-game-test-client".to_string(),
        };
        let jwt_service = Arc::new(JwtService::new(jwt_config));

        let session_config = SessionConfig::default();
        let session_manager = Arc::new(SessionManager::new(
            session_repo.clone(),
            session_config,
        ));

        Self {
            player_repo,
            race_repo,
            session_repo,
            jwt_service,
            session_manager,
        }
    }
}

impl Default for TestAppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Test application builder for integration tests
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub state: TestAppState,
}

impl TestApp {
    /// Create a new test application with mock repositories
    pub async fn new() -> Self {
        let state = TestAppState::new();
        let app = create_test_router(&state).await;
        
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{port}");

        let server = axum::serve(listener, app);
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(async move { server.await.expect("Server failed to start") });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        Self {
            address,
            client,
            state,
        }
    }

    /// Create a test application with pre-populated test data
    pub async fn with_test_data(
        players: Vec<crate::domain::Player>,
        races: Vec<crate::domain::Race>,
        sessions: Vec<crate::services::session::Session>,
    ) -> Self {
        let state = TestAppState::with_test_data(players, races, sessions);
        let app = create_test_router(&state).await;
        
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{port}");

        let server = axum::serve(listener, app);
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(async move { server.await.expect("Server failed to start") });

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let client = reqwest::Client::new();

        Self {
            address,
            client,
            state,
        }
    }

    // Helper methods for common test operations
    pub async fn post_json(&self, path: &str, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(format!("{}{}", &self.address, path))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", &self.address, path))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn get_with_cookies(&self, path: &str, cookies: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", &self.address, path))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn extract_cookies(response: &reqwest::Response) -> String {
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Create a test router using mock repositories instead of real database
async fn create_test_router(_state: &TestAppState) -> Router {
    use crate::routes::{health_check};
    use axum::{routing::get, Router};
    use axum::http::Method;
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;

    // For now, create a simple router with health check
    // TODO: Add routes that use the mock repositories when needed
    Router::new()
        .route("/health_check", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse().unwrap(),
                    "http://localhost:5174".parse().unwrap(),
                    "http://localhost:5175".parse().unwrap(),
                ])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
}