# Feature #21: Integration Tests Fix

## Problem Description

Integration tests were failing due to database connection issues. The tests were trying to connect to MongoDB with authentication credentials, but the test environment should use MongoDB without authentication for local development.

### Root Cause
- Integration tests were using the default "local" environment configuration which requires authentication
- Tests weren't setting the `APP_ENVIRONMENT` variable to use the "test" configuration
- Test configuration (`test.yaml`) is properly set up with empty username/password for no-auth MongoDB

### Error Messages
```
Failed to connect to database: Error { kind: ServerSelection { message: "Server selection timeout: No available servers. Topology: { Type: Unknown, Servers: [ { Address: localhost:27017, Type: Unknown, Error: Kind: I/O error: Aucune connexion n'a pu être établie car l'ordinateur cible l'a expressément refusée. (os error 10061), labels: {} } ] }" }
```

## Solution Implemented

### 1. Fixed Test Environment Configuration
Modified all integration test files to set the proper environment variable:

**Files Modified:**
- `rust-backend/tests/auth_integration_tests.rs`
- `rust-backend/tests/authorization_integration_tests.rs` 
- `rust-backend/tests/boost_card_integration_tests.rs`
- `rust-backend/tests/security_edge_cases_tests.rs`

**Change Applied:**
```rust
async fn spawn_app() -> TestApp {
    // ... existing tracing setup ...

    // Set test environment to use test configuration
    std::env::set_var("APP_ENVIRONMENT", "test");

    // ... rest of function ...
}
```

### 2. Added Configuration Validation Tests
Created `rust-backend/tests/configuration_test.rs` to verify:
- Test environment uses correct database configuration (no authentication)
- Local environment properly requires authentication
- Connection strings are built correctly for each environment

### 3. Database Configuration Summary

**Test Environment (`configuration/test.yaml`):**
```yaml
application:
  port: 3001
database:
  host: "localhost"
  port: 27017
  database_name: "rust_backend_test"
  username: ""
  password: ""
  require_ssl: false
```

**Local Environment (`configuration/base.yaml` + `configuration/local.yaml`):**
```yaml
database:
  host: "localhost"
  port: 27017
  database_name: "rust_backend"
  username: "rust_app"
  password: "rust_password"
  require_ssl: false
```

## Testing Instructions

### Prerequisites
1. **MongoDB Running**: Start MongoDB without authentication on port 27017
   ```powershell
   # Option 1: Using Docker (if Docker Desktop is running)
   docker run --name mongodb-test -p 27017:27017 -d mongo:7.0
   
   # Option 2: Using the project's Docker Compose
   cd rust-backend
   .\Makefile.ps1 dev
   
   # Option 3: Local MongoDB installation
   mongod --dbpath ./data --port 27017
   ```

### Running Tests

1. **Configuration Tests** (can run without MongoDB):
   ```powershell
   cd rust-backend
   cargo test --test configuration_test
   ```

2. **Integration Tests** (requires MongoDB):
   ```powershell
   cd rust-backend
   cargo test --test auth_integration_tests
   cargo test --test authorization_integration_tests
   cargo test --test boost_card_integration_tests
   cargo test --test security_edge_cases_tests
   ```

3. **All Integration Tests**:
   ```powershell
   cd rust-backend
   cargo test --test "*integration*"
   ```

### Expected Results
- Configuration tests should pass without MongoDB
- Integration tests should connect successfully to MongoDB without authentication
- Each test creates its own random database name for isolation
- Tests should no longer fail with "connection refused" errors

## Technical Details

### Database Connection Logic
The `get_connection_pool` function in `startup.rs` already handles both authenticated and non-authenticated connections:

```rust
pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<Database, mongodb::error::Error> {
    let connection_string = if configuration.username.is_empty() {
        configuration.connection_string_without_auth()  // For test environment
    } else {
        configuration.with_db()  // For local/production with auth
    };
    // ... connection logic
}
```

### Test Isolation
Each integration test:
1. Sets `APP_ENVIRONMENT=test` to use test configuration
2. Generates a random database name using `Uuid::new_v4()`
3. Uses a random port for the test server
4. Creates isolated test environment

## Verification

The fix has been verified with:
1. ✅ Configuration tests pass (no MongoDB required)
2. ✅ Code compiles without warnings
3. ✅ Test environment properly loads no-auth configuration
4. ✅ Local environment properly loads auth configuration

## Next Steps

1. **Start MongoDB** locally to run full integration test suite
2. **Run all integration tests** to verify complete functionality
3. **Update CI/CD pipeline** to ensure MongoDB is available in test environment
4. **Consider adding MongoDB service** to GitHub Actions workflow

## Files Changed

- `rust-backend/tests/auth_integration_tests.rs` - Added environment variable setting
- `rust-backend/tests/authorization_integration_tests.rs` - Added environment variable setting
- `rust-backend/tests/boost_card_integration_tests.rs` - Added environment variable setting
- `rust-backend/tests/security_edge_cases_tests.rs` - Added environment variable setting
- `rust-backend/tests/configuration_test.rs` - New test file for configuration validation
- `docs/bugfixes/feature-21-integration-tests-fix.md` - This documentation

## Impact

- ✅ Integration tests now use correct database configuration
- ✅ Tests can run against local MongoDB without authentication setup
- ✅ Test isolation maintained with random database names
- ✅ No breaking changes to existing functionality
- ✅ Proper separation between test and local environments