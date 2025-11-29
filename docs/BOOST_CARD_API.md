# Boost Card System API Documentation

## Overview

The Boost Card System introduces strategic resource management to the Web3 Racing Game. Players receive 5 specific boost cards (values 0-4) that can be used once per cycle. When all cards are exhausted, the entire hand replenishes automatically.

## Core Concepts

### Boost Hand Structure
Each player has a boost hand containing:
- **5 boost cards** with values 0, 1, 2, 3, 4
- **Availability state** tracking which cards are used/available
- **Cycle information** tracking current cycle and completion count
- **Usage history** recording all boost card usage

### Boost Cycles
- A **cycle** starts with all 5 cards available
- Cards become unavailable after use
- When all 5 cards are used, **replenishment** occurs
- All cards become available again and cycle counter increments

### Performance Calculation
Boost cards multiply base performance:
```
final_performance = base_performance * (1 + boost_value * 0.08)
```

| Boost Value | Multiplier | Performance Increase |
|-------------|------------|---------------------|
| 0           | 1.00x      | No boost            |
| 1           | 1.08x      | +8%                 |
| 2           | 1.16x      | +16%                |
| 3           | 1.24x      | +24%                |
| 4           | 1.32x      | +32%                |

## API Endpoints

### Apply Lap Action
**POST** `/api/v1/races/{race_uuid}/apply-lap`

Processes a player's lap action with boost card validation.

#### Request Body
```json
{
  "player_uuid": "550e8400-e29b-41d4-a716-446655440000",
  "car_uuid": "550e8400-e29b-41d4-a716-446655440001",
  "boost_value": 3
}
```

#### Success Response (200)
```json
{
  "race_progress": {
    "status": "Ongoing",
    "current_lap": 2,
    "total_laps": 5,
    "participants_count": 4,
    "finished_participants": 0
  },
  "player_data": {
    "boost_availability": {
      "available_cards": [0, 1, 2, 4],
      "hand_state": {
        "0": true,
        "1": true,
        "2": true,
        "3": false,
        "4": true
      },
      "current_cycle": 1,
      "cycles_completed": 0,
      "cards_remaining": 4,
      "next_replenishment_at": 4,
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
          "movement_probability": "MoveUp"
        }
      ]
    }
  }
}
```

#### Error Responses

##### Boost Card Not Available (400)
```json
{
  "error_code": "BOOST_CARD_NOT_AVAILABLE",
  "message": "Boost card 3 is not available. Available cards: [0, 1, 2, 4]",
  "available_cards": [0, 1, 2, 4],
  "current_cycle": 1,
  "cards_remaining": 4
}
```

##### Invalid Boost Value (400)
```json
{
  "error_code": "INVALID_BOOST_VALUE",
  "message": "Invalid boost value: 5. Must be between 0 and 4",
  "available_cards": [0, 1, 2, 3, 4],
  "current_cycle": 1,
  "cards_remaining": 5
}
```

### Get Detailed Race Status
**GET** `/api/v1/races/{race_uuid}/status-detailed?player_uuid={player_uuid}`

Returns comprehensive race status including boost hand information.

#### Query Parameters
- `player_uuid` (optional): Include player-specific boost hand data
- `include_history` (optional): Include detailed usage history

#### Response with Boost Hand Data
```json
{
  "race_progress": {
    "status": "Ongoing",
    "current_lap": 3,
    "total_laps": 5,
    "participants_count": 2,
    "finished_participants": 0
  },
  "player_data": {
    "boost_availability": {
      "available_cards": [1, 4],
      "hand_state": {
        "0": false,
        "1": true,
        "2": false,
        "3": false,
        "4": true
      },
      "current_cycle": 1,
      "cycles_completed": 0,
      "cards_remaining": 2,
      "next_replenishment_at": 2,
      "boost_impact_preview": [
        {
          "boost_value": 0,
          "is_available": false,
          "predicted_final_value": 20,
          "movement_probability": "Stay"
        },
        {
          "boost_value": 1,
          "is_available": true,
          "predicted_final_value": 22,
          "movement_probability": "MoveUp"
        }
      ]
    },
    "boost_usage_history": [
      {
        "lap_number": 1,
        "boost_value": 2,
        "cycle_number": 1,
        "cards_remaining_after": 3,
        "replenishment_occurred": false
      },
      {
        "lap_number": 2,
        "boost_value": 0,
        "cycle_number": 1,
        "cards_remaining_after": 2,
        "replenishment_occurred": false
      }
    ],
    "boost_cycle_summaries": [
      {
        "cycle_number": 1,
        "cards_used": [2, 0, 3],
        "laps_in_cycle": [1, 2, 3],
        "average_boost": 1.67
      }
    ]
  }
}
```

## Data Models

### BoostAvailability
Contains complete boost hand state information.

```json
{
  "available_cards": [0, 1, 4],
  "hand_state": {
    "0": true,
    "1": true,
    "2": false,
    "3": false,
    "4": true
  },
  "current_cycle": 1,
  "cycles_completed": 0,
  "cards_remaining": 3,
  "next_replenishment_at": 3,
  "boost_impact_preview": [...]
}
```

**Fields:**
- `available_cards`: Array of boost values currently available
- `hand_state`: Boolean map of each card's availability (0-4)
- `current_cycle`: Current cycle number (starts at 1)
- `cycles_completed`: Number of complete cycles finished
- `cards_remaining`: Cards left before next replenishment
- `next_replenishment_at`: Cards remaining until replenishment (null if hand empty)
- `boost_impact_preview`: Performance prediction for each boost option

### BoostImpactOption
Performance preview for a specific boost card.

```json
{
  "boost_value": 2,
  "is_available": true,
  "predicted_final_value": 25,
  "movement_probability": "MoveUp"
}
```

**Fields:**
- `boost_value`: The boost card value (0-4)
- `is_available`: Whether this card can currently be used
- `predicted_final_value`: Expected performance after applying boost
- `movement_probability`: Likely sector movement ("Stay", "MoveUp", "MoveDown")

### BoostUsageRecord
Historical record of boost card usage.

```json
{
  "lap_number": 3,
  "boost_value": 2,
  "cycle_number": 1,
  "cards_remaining_after": 2,
  "replenishment_occurred": false
}
```

**Fields:**
- `lap_number`: Lap when boost was used
- `boost_value`: Which boost card was used (0-4)
- `cycle_number`: Which cycle the usage occurred in
- `cards_remaining_after`: Cards left after this usage
- `replenishment_occurred`: Whether replenishment happened after this usage

### BoostCycleSummary
Aggregated statistics for a complete boost cycle.

```json
{
  "cycle_number": 1,
  "cards_used": [2, 0, 4, 1, 3],
  "laps_in_cycle": [1, 2, 3, 4, 5],
  "average_boost": 2.0
}
```

**Fields:**
- `cycle_number`: The cycle number
- `cards_used`: Boost cards used in order
- `laps_in_cycle`: Lap numbers when cards were used
- `average_boost`: Average boost value for the cycle

### BoostCardErrorResponse
Error response for boost card validation failures.

```json
{
  "error_code": "BOOST_CARD_NOT_AVAILABLE",
  "message": "Boost card 3 is not available. Available cards: [0, 1, 2, 4]",
  "available_cards": [0, 1, 2, 4],
  "current_cycle": 1,
  "cards_remaining": 4
}
```

**Error Codes:**
- `BOOST_CARD_NOT_AVAILABLE`: Selected card already used in current cycle
- `INVALID_BOOST_VALUE`: Boost value outside 0-4 range
- `CAR_VALIDATION_FAILED`: Invalid car/player combination
- `INVALID_UUID`: Malformed UUID in request

## Usage Examples

### Complete Boost Cycle Flow

1. **Initial State** - All cards available
```json
{
  "available_cards": [0, 1, 2, 3, 4],
  "cards_remaining": 5,
  "current_cycle": 1,
  "cycles_completed": 0
}
```

2. **Use Boost Card 2**
```bash
POST /api/v1/races/{race_uuid}/apply-lap
{
  "player_uuid": "...",
  "car_uuid": "...",
  "boost_value": 2
}
```

3. **After Usage** - Card 2 unavailable
```json
{
  "available_cards": [0, 1, 3, 4],
  "cards_remaining": 4,
  "current_cycle": 1,
  "cycles_completed": 0
}
```

4. **Use Remaining Cards** - Continue until all used

5. **After Last Card** - Replenishment occurs
```json
{
  "available_cards": [0, 1, 2, 3, 4],
  "cards_remaining": 5,
  "current_cycle": 2,
  "cycles_completed": 1
}
```

### Error Handling Example

```bash
# Try to use unavailable card
POST /api/v1/races/{race_uuid}/apply-lap
{
  "boost_value": 2  # Already used
}

# Response: 400 Bad Request
{
  "error_code": "BOOST_CARD_NOT_AVAILABLE",
  "message": "Boost card 2 is not available. Available cards: [0, 1, 3, 4]",
  "available_cards": [0, 1, 3, 4],
  "current_cycle": 1,
  "cards_remaining": 4
}
```

## Integration Guidelines

### Frontend Implementation
1. **Display Available Cards**: Show which boost cards can be selected
2. **Visual Feedback**: Indicate used vs available cards clearly
3. **Cycle Progress**: Show cards remaining until replenishment
4. **Performance Preview**: Display expected performance for each boost option
5. **Error Handling**: Handle boost card errors gracefully with user feedback

### Strategic Considerations
- **Early Game**: Save high boost cards for critical moments
- **Late Game**: Use remaining cards efficiently before race end
- **Cycle Planning**: Consider replenishment timing for optimal strategy
- **Risk Management**: Balance high boost usage with availability needs

## Migration and Compatibility

### Existing Races
- Races created before boost card system have default boost hands initialized
- API responses include boost hand data when available
- Backward compatibility maintained for races without boost hands

### Database Schema
- `boost_hand` field added to `RaceParticipant` documents
- `boost_usage_history` tracks all boost card usage
- Migration scripts available for existing data