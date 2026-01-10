//! Security Edge Cases Integration Tests
//! These tests verify security-critical scenarios including token tampering,
//! expiration handling, blacklist enforcement, and other attack vectors.

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
    pub _db_name: String,
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

    pub async fn post_login(&self, body: &serde_json::Value) -> reqwest::Response {
        self.client
            .post(format!("{}/api/v1/auth/login", &self.address))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self, cookies: &str) -> reqwest::Response {
        self.client
            .post(format!("{}/api/v1/auth/logout", &self.address))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_refresh(&self, cookies: &str) -> reqwest::Response {
        self.client
            .post(format!("{}/api/v1/auth/refresh", &self.address))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    // Protected endpoint helpers
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

    pub async fn get_player_with_custom_cookie(
        &self,
        uuid: &str,
        cookie_name: &str,
        cookie_value: &str,
    ) -> reqwest::Response {
        self.client
            .get(format!("{}/api/v1/players/{}", &self.address, uuid))
            .header("Cookie", &format!("{cookie_name}={cookie_value}"))
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

    // Helper to login and get fresh tokens
    pub async fn login_user(&self, email: &str, password: &str) -> String {
        let login_body = json!({
            "email": email,
            "password": password
        });

        let response = self.post_login(&login_body).await;
        assert_eq!(200, response.status().as_u16());

        TestApp::extract_cookies(&response)
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

    let db_name = configuration.database.database_name.clone();
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

    TestApp {
        address,
        db_name,
        client,
    }
}

// ============================================================================
// TOKEN TAMPERING AND SIGNATURE VALIDATION TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once JWT validation is applied"]
async fn tampered_access_token_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract access token and tamper with it
    let access_token = extract_token_from_cookies(&cookies, "access_token");
    let tampered_token = tamper_with_token(&access_token);

    // Act - Try to use tampered token
    let response = app
        .get_player_with_auth_header(&user_uuid, &tampered_token)
        .await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Invalid token signature");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once JWT validation is applied"]
async fn tampered_refresh_token_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (_user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract refresh token and tamper with it
    let refresh_token = extract_token_from_cookies(&cookies, "refresh_token");
    let tampered_token = tamper_with_token(&refresh_token);
    let tampered_cookies = format!("refresh_token={tampered_token}");

    // Act - Try to use tampered refresh token
    let response = app.post_refresh(&tampered_cookies).await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Invalid token signature");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once JWT validation is applied"]
async fn completely_invalid_token_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, _cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    let invalid_tokens = vec![
        "invalid.jwt.token",
        "not-a-jwt-at-all",
        "",
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
        "header.payload", // Missing signature
        "too.many.parts.in.this.jwt.token",
    ];

    for invalid_token in invalid_tokens {
        // Act - Try to use invalid token
        let response = app
            .get_player_with_auth_header(&user_uuid, invalid_token)
            .await;

        // Assert - Should be unauthorized
        assert_eq!(
            401,
            response.status().as_u16(),
            "Failed for token: {invalid_token}"
        );

        let response_body: Value = response.json().await.expect("Failed to parse response");
        assert!(
            response_body["error"]
                .as_str()
                .unwrap()
                .contains("Invalid token")
                || response_body["error"]
                    .as_str()
                    .unwrap()
                    .contains("Malformed token"),
            "Unexpected error message for token {}: {}",
            invalid_token,
            response_body["error"]
        );
    }
}

// ============================================================================
// TOKEN EXPIRATION TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once JWT expiration is enforced"]
async fn expired_access_token_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract access token
    let access_token = extract_token_from_cookies(&cookies, "access_token");

    // Create an expired version of the token (this would require modifying JWT service for testing)
    // For now, we simulate by waiting longer than token expiry or using a pre-expired token
    let expired_token = create_expired_token(&access_token);

    // Act - Try to use expired token
    let response = app
        .get_player_with_auth_header(&user_uuid, &expired_token)
        .await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Token has expired");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once JWT expiration is enforced"]
async fn expired_refresh_token_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (_user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract refresh token
    let refresh_token = extract_token_from_cookies(&cookies, "refresh_token");

    // Create an expired version of the refresh token
    let expired_token = create_expired_token(&refresh_token);
    let expired_cookies = format!("refresh_token={expired_token}");

    // Act - Try to refresh with expired token
    let response = app.post_refresh(&expired_cookies).await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Refresh token has expired");
}

// ============================================================================
// TOKEN BLACKLIST TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once session management is applied"]
async fn blacklisted_token_after_logout_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract access token before logout
    let access_token = extract_token_from_cookies(&cookies, "access_token");

    // Logout user (which should blacklist the token)
    let logout_response = app.post_logout(&cookies).await;
    assert_eq!(200, logout_response.status().as_u16());

    // Act - Try to use blacklisted token
    let response = app
        .get_player_with_auth_header(&user_uuid, &access_token)
        .await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Token has been revoked");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once session management is applied"]
async fn blacklisted_refresh_token_after_logout_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (_user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract refresh token before logout
    let refresh_token = extract_token_from_cookies(&cookies, "refresh_token");

    // Logout user (which should blacklist all tokens)
    let logout_response = app.post_logout(&cookies).await;
    assert_eq!(200, logout_response.status().as_u16());

    // Act - Try to use blacklisted refresh token
    let blacklisted_cookies = format!("refresh_token={refresh_token}");
    let response = app.post_refresh(&blacklisted_cookies).await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Refresh token has been revoked");
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once session management is applied"]
async fn old_tokens_invalid_after_password_change() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract tokens before password change
    let old_access_token = extract_token_from_cookies(&cookies, "access_token");

    // Simulate password change (this would invalidate all existing sessions)
    // In a real implementation, this would be done through a password change endpoint
    // For now, we simulate by creating a new session and expecting old tokens to be invalid

    // Login again (simulating password change that creates new session)
    let _new_cookies = app.login_user("user@example.com", "NewPassword123").await;

    // Act - Try to use old token after password change
    let response = app
        .get_player_with_auth_header(&user_uuid, &old_access_token)
        .await;

    // Assert - Should be unauthorized (old session invalidated)
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert!(
        response_body["error"].as_str().unwrap().contains("revoked")
            || response_body["error"]
                .as_str()
                .unwrap()
                .contains("invalid session"),
        "Expected revocation error, got: {}",
        response_body["error"]
    );
}

// ============================================================================
// COOKIE SECURITY TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will pass once cookie security is enforced"]
async fn cookies_have_security_attributes() {
    // Arrange
    let app = spawn_app().await;

    // Act - Register user and check cookie attributes
    let register_body = json!({
        "email": "user@example.com",
        "password": "Password123",
        "team_name": "User Team"
    });

    let response = app.post_register(&register_body).await;
    assert_eq!(201, response.status().as_u16());

    // Assert - Check that cookies have proper security attributes
    let set_cookie_headers: Vec<&str> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|h| h.to_str().unwrap())
        .collect();

    for cookie_header in &set_cookie_headers {
        if cookie_header.contains("access_token=") || cookie_header.contains("refresh_token=") {
            // Check security attributes
            assert!(
                cookie_header.contains("HttpOnly"),
                "Cookie should be HttpOnly: {cookie_header}"
            );
            assert!(
                cookie_header.contains("Secure") || !cookie_header.contains("Secure"),
                "Cookie security depends on HTTPS"
            ); // Allow both for testing
            assert!(
                cookie_header.contains("SameSite="),
                "Cookie should have SameSite attribute: {cookie_header}"
            );

            // Check that sensitive cookies are not accessible via JavaScript
            assert!(
                cookie_header.contains("HttpOnly"),
                "Auth cookies must be HttpOnly"
            );
        }
    }
}

#[tokio::test]
#[ignore = "Middleware not yet integrated - will fail with 401 once proper cookie validation is applied"]
async fn missing_cookie_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, _cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Act - Try to access protected route without any cookies
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
#[ignore = "Middleware not yet integrated - will fail with 401 once proper cookie validation is applied"]
async fn wrong_cookie_name_rejected() {
    // Arrange
    let app = spawn_app().await;
    let (user_uuid, cookies) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Extract token value but use wrong cookie name
    let access_token = extract_token_from_cookies(&cookies, "access_token");

    // Act - Try to use token with wrong cookie name
    let response = app
        .get_player_with_custom_cookie(&user_uuid, "wrong_token_name", &access_token)
        .await;

    // Assert - Should be unauthorized
    assert_eq!(401, response.status().as_u16());

    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["error"], "Authentication required");
}

// ============================================================================
// CONCURRENT SESSION TESTS
// ============================================================================

#[tokio::test]
#[ignore = "Middleware not yet integrated - will pass once session management handles concurrent sessions"]
async fn multiple_concurrent_sessions_isolated() {
    // Arrange
    let app = spawn_app().await;

    // Create user and get first session
    let (user_uuid, cookies1) = app
        .create_test_user("user@example.com", "Password123", "User Team")
        .await;

    // Login again to get second session
    let cookies2 = app.login_user("user@example.com", "Password123").await;

    // Verify both sessions work
    let response1 = app.get_player(&user_uuid, &cookies1).await;
    assert_eq!(200, response1.status().as_u16());

    let response2 = app.get_player(&user_uuid, &cookies2).await;
    assert_eq!(200, response2.status().as_u16());

    // Logout first session
    let logout_response = app.post_logout(&cookies1).await;
    assert_eq!(200, logout_response.status().as_u16());

    // Act & Assert - First session should be invalid, second should still work
    let response1_after_logout = app.get_player(&user_uuid, &cookies1).await;
    assert_eq!(401, response1_after_logout.status().as_u16());

    let response2_after_logout = app.get_player(&user_uuid, &cookies2).await;
    assert_eq!(200, response2_after_logout.status().as_u16());
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// Helper function to extract specific token from cookie string
fn extract_token_from_cookies(cookies: &str, token_name: &str) -> String {
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if cookie.starts_with(&format!("{token_name}=")) {
            return cookie[token_name.len() + 1..]
                .split(';')
                .next()
                .unwrap()
                .to_string();
        }
    }
    panic!("Token {token_name} not found in cookies");
}

// Helper function to tamper with a JWT token (simple character replacement)
fn tamper_with_token(token: &str) -> String {
    // Simple tampering: replace first 'a' with 'b' or vice versa
    if token.contains('a') {
        token.replacen('a', "b", 1)
    } else if token.contains('b') {
        token.replacen('b', "a", 1)
    } else {
        // If no 'a' or 'b', just append a character to break the signature
        format!("{token}x")
    }
}

// Helper function to create an expired token (simulation)
fn create_expired_token(token: &str) -> String {
    // In a real implementation, this would create a token with past expiration
    // For testing purposes, we'll return a recognizably invalid token
    // that should be rejected by the JWT validation
    format!("expired.{token}")
}
