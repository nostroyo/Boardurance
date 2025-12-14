# Frontend-Backend Separation of Concerns

## Critical Architecture Issue Identified

### Problem: Frontend Calculating Game Logic

The recently implemented `performanceCalculation.ts` utility **violates the separation of concerns** by duplicating backend game logic in the frontend.

## Current State (INCORRECT)

### Frontend (`performanceCalculation.ts`)
```typescript
// ❌ WRONG: Frontend calculates game logic
export const calculatePerformance = (
  pilot: Pilot,
  engine: Engine,
  body: Body,
  sector: Sector,
  lapCharacteristic: 'Straight' | 'Curve',
  boost: number
): PerformanceBreakdown => {
  // Calculates engine + body + pilot values
  // Applies sector ceiling
  // Adds boost value
  // Returns final performance
}
```

### Backend (`race.rs`)
```rust
// ✅ CORRECT: Backend has authoritative game logic
fn calculate_performance_with_car_data(
    &self,
    participant: &RaceParticipant,
    boost_value: u32,
    car_data: &ValidatedCarData,
    lap_characteristic: &LapCharacteristic,
) -> PerformanceCalculation {
    // Authoritative calculation
    // Applies boost multiplier: 1.0 + (boost * 0.08)
}
```

## Why This Is Wrong

### 1. **Duplication of Game Logic**
- Performance calculation exists in TWO places
- Changes to game mechanics require updating BOTH frontend and backend
- Risk of frontend/backend calculations diverging

### 2. **Security Risk**
- Frontend calculations can be manipulated by users
- Users could modify JavaScript to show false performance predictions
- Backend must recalculate anyway, making frontend calculation pointless

### 3. **Inconsistency Risk**
- Frontend uses simple addition: `base + boost`
- Backend uses multiplier: `base * (1.0 + boost * 0.08)`
- **These produce different results!**

### 4. **Maintenance Burden**
- Game balance changes require updating multiple codebases
- Testing must verify both implementations match
- Increased complexity and bug surface area

## Correct Architecture

### Principle: **Backend is Source of Truth**

```
┌─────────────┐                    ┌─────────────┐
│   Frontend  │                    │   Backend   │
│             │                    │             │
│  1. User    │                    │  3. Backend │
│     selects │  2. API Request    │     calculates│
│     boost   │ ──────────────────>│     performance│
│             │                    │             │
│  5. Display │  4. Return data    │  4. Returns │
│     preview │ <──────────────────│     preview │
│             │                    │             │
└─────────────┘                    └─────────────┘
```

### Frontend Responsibilities (Display Only)
- ✅ Collect user input (boost selection)
- ✅ Request performance preview from backend API
- ✅ Display performance data returned by backend
- ✅ Show visual feedback and UI elements
- ❌ **NEVER** calculate game logic

### Backend Responsibilities (Game Logic)
- ✅ Store authoritative car/pilot/engine/body data
- ✅ Calculate performance using game rules
- ✅ Apply sector ceilings and boost multipliers
- ✅ Validate boost card availability
- ✅ Return performance previews to frontend

## Required Changes

### 1. Backend API Enhancement

The backend already has `PerformancePreview` structure but needs to implement it properly:

```rust
// In rust-backend/src/routes/races.rs
pub struct PerformancePreview {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
    // Add boost preview for each available card
    pub boost_previews: Vec<BoostPreview>,
}

pub struct BoostPreview {
    pub boost_value: u32,
    pub is_available: bool,
    pub final_value: u32,
    pub movement_probability: MovementProbability,
}
```

**New Endpoint Needed:**
```
GET /api/v1/races/{race_uuid}/performance-preview?player_uuid={uuid}&boost_value={0-4}
```

**Response:**
```json
{
  "engine_contribution": 25,
  "body_contribution": 20,
  "pilot_contribution": 15,
  "base_value": 60,
  "sector_ceiling": 50,
  "capped_base_value": 50,
  "boost_previews": [
    {
      "boost_value": 0,
      "is_available": false,
      "final_value": 50,
      "movement_probability": "Stay"
    },
    {
      "boost_value": 1,
      "is_available": true,
      "final_value": 54,
      "movement_probability": "MoveUp"
    }
  ]
}
```

### 2. Frontend Refactoring

**Remove:**
- `empty-project/src/utils/performanceCalculation.ts` (entire file)

**Update:**
- `PerformanceCalculator.tsx` - fetch data from API instead of calculating
- `PlayerCarCard.tsx` - display backend-provided data only

**New API Service:**
```typescript
// empty-project/src/services/performanceAPI.ts
export const getPerformancePreview = async (
  raceUuid: string,
  playerUuid: string,
  boostValue: number
): Promise<PerformancePreview> => {
  const response = await fetch(
    `/api/v1/races/${raceUuid}/performance-preview?player_uuid=${playerUuid}&boost_value=${boostValue}`
  );
  return response.json();
};
```

**Updated Component:**
```typescript
// PerformanceCalculator.tsx
export const PerformanceCalculator: React.FC<Props> = ({
  raceUuid,
  playerUuid,
  selectedBoost,
  onBoostChange
}) => {
  const [preview, setPreview] = useState<PerformancePreview | null>(null);
  
  useEffect(() => {
    // Fetch preview from backend when boost changes
    getPerformancePreview(raceUuid, playerUuid, selectedBoost)
      .then(setPreview);
  }, [raceUuid, playerUuid, selectedBoost]);
  
  // Display preview data (no calculations)
  return <div>{/* Display preview */}</div>;
};
```

## Benefits of Correct Architecture

### 1. **Single Source of Truth**
- Game logic exists ONLY in backend
- One place to update game mechanics
- Guaranteed consistency

### 2. **Security**
- Frontend cannot manipulate calculations
- Backend validates all game logic
- Prevents cheating

### 3. **Maintainability**
- Game balance changes only touch backend
- Frontend is simpler (display only)
- Easier to test and debug

### 4. **Flexibility**
- Backend can change calculation formulas
- Frontend automatically shows correct values
- No frontend deployment needed for game balance

## Implementation Priority

### Immediate (Critical)
1. ❌ **DO NOT USE** `performanceCalculation.ts` in production
2. ✅ Document this architectural issue
3. ✅ Plan backend API enhancement

### Short Term (Next Sprint)
1. Implement backend performance preview endpoint
2. Add boost preview for all available cards
3. Include movement probability predictions

### Medium Term (Following Sprint)
1. Refactor frontend to use backend API
2. Remove frontend calculation logic
3. Update tests to verify API integration

## Testing Strategy

### Backend Tests
- Unit tests for performance calculation logic
- Integration tests for preview API endpoint
- Verify boost multiplier calculations
- Test sector ceiling application

### Frontend Tests
- Mock API responses for preview data
- Test UI rendering with various preview states
- Verify boost selection triggers API calls
- Test error handling for API failures

## Conclusion

**The frontend should be a "dumb client"** that:
- Collects user input
- Requests data from backend
- Displays backend responses
- Provides visual feedback

**The backend should be the "smart server"** that:
- Stores authoritative game state
- Calculates all game logic
- Validates all actions
- Returns computed results

This separation ensures security, consistency, and maintainability.

## Status

- ⚠️ **Current Implementation**: Frontend calculates game logic (INCORRECT)
- ✅ **Documented Issue**: This document
- 🔄 **Next Steps**: Implement backend API and refactor frontend
