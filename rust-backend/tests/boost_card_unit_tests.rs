//! Unit tests for boost card system without database dependencies
//! These tests verify the boost card hand management, validation, and business logic
//! using mocked data and in-memory structures instead of requiring `MongoDB`.

use rust_backend::domain::{
    boost_hand_manager::{BoostAvailability, BoostImpactOption, BoostUsageResult},
    BoostHand, BoostUsageRecord, MovementProbability, Race, Sector, SectorType, Track, TyreType,
};
use uuid::Uuid;

// Helper function to create a test track
fn create_test_track() -> Track {
    Track {
        uuid: Uuid::new_v4(),
        name: "Test Track".to_string(),
        sectors: vec![
            Sector {
                id: 0,
                name: "Sector 1".to_string(),
                min_value: 10,
                max_value: 20,
                slot_capacity: Some(5),
                sector_type: SectorType::Straight,
            },
            Sector {
                id: 1,
                name: "Sector 2".to_string(),
                min_value: 15,
                max_value: 25,
                slot_capacity: Some(5),
                sector_type: SectorType::Curve,
            },
        ],
    }
}

// Helper function to create a test race with participants
fn create_test_race_with_participants(participant_count: usize) -> (Race, Vec<Uuid>) {
    let track = create_test_track();
    let mut race = Race::new("Test Race".to_string(), track, 3);

    let mut player_uuids = Vec::new();
    for _ in 0..participant_count {
        let player_uuid = Uuid::new_v4();
        let car_uuid = Uuid::new_v4();
        let pilot_uuid = Uuid::new_v4();
        race.add_participant(player_uuid, car_uuid, pilot_uuid)
            .unwrap();
        player_uuids.push(player_uuid);
    }

    // Set participants to sector 0 and start race
    for participant in &mut race.participants {
        participant.current_sector = 0;
    }
    race.start_race().unwrap();

    (race, player_uuids)
}

#[test]
fn test_boost_hand_initializes_with_medium_pool() {
    // Arrange
    let (race, _player_uuids) = create_test_race_with_participants(1);
    let participant = &race.participants[0];

    // Assert - Default (Medium) pool is [2, 2, 3, 3, 4] -> 5 cards.
    assert_eq!(participant.boost_hand.tyre_type, TyreType::Medium);
    assert_eq!(participant.boost_hand.cards_remaining, 5);
    assert_eq!(participant.boost_hand.pit_stops_completed, 0);

    // Boost 0 (free no-op) plus the Medium pool values are available; 1 is not.
    for value in [0, 2, 3, 4] {
        assert!(participant.boost_hand.is_card_available(value));
    }
    assert!(!participant.boost_hand.is_card_available(1));

    let available_cards = participant.boost_hand.get_available_cards();
    assert_eq!(available_cards, vec![0, 2, 3, 4]);
}

#[test]
fn test_using_boost_card_decrements_count() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use boost card 4 (single copy in the Medium pool)
    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    let result = participant.boost_hand.use_card(4);
    assert!(result.is_ok());

    // Assert - Verify boost hand state updated. The lone value-4 card is gone.
    assert_eq!(participant.boost_hand.cards_remaining, 4);
    assert!(!participant.boost_hand.is_card_available(4));

    let available_cards = participant.boost_hand.get_available_cards();
    // 0 is always present, plus the remaining 2s and 3s.
    assert_eq!(available_cards, vec![0, 2, 3]);
    assert!(!available_cards.contains(&4));
}

#[test]
fn test_duplicate_card_can_be_used_until_depleted() {
    // Arrange - Medium pool has two value-2 cards.
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Use boost card 2 twice (both copies)
    assert!(participant.boost_hand.use_card(2).is_ok());
    // One copy remains, still available.
    assert!(participant.boost_hand.is_card_available(2));
    assert!(participant.boost_hand.use_card(2).is_ok());

    // Now depleted; a third use fails.
    let result3 = participant.boost_hand.use_card(2);
    assert!(result3.is_err());
    assert_eq!(result3.unwrap_err(), "Boost card 2 is not available");

    let available_cards = participant.boost_hand.get_available_cards();
    assert!(!available_cards.contains(&2));
}

#[test]
fn test_boost_zero_is_always_free_and_never_decrements() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Spend the entire Medium pool [2, 2, 3, 3, 4].
    for value in [2, 2, 3, 3, 4] {
        participant.boost_hand.use_card(value).unwrap();
    }

    // Assert - No auto-replenish. Only the free 0 remains.
    assert_eq!(participant.boost_hand.cards_remaining, 0);
    assert_eq!(participant.boost_hand.pit_stops_completed, 0);
    assert_eq!(participant.boost_hand.get_available_cards(), vec![0]);

    // Boost 0 is still usable for free and does NOT decrement cards_remaining.
    assert!(participant.boost_hand.use_card(0).is_ok());
    assert!(participant.boost_hand.use_card(0).is_ok());
    assert_eq!(participant.boost_hand.cards_remaining, 0);
}

#[test]
fn test_pit_refill_restores_pool() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Spend the whole Medium pool, then pit-refill onto Soft tyres.
    for value in [2, 2, 3, 3, 4] {
        participant.boost_hand.use_card(value).unwrap();
    }
    assert_eq!(participant.boost_hand.cards_remaining, 0);

    participant.boost_hand.refill(TyreType::Soft);

    // Assert - Soft pool is [3, 4, 4] -> 3 cards; pit count incremented.
    assert_eq!(participant.boost_hand.tyre_type, TyreType::Soft);
    assert_eq!(participant.boost_hand.cards_remaining, 3);
    assert_eq!(participant.boost_hand.pit_stops_completed, 1);
    assert_eq!(participant.boost_hand.get_available_cards(), vec![0, 3, 4]);

    // A second refill keeps incrementing the pit count.
    participant.boost_hand.refill(TyreType::Medium);
    assert_eq!(participant.boost_hand.tyre_type, TyreType::Medium);
    assert_eq!(participant.boost_hand.cards_remaining, 5);
    assert_eq!(participant.boost_hand.pit_stops_completed, 2);
}

#[test]
fn test_boost_hand_state_persists_across_operations() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use some boost cards (valid Medium-pool values).
    {
        let participant = race
            .participants
            .iter_mut()
            .find(|p| p.player_uuid == player_uuid)
            .unwrap();

        participant.boost_hand.use_card(2).unwrap();
        participant.boost_hand.use_card(3).unwrap();
    }

    // Assert - Verify persisted state is correct
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    assert_eq!(participant.boost_hand.cards_remaining, 3);

    let available_cards = participant.boost_hand.get_available_cards();
    // One 2, one 3, and the lone 4 remain, plus the free 0.
    assert_eq!(available_cards, vec![0, 2, 3, 4]);
}

#[test]
fn test_boost_usage_history_tracks_all_usages() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use several boost cards and track history manually
    let boost_sequence = [2, 0, 4];
    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    for (lap_number, &boost_value) in boost_sequence.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();

        // Manually add to history (simulating what the race engine would do).
        // `cycle_number` is the pit segment (pit_stops_completed at time of use),
        // which is 0 before any pit stop.
        let usage_record = BoostUsageRecord {
            boost_value,
            lap_number: (lap_number + 1) as u32,
            cycle_number: participant.boost_hand.pit_stops_completed,
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: false,
        };
        participant.boost_usage_history.push(usage_record);
    }

    // Assert - Verify usage history
    assert_eq!(participant.boost_usage_history.len(), 3);

    for (i, &boost_value) in boost_sequence.iter().enumerate() {
        assert_eq!(participant.boost_usage_history[i].boost_value, boost_value);
        assert_eq!(participant.boost_usage_history[i].cycle_number, 0);
        assert_eq!(
            participant.boost_usage_history[i].lap_number,
            (i + 1) as u32
        );
    }
}

#[test]
fn test_invalid_boost_value_handling() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act & Assert - Out-of-pool / out-of-range values are unavailable.
    assert!(!participant.boost_hand.is_card_available(1)); // not in Medium pool
    assert!(!participant.boost_hand.is_card_available(5));
    assert!(!participant.boost_hand.is_card_available(10));
    assert!(!participant.boost_hand.is_card_available(255));

    // Free move and Medium-pool values are available.
    for value in [0, 2, 3, 4] {
        assert!(participant.boost_hand.is_card_available(value));
    }
}

#[test]
fn test_boost_availability_response_structure() {
    // Arrange
    let (race, _player_uuids) = create_test_race_with_participants(1);
    let participant = &race.participants[0];

    // Act - Create boost availability response
    let boost_availability = BoostAvailability {
        cards_remaining: participant.boost_hand.cards_remaining,
        tyre_type: participant.boost_hand.tyre_type,
        pit_stops_completed: participant.boost_hand.pit_stops_completed,
        available_cards: participant.boost_hand.get_available_cards(),
        hand_state: participant.boost_hand.cards.clone(),
        boost_impact_preview: (0..=4)
            .map(|boost_value| BoostImpactOption {
                boost_value,
                is_available: participant.boost_hand.is_card_available(boost_value),
                predicted_final_value: u32::from(boost_value) * 10, // Mock calculation
                movement_probability: MovementProbability::Stay,    // Mock value
            })
            .collect(),
    };

    // Assert - Verify response structure
    assert_eq!(boost_availability.cards_remaining, 5);
    assert_eq!(boost_availability.tyre_type, TyreType::Medium);
    assert_eq!(boost_availability.pit_stops_completed, 0);
    assert_eq!(boost_availability.available_cards, vec![0, 2, 3, 4]);
    assert_eq!(boost_availability.boost_impact_preview.len(), 5);

    // Cards 0, 2, 3, 4 available; 1 is not (not in Medium pool).
    for option in &boost_availability.boost_impact_preview {
        let expected = matches!(option.boost_value, 0 | 2 | 3 | 4);
        assert_eq!(option.is_available, expected);
    }
}

#[test]
fn test_pit_segments_track_correctly_in_history() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Spend the whole pool (segment 0), pit-refill, then use more (segment 1).
    let segment0 = [2, 2, 3, 3, 4];
    for (lap_number, &boost_value) in segment0.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();
        participant.boost_usage_history.push(BoostUsageRecord {
            boost_value,
            lap_number: (lap_number + 1) as u32,
            cycle_number: participant.boost_hand.pit_stops_completed,
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: false,
        });
    }

    participant.boost_hand.refill(TyreType::Medium);

    let segment1 = [2, 4];
    for (offset, &boost_value) in segment1.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();
        participant.boost_usage_history.push(BoostUsageRecord {
            boost_value,
            lap_number: (segment0.len() + offset + 1) as u32,
            cycle_number: participant.boost_hand.pit_stops_completed,
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: false,
        });
    }

    // Assert - Segment tagging via cycle_number (= pit_stops_completed).
    assert_eq!(participant.boost_hand.pit_stops_completed, 1);
    assert_eq!(participant.boost_hand.cards_remaining, 3);
    assert_eq!(participant.boost_usage_history.len(), 7);

    for i in 0..5 {
        assert_eq!(participant.boost_usage_history[i].cycle_number, 0);
    }
    for i in 5..7 {
        assert_eq!(participant.boost_usage_history[i].cycle_number, 1);
    }
}

#[test]
fn test_boost_cycle_summaries_calculated_correctly() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Spend the whole Medium pool in a specific order and track history.
    // All uses are in pit segment 0 (no pit stop yet); summaries group by
    // `cycle_number`, which is now the pit segment.
    let boost_sequence = vec![2, 0, 4, 3, 2];
    for (lap_number, &boost_value) in boost_sequence.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();

        let usage_record = BoostUsageRecord {
            boost_value,
            lap_number: (lap_number + 1) as u32,
            cycle_number: participant.boost_hand.pit_stops_completed,
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: false,
        };
        participant.boost_usage_history.push(usage_record);
    }

    // Assert - Verify cycle summary can be calculated
    let cycle_summaries = participant.get_boost_cycle_summaries();
    assert_eq!(cycle_summaries.len(), 1);

    let cycle1 = &cycle_summaries[0];
    assert_eq!(cycle1.cycle_number, 0);
    assert_eq!(cycle1.cards_used, boost_sequence);
    assert_eq!(cycle1.laps_in_cycle, vec![1, 2, 3, 4, 5]);

    // Verify average boost
    let expected_average = (2.0 + 0.0 + 4.0 + 3.0 + 2.0) / 5.0;
    assert!((cycle1.average_boost - expected_average).abs() < 0.01);
}

#[test]
fn test_concurrent_players_have_independent_boost_hands() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(2);
    let player1_uuid = player_uuids[0];
    let player2_uuid = player_uuids[1];

    // Act - Both players use boost card 2 (valid Medium-pool value)
    {
        let participant1 = race
            .participants
            .iter_mut()
            .find(|p| p.player_uuid == player1_uuid)
            .unwrap();
        participant1.boost_hand.use_card(2).unwrap();
    }

    {
        let participant2 = race
            .participants
            .iter_mut()
            .find(|p| p.player_uuid == player2_uuid)
            .unwrap();
        participant2.boost_hand.use_card(2).unwrap();
    }

    // Assert - Both should succeed (separate boost hands)
    let participant1 = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player1_uuid)
        .unwrap();
    let participant2 = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player2_uuid)
        .unwrap();

    // Verify each player's boost hand is independent. One value-2 card remains
    // for each (the pool has two), so 2 is still available.
    assert_eq!(participant1.boost_hand.cards_remaining, 4);
    assert_eq!(participant2.boost_hand.cards_remaining, 4);
    assert!(participant1.boost_hand.is_card_available(2));
    assert!(participant2.boost_hand.is_card_available(2));
}

#[test]
fn test_boost_usage_result_structure() {
    // Arrange
    let mut boost_hand = BoostHand::new();

    // Act - Use a card and create usage result
    boost_hand.use_card(3).unwrap();

    let usage_result = BoostUsageResult {
        boost_value: 3,
        cards_remaining: boost_hand.cards_remaining,
        pit_stops_completed: boost_hand.pit_stops_completed,
    };

    // Assert - Verify result structure
    assert_eq!(usage_result.boost_value, 3);
    assert_eq!(usage_result.cards_remaining, 4);
    assert_eq!(usage_result.pit_stops_completed, 0);
}

#[test]
fn test_boost_hand_serialization_compatibility() {
    // Arrange
    let mut boost_hand = BoostHand::new();
    boost_hand.use_card(2).unwrap();
    boost_hand.use_card(3).unwrap();

    // Act - Verify the hand can be serialized/deserialized (important for database storage)
    let serialized = serde_json::to_string(&boost_hand).expect("Should serialize");
    let deserialized: BoostHand = serde_json::from_str(&serialized).expect("Should deserialize");

    // Assert - Verify state is preserved
    assert_eq!(deserialized.cards_remaining, boost_hand.cards_remaining);
    assert_eq!(deserialized.tyre_type, boost_hand.tyre_type);
    assert_eq!(
        deserialized.pit_stops_completed,
        boost_hand.pit_stops_completed
    );

    for i in 0..=4 {
        assert_eq!(
            deserialized.is_card_available(i),
            boost_hand.is_card_available(i)
        );
    }
}
