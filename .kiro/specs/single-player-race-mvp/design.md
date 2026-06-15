# Design Document

## Overview

The Single Player Race MVP integrates the backend race API enhancements with the frontend player game interface to create a complete end-to-end racing experience. This design focuses on connecting React components to the six backend API endpoints, implementing the turn-based racing loop, and ensuring proper separation of concerns where the frontend acts as a display layer while the backend handles all game logic calculations.

The implementation builds upon existing components in `empty-project/src/components/player-game-interface/` and integrates them with the backend APIs at `/api/v1/races/{race_uuid}/`. The core racing loop consists of: fetch race state → display performance preview → select boost → submit action → poll for results → repeat until race completion.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     React Frontend                               │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐   │
│  │  RaceContainer (New)                                    │   │
│  │  - Orchestrates race flow                               │   │
│  │  - Manages polling and state updates                    │   │
│  │  - Coordinates child components                         │   │
│  └────────────────────────────────────────────────────────┘   │
│                           │                                     │
│         ┌─────────────────┼─────────────────┐                 │
│         ▼                 ▼                 ▼                  │
│  ┌──────────┐      ┌──────────┐     ┌──────────┐            │
│  │ Race     │      │ Boost    │     │ Local    │            │
│  │ Status   │      │ Selection│     │ View     │            │
│  │ Panel    │      │ UI       │     │ Display  │            │
│  └──────────┘      └──────────┘     └──────────┘            │
│                                                                │
└────────────────────────┬───────────────────────────────────────┘
                         │ HTTP Requests
                         │ (GET/POST)
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Backend API Endpoints                           │
│                                                                  │
│  GET  /car-data              - Car, pilot, engine, body info    │
│  GET  /performance-preview   - Boost options with predictions   │
│  GET  /turn-phase            - Current turn state               │
│  GET  /local-view            - Visible sectors (±2)             │
│  GET  /boost-availability    - Available boost cards            │
│  GET  /lap-history           - Performance history              │
│  POST /submit-action         - Submit boost selection           │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Component Hierarchy

```
RaceContainer (New)
├── RaceStatusPanel (Existing - Update)
│   ├── Current lap / total laps
│   ├── Lap characteristic indicator
│   ├── Turn phase status
│   └── Race timer
├── PlayerCarCard (Existing - Update)
│   ├── Car specifications
│   ├── Pilot information
│   ├── Engine/Body stats
│   └── Performance history
├── PerformancePreview (New)
│   ├── Base performance breakdown
│   ├── Sector ceiling visualization
│   ├── Boost options (0-4)
│   └── Movement probability indicators
├── BoostSelector (Existing - Update)
│   ├── Boost card selection UI
│   ├── Availability indicators
│   ├── Submit button
│   └── Loading state
└── LocalSectorDisplay (Existing - Update)
    ├── 5-sector view (current ±2)
    ├── Sector details
    ├── Participant positions
    └── Movement animations
```

### Data Flow

```
1. Race Initialization
   User navigates to race
   → RaceContainer fetches initial state
   → Display race info, car data, local view

2. Turn Loop (Repeats)
   a. Display Phase
      → Fetch performance preview
      → Show boost options with predictions
      → Display available boost cards
   
   b. Selection Phase
      → User selects boost card
      → Validate availability
      → Enable submit button
   
   c. Submission Phase
      → POST boost selection to backend
      → Show loading state
      → Disable further selections
   
   d. Processing Phase
      → Poll turn phase endpoint
      → Wait for "Processing" → "Complete"
      → Fetch updated race state
   
   e. Update Phase
      → Display new sector position
      → Show movement animation
      → Update lap number if changed
      → Refresh boost availability
      → Loop back to Display Phase

3. Race Completion
   Detect player finished
   → Display completion message
   → Show final lap history
   → Disable turn controls
   → Offer navigation options
```

## Components and Interfaces

### 1. RaceContainer (New Component)

**Purpose**: Main orchestrator component that manages the complete race flow and coordinates all child components.

**Location**: `empty-project/src/components/player-game-interface/RaceContainer.tsx`

**Props**:
```typescript
interface RaceContainerProps {
  raceUuid: string;
  playerUuid: string;
  onRaceComplete?: (finalPosition: number) => void;
  onError?: (error: Error) => void;
}
```

**State**:
```typescript
interface RaceContainerState {
  // Race data
  raceState: RaceState | null;
  carData: CarData | null;
  localView: LocalView | null;
  performancePreview: PerformancePreview | null;
  lapHistory: LapHistory | null;
  
  // UI state
  selectedBoost: number | null;
  isSubmitting: boolean;
  isPolling: boolean;
  hasSubmittedThisTurn: boolean;
  
  // Error state
  error: string | null;
  
  // Loading states
  isLoadingInitial: boolean;
  isLoadingPreview: boolean;
}
```

**Key Methods**:
- `initializeRace()` - Fetch initial race state, car data, local view
- `fetchPerformancePreview()` - Get performance predictions for all boost options
- `handleBoostSelection(boost: number)` - Update selected boost
- `submitTurnAction()` - POST boost selection to backend
- `startPolling()` - Begin polling for turn completion
- `stopPolling()` - End polling when turn completes
- `handleTurnComplete()` - Process turn completion and prepare for next turn
- `checkRaceCompletion()` - Detect if player has finished race

**Polling Strategy**:
```typescript
const POLL_INTERVAL = 2000; // 2 seconds
const MAX_POLL_ATTEMPTS = 60; // 2 minutes max

useEffect(() => {
  if (!isPolling) return;
  
  const pollTimer = setInterval(async () => {
    const turnPhase = await fetchTurnPhase();
    
    if (turnPhase.phase === 'Complete') {
      stopPolling();
      await handleTurnComplete();
    }
  }, POLL_INTERVAL);
  
  return () => clearInterval(pollTimer);
}, [isPolling]);
```

### 2. RaceStatusPanel (Update Existing)

**Purpose**: Display current race status including lap, turn phase, and race progress.

**Location**: `empty-project/src/components/player-game-interface/RaceStatusPanel.tsx`

**Props**:
```typescript
interface RaceStatusPanelProps {
  currentLap: number;
  totalLaps: number;
  lapCharacteristic: 'Straight' | 'Curve';
  turnPhase: TurnPhase;
  raceStatus: 'NotStarted' | 'InProgress' | 'Completed';
}
```

**Updates Needed**:
- Add turn phase color indicators (Waiting=yellow, AllSubmitted=blue, Processing=orange, Complete=green)
- Display lap characteristic with icon (🏁 for Straight, 🌀 for Curve)
- Show turn phase description text
- Add progress bar for lap completion

### 3. PerformancePreview (New Component)

**Purpose**: Display performance calculations and boost options from backend.

**Location**: `empty-project/src/components/player-game-interface/PerformancePreview.tsx`

**Props**:
```typescript
interface PerformancePreviewProps {
  preview: PerformancePreview;
  selectedBoost: number | null;
  onBoostSelect: (boost: number) => void;
  availableBoosts: number[];
}
```

**Display Structure**:
```
┌─────────────────────────────────────────┐
│ Base Performance Breakdown              │
│ ├─ Engine: 25                           │
│ ├─ Body: 20                             │
│ ├─ Pilot: 15                            │
│ └─ Base Total: 60                       │
│                                          │
│ Sector Ceiling: 50                      │
│ Capped Base: 50                         │
│                                          │
│ Boost Options:                          │
│ ┌──────────────────────────────────┐   │
│ │ [0] 50  ⚪ Stay     (Used)       │   │
│ │ [1] 54  ⬆️ MoveUp   (Available)  │   │
│ │ [2] 58  ⬆️ MoveUp   (Available)  │   │
│ │ [3] 62  ⬆️ MoveUp   (Available)  │   │
│ │ [4] 66  ⬆️ MoveUp   (Available)  │   │
│ └──────────────────────────────────┘   │
│                                          │
│ Boost Cycle: 2/5 cards used            │
│ Next replenishment: Lap 8               │
└─────────────────────────────────────────┘
```

### 4. BoostSelector (Update Existing)

**Purpose**: UI for selecting boost card with availability validation.

**Location**: `empty-project/src/components/player-game-interface/BoostSelector.tsx`

**Props**:
```typescript
interface BoostSelectorProps {
  selectedBoost: number | null;
  availableBoosts: number[];
  onBoostSelect: (boost: number) => void;
  onSubmit: () => void;
  isSubmitting: boolean;
  hasSubmitted: boolean;
}
```

**Updates Needed**:
- Disable unavailable boost cards visually
- Show "Already Used" badge on unavailable cards
- Disable submit button if no boost selected or boost unavailable
- Show loading spinner during submission
- Display "Action Submitted" state after successful submission

### 5. LocalSectorDisplay (Update Existing)

**Purpose**: Display 5-sector local view with participant positions.

**Location**: `empty-project/src/components/player-game-interface/LocalSectorDisplay.tsx`

**Props**:
```typescript
interface LocalSectorDisplayProps {
  localView: LocalView;
  playerUuid: string;
  onSectorClick?: (sectorId: number) => void;
}
```

**Updates Needed**:
- Fetch data from backend local-view endpoint instead of calculating locally
- Highlight player's current sector with distinct styling
- Show sector occupancy and capacity
- Display participant names and positions within sectors
- Add smooth animations for position changes

### 6. PlayerCarCard (Update Existing)

**Purpose**: Display player's car, pilot, engine, and body information.

**Location**: `empty-project/src/components/player-game-interface/PlayerCarCard.tsx`

**Props**:
```typescript
interface PlayerCarCardProps {
  carData: CarData;
  lapHistory?: LapHistory;
}
```

**Updates Needed**:
- Fetch data from backend car-data endpoint
- Display pilot skills breakdown (reaction_time, precision, focus, stamina)
- Show engine and body performance values for straight/curve
- Add lap history visualization if provided

## Data Models

### API Response Types

```typescript
// From backend car-data endpoint
interface CarData {
  car: {
    uuid: string;
    name: string;
  };
  pilot: {
    uuid: string;
    name: string;
    pilot_class: string;
    rarity: string;
    skills: {
      reaction_time: number;
      precision: number;
      focus: number;
      stamina: number;
    };
    performance: {
      straight_value: number;
      curve_value: number;
    };
  };
  engine: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
  };
  body: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
  };
}

// From backend performance-preview endpoint
interface PerformancePreview {
  base_performance: {
    engine_contribution: number;
    body_contribution: number;
    pilot_contribution: number;
    base_value: number;
    sector_ceiling: number;
    capped_base_value: number;
    lap_characteristic: 'Straight' | 'Curve';
  };
  boost_options: Array<{
    boost_value: number;
    is_available: boolean;
    final_value: number;
    movement_probability: 'MoveUp' | 'Stay' | 'MoveDown';
  }>;
  boost_cycle_info: {
    current_cycle: number;
    cycles_completed: number;
    cards_remaining: number;
    available_cards: number[];
  };
}

// From backend turn-phase endpoint
interface TurnPhase {
  turn_phase: 'WaitingForPlayers' | 'AllSubmitted' | 'Processing' | 'Complete';
  current_lap: number;
  lap_characteristic: 'Straight' | 'Curve';
  submitted_players: string[];
  pending_players: string[];
  total_active_players: number;
}

// From backend local-view endpoint
interface LocalView {
  center_sector: number;
  visible_sectors: Array<{
    id: number;
    name: string;
    min_value: number;
    max_value: number;
    slot_capacity: number | null;
    sector_type: string;
    current_occupancy: number;
  }>;
  visible_participants: Array<{
    player_uuid: string;
    player_name: string | null;
    car_name: string;
    current_sector: number;
    position_in_sector: number;
    total_value: number;
    current_lap: number;
    is_finished: boolean;
  }>;
}

// From backend boost-availability endpoint
interface BoostAvailability {
  available_cards: number[];
  hand_state: Record<string, boolean>;
  current_cycle: number;
  cycles_completed: number;
  cards_remaining: number;
  next_replenishment_at: number | null;
}

// From backend lap-history endpoint
interface LapHistory {
  laps: Array<{
    lap_number: number;
    lap_characteristic: string;
    boost_used: number;
    boost_cycle: number;
    base_value: number;
    final_value: number;
    from_sector: number;
    to_sector: number;
    movement_type: string;
  }>;
  cycle_summaries: Array<{
    cycle_number: number;
    cards_used: number[];
    laps_in_cycle: number[];
    average_boost: number;
  }>;
}
```

### API Service Layer

**Location**: `empty-project/src/services/raceAPI.ts`

```typescript
export class RaceAPIService {
  private baseUrl: string;
  
  constructor(baseUrl: string = '/api/v1') {
    this.baseUrl = baseUrl;
  }
  
  async getCarData(raceUuid: string, playerUuid: string): Promise<CarData> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/car-data`
    );
    if (!response.ok) throw new Error('Failed to fetch car data');
    return response.json();
  }
  
  async getPerformancePreview(
    raceUuid: string,
    playerUuid: string
  ): Promise<PerformancePreview> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/performance-preview`
    );
    if (!response.ok) throw new Error('Failed to fetch performance preview');
    return response.json();
  }
  
  async getTurnPhase(raceUuid: string): Promise<TurnPhase> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/turn-phase`
    );
    if (!response.ok) throw new Error('Failed to fetch turn phase');
    return response.json();
  }
  
  async getLocalView(
    raceUuid: string,
    playerUuid: string
  ): Promise<LocalView> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/local-view`
    );
    if (!response.ok) throw new Error('Failed to fetch local view');
    return response.json();
  }
  
  async getBoostAvailability(
    raceUuid: string,
    playerUuid: string
  ): Promise<BoostAvailability> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/boost-availability`
    );
    if (!response.ok) throw new Error('Failed to fetch boost availability');
    return response.json();
  }
  
  async getLapHistory(
    raceUuid: string,
    playerUuid: string
  ): Promise<LapHistory> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/players/${playerUuid}/lap-history`
    );
    if (!response.ok) throw new Error('Failed to fetch lap history');
    return response.json();
  }
  
  async submitTurnAction(
    raceUuid: string,
    playerUuid: string,
    boostValue: number
  ): Promise<void> {
    const response = await fetch(
      `${this.baseUrl}/races/${raceUuid}/submit-action`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          player_uuid: playerUuid,
          boost_value: boostValue
        })
      }
    );
    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to submit action');
    }
  }
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

**Property 1: Boost availability display consistency**
*For any* boost hand state received from the backend, the UI should display availability that exactly matches the backend response
**Validates: Requirements 2.5**

**Property 2: Boost selection validation**
*For any* boost card selection, if the card is not in the available_cards list from the backend, the submission should be prevented
**Validates: Requirements 3.2**

**Property 3: Turn phase transition detection**
*For any* sequence of turn phase states, when the phase changes from one value to another, the UI should detect and respond to the change
**Validates: Requirements 4.2**

**Property 4: Race completion detection**
*For any* race state where the player's is_finished flag is true, the UI should display the race completion state
**Validates: Requirements 7.1**

**Property 5: Turn phase color mapping**
*For any* turn phase value, the UI should apply the correct color indicator (WaitingForPlayers=yellow, AllSubmitted=blue, Processing=orange, Complete=green)
**Validates: Requirements 8.1**

**Property 6: Network error handling**
*For any* API request that fails with a network error, the UI should display a user-friendly error message
**Validates: Requirements 9.1**

**Property 7: No local game logic calculations**
*For any* game state display, the data should originate from a backend API response, not from local calculations
**Validates: Requirements 10.1, 10.2, 10.3, 10.4**

**Property 8: Backend API data source**
*For any* displayed game information (performance, movement, boost state), the source should be a backend API endpoint
**Validates: Requirements 10.4**

## Error Handling

### Error Categories

**1. Network Errors**
- Connection timeout
- Server unavailable
- DNS resolution failure

**Strategy**: 
- Display toast notification: "Connection lost. Retrying..."
- Implement exponential backoff (1s, 2s, 4s, 8s)
- Max 5 retry attempts
- Show "Unable to connect" after max retries with manual retry button

**2. API Errors**
- 400 Bad Request (invalid boost selection)
- 404 Not Found (race or player not found)
- 409 Conflict (race not in progress, player finished)
- 500 Internal Server Error

**Strategy**:
- Parse error message from backend response
- Display specific error in toast notification
- For 404 errors: redirect to race lobby
- For 409 errors: refresh race state
- For 400 errors: show validation message and allow correction

**3. State Inconsistency Errors**
- Frontend state doesn't match backend
- Unexpected turn phase transition
- Missing required data

**Strategy**:
- Log error to console for debugging
- Force refresh from backend
- Display: "Synchronizing race state..."
- If refresh fails, show error and offer to return to lobby

**4. Validation Errors**
- Boost card not available
- No boost selected
- Action already submitted

**Strategy**:
- Prevent submission with disabled button
- Show inline validation message
- Highlight the issue in UI

### Error Response Format

```typescript
interface ErrorState {
  type: 'network' | 'api' | 'validation' | 'state';
  message: string;
  retryable: boolean;
  retryCount: number;
  originalError?: Error;
}

const handleError = (error: Error, context: string): ErrorState => {
  // Categorize error
  // Determine if retryable
  // Format user-friendly message
  // Log for debugging
  return errorState;
};
```

## Testing Strategy

### Unit Tests

**Component Tests**:
- RaceContainer state management
- PerformancePreview rendering with various boost options
- BoostSelector availability validation
- RaceStatusPanel turn phase display
- LocalSectorDisplay sector highlighting

**Service Tests**:
- RaceAPIService request formatting
- Error parsing and handling
- Response data transformation

**Utility Tests**:
- Polling interval timing
- Retry logic with exponential backoff
- State transition detection

### Integration Tests

**API Integration Tests**:
- Mock backend responses for all 6 endpoints
- Test complete race flow from start to finish
- Verify correct API calls at each stage
- Test error handling for each endpoint

**User Flow Tests**:
1. **Race Initialization Flow**
   - Navigate to race
   - Verify initial data fetch (car data, local view, turn phase)
   - Confirm UI displays all information

2. **Turn Submission Flow**
   - Select boost card
   - Submit action
   - Verify POST request sent
   - Confirm UI shows "submitted" state

3. **Turn Processing Flow**
   - Start polling after submission
   - Detect turn phase change
   - Fetch updated race state
   - Verify UI updates with new position

4. **Multi-Lap Flow**
   - Complete multiple turns
   - Verify lap transitions
   - Check boost replenishment
   - Confirm lap history updates

5. **Race Completion Flow**
   - Detect race finish
   - Display completion message
   - Show final lap history
   - Verify controls disabled

**Error Scenario Tests**:
- Network failure during fetch
- Network failure during submission
- Invalid boost selection
- Race not found
- Player already finished

### Property-Based Tests

Due to the frontend nature of this feature (UI display and API integration), traditional property-based testing is less applicable. However, we can implement property-style tests:

**Property Test 1: API Response Display Consistency**
- Generate random valid API responses
- Verify UI always displays the data correctly
- Ensure no local calculations modify the data

**Property Test 2: Boost Availability Validation**
- Generate random boost hand states
- Verify UI always prevents selection of unavailable cards
- Ensure submission is blocked for invalid selections

**Property Test 3: Turn Phase State Machine**
- Generate random sequences of turn phases
- Verify UI transitions are always valid
- Ensure no invalid state transitions occur

## Implementation Notes

### Performance Optimization

**1. Caching Strategy**
```typescript
// Cache car data for entire race (doesn't change)
const carDataCache = useMemo(() => carData, [raceUuid]);

// Cache performance preview for current lap
const previewCache = useMemo(
  () => performancePreview,
  [currentLap, lapCharacteristic]
);
```

**2. Debouncing**
```typescript
// Debounce boost selection to avoid excessive preview fetches
const debouncedBoostSelect = useMemo(
  () => debounce((boost: number) => {
    fetchPerformancePreview();
  }, 300),
  []
);
```

**3. Memoization**
```typescript
// Memoize expensive components
const MemoizedLocalSectorDisplay = React.memo(LocalSectorDisplay);
const MemoizedPerformancePreview = React.memo(PerformancePreview);
```

**4. Lazy Loading**
```typescript
// Lazy load lap history component
const LapHistoryPanel = lazy(() => 
  import('./LapHistoryPanel')
);
```

### Polling Management

**Start Polling Conditions**:
- After successful action submission
- When turn phase is "WaitingForPlayers" or "AllSubmitted"

**Stop Polling Conditions**:
- Turn phase becomes "Complete"
- Race is finished
- Component unmounts
- Max poll attempts reached
- Error occurs

**Polling Implementation**:
```typescript
const useRacePolling = (
  raceUuid: string,
  enabled: boolean,
  onComplete: () => void
) => {
  useEffect(() => {
    if (!enabled) return;
    
    let attempts = 0;
    const maxAttempts = 60;
    
    const poll = async () => {
      try {
        const turnPhase = await raceAPI.getTurnPhase(raceUuid);
        
        if (turnPhase.turn_phase === 'Complete') {
          onComplete();
          return;
        }
        
        attempts++;
        if (attempts < maxAttempts) {
          setTimeout(poll, 2000);
        }
      } catch (error) {
        console.error('Polling error:', error);
        // Continue polling on error
        if (attempts < maxAttempts) {
          setTimeout(poll, 2000);
        }
      }
    };
    
    poll();
    
    return () => {
      attempts = maxAttempts; // Stop polling on unmount
    };
  }, [raceUuid, enabled, onComplete]);
};
```

### State Management

**Local State vs Context**:
- Use local state in RaceContainer for race-specific data
- Consider React Context if multiple sibling components need shared state
- Avoid prop drilling by using context for deeply nested components

**State Update Strategy**:
```typescript
// Atomic state updates
const handleTurnComplete = async () => {
  // Fetch all updated data
  const [localView, boostAvailability, lapHistory] = await Promise.all([
    raceAPI.getLocalView(raceUuid, playerUuid),
    raceAPI.getBoostAvailability(raceUuid, playerUuid),
    raceAPI.getLapHistory(raceUuid, playerUuid)
  ]);
  
  // Update state atomically
  setState(prev => ({
    ...prev,
    localView,
    boostAvailability,
    lapHistory,
    hasSubmittedThisTurn: false,
    selectedBoost: null
  }));
};
```

### Routing Integration

**Route Structure**:
```
/races/:raceUuid/play
```

**Component Integration**:
```typescript
// In App.tsx or router configuration
<Route 
  path="/races/:raceUuid/play" 
  element={
    <ProtectedRoute>
      <RaceContainer 
        raceUuid={params.raceUuid}
        playerUuid={currentUser.uuid}
        onRaceComplete={(position) => {
          navigate('/races/lobby');
        }}
        onError={(error) => {
          console.error(error);
          navigate('/races/lobby');
        }}
      />
    </ProtectedRoute>
  } 
/>
```

## Dependencies

### Existing Dependencies (No New Packages Needed)
- React 19.1.1
- TypeScript 5.8.3
- React Router DOM (for navigation)
- Existing API utilities in `src/utils/raceAPI.ts`

### Internal Dependencies
- Backend race API endpoints (already implemented)
- Existing player-game-interface components
- Authentication context for player UUID

## Security Considerations

**1. Input Validation**
- Validate boost selection is 0-4 before submission
- Verify player UUID matches authenticated user
- Sanitize all user inputs

**2. API Security**
- Use HTTP-only cookies for authentication
- Include CSRF tokens in POST requests
- Validate all API responses before using data

**3. Data Exposure**
- Only display data for the authenticated player
- Don't expose other players' boost selections
- Hide sensitive race data until appropriate

**4. Client-Side Security**
- Don't store sensitive data in localStorage
- Clear race data on logout
- Implement proper error boundaries

## Future Enhancements

**1. Real-Time Updates**
- WebSocket integration for instant turn updates
- Eliminate polling for better performance
- Real-time notifications for race events

**2. Multiplayer Synchronization**
- Display other players' actions in real-time
- Show "waiting for players" with countdown
- Implement spectator mode

**3. Enhanced Visualizations**
- 3D track visualization
- Animated car movements
- Performance graphs and charts

**4. Offline Support**
- Cache race data for offline viewing
- Queue actions when offline
- Sync when connection restored

**5. Mobile Optimization**
- Touch gestures for boost selection
- Responsive layout for small screens
- Reduced data usage mode

**6. Analytics**
- Track user behavior and race patterns
- Performance metrics dashboard
- Strategy recommendations based on history
