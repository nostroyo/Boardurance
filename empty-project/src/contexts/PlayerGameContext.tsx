import React, { createContext, useContext, useReducer, useEffect, useCallback } from 'react';
import type {
  PlayerGameState,
  Race,
  RaceParticipant,
  LocalRaceView,
  AnimationState,
  TurnPhase,
} from '../types';
import type { TurnPhase as TurnPhaseResponse } from '../types/race-api';
import { raceAPIService } from '../services/raceAPI';
import { raceAPI } from '../utils/raceAPI';

// Context type definition
interface PlayerGameContextType {
  state: PlayerGameState;
  actions: {
    initializeRace: (raceUuid: string, playerUuid: string) => Promise<void>;
    updateRaceData: () => Promise<void>;
    selectBoost: (boost: number) => void;
    submitBoostAction: () => Promise<void>;
    setError: (error: string | null) => void;
    clearError: () => void;
    setAnimationState: (animationState: AnimationState) => void;
  };
}

// Initial state
const initialState: PlayerGameState = {
  race: null,
  localView: {
    centerSector: 0,
    visibleSectors: [],
    visibleParticipants: [],
  },
  playerUuid: '',
  playerParticipant: null,
  currentTurnPhase: 'WaitingForPlayers',
  selectedBoost: null,
  hasSubmittedAction: false,
  isLoading: false,
  error: null,
  animationState: {
    isAnimating: false,
    movements: [],
    duration: 0,
  },
};

// Action types
type PlayerGameAction =
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_RACE_DATA'; payload: Race }
  | { type: 'SET_PLAYER_UUID'; payload: string }
  | { type: 'SET_PLAYER_PARTICIPANT'; payload: RaceParticipant | null }
  | { type: 'SET_LOCAL_VIEW'; payload: LocalRaceView }
  | { type: 'SET_TURN_PHASE'; payload: TurnPhase }
  | { type: 'SET_SELECTED_BOOST'; payload: number | null }
  | { type: 'SET_HAS_SUBMITTED'; payload: boolean }
  | { type: 'SET_ANIMATION_STATE'; payload: AnimationState }
  | { type: 'RESET_STATE' };

// Reducer function
function playerGameReducer(state: PlayerGameState, action: PlayerGameAction): PlayerGameState {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };

    case 'SET_ERROR':
      return { ...state, error: action.payload, isLoading: false };

    case 'SET_RACE_DATA':
      return { ...state, race: action.payload };

    case 'SET_PLAYER_UUID':
      return { ...state, playerUuid: action.payload };

    case 'SET_PLAYER_PARTICIPANT':
      return { ...state, playerParticipant: action.payload };

    case 'SET_LOCAL_VIEW':
      return { ...state, localView: action.payload };

    case 'SET_TURN_PHASE':
      return { ...state, currentTurnPhase: action.payload };

    case 'SET_SELECTED_BOOST':
      return { ...state, selectedBoost: action.payload };

    case 'SET_HAS_SUBMITTED':
      return { ...state, hasSubmittedAction: action.payload };

    case 'SET_ANIMATION_STATE':
      return { ...state, animationState: action.payload };

    case 'RESET_STATE':
      return { ...initialState };

    default:
      return state;
  }
}

// Enhanced utility function to calculate local view (current sector ±2 sectors)
function calculateLocalView(race: Race, playerParticipant: RaceParticipant | null): LocalRaceView {
  if (!race || !playerParticipant) {
    return {
      centerSector: 0,
      visibleSectors: [],
      visibleParticipants: [],
    };
  }

  const centerSector = playerParticipant.current_sector;
  const allSectors = race.track.sectors.sort((a, b) => a.id - b.id);
  const totalSectors = allSectors.length;

  // Calculate visible sector IDs (center ±2, handling circular tracks)
  const visibleSectorIds: number[] = [];
  for (let offset = -2; offset <= 2; offset++) {
    let sectorId = centerSector + offset;

    // Handle wrapping for circular tracks
    if (sectorId < 0) {
      sectorId = totalSectors + sectorId;
    } else if (sectorId >= totalSectors) {
      sectorId = sectorId - totalSectors;
    }

    visibleSectorIds.push(sectorId);
  }

  // Get visible sectors in order
  const visibleSectors = visibleSectorIds
    .map((id) => allSectors.find((s) => s.id === id))
    .filter((sector) => sector !== undefined) as import('../types/race').Sector[];

  // Get participants in visible sectors
  const visibleParticipants = race.participants.filter((p) =>
    visibleSectorIds.includes(p.current_sector),
  );

  return {
    centerSector,
    visibleSectors,
    visibleParticipants,
  };
}

// Context creation
const PlayerGameContext = createContext<PlayerGameContextType | undefined>(undefined);

// Custom hook to use the context
export const usePlayerGameContext = () => {
  const context = useContext(PlayerGameContext);
  if (context === undefined) {
    throw new Error('usePlayerGameContext must be used within a PlayerGameProvider');
  }
  return context;
};

// Provider component
interface PlayerGameProviderProps {
  children: React.ReactNode;
}

export const PlayerGameProvider: React.FC<PlayerGameProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(playerGameReducer, initialState);

  // Initialize race data
  const initializeRace = useCallback(async (raceUuid: string, playerUuid: string) => {
    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });
    dispatch({ type: 'SET_PLAYER_UUID', payload: playerUuid });

    try {
      const response: any = await raceAPI.getRace(raceUuid);

      if (response.success && response.data) {
        const race = response.data;
        dispatch({ type: 'SET_RACE_DATA', payload: race });

        // Find player participant
        const playerParticipant =
          race.participants.find((p: any) => p.player_uuid === playerUuid) || null;
        dispatch({ type: 'SET_PLAYER_PARTICIPANT', payload: playerParticipant });

        // Calculate local view
        const localView = calculateLocalView(race, playerParticipant);
        dispatch({ type: 'SET_LOCAL_VIEW', payload: localView });

        // Get actual turn phase from backend instead of guessing
        try {
          const turnPhaseResponse: TurnPhaseResponse = await raceAPIService.getTurnPhase(raceUuid);
          if (turnPhaseResponse.turn_phase) {
            dispatch({ type: 'SET_TURN_PHASE', payload: turnPhaseResponse.turn_phase as TurnPhase });
          } else {
            // Fallback logic if turn phase call fails
            let turnPhase: TurnPhase = 'WaitingForPlayers';
            if (race.status === 'InProgress') {
              turnPhase = 'WaitingForPlayers';
            } else if (race.status === 'Finished') {
              turnPhase = 'Complete';
            }
            dispatch({ type: 'SET_TURN_PHASE', payload: turnPhase });
          }
        } catch (error) {
          console.error('Failed to get turn phase:', error);
          // Fallback logic if turn phase call fails
          let turnPhase: TurnPhase = 'WaitingForPlayers';
          if (race.status === 'InProgress') {
            turnPhase = 'WaitingForPlayers';
          } else if (race.status === 'Finished') {
            turnPhase = 'Complete';
          }
          dispatch({ type: 'SET_TURN_PHASE', payload: turnPhase });
        }
      } else {
        dispatch({ type: 'SET_ERROR', payload: response.error || 'Failed to load race data' });
      }
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: 'Network error while loading race data' });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, []);

  // Update race data (for polling)
  const updateRaceData = useCallback(async () => {
    if (!state.race) return;

    try {
      const response: any = await raceAPI.getRace(state.race.uuid);

      if (response.success && response.data) {
        const race = response.data;
        dispatch({ type: 'SET_RACE_DATA', payload: race });

        // Update player participant
        const playerParticipant =
          race.participants.find((p: any) => p.player_uuid === state.playerUuid) || null;
        dispatch({ type: 'SET_PLAYER_PARTICIPANT', payload: playerParticipant });

        // Recalculate local view
        const localView = calculateLocalView(race, playerParticipant);
        dispatch({ type: 'SET_LOCAL_VIEW', payload: localView });
      }
    } catch (error) {
      console.error('Failed to update race data:', error);
      // Don't set error for polling failures to avoid disrupting UI
    }
  }, [state.race, state.playerUuid]);

  // Select boost value
  const selectBoost = useCallback((boost: number) => {
    if (boost >= 0 && boost <= 4) {
      dispatch({ type: 'SET_SELECTED_BOOST', payload: boost });
    }
  }, []);

  // Turn completion polling - enhanced for better reliability
  const startTurnCompletionPolling = useCallback(() => {
    if (!state.race) return;

    console.log('Starting turn completion polling for race:', state.race.uuid);
    let pollCount = 0;
    const maxPolls = 30; // 60 seconds max (30 * 2s)

    const pollInterval = setInterval(async () => {
      pollCount++;
      
      try {
        const turnPhaseResponse: TurnPhaseResponse = await raceAPIService.getTurnPhase(state.race!.uuid);
        console.log(`Poll ${pollCount}: Turn phase is ${turnPhaseResponse.turn_phase}`);
        
        if (turnPhaseResponse.turn_phase === 'WaitingForPlayers') {
          // Turn processing complete - reset for next turn
          console.log('Turn processing completed, resetting for next turn');
          clearInterval(pollInterval);
          
          // Reset submission state
          dispatch({ type: 'SET_HAS_SUBMITTED', payload: false });
          dispatch({ type: 'SET_SELECTED_BOOST', payload: null });
          dispatch({ type: 'SET_TURN_PHASE', payload: 'WaitingForPlayers' });
          
          // Refresh race data with a small delay
          setTimeout(async () => {
            await updateRaceData();
          }, 500);
        } else if (pollCount >= maxPolls) {
          // Timeout - stop polling and reset anyway
          console.warn('Turn completion polling timed out, forcing reset');
          clearInterval(pollInterval);
          
          dispatch({ type: 'SET_HAS_SUBMITTED', payload: false });
          dispatch({ type: 'SET_SELECTED_BOOST', payload: null });
          dispatch({ type: 'SET_TURN_PHASE', payload: 'WaitingForPlayers' });
          dispatch({ type: 'SET_ERROR', payload: 'Turn processing took too long. Please refresh the page.' });
        }
      } catch (error) {
        console.error('Turn completion polling error:', error);
        
        if (pollCount >= maxPolls) {
          clearInterval(pollInterval);
          dispatch({ type: 'SET_ERROR', payload: 'Failed to check turn status. Please refresh the page.' });
        }
      }
    }, 2000); // Poll every 2 seconds
  }, [state.race, updateRaceData]);

  // Submit boost action
  const submitBoostAction = useCallback(async () => {
    if (!state.race || !state.playerParticipant || state.hasSubmittedAction || state.selectedBoost === null) {
      return;
    }

    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });

    try {
      // Use individual submission endpoint instead of batch processing
      const response = await raceAPIService.submitTurnAction(
        state.race.uuid,
        state.playerUuid,
        state.selectedBoost
      );

      // Response is SubmitActionResponse directly, not wrapped
      if (response.success) {
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
          
        } else if (response.turn_phase === 'Processing') {
          // Turn is being processed
          dispatch({ type: 'SET_HAS_SUBMITTED', payload: true });
          dispatch({ type: 'SET_TURN_PHASE', payload: 'Processing' });
          startTurnCompletionPolling();
          
        } else {
          // Handle other turn phases
          dispatch({ type: 'SET_HAS_SUBMITTED', payload: true });
          dispatch({ type: 'SET_TURN_PHASE', payload: response.turn_phase as TurnPhase });
        }
      } else {
        dispatch({ type: 'SET_ERROR', payload: response.message || 'Failed to submit action' });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Network error while submitting action';
      dispatch({ type: 'SET_ERROR', payload: errorMessage });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, [
    state.race,
    state.playerParticipant,
    state.playerUuid,
    state.selectedBoost,
    state.hasSubmittedAction,
    startTurnCompletionPolling,
    updateRaceData,
  ]);

  // Set error
  const setError = useCallback((error: string | null) => {
    dispatch({ type: 'SET_ERROR', payload: error });
  }, []);

  // Clear error
  const clearError = useCallback(() => {
    dispatch({ type: 'SET_ERROR', payload: null });
  }, []);

  // Set animation state
  const setAnimationState = useCallback((animationState: AnimationState) => {
    dispatch({ type: 'SET_ANIMATION_STATE', payload: animationState });
  }, []);

  // Polling effect for race updates
  useEffect(() => {
    if (!state.race || state.race.status === 'Finished') {
      return;
    }

    const pollInterval = setInterval(() => {
      updateRaceData();
    }, 2000); // Poll every 2 seconds

    return () => clearInterval(pollInterval);
  }, [state.race, updateRaceData]);

  const contextValue: PlayerGameContextType = {
    state,
    actions: {
      initializeRace,
      updateRaceData,
      selectBoost,
      submitBoostAction,
      setError,
      clearError,
      setAnimationState,
    },
  };

  return <PlayerGameContext.Provider value={contextValue}>{children}</PlayerGameContext.Provider>;
};
