# Boost Card System Design Document

## Overview

The Boost Card System introduces a hand-based resource management mechanic to the Web3 Racing Game. Players receive 5 specific boost cards (values 0, 1, 2, 3, 4) at race start. Each card can be used once per cycle, and when all cards are exhausted, the entire hand replenishes automatically. This design integrates seamlessly with the existing race domain model and API structure while adding strategic depth to boost selection.

## Architecture

### System Components
```
┌─────────────────────────────────────────────────────────────┐
│                    Race Participant                          │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Boost Hand State                          │ │
│  │  Cards: [0: Available, 1: Used, 2: Available,         │ │
│  │          3: Used, 4: Available]                        │ │
│  │  Cycle: 2                                              │ │
│  │  Cards Remaining: 3                                    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    Lap Action Submitted
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Boost Card Validation                           │
│  1. Check if selected card is available                     │
│  2. Mark card as used                                        │
│  3. Check if all cards used → Trigger replenishment         │
│  4. Update cycle counter if replenished                      │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    Performance Calculation
                            ↓
                    Race State Update
```

### Data Flow
```
Player Selects Boost Card (e.g., 3)
    ↓
Validate Card Available in Hand
    ↓
Mark Card as Used
    ↓
Calculate Performance (existing formula with boost=3)
    ↓
Check if All Cards Used (0,1,2,3,4 all marked used)
    ↓
If Yes: Replenish All Cards + Increment Cycle
    ↓
Return Updated Race Status with Boost Hand State
```

## Data Models

### 1. Boost Hand State

**New Structure in RaceParticipant**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostHand {
    /// Availability state for each boost card (0-4)
    /// true = available, false = used
    pub cards: HashMap<u32, bool>,
    
    /// Current cycle number (starts at 1)
    pub current_cycle: u32,
    
    /// Total number of cycles completed
    pub cycles_completed: u32,
    
    /// Number of cards remaining in current cycle
    pub cards_remaining: u32,
}

impl BoostHand {
    /// Initialize a new boost hand with all cards available
    pub fn new() -> Self {
        let mut cards = HashMap::new();
        for i in 0..=4 {
            cards.insert(i, true);
        }
        
        Self {
            cards,
            current_cycle: 1,
            cycles_completed: 0,
            cards_remaining: 5,
        }
    }
    
    /// Check if a specific boost card is available
    pub fn is_card_available(&self, boost_value: u32) -> bool {
        self.cards.get(&boost_value).copied().unwrap_or(false)
    }
    
    /// Use a boost card (mark as unavailable)
    pub fn use_card(&mut self, boost_value: u32) -> Result<(), String> {
        if !self.is_card_available(boost_value) {
            return Err(format!("Boost card {} is not available", boost_value));
        }
        
        self.cards.insert(boost_value, false);
        self.cards_remaining -= 1;
        
        // Check if all cards are used
        if self.cards_remaining == 0 {
            self.replenish();
        }
        
        Ok(())
    }
    
    /// Replenish all boost cards
    fn replenish(&mut self) {
        for i in 0..=4 {
            self.cards.insert(i, true);
        }
        self.cards_remaining = 5;
        self.cycles_completed += 1;
        self.current_cycle += 1;
    }
    
    /// Get list of available boost card values
    pub fn get_available_cards(&self) -> Vec<u32> {
        self.cards
            .iter()
            .filter(|(_, &available)| available)
            .map(|(&value, _)| value)
            .collect()
    }
}
```

### 2. Extended RaceParticipant

**Modified Structure**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RaceParticipant {
    #[serde(with = "uuid_as_string")]
    pub player_uuid: Uuid,
    #[serde(with = "uuid_as_string")]
    pub car_uuid: Uuid,
    #[serde(with = "uuid_as_string")]
    pub pilot_uuid: Uuid,
    pub current_sector: u32,
    pub current_position_in_sector: u32,
    pub current_lap: u32,
    pub total_value: u32,
    pub is_finished: bool,
    pub finish_position: Option<u32>,
    
    // NEW: Boost hand state
    pub boost_hand: BoostHand,
}
```

### 3. Boost Usage History

**New Structure for Tracking**:
```rust
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostUsageRecord {
    pub lap_number: u32,
    pub boost_value: u32,
    pub cycle_number: u32,
    pub cards_remaining_after: u32,
    pub replenishment_occurred: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BoostCycleSummary {
    pub cycle_number: u32,
    pub cards_used: Vec<u32>,
    pub laps_in_cycle: Vec<u32>,
    pub average_boost: f32,
}
```

### 4. API Response Models

**Enhanced Boost Availability Response**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct BoostAvailability {
    /// Available boost card values
    pub available_cards: Vec<u32>,
    
    /// Full hand state (for detailed view)
    pub hand_state: HashMap<u32, bool>,
    
    /// Current cycle information
    pub current_cycle: u32,
    pub cycles_completed: u32,
    pub cards_remaining: u32,
    
    /// Replenishment indicator
    pub next_replenishment_at: Option<u32>, // Cards remaining until replenish
    
    /// Performance preview for available cards only
    pub boost_impact_preview: Vec<BoostImpactOption>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostImpactOption {
    pub boost_value: u32,
    pub is_available: bool,
    pub predicted_final_value: u32,
    pub movement_probability: MovementProbability,
}
```

## Components and Interfaces

### 1. Boost Hand Manager

**Core Logic Component**:
```rust
pub struct BoostHandManager;

impl BoostHandManager {
    /// Validate boost card selection
    pub fn validate_boost_selection(
        boost_hand: &BoostHand,
        boost_value: u32,
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
    pub fn use_boost_card(
        boost_hand: &mut BoostHand,
        boost_value: u32,
    ) -> Result<BoostUsageResult, BoostCardError> {
        // Validate first
        Self::validate_boost_selection(boost_hand, boost_value)?;
        
        // Use the card
        boost_hand.use_card(boost_value)?;
        
        // Check if replenishment occurred
        let replenishment_occurred = boost_hand.cards_remaining == 5;
        
        Ok(BoostUsageResult {
            boost_value,
            cards_remaining: boost_hand.cards_remaining,
            current_cycle: boost_hand.current_cycle,
            replenishment_occurred,
        })
    }
    
    /// Get boost availability for API response
    pub fn get_boost_availability(
        boost_hand: &BoostHand,
        current_sector: &Sector,
        base_performance: u32,
    ) -> BoostAvailability {
        let available_cards = boost_hand.get_available_cards();
        
        // Generate impact preview only for available cards
        let boost_impact_preview = (0..=4)
            .map(|boost| {
                let is_available = boost_hand.is_card_available(boost);
                let capped_base = std::cmp::min(base_performance, current_sector.max_value);
                let boost_multiplier = 1.0 + (boost as f64 * 0.08);
                let predicted_final = (capped_base as f64 * boost_multiplier).round() as u32;
                
                BoostImpactOption {
                    boost_value: boost,
                    is_available,
                    predicted_final_value: predicted_final,
                    movement_probability: calculate_movement_probability(
                        predicted_final,
                        current_sector,
                    ),
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
}

#[derive(Debug)]
pub struct BoostUsageResult {
    pub boost_value: u32,
    pub cards_remaining: u32,
    pub current_cycle: u32,
    pub replenishment_occurred: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum BoostCardError {
    #[error("Invalid boost value: {0}. Must be between 0 and 4")]
    InvalidBoostValue(u32),
    
    #[error("Boost card {boost_value} is not available. Available cards: {available_cards:?}")]
    CardNotAvailable {
        boost_value: u32,
        available_cards: Vec<u32>,
    },
}
```

### 2. Race Domain Integration

**Modified Race Methods**:
```rust
impl Race {
    /// Process individual lap action with boost card validation
    pub fn process_individual_lap_action(
        &mut self,
        player_uuid: Uuid,
        boost_value: u32,
        car_data: &ValidatedCarData,
    ) -> Result<LapActionResult, String> {
        // Find participant
        let participant = self.participants
            .iter_mut()
            .find(|p| p.player_uuid == player_uuid)
            .ok_or("Player not found in race")?;
        
        // Validate and use boost card
        BoostHandManager::validate_boost_selection(&participant.boost_hand, boost_value)
            .map_err(|e| e.to_string())?;
        
        let boost_result = BoostHandManager::use_boost_card(
            &mut participant.boost_hand,
            boost_value,
        ).map_err(|e| e.to_string())?;
        
        // Calculate performance (existing logic)
        let performance = self.calculate_performance_with_car_data(
            participant,
            boost_value,
            car_data,
            &self.lap_characteristic,
        )?;
        
        // Store pending action
        self.pending_actions.push(LapAction {
            player_uuid,
            boost_value,
        });
        
        self.pending_performance_calculations.insert(player_uuid, performance.clone());
        
        Ok(LapActionResult {
            performance,
            boost_usage: boost_result,
        })
    }
}

#[derive(Debug)]
pub struct LapActionResult {
    pub performance: PerformanceCalculation,
    pub boost_usage: BoostUsageResult,
}
```

### 3. API Endpoint Updates

**Modified Apply Lap Endpoint**:
```rust
pub async fn apply_lap_action(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ApplyLapRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode> {
    // Parse UUIDs
    let race_uuid = Uuid::parse_str(&race_uuid_str)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let player_uuid = Uuid::parse_str(&payload.player_uuid)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let car_uuid = Uuid::parse_str(&payload.car_uuid)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Validate car
    let car_data = CarValidationService::validate_car_for_race(
        &database,
        player_uuid,
        car_uuid,
    )
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Get race
    let collection = database.collection::<Race>("races");
    let filter = doc! { "uuid": race_uuid.to_string() };
    let mut race = collection
        .find_one(filter.clone(), None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Process lap action with boost card validation
    let lap_result = race
        .process_individual_lap_action(
            player_uuid,
            payload.boost_value,
            &car_data,
        )
        .map_err(|e| {
            // Return specific error for boost card issues
            eprintln!("Boost card error: {}", e);
            StatusCode::BAD_REQUEST
        })?;
    
    // Update race in database
    let update = doc! {
        "$set": {
            "participants": mongodb::bson::to_bson(&race.participants).unwrap(),
            "pending_actions": mongodb::bson::to_bson(&race.pending_actions).unwrap(),
            "pending_performance_calculations": mongodb::bson::to_bson(&race.pending_performance_calculations).unwrap(),
            "updated_at": BsonDateTime::from_chrono(Utc::now()),
        }
    };
    
    collection
        .update_one(filter, update, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Build response with boost hand state
    let response = build_detailed_race_status(
        &race,
        Some(player_uuid),
        &database,
    )
    .await?;
    
    Ok(Json(response))
}
```

## Error Handling

### Boost Card Specific Errors

```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct BoostCardErrorResponse {
    pub error_code: String,
    pub message: String,
    pub available_cards: Vec<u32>,
    pub current_cycle: u32,
    pub cards_remaining: u32,
}

// Error codes:
// - BOOST_CARD_NOT_AVAILABLE: Selected card already used
// - INVALID_BOOST_VALUE: Boost value not in 0-4 range
// - BOOST_HAND_STATE_ERROR: Internal state inconsistency
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boost_hand_initialization() {
        let hand = BoostHand::new();
        assert_eq!(hand.cards_remaining, 5);
        assert_eq!(hand.current_cycle, 1);
        assert_eq!(hand.cycles_completed, 0);
        assert!(hand.is_card_available(0));
        assert!(hand.is_card_available(4));
    }
    
    #[test]
    fn test_use_card_and_replenishment() {
        let mut hand = BoostHand::new();
        
        // Use all cards
        hand.use_card(2).unwrap();
        assert_eq!(hand.cards_remaining, 4);
        
        hand.use_card(0).unwrap();
        hand.use_card(4).unwrap();
        hand.use_card(1).unwrap();
        assert_eq!(hand.cards_remaining, 1);
        
        // Use last card - should trigger replenishment
        hand.use_card(3).unwrap();
        assert_eq!(hand.cards_remaining, 5);
        assert_eq!(hand.current_cycle, 2);
        assert_eq!(hand.cycles_completed, 1);
        
        // All cards should be available again
        assert!(hand.is_card_available(0));
        assert!(hand.is_card_available(1));
        assert!(hand.is_card_available(2));
        assert!(hand.is_card_available(3));
        assert!(hand.is_card_available(4));
    }
    
    #[test]
    fn test_cannot_use_same_card_twice() {
        let mut hand = BoostHand::new();
        
        hand.use_card(3).unwrap();
        let result = hand.use_card(3);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Boost card 3 is not available"
        );
    }
    
    #[test]
    fn test_get_available_cards() {
        let mut hand = BoostHand::new();
        
        hand.use_card(1).unwrap();
        hand.use_card(3).unwrap();
        
        let available = hand.get_available_cards();
        assert_eq!(available.len(), 3);
        assert!(available.contains(&0));
        assert!(available.contains(&2));
        assert!(available.contains(&4));
        assert!(!available.contains(&1));
        assert!(!available.contains(&3));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_lap_action_with_boost_cards() {
    // Test full lap processing with boost card validation
    // Verify boost hand state updates correctly
    // Test replenishment trigger
}

#[tokio::test]
async fn test_boost_card_error_handling() {
    // Test using unavailable card returns proper error
    // Test error response includes available cards
}

#[tokio::test]
async fn test_multiple_cycles() {
    // Test multiple complete cycles
    // Verify cycle counter increments
    // Verify statistics tracking
}
```

## Migration Strategy

### Database Migration

```rust
// Migration script to add boost_hand to existing participants
pub async fn migrate_add_boost_hand(database: &Database) -> Result<(), mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    
    // Find all races
    let mut cursor = collection.find(None, None).await?;
    
    while let Some(race) = cursor.try_next().await? {
        let mut updated_race = race;
        
        // Add boost hand to each participant
        for participant in &mut updated_race.participants {
            participant.boost_hand = BoostHand::new();
        }
        
        // Update race
        let filter = doc! { "uuid": updated_race.uuid.to_string() };
        let update = doc! {
            "$set": {
                "participants": mongodb::bson::to_bson(&updated_race.participants)?,
            }
        };
        
        collection.update_one(filter, update, None).await?;
    }
    
    Ok(())
}
```

### Backward Compatibility

- Existing races without boost hand will initialize with default state on first access
- API responses include boost hand data only when present
- Frontend can gracefully handle missing boost hand data (fallback to unlimited boost display)

## Performance Considerations

### Optimization Strategies

1. **In-Memory State**: Boost hand state is small (5 booleans + counters), minimal memory overhead
2. **Atomic Updates**: Boost card usage and replenishment are atomic operations
3. **Efficient Queries**: No additional database queries needed, boost hand is part of participant document
4. **Caching**: Boost availability can be cached in race status responses

### Scalability

- Boost hand state adds ~100 bytes per participant
- No additional database collections needed
- No performance impact on existing race processing logic
- Replenishment logic is O(1) operation

## UI/UX Considerations

### Visual Representation

```
┌─────────────────────────────────────┐
│        Your Boost Cards             │
│                                     │
│  [0]  [1]  [2]  [3]  [4]           │
│   ✓    ✗    ✓    ✗    ✓            │
│                                     │
│  Cards Remaining: 3/5               │
│  Current Cycle: 2                   │
│  Next Replenish: Use 3 more cards   │
└─────────────────────────────────────┘
```

### User Feedback

- Visual indication of available vs. used cards
- Clear replenishment countdown
- Highlight when replenishment occurs
- Show cycle progression for strategic planning

---

This design provides a complete, production-ready implementation of the boost card system that integrates seamlessly with the existing race infrastructure while adding meaningful strategic depth to gameplay.
