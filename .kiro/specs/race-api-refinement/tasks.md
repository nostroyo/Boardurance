# Implementation Plan

- [x] 1. Set up enhanced race route endpoints





  - Create new endpoint functions in `rust-backend/src/routes/races.rs`
  - Implement `register_player` endpoint for race registration with race UUID
  - Implement `get_race_status_detailed` endpoint for comprehensive race status
  - Implement `apply_lap_action` endpoint for individual lap processing
  - Add proper OpenAPI documentation with utoipa macros
  - _Requirements: 1.1, 1.4, 2.1, 3.1, 6.1_

- [x] 2. Create request/response data models





  - [x] 2.1 Implement registration data structures


    - Create `RegisterPlayerRequest` with player_uuid and car_uuid fields
    - Create `RegisterPlayerResponse` with success status and race position
    - Create `PlayerRacePosition` struct for starting position data
    - _Requirements: 1.2, 1.6_

  - [x] 2.2 Implement status query data structures


    - Create `StatusQueryParams` for optional player-specific data
    - Create `DetailedRaceStatusResponse` with comprehensive race information
    - Create `TrackSituationData` with sector and participant details
    - Create `PlayerSpecificData` for boost availability and performance
    - _Requirements: 2.2, 2.3, 2.4, 5.1, 5.2_

  - [x] 2.3 Implement lap action data structures


    - Create `ApplyLapRequest` with player, car, and boost parameters
    - Ensure boost value validation (0-5 range)
    - _Requirements: 3.2, 3.3, 5.3_

- [x] 3. Implement car validation service





  - [x] 3.1 Create CarValidationService with ownership verification


    - Implement `validate_car_for_race` method with complete component checking
    - Verify car belongs to player making the request
    - Validate car has engine, body, and pilot components
    - Return `ValidatedCarData` struct with all components
    - _Requirements: 1.3, 3.1_



  - [x] 3.2 Add comprehensive car validation error handling





    - Create `CarValidationError` enum with specific error types
    - Handle missing components, ownership issues, and database errors
    - _Requirements: 1.5, 7.3_

- [x] 4. Enhance race processing logic for individual actions


  - [x] 4.1 Implement individual lap action processing





    - Add `process_individual_lap_action` method to Race struct
    - Store pending actions until all players submit
    - Calculate performance using validated car data and boost selection
    - Process simultaneous turn resolution when all actions received
    - _Requirements: 3.3, 3.4, 3.5, 6.3_

  - [x] 4.2 Add performance calculation with car components




    - Implement detailed performance calculation using engine, body, pilot stats
    - Apply sector ceiling limits to base performance
    - Add boost value to final calculation
    - Return `PerformanceCalculation` struct with breakdown
    - _Requirements: 3.3, 5.4, 5.5_

- [ ] 5. Implement comprehensive race status data builders
  - [ ] 5.1 Create race progress status builder
    - Build `RaceProgressStatus` with current lap, total laps, and race state
    - Include participant counts and turn phase information
    - Handle error states and finished race status
    - _Requirements: 2.1, 8.1_

  - [ ] 5.2 Create track situation data builder
    - Build `TrackSituationData` with sector-by-sector participant information
    - Include sector capacity, occupancy, and position rankings
    - Add recent movement indicators and leaderboard data
    - _Requirements: 2.2, 2.3, 4.1, 4.2, 4.3_

  - [ ] 5.3 Create player-specific data builder
    - Build `PlayerSpecificData` with boost availability and performance preview
    - Calculate boost impact options with movement probabilities
    - Include current position and lap history
    - _Requirements: 2.4, 5.1, 5.2, 5.5_

- [ ] 6. Implement database operations for race registration
  - [ ] 6.1 Create player registration database functions
    - Implement `register_player_in_race` function with race validation
    - Check race status and prevent registration for started/finished races
    - Add player to race participants with proper positioning
    - _Requirements: 1.1, 1.4, 1.5_

  - [ ] 6.2 Add race participant management
    - Implement participant lookup and position tracking
    - Handle starting position assignment and qualification ranking
    - _Requirements: 1.6, 4.4_

- [ ] 7. Implement consistent JSON response formatting
  - [ ] 7.1 Create standardized response builders
    - Ensure all endpoints return consistent JSON schemas
    - Implement proper error response formatting with codes and messages
    - Add timestamp and request ID tracking for debugging
    - _Requirements: 7.1, 7.2, 7.3, 7.4_

  - [ ] 7.2 Add comprehensive error handling
    - Create `ApiErrorResponse` and `ApiError` structures
    - Define error code constants for all failure scenarios
    - Implement suggested action guidance in error responses
    - _Requirements: 7.3, 8.3_

- [ ] 8. Add race metadata and monitoring capabilities
  - [ ] 8.1 Implement race metadata tracking
    - Add race timing information (start time, estimated completion)
    - Track participant engagement and interaction logs
    - Include race summary data for completed races
    - _Requirements: 8.1, 8.2, 8.4_

  - [ ] 8.2 Add race health monitoring
    - Log registration events, lap submissions, and status requests
    - Capture error context for debugging and alerts
    - _Requirements: 8.2, 8.3, 8.5_

- [ ] 9. Create comprehensive test coverage
  - [ ] 9.1 Write unit tests for car validation service
    - Test valid car registration with all components
    - Test validation failures for missing components
    - Test car ownership verification
    - _Requirements: 1.3, 3.1_

  - [ ] 9.2 Write API endpoint integration tests
    - Test successful player registration flow
    - Test registration failures for various error conditions
    - Test detailed status endpoint with player-specific data
    - Test individual lap action processing
    - _Requirements: 1.1, 2.1, 3.1, 6.1_

  - [ ] 9.3 Write end-to-end race flow tests
    - Test complete race workflow from registration to completion
    - Test concurrent player actions and status consistency
    - Test error handling and recovery scenarios
    - _Requirements: 3.5, 6.3, 6.4_

- [ ] 10. Wire up new endpoints in application routing
  - Add new race endpoints to the main application router
  - Ensure proper middleware integration (auth, logging, error handling)
  - Update OpenAPI documentation generation
  - Test endpoint accessibility and response formats
  - _Requirements: 7.1, 7.4_