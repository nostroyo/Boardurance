# Feature #6: Turn Phase Detection Fix

## Problem Description

When first loading a race, the UI shows "phase complete" instead of allowing boost selection, even though the race is in progress and no players have submitted actions yet.

## Root Cause Analysis

### Issue 1: Frontend Hardcoded Turn Phase
The frontend was **ignoring** the backend's turn phase logic and hardcoding the phase based only on race status:

```typescript
// WRONG: Hardcoded logic
let turnPhase: TurnPhase = 'WaitingForPlayers';
if (race.status === 'InProgress') {
  turnPhase = 'WaitingForPlayers'; // Always sets to waiting
} else if (race.status === 'Finished') {
  turnPhase = 'Complete';
}
```

### Issue 2: Backend Logic Flaw
The `all_actions_submitted()` method returned `true` when both active participants and submitted actions were empty sets, causing "AllSubmitted" phase for fresh races.

```rust
// WRONG: Empty sets comparison
active_participants == submitted_actions  // true when both empty
```

## Solution Implemented

### Frontend Fix
- **Use actual backend turn phase**: Call `getTurnPhase()` endpoint instead of guessing
- **Proper error handling**: Fallback to race status logic only if API call fails
- **Async initialization**: Properly await turn phase response

### Backend Fix  
- **Explicit empty check**: If no submitted actions exist, return `false` for `all_actions_submitted()`
- **Logical flow**: Only return `true` when all active participants have actually submitted

```rust
// FIXED: Explicit logic
if active_participants.is_empty() {
    return true;  // No players = all submitted
}

if submitted_actions.is_empty() {
    return false; // Players exist but none submitted
}

active_participants == submitted_actions
```

## Files Modified

### Backend
- `rust-backend/src/domain/race.rs` - Fixed `all_actions_submitted()` logic

### Frontend  
- `empty-project/src/contexts/PlayerGameContext.tsx` - Use backend turn phase API

## Expected Behavior After Fix

1. **Fresh race load**: Shows "WaitingForPlayers" phase with boost selector enabled
2. **Partial submissions**: Shows "X/Y players submitted" 
3. **All submitted**: Shows "AllSubmitted" → auto-processes → "WaitingForPlayers"
4. **Race finished**: Shows "Complete" phase

## Testing Checklist

- [ ] Load fresh race → Should show "WaitingForPlayers"
- [ ] Boost selector should be enabled and functional
- [ ] Submit boost → Should show updated player count
- [ ] All players submit → Should auto-process and reset
- [ ] Finished race → Should show "Complete"