use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

use super::race::{BoostHand, MovementProbability, Sector, TyreType};

/// Error types for boost card operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize, ToSchema)]
#[serde(tag = "error_type", content = "details")]
pub enum BoostCardError {
    #[error("Invalid boost value: {0}. Must be between 0 and 4")]
    InvalidBoostValue(u8),

    #[error("Boost card {boost_value} is not available. Available cards: {available_cards:?}")]
    CardNotAvailable {
        boost_value: u8,
        available_cards: Vec<u8>,
    },
}

/// Result of using a boost card
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoostUsageResult {
    pub boost_value: u8,
    pub cards_remaining: u32,
    pub pit_stops_completed: u32,
}

/// Boost availability information for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoostAvailability {
    /// Available boost card values (always includes 0, the free no-boost move)
    pub available_cards: Vec<u8>,

    /// Remaining count per boost card value (keys "1".."4")
    /// Using String keys for `MongoDB` compatibility
    pub hand_state: HashMap<String, u32>,

    /// Currently fitted tyre and pit-stop count
    pub tyre_type: TyreType,
    pub pit_stops_completed: u32,
    pub cards_remaining: u32,

    /// Performance preview for available cards only
    pub boost_impact_preview: Vec<BoostImpactOption>,
}

/// Boost impact option with availability flag
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoostImpactOption {
    pub boost_value: u8,
    pub is_available: bool,
    pub predicted_final_value: u32,
    pub movement_probability: MovementProbability,
}

/// Error response struct with available cards information
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BoostCardErrorResponse {
    pub error_code: String,
    pub message: String,
    pub available_cards: Vec<u8>,
    pub pit_stops_completed: u32,
    pub cards_remaining: u32,
}

impl BoostCardErrorResponse {
    /// Create error response from `BoostCardError`
    #[must_use]
    pub fn from_error(error: &BoostCardError, boost_hand: &BoostHand) -> Self {
        let (error_code, message) = match error {
            BoostCardError::InvalidBoostValue(value) => (
                "INVALID_BOOST_VALUE".to_string(),
                format!("Invalid boost value: {value}. Must be between 0 and 4"),
            ),
            BoostCardError::CardNotAvailable { boost_value, available_cards } => (
                "BOOST_CARD_NOT_AVAILABLE".to_string(),
                format!(
                    "Boost card {boost_value} is not available. Available cards: {available_cards:?}"
                ),
            ),
        };

        Self {
            error_code,
            message,
            available_cards: boost_hand.get_available_cards(),
            pit_stops_completed: boost_hand.pit_stops_completed,
            cards_remaining: boost_hand.cards_remaining,
        }
    }
}

/// Manager for boost hand operations and validation
pub struct BoostHandManager;

impl BoostHandManager {
    /// Validate boost card selection
    ///
    /// Checks if the selected boost card is valid and available in the hand.
    /// Boost value 0 (the free no-boost move) is always valid.
    ///
    /// # Arguments
    /// * `boost_hand` - The player's boost hand
    /// * `boost_value` - The boost card value to validate (0-4)
    ///
    /// # Returns
    /// * `Ok(())` if the card is valid and available
    /// * `Err(BoostCardError)` if validation fails
    pub fn validate_boost_selection(
        boost_hand: &BoostHand,
        boost_value: u8,
    ) -> Result<(), BoostCardError> {
        // Validate boost value is in range 0-4
        if boost_value > 4 {
            return Err(BoostCardError::InvalidBoostValue(boost_value));
        }

        // Check if card is available
        if !boost_hand.is_card_available(boost_value) {
            return Err(BoostCardError::CardNotAvailable {
                boost_value,
                available_cards: boost_hand.get_available_cards(),
            });
        }

        Ok(())
    }

    /// Process boost card usage
    ///
    /// Validates the boost card selection and consumes one matching card.
    /// Boost value 0 is a free no-op. The pool does NOT auto-replenish; only a
    /// pit stop (`BoostHand::refill`) restores cards.
    ///
    /// # Arguments
    /// * `boost_hand` - Mutable reference to the player's boost hand
    /// * `boost_value` - The boost card value to use (0-4)
    ///
    /// # Returns
    /// * `Ok(BoostUsageResult)` with usage details if successful
    /// * `Err(BoostCardError)` if validation fails or card cannot be used
    pub fn use_boost_card(
        boost_hand: &mut BoostHand,
        boost_value: u8,
    ) -> Result<BoostUsageResult, BoostCardError> {
        // Validate first
        Self::validate_boost_selection(boost_hand, boost_value)?;

        // Use the card (boost 0 is a free no-op; the pool does not auto-replenish)
        boost_hand
            .use_card(boost_value)
            .map_err(|_| BoostCardError::CardNotAvailable {
                boost_value,
                available_cards: boost_hand.get_available_cards(),
            })?;

        Ok(BoostUsageResult {
            boost_value,
            cards_remaining: boost_hand.cards_remaining,
            pit_stops_completed: boost_hand.pit_stops_completed,
        })
    }

    /// Get boost availability for API response
    ///
    /// Generates a comprehensive boost availability response including
    /// available cards, hand state, tyre/pit information, and performance
    /// impact preview for each boost option.
    ///
    /// # Arguments
    /// * `boost_hand` - The player's boost hand
    /// * `current_sector` - The sector the player is currently in
    /// * `base_performance` - The player's base performance value (before boost)
    ///
    /// # Returns
    /// * `BoostAvailability` struct with complete boost hand information
    #[must_use]
    pub fn get_boost_availability(
        boost_hand: &BoostHand,
        current_sector: &Sector,
        base_performance: u32,
    ) -> BoostAvailability {
        let available_cards = boost_hand.get_available_cards();

        // Generate impact preview for all boost cards (0-4)
        let boost_impact_preview = (0..=4)
            .map(|boost| {
                let is_available = boost_hand.is_card_available(boost);

                // Calculate predicted final value with boost. Additive model,
                // matching lap resolution: final = min(base, max) + boost.
                let capped_base = std::cmp::min(base_performance, current_sector.max_value);
                let predicted_final = capped_base + u32::from(boost);

                // Calculate movement probability
                let movement_probability =
                    Self::calculate_movement_probability(predicted_final, current_sector);

                BoostImpactOption {
                    boost_value: boost,
                    is_available,
                    predicted_final_value: predicted_final,
                    movement_probability,
                }
            })
            .collect();

        BoostAvailability {
            available_cards,
            hand_state: boost_hand.cards.clone(),
            tyre_type: boost_hand.tyre_type,
            pit_stops_completed: boost_hand.pit_stops_completed,
            cards_remaining: boost_hand.cards_remaining,
            boost_impact_preview,
        }
    }

    /// Calculate movement probability based on performance and sector
    ///
    /// # Arguments
    /// * `final_value` - The final performance value after boost
    /// * `sector` - The current sector
    ///
    /// # Returns
    /// * `MovementProbability` indicating likelihood of moving up, staying, or moving down
    fn calculate_movement_probability(final_value: u32, sector: &Sector) -> MovementProbability {
        if final_value < sector.min_value {
            MovementProbability::MoveDown
        } else if final_value > sector.max_value {
            MovementProbability::MoveUp
        } else {
            MovementProbability::Stay
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::race::SectorType;

    // Medium tyre pool is [2, 2, 3, 3, 4] -> counts {2:2, 3:2, 4:1}, 5 cards.
    fn create_test_boost_hand() -> BoostHand {
        BoostHand::with_tyre(TyreType::Medium)
    }

    fn create_test_sector() -> Sector {
        Sector {
            id: 1,
            name: "Test Sector".to_string(),
            min_value: 10,
            max_value: 20,
            slot_capacity: Some(3),
            sector_type: SectorType::Straight,
        }
    }

    #[test]
    fn test_validate_boost_zero_always_available() {
        let mut hand = create_test_boost_hand();
        // Spend the whole pool; 0 must still validate.
        for value in [2, 2, 3, 3, 4] {
            hand.use_card(value).unwrap();
        }
        assert_eq!(hand.cards_remaining, 0);
        assert!(BoostHandManager::validate_boost_selection(&hand, 0).is_ok());
    }

    #[test]
    fn test_validate_boost_selection_pool_values() {
        let hand = create_test_boost_hand();
        // Values present in the Medium pool (plus the free 0) validate.
        for value in [0, 2, 3, 4] {
            assert!(
                BoostHandManager::validate_boost_selection(&hand, value).is_ok(),
                "Card {value} should be valid"
            );
        }
        // Value 1 is not in the Medium pool.
        assert!(matches!(
            BoostHandManager::validate_boost_selection(&hand, 1).unwrap_err(),
            BoostCardError::CardNotAvailable { boost_value: 1, .. }
        ));
    }

    #[test]
    fn test_validate_boost_selection_invalid_value() {
        let hand = create_test_boost_hand();

        let result = BoostHandManager::validate_boost_selection(&hand, 5);
        assert!(matches!(
            result.unwrap_err(),
            BoostCardError::InvalidBoostValue(5)
        ));

        let result = BoostHandManager::validate_boost_selection(&hand, 10);
        assert!(matches!(
            result.unwrap_err(),
            BoostCardError::InvalidBoostValue(10)
        ));
    }

    #[test]
    fn test_validate_boost_selection_unavailable_card() {
        let mut hand = create_test_boost_hand();

        // Use the single value-4 card; it should now be unavailable.
        hand.use_card(4).unwrap();

        let result = BoostHandManager::validate_boost_selection(&hand, 4);
        match result.unwrap_err() {
            BoostCardError::CardNotAvailable {
                boost_value,
                available_cards,
            } => {
                assert_eq!(boost_value, 4);
                assert!(!available_cards.contains(&4));
                assert!(available_cards.contains(&0), "0 is always available");
            }
            _ => panic!("Expected CardNotAvailable error"),
        }
    }

    #[test]
    fn test_use_boost_card_success() {
        let mut hand = create_test_boost_hand();

        let usage_result = BoostHandManager::use_boost_card(&mut hand, 2).unwrap();
        assert_eq!(usage_result.boost_value, 2);
        assert_eq!(usage_result.cards_remaining, 4);
        assert_eq!(usage_result.pit_stops_completed, 0);

        // One value-2 card remains, so 2 is still available.
        assert!(hand.is_card_available(2));
    }

    #[test]
    fn test_use_boost_card_depletes_duplicate() {
        let mut hand = create_test_boost_hand();

        // Two value-2 cards: spend both.
        BoostHandManager::use_boost_card(&mut hand, 2).unwrap();
        BoostHandManager::use_boost_card(&mut hand, 2).unwrap();

        assert!(!hand.is_card_available(2));
        let err = BoostHandManager::use_boost_card(&mut hand, 2).unwrap_err();
        assert!(matches!(
            err,
            BoostCardError::CardNotAvailable { boost_value: 2, .. }
        ));
    }

    #[test]
    fn test_use_boost_card_no_auto_replenish() {
        let mut hand = create_test_boost_hand();

        // Spend the whole pool.
        for value in [2, 2, 3, 3, 4] {
            BoostHandManager::use_boost_card(&mut hand, value).unwrap();
        }

        assert_eq!(hand.cards_remaining, 0);
        assert_eq!(hand.pit_stops_completed, 0, "No pit stop occurred");
        // Only the free 0 remains.
        assert_eq!(hand.get_available_cards(), vec![0]);
        // Boost 0 is still usable for free.
        assert!(BoostHandManager::use_boost_card(&mut hand, 0).is_ok());
        assert_eq!(hand.cards_remaining, 0);
    }

    #[test]
    fn test_use_boost_card_invalid() {
        let mut hand = create_test_boost_hand();

        let result = BoostHandManager::use_boost_card(&mut hand, 5);
        assert!(matches!(
            result.unwrap_err(),
            BoostCardError::InvalidBoostValue(5)
        ));
    }

    #[test]
    fn test_get_boost_availability() {
        let hand = create_test_boost_hand();
        let sector = create_test_sector();
        let base_performance = 15;

        let availability =
            BoostHandManager::get_boost_availability(&hand, &sector, base_performance);

        assert_eq!(availability.tyre_type, TyreType::Medium);
        assert_eq!(availability.pit_stops_completed, 0);
        assert_eq!(availability.cards_remaining, 5);

        // Preview always covers 0-4.
        assert_eq!(availability.boost_impact_preview.len(), 5);

        for option in &availability.boost_impact_preview {
            let expected_available = matches!(option.boost_value, 0 | 2 | 3 | 4);
            assert_eq!(
                option.is_available, expected_available,
                "Availability mismatch for boost {}",
                option.boost_value
            );
        }
    }

    #[test]
    fn test_get_boost_availability_with_used_cards() {
        let mut hand = create_test_boost_hand();
        let sector = create_test_sector();
        let base_performance = 15;

        // Use the value-4 card.
        hand.use_card(4).unwrap();

        let availability =
            BoostHandManager::get_boost_availability(&hand, &sector, base_performance);

        assert_eq!(availability.cards_remaining, 4);
        assert!(availability.available_cards.contains(&0));
        assert!(!availability.available_cards.contains(&4));

        for option in &availability.boost_impact_preview {
            if option.boost_value == 4 {
                assert!(!option.is_available, "Spent value-4 card is unavailable");
            }
        }
    }

    #[test]
    fn test_calculate_movement_probability() {
        let sector = create_test_sector(); // min: 10, max: 20

        let prob = BoostHandManager::calculate_movement_probability(5, &sector);
        assert!(matches!(prob, MovementProbability::MoveDown));

        let prob = BoostHandManager::calculate_movement_probability(15, &sector);
        assert!(matches!(prob, MovementProbability::Stay));

        let prob = BoostHandManager::calculate_movement_probability(25, &sector);
        assert!(matches!(prob, MovementProbability::MoveUp));
    }

    #[test]
    fn test_boost_card_error_response_from_error() {
        let hand = create_test_boost_hand();

        // InvalidBoostValue error
        let error = BoostCardError::InvalidBoostValue(5);
        let response = BoostCardErrorResponse::from_error(&error, &hand);

        assert_eq!(response.error_code, "INVALID_BOOST_VALUE");
        assert!(response.message.contains("Invalid boost value"));
        assert_eq!(response.pit_stops_completed, 0);
        assert_eq!(response.cards_remaining, 5);

        // CardNotAvailable error
        let mut hand = create_test_boost_hand();
        hand.use_card(4).unwrap();

        let error = BoostCardError::CardNotAvailable {
            boost_value: 4,
            available_cards: hand.get_available_cards(),
        };
        let response = BoostCardErrorResponse::from_error(&error, &hand);

        assert_eq!(response.error_code, "BOOST_CARD_NOT_AVAILABLE");
        assert!(response.message.contains("not available"));
        assert!(!response.available_cards.contains(&4));
    }

    #[test]
    fn test_boost_impact_calculation() {
        let hand = create_test_boost_hand();
        let sector = create_test_sector(); // min: 10, max: 20
        let base_performance = 15;

        let availability =
            BoostHandManager::get_boost_availability(&hand, &sector, base_performance);

        // Additive model: final = min(base, max) + boost. Base 15 (under ceiling 20).
        for option in &availability.boost_impact_preview {
            let expected_value = 15 + u32::from(option.boost_value);
            assert_eq!(option.predicted_final_value, expected_value);
            // Values 15-19 all stay within the 10-20 range.
            assert!(matches!(
                option.movement_probability,
                MovementProbability::Stay
            ));
        }
    }
}
