# Implementation Plan

- [x] 1. Implement core boost hand data structures





  - Create `BoostHand` struct with card state tracking (HashMap<u8, bool>)
  - Implement `new()`, `is_card_available()`, `use_card()`, and `replenish()` methods
  - Add cycle tracking fields (current_cycle, cycles_completed, cards_remaining)
  - Implement `get_available_cards()` helper method
  - _Requirements: 1.1, 1.2, 2.1, 2.2_

- [x] 2. Extend RaceParticipant with boost hand






  - Add `boost_hand: BoostHand` field to `RaceParticipant` struct
  - Update participant initialization to include default boost hand
  - Ensure serialization/deserialization works correctly with MongoDB
  - _Requirements: 1.1, 3.1_

- [x] 3. Create boost hand manager and validation logic





  - [x] 3.1 Implement `BoostHandManager` struct with validation methods


    - Create `validate_boost_selection()` to check card availability
    - Implement `use_boost_card()` to mark cards as used and trigger replenishment
    - Add `get_boost_availability()` for API response generation
    - _Requirements: 1.3, 1.4, 5.1, 5.2_

  - [x] 3.2 Define boost card error types


    - Create `BoostCardError` enum with specific error variants
    - Implement error messages for unavailable cards and invalid values
    - Add error response struct with available cards information
    - _Requirements: 1.4, 6.2_

- [ ] 4. Integrate boost card validation into race domain
  - [ ] 4.1 Update `Race::process_individual_lap_action()` method
    - Add boost card validation before performance calculation
    - Call `BoostHandManager::use_boost_card()` to update hand state
    - Handle boost card errors and return appropriate error messages
    - _Requirements: 1.2, 1.3, 2.1, 8.2_

  - [ ] 4.2 Ensure replenishment triggers correctly
    - Verify replenishment occurs when all 5 cards are used
    - Test cycle counter increments properly
    - Validate all cards become available after replenishment
    - _Requirements: 2.1, 2.2, 2.3_

- [ ] 5. Update API response models
  - [ ] 5.1 Create enhanced `BoostAvailability` struct
    - Add `available_cards` vector showing usable boost values
    - Include `hand_state` HashMap for detailed card status
    - Add cycle information fields (current_cycle, cycles_completed, cards_remaining)
    - Include `next_replenishment_at` indicator
    - _Requirements: 3.1, 3.2, 3.3, 4.1_

  - [ ] 5.2 Update `BoostImpactOption` with availability flag
    - Add `is_available` boolean field
    - Filter impact preview to show only available cards
    - Maintain existing performance prediction logic
    - _Requirements: 3.3, 8.3_

- [ ] 6. Modify race API endpoints
  - [ ] 6.1 Update `apply_lap_action` endpoint
    - Add boost card validation before processing lap
    - Return specific error responses for boost card issues
    - Include updated boost hand state in response
    - _Requirements: 1.3, 1.4, 6.1, 8.2_

  - [ ] 6.2 Update `get_race_status_detailed` endpoint
    - Include boost hand state in player-specific data
    - Show available cards and cycle information
    - Display replenishment countdown
    - _Requirements: 3.1, 3.2, 3.3, 6.1_

- [ ] 7. Implement boost usage history tracking
  - Create `BoostUsageRecord` struct for lap-by-lap tracking
  - Add `BoostCycleSummary` for cycle-level statistics
  - Store usage records in race participant or separate collection
  - Implement history retrieval in status endpoint
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 7.1, 7.2_

- [ ] 8. Add database migration support
  - Create migration script to add `boost_hand` to existing participants
  - Initialize boost hand with default state for existing races
  - Test migration on sample data
  - Document migration process
  - _Requirements: 8.5_

- [ ] 9. Write comprehensive unit tests
  - [ ] 9.1 Test boost hand initialization and basic operations
    - Test `BoostHand::new()` creates correct initial state
    - Test `use_card()` marks cards as unavailable
    - Test `is_card_available()` returns correct status
    - Test `get_available_cards()` returns correct list
    - _Requirements: 1.1, 1.2_

  - [ ] 9.2 Test replenishment logic
    - Test replenishment triggers when all cards used
    - Test cycle counter increments correctly
    - Test all cards become available after replenishment
    - Test multiple cycles work correctly
    - _Requirements: 2.1, 2.2, 2.3, 4.4_

  - [ ] 9.3 Test error handling
    - Test using unavailable card returns error
    - Test using invalid boost value returns error
    - Test error messages include available cards
    - _Requirements: 1.4, 5.1_

- [ ]* 10. Write integration tests
  - Test full lap processing with boost card validation
  - Test boost hand state persists correctly in database
  - Test concurrent lap submissions handle boost cards correctly
  - Test API endpoints return correct boost hand data
  - _Requirements: 5.1, 5.2, 6.1, 8.2_

- [ ] 11. Update API documentation
  - Document new boost hand fields in OpenAPI schema
  - Add examples showing boost card usage flow
  - Document error responses for boost card issues
  - Update endpoint descriptions with boost card behavior
  - _Requirements: 6.1, 6.2, 6.3_

- [ ] 12. Implement backward compatibility handling
  - Add default boost hand initialization for races without it
  - Ensure API responses handle missing boost hand gracefully
  - Test with existing race data
  - _Requirements: 8.5_
