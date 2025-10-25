# PowerShell Test Scripts Audit

## Current Status Analysis

### ✅ RELEVANT & WORKING SCRIPTS

#### Authentication Tests
- `test-auth-endpoints.ps1` - **KEEP** - Comprehensive auth testing with proper error handling
- `test-auth-simple.ps1` - **REMOVE** - Redundant, simpler version of above

#### Infrastructure & Setup
- `Makefile.ps1` - **KEEP** - Essential build/dev commands
- `scripts/start-mongodb.ps1` - **KEEP** - Core MongoDB startup
- `verify-docker-setup.ps1` - **KEEP** - Validates Docker setup
- `test-docker-setup.ps1` - **KEEP** - Tests complete Docker setup

#### Player Management (Updated for new auth system)
- `test-player-endpoints.ps1` - **UPDATE NEEDED** - Still uses wallet-based creation, needs auth integration
- `test-player-uuid-endpoints.ps1` - **UPDATE NEEDED** - Same issue as above

#### Basic Infrastructure
- `test_endpoints.ps1` - **KEEP** - Good general API testing
- `simple_test.ps1` - **KEEP** - Basic structure validation

### ❌ OUTDATED/PROBLEMATIC SCRIPTS

#### Deprecated/Broken
- `test_server.ps1` - **REMOVE** - Wrong health endpoint URL, redundant with test_endpoints.ps1
- `test-create-player-with-assets.ps1` - **REMOVE** - Uses wrong port (8000), outdated endpoint
- `test-moveable-configuration.ps1` - **REMOVE** - Uses wrong port (8000), endpoint doesn't exist

#### Scripts Directory
- `scripts/test-with-mongodb.ps1` - **UPDATE NEEDED** - Uses wrong port (3001), needs auth integration

### 🔧 MISSING SCRIPTS NEEDED
- Comprehensive integration test combining auth + player management
- Database cleanup/reset script
- Performance/load testing script

## Recommended Actions

1. **Create `tests/` directory** for organized test scripts
2. **Update existing scripts** to work with new auth system
3. **Remove redundant/broken scripts**
4. **Create missing integration tests**