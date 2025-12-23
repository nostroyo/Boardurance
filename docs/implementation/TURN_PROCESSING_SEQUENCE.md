# Turn Processing Sequence Flow

## Complete Turn Processing Flow - From Boost Submission to Next Turn

```mermaid
sequenceDiagram
    participant P1 as Player 1 (Frontend)
    participant P2 as Player 2 (Frontend)
    participant BE as Backend API
    participant DB as MongoDB
    participant PROC as Turn Processor

    Note over P1,PROC: Turn N - Players Submit Boosts

    %% Player 1 submits boost
    P1->>BE: POST /races/{uuid}/submit-action
    Note right of P1: {player_uuid: "p1", boost_value: 2}
    
    BE->>DB: Store action in pending_actions[]
    BE->>P1: Response: {turn_phase: "WaitingForPlayers", players_submitted: 1, total_players: 2}
    
    Note over P1: Shows "Waiting for other players..."
    P1->>P1: Start polling /turn-phase every 2s

    %% Player 2 submits boost (last player)
    P2->>BE: POST /races/{uuid}/submit-action
    Note right of P2: {player_uuid: "p2", boost_value: 4}
    
    BE->>DB: Store action in pending_actions[]
    BE->>P2: Response: {turn_phase: "AllSubmitted", players_submitted: 2, total_players: 2}

    Note over P2: Receives "AllSubmitted" - triggers processing

    %% Frontend triggers turn processing
    P2->>BE: POST /races/{uuid}/turn
    Note right of P2: {actions: [{player_uuid: "p1", boost_value: 2}, {player_uuid: "p2", boost_value: 4}]}
    
    BE->>BE: Set turn_phase = "Processing"
    BE->>P2: Response: {turn_phase: "Processing"}

    %% Both players start polling
    Note over P1,P2: Both players polling /turn-phase
    P1->>BE: GET /races/{uuid}/turn-phase
    BE->>P1: {turn_phase: "Processing"}
    P2->>BE: GET /races/{uuid}/turn-phase  
    BE->>P2: {turn_phase: "Processing"}

    %% Backend processes the turn
    BE->>PROC: Process Turn N
    PROC->>PROC: Calculate performance with boosts
    PROC->>PROC: Update player positions
    PROC->>PROC: Update lap progress
    PROC->>PROC: Check if lap/race complete
    
    PROC->>DB: Update race state
    Note right of DB: - Clear pending_actions[]<br/>- Update participant positions<br/>- Increment current_lap if needed<br/>- Update boost hands<br/>- Set turn_phase = "WaitingForPlayers"

    PROC->>BE: Turn processing complete

    %% Players poll and get updated state
    P1->>BE: GET /races/{uuid}/turn-phase
    BE->>P1: {turn_phase: "WaitingForPlayers", current_lap: N+1}
    
    P2->>BE: GET /races/{uuid}/turn-phase
    BE->>P2: {turn_phase: "WaitingForPlayers", current_lap: N+1}

    %% Players fetch updated race data
    P1->>BE: GET /races/{uuid}/players/{p1}/local-view
    BE->>P1: Updated positions and sectors
    
    P1->>BE: GET /races/{uuid}/players/{p1}/boost-availability  
    BE->>P1: Updated boost hand (cards used/replenished)

    P2->>BE: GET /races/{uuid}/players/{p2}/local-view
    BE->>P2: Updated positions and sectors
    
    P2->>BE: GET /races/{uuid}/players/{p2}/boost-availability
    BE->>P2: Updated boost hand (cards used/replenished)

    Note over P1,P2: Turn N+1 - Ready for next boost submissions
    Note over P1: Reset hasSubmittedAction = false
    Note over P2: Reset hasSubmittedAction = false
    Note over P1,P2: Players can select new boosts
```

## Key State Transitions

### 1. Individual Submission Phase
- **State**: `WaitingForPlayers`
- **Action**: Players submit boosts individually
- **Response**: Current submission count and phase status

### 2. All Submitted Trigger
- **State**: `AllSubmitted` 
- **Action**: Last player's submission triggers turn processing
- **Critical**: Frontend must immediately call `/turn` endpoint

### 3. Processing Phase
- **State**: `Processing`
- **Action**: Backend calculates turn results
- **Duration**: 1-3 seconds typically
- **Frontend**: Polls every 2 seconds for completion

### 4. Next Turn Ready
- **State**: `WaitingForPlayers` (new turn)
- **Action**: Clear submitted flags, update race data
- **Frontend**: Fetch updated positions and boost availability

## Implementation Requirements

### Backend Changes Needed:

1. **Automatic Processing Option** (Alternative approach):
```rust
// In submit_turn_action, after storing the action:
if players_submitted >= total_players {
    // Option A: Trigger processing immediately
    tokio::spawn(async move {
        process_lap_in_db(&database, race_uuid, all_actions).await
    });
    
    return Ok(SubmitTurnActionResponse {
        turn_phase: "Processing".to_string(),
        // ...
    });
}
```

2. **Turn Processing Endpoint Enhancement**:
```rust
// Ensure process_turn clears pending_actions and resets state
pub async fn process_turn() {
    // Process the turn
    let result = process_lap_in_db(database, race_uuid, actions).await?;
    
    // Reset for next turn
    // - Clear pending_actions
    // - Set turn_phase = "WaitingForPlayers" 
    // - Update boost hands
    // - Increment lap if needed
}
```

### Frontend Changes Needed:

1. **Fix API Call**:
```typescript
// In PlayerGameContext.tsx
const submitBoostAction = async () => {
  // Use submitTurnAction instead of processRaceTurn
  const response = await raceAPIService.submitTurnAction(
    state.race.uuid, 
    state.playerUuid, 
    state.selectedBoost
  );
  
  if (response.turn_phase === "AllSubmitted") {
    // Trigger turn processing
    await triggerTurnProcessing();
  }
};
```

2. **Add Turn Processing Trigger**:
```typescript
const triggerTurnProcessing = async () => {
  // Get all pending actions and process the turn
  const actions = await getAllPendingActions();
  await raceAPIService.processRaceTurn(state.race.uuid, actions);
  
  // Start polling for completion
  startTurnPolling();
};
```

3. **Enhanced Polling Logic**:
```typescript
const startTurnPolling = () => {
  // Poll until turn_phase changes from "Processing" to "WaitingForPlayers"
  // Then refresh all race data and reset submission state
};
```

## Recommended Approach

I recommend **Option A: Backend Auto-Processing** because:

1. **Simpler Frontend**: No complex orchestration needed
2. **Atomic Operation**: Turn processing happens immediately when ready
3. **Better UX**: Faster transition, no additional API calls
4. **Fault Tolerant**: No risk of frontend failing to trigger processing

Would you like me to implement this approach?