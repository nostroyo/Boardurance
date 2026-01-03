//! Unit tests for boost card system without database dependencies
//! These tests verify the boost card hand management, validation, and business logic
//! using mocked data and in-memory structures instead of requiring MongoDB.

use rust_backend::domain::{
    boost_hand_manager::{BoostAvailability, BoostImpactOption, BoostUsageResult},
    BoostHand, BoostUsageRecord, MovementProbability, Race, Sector, SectorType, Track,
};
use std::collections::HashMap;
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
fn test_boost_hand_initializes_with_all_cards_available() {
    // Arrange
    let (race, _player_uuids) = create_test_race_with_participants(1);
    let participant = &race.participants[0];

    // Assert - Verify boost hand is initialized correctly
    assert_eq!(participant.boost_hand.cards_remaining, 5);
    assert_eq!(participant.boost_hand.current_cycle, 1);
    assert_eq!(participant.boost_hand.cycles_completed, 0);

    // Verify all cards 0-4 are available
    for i in 0..=4 {
        assert!(participant.boost_hand.is_card_available(i));
    }

    let available_cards = participant.boost_hand.get_available_cards();
    assert_eq!(available_cards, vec![0, 1, 2, 3, 4]);
}

#[test]
fn test_using_boost_card_marks_it_unavailable() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use boost card 2
    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    let result = participant.boost_hand.use_card(2);
    assert!(result.is_ok());

    // Assert - Verify boost hand state updated
    assert_eq!(participant.boost_hand.cards_remaining, 4);
    assert!(!participant.boost_hand.is_card_available(2));

    let available_cards = participant.boost_hand.get_available_cards();
    assert_eq!(available_cards.len(), 4);
    assert!(!available_cards.contains(&2));
}

#[test]
fn test_cannot_use_same_boost_card_twice() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Use boost card 3
    let result1 = participant.boost_hand.use_card(3);
    assert!(result1.is_ok());

    // Try to use boost card 3 again
    let result2 = participant.boost_hand.use_card(3);

    // Assert - Should return error
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), "Boost card 3 is not available");

    let available_cards = participant.boost_hand.get_available_cards();
    assert!(!available_cards.contains(&3));
}

#[test]
fn test_boost_hand_replenishes_after_all_cards_used() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Use all 5 boost cards
    for boost_value in 0..=4 {
        let result = participant.boost_hand.use_card(boost_value);
        assert!(result.is_ok(), "Failed to use boost card {boost_value}");
    }

    // Assert - Verify replenishment occurred
    assert_eq!(participant.boost_hand.cards_remaining, 5);
    assert_eq!(participant.boost_hand.current_cycle, 2);
    assert_eq!(participant.boost_hand.cycles_completed, 1);

    let available_cards = participant.boost_hand.get_available_cards();
    assert_eq!(available_cards.len(), 5);

    // All cards should be available again
    for i in 0..=4 {
        assert!(participant.boost_hand.is_card_available(i));
    }
}

#[test]
fn test_boost_hand_state_persists_across_operations() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use some boost cards
    {
        let participant = race
            .participants
            .iter_mut()
            .find(|p| p.player_uuid == player_uuid)
            .unwrap();

        participant.boost_hand.use_card(1).unwrap();
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
    assert_eq!(available_cards.len(), 3);
    assert!(available_cards.contains(&0));
    assert!(available_cards.contains(&2));
    assert!(available_cards.contains(&4));
    assert!(!available_cards.contains(&1));
    assert!(!available_cards.contains(&3));
}

#[test]
fn test_boost_usage_history_tracks_all_usages() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    // Act - Use several boost cards and track history manually
    let boost_sequence = vec![2, 0, 4];
    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    for (lap_number, &boost_value) in boost_sequence.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();

        // Manually add to history (simulating what the race engine would do)
        let usage_record = BoostUsageRecord {
            boost_value,
            lap_number: (lap_number + 1) as u32,
            cycle_number: participant.boost_hand.current_cycle,
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: false,
        };
        participant.boost_usage_history.push(usage_record);
    }

    // Assert - Verify usage history
    assert_eq!(participant.boost_usage_history.len(), 3);

    for (i, &boost_value) in boost_sequence.iter().enumerate() {
        assert_eq!(participant.boost_usage_history[i].boost_value, boost_value);
        assert_eq!(participant.boost_usage_history[i].cycle_number, 1);
        assert_eq!(participant.boost_usage_history[i].lap_number, (i + 1) as u32);
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

    // Act & Assert - Try to use invalid boost values
    assert!(!participant.boost_hand.is_card_available(5));
    assert!(!participant.boost_hand.is_card_available(10));
    assert!(!participant.boost_hand.is_card_available(255));

    // Valid cards should still be available
    for i in 0..=4 {
        assert!(participant.boost_hand.is_card_available(i));
    }
}

#[test]
fn test_boost_availability_response_structure() {
    // Arrange
    let (race, player_uuids) = create_test_race_with_participants(1);
    let participant = &race.participants[0];

    // Act - Create boost availability response
    let boost_availability = BoostAvailability {
        cards_remaining: participant.boost_hand.cards_remaining,
        current_cycle: participant.boost_hand.current_cycle,
        cycles_completed: participant.boost_hand.cycles_completed,
        available_cards: participant.boost_hand.get_available_cards(),
        hand_state: {
            let mut hand_state = HashMap::new();
            for i in 0..=4 {
                hand_state.insert(i.to_string(), participant.boost_hand.is_card_available(i));
            }
            hand_state
        },
        next_replenishment_at: Some(5 - participant.boost_hand.cards_remaining),
        boost_impact_preview: (0..=4)
            .map(|boost_value| BoostImpactOption {
                boost_value,
                is_available: participant.boost_hand.is_card_available(boost_value),
                predicted_final_value: boost_value as u32 * 10, // Mock calculation
                movement_probability: MovementProbability::Stay, // Mock value
            })
            .collect(),
    };

    // Assert - Verify response structure
    assert_eq!(boost_availability.cards_remaining, 5);
    assert_eq!(boost_availability.current_cycle, 1);
    assert_eq!(boost_availability.cycles_completed, 0);
    assert_eq!(boost_availability.available_cards.len(), 5);
    assert_eq!(boost_availability.boost_impact_preview.len(), 5);

    // All cards should be available initially
    for option in &boost_availability.boost_impact_preview {
        assert!(option.is_available);
    }
}

#[test]
fn test_multiple_cycles_track_correctly() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(1);
    let player_uuid = player_uuids[0];

    let participant = race
        .participants
        .iter_mut()
        .find(|p| p.player_uuid == player_uuid)
        .unwrap();

    // Act - Complete first cycle
    for boost_value in 0..=4 {
        participant.boost_hand.use_card(boost_value).unwrap();
    }

    // Use some cards from second cycle
    participant.boost_hand.use_card(1).unwrap();
    participant.boost_hand.use_card(4).unwrap();

    // Assert - Verify cycle tracking
    assert_eq!(participant.boost_hand.current_cycle, 2);
    assert_eq!(participant.boost_hand.cycles_completed, 1);
    assert_eq!(participant.boost_hand.cards_remaining, 3);

    let available_cards = participant.boost_hand.get_available_cards();
    assert_eq!(available_cards, vec![0, 2, 3]);
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

    // Act - Complete first cycle with specific sequence and track history
    let boost_sequence = vec![2, 0, 4, 1, 3];
    for (lap_number, &boost_value) in boost_sequence.iter().enumerate() {
        participant.boost_hand.use_card(boost_value).unwrap();

        // Manually add to history
        let usage_record = BoostUsageRecord {
            boost_value,
            lap_number: (lap_number + 1) as u32,
            cycle_number: 1, // First cycle
            cards_remaining_after: participant.boost_hand.cards_remaining,
            replenishment_occurred: lap_number == 4, // Last card triggers replenishment
        };
        participant.boost_usage_history.push(usage_record);
    }

    // Assert - Verify cycle summary can be calculated
    let cycle_summaries = participant.get_boost_cycle_summaries();
    assert_eq!(cycle_summaries.len(), 1);

    let cycle1 = &cycle_summaries[0];
    assert_eq!(cycle1.cycle_number, 1);
    assert_eq!(cycle1.cards_used, boost_sequence);
    assert_eq!(cycle1.laps_in_cycle, vec![1, 2, 3, 4, 5]);

    // Verify average boost
    let expected_average = (2.0 + 0.0 + 4.0 + 1.0 + 3.0) / 5.0;
    assert!((cycle1.average_boost - expected_average).abs() < 0.01);
}

#[test]
fn test_concurrent_players_have_independent_boost_hands() {
    // Arrange
    let (mut race, player_uuids) = create_test_race_with_participants(2);
    let player1_uuid = player_uuids[0];
    let player2_uuid = player_uuids[1];

    // Act - Both players use boost card 2
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

    // Verify each player's boost hand is independent
    assert_eq!(participant1.boost_hand.cards_remaining, 4);
    assert_eq!(participant2.boost_hand.cards_remaining, 4);
    assert!(!participant1.boost_hand.is_card_available(2));
    assert!(!participant2.boost_hand.is_card_available(2));
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
        current_cycle: boost_hand.current_cycle,
        replenishment_occurred: false,
    };

    // Assert - Verify result structure
    assert_eq!(usage_result.boost_value, 3);
    assert_eq!(usage_result.cards_remaining, 4);
    assert_eq!(usage_result.current_cycle, 1);
    assert!(!usage_result.replenishment_occurred);
}

#[test]
fn test_boost_hand_serialization_compatibility() {
    // Arrange
    let mut boost_hand = BoostHand::new();
    boost_hand.use_card(1).unwrap();
    boost_hand.use_card(3).unwrap();

    // Act - Verify the hand can be serialized/deserialized (important for database storage)
    let serialized = serde_json::to_string(&boost_hand).expect("Should serialize");
    let deserialized: BoostHand =
        serde_json::from_str(&serialized).expect("Should deserialize");

    // Assert - Verify state is preserved
    assert_eq!(deserialized.cards_remaining, boost_hand.cards_remaining);
    assert_eq!(deserialized.current_cycle, boost_hand.current_cycle);
    assert_eq!(deserialized.cycles_completed, boost_hand.cycles_completed);

    for i in 0..=4 {
        assert_eq!(
            deserialized.is_card_available(i),
            boost_hand.is_card_available(i)
        );
    }
}