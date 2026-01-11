//! Test to verify that the test environment configuration works correctly

use rust_backend::configuration::get_configuration;
use secrecy::ExposeSecret;

#[tokio::test]
async fn test_environment_uses_correct_database_config() {
    // Set test environment
    std::env::set_var("APP_ENVIRONMENT", "test");
    
    // Get configuration
    let config = get_configuration().expect("Failed to read configuration");
    
    // Verify test configuration is loaded
    assert_eq!(config.database.host, "localhost");
    assert_eq!(config.database.port, 27017);
    assert_eq!(config.database.database_name, "rust_backend_test");
    assert_eq!(config.database.username, ""); // Empty for test environment
    assert_eq!(config.database.password.expose_secret(), ""); // Empty for test environment
    assert_eq!(config.database.require_ssl, false);
    
    // Verify connection string is built without authentication
    let connection_string = config.database.connection_string_without_auth();
    assert!(connection_string.contains("mongodb://localhost:27017/rust_backend_test"));
    assert!(!connection_string.contains("@")); // No authentication in connection string
}

#[tokio::test]
async fn test_local_environment_uses_authentication() {
    // Set local environment
    std::env::set_var("APP_ENVIRONMENT", "local");
    
    // Get configuration
    let config = get_configuration().expect("Failed to read configuration");
    
    // Verify local configuration requires authentication
    assert_eq!(config.database.username, "rust_app");
    assert_eq!(config.database.password.expose_secret(), "rust_password");
    
    // Verify connection string includes authentication
    let connection_string = config.database.with_db();
    assert!(connection_string.contains("rust_app:rust_password@"));
}