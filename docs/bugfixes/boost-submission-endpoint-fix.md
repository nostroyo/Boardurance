# Boost Submission Endpoint Fix

## Issue Description

**Error**: "Resource not found. The race or player may no longer exist."

**Root Cause**: The frontend was attempting to POST to `/races/{race_uuid}/submit-action`, but this endpoint did not exist in the backend. The backend only had `/races/:race_uuid/turn` which expected a batch of actions from all players, not individual player submissions.

## Solution

### Backend Changes

1. **Added new endpoint**: `POST /races/{race_uuid}/submit-action`
   - Accepts individual player boost submissions
   - Stores actions in `pending_actions` until all players submit
   - Returns current turn phase status

2. **New request/response types**:
   ```rust
   pub struct SubmitTurnActionRequest {
       pub player_uuid: String,
       pub boost_value: u32,
   }

   pub struct SubmitTurnActionResponse {
       pub success: bool,
       pub message: String,
       pub turn_phase: String, // "WaitingForPlayers" or "AllSubmitted"
       pub players_submitted: u32,
       pub total_players: u32,
   }
   ```

3. **Implementation details**:
   - Validates race exists and is in progress
   - Validates player is a participant
   - Prevents duplicate submissions per turn
   - Updates turn phase based on submission count

### API Behavior

- **Success (200)**: Action submitted successfully
- **Bad Request (400)**: Invalid UUID or boost value
- **Not Found (404)**: Race or player doesn't exist
- **Conflict (409)**: Action already submitted or race not in progress

### Frontend Compatibility

The existing frontend code in `raceAPI.ts` already calls the correct endpoint:
```typescript
async submitTurnAction(
  raceUuid: string,
  playerUuid: string,
  boostValue: number,
): Promise<SubmitActionResponse>
```

## Testing

1. Start a race with multiple players
2. Submit boost selections individually
3. Verify turn phase updates correctly
4. Confirm all players can submit before turn processing

## Files Modified

- `rust-backend/src/routes/races.rs`: Added endpoint and implementation
- `rust-backend/src/startup.rs`: Added OpenAPI documentation
- `docs/bugfixes/boost-submission-endpoint-fix.md`: This documentation

## Related Issues

This fix resolves the mismatch between frontend expectations and backend API design for individual player action submissions in the simultaneous turn-based racing system.