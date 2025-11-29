# Boost Card Integration Tests

## Overview

This test suite provides comprehensive integration tests for the boost card system, covering:

- Boost hand initialization with all 5 cards available
- Boost card usage and availability tracking
- Boost hand replenishment after all cards are used
- Database persistence of boost hand state
- Boost usage history tracking
- Boost cycle summaries
- Error handling for invalid boost values and unavailable cards
- Concurrent player actions with independent boost hands
- API endpoint responses with correct boost hand data

## Prerequisites

### Required Services

1. **MongoDB** must be running before executing these tests
   ```powershell
   # Start MongoDB using Docker Compose
   docker-compose up -d mongodb
   
   # Or use the startup script (if execution policy allows)
   .\scripts\start-mongodb.ps1
   ```

2. **Docker Desktop** must be installed and running

### Verify MongoDB is Running

```powershell
# Check if MongoDB container is running
docker ps | findstr mongodb

# Test MongoDB connection
docker exec rust-backend-mongodb mongosh --quiet --eval "db.adminCommand('ping')"
```

## Running the Tests

### Run All Boost Card Integration Tests

```powershell
# From rust-backend directory
cargo test --test boost_card_integration_tests -- --test-threads=1
```

### Run Specific Test

```powershell
# Run a single test
cargo test --test boost_card_integration_tests test_boost_hand_initializes_with_all_cards_available -- --test-threads=1

# Run with output
cargo test --test boost_card_integration_tests test_boost_hand_initializes_with_all_cards_available -- --test-threads=1 --nocapture
```

### Run with Logging

```powershell
# Set environment variable to see test logs
$env:TEST_LOG="1"
cargo test --test boost_card_integration_tests -- --test-threads=1 --nocapture
```

## Test Coverage

### Requirements Covered

These tests verify the following requirements from the spec:

- **Requirement 5.1**: Boost card validation and availability checking
- **Requirement 5.2**: Boost hand state management and replenishment
- **Requirement 6.1**: Database persistence of boost hand state
- **Requirement 8.2**: API endpoints return correct boost hand data

### Test Cases

1. **test_boost_hand_initializes_with_all_cards_available**
   - Verifies new players start with all 5 boost cards (0-4) available
   - Checks initial cycle state (cycle 1, 0 cycles completed)

2. **test_using_boost_card_marks_it_unavailable**
   - Tests that using a boost card marks it as unavailable
   - Verifies cards_remaining decrements correctly

3. **test_cannot_use_same_boost_card_twice**
   - Validates error handling when attempting to reuse a card
   - Checks error response includes available cards information

4. **test_boost_hand_replenishes_after_all_cards_used**
   - Tests automatic replenishment when all 5 cards are used
   - Verifies cycle counter increments and all cards become available again

5. **test_boost_hand_state_persists_in_database**
   - Confirms boost hand state is correctly saved to and loaded from database
   - Tests persistence across multiple lap actions

6. **test_boost_usage_history_tracks_all_usages**
   - Verifies lap-by-lap boost usage is recorded
   - Checks usage history includes lap number, boost value, and cycle number

7. **test_invalid_boost_value_returns_error**
   - Tests error handling for boost values outside 0-4 range
   - Validates error response format

8. **test_boost_impact_preview_shows_only_available_cards**
   - Verifies boost impact preview correctly marks used cards as unavailable
   - Tests that all 5 cards are shown with availability flags

9. **test_multiple_cycles_track_correctly**
   - Tests cycle tracking across multiple replenishments
   - Verifies usage history spans multiple cycles correctly

10. **test_boost_cycle_summaries_calculated_correctly**
    - Tests cycle summary generation after completing a cycle
    - Verifies average boost calculation

11. **test_concurrent_lap_submissions_handle_boost_cards_correctly**
    - Tests that multiple players have independent boost hands
    - Verifies concurrent actions don't interfere with each other

## Test Architecture

### Test Setup

Each test:
1. Spawns a new test application with isolated database
2. Creates test users with authentication
3. Creates a race with test track configuration
4. Registers players and starts the race
5. Executes lap actions with boost cards
6. Verifies results through API responses

### Test Isolation

- Each test uses a unique database (UUID-based name)
- Tests can run in parallel (use `--test-threads=1` for sequential execution)
- No shared state between tests

### Helper Methods

The `TestApp` struct provides helper methods for:
- User creation and authentication
- Race creation and management
- Player registration
- Lap action submission
- Race status retrieval

## Troubleshooting

### MongoDB Connection Errors

```
Error: Server selection timeout: No available servers
```

**Solution**: Start MongoDB before running tests
```powershell
docker-compose up -d mongodb
```

### Docker Not Running

```
Error: Cannot connect to Docker daemon
```

**Solution**: Start Docker Desktop and wait for it to be ready

### PowerShell Execution Policy

```
Error: execution of scripts is disabled on this system
```

**Solution**: Use docker-compose directly or update execution policy:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Test Timeouts

If tests timeout waiting for MongoDB:
1. Check MongoDB is running: `docker ps`
2. Check MongoDB logs: `docker logs rust-backend-mongodb`
3. Restart MongoDB: `docker-compose restart mongodb`

## Integration with CI/CD

For automated testing in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Start MongoDB
  run: docker-compose up -d mongodb

- name: Wait for MongoDB
  run: |
    timeout 30 bash -c 'until docker exec rust-backend-mongodb mongosh --quiet --eval "db.adminCommand(\"ping\")" > /dev/null 2>&1; do sleep 1; done'

- name: Run Integration Tests
  run: cargo test --test boost_card_integration_tests -- --test-threads=1
```

## Future Enhancements

Potential improvements for this test suite:

- Add tests for boost card usage across multiple laps in a single race
- Test boost card behavior with different track configurations
- Add performance tests for concurrent boost card operations
- Test boost card state recovery after server restart
- Add tests for boost card analytics and reporting endpoints
