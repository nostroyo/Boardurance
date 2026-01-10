//! Authorization Integration Tests
//! These tests verify ownership validation, role-based access control,
//! and admin privileges across different endpoints.

use rust_backend::configuration::get_configuration;
use rust_backend::startup::{get_connection_pool, run};
use rust_backend::telemetry::{get_subscriber, init_subscriber};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use uuid::Uuid;

// Ensure that the `tracing` stack is only initialised once using `std::sync::Once`
static TRACING: std::sync::Once = std::sync::Once::new();

struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
}

impl TestApp {
    // Authentication helpers
    pub async fn post_register(&self, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(format!("{}/api/v1/auth/register", &self.address))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // Player endpoint helpers
    pub async fn get_player(&self, uuid: &str, cookies: &str) -> reqwest::Response {
        self.client
            .get(format!("{}/api/v1/players/{}", &self.address, uuid))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_player_with_auth_header(&self, uuid: &str, token: &str) -> reqwest::Response {
        self.client
            .get(format!("{}/api/v1/players/{}", &self.address, uuid))
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_all_players(&self, cookies: &str) -> reqwest::Response {
        self.client
            .get(format!("{}/api/v1/players", &self.address))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // Helper to extract cookies from response headers
    pub fn extract_cookies(response: &reqwest::Response) -> String {
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("; ")
    }

    // Helper to create a test user and return their UUID and cookies
    pub async fn create_test_user(
        &self,
        email: &str,
        password: &str,
        team_name: &str,
    ) -> (String, String) {
        let register_body = json!({
            "email": email,
            "password": password,
            "team_name": team_name
        });

        let response = self.post_register(&register_body).await;
        assert_eq!(201, response.status().as_u16());

        let cookies = TestApp::extract_cookies(&response);
        let response_body: Value = response.json().await.expect("Failed to parse response");
        let user_uuid = response_body["user"]["uuid"].as_str().unwrap().to_string();

        (user_uuid, cookies)
    }

    // Helper to create an admin user (would need to be implemented in the backend)
    pub async fn create_admin_user(
        &self,
        email: &str,
        password: &str,
        team_name: &str,
    ) -> (String, String) {
        // For now, this creates a regular user. In a real implementation,
        // we would need a way to create admin users or promote users to admin
        self.create_test_user(email, password, team_name).await
    }
}

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    TRACING.call_once(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let base_url = "http://127.0.0.1".to_string();

    // Get database connection
    let database = get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to database");

    let server = run(listener, database, base_url)
        .await
        .expect("Failed to build application.");
    let _ = tokio::spawn(async move { server.await.expect("Server failed to start") });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    TestApp { address, client }
}

// ============================================================================
// OWNERSHIP VALIDATION TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will pass once RequireOwnership middleware is applied"]
async fn player_can_access_own_resources() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - User tries to access their own player resource
    let response = app.get_player(&user_uuid, &cookies).await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["uuid"], user_uuid);
    assert_eq!(response_body["email"], "user@example.com");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 403 once RequireOwnership middleware is applied"]
async fn player_cannot_access_other_player_resources() {
    // Arrange
    let app = spawn_app().await;
    let (_user1_uuid, user1_cookies) = app
        .create_test_user("user1@example.com", "Password123", "User 1")
        .await;
    let (user2_uuid, _user2_cookies) = app
        .create_test_user("user2@example.com", "Password123", "User 2")
        .await;

    // Act - User 1 tries to access User 2's player resource
    let response = app.get_player(&user2_uuid, &user1_cookies).await;

    // Assert - Should be forbidden
    assert_eq!(403, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(
        response_body["error"],
        "Access denied: insufficient permissions"
    );
}

// ============================================================================
// ROLE-BASED ACCESS CONTROL TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will pass once RequireRole middleware is applied"]
async fn admin_can_access_any_player_resources() {
    // Arrange
    let app = spawn_app().await;
    let (_admin_uuid, admin_cookies) = app
        .create_admin_user("admin@example.com", "Password123", "Admin Team")
        .await;
    let (user_uuid, _user_cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Admin tries to access regular user's resource
    let response = app.get_player(&user_uuid, &admin_cookies).await;

    // Assert - Should be allowed
    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["uuid"], user_uuid);
    assert_eq!(response_body["email"], "user@example.com");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 403 once RequireRole middleware is applied"]
async fn regular_user_cannot_access_admin_only_routes() {
    // Arrange
    let app = spawn_app().await;
    let (_user_uuid, user_cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Regular user tries to access admin-only route (get all players)
    let response = app.get_all_players(&user_cookies).await;

    // Assert - Should be forbidden
    assert_eq!(403, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Access denied: admin role required");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will pass once RequireRole middleware is applied"]
async fn admin_can_access_admin_only_routes() {
    // Arrange
    let app = spawn_app().await;
    let (_admin_uuid, admin_cookies) = app
        .create_admin_user("admin@example.com", "Password123", "Admin Team")
        .await;
    let (_user_uuid, _user_cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Admin tries to access admin-only route (get all players)
    let response = app.get_all_players(&admin_cookies).await;

    // Assert - Should be allowed
    assert_eq!(200, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(response_body.is_array());
    assert!(response_body.as_array().unwrap().len() >= 2); // At least admin and user
}

// ============================================================================
// AUTHENTICATION REQUIREMENT TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once AuthMiddleware is applied"]
async fn unauthenticated_access_to_protected_routes_blocked() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, _user_cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Try to access protected route without authentication
    let response = app
        .client
        .get(format!("{}/api/v1/players/{}", &app.address, user_uuid))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Authentication required");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once AuthMiddleware is applied"]
async fn invalid_token_access_blocked() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, _user_cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Try to access protected route with invalid token
    let response = app
        .get_player_with_auth_header(&user_uuid, "invalid.jwt.token")
        .await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Invalid token");
}


