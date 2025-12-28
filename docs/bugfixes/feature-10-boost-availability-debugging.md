# Feature #10: Boost Availability Debugging Fix

## Problem Description

User is blocked with the following status:
- Race UUID: `53d90c6a-6ba5-4e66-8335-1cb28f2681a7`
- Player UUID: `9c48b7ac` (truncated in frontend)
- Selected Boost: None
- Car Data: ✓ Loaded
- Local View: ✓ Loaded  
- Boost Availability: ✗ Missing

## Root Cause Analysis

### Issue 1: Race Not Started
The race status was "Waiting" instead of "InProgress", causing the boost availability endpoint to return:
```json
{"error":"RACE_NOT_IN_PROGRESS","message":"Race is not in progress"}
```

### Issue 2: Player UUID Mismatch
- **Frontend shows**: `9c48b7ac` (truncated)
- **Backend has**: `068564eb-8109-4862-9875-87089c48b7ac` (full UUID)
- **API calls fail** when using truncated UUID

### Issue 3: Auto-Start Not Applied
The race was created before Feature #9 (auto-start) was implemented, so it remained in "Waiting" status.

## Solutions Applied

### Immediate Fix
1. **Started the race manually**: `POST /races/{uuid}/start`
2. **Verified boost availability**: Now returns proper data
   ```json
   {
     "available_cards": [0,1,2,3,4],
     "hand_state": {"4":true,"2":true,"1":true,"0":true,"3":true},
     "current_cycle": 1,
     "cycles_completed": 0,
     "cards_remaining": 5,
     "next_replenishment_at": 5
   }
   ```

### Frontend UUID Issue
The frontend needs to use the **full player UUID** for API calls:
- ✅ **Correct**: `068564eb-8109-4862-9875-87089c48b7ac`
- ❌ **Wrong**: `9c48b7ac`

## Testing Results

### Before Fix
```bash
# Race status
curl /races/{uuid} → "status": "Waiting"

# Boost availability  
curl /races/{uuid}/players/{uuid}/boost-availability → "RACE_NOT_IN_PROGRESS"
```

### After Fix
```bash
# Race status
curl /races/{uuid} → "status": "InProgress"

# Boost availability
curl /races/{uuid}/players/{uuid}/boost-availability → Available cards [0,1,2,3,4]
```

## Recommendations

### For Current Issue
1. **Refresh the frontend** - the race is now started and should work
2. **Verify player UUID** - ensure frontend uses full UUID for API calls
3. **Check boost selector** - should now appear and be functional

### For Future Prevention
1. **Auto-start feature** - New races will start automatically (Feature #9)
2. **UUID validation** - Frontend should validate full UUIDs
3. **Better error handling** - Show specific error messages for debugging

## Expected Behavior After Fix

1. **Race loads** → Status "InProgress" ✅
2. **Boost availability loads** → Cards [0,1,2,3,4] available ✅
3. **Boost selector appears** → Player can select boost values ✅
4. **Turn processing works** → Submit boost → Race progresses ✅

## Files Affected

- **Manual fix applied**: Race started via API call
- **Future races**: Will auto-start due to Feature #9
- **Frontend**: May need UUID validation improvements

## Debugging Commands

```bash
# Check race status
curl -X GET "http://localhost:3000/api/v1/races/53d90c6a-6ba5-4e66-8335-1cb28f2681a7"

# Check boost availability (use full UUID)
curl -X GET "http://localhost:3000/api/v1/races/53d90c6a-6ba5-4e66-8335-1cb28f2681a7/players/068564eb-8109-4862-9875-87089c48b7ac/boost-availability"

# Check turn phase
curl -X GET "http://localhost:3000/api/v1/races/53d90c6a-6ba5-4e66-8335-1cb28f2681a7/turn-phase"
```