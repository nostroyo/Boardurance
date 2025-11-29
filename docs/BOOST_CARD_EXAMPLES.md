# Boost Card System - Usage Examples

This document provides comprehensive examples of the boost card system API usage, including common scenarios, error handling, and strategic patterns.

## Table of Contents
- [Basic Usage Flow](#basic-usage-flow)
- [Complete Cycle Example](#complete-cycle-example)
- [Error Handling Examples](#error-handling-examples)
- [Strategic Usage Patterns](#strategic-usage-patterns)
- [Multi-Player Scenarios](#multi-player-scenarios)
- [Performance Impact Examples](#performance-impact-examples)

## Basic Usage Flow

### 1. Initial Race Status Check
```bash
GET /api/v1/races/550e8400-e29b-41d4-a716-446655440000/status-detailed?player_uuid=550e8400-e29b-41d4-a716-446655440001
```

**Response:**
```json
{
  "race_progress": {
    "status": "Ongoing",
    "current_lap": 1,
    "total_laps": 5
  },
  "player_data": {
    "boost_availability": {
      "available_cards": [0, 1, 2, 3, 4],
      "hand_state": {
        "0": true,
        "1": true,
        "2": true,
        "3": true,
        "4": true
      },
      "current_cycle": 1,
      "cycles_completed": 0,
      "cards_remaining": 5,
      "next_replenishment_at": 5,
      "boost_impact_preview": [
        {
          "boost_value": 0,
          "is_available": true,
          "predicted_final_value": 20,
          "movement_probability": "Stay"
        },
        {
          "boost_value": 1,
          "is_available": true,
          "predicted_final_value": 22,
          "movement_probability": "Stay"
        },
        {
          "boost_value": 2,
          "is_available": true,
          "predicted_final_value": 23,
          "movement_probability": "MoveUp"
        },
        {
          "boost_value": 3,
          "is_available": true,
          "predicted_final_value": 25,
          "movement_probability": "MoveUp"
        },
        {
          "boost_value": 4,
          "is_available": true,
          "predicted_final_value": 26,
          "movement_probability": "MoveUp"
        }
      ]
    },
    "boost_usage_history": [],
    "boost_cycle_summaries": []
  }
}
```

### 2. Apply First Lap Action
```bash
POST /api/v1/races/550e8400-e29b-41d4-a716-446655440000/apply-lap
Content-Type: application/json

{
  "player_uuid": "550e8400-e29b-41d4-a716-446655440001",
  "car_uuid": "550e8400-e29b-41d4-a716-446655440002",
  "boost_value": 2
}
```

**Response:**
```json
{
  "race_progress": {
    "status": "Ongoing",
    "current_lap": 1,
    "total_laps": 5
  },
  "player_data": {
    "boost_availability": {
      "available_cards": [0, 1, 3, 4],
      "hand_state": {
        "0": true,
        "1": true,
        "2": false,
        "3": true,
        "4": true
      },
      "current_cycle": 1,
      "cycles_completed": 0,
      "cards_remaining": 4,
      "next_replenishment_at": 4
    },
    "boost_usage_history": [
      {
        "lap_number": 1,
        "boost_value": 2,
        "cycle_number": 1,
        "cards_remaining_after": 4,
        "replenishment_occurred": false
      }
    ]
  }
}
```

## Complete Cycle Example

This example shows a complete boost cycle from start to replenishment.

### Lap 1: Use Boost Card 3
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "player_uuid": "{player_uuid}",
  "car_uuid": "{car_uuid}",
  "boost_value": 3
}
```

**Result:** Cards remaining: 4, Available: [0, 1, 2, 4]

### Lap 2: Use Boost Card 0
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 0
}
```

**Result:** Cards remaining: 3, Available: [1, 2, 4]

### Lap 3: Use Boost Card 4
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 4
}
```

**Result:** Cards remaining: 2, Available: [1, 2]

### Lap 4: Use Boost Card 1
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 1
}
```

**Result:** Cards remaining: 1, Available: [2]

### Lap 5: Use Last Card (Triggers Replenishment)
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 2
}
```

**Response with Replenishment:**
```json
{
  "player_data": {
    "boost_availability": {
      "available_cards": [0, 1, 2, 3, 4],
      "hand_state": {
        "0": true,
        "1": true,
        "2": true,
        "3": true,
        "4": true
      },
      "current_cycle": 2,
      "cycles_completed": 1,
      "cards_remaining": 5,
      "next_replenishment_at": 5
    },
    "boost_usage_history": [
      {
        "lap_number": 1,
        "boost_value": 3,
        "cycle_number": 1,
        "cards_remaining_after": 4,
        "replenishment_occurred": false
      },
      {
        "lap_number": 2,
        "boost_value": 0,
        "cycle_number": 1,
        "cards_remaining_after": 3,
        "replenishment_occurred": false
      },
      {
        "lap_number": 3,
        "boost_value": 4,
        "cycle_number": 1,
        "cards_remaining_after": 2,
        "replenishment_occurred": false
      },
      {
        "lap_number": 4,
        "boost_value": 1,
        "cycle_number": 1,
        "cards_remaining_after": 1,
        "replenishment_occurred": false
      },
      {
        "lap_number": 5,
        "boost_value": 2,
        "cycle_number": 1,
        "cards_remaining_after": 5,
        "replenishment_occurred": true
      }
    ],
    "boost_cycle_summaries": [
      {
        "cycle_number": 1,
        "cards_used": [3, 0, 4, 1, 2],
        "laps_in_cycle": [1, 2, 3, 4, 5],
        "average_boost": 2.0
      }
    ]
  }
}
```

## Error Handling Examples

### 1. Boost Card Not Available
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 2  # Already used
}
```

**Response (400 Bad Request):**
```json
{
  "error_code": "BOOST_CARD_NOT_AVAILABLE",
  "message": "Boost card 2 is not available. Available cards: [0, 1, 3, 4]",
  "available_cards": [0, 1, 3, 4],
  "current_cycle": 1,
  "cards_remaining": 4
}
```

### 2. Invalid Boost Value
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 5  # Outside valid range
}
```

**Response (400 Bad Request):**
```json
{
  "error_code": "INVALID_BOOST_VALUE",
  "message": "Invalid boost value: 5. Must be between 0 and 4",
  "available_cards": [0, 1, 3, 4],
  "current_cycle": 1,
  "cards_remaining": 4
}
```

### 3. Car Validation Failed
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "player_uuid": "valid-player-uuid",
  "car_uuid": "invalid-car-uuid",  # Player doesn't own this car
  "boost_value": 2
}
```

**Response (400 Bad Request):**
```json
{
  "error_code": "CAR_VALIDATION_FAILED",
  "message": "Car validation failed: Player does not own this car",
  "available_cards": [0, 1, 2, 3, 4],
  "current_cycle": 1,
  "cards_remaining": 5
}
```

## Strategic Usage Patterns

### 1. Conservative Strategy (Save High Boosts)
```bash
# Early laps: Use low boost cards
Lap 1: boost_value: 0  # No boost
Lap 2: boost_value: 1  # Small boost
Lap 3: boost_value: 2  # Medium boost

# Critical moments: Use high boost cards
Lap 4: boost_value: 4  # Maximum boost for overtaking
Lap 5: boost_value: 3  # High boost for final push
```

### 2. Aggressive Strategy (Front-load High Boosts)
```bash
# Early advantage: Use high boosts immediately
Lap 1: boost_value: 4  # Maximum boost for early lead
Lap 2: boost_value: 3  # High boost to maintain position
Lap 3: boost_value: 2  # Medium boost

# Later laps: Use remaining cards
Lap 4: boost_value: 1  # Small boost
Lap 5: boost_value: 0  # No boost (triggers replenishment)
```

### 3. Balanced Strategy (Even Distribution)
```bash
# Distribute boosts evenly across cycle
Lap 1: boost_value: 2  # Medium boost
Lap 2: boost_value: 1  # Small boost
Lap 3: boost_value: 3  # High boost
Lap 4: boost_value: 0  # No boost
Lap 5: boost_value: 4  # Maximum boost
```

## Multi-Player Scenarios

### Independent Boost Hands
Each player has their own boost hand state:

**Player 1 Status:**
```json
{
  "boost_availability": {
    "available_cards": [0, 1, 4],
    "cards_remaining": 3,
    "current_cycle": 1
  }
}
```

**Player 2 Status:**
```json
{
  "boost_availability": {
    "available_cards": [1, 2, 3, 4],
    "cards_remaining": 4,
    "current_cycle": 1
  }
}
```

Both players can use the same boost value simultaneously:
```bash
# Both players use boost card 1 in the same lap
Player 1: POST /apply-lap { "boost_value": 1 }  # ✅ Success
Player 2: POST /apply-lap { "boost_value": 1 }  # ✅ Success
```

### Concurrent Lap Submissions
```bash
# Simultaneous requests (handled atomically)
Player 1: POST /apply-lap { "boost_value": 2 }
Player 2: POST /apply-lap { "boost_value": 3 }

# Both succeed with independent boost hand updates
```

## Performance Impact Examples

### Base Performance: 20 points

| Boost Card | Multiplier | Final Performance | Increase |
|------------|------------|------------------|----------|
| 0          | 1.00x      | 20               | +0       |
| 1          | 1.08x      | 22               | +2       |
| 2          | 1.16x      | 23               | +3       |
| 3          | 1.24x      | 25               | +5       |
| 4          | 1.32x      | 26               | +6       |

### Sector Movement Examples

**Sector: Min 15, Max 25**

```json
{
  "boost_impact_preview": [
    {
      "boost_value": 0,
      "predicted_final_value": 20,
      "movement_probability": "Stay"  # 20 is within 15-25
    },
    {
      "boost_value": 3,
      "predicted_final_value": 25,
      "movement_probability": "Stay"  # 25 equals max
    },
    {
      "boost_value": 4,
      "predicted_final_value": 26,
      "movement_probability": "MoveUp"  # 26 exceeds max (25)
    }
  ]
}
```

**Low Performance Scenario (Base: 10)**

```json
{
  "boost_impact_preview": [
    {
      "boost_value": 0,
      "predicted_final_value": 10,
      "movement_probability": "MoveDown"  # 10 below min (15)
    },
    {
      "boost_value": 4,
      "predicted_final_value": 13,
      "movement_probability": "MoveDown"  # Still below min
    }
  ]
}
```

## Advanced Usage Scenarios

### 1. Mid-Race Strategy Adjustment
```bash
# Check current status to plan remaining boosts
GET /status-detailed?player_uuid={player_uuid}

# Response shows 2 cards remaining: [1, 4]
# Strategy: Save boost 4 for final lap, use 1 now
POST /apply-lap { "boost_value": 1 }
```

### 2. Replenishment Timing
```bash
# 1 card remaining before important final laps
# Option 1: Use last card now to trigger replenishment
POST /apply-lap { "boost_value": 3 }  # Triggers replenishment

# Option 2: Save last card for critical moment
# (Risk: No replenishment until much later)
```

### 3. Performance Threshold Management
```bash
# Check impact preview to ensure sector advancement
GET /status-detailed?player_uuid={player_uuid}

# Use minimum boost needed to move up
# If boost 2 gives "MoveUp", don't waste boost 4
POST /apply-lap { "boost_value": 2 }
```

## Integration Best Practices

### 1. Error Handling in Frontend
```javascript
async function applyLapAction(raceUuid, playerUuid, carUuid, boostValue) {
  try {
    const response = await fetch(`/api/v1/races/${raceUuid}/apply-lap`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        player_uuid: playerUuid,
        car_uuid: carUuid,
        boost_value: boostValue
      })
    });

    if (!response.ok) {
      const error = await response.json();
      
      switch (error.error_code) {
        case 'BOOST_CARD_NOT_AVAILABLE':
          showBoostCardError(error.available_cards);
          break;
        case 'INVALID_BOOST_VALUE':
          showInvalidBoostError();
          break;
        default:
          showGenericError(error.message);
      }
      return null;
    }

    return await response.json();
  } catch (error) {
    showNetworkError();
    return null;
  }
}
```

### 2. Real-time Boost Hand Updates
```javascript
// Update UI after successful lap action
function updateBoostHandUI(boostAvailability) {
  const { available_cards, cards_remaining, current_cycle } = boostAvailability;
  
  // Update available card buttons
  for (let i = 0; i <= 4; i++) {
    const button = document.getElementById(`boost-card-${i}`);
    button.disabled = !available_cards.includes(i);
    button.classList.toggle('available', available_cards.includes(i));
  }
  
  // Update cycle information
  document.getElementById('cards-remaining').textContent = cards_remaining;
  document.getElementById('current-cycle').textContent = current_cycle;
  
  // Show replenishment indicator
  if (cards_remaining <= 2) {
    document.getElementById('replenishment-warning').style.display = 'block';
  }
}
```

### 3. Strategic Decision Support
```javascript
// Help players make informed boost decisions
function showBoostImpactPreview(boostImpactPreview) {
  boostImpactPreview.forEach(option => {
    const element = document.getElementById(`boost-preview-${option.boost_value}`);
    
    if (option.is_available) {
      element.innerHTML = `
        <div class="boost-option available">
          <span class="boost-value">Boost ${option.boost_value}</span>
          <span class="performance">${option.predicted_final_value} pts</span>
          <span class="movement ${option.movement_probability.toLowerCase()}">
            ${option.movement_probability}
          </span>
        </div>
      `;
    } else {
      element.innerHTML = `
        <div class="boost-option unavailable">
          <span class="boost-value">Boost ${option.boost_value}</span>
          <span class="status">Used</span>
        </div>
      `;
    }
  });
}
```

This comprehensive example set demonstrates all aspects of the boost card system API, from basic usage to advanced strategic scenarios and integration patterns.