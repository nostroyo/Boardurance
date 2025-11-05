# Requirements Document

## Introduction

The Player Game Interface is the main gameplay UI for the Web3 Racing Game, providing players with an interactive interface to participate in simultaneous turn-based racing competitions using their single NFT car and pilot. This MVP interface focuses on the player's immediate racing environment, showing only their current sector and nearby sectors (±2) to create an immersive, focused racing experience where all players submit boost actions simultaneously before lap resolution.

## Glossary

- **Player_Game_Interface**: The main React component that renders the focused single-car racing experience
- **Local_Race_Viewer**: Component displaying the player's local race view (current sector ±2 sectors)
- **Simultaneous_Turn_Controller**: Interface component allowing players to submit boost actions during simultaneous turn phases
- **Limited_Sector_Display**: Visual representation showing only 5 sectors (player's current ±2) of the track
- **Player_Car_Card**: UI element showing the player's own car and pilot information with performance stats
- **Boost_Selector**: Input component for players to choose their boost value (0-5) for each lap
- **Race_Status_Panel**: Information display showing current lap, lap characteristic, and turn phase status
- **Lap_Characteristic**: The current lap's focus type (Straight or Curve) affecting all car calculations
- **Local_Sector_Movement**: Visual indicators showing participant movement within the visible 5-sector range
- **Performance_Calculator**: Display component showing how the player's car stats, pilot skills, and boost combine for final values
- **Turn_Phase**: The current state of the lap (Waiting for Players, All Submitted, Processing, Complete)

## Requirements

### Requirement 1

**User Story:** As a player, I want to view my local race environment and nearby participants, so that I can understand my immediate competitive situation and make strategic decisions.

#### Acceptance Criteria

1. WHEN a player navigates to an active race, THE Player_Game_Interface SHALL display the local race view showing only the player's current sector plus 2 sectors above and 2 sectors below
2. WHILE viewing the local race state, THE Local_Race_Viewer SHALL show participants within the 5-sector visible range with their positions and basic information
3. THE Limited_Sector_Display SHALL render exactly 5 sectors with their capacity limits, value ranges, and current occupants within the visible range
4. THE Race_Status_Panel SHALL display the current lap number, total laps, current lap characteristic (Straight or Curve), and turn phase status
5. WHEN race data updates, THE Player_Game_Interface SHALL refresh the local view within 2 seconds to reflect the latest state

### Requirement 2

**User Story:** As a player, I want to submit my boost action during the simultaneous turn phase, so that I can influence my car's performance and sector positioning.

#### Acceptance Criteria

1. WHEN the turn phase is "Waiting for Players", THE Simultaneous_Turn_Controller SHALL become active and allow boost selection
2. THE Boost_Selector SHALL provide options from 0 to 5 for the player's boost value
3. WHILE selecting boost, THE Performance_Calculator SHALL show the predicted final value based on the player's car stats, pilot skills, and selected boost
4. WHEN the player confirms their boost selection, THE Simultaneous_Turn_Controller SHALL submit the action to the race API
5. THE Simultaneous_Turn_Controller SHALL disable input and show "Waiting for other players" state after successful submission

### Requirement 3

**User Story:** As a player, I want to see how my car's performance is calculated for each lap, so that I can understand the impact of my car's stats and boost decisions.

#### Acceptance Criteria

1. THE Performance_Calculator SHALL display the base value calculation using the player's engine stats, body stats, and pilot skills for the current lap characteristic
2. WHEN the current lap characteristic is "Straight", THE Performance_Calculator SHALL show straight-focused stat calculations for the player's car
3. WHEN the current lap characteristic is "Curve", THE Performance_Calculator SHALL show curve-focused stat calculations for the player's car
4. THE Performance_Calculator SHALL apply the player's current sector's maximum value ceiling to the base calculation before adding boost
5. THE Performance_Calculator SHALL show the final value as the sum of capped base value and player boost

### Requirement 4

**User Story:** As a player, I want to see visual feedback of sector movements during lap processing within my visible range, so that I can follow the local race action and understand position changes.

#### Acceptance Criteria

1. WHEN a lap is being processed, THE Local_Sector_Movement SHALL display animated transitions for participants moving between sectors within the 5-sector visible range
2. THE Local_Sector_Movement SHALL show upward movement with positive visual indicators for participants advancing to higher sectors within view
3. THE Local_Sector_Movement SHALL show downward movement with appropriate visual indicators for participants dropping to lower sectors within view
4. WHILE participants remain in the same sector, THE Local_Sector_Movement SHALL show position reordering based on total accumulated values
5. THE Local_Sector_Movement SHALL complete all animations before allowing the next turn phase to begin

### Requirement 5

**User Story:** As a player, I want to view detailed information about my own car and pilot, so that I can understand my performance capabilities and make informed boost decisions.

#### Acceptance Criteria

1. THE Player_Car_Card SHALL display detailed specifications of the player's car including engine and body stats for both straight and curve characteristics
2. THE Player_Car_Card SHALL show the player's pilot information including straight and curve skill values
3. THE Player_Car_Card SHALL display the player's performance history including lap-by-lap final values and sector movements
4. WHERE the player's car has moved sectors during the race, THE Player_Car_Card SHALL show the movement history with visual indicators
5. THE Performance_Calculator SHALL allow the player to simulate different boost values to preview potential performance outcomes

### Requirement 6

**User Story:** As a player, I want to receive clear notifications about turn phases and race events, so that I stay informed about when action is required and race progression.

#### Acceptance Criteria

1. WHEN the turn phase changes to "Waiting for Players", THE Player_Game_Interface SHALL display a prominent notification requiring boost selection
2. WHEN the turn phase changes to "Processing", THE Player_Game_Interface SHALL show a processing indicator with lap resolution progress
3. WHEN a lap completes processing, THE Player_Game_Interface SHALL show a summary of local sector movements and position changes within the visible range
4. WHEN the race finishes, THE Player_Game_Interface SHALL display the player's final position and performance summary
5. IF the race API returns an error, THE Player_Game_Interface SHALL show user-friendly error messages with suggested actions

### Requirement 7

**User Story:** As a player, I want a streamlined interface focused on essential race information, so that I can quickly make decisions without information overload.

#### Acceptance Criteria

1. THE Player_Game_Interface SHALL provide a single-view layout optimized for the player's immediate racing needs
2. THE Player_Game_Interface SHALL prioritize the player's car information, local sector view, and boost selection interface
3. THE Race_Status_Panel SHALL show only essential information: current lap, total laps, lap characteristic, and turn phase
4. THE Limited_Sector_Display SHALL focus on the 5-sector local view without overwhelming detail about distant sectors
5. THE Player_Game_Interface SHALL maintain responsive design and fast loading times for optimal racing experience