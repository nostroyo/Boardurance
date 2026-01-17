//! Test to verify that the integration test environment setup works correctly

use rust_backend::configuration::get_configuration;
use secrecy::ExposeSecret;

#[tokio::test]
async fn test_integration_test_environment_setup() {
    // This test verifies that setting APP_ENVIRONMENT=test works
    // which is the key fix for integration tests

    // Set test environment (same as integration tests do)
    std::env::set_var("APP_ENVIRONMENT", "test");

    // Get configuration (same as integration tests do)
    let config = get_configuration().expect("Failed to read configuration");

    // The key requirement: username should be empty for test environment
    // This allows connection without authentication
    assert_eq!(config.database.username, "");
    assert_eq!(config.database.password.expose_secret(), "");

    // Verify connection string is built without authentication
    let connection_string = config.database.connection_string_without_auth();
    assert!(!connection_string.contains('@')); // No authentication in connection string
    assert!(connection_string.starts_with("mongodb://localhost:27017/"));

    println!("✅ Integration test environment setup works correctly");
    println!("   Database: {}", config.database.database_name);
    println!("   Connection: {connection_string}");
}
