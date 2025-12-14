# Requirements Document

## Introduction

The Backend Race API Enhancements provide critical missing endpoints to support the Player Game Interface frontend. These endpoints ensure proper separation of concerns by keeping all game logic calculations on the backend while providing the frontend with the data it needs to display the player's racing experience. This spec addresses the architectural requirement that the frontend should be a "dumb client" that only displays backend-calculated data.

## Glossary

- **Performance_Preview_API**: Backend endpoint that calculates and returns performance predictions for all boost options
- **Car_Data_API**: Backend endpoint that retrieves complete car, pilot, engine, and body data for a player's race entry
- **Turn_Phase_API**: Backend endpoint that returns the current turn phase state for simultaneous turn resolution
- **Local_View_API**: Backend endpoint that calculates and returns the player's local race view (current sector ±2)
- **Boost_Availability_API**: Backend endpoint that returns which boost cards are currently available for use
- **Performance_Calculation**: Server-side calculation of final performance values using car components and boost multipliers
- **Boost_Card_System**: Server-managed system tracking which boost cards (0-4) have been used in the current cycle
- **Movement_Probability**: Server-calculated prediction of whether a boost choice will result in moving up, staying, or moving down
- **Lap_History_API**: Backend endpoint that returns a player's lap-by-lap performance history for the current race

## Requirements

### Requirement 1

**User Story:** As a frontend developer, I need an API endpoint that provides complete car data for a player's race entry, so that the UI can display car specifications without calculating game logic.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/car-data` that returns complete car information
2. THE Car_Data_API SHALL return the player's car, pilot, engine, and body data with all stat values
3. THE Car_Data_API SHALL include straight and curve performance values for engine, body, and pilot
4. THE Car_Data_API SHALL return pilot skills breakdown (reaction_time, precision, focus, stamina)
5. IF the player is not in the race, THE Car_Data_API SHALL return HTTP 404 with an appropriate error message

### Requirement 2

**User Story:** As a frontend developer, I need an API endpoint that calculates performance previews for all boost options, so that the UI can show accurate predictions without duplicating game logic.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview` that calculates performance for all boost options
2. THE Performance_Preview_API SHALL calculate base performance using the player's engine, body, and pilot stats for the current lap characteristic
3. THE Performance_Preview_API SHALL apply the current sector's maximum value ceiling to the base performance
4. THE Performance_Preview_API SHALL calculate final performance for each boost card (0-4) using the boost multiplier formula: `base * (1.0 + boost * 0.08)`
5. THE Performance_Preview_API SHALL return movement probability (MoveUp, Stay, MoveDown) for each boost option based on sector thresholds
6. THE Performance_Preview_API SHALL indicate which boost cards are available vs already used in the current cycle
7. THE Performance_Preview_API SHALL include boost cycle information (current cycle number, cycles completed, cards remaining)

### Requirement 3

**User Story:** As a frontend developer, I need an API endpoint that returns the current turn phase, so that the UI can show appropriate controls and messaging without managing turn state.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/turn-phase` that returns the current turn phase state
2. THE Turn_Phase_API SHALL return one of: "WaitingForPlayers", "AllSubmitted", "Processing", or "Complete"
3. THE Turn_Phase_API SHALL include a list of player UUIDs who have submitted actions for the current turn
4. THE Turn_Phase_API SHALL include a list of player UUIDs who are still pending action submission
5. THE Turn_Phase_API SHALL include the current lap number and lap characteristic

### Requirement 4

**User Story:** As a frontend developer, I need an API endpoint that calculates the player's local race view, so that the UI can display nearby sectors without implementing view logic.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/local-view` that calculates the 5-sector local view
2. THE Local_View_API SHALL return the player's current sector plus 2 sectors above and 2 sectors below
3. THE Local_View_API SHALL include all participants within the 5-sector visible range with their positions
4. THE Local_View_API SHALL return sector details (id, name, min_value, max_value, slot_capacity, sector_type) for each visible sector
5. THE Local_View_API SHALL handle track wrapping for circular tracks when calculating sector ranges

### Requirement 5

**User Story:** As a frontend developer, I need an API endpoint that returns boost card availability, so that the UI can show which cards are usable without managing boost hand state.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability` that returns current boost hand state
2. THE Boost_Availability_API SHALL return a boolean map of which boost cards (0-4) are currently available
3. THE Boost_Availability_API SHALL return the current cycle number and total cycles completed
4. THE Boost_Availability_API SHALL return the number of cards remaining before automatic replenishment
5. THE Boost_Availability_API SHALL return the lap number when the next replenishment will occur (when cards_remaining reaches 0)

### Requirement 6

**User Story:** As a frontend developer, I need an API endpoint that returns lap history, so that the UI can display performance trends without storing historical data.

#### Acceptance Criteria

1. THE Backend SHALL provide a GET endpoint at `/api/v1/races/{race_uuid}/players/{player_uuid}/lap-history` that returns lap-by-lap performance
2. THE Lap_History_API SHALL return each lap's number, lap characteristic, boost used, base value, and final value
3. THE Lap_History_API SHALL return movement information (from_sector, to_sector, movement_type) for each lap
4. THE Lap_History_API SHALL return boost cycle information showing which cycle each boost card was used in
5. THE Lap_History_API SHALL return cycle summaries showing average boost per cycle and cards used per cycle

### Requirement 7

**User Story:** As a backend developer, I need to ensure all performance calculations use the authoritative boost multiplier formula, so that game balance is consistent and secure.

#### Acceptance Criteria

1. THE Backend SHALL use the boost multiplier formula `final_value = base_value * (1.0 + boost_value * 0.08)` for all performance calculations
2. THE Backend SHALL apply sector ceiling to base performance BEFORE applying the boost multiplier
3. THE Backend SHALL validate that boost values are in the range 0-4 (boost cards) for all API endpoints
4. THE Backend SHALL validate that the selected boost card is available in the player's current boost hand before processing
5. THE Backend SHALL return detailed error messages when boost validation fails (card not available, invalid value, etc.)

### Requirement 8

**User Story:** As a backend developer, I need comprehensive error handling for all new endpoints, so that the frontend receives clear, actionable error messages.

#### Acceptance Criteria

1. WHEN a player is not found in a race, THE Backend SHALL return HTTP 404 with message "Player not found in race"
2. WHEN a race is not found, THE Backend SHALL return HTTP 404 with message "Race not found"
3. WHEN a race is not in progress, THE Backend SHALL return HTTP 409 with message "Race is not in progress"
4. WHEN a player has already finished, THE Backend SHALL return HTTP 409 with message "Player has already finished the race"
5. WHEN a boost card is not available, THE Backend SHALL return HTTP 400 with message "Boost card {value} is not available in current cycle"
6. WHEN invalid UUIDs are provided, THE Backend SHALL return HTTP 400 with message "Invalid UUID format"

### Requirement 9

**User Story:** As a system architect, I want all new endpoints to follow RESTful conventions and OpenAPI documentation standards, so that the API is consistent and well-documented.

#### Acceptance Criteria

1. THE Backend SHALL register all new endpoints in the OpenAPI/Swagger documentation with complete schemas
2. THE Backend SHALL use appropriate HTTP methods (GET for reads, POST for actions)
3. THE Backend SHALL use consistent URL patterns following `/api/v1/races/{race_uuid}/players/{player_uuid}/{resource}` structure
4. THE Backend SHALL include request/response examples in OpenAPI documentation for all endpoints
5. THE Backend SHALL use consistent error response format across all endpoints

### Requirement 10

**User Story:** As a backend developer, I need integration tests for all new endpoints, so that API behavior is verified and regressions are prevented.

#### Acceptance Criteria

1. THE Backend SHALL include integration tests that verify performance preview calculations match the domain logic
2. THE Backend SHALL include integration tests that verify boost card availability tracking across multiple laps
3. THE Backend SHALL include integration tests that verify local view calculation handles track wrapping correctly
4. THE Backend SHALL include integration tests that verify turn phase transitions work correctly
5. THE Backend SHALL include integration tests that verify error responses for all failure scenarios
