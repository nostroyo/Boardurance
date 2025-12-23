# Turn Processing Implementation Plan

## Problem Statement

Currently, when all players submit their boosts (`AllSubmitted` state), there's no automatic turn processing. The race gets stuck waiting for manual intervention.

## Solution: Backend Auto-Processing

### Approach
When the last player submits their boost action, automatically trigger turn processing in the background and immediately return `Processing` state.

## Implementation Steps

### 1. Backend Changes

#### A. Modify `submit_turn_action` function
```rust
pub async fn submit_turn_action(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<SubmitTurnActionRequest>,
) -> Result<Json<SubmitTurnActionResponse>, StatusCode> {
    // ... existing validation code ...

    // Store the action
    collection.update_one(filter, update, None).await?;

    // Check if all players have submitted
    let players_submitted = race.pending_actions.len() as u32 + 1;
    let total_players = race.participants.len() as u32;
    
    if players_submitted >= total_players {
        // All players submitted - trigger automatic processing
        let database_clone = database.clone();
        let race_uuid_clone = race_uuid;
        
        tokio::spawn(async move {
            // Get all pending actions
            let all_actions: Vec<LapAction> = race.pending_actions;
            
            // Process the turn
            match process_lap_in_db(&database_clone, race_uuid_clone, all_actions).await {
                Ok(_) => {
                    tracing::info!("Turn processed automatically for race {}", race_uuid_clone);
                }
                Err(e) => {
                    tracing::error!("Failed to auto-process turn: {:?}", e);
                }
            }
        });
        
        return Ok(Json(SubmitTurnActionResponse {
            success: true,
            message: "Action submitted. Processing turn...".to_string(),
            turn_phase: "Processing".to_string(),
            players_submitted,
            total_players,
        }));
    }
    
    // Not all players submitted yet
    Ok(Json(SubmitTurnActionResponse {
        success: true,
        message: "Action submitted successfully".to_string(),
        turn_phase: "WaitingForPlayers".to_string(),
        players_submitted,
        total_players,
    }))
}
```

#### B. Enhance `process_lap_in_db` function
```rust
pub async fn process_lap_in_db(
    database: &Database,
    race_uuid: Uuid,
    actions: Vec<LapAction>,
) -> Result<Option<(LapResult, RaceStatus)>, mongodb::error::Error> {
    // ... existing processing logic ...
    
    // After processing, reset for next turn
    let reset_update = doc! {
        "$set": {
            "pending_actions": [],  // Clear pending actions
            "turn_phase": "WaitingForPlayers",  // Reset to waiting
            "updated_at": BsonDateTime::now()
        }
    };
    
    collection.update_one(
        doc! { "uuid": race_uuid.to_string() }, 
        reset_update, 
        None
    ).await?;
    
    Ok(Some((lap_result, race.status)))
}
```

### 2. Frontend Changes

#### A. Fix `submitBoostAction` in PlayerGameContext
```typescript
const submitBoostAction = useCallback(async () => {
  if (!state.race || !state.playerParticipant || state.hasSubmittedAction) {
    return;
  }

  dispatch({ type: 'SET_LOADING', payload: true });
  dispatch({ type: 'SET_ERROR', payload: null });

  try {
    // Use the correct individual submission endpoint
    const response = await raceAPIService.submitTurnAction(
      state.race.uuid,
      state.playerUuid,
      state.selectedBoost
    );

    if (response.success) {
      dispatch({ type: 'SET_HAS_SUBMITTED', payload: true });
      dispatch({ type: 'SET_TURN_PHASE', payload: response.turn_phase });
      
      // If processing started, begin polling for completion
      if (response.turn_phase === 'Processing') {
        startTurnCompletionPolling();
      }
    } else {
      dispatch({ type: 'SET_ERROR', payload: 'Failed to submit action' });
    }
  } catch (error) {
    dispatch({ type: 'SET_ERROR', payload: 'Network error while submitting action' });
  } finally {
    dispatch({ type: 'SET_LOADING', payload: false });
  }
}, [state.race, state.playerParticipant, state.hasSubmittedAction, state.selectedBoost, state.playerUuid]);
```

#### B. Add Turn Completion Polling
```typescript
const startTurnCompletionPolling = useCallback(() => {
  const pollInterval = setInterval(async () => {
    try {
      const turnPhase = await raceAPIService.getTurnPhase(state.race.uuid);
      
      if (turnPhase.turn_phase === 'WaitingForPlayers') {
        // Turn processing complete - refresh race data
        clearInterval(pollInterval);
        
        // Reset submission state for next turn
        dispatch({ type: 'SET_HAS_SUBMITTED', payload: false });
        dispatch({ type: 'SET_SELECTED_BOOST', payload: null });
        dispatch({ type: 'SET_TURN_PHASE', payload: 'WaitingForPlayers' });
        
        // Refresh race data
        await updateRaceData();
      }
    } catch (error) {
      console.error('Turn completion polling error:', error);
    }
  }, 2000); // Poll every 2 seconds
  
  // Cleanup after 60 seconds max
  setTimeout(() => clearInterval(pollInterval), 60000);
}, [state.race, updateRaceData]);
```

### 3. Race State Management

#### A. Add Turn Phase to Race Domain
```rust
// In race.rs domain
impl Race {
    pub fn all_actions_submitted(&self) -> bool {
        self.pending_actions.len() >= self.participants.len()
    }
    
    pub fn reset_for_next_turn(&mut self) {
        self.pending_actions.clear();
        // Don't reset turn_phase here - let the processing function handle it
    }
}
```

#### B. Enhanced Turn Phase Endpoint
```rust
pub async fn get_turn_phase(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<TurnPhaseResponse>, StatusCode> {
    // ... existing code ...
    
    // Add more detailed phase information
    let turn_phase = if race.status != RaceStatus::InProgress {
        "Complete".to_string()
    } else if race.pending_actions.is_empty() && race.current_lap > 1 {
        // New turn ready
        "WaitingForPlayers".to_string()
    } else if race.all_actions_submitted() {
        // Check if currently processing
        "Processing".to_string() // This will be set by the processing function
    } else {
        "WaitingForPlayers".to_string()
    };
    
    // ... rest of response ...
}
```

## Benefits of This Approach

1. **Automatic Processing**: No manual intervention needed
2. **Immediate Feedback**: Players see "Processing" immediately
3. **Fault Tolerant**: Backend handles all orchestration
4. **Scalable**: Works with any number of players
5. **Simple Frontend**: Just submit and poll for completion

## Testing Strategy

1. **Unit Tests**: Test auto-processing logic
2. **Integration Tests**: Test complete turn cycle
3. **Load Tests**: Test with multiple concurrent players
4. **Error Handling**: Test processing failures

## Rollout Plan

1. **Phase 1**: Implement backend auto-processing
2. **Phase 2**: Update frontend to use correct endpoints
3. **Phase 3**: Add enhanced polling and state management
4. **Phase 4**: Test complete turn cycles
5. **Phase 5**: Deploy and monitor

Would you like me to start implementing this solution?