# Requirements Document

## Introduction

This document specifies the requirements for refining the backend race API to support a streamlined player-centric racing experience. The system will provide specific API endpoints for player registration to races, real-time race status tracking with detailed track situation data, and individual lap processing with boost selection. This refinement focuses on creating a smooth frontend-backend interaction pattern for the Web3 Racing Game.

## Glossary

- **Race_Registration_System**: Backend service handling player registration to specific races using race UUIDs
- **Race_Status_API**: Real-time API endpoint providing comprehensive race state including lap progress, track situation, and player-specific data
- **Lap_Processing_API**: Individual lap processing endpoint that accepts player actions and returns updated race state
- **Track_Situation_Data**: JSON response containing all sectors with current participant positions and race dynamics
- **Player_Boost_Availability**: System tracking and validating available boost options for each player per lap
- **Race_UUID**: Unique identifier for each race instance used across all API interactions
- **Car_UUID**: Unique identifier for player's car participating in the race
- **Lap_Characteristic**: Current lap's performance focus (Straight or Curve) affecting all calculations
- **Sector_Position_Data**: Detailed information about participant positions within each track sector
- **Race_Progress_Status**: Current state of race (Ongoing with lap number, Error states, Finished status)

## Requirements

### Requirement 1

**User Story:** As a player, I want to register for a race using the race UUID, so that I can participate in the racing competition with my selected car and pilot.

#### Acceptance Criteria

1. WHEN a player calls the register_player endpoint with race UUID, THE Race_Registration_System SHALL validate the race exists and is accepting participants
2. WHEN registering for a race, THE Race_Registration_System SHALL require player UUID and car UUID as mandatory parameters (pilot UUID is retrieved from car association)
3. WHEN validating car registration, THE Race_Registration_System SHALL verify the car has associated body, engine, and pilot components before allowing registration
4. WHEN a valid registration request is received, THE Race_Registration_System SHALL add the player to the race participants list
5. IF the race is already in progress or finished, THEN THE Race_Registration_System SHALL return an error status indicating registration is not allowed
6. WHEN registration is successful, THE Race_Registration_System SHALL return confirmation with the player's starting position and race details

### Requirement 2

**User Story:** As a frontend application, I want to query race status using the race UUID, so that I can display current race progress, track situation, and available player actions.

#### Acceptance Criteria

1. WHEN the frontend requests race status, THE Race_Status_API SHALL return the current race progress status (Ongoing with lap number, Error states, or Finished)
2. WHEN the race is ongoing, THE Race_Status_API SHALL provide complete track situation data including all sectors with participant positions in JSON format
3. WHEN providing track situation, THE Race_Status_API SHALL include sector capacity, current occupants, and position rankings within each sector
4. WHEN a specific player requests status, THE Race_Status_API SHALL include player-specific boost availability and current performance metrics
5. WHEN the race status changes, THE Race_Status_API SHALL reflect updates within 2 seconds of any race state modification

### Requirement 3

**User Story:** As a player, I want to submit my lap action with boost selection, so that I can influence my car's performance and advance through the race.

#### Acceptance Criteria

1. WHEN a player calls ApplyLap with race UUID and car UUID, THE Lap_Processing_API SHALL validate the player is registered in the specified race
2. WHEN submitting lap action, THE Lap_Processing_API SHALL require boost value selection (0-5) as a mandatory parameter
3. WHEN processing the lap action, THE Lap_Processing_API SHALL calculate performance based on car stats, pilot skills, current sector ceiling, and selected boost
4. WHEN lap processing is complete, THE Lap_Processing_API SHALL increment the lap number and return updated race status identical to the status endpoint response
5. WHEN all participants have submitted actions, THE Lap_Processing_API SHALL process simultaneous turn resolution and update all participant positions

### Requirement 4

**User Story:** As a race participant, I want to receive detailed track situation data, so that I can understand the current competitive landscape and make strategic decisions.

#### Acceptance Criteria

1. WHEN requesting track situation, THE Race_Status_API SHALL provide sector-by-sector participant information including player names, car details, and current positions
2. WHEN displaying sector data, THE Race_Status_API SHALL include sector capacity limits, current occupancy count, and available slots
3. WHEN showing participant positions, THE Race_Status_API SHALL rank participants within each sector by their total accumulated performance values
4. WHEN track situation updates, THE Race_Status_API SHALL include movement indicators showing recent sector changes and position improvements
5. WHERE participants have finished the race, THE Race_Status_API SHALL display final positions and completion status separately from active participants

### Requirement 5

**User Story:** As a player, I want to know my available boost options for each lap, so that I can make informed strategic decisions about performance enhancement.

#### Acceptance Criteria

1. WHEN a player requests race status, THE Race_Status_API SHALL include current boost availability (0-5 range) for the requesting player
2. WHEN displaying boost options, THE Race_Status_API SHALL show the performance impact preview based on current sector ceiling and car capabilities
3. WHEN boost has been used in previous laps, THE Race_Status_API SHALL maintain boost availability according to game rules (unlimited per lap)
4. WHEN calculating boost impact, THE Race_Status_API SHALL apply sector performance ceiling to base value before adding boost value
5. WHERE boost selection affects movement probability, THE Race_Status_API SHALL provide performance threshold indicators for sector advancement

### Requirement 6

**User Story:** As a race system, I want to process individual lap actions and return consistent status data, so that the frontend receives uniform race information regardless of the API endpoint used.

#### Acceptance Criteria

1. WHEN processing individual lap actions, THE Lap_Processing_API SHALL return race status data in the same format as the dedicated status endpoint
2. WHEN lap processing completes, THE Lap_Processing_API SHALL include updated participant positions, sector occupancy, and race progress information
3. WHEN returning processed lap results, THE Lap_Processing_API SHALL maintain data consistency with concurrent status requests from other players
4. WHEN multiple players submit actions simultaneously, THE Lap_Processing_API SHALL ensure atomic processing and consistent state updates
5. WHEN lap processing encounters errors, THE Lap_Processing_API SHALL return detailed error information while maintaining race state integrity

### Requirement 7

**User Story:** As a frontend developer, I want consistent JSON response formats across all race API endpoints, so that I can implement reliable data parsing and display logic.

#### Acceptance Criteria

1. WHEN any race API endpoint returns data, THE Race_Registration_System SHALL use standardized JSON schemas for all response types
2. WHEN providing track situation data, THE Race_Status_API SHALL format sector information, participant data, and race metadata consistently
3. WHEN returning error responses, THE Race_Registration_System SHALL include error codes, descriptive messages, and suggested resolution actions
4. WHEN race status changes, THE Race_Status_API SHALL maintain backward compatibility with existing frontend implementations
5. WHERE new data fields are added, THE Race_Registration_System SHALL ensure optional field handling and graceful degradation for older clients

### Requirement 8

**User Story:** As a race administrator, I want to track race progression and participant engagement, so that I can monitor race health and identify potential issues.

#### Acceptance Criteria

1. WHEN races are in progress, THE Race_Status_API SHALL provide race metadata including start time, current lap, total laps, and estimated completion time
2. WHEN participants interact with the race, THE Race_Registration_System SHALL log registration events, lap submissions, and status requests for monitoring
3. WHEN race errors occur, THE Race_Registration_System SHALL capture detailed error context including participant state, race conditions, and system status
4. WHEN races complete, THE Race_Status_API SHALL provide comprehensive race summary including final standings, lap-by-lap performance, and participation statistics
5. WHERE race performance issues are detected, THE Race_Registration_System SHALL generate alerts for administrative review and intervention