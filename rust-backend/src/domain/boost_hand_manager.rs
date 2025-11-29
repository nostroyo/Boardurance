use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::collections::HashMap;

use super::race::{BoostHand, Sector, MovementProbability};

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
    pub current_cycle: u32,
    pub replenishment_occurred: bool,
}

/// Boost availability information for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BoostAvailability {
    /// Available boost card values
    pub available_cards: Vec<u8>,
    
    /// Full hand state (for detailed view)
    /// Using String keys for MongoDB compatibility
    pub hand_state: HashMap<String, bool>,
    
    /// Current cycle information
    pub current_cycle: u32,
    pub cycles_completed: u32,
    pub cards_remaining: u32,
    
    /// Replenishment indicator (cards remaining until replenish)
    pub next_replenishment_at: Option<u32>,
    
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
    pub current_cycle: u32,
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
            current_cycle: boost_hand.current_cycle,
            cards_remaining: boost_hand.cards_remaining,
        }
    }
}

/// Manager for boost hand operations and validation
pub struct BoostHandManager;

impl BoostHandManager {
    /// Validate boost card selection
    /// 
    /// Checks if the selected boost card is valid and available in the hand
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
    /// Validates the boost card selection, marks it as used, and triggers
    /// replenishment if all cards have been used.
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
        
        // Track state before using card
        let cards_before = boost_hand.cards_remaining;
        
        // Use the card
        boost_hand.use_card(boost_value)
            .map_err(|_| BoostCardError::CardNotAvailable {
                boost_value,
                available_cards: boost_hand.get_available_cards(),
            })?;
        
        // Check if replenishment occurred
        // Replenishment happens when all cards were used (cards_before was 1)
        // and now cards_remaining is 5 again
        let replenishment_occurred = cards_before == 1 && boost_hand.cards_remaining == 5;
        
        Ok(BoostUsageResult {
            boost_value,
            cards_remaining: boost_hand.cards_remaining,
            current_cycle: boost_hand.current_cycle,
            replenishment_occurred,
        })
    }
    
    /// Get boost availability for API response
    /// 
    /// Generates a comprehensive boost availability response including
    /// available cards, hand state, cycle information, and performance
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
                
                // Calculate predicted final value with boost
                let capped_base = std::cmp::min(base_performance, current_sector.max_value);
                let boost_multiplier = 1.0 + (f64::from(boost) * 0.08);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let predicted_final = (f64::from(capped_base) * boost_multiplier).round() as u32;
                
                // Calculate movement probability
                let movement_probability = Self::calculate_movement_probability(
                    predicted_final,
                    current_sector,
                );
                
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
            current_cycle: boost_hand.current_cycle,
            cycles_completed: boost_hand.cycles_completed,
            cards_remaining: boost_hand.cards_remaining,
            next_replenishment_at: if boost_hand.cards_remaining > 0 {
                Some(boost_hand.cards_remaining)
            } else {
                None
            },
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
    fn calculate_movement_probability(
        final_value: u32,
        sector: &Sector,
    ) -> MovementProbability {
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

    fn create_test_boost_hand() -> BoostHand {
        BoostHand::new()
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
    fn test_validate_boost_selection_valid() {
        let hand = create_test_boost_hand();
        
        // All cards should be valid initially
        for i in 0..=4 {
            let result = BoostHandManager::validate_boost_selection(&hand, i);
            assert!(result.is_ok(), "Card {} should be valid", i);
        }
    }

    #[test]
    fn test_validate_boost_selection_invalid_value() {
        let hand = create_test_boost_hand();
        
        // Test invalid boost values
        let result = BoostHandManager::validate_boost_selection(&hand, 5);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoostCardError::InvalidBoostValue(5)));
        
        let result = BoostHandManager::validate_boost_selection(&hand, 10);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoostCardError::InvalidBoostValue(10)));
    }

    #[test]
    fn test_validate_boost_selection_unavailable_card() {
        let mut hand = create_test_boost_hand();
        
        // Use card 2
        hand.use_card(2).unwrap();
        
        // Card 2 should now be unavailable
        let result = BoostHandManager::validate_boost_selection(&hand, 2);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BoostCardError::CardNotAvailable { boost_value, available_cards } => {
                assert_eq!(boost_value, 2);
                assert_eq!(available_cards.len(), 4);
                assert!(!available_cards.contains(&2));
            }
            _ => panic!("Expected CardNotAvailable error"),
        }
    }

    #[test]
    fn test_use_boost_card_success() {
        let mut hand = create_test_boost_hand();
        
        let result = BoostHandManager::use_boost_card(&mut hand, 3);
        assert!(result.is_ok());
        
        let usage_result = result.unwrap();
        assert_eq!(usage_result.boost_value, 3);
        assert_eq!(usage_result.cards_remaining, 4);
        assert_eq!(usage_result.current_cycle, 1);
        assert!(!usage_result.replenishment_occurred);
        
        // Verify card is now unavailable
        assert!(!hand.is_card_available(3));
    }

    #[test]
    fn test_use_boost_card_triggers_replenishment() {
        let mut hand = create_test_boost_hand();
        
        // Use 4 cards
        BoostHandManager::use_boost_card(&mut hand, 0).unwrap();
        BoostHandManager::use_boost_card(&mut hand, 1).unwrap();
        BoostHandManager::use_boost_card(&mut hand, 2).unwrap();
        BoostHandManager::use_boost_card(&mut hand, 3).unwrap();
        
        // Use last card - should trigger replenishment
        let result = BoostHandManager::use_boost_card(&mut hand, 4);
        assert!(result.is_ok());
        
        let usage_result = result.unwrap();
        assert_eq!(usage_result.cards_remaining, 5);
        assert_eq!(usage_result.current_cycle, 2);
        assert!(usage_result.replenishment_occurred);
        
        // All cards should be available again
        for i in 0..=4 {
            assert!(hand.is_card_available(i));
        }
    }

    #[test]
    fn test_use_boost_card_invalid() {
        let mut hand = create_test_boost_hand();
        
        // Try to use invalid card
        let result = BoostHandManager::use_boost_card(&mut hand, 5);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BoostCardError::InvalidBoostValue(5)));
    }

    #[test]
    fn test_use_boost_card_unavailable() {
        let mut hand = create_test_boost_hand();
        
        // Use card 2
        BoostHandManager::use_boost_card(&mut hand, 2).unwrap();
        
        // Try to use card 2 again
        let result = BoostHandManager::use_boost_card(&mut hand, 2);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            BoostCardError::CardNotAvailable { boost_value, .. } => {
                assert_eq!(boost_value, 2);
            }
            _ => panic!("Expected CardNotAvailable error"),
        }
    }

    #[test]
    fn test_get_boost_availability() {
        let hand = create_test_boost_hand();
        let sector = create_test_sector();
        let base_performance = 15;
        
        let availability = BoostHandManager::get_boost_availability(
            &hand,
            &sector,
            base_performance,
        );
        
        // Verify basic fields
        assert_eq!(availability.available_cards.len(), 5);
        assert_eq!(availability.current_cycle, 1);
        assert_eq!(availability.cycles_completed, 0);
        assert_eq!(availability.cards_remaining, 5);
        assert_eq!(availability.next_replenishment_at, Some(5));
        
        // Verify boost impact preview
        assert_eq!(availability.boost_impact_preview.len(), 5);
        
        // All cards should be available
        for option in &availability.boost_impact_preview {
            assert!(option.is_available);
        }
        
        // Verify predicted values increase with boost
        let values: Vec<u32> = availability.boost_impact_preview
            .iter()
            .map(|o| o.predicted_final_value)
            .collect();
        
        for i in 1..values.len() {
            assert!(values[i] >= values[i - 1], "Values should increase with boost");
        }
    }

    #[test]
    fn test_get_boost_availability_with_used_cards() {
        let mut hand = create_test_boost_hand();
        let sector = create_test_sector();
        let base_performance = 15;
        
        // Use some cards
        hand.use_card(1).unwrap();
        hand.use_card(3).unwrap();
        
        let availability = BoostHandManager::get_boost_availability(
            &hand,
            &sector,
            base_performance,
        );
        
        // Verify available cards
        assert_eq!(availability.available_cards.len(), 3);
        assert!(availability.available_cards.contains(&0));
        assert!(availability.available_cards.contains(&2));
        assert!(availability.available_cards.contains(&4));
        
        // Verify cards_remaining
        assert_eq!(availability.cards_remaining, 3);
        assert_eq!(availability.next_replenishment_at, Some(3));
        
        // Verify boost impact preview shows correct availability
        for option in &availability.boost_impact_preview {
            if option.boost_value == 1 || option.boost_value == 3 {
                assert!(!option.is_available, "Used cards should not be available");
            } else {
                assert!(option.is_available, "Unused cards should be available");
            }
        }
    }

    #[test]
    fn test_calculate_movement_probability() {
        let sector = create_test_sector(); // min: 10, max: 20
        
        // Test move down (below min)
        let prob = BoostHandManager::calculate_movement_probability(5, &sector);
        assert!(matches!(prob, MovementProbability::MoveDown));
        
        // Test stay (within range)
        let prob = BoostHandManager::calculate_movement_probability(15, &sector);
        assert!(matches!(prob, MovementProbability::Stay));
        
        // Test move up (above max)
        let prob = BoostHandManager::calculate_movement_probability(25, &sector);
        assert!(matches!(prob, MovementProbability::MoveUp));
    }

    #[test]
    fn test_boost_card_error_response_from_error() {
        let hand = create_test_boost_hand();
        
        // Test InvalidBoostValue error
        let error = BoostCardError::InvalidBoostValue(5);
        let response = BoostCardErrorResponse::from_error(&error, &hand);
        
        assert_eq!(response.error_code, "INVALID_BOOST_VALUE");
        assert!(response.message.contains("Invalid boost value"));
        assert_eq!(response.available_cards.len(), 5);
        assert_eq!(response.current_cycle, 1);
        assert_eq!(response.cards_remaining, 5);
        
        // Test CardNotAvailable error
        let mut hand = create_test_boost_hand();
        hand.use_card(2).unwrap();
        
        let error = BoostCardError::CardNotAvailable {
            boost_value: 2,
            available_cards: hand.get_available_cards(),
        };
        let response = BoostCardErrorResponse::from_error(&error, &hand);
        
        assert_eq!(response.error_code, "BOOST_CARD_NOT_AVAILABLE");
        assert!(response.message.contains("not available"));
        assert_eq!(response.available_cards.len(), 4);
        assert!(!response.available_cards.contains(&2));
    }

    #[test]
    fn test_boost_impact_calculation() {
        let hand = create_test_boost_hand();
        let sector = create_test_sector(); // min: 10, max: 20
        let base_performance = 15;
        
        let availability = BoostHandManager::get_boost_availability(
            &hand,
            &sector,
            base_performance,
        );
        
        // Verify boost calculations
        // Base is 15, capped to sector max (20)
        // Boost 0: 15 * 1.0 = 15 (Stay)
        // Boost 1: 15 * 1.08 = 16.2 ≈ 16 (Stay)
        // Boost 2: 15 * 1.16 = 17.4 ≈ 17 (Stay)
        // Boost 3: 15 * 1.24 = 18.6 ≈ 19 (Stay)
        // Boost 4: 15 * 1.32 = 19.8 ≈ 20 (Stay)
        
        for option in &availability.boost_impact_preview {
            let expected_multiplier = 1.0 + (f64::from(option.boost_value) * 0.08);
            let expected_value = (15.0 * expected_multiplier).round() as u32;
            
            assert_eq!(
                option.predicted_final_value,
                expected_value,
                "Boost {} should produce value {}",
                option.boost_value,
                expected_value
            );
            
            // All should stay in sector (values 15-20 are within 10-20 range)
            assert!(matches!(option.movement_probability, MovementProbability::Stay));
        }
    }
}
