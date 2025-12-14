# Implementation Plan

- [x] 1. Set up API service layer and type definitions





  - [x] 1.1 Create TypeScript interfaces for all API responses


    - Create interfaces for CarData, PerformancePreview, TurnPhase, LocalView, BoostAvailability, LapHistory
    - Add type definitions in `empty-project/src/types/race-api.ts`
    - Ensure all fields match backend API response schemas
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6_
  
  - [x] 1.2 Implement RaceAPIService class


    - Create `empty-project/src/services/raceAPI.ts` with RaceAPIService class
    - Implement methods for all 6 GET endpoints (car-data, performance-preview, turn-phase, local-view, boost-availability, lap-history)
    - Implement submitTurnAction POST method
    - Add error handling and response parsing for all methods
    - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5, 11.6, 11.7_
  
  - [x] 1.3 Add error handling utilities


    - Create error categorization function (network, api, validation, state)
    - Implement retry logic with exponential backoff
    - Add error message formatting for user display
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 2. Create RaceContainer main orchestrator component





  - [x] 2.1 Implement RaceContainer component structure


    - Create `empty-project/src/components/player-game-interface/RaceContainer.tsx`
    - Define component props (raceUuid, playerUuid, onRaceComplete, onError)
    - Set up state management for race data, UI state, and error state
    - Add loading states for initial load and API requests
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 2.2 Implement race initialization logic


    - Create initializeRace() method to fetch initial data
    - Fetch car data, local view, and turn phase in parallel
    - Handle initialization errors with user-friendly messages
    - Set up initial UI state based on fetched data
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  

  - [x] 2.3 Implement performance preview fetching

    - Create fetchPerformancePreview() method
    - Call backend performance-preview endpoint
    - Update state with boost options and predictions
    - Handle preview fetch errors
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [x] 2.4 Implement boost selection handling


    - Create handleBoostSelection() method
    - Validate boost is available before allowing selection
    - Update selectedBoost state
    - Provide visual feedback for selection
    - _Requirements: 3.1, 3.2_
  
  - [x] 2.5 Implement turn action submission


    - Create submitTurnAction() method
    - Validate boost selection before submission
    - Call backend submit-action endpoint
    - Handle submission errors with retry options
    - Update UI to show "action submitted" state
    - Disable further selections after successful submission
    - _Requirements: 3.3, 3.4, 3.5, 3.6, 3.7_

- [x] 3. Implement polling and turn completion logic





  - [x] 3.1 Create polling hook for turn phase monitoring


    - Implement useRacePolling custom hook
    - Set up 2-second polling interval
    - Add max attempts limit (60 attempts = 2 minutes)
    - Handle polling errors gracefully
    - Stop polling on component unmount
    - _Requirements: 4.1, 12.1_
  
  - [x] 3.2 Implement turn phase change detection


    - Monitor turn phase transitions (WaitingForPlayers → AllSubmitted → Processing → Complete)
    - Trigger appropriate actions on phase changes
    - Update UI to reflect current phase
    - _Requirements: 4.2_
  
  - [x] 3.3 Implement turn completion handler


    - Create handleTurnComplete() method
    - Fetch updated race state (local view, boost availability, lap history)
    - Update all relevant state atomically
    - Prepare UI for next turn
    - Check for race completion
    - _Requirements: 4.3, 4.4, 4.5, 4.6, 5.1_
  
  - [x] 3.4 Implement race completion detection


    - Create checkRaceCompletion() method
    - Detect when player's is_finished flag is true
    - Stop polling when race is complete
    - Trigger race completion UI
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_


- [x] 4. Create PerformancePreview component





  - [x] 4.1 Implement PerformancePreview component structure

    - Create `empty-project/src/components/player-game-interface/PerformancePreview.tsx`
    - Define props (preview, selectedBoost, onBoostSelect, availableBoosts)
    - Set up component layout with base performance and boost options sections
    - _Requirements: 2.1, 2.2, 2.3, 2.4_
  

  - [x] 4.2 Implement base performance breakdown display

    - Display engine, body, and pilot contributions
    - Show base total calculation
    - Display sector ceiling and capped base value
    - Add visual indicators for lap characteristic
    - _Requirements: 2.2, 2.3_

  

  - [x] 4.3 Implement boost options display

    - Create boost option cards for each boost value (0-4)
    - Display final value with boost multiplier applied
    - Show movement probability indicators (⬆️ MoveUp, ⚪ Stay, ⬇️ MoveDown)
    - Indicate availability status (Available vs Used)
    - Highlight selected boost option

    - _Requirements: 2.4, 2.5, 2.6_

  

  - [x] 4.4 Add boost cycle information display

    - Show current cycle and cards remaining
    - Display next replenishment lap number
    - Add visual progress indicator for cycle completion
    - _Requirements: 2.7_


- [x] 5. Update RaceStatusPanel component




  - [x] 5.1 Add turn phase status display


    - Update RaceStatusPanel to accept turnPhase prop
    - Implement color-coded indicators (Waiting=yellow, AllSubmitted=blue, Processing=orange, Complete=green)
    - Display turn phase description text
    - Add turn phase icon indicators
    - _Requirements: 1.5, 8.1_
  
  - [x] 5.2 Enhance lap information display


    - Display current lap / total laps
    - Add lap characteristic indicator with icons (🏁 Straight, 🌀 Curve)
    - Show lap progress bar
    - _Requirements: 1.2_
  
  - [x] 5.3 Add race status notifications


    - Implement toast notifications for phase changes
    - Add notifications for lap completion
    - Show action submission confirmation
    - _Requirements: 8.4_

- [-] 6. Update BoostSelector component



  - [x] 6.1 Enhance boost selection UI


    - Update BoostSelector to show availability status
    - Disable unavailable boost cards visually
    - Add "Already Used" badges on unavailable cards
    - Highlight selected boost card
    - _Requirements: 3.1, 3.2_
  
  - [x] 6.2 Implement submission controls

    - Add submit button with loading state
    - Disable submit if no boost selected or boost unavailable
    - Show "Action Submitted" state after successful submission
    - Add confirmation dialog for submission
    - _Requirements: 3.3, 3.4, 3.6, 3.7_

- [x] 7. Update LocalSectorDisplay component





  - [x] 7.1 Integrate with backend local-view endpoint


    - Update LocalSectorDisplay to use backend API data
    - Remove any local calculation logic
    - Display 5 sectors from backend response
    - _Requirements: 1.3, 10.1, 10.4_
  
  - [x] 7.2 Enhance sector visualization


    - Highlight player's current sector with distinct styling
    - Show sector occupancy and capacity
    - Display sector type indicators (Start/Straight/Curve/Finish)
    - Add sector value ranges (min/max)
    - _Requirements: 1.3_
  
  - [x] 7.3 Implement participant display


    - Show participant names and car names within sectors
    - Display position within sector
    - Highlight player's car
    - Show participant lap numbers
    - _Requirements: 1.3_
  
  - [x] 7.4 Add movement animations


    - Implement smooth sector transition animations
    - Add position change animations within sectors
    - Use CSS transitions for performance
    - Trigger animations on race state updates
    - _Requirements: 4.4, 8.5_

- [x] 8. Update PlayerCarCard component





  - [x] 8.1 Integrate with backend car-data endpoint


    - Update PlayerCarCard to use backend API data
    - Remove any local calculation logic
    - Display car, pilot, engine, and body information
    - _Requirements: 1.4, 10.1, 10.4_
  
  - [x] 8.2 Enhance pilot information display


    - Show pilot skills breakdown (reaction_time, precision, focus, stamina)
    - Display pilot performance values (straight_value, curve_value)
    - Add pilot class and rarity information
    - _Requirements: 1.4_
  
  - [x] 8.3 Add lap history visualization


    - Integrate lap history data from backend
    - Display lap-by-lap performance chart
    - Show boost usage patterns
    - Add cycle summaries
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

- [x] 9. Implement error handling and user feedback





  - [x] 9.1 Add comprehensive error handling


    - Implement error boundaries for component errors
    - Add network error handling with retry logic
    - Handle API errors with specific messages
    - Implement state inconsistency recovery
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_
  
  - [x] 9.2 Implement user notification system


    - Create toast notification component
    - Add notifications for turn events (submitted, completed, lap finished)
    - Show error notifications with retry options
    - Display success messages for actions
    - _Requirements: 8.4_
  
  - [x] 9.3 Add loading states


    - Show loading spinner during initial race load
    - Display loading state during action submission
    - Add skeleton loaders for data fetching
    - Show polling status indicator
    - _Requirements: 3.4_

- [x] 10. Implement routing and navigation





  - [x] 10.1 Add race play route


    - Create route `/races/:raceUuid/play` in router configuration
    - Integrate RaceContainer component with route
    - Add route protection for authenticated users
    - Extract raceUuid and playerUuid from route and auth context
    - _Requirements: 1.1_
  
  - [x] 10.2 Implement navigation handlers


    - Add navigation to race lobby on race completion
    - Handle navigation on errors (race not found, player not in race)
    - Add "Return to Lobby" button in race completion screen
    - Implement confirmation dialog for leaving active race
    - _Requirements: 7.5, 9.4_

- [x] 11. Add performance optimizations





  - [x] 11.1 Implement caching strategies


    - Cache car data for entire race duration
    - Cache performance preview for current lap
    - Use useMemo for expensive calculations
    - Implement React.memo for expensive components
    - _Requirements: 12.2, 12.4_
  
  - [x] 11.2 Optimize polling and API calls


    - Implement efficient 2-second polling interval
    - Debounce boost selection changes
    - Batch API calls where possible
    - Cancel pending requests on component unmount
    - _Requirements: 12.1, 12.3_
  
  - [x] 11.3 Add lazy loading


    - Lazy load lap history component
    - Lazy load race completion screen
    - Implement code splitting for race components
    - _Requirements: 12.5_

- [x] 12. Checkpoint - Verify complete race flow works





  - Ensure all tests pass, ask the user if questions arise.

- [ ]* 13. Write unit tests for components
  - [ ]* 13.1 Write tests for RaceContainer
    - Test race initialization
    - Test boost selection handling
    - Test turn submission
    - Test polling start/stop
    - Test race completion detection
  
  - [ ]* 13.2 Write tests for PerformancePreview
    - Test base performance display
    - Test boost options rendering
    - Test availability indicators
    - Test movement probability display
  
  - [ ]* 13.3 Write tests for updated components
    - Test RaceStatusPanel turn phase display
    - Test BoostSelector availability validation
    - Test LocalSectorDisplay sector highlighting
    - Test PlayerCarCard data display

- [ ]* 14. Write integration tests
  - [ ]* 14.1 Write API integration tests
    - Mock all backend API endpoints
    - Test complete race flow from start to finish
    - Test error handling for each endpoint
    - Test polling behavior
  
  - [ ]* 14.2 Write user flow tests
    - Test race initialization flow
    - Test turn submission flow
    - Test turn processing flow
    - Test multi-lap flow
    - Test race completion flow
  
  - [ ]* 14.3 Write error scenario tests
    - Test network failure during fetch
    - Test network failure during submission
    - Test invalid boost selection
    - Test race not found
    - Test player already finished

- [ ]* 15. Write property-based tests
  - [ ]* 15.1 Write property test for boost availability display consistency
    - **Property 1: Boost availability display consistency**
    - **Validates: Requirements 2.5**
    - For any boost hand state from backend, verify UI displays matching availability
  
  - [ ]* 15.2 Write property test for boost selection validation
    - **Property 2: Boost selection validation**
    - **Validates: Requirements 3.2**
    - For any boost selection, verify unavailable cards are prevented from submission
  
  - [ ]* 15.3 Write property test for turn phase transition detection
    - **Property 3: Turn phase transition detection**
    - **Validates: Requirements 4.2**
    - For any turn phase sequence, verify UI detects and responds to changes
  
  - [ ]* 15.4 Write property test for race completion detection
    - **Property 4: Race completion detection**
    - **Validates: Requirements 7.1**
    - For any race state with is_finished=true, verify UI shows completion
  
  - [ ]* 15.5 Write property test for turn phase color mapping
    - **Property 5: Turn phase color mapping**
    - **Validates: Requirements 8.1**
    - For any turn phase value, verify correct color indicator is applied
  
  - [ ]* 15.6 Write property test for network error handling
    - **Property 6: Network error handling**
    - **Validates: Requirements 9.1**
    - For any network error, verify user-friendly error message is displayed
  
  - [ ]* 15.7 Write property test for no local game logic
    - **Property 7: No local game logic calculations**
    - **Validates: Requirements 10.1, 10.2, 10.3, 10.4**
    - For any game state display, verify data originates from backend API
  
  - [ ]* 15.8 Write property test for backend API data source
    - **Property 8: Backend API data source**
    - **Validates: Requirements 10.4**
    - For any displayed game information, verify source is backend API endpoint

- [ ] 16. Final Checkpoint - Complete end-to-end testing
  - Ensure all tests pass, ask the user if questions arise.
