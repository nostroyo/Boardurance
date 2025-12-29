# Feature #15: Single Player Turn Processing Fix

## Problem Description

When a single player submits a boost in a race, the game gets stuck in "waiting for players" state even though the turn should be processed immediately since there's only one player.

### Root Cause Analysis

The backend logic was actually working correctly:
1. Single player submits boost action
2. Backend detects `players_submitted >= total_players` (1 >= 1)
3. Backend auto-processes the turn immediately
4. Backend returns `turn_phase: "WaitingForPlayers"` and `players_submitted: 0` for next turn

The issue was in the frontend logic in `PlayerGameContext.tsx`:
- The response handling wasn't properly distinguishing between "still waiting" vs "turn completed"
- The condition `response.turn_phase === 'WaitingForPlayers'` was ambiguous
- Race conditions in state updates and data refreshing

## Solution Implemented

### Enhanced Response Handling

Updated `submitBoostAction` in `PlayerGameContext.tsx` to properly handle different scenarios:

```typescript
// Check if turn was immediately processed (single player or all players submitted)
if (response.turn_phase === 'WaitingForPlayers' && response.players_submitted === 0) {
  // Turn was auto-processed and completed - reset for next turn immediately
  console.log('Turn auto-processed, resetting for next turn');
  dispatch({ type: 'SET_HAS_SUBMITTED', payload: false });
  dispatch({ type: 'SET_SELECTED_BOOST', payload: null });
  dispatch({ type: 'SET_TURN_PHASE', payload: 'WaitingForPlayers' });
  
  // Refresh race data to get updated positions
  setTimeout(async () => {
    await updateRaceData();
  }, 500); // Small delay to ensure backend has updated
  
} else if (response.turn_phase === 'WaitingForPlayers' && response.players_submitted > 0) {
  // Still waiting for other players
  dispatch({ type: 'SET_HAS_SUBMITTED', payload: true });
  dispatch({ type: 'SET_TURN_PHASE', payload: 'WaitingForPlayers' });
}
```

### Improved Turn Completion Polling

Enhanced the polling logic with:
- Better logging for debugging
- Timeout protection (30 polls max = 60 seconds)
- More robust error handling
- Forced reset on timeout

```typescript
const startTurnCompletionPolling = useCallback(() => {
  console.log('Starting turn completion polling for race:', state.race.uuid);
  let pollCount = 0;
  const maxPolls = 30; // 60 seconds max (30 * 2s)

  const pollInterval = setInterval(async () => {
    pollCount++;
    
    try {
      const turnPhaseResponse = await raceAPIService.getTurnPhase(state.race!.uuid);
      console.log(`Poll ${pollCount}: Turn phase is ${turnPhaseResponse.turn_phase}`);
      
      if (turnPhaseResponse.turn_phase === 'WaitingForPlayers') {
        // Turn processing complete - reset for next turn
        console.log('Turn processing completed, resetting for next turn');
        clearInterval(pollInterval);
        
        // Reset submission state and refresh data
        // ... reset logic
      } else if (pollCount >= maxPolls) {
        // Timeout protection
        console.warn('Turn completion polling timed out, forcing reset');
        // ... force reset
      }
    } catch (error) {
      console.error('Turn completion polling error:', error);
      // ... error handling
    }
  }, 2000);
}, [state.race, updateRaceData]);
```

## Key Improvements

1. **Precise State Detection**: Uses both `turn_phase` and `players_submitted` to distinguish scenarios
2. **Timing Fixes**: Added 500ms delay before data refresh to ensure backend consistency
3. **Better Logging**: Console logs for debugging turn processing flow
4. **Timeout Protection**: Prevents infinite polling with 60-second max
5. **Error Recovery**: Graceful handling of network errors and timeouts

## Testing Scenarios

### Single Player Race
1. Player selects boost (0-4)
2. Player clicks submit
3. Backend immediately processes turn (1/1 players)
4. Frontend detects `players_submitted: 0` and resets for next turn
5. Race data refreshes with updated positions
6. Player can submit next turn

### Multi-Player Race
1. First player submits → `players_submitted: 1, total_players: 2`
2. Frontend shows "waiting for players"
3. Second player submits → Backend auto-processes
4. Frontend polling detects completion and resets

## Files Modified

- `empty-project/src/contexts/PlayerGameContext.tsx`
  - Enhanced `submitBoostAction` response handling
  - Improved `startTurnCompletionPolling` with timeout protection
  - Added better logging and error handling

## Branch Information

- **Branch**: `feature/15-single-player-turn-processing-fix`
- **Status**: Implementation complete, ready for testing
- **Next Step**: End-to-end testing with single and multi-player races

## Testing Instructions

1. Start the full stack: `.\start-full-stack.ps1`
2. Create a single-player race
3. Submit boost actions and verify immediate processing
4. Check browser console for debugging logs
5. Verify race data updates correctly after each turn

The fix ensures single-player races work smoothly while maintaining compatibility with multi-player scenarios.