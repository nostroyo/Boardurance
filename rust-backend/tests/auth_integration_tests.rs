//! Integration tests for authentication endpoints
//! These tests verify the basic authentication functionality including
//! user registration and login.

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
    pub db_name: String,
    pub client: reqwest::Client,
}

impl TestApp {
    pub async fn post_register(&self, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/v1/auth/register", &self.address))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login(&self, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(&format!("{}/api/v1/auth/login", &self.address))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
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

    // Create and migrate the database
    let database = get_connection_pool(&configuration.database).await
        .expect("Failed to connect to database");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let server = run(listener, database, configuration.application.base_url)
        .await
        .expect("Failed to build application.");
    let _ = tokio::spawn(async move {
        server.await.expect("Server failed to start")
    });

    let client = reqwest::Client::new();

    TestApp {
        address,
        db_name: configuration.database.database_name,
        client,
    }
}

#[tokio::test]
async fn register_returns_201_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = json!({
        "email": "test@example.com",
        "password": "password123",
        "team_name": "Test Team"
    });

    // Act
    let response = app.post_register(&body).await;

    // Assert
    assert_eq!(201, response.status().as_u16());
    
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["message"], "User registered successfully");
    assert!(response_body["user"]["uuid"].is_string());
    assert_eq!(response_body["user"]["email"], "test@example.com");
    assert_eq!(response_body["user"]["team_name"], "Test Team");
    assert_eq!(response_body["user"]["role"], "Player");
}

#[tokio::test]
async fn register_returns_400_for_invalid_email() {
    // Arrange
    let app = spawn_app().await;
    let body = json!({
        "email": "invalid-email",
        "password": "password123",
        "team_name": "Test Team"
    });

    // Act
    let response = app.post_register(&body).await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn register_returns_400_for_weak_password() {
    // Arrange
    let app = spawn_app().await;
    let body = json!({
        "email": "test@example.com",
        "password": "123", // Too short
        "team_name": "Test Team"
    });

    // Act
    let response = app.post_register(&body).await;

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn register_returns_409_for_duplicate_email() {
    // Arrange
    let app = spawn_app().await;
    let body = json!({
        "email": "test@example.com",
        "password": "password123",
        "team_name": "Test Team"
    });

    // Act - Register first user
    let _response1 = app.post_register(&body).await;
    
    // Act - Try to register same email again
    let response2 = app.post_register(&body).await;

    // Assert
    assert_eq!(409, response2.status().as_u16());
    
    let response_body: Value = response2.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "User with this email already exists");
}

#[tokio::test]
async fn login_returns_200_for_valid_credentials() {
    // Arrange
    let app = spawn_app().await;
    let register_body = json!({
        "email": "test@example.com",
        "password": "password123",
        "team_name": "Test Team"
    });
    
    // Register user first
    let _register_response = app.post_register(&register_body).await;
    
    let login_body = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    // Act
    let response = app.post_login(&login_body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
    
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["message"], "Login successful");
    assert!(response_body["user"]["uuid"].is_string());
    assert_eq!(response_body["user"]["email"], "test@example.com");
    assert_eq!(response_body["user"]["team_name"], "Test Team");
}

#[tokio::test]
async fn login_returns_401_for_invalid_credentials() {
    // Arrange
    let app = spawn_app().await;
    let register_body = json!({
        "email": "test@example.com",
        "password": "password123",
        "team_name": "Test Team"
    });
    
    // Register user first
    let _register_response = app.post_register(&register_body).await;
    
    let login_body = json!({
        "email": "test@example.com",
        "password": "wrongpassword"
    });

    // Act
    let response = app.post_login(&login_body).await;

    // Assert
    assert_eq!(401, response.status().as_u16());
    
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Invalid credentials");
}

#[tokio::test]
async fn login_returns_401_for_nonexistent_user() {
    // Arrange
    let app = spawn_app().await;
    let login_body = json!({
        "email": "nonexistent@example.com",
        "password": "password123"
    });

    // Act
    let response = app.post_login(&login_body).await;

    // Assert
    assert_eq!(401, response.status().as_u16());
    
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Invalid credentials");
}

#[tokio::test]
async fn basic_authentication_flow_works() {
    // Arrange
    let app = spawn_app().await;
    let register_body = json!({
        "email": "test@example.com",
        "password": "password123",
        "team_name": "Test Team"
    });

    // Act & Assert - Register
    let register_response = app.post_register(&register_body).await;
    assert_eq!(201, register_response.status().as_u16());
    
    let register_response_body: Value = register_response.json().await.expect("Failed to parse response");
    let user_uuid = register_response_body["user"]["uuid"].as_str().unwrap();

    // Act & Assert - Login with same credentials
    let login_body = json!({
        "email": "test@example.com",
        "password": "password123"
    });
    
    let login_response = app.post_login(&login_body).await;
    assert_eq!(200, login_response.status().as_u16());
    
    let login_response_body: Value = login_response.json().await.expect("Failed to parse response");
    assert_eq!(login_response_body["user"]["uuid"], user_uuid);
    assert_eq!(login_response_body["message"], "Login successful");
}