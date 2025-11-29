# Backend Test Suite

This directory contains all PowerShell test scripts for the Rust backend, organized by category and updated for the new authentication system.

## Directory Structure

```
rust-backend/tests/
├── api/                           # API endpoint tests
│   ├── test-auth-endpoints.ps1    # Authentication (register/login) tests
│   ├── test-general-endpoints.ps1 # Health check, OpenAPI, basic endpoints
│   └── test-player-endpoints.ps1  # Player management with auth integration
├── infrastructure/                # Infrastructure and setup tests
│   ├── test-docker-setup.ps1      # Complete Docker MongoDB setup test
│   ├── test-project-structure.ps1 # Project file structure validation
│   ├── test-with-mongodb.ps1      # Full integration test with MongoDB
│   └── verify-docker-setup.ps1    # Docker setup file verification
├── run-all-tests.ps1             # Comprehensive test runner
└── README.md                     # Test suite documentation
```

## Running Tests

### Quick Start
```powershell
# Run all tests
.\tests\run-all-tests.ps1

# Or use the Makefile
.\Makefile.ps1 test
```

### Specific Test Suites
```powershell
# API tests only (requires running server)
.\tests\run-all-tests.ps1 -TestSuite api

# Infrastructure tests only
.\tests\run-all-tests.ps1 -TestSuite infrastructure

# Unit tests only
.\tests\run-all-tests.ps1 -TestSuite unit

# With verbose output
.\tests\run-all-tests.ps1 -Verbose
```

### Individual Tests
```powershell
# Authentication tests
.\tests\api\test-auth-endpoints.ps1

# Player management tests
.\tests\api\test-player-endpoints.ps1

# MongoDB integration test
.\tests\infrastructure\test-with-mongodb.ps1
```

## Test Categories

### API Tests
- **Authentication**: User registration, login, password validation
- **Player Management**: CRUD operations with auth integration
- **General Endpoints**: Health check, OpenAPI documentation

### Infrastructure Tests
- **Project Structure**: Validates all required files exist
- **Docker Setup**: Tests MongoDB container setup and initialization
- **Integration**: Full stack test with database operations

### Unit Tests
- **Rust Tests**: Domain logic, password hashing, validation

## Prerequisites

### For API Tests
1. Start the backend server:
   ```powershell
   .\Makefile.ps1 dev
   ```

### For Infrastructure Tests
1. Docker Desktop installed and running
2. MongoDB containers available

## Test Features

### Updated for Authentication System
- ✅ **Email/Password Registration**: Secure user creation with Argon2 hashing
- ✅ **Login Validation**: Password verification and user authentication
- ✅ **UUID-based Identification**: Primary keys use UUIDs, not wallet addresses
- ✅ **Optional Wallet Connection**: Users can connect wallets after registration
- ✅ **Backward Compatibility**: Legacy endpoints still work for existing tests

### Error Handling
- ✅ **Graceful Failures**: Tests handle expected errors (duplicates, validation)
- ✅ **Detailed Reporting**: Clear success/failure messages with context
- ✅ **Timeout Protection**: All HTTP requests have reasonable timeouts

### Test Data Management
- ✅ **Idempotent Tests**: Can be run multiple times safely
- ✅ **Cleanup Handling**: Proper resource cleanup after tests
- ✅ **Isolated Test Data**: Each test uses unique identifiers

## Removed Scripts

The following outdated scripts were removed during the reorganization:

- `test-auth-simple.ps1` - Redundant, replaced by comprehensive auth tests
- `test_server.ps1` - Wrong endpoints, replaced by general endpoint tests
- `test-create-player-with-assets.ps1` - Wrong port, outdated API
- `test-moveable-configuration.ps1` - Non-existent endpoints
- `test-player-endpoints.ps1` - Updated for auth system
- `test-player-uuid-endpoints.ps1` - Updated for auth system
- `scripts/test-with-mongodb.ps1` - Moved and updated

## Contributing

When adding new tests:

1. **Place in appropriate directory** (`api/` or `infrastructure/`)
2. **Follow naming convention** (`test-feature-name.ps1`)
3. **Include proper error handling** with try/catch blocks
4. **Add to test runner** if it should be part of the main suite
5. **Update this README** with new test descriptions

## Troubleshooting

### Common Issues

**"Server not running" errors**:
```powershell
# Start the development server first
.\Makefile.ps1 dev
```

**MongoDB connection failures**:
```powershell
# Ensure MongoDB is running
.\scripts\start-mongodb.ps1
```

**Docker issues**:
```powershell
# Verify Docker setup
.\tests\infrastructure\verify-docker-setup.ps1
```

**Authentication test failures**:
- Check if user already exists (tests handle this gracefully)
- Verify password requirements (8+ chars, uppercase, lowercase, digit)
- Ensure email format is valid
