# Requirements Document

## Introduction

The Single Player Race MVP enables a complete end-to-end racing experience where a single user can participate in a race from start to finish. This feature integrates the backend race API enhancements with the frontend player game interface to create a functional racing game where players can select boost cards each turn, navigate through sectors, and complete races. The MVP focuses on the core racing loop without multiplayer synchronization, providing the foundation for future multi-player features.

## Glossary

- **Race_MVP**: Minimum viable product for single-player racing experience
- **Turn_Loop**: The repeating cycle of boost selection, action submission, and race state update
- **Frontend_Integration**: Connection between React frontend and Rust backend race APIs
- **Race_Flow**: Complete sequence from race start through lap progression to race completion
- **Boost_Selection_UI**: User interface for selecting and submitting boost card choices
- **Race_State_Display**: Visual representation of current race position, lap, and sector information
- **Performance_Preview**: Backend-calculated prediction of performance for each boost option
- **Local_View**: Player's visible race area showing current sector ±2 sectors
- **Turn_Submission**: Process of sending boost selection to backend and receiving updated race state
- **Race_Completion**: Final state when player crosses finish line after completing all laps

## Requirements

### Requirement 1

**User Story:** As a player, I want to start a race and see my initial position, so that I understand where I am on the track.

#### Acceptance Criteria

1. WHEN a player navigates to a race, THE Frontend SHALL fetch and display the race state from the backend
2. THE Frontend SHALL display the player's current sector, lap number, and lap characteristic
3. THE Frontend SHALL show the local view (current sector ±2 sectors) with sector details
4. THE Frontend SHALL display the player's car information including pilot, engine, and body stats
5. THE Frontend SHALL indicate the current turn phase (WaitingForPlayers, AllSubmitted, Processing, Complete)

### Requirement 2

**User Story:** As a player, I want to see performance predictions for each boost option, so that I can make informed decisions about which boost card to use.

#### Acceptance Criteria

1. WHEN viewing boost options, THE Frontend SHALL request performance preview from the backend for all boost cards (0-4)
2. THE Frontend SHALL display base performance calculation breakdown (engine + body + pilot contributions)
3. THE Frontend SHALL show sector ceiling application and capped base value
4. THE Frontend SHALL display final performance value for each boost option with multiplier applied
5. THE Frontend SHALL indicate which boost cards are available vs already used in the current cycle
6. THE Frontend SHALL show movement probability (MoveUp, Stay, MoveDown) for each boost option
7. THE Frontend SHALL display boost cycle information (current cycle, cards remaining, next replenishment)

### Requirement 3

**User Story:** As a player, I want to select a boost card and submit my turn action, so that I can progress through the race.

#### Acceptance Criteria

1. THE Frontend SHALL provide a boost selection interface with options 0-4
2. WHEN a player selects a boost card, THE Frontend SHALL validate the card is available before submission
3. WHEN a player submits a boost selection, THE Frontend SHALL send the action to the backend via POST request
4. THE Frontend SHALL display loading state during action submission
5. THE Frontend SHALL handle submission errors with clear error messages and retry options
6. WHEN submission succeeds, THE Frontend SHALL update to show "action submitted" state
7. THE Frontend SHALL disable boost selection after successful submission until next turn

### Requirement 4

**User Story:** As a player, I want to see my position update after submitting my turn, so that I know the result of my boost choice.

#### Acceptance Criteria

1. WHEN the backend processes the turn, THE Frontend SHALL poll for updated race state
2. THE Frontend SHALL detect when turn phase changes from "WaitingForPlayers" to "Processing" to "Complete"
3. THE Frontend SHALL fetch updated local view showing new sector positions
4. THE Frontend SHALL display movement animation when player changes sectors
5. THE Frontend SHALL update lap number when player completes a lap
6. THE Frontend SHALL show updated boost hand state after turn processing

### Requirement 5

**User Story:** As a player, I want to continue racing through multiple laps, so that I can complete the entire race.

#### Acceptance Criteria

1. WHEN a turn completes, THE Frontend SHALL automatically prepare for the next turn
2. THE Frontend SHALL update lap characteristic display when lap changes
3. THE Frontend SHALL refresh performance preview for the new lap characteristic
4. THE Frontend SHALL show boost card replenishment when cycle completes (every 5 cards)
5. THE Frontend SHALL continue the turn loop until race completion

### Requirement 6

**User Story:** As a player, I want to see my lap history and performance trends, so that I can understand my racing strategy effectiveness.

#### Acceptance Criteria

1. THE Frontend SHALL fetch and display lap-by-lap performance history from the backend
2. THE Frontend SHALL show each lap's boost used, base value, and final value
3. THE Frontend SHALL display movement information (from_sector, to_sector, movement_type) for each lap
4. THE Frontend SHALL show boost cycle summaries with average boost per cycle
5. THE Frontend SHALL update lap history after each turn completion

### Requirement 7

**User Story:** As a player, I want to know when I've finished the race, so that I can see my final results.

#### Acceptance Criteria

1. WHEN a player crosses the finish line, THE Frontend SHALL detect race completion
2. THE Frontend SHALL display race completion message with final position
3. THE Frontend SHALL show complete lap history and performance summary
4. THE Frontend SHALL disable boost selection and turn submission
5. THE Frontend SHALL provide option to return to race lobby or start new race

### Requirement 8

**User Story:** As a player, I want clear visual feedback during the race, so that I understand what's happening at each stage.

#### Acceptance Criteria

1. THE Frontend SHALL use color-coded indicators for turn phase status
2. THE Frontend SHALL highlight the player's current sector in the local view
3. THE Frontend SHALL show loading spinners during API requests
4. THE Frontend SHALL display toast notifications for important events (turn submitted, lap completed, race finished)
5. THE Frontend SHALL use animations for sector transitions and position changes

### Requirement 9

**User Story:** As a player, I want the interface to handle errors gracefully, so that temporary issues don't ruin my race experience.

#### Acceptance Criteria

1. WHEN network errors occur, THE Frontend SHALL display user-friendly error messages
2. THE Frontend SHALL implement retry logic with exponential backoff for failed API requests
3. WHEN race state becomes inconsistent, THE Frontend SHALL refresh from backend
4. THE Frontend SHALL handle race not found errors with navigation back to lobby
5. THE Frontend SHALL log errors for debugging while showing helpful messages to users

### Requirement 10

**User Story:** As a developer, I want the frontend to use backend APIs exclusively for game logic, so that calculations are consistent and secure.

#### Acceptance Criteria

1. THE Frontend SHALL NOT calculate performance values locally
2. THE Frontend SHALL NOT determine movement outcomes locally
3. THE Frontend SHALL NOT manage boost hand state locally
4. THE Frontend SHALL fetch all game logic results from backend APIs
5. THE Frontend SHALL only perform UI state management and display logic

### Requirement 11

**User Story:** As a developer, I want comprehensive integration between frontend and backend, so that the race experience is seamless.

#### Acceptance Criteria

1. THE Frontend SHALL use the car data endpoint to display player's car information
2. THE Frontend SHALL use the performance preview endpoint for boost selection guidance
3. THE Frontend SHALL use the turn phase endpoint to synchronize turn state
4. THE Frontend SHALL use the local view endpoint to display nearby sectors
5. THE Frontend SHALL use the boost availability endpoint to show available cards
6. THE Frontend SHALL use the lap history endpoint to display performance trends
7. THE Frontend SHALL use the submit action endpoint to process turn submissions

### Requirement 12

**User Story:** As a player, I want responsive performance, so that the racing experience feels smooth and immediate.

#### Acceptance Criteria

1. THE Frontend SHALL implement efficient polling with 2-second intervals for race state updates
2. THE Frontend SHALL cache car data for the duration of the race
3. THE Frontend SHALL debounce boost selection changes to avoid excessive API calls
4. THE Frontend SHALL use React.memo and useMemo for expensive component renders
5. THE Frontend SHALL implement lazy loading for non-critical components
