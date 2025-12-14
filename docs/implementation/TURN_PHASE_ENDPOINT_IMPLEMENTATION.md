# Turn Phase Endpoint Implementation

## Overview

This document describes the implementation of the Turn Phase endpoint for the Backend Race API Enhancements feature.

## Implementation Details

### Task 3: Implement Turn Phase Endpoint

**Status**: ✅ Complete

All subtasks have been successfully implemented:

#### 3.1 Create response models for turn phase ✅
- Created `TurnPhaseResponse` struct with all required fields:
  - `turn_phase`: String indicating current phase
  - `current_lap`: Current lap number
  - `lap_characteristic`: Lap type (Straight/Curve)
  - `submitted_players`: List of player UUIDs who submitted actions
  - `pending_players`: List of player UUIDs still pending
  - `total_active_players`: Count of active (not finished) players

#### 3.2 Implement `get_turn_phase` endpoint handler ✅
- Created `GET /api/v1/races/{race_uuid}/turn-phase` route
- Implemented turn phase determination logic:
  - "Complete": Race status is not InProgress
  - "AllSubmitted": All active participants have submitted actions
  - "WaitingForPlayers": Some participants haven't submitted actions
- Fetches race from database
- Uses `race.all_actions_submitted()` to check submission status
- Uses `race.get_pending_players()` to get pending player list
- Extracts submitted players from `race.pending_actions`
- Returns comprehensive turn phase information

#### 3.3 Add OpenAPI documentation for turn phase endpoint ✅
- Added `#[utoipa::path]` attribute with complete documentation
- Documented all turn phase states
- Included example response showing all fields
- Added error response documentation (400, 404, 500)

#### 3.4 Register turn phase route ✅
- Added route registration in `routes()` function
- Registered endpoint in OpenAPI schema in `startup.rs`
- Added `TurnPhaseResponse` schema to OpenAPI components

## API Endpoint

### GET /api/v1/races/{race_uuid}/turn-phase

Returns the current turn phase state for simultaneous turn resolution.

**Path Parameters:**
- `race_uuid` (String): Race UUID

**Response (200 OK):**
```json
{
  "turn_phase": "WaitingForPlayers",
  "current_lap": 2,
  "lap_characteristic": "Straight",
  "submitted_players": [
    "550e8400-e29b-41d4-a716-446655440000",
    "550e8400-e29b-41d4-a716-446655440001"
  ],
  "pending_players": [
    "550e8400-e29b-41d4-a716-446655440002"
  ],
  "total_active_players": 3
}
```

**Turn Phase Values:**
- `WaitingForPlayers`: Some participants haven't submitted actions yet
- `AllSubmitted`: All active participants have submitted actions
- `Processing`: Actions are being processed (future use)
- `Complete`: Race is finished or not in progress

**Error Responses:**
- `400 Bad Request`: Invalid UUID format
- `404 Not Found`: Race not found
- `500 Internal Server Error`: Database or server error

## Testing

### Manual Testing

1. Start the backend server:
```powershell
cd rust-backend
.\Makefile.ps1 dev
```

2. Create a race and register players (use existing test scripts or API calls)

3. Test the turn phase endpoint:
```powershell
# Get turn phase for a race
curl http://localhost:3000/api/v1/races/{race_uuid}/turn-phase
```

### Integration Testing

Integration tests can be added to `rust-backend/tests/boost_card_integration_tests.rs` or a new test file following the existing patterns.

Example test structure:
```rust
#[tokio::test]
async fn test_turn_phase_waiting_for_players() {
    let app = spawn_app().await;
    let (player1_uuid, cookies1) = app.create_test_user("p1@test.com", "Pass123", "P1").await;
    let (player2_uuid, cookies2) = app.create_test_user("p2@test.com", "Pass123", "P2").await;
    
    let race_uuid = app.create_race(&cookies1).await;
    
    // Register both players
    let car1 = app.get_player_first_car(&player1_uuid, &cookies1).await;
    let car2 = app.get_player_first_car(&player2_uuid, &cookies2).await;
    app.register_for_race(&race_uuid, &player1_uuid, &car1, &cookies1).await;
    app.register_for_race(&race_uuid, &player2_uuid, &car2, &cookies2).await;
    
    // Start race
    app.start_race(&race_uuid, &cookies1).await;
    
    // Check turn phase - should be WaitingForPlayers
    let response = app.get_turn_phase(&race_uuid).await;
    assert_eq!(200, response.status().as_u16());
    
    let turn_phase: TurnPhaseResponse = response.json().await.unwrap();
    assert_eq!("WaitingForPlayers", turn_phase.turn_phase);
    assert_eq!(2, turn_phase.total_active_players);
    assert_eq!(0, turn_phase.submitted_players.len());
    assert_eq!(2, turn_phase.pending_players.len());
}
```

## Files Modified

1. **rust-backend/src/routes/races.rs**
   - Added `TurnPhaseResponse` struct
   - Implemented `get_turn_phase` endpoint handler
   - Registered route in `routes()` function

2. **rust-backend/src/startup.rs**
   - Added `get_turn_phase` to OpenAPI paths
   - Added `TurnPhaseResponse` to OpenAPI schemas

## Requirements Validated

This implementation satisfies the following requirements from the design document:

- **Requirement 3.1**: Provides GET endpoint at specified path ✅
- **Requirement 3.2**: Returns correct turn phase values ✅
- **Requirement 3.3**: Includes submitted players list ✅
- **Requirement 3.4**: Includes pending players list ✅
- **Requirement 3.5**: Includes current lap and lap characteristic ✅
- **Requirement 9.1**: Registered in OpenAPI documentation ✅
- **Requirement 9.2**: Uses appropriate HTTP method (GET) ✅
- **Requirement 9.3**: Follows consistent URL pattern ✅
- **Requirement 9.4**: Includes request/response examples ✅

## Next Steps

The following tasks remain in the Backend Race API Enhancements feature:

- Task 4: Implement Local View Endpoint
- Task 5: Implement Boost Availability Endpoint
- Task 6: Implement Lap History Endpoint
- Task 7: Implement Comprehensive Error Handling
- Task 8: Update OpenAPI Documentation
- Task 9: Checkpoint - Verify all endpoints work correctly
- Task 10: Write Integration Tests for New Endpoints
- Task 11: Write Property-Based Tests
- Task 12: Final Checkpoint - Comprehensive Testing

## Notes

- The endpoint uses existing domain methods (`all_actions_submitted()` and `get_pending_players()`) from the Race struct
- Turn phase determination is based on race status and action submission state
- The implementation follows the existing patterns in the codebase for consistency
- All code compiles without warnings or errors
- OpenAPI documentation is complete and follows the established format
