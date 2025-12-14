# Implementation Plan

- [x] 1. Implement Car Data Endpoint





  - [x] 1.1 Create response models for car data endpoint


    - Create `CarDataResponse`, `CarInfo`, `PilotInfo`, `PilotSkills`, `PilotPerformance`, `EngineInfo`, `BodyInfo` structs
    - Add `#[derive(Debug, Serialize, ToSchema)]` attributes for OpenAPI documentation
    - _Requirements: 1.1, 1.2, 1.3, 1.4_
  
  - [x] 1.2 Implement `get_car_data` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/players/{player_uuid}/car-data` route
    - Validate race_uuid and player_uuid parameters
    - Fetch race and find participant by player_uuid
    - Use `CarValidationService::validate_car_for_race()` to get car data
    - Transform domain models to API response models
    - Return 404 if player not in race
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 1.3 Add OpenAPI documentation for car data endpoint

    - Add `#[utoipa::path]` attribute with complete request/response schemas
    - Include example responses in documentation
    - Document error responses (404, 400, 500)
    - _Requirements: 9.1, 9.4_
  

  - [x] 1.4 Register car data route in routes() function

    - Add route registration in `routes()` function
    - Follow RESTful URL pattern `/api/v1/races/{race_uuid}/players/{player_uuid}/car-data`
    - _Requirements: 9.2, 9.3_

- [x] 2. Implement Performance Preview Endpoint




  - [x] 2.1 Create response models for performance preview


    - Create `PerformancePreviewResponse`, `BasePerformance`, `BoostOption`, `BoostCycleInfo` structs
    - Add serialization and schema attributes
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [x] 2.2 Implement movement probability calculation helper


    - Create `calculate_movement_probability()` function
    - Implement logic: MoveUp (≥max), Stay (between min/max), MoveDown (<min)
    - Return `MovementProbability` enum
    - _Requirements: 2.5_
  
  - [x] 2.3 Implement `get_performance_preview` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview` route
    - Validate UUIDs and fetch race
    - Find participant and validate car data using `CarValidationService`
    - Calculate base performance using `Race::calculate_performance_with_car_data()`
    - For each boost card (0-4): check availability, calculate final value, determine movement probability
    - Get boost cycle info from participant's boost_hand
    - Return complete preview with all boost options
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 2.4 Add OpenAPI documentation for performance preview endpoint

    - Add `#[utoipa::path]` attribute with schemas
    - Include example responses showing boost options
    - Document error responses
    - _Requirements: 9.1, 9.4_
  
  - [x] 2.5 Register performance preview route


    - Add route registration in `routes()` function
    - _Requirements: 9.2, 9.3_

- [x] 3. Implement Turn Phase Endpoint





  - [x] 3.1 Create response models for turn phase


    - Create `TurnPhaseResponse` struct with phase, lap info, and player lists
    - Define turn phase logic based on race state
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 3.2 Implement `get_turn_phase` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/turn-phase` route
    - Fetch race from database
    - Determine turn phase using `race.all_actions_submitted()` and race status
    - Get submitted players from `race.pending_actions`
    - Get pending players using `race.get_pending_players()`
    - Return phase information with player lists
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 3.3 Add OpenAPI documentation for turn phase endpoint

    - Add `#[utoipa::path]` attribute
    - Document all turn phase states
    - _Requirements: 9.1, 9.4_
  
  - [x] 3.4 Register turn phase route


    - Add route registration in `routes()` function
    - _Requirements: 9.2, 9.3_

- [x] 4. Implement Local View Endpoint





  - [x] 4.1 Create response models for local view


    - Create `LocalViewResponse`, `SectorInfo`, `ParticipantInfo` structs
    - Add serialization attributes
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [x] 4.2 Implement sector range calculation helper


    - Create `get_visible_sector_ids()` function
    - Calculate center ±2 sectors with proper wrapping for circular tracks
    - Handle edge cases at track boundaries
    - _Requirements: 4.2_
  
  - [x] 4.3 Implement `get_local_view` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/players/{player_uuid}/local-view` route
    - Fetch race and find participant
    - Calculate visible sector IDs using helper function
    - Filter sectors to visible range
    - Filter participants to visible range
    - Optionally fetch player names from database
    - Return local view data with 5 sectors
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [x] 4.4 Add OpenAPI documentation for local view endpoint

    - Add `#[utoipa::path]` attribute
    - Include example showing 5-sector view
    - _Requirements: 9.1, 9.4_
  
  - [x] 4.5 Register local view route


    - Add route registration in `routes()` function
    - _Requirements: 9.2, 9.3_

- [x] 5. Implement Boost Availability Endpoint





  - [x] 5.1 Create response models for boost availability


    - Create `BoostAvailabilityResponse` struct
    - Include available cards, hand state, cycle info, next replenishment
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  
  - [x] 5.2 Implement `get_boost_availability` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability` route
    - Fetch race and find participant
    - Get boost hand from participant
    - Extract availability information using `BoostHandManager::get_boost_availability()`
    - Calculate next replenishment lap (current_lap + cards_remaining)
    - Return availability data
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  
  - [x] 5.3 Add OpenAPI documentation for boost availability endpoint

    - Add `#[utoipa::path]` attribute
    - Document boost hand state structure
    - _Requirements: 9.1, 9.4_
  
  - [x] 5.4 Register boost availability route


    - Add route registration in `routes()` function
    - _Requirements: 9.2, 9.3_

- [x] 6. Implement Lap History Endpoint





  - [x] 6.1 Create response models for lap history


    - Create `LapHistoryResponse`, `LapRecord`, `CycleSummary` structs
    - Add serialization attributes
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 6.2 Implement `get_lap_history` endpoint handler


    - Create `GET /api/v1/races/{race_uuid}/players/{player_uuid}/lap-history` route
    - Fetch race and find participant
    - Get boost usage history from participant
    - Build lap records from usage history
    - Get cycle summaries using `participant.get_boost_cycle_summaries()`
    - Return history data with lap records and cycle summaries
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 6.3 Add OpenAPI documentation for lap history endpoint

    - Add `#[utoipa::path]` attribute
    - Include example with lap records and cycle summaries
    - _Requirements: 9.1, 9.4_
  
  - [x] 6.4 Register lap history route


    - Add route registration in `routes()` function
    - _Requirements: 9.2, 9.3_

- [x] 7. Implement Comprehensive Error Handling





  - [x] 7.1 Add consistent error responses for all endpoints


    - Implement error handling for player not found (404)
    - Implement error handling for race not found (404)
    - Implement error handling for race not in progress (409)
    - Implement error handling for player already finished (409)
    - Implement error handling for boost card not available (400)
    - Implement error handling for invalid UUID format (400)
    - Use exact error messages specified in requirements
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_
  


  - [x] 7.2 Ensure consistent error response format



    - Use `ErrorResponse` struct across all endpoints


    - Include error code, message, and optional details
    - _Requirements: 9.5_



- [ ] 8. Update OpenAPI Documentation

  - [ ] 8.1 Register all new endpoints in OpenAPI schema
    - Ensure all 6 new endpoints appear in Swagger UI
    - Verify schemas are complete and accurate
    - _Requirements: 9.1_
  
  - [ ] 8.2 Add request/response examples for all endpoints
    - Include realistic example data in OpenAPI docs
    - Show both success and error response examples
    - _Requirements: 9.4_

- [ ] 9. Checkpoint - Verify all endpoints work correctly
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Write Integration Tests for New Endpoints
  - [ ] 10.1 Write integration tests for car data endpoint
    - Test valid player returns complete car data
    - Test invalid player UUID returns 404
    - Test player not in race returns 404
    - Test response includes all required fields
    - _Requirements: 10.1_
  
  - [ ] 10.2 Write integration tests for performance preview endpoint
    - Test calculates correct base performance for Straight lap
    - Test calculates correct base performance for Curve lap
    - Test applies sector ceiling correctly
    - Test calculates boost multipliers correctly (0-4)
    - Test shows correct boost availability
    - Test returns movement probabilities
    - _Requirements: 10.1_
  
  - [ ] 10.3 Write integration tests for turn phase endpoint
    - Test returns "WaitingForPlayers" when actions pending
    - Test returns "AllSubmitted" when all submitted
    - Test returns "Complete" when race finished
    - Test lists correct submitted/pending players
    - _Requirements: 10.4_
  
  - [ ] 10.4 Write integration tests for local view endpoint
    - Test returns 5 sectors centered on player
    - Test handles track wrapping correctly
    - Test includes only visible participants
    - Test returns correct sector occupancy
    - _Requirements: 10.3_
  
  - [ ] 10.5 Write integration tests for boost availability endpoint
    - Test returns correct available cards
    - Test shows correct cycle information
    - Test updates after card usage
    - Test handles replenishment correctly
    - _Requirements: 10.2_
  
  - [ ] 10.6 Write integration tests for lap history endpoint
    - Test returns all lap records
    - Test includes boost cycle information
    - Test returns cycle summaries
    - Test handles empty history
    - _Requirements: 10.1_
  
  - [ ] 10.7 Write integration tests for error handling
    - Test invalid UUIDs return 400
    - Test missing race returns 404
    - Test missing player returns 404
    - Test race not in progress returns 409
    - Test player finished returns 409
    - _Requirements: 10.5_

- [ ] 11. Write Property-Based Tests
  - [ ] 11.1 Write property test for car data response completeness
    - **Property 1: Car data response completeness**
    - **Validates: Requirements 1.2, 1.3, 1.4**
    - For any valid player in a race, verify car data endpoint returns all required fields
  
  - [ ] 11.2 Write property test for performance calculation correctness
    - **Property 2: Performance calculation correctness**
    - **Validates: Requirements 2.2, 2.3, 2.4, 7.2**
    - For any car data and boost value, verify performance preview matches domain logic
  
  - [ ] 11.3 Write property test for turn phase state consistency
    - **Property 3: Turn phase state consistency**
    - **Validates: Requirements 3.2, 3.3, 3.4**
    - For any race state, verify turn phase matches actual race state
  
  - [ ] 11.4 Write property test for local view sector range
    - **Property 4: Local view sector range**
    - **Validates: Requirements 4.2, 4.3**
    - For any player position, verify local view returns exactly 5 sectors with correct wrapping
  
  - [ ] 11.5 Write property test for boost availability consistency
    - **Property 5: Boost availability consistency**
    - **Validates: Requirements 5.2, 5.3, 5.4**
    - For any boost hand state, verify availability endpoint matches boost hand
  
  - [ ] 11.6 Write property test for lap history completeness
    - **Property 6: Lap history completeness**
    - **Validates: Requirements 6.2, 6.3, 6.4**
    - For any participant with lap history, verify endpoint returns complete information
  
  - [ ] 11.7 Write property test for movement probability accuracy
    - **Property 7: Movement probability accuracy**
    - **Validates: Requirements 2.5**
    - For any performance value and sector thresholds, verify movement probability is correct

- [ ] 12. Final Checkpoint - Comprehensive Testing
  - Ensure all tests pass, ask the user if questions arise.
