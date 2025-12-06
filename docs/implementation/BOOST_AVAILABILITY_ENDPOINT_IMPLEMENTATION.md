# Boost Availability Endpoint Implementation

## Overview

Implemented the Boost Availability endpoint as part of the Backend Race API Enhancements specification. This endpoint provides the frontend with complete information about a player's boost hand state, enabling the UI to display which boost cards are currently available for use.

## Implementation Date

December 6, 2025

## Changes Made

### 1. Response Models (Task 5.1)

Added `BoostAvailabilityResponse` struct in `rust-backend/src/routes/races.rs`:

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct BoostAvailabilityResponse {
    pub available_cards: Vec<u8>,
    pub hand_state: std::collections::HashMap<String, bool>,
    pub current_cycle: u32,
    pub cycles_completed: u32,
    pub cards_remaining: u32,
    pub next_replenishment_at: Option<u32>,
}
```

**Fields:**
- `available_cards`: List of boost card values (0-4) that can currently be used
- `hand_state`: Complete map showing availability status of each card
- `current_cycle`: Current boost cycle number (starts at 1)
- `cycles_completed`: Total number of complete cycles finished
- `cards_remaining`: Number of cards left before automatic replenishment
- `next_replenishment_at`: Cards remaining until next replenishment (None if hand is full)

### 2. Endpoint Handler (Task 5.2)

Implemented `get_boost_availability` endpoint handler with the following logic:

**Route:** `GET /api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability`

**Implementation Steps:**
1. Parse and validate race_uuid and player_uuid
2. Fetch race from database
3. Validate race is in progress (return 409 if not)
4. Find participant by player_uuid (return 404 if not found)
5. Check if player has finished (return 409 if finished)
6. Extract boost hand information from participant
7. Calculate next replenishment timing
8. Return complete availability data

**Error Handling:**
- 400 Bad Request: Invalid UUID format
- 404 Not Found: Race not found or player not in race
- 409 Conflict: Race not in progress or player already finished
- 500 Internal Server Error: Database errors

### 3. OpenAPI Documentation (Task 5.3)

Added comprehensive OpenAPI documentation using `#[utoipa::path]` attribute:

**Documentation Includes:**
- Complete request/response schemas
- Example response showing boost hand state
- All error response codes and descriptions
- Parameter descriptions
- Detailed endpoint description explaining boost hand mechanics

**Example Response:**
```json
{
  "available_cards": [1, 2, 4],
  "hand_state": {
    "0": false,
    "1": true,
    "2": true,
    "3": false,
    "4": true
  },
  "current_cycle": 1,
  "cycles_completed": 0,
  "cards_remaining": 3,
  "next_replenishment_at": 3
}
```

### 4. Route Registration (Task 5.4)

Registered the new endpoint in the `routes()` function:

```rust
.route("/races/:race_uuid/players/:player_uuid/boost-availability", get(get_boost_availability))
```

The route follows the RESTful pattern established by other player-specific endpoints.

## Boost Hand Mechanics

The endpoint exposes the boost hand system mechanics:

1. **5 Cards Per Cycle**: Each player has boost cards 0-4 available per cycle
2. **Single Use**: Each card can only be used once per cycle
3. **Automatic Replenishment**: When all 5 cards are used, the hand replenishes
4. **Cycle Tracking**: System tracks current cycle number and total cycles completed

## Integration with Existing Code

The implementation leverages existing domain logic:

- Uses `BoostHand` struct from `race.rs` domain model
- Calls `get_available_cards()` method on boost hand
- Accesses boost hand state through participant data
- Follows same error handling patterns as other endpoints

## Frontend Integration

This endpoint enables the frontend to:

1. Display which boost cards are currently available
2. Show boost cycle progress (X cards remaining)
3. Indicate when next replenishment will occur
4. Disable UI elements for unavailable cards
5. Show cycle statistics (current cycle, cycles completed)

## Testing Recommendations

### Manual Testing

Test the endpoint with:
```bash
GET /api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability
```

**Test Cases:**
1. Fresh boost hand (all 5 cards available)
2. Partially used hand (some cards used)
3. Nearly depleted hand (1 card remaining)
4. After replenishment (verify cycle increments)
5. Invalid UUIDs (verify 400 response)
6. Player not in race (verify 404 response)
7. Race not in progress (verify 409 response)
8. Player finished (verify 409 response)

### Integration Testing

Should be covered by task 10.5 in the implementation plan:
- Test returns correct available cards
- Test shows correct cycle information
- Test updates after card usage
- Test handles replenishment correctly

## Requirements Validation

This implementation satisfies requirements:

- **5.1**: Provides GET endpoint at specified path
- **5.2**: Returns boolean map of card availability
- **5.3**: Returns current cycle and cycles completed
- **5.4**: Returns cards remaining before replenishment
- **5.5**: Returns next replenishment lap number

## API Documentation

The endpoint is automatically included in the Swagger UI at:
```
http://localhost:3000/swagger-ui
```

Look for the endpoint under the "races" tag.

## Files Modified

- `rust-backend/src/routes/races.rs`: Added response model, endpoint handler, and route registration

## Compilation Status

✅ Code compiles successfully with no warnings or errors
✅ No diagnostic issues detected
✅ Follows Rust best practices and project conventions

## Next Steps

The next task in the implementation plan is:
- Task 6: Implement Lap History Endpoint

## Notes

- The endpoint reuses the existing `BoostHand` domain logic, maintaining single source of truth
- Error handling follows the consistent pattern established by other endpoints
- OpenAPI documentation ensures the endpoint is well-documented for frontend developers
- The implementation is stateless and doesn't modify any data (read-only operation)
