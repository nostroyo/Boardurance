# Testing Documentation

This folder contains all testing documentation, strategies, and test suite information.

## Contents

- `TESTING_GUIDE.md` - Comprehensive testing strategies
- `BACKEND_TEST_SUITE.md` - Backend test organization and usage
- `BOOST_CARD_INTEGRATION_TESTS.md` - Boost card test suite
- `TEST_PLAYER_CREATION.md` - Player creation test scenarios
- `TEST_REORGANIZATION_SUMMARY.md` - Test suite restructuring

## Testing Strategy

### Backend Testing
- **Unit tests** - Domain logic and utilities
- **Integration tests** - API endpoints and database
- **Security tests** - Authorization and edge cases

### Frontend Testing
- **Component tests** - React component testing
- **Integration tests** - User flow testing
- **E2E tests** - Full stack testing

### Blockchain Testing
- **Contract tests** - Smart contract functionality
- **Simulation tests** - NFT minting and trading

## Test Execution

```powershell
# Backend comprehensive testing
cd rust-backend
.\tests\run-all-tests.ps1

# Frontend testing
cd empty-project
.\test-frontend-auth.ps1
```

## Related Features

All features have associated tests documented here.
