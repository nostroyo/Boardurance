# Requirements Document

## Introduction

This document specifies the requirements for implementing a boost card management system in the Web3 Racing Game. The current system provides unlimited boost selection (0-4) per lap, which reduces strategic depth and decision-making. This feature will introduce a hand-based boost system where players have 5 specific boost cards (0, 1, 2, 3, 4) that become unavailable after use until all cards are exhausted, at which point the entire hand replenishes. This creates meaningful tactical decisions about boost timing and sequencing.

## Glossary

- **Boost_Hand**: Set of 5 boost cards (values 0, 1, 2, 3, 4) available to each player
- **Boost_Card**: Individual boost option that can be used once per cycle
- **Boost_Card_State**: Tracking system indicating which boost cards are available or used
- **Boost_Cycle**: Period from full hand to empty hand, after which all cards replenish
- **Boost_Replenishment**: Automatic restoration of all boost cards when the hand is completely exhausted
- **Race_Participant_State**: Extended participant data structure tracking available boost cards and usage history
- **Boost_Selection_Constraint**: Validation ensuring players can only select available (unused) boost cards
- **Boost_Usage_History**: Record of which boost cards were used in which order during the race
- **Hand_Depletion_Indicator**: UI element showing how many boost cards remain before replenishment

## Requirements

### Requirement 1

**User Story:** As a player, I want to have 5 specific boost cards (0, 1, 2, 3, 4) that I can use once each, so that I must make strategic decisions about which boost to use and when.

#### Acceptance Criteria

1. WHEN a race starts, THE Boost_Hand SHALL initialize each participant with 5 available boost cards with values 0, 1, 2, 3, and 4
2. WHEN a player selects a boost card for a lap action, THE Boost_Card_State SHALL mark that specific boost card as used and unavailable
3. WHEN displaying boost options, THE Boost_Selection_Constraint SHALL show only boost cards that have not been used in the current cycle
4. WHEN a player attempts to select an already-used boost card, THE Race_Participant_State SHALL reject the action and return an error indicating the card is unavailable
5. WHEN all 5 boost cards have been used, THE Boost_Hand SHALL remain empty until replenishment occurs

### Requirement 2

**User Story:** As a player, I want all my boost cards to replenish automatically when I've used all of them, so that I can continue making strategic boost decisions throughout the race.

#### Acceptance Criteria

1. WHEN all 5 boost cards have been used, THE Boost_Replenishment SHALL immediately restore all boost cards to available state
2. WHEN replenishment occurs, THE Boost_Card_State SHALL reset all cards (0, 1, 2, 3, 4) to unused status
3. WHEN a new boost cycle begins, THE Race_Participant_State SHALL track the cycle number for statistics and display
4. WHEN replenishment happens, THE Boost_Usage_History SHALL record the completion of a boost cycle
5. WHERE a player finishes the race, THE Boost_Replenishment SHALL stop occurring for that participant

### Requirement 3

**User Story:** As a player, I want to see which boost cards are available and which have been used, so that I can plan my remaining boost strategy for the current cycle.

#### Acceptance Criteria

1. WHEN requesting race status, THE Race_Participant_State SHALL include the availability state of all 5 boost cards
2. WHEN displaying boost options, THE Boost_Card_State SHALL visually distinguish between available and used boost cards
3. WHEN showing boost hand status, THE Hand_Depletion_Indicator SHALL display how many cards remain before replenishment
4. WHEN a boost card is used, THE Race_Participant_State SHALL update the boost hand display immediately
5. WHERE replenishment occurs, THE Boost_Hand SHALL provide visual feedback indicating all cards are available again

### Requirement 4

**User Story:** As a player, I want to see my boost usage history, so that I can review my strategic decisions and understand my boost patterns.

#### Acceptance Criteria

1. WHEN a boost card is used, THE Boost_Usage_History SHALL record the lap number, boost value, and cycle number
2. WHEN requesting race status, THE Race_Participant_State SHALL include complete boost usage history for the current race
3. WHEN displaying usage history, THE Boost_Usage_History SHALL group boost usage by cycle for clarity
4. WHEN a cycle completes, THE Boost_Usage_History SHALL mark the cycle completion event
5. WHERE multiple cycles have occurred, THE Race_Participant_State SHALL show total cycles completed and current cycle progress

### Requirement 5

**User Story:** As a race system, I want to validate boost card selection, so that players cannot use unavailable boost cards or exploit the system.

#### Acceptance Criteria

1. WHEN processing a lap action, THE Boost_Selection_Constraint SHALL verify the selected boost card is currently available
2. WHEN validation fails, THE Race_Participant_State SHALL return a specific error code indicating which boost card was invalid
3. WHEN a boost card is successfully used, THE Boost_Card_State SHALL atomically update the card state to prevent race conditions
4. WHERE concurrent lap submissions occur, THE Boost_Selection_Constraint SHALL ensure consistent boost card state across all operations
5. WHEN a player attempts to use the same boost card twice in one cycle, THE Race_Participant_State SHALL reject the second attempt

### Requirement 6

**User Story:** As a frontend developer, I want clear boost card state data in API responses, so that I can display accurate boost availability to players.

#### Acceptance Criteria

1. WHEN returning race status, THE Race_Participant_State SHALL include a boost_hand object with availability for each card (0-4)
2. WHEN providing boost options, THE Boost_Card_State SHALL use a consistent format indicating available cards as boolean or status flags
3. WHEN boost state changes, THE Race_Participant_State SHALL include the updated boost hand in the lap processing response
4. WHERE replenishment occurs, THE Boost_Replenishment SHALL include a flag indicating a new cycle has started
5. WHEN displaying boost data, THE Race_Participant_State SHALL include cycle count and cards remaining in current cycle

### Requirement 7

**User Story:** As a race analyst, I want to track boost card usage patterns and statistics, so that game balance and player strategies can be analyzed.

#### Acceptance Criteria

1. WHEN a race completes, THE Boost_Usage_History SHALL provide summary statistics including total cycles completed
2. WHEN analyzing boost patterns, THE Race_Participant_State SHALL track which boost cards are used most frequently
3. WHEN generating race reports, THE Boost_Usage_History SHALL include average boost value per lap and per cycle
4. WHERE players complete multiple cycles, THE Race_Participant_State SHALL calculate boost efficiency metrics per cycle
5. WHEN comparing players, THE Boost_Usage_History SHALL provide boost sequencing patterns for strategic analysis

### Requirement 8

**User Story:** As a game designer, I want the boost card system to work seamlessly with existing race mechanics, so that the feature integrates naturally without breaking current functionality.

#### Acceptance Criteria

1. WHEN calculating performance, THE Race_Participant_State SHALL use the selected boost card value in the existing performance calculation formula
2. WHEN processing lap actions, THE Boost_Card_State SHALL integrate with the current lap processing workflow without requiring major refactoring
3. WHEN displaying boost impact preview, THE Boost_Selection_Constraint SHALL only show previews for available boost cards
4. WHERE boost cards are unavailable, THE Race_Participant_State SHALL clearly communicate which cards can be selected
5. WHEN migrating existing races, THE Boost_Hand SHALL initialize with appropriate default state for backward compatibility
