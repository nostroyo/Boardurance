# Boost Calculation Simplification

## Overview
Simplified the boost calculation from a multiplicative system to an additive system for better game balance and player understanding.

## Change Summary

### Previous Formula (Multiplicative)
```
boost_multiplier = 1.0 + (boost_value × 0.08)
final_value = round(min(engine + body + pilot, sector_max) × boost_multiplier)
```

**Boost Effects:**
- Boost 0: 1.0x (no change)
- Boost 1: 1.08x (+8%)
- Boost 2: 1.16x (+16%)
- Boost 3: 1.24x (+24%)
- Boost 4: 1.32x (+32%)

### New Formula (Additive)
```
final_value = min(engine + body + pilot, sector_max) + boost_value
```

**Boost Effects:**
- Boost 0: +0
- Boost 1: +1
- Boost 2: +2
- Boost 3: +3
- Boost 4: +4

## Rationale

### Advantages of Additive System
1. **Simpler to understand**: Players can easily calculate the exact impact of each boost card
2. **More predictable**: No rounding or percentage calculations needed
3. **Fairer**: All players get the same absolute benefit from boost cards, regardless of base performance
4. **Better balance**: Prevents high-performance cars from getting disproportionate advantages from boosts

### Example Comparison

**Scenario**: Car with base value of 30, sector max of 30, using boost 3

**Old System (Multiplicative):**
```
capped_base = min(30, 30) = 30
final = round(30 × 1.24) = 37
Benefit: +7 points
```

**New System (Additive):**
```
capped_base = min(30, 30) = 30
final = 30 + 3 = 33
Benefit: +3 points
```

## Implementation Details

### Backend Changes
- **File**: `rust-backend/src/domain/race.rs`
- **Method**: `calculate_performance_with_car_data`
- **Lines**: Removed boost multiplier calculation, replaced with simple addition

### Frontend Changes
- **File**: `empty-project/src/utils/performanceCalculation.ts`
- **Status**: Already implemented correctly with additive logic
- **No changes needed**

### Test Updates
- Fixed test helpers in `car_validation.rs` and `jwt.rs` to create complete cars with 3 pilots, engine, and body
- All 101 unit tests passing

## Migration Notes

### Database
- No database migration needed
- Boost values remain 0-4 (boost cards)
- Historical race data remains valid

### API
- No API changes required
- Performance calculation happens server-side
- Response format unchanged

## Performance Impact

The additive system is actually more performant:
- No floating-point multiplication
- No rounding operation
- Simple integer addition

## Game Balance Impact

Players will notice:
- Boost cards provide consistent, predictable benefits
- Strategic boost usage becomes more about timing than maximizing percentage gains
- Lower-tier cars remain competitive when using boosts wisely

## Related Files

- `rust-backend/src/domain/race.rs` - Backend calculation
- `empty-project/src/utils/performanceCalculation.ts` - Frontend calculation
- `docs/GAME_MECHANICS.md` - Game mechanics documentation
- `docs/BOOST_CARD_API.md` - Boost card API documentation

## Testing

All tests passing:
```bash
cd rust-backend
cargo test --lib
# Result: ok. 101 passed; 0 failed
```

## Date
December 5, 2025

## Branch
`feature/simplify-boost-calculation`
