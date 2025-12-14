# Design Document

## Overview

The Backend Race API Enhancements add six new RESTful endpoints to the Rust backend that provide the Player Game Interface frontend with all necessary data for displaying race information. This design ensures proper separation of concerns by keeping all game logic calculations on the server while providing rich, calculated data to the client. The endpoints leverage existing domain logic in `race.rs` and `boost_hand_manager.rs` while adding new API routes in `races.rs`.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (React)                          │
│  - PlayerGameInterface                                       │
│  - PerformanceCalculator                                     │
│  - PlayerCarCard                                             │
│  - LocalSectorDisplay                                        │
└────────────────┬────────────────────────────────────────────┘
                 │ HTTP GET Requests
                 │ (No calculations, only display)
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              Backend API Layer (Axum Routes)                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  New Endpoints (races.rs)                           │   │
│  │  - GET /car-data                                     │   │
│  │  - GET /performance-preview                          │   │
│  │  - GET /turn-phase                                   │   │
│  │  - GET /local-view                                   │   │
│  │  - GET /boost-availability                           │   │
│  │  - GET /lap-history                                  │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────┬────────────────────────────────────────────┘
                 │ Calls domain logic
                 ▼
┌─────────────────────────────────────────────────────────────┐
│              Domain Layer (Business Logic)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Existing Domain Logic                               │   │
│  │  - Race::calculate_performance_with_car_data()       │   │
│  │  - BoostHandManager::get_boost_availability()        │   │
│  │  - Race::get_pending_players()                       │   │
│  │  - calculateLocalView() utility                      │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────┬────────────────────────────────────────────┘
                 │ Queries/Updates
                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    MongoDB Database                          │
│  - races collection                                          │
│  - players collection                                        │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Single Source of Truth**: All game logic calculations happen on the backend
2. **Stateless API**: Each endpoint is independent and doesn't maintain session state
3. **Reuse Existing Logic**: Leverage existing domain methods rather than duplicating code
4. **Rich Responses**: Return complete, calculated data to minimize frontend logic
5. **Consistent Error Handling**: Use standard HTTP status codes and error messages

## Components and Interfaces

### 1. Car Data Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/players/{player_uuid}/car-data`

**Purpose**: Retrieve complete car, pilot, engine, and body data for a player's race entry.

**Request Parameters**:
- `race_uuid`: UUID (path parameter)
- `player_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct CarDataResponse {
    pub car: CarInfo,
    pub pilot: PilotInfo,
    pub engine: EngineInfo,
    pub body: BodyInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CarInfo {
    pub uuid: String,
    pub name: String,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotInfo {
    pub uuid: String,
    pub name: String,
    pub pilot_class: String,
    pub rarity: String,
    pub skills: PilotSkills,
    pub performance: PilotPerformance,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotSkills {
    pub reaction_time: u8,
    pub precision: u8,
    pub focus: u8,
    pub stamina: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotPerformance {
    pub straight_value: u8,
    pub curve_value: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EngineInfo {
    pub uuid: String,
    pub name: String,
    pub rarity: String,
    pub straight_value: u8,
    pub curve_value: u8,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BodyInfo {
    pub uuid: String,
    pub name: String,
    pub rarity: String,
    pub straight_value: u8,
    pub curve_value: u8,
    pub nft_mint_address: Option<String>,
}
```

**Implementation Strategy**:
1. Validate race_uuid and player_uuid
2. Fetch race from database
3. Find participant by player_uuid
4. Use `CarValidationService::validate_car_for_race()` to get car data
5. Transform domain models to API response models
6. Return 404 if player not in race

### 2. Performance Preview Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview`

**Purpose**: Calculate performance predictions for all boost card options (0-4).

**Request Parameters**:
- `race_uuid`: UUID (path parameter)
- `player_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct PerformancePreviewResponse {
    pub base_performance: BasePerformance,
    pub boost_options: Vec<BoostOption>,
    pub boost_cycle_info: BoostCycleInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BasePerformance {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
    pub lap_characteristic: String, // "Straight" or "Curve"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostOption {
    pub boost_value: u8,
    pub is_available: bool,
    pub final_value: u32,
    pub movement_probability: String, // "MoveUp", "Stay", "MoveDown"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostCycleInfo {
    pub current_cycle: u32,
    pub cycles_completed: u32,
    pub cards_remaining: u32,
    pub available_cards: Vec<u8>,
}
```

**Implementation Strategy**:
1. Validate UUIDs and fetch race
2. Find participant and validate car data
3. Calculate base performance using `Race::calculate_performance_with_car_data()`
4. For each boost card (0-4):
   - Check availability using `BoostHand::is_card_available()`
   - Calculate final value with boost multiplier
   - Determine movement probability based on sector thresholds
5. Get boost cycle info from participant's `boost_hand`
6. Return complete preview

**Movement Probability Logic**:
```rust
fn calculate_movement_probability(
    final_value: u32,
    current_sector: &Sector,
    next_sector: Option<&Sector>,
) -> MovementProbability {
    if final_value >= current_sector.max_value {
        MovementProbability::MoveUp
    } else if final_value < current_sector.min_value {
        MovementProbability::MoveDown
    } else {
        MovementProbability::Stay
    }
}
```

### 3. Turn Phase Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/turn-phase`

**Purpose**: Return current turn phase state for simultaneous turn resolution.

**Request Parameters**:
- `race_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct TurnPhaseResponse {
    pub turn_phase: String, // "WaitingForPlayers", "AllSubmitted", "Processing", "Complete"
    pub current_lap: u32,
    pub lap_characteristic: String,
    pub submitted_players: Vec<String>, // UUIDs
    pub pending_players: Vec<String>,   // UUIDs
    pub total_active_players: u32,
}
```

**Implementation Strategy**:
1. Fetch race from database
2. Determine turn phase:
   - If `race.status != InProgress`: "Complete"
   - If `race.all_actions_submitted()`: "AllSubmitted"
   - Otherwise: "WaitingForPlayers"
3. Get submitted players from `race.pending_actions`
4. Get pending players using `race.get_pending_players()`
5. Return phase information

### 4. Local View Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/players/{player_uuid}/local-view`

**Purpose**: Calculate the player's 5-sector local view (current ±2 sectors).

**Request Parameters**:
- `race_uuid`: UUID (path parameter)
- `player_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct LocalViewResponse {
    pub center_sector: u32,
    pub visible_sectors: Vec<SectorInfo>,
    pub visible_participants: Vec<ParticipantInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorInfo {
    pub id: u32,
    pub name: String,
    pub min_value: u32,
    pub max_value: u32,
    pub slot_capacity: Option<u32>,
    pub sector_type: String,
    pub current_occupancy: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParticipantInfo {
    pub player_uuid: String,
    pub player_name: Option<String>,
    pub car_name: String,
    pub current_sector: u32,
    pub position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
    pub is_finished: bool,
}
```

**Implementation Strategy**:
1. Fetch race and find participant
2. Calculate visible sector IDs (center ±2)
3. Filter sectors to visible range
4. Filter participants to visible range
5. Fetch player names from database (optional)
6. Return local view data

**Sector Range Calculation**:
```rust
fn get_visible_sector_ids(center: u32, total_sectors: usize) -> Vec<u32> {
    let mut ids = Vec::new();
    for offset in -2..=2 {
        let sector_id = (center as i32 + offset).rem_euclid(total_sectors as i32) as u32;
        ids.push(sector_id);
    }
    ids
}
```

### 5. Boost Availability Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability`

**Purpose**: Return which boost cards are currently available for use.

**Request Parameters**:
- `race_uuid`: UUID (path parameter)
- `player_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct BoostAvailabilityResponse {
    pub available_cards: Vec<u8>,
    pub hand_state: HashMap<String, bool>, // "0" -> true/false
    pub current_cycle: u32,
    pub cycles_completed: u32,
    pub cards_remaining: u32,
    pub next_replenishment_at: Option<u32>, // Lap number
}
```

**Implementation Strategy**:
1. Fetch race and find participant
2. Get boost hand from participant
3. Extract availability information
4. Calculate next replenishment lap (current_lap + cards_remaining)
5. Return availability data

### 6. Lap History Endpoint

**Route**: `GET /api/v1/races/{race_uuid}/players/{player_uuid}/lap-history`

**Purpose**: Return lap-by-lap performance history for the player.

**Request Parameters**:
- `race_uuid`: UUID (path parameter)
- `player_uuid`: UUID (path parameter)

**Response Schema**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct LapHistoryResponse {
    pub laps: Vec<LapRecord>,
    pub cycle_summaries: Vec<CycleSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LapRecord {
    pub lap_number: u32,
    pub lap_characteristic: String,
    pub boost_used: u8,
    pub boost_cycle: u32,
    pub base_value: u32,
    pub final_value: u32,
    pub from_sector: u32,
    pub to_sector: u32,
    pub movement_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CycleSummary {
    pub cycle_number: u32,
    pub cards_used: Vec<u8>,
    pub laps_in_cycle: Vec<u32>,
    pub average_boost: f32,
}
```

**Implementation Strategy**:
1. Fetch race and find participant
2. Get boost usage history from participant
3. Build lap records from usage history
4. Get cycle summaries using `participant.get_boost_cycle_summaries()`
5. Return history data

## Data Models

### Existing Domain Models (Reused)

From `race.rs`:
- `Race`
- `RaceParticipant`
- `Sector`
- `BoostHand`
- `BoostUsageRecord`
- `BoostCycleSummary`
- `PerformanceCalculation`

From `car_validation.rs`:
- `ValidatedCarData`
- Contains: `Car`, `Pilot`, `Engine`, `Body`

### New API Response Models

All response models listed in Components section above.

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Acceptence Criteria Testing Prework:

**1.1** THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/car-data` that returns complete car information
Thoughts: This is testing that an endpoint exists and returns data. We can test this by making a request and verifying the response structure.
Testable: yes - example

**1.2** THE Car_Data_API SHALL return the player's car, pilot, engine, and body data with all stat values
Thoughts: This is testing that specific fields are present in the response. We can generate random car data and verify all fields are included.
Testable: yes - property

**2.1** THE Performance_Preview_API SHALL calculate base performance using the player's engine, body, and pilot stats for the current lap characteristic
Thoughts: This is testing that the calculation uses the correct inputs. We can verify the calculation matches the domain logic.
Testable: yes - property

**2.2** THE Performance_Preview_API SHALL apply the current sector's maximum value ceiling to the base performance
Thoughts: This is testing that sector ceiling is applied correctly. We can test with base values above and below the ceiling.
Testable: yes - property

**2.3** THE Performance_Preview_API SHALL calculate final performance for each boost card (0-4) using the boost multiplier formula
Thoughts: This is testing the boost multiplier calculation. We can verify it matches the domain logic formula.
Testable: yes - property

**3.1** THE Turn_Phase_API SHALL return one of: "WaitingForPlayers", "AllSubmitted", "Processing", or "Complete"
Thoughts: This is testing that the API returns valid phase values. We can test different race states.
Testable: yes - property

**4.1** THE Local_View_API SHALL return the player's current sector plus 2 sectors above and 2 sectors below
Thoughts: This is testing the sector range calculation. We can verify it returns exactly 5 sectors centered on the player.
Testable: yes - property

**5.1** THE Boost_Availability_API SHALL return a boolean map of which boost cards (0-4) are currently available
Thoughts: This is testing that availability state is correctly returned. We can verify it matches the boost hand state.
Testable: yes - property

**6.1** THE Lap_History_API SHALL return each lap's number, lap characteristic, boost used, base value, and final value
Thoughts: This is testing that all required fields are present in lap records. We can verify the structure.
Testable: yes - property

**7.2** THE Backend SHALL apply sector ceiling to base performance BEFORE applying the boost multiplier
Thoughts: This is testing the order of operations in performance calculation. We can verify with values that exceed the ceiling.
Testable: yes - property

**8.1** WHEN a player is not found in a race, THE Backend SHALL return HTTP 404 with message "Player not found in race"
Thoughts: This is testing error handling for a specific case. We can test with invalid player UUIDs.
Testable: yes - example

### Property Reflection

After reviewing all properties, I identify the following:
- Properties 2.1, 2.2, and 2.3 all test performance calculation and can be combined into one comprehensive property
- Property 1.2 and 6.1 both test response structure completeness and can use similar testing approach
- All other properties provide unique validation value

### Correctness Properties

**Property 1: Car data response completeness**
*For any* valid player in a race, the car data endpoint should return all required fields (car, pilot, engine, body with all stats)
**Validates: Requirements 1.2, 1.3, 1.4**

**Property 2: Performance calculation correctness**
*For any* car data and boost value, the performance preview should match the domain logic calculation (base value, sector ceiling, boost multiplier)
**Validates: Requirements 2.2, 2.3, 2.4, 7.2**

**Property 3: Turn phase state consistency**
*For any* race state, the turn phase endpoint should return a phase that matches the race's actual state (pending actions, all submitted, etc.)
**Validates: Requirements 3.2, 3.3, 3.4**

**Property 4: Local view sector range**
*For any* player position, the local view should return exactly 5 sectors (current ±2) with correct wrapping for circular tracks
**Validates: Requirements 4.2, 4.3**

**Property 5: Boost availability consistency**
*For any* boost hand state, the availability endpoint should return availability that matches the participant's boost hand
**Validates: Requirements 5.2, 5.3, 5.4**

**Property 6: Lap history completeness**
*For any* participant with lap history, the lap history endpoint should return all laps with complete information (lap number, boost, values, movement)
**Validates: Requirements 6.2, 6.3, 6.4**

**Property 7: Movement probability accuracy**
*For any* performance value and sector thresholds, the movement probability should correctly predict MoveUp (≥max), Stay (between min/max), or MoveDown (<min)
**Validates: Requirements 2.5**

## Error Handling

### HTTP Status Codes

- **200 OK**: Successful GET request
- **400 Bad Request**: Invalid UUID format, invalid boost value
- **404 Not Found**: Race not found, player not in race
- **409 Conflict**: Race not in progress, player already finished
- **500 Internal Server Error**: Database errors, unexpected failures

### Error Response Format

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}
```

### Error Handling Strategy

1. **UUID Validation**: Parse UUIDs early and return 400 for invalid format
2. **Race Validation**: Check race exists and is in correct state
3. **Player Validation**: Verify player is participant and not finished
4. **Database Errors**: Log detailed errors, return generic 500 to client
5. **Consistent Messages**: Use exact error messages specified in requirements

## Testing Strategy

### Unit Tests

Unit tests will focus on helper functions and data transformations:

- `get_visible_sector_ids()` - sector range calculation
- `calculate_movement_probability()` - movement prediction logic
- Response model construction from domain models
- Error message formatting

### Integration Tests

Integration tests will verify end-to-end API behavior:

**Test Suite 1: Car Data Endpoint**
- Valid player returns complete car data
- Invalid player UUID returns 404
- Player not in race returns 404
- Response includes all required fields

**Test Suite 2: Performance Preview Endpoint**
- Calculates correct base performance for Straight lap
- Calculates correct base performance for Curve lap
- Applies sector ceiling correctly
- Calculates boost multipliers correctly (0-4)
- Shows correct boost availability
- Returns movement probabilities

**Test Suite 3: Turn Phase Endpoint**
- Returns "WaitingForPlayers" when actions pending
- Returns "AllSubmitted" when all submitted
- Returns "Complete" when race finished
- Lists correct submitted/pending players

**Test Suite 4: Local View Endpoint**
- Returns 5 sectors centered on player
- Handles track wrapping correctly
- Includes only visible participants
- Returns correct sector occupancy

**Test Suite 5: Boost Availability Endpoint**
- Returns correct available cards
- Shows correct cycle information
- Updates after card usage
- Handles replenishment correctly

**Test Suite 6: Lap History Endpoint**
- Returns all lap records
- Includes boost cycle information
- Returns cycle summaries
- Handles empty history

**Test Suite 7: Error Handling**
- Invalid UUIDs return 400
- Missing race returns 404
- Missing player returns 404
- Race not in progress returns 409
- Player finished returns 409

### Property-Based Tests

Property-based tests will verify universal properties:

- Performance calculations match domain logic for all inputs
- Local view always returns exactly 5 sectors
- Boost availability matches boost hand state
- Movement probability correctly categorizes all values

## Implementation Notes

### Code Organization

New code will be added to `rust-backend/src/routes/races.rs`:

```rust
// New endpoint handlers (around line 1800+)
pub async fn get_car_data(...) -> Result<Json<CarDataResponse>, StatusCode>
pub async fn get_performance_preview(...) -> Result<Json<PerformancePreviewResponse>, StatusCode>
pub async fn get_turn_phase(...) -> Result<Json<TurnPhaseResponse>, StatusCode>
pub async fn get_local_view(...) -> Result<Json<LocalViewResponse>, StatusCode>
pub async fn get_boost_availability(...) -> Result<Json<BoostAvailabilityResponse>, StatusCode>
pub async fn get_lap_history(...) -> Result<Json<LapHistoryResponse>, StatusCode>

// Helper functions
fn get_visible_sector_ids(center: u32, total: usize) -> Vec<u32>
fn calculate_movement_probability(value: u32, sector: &Sector) -> MovementProbability
fn build_car_data_response(car_data: &ValidatedCarData) -> CarDataResponse
```

### Route Registration

Update the `routes()` function to register new endpoints:

```rust
pub fn routes() -> Router<Database> {
    Router::new()
        // ... existing routes ...
        
        // New player-specific endpoints
        .route("/races/:race_uuid/players/:player_uuid/car-data", get(get_car_data))
        .route("/races/:race_uuid/players/:player_uuid/performance-preview", get(get_performance_preview))
        .route("/races/:race_uuid/players/:player_uuid/local-view", get(get_local_view))
        .route("/races/:race_uuid/players/:player_uuid/boost-availability", get(get_boost_availability))
        .route("/races/:race_uuid/players/:player_uuid/lap-history", get(get_lap_history))
        
        // Race-level endpoint
        .route("/races/:race_uuid/turn-phase", get(get_turn_phase))
}
```

### OpenAPI Documentation

All endpoints will be documented with `#[utoipa::path]` attributes including:
- Complete request/response schemas
- Example requests and responses
- Error response documentation
- Parameter descriptions

### Performance Considerations

- **Caching**: Consider caching car data for duration of race
- **Database Queries**: Minimize queries by fetching race once per request
- **Response Size**: Keep responses focused on necessary data
- **Computation**: Reuse existing domain logic rather than recalculating

## Dependencies

### Existing Code Dependencies

- `crate::domain::Race` - race domain logic
- `crate::domain::BoostHand` - boost card management
- `crate::services::car_validation::CarValidationService` - car data retrieval
- `mongodb::Database` - database access
- `axum` - web framework
- `utoipa` - OpenAPI documentation

### No New External Dependencies Required

All functionality can be implemented using existing crates.

## Security Considerations

1. **Input Validation**: Validate all UUIDs before database queries
2. **Authorization**: Future enhancement - verify player owns the requested data
3. **Rate Limiting**: Future enhancement - prevent API abuse
4. **Data Exposure**: Only return data relevant to the requesting player
5. **Error Messages**: Don't expose internal implementation details

## Future Enhancements

1. **Caching Layer**: Add Redis caching for frequently accessed data
2. **WebSocket Support**: Real-time updates for turn phase changes
3. **Batch Endpoints**: Single endpoint returning multiple data types
4. **Historical Analytics**: Extended lap history with statistical analysis
5. **Authentication**: JWT-based authentication for player verification
