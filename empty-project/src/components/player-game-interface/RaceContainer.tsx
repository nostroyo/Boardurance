/**
 * RaceContainer - Main orchestrator component for single-player race experience
 *
 * This component manages the complete race flow:
 * - Race initialization and data fetching
 * - Turn-based racing loop (display → select → submit → poll → update)
 * - State management for race data, UI state, and errors
 * - Coordination of child components
 *
 * Performance optimizations:
 * - Memoized car data for entire race duration
 * - Cached performance preview for current lap
 * - Optimized re-renders with useMemo and useCallback
 *
 * Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 12.2, 12.4
 */

import { useState, useEffect, useCallback, useMemo, Suspense, lazy } from 'react';
import type {
  CarData,
  PerformancePreview,
  TurnPhase,
  LocalView,
  BoostAvailability,
  LapHistory,
} from '../../types/race-api';
import { raceAPIService } from '../../services/raceAPI';
import type { ErrorState } from '../../services/errorHandling';
import { createErrorState, withErrorHandling } from '../../services/errorHandling';
import { useRacePolling } from '../../hooks/useRacePolling';
import { useToast } from '../../hooks/useToast';
import { useLoadingState, LOADING_KEYS } from '../../hooks/useLoadingState';
// import { useDebounce } from '../../hooks/useDebouncedCallback';
import { ToastContainer } from './ToastContainer';
import { RaceLoadingState } from './RaceLoadingState';

// Lazy load heavy components for better performance (Requirements 12.5)
const RaceCompletionScreen = lazy(() => import('./RaceCompletionScreen'));
const RaceInterface = lazy(() => import('./RaceInterface'));

/**
 * Props for RaceContainer component
 */
export interface RaceContainerProps {
  raceUuid: string;
  playerUuid: string;
  onRaceComplete?: (finalPosition: number) => void;
  onError?: (error: Error) => void;
  onReturnToLobby?: () => void;
}

/**
 * Internal state interface for RaceContainer
 */
interface RaceContainerState {
  // Race data from backend
  carData: CarData | null;
  localView: LocalView | null;
  turnPhase: TurnPhase | null;
  performancePreview: PerformancePreview | null;
  boostAvailability: BoostAvailability | null;
  lapHistory: LapHistory | null;

  // UI state
  selectedBoost: number | null;
  isSubmitting: boolean;
  isPolling: boolean;
  hasSubmittedThisTurn: boolean;
  isRaceComplete: boolean;
  finalPosition: number | null;

  // Error state
  error: ErrorState | null;
}

/**
 * RaceContainer component
 */
export function RaceContainer({
  raceUuid,
  playerUuid,
  onRaceComplete,
  onError,
  onReturnToLobby,
}: RaceContainerProps) {
  // Toast notifications (Requirements 8.4)
  const { toasts, removeToast } = useToast();

  // Loading state management (Requirements 3.4)
  const { startLoading, stopLoading, updateLoading, isLoading, getLoadingState, isAnyLoading } =
    useLoadingState();

  // State management
  const [state, setState] = useState<RaceContainerState>({
    // Race data
    carData: null,
    localView: null,
    turnPhase: null,
    performancePreview: null,
    boostAvailability: null,
    lapHistory: null,

    // UI state
    selectedBoost: null,
    isSubmitting: false,
    isPolling: false,
    hasSubmittedThisTurn: false,
    isRaceComplete: false,
    finalPosition: null,

    // Error state
    error: null,
  });

  /**
   * Handle errors with user-friendly messages
   */
  const handleError = useCallback(
    (error: Error | string, context: string) => {
      const errorState = createErrorState(error, context);

      setState((prev) => ({
        ...prev,
        error: errorState,
        isSubmitting: false,
      }));

      // Stop all loading states on error
      stopLoading(LOADING_KEYS.RACE_INIT);
      stopLoading(LOADING_KEYS.PERFORMANCE_PREVIEW);
      stopLoading(LOADING_KEYS.SUBMIT_ACTION);

      // Call parent error handler if provided
      if (onError) {
        const errorObj = typeof error === 'string' ? new Error(error) : error;
        onError(errorObj);
      }

      console.error(`[RaceContainer] Error in ${context}:`, error);
    },
    [onError],
  );

  /**
   * Clear error state
   */
  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  // Note: Phase change, lap completion, and action submission handlers
  // will be implemented when RaceStatusPanel is integrated in future tasks

  // Memoized car data - cache for entire race duration (Requirements 12.2)
  const cachedCarData = useMemo(() => state.carData, [raceUuid, state.carData]);

  // Memoized performance preview - cache for current lap (Requirements 12.2)
  const cachedPerformancePreview = useMemo(() => {
    return state.performancePreview;
  }, [state.turnPhase?.current_lap, state.turnPhase?.lap_characteristic, state.performancePreview]);

  // Memoized boost availability for performance
  const cachedBoostAvailability = useMemo(() => state.boostAvailability, [state.boostAvailability]);

  /**
   * Initialize race - fetch initial data
   * Requirements: 1.1, 1.2, 1.3, 1.4, 1.5
   */
  const initializeRace = useCallback(async () => {
    console.log('[RaceContainer] Initializing race...');

    // Start loading with progress tracking
    startLoading(LOADING_KEYS.RACE_INIT, 'initial', {
      message: 'Loading race data...',
      progress: 0,
    });

    setState((prev) => ({
      ...prev,
      error: null,
    }));

    try {
      // Update progress as we fetch data
      updateLoading(LOADING_KEYS.RACE_INIT, {
        message: 'Fetching car data...',
        progress: 20,
      });

      // Fetch initial data in parallel for better performance
      const [carData, localView, turnPhase] = await Promise.all([
        withErrorHandling(
          () => raceAPIService.getCarData(raceUuid, playerUuid),
          'fetching car data',
        ),
        withErrorHandling(
          () => raceAPIService.getLocalView(raceUuid, playerUuid),
          'fetching local view',
        ),
        withErrorHandling(() => raceAPIService.getTurnPhase(raceUuid), 'fetching turn phase'),
      ]);

      updateLoading(LOADING_KEYS.RACE_INIT, {
        message: 'Processing race data...',
        progress: 80,
      });

      console.log('[RaceContainer] Initial data loaded successfully');

      // Update state with fetched data
      setState((prev) => ({
        ...prev,
        carData,
        localView,
        turnPhase,
        error: null,
      }));

      updateLoading(LOADING_KEYS.RACE_INIT, {
        message: 'Loading complete',
        progress: 100,
      });

      // Stop loading after a brief delay to show completion
      setTimeout(() => {
        stopLoading(LOADING_KEYS.RACE_INIT);
      }, 500);

      // Fetch additional data after initial load
      // These are less critical and can load after the main UI is displayed
      fetchPerformancePreview();

      // Fetch boost availability
      try {
        const boostAvailability = await raceAPIService.getBoostAvailability(raceUuid, playerUuid);
        setState((prev) => ({ ...prev, boostAvailability }));
      } catch (error) {
        console.warn('[RaceContainer] Failed to fetch boost availability:', error);
      }

      // Fetch lap history
      try {
        const lapHistory = await raceAPIService.getLapHistory(raceUuid, playerUuid);
        setState((prev) => ({ ...prev, lapHistory }));
      } catch (error) {
        console.warn('[RaceContainer] Failed to fetch lap history:', error);
      }
    } catch (error) {
      const errorObj = error instanceof Error ? error : new Error(String(error));
      stopLoading(LOADING_KEYS.RACE_INIT);
      handleError(errorObj, 'initializing race');
    }
  }, [raceUuid, playerUuid, handleError, startLoading, updateLoading, stopLoading]);

  /**
   * Fetch performance preview for current lap
   * Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7
   */
  const fetchPerformancePreview = useCallback(async () => {
    console.log('[RaceContainer] Fetching performance preview...');

    startLoading(LOADING_KEYS.PERFORMANCE_PREVIEW, 'data', {
      message: 'Loading performance preview...',
    });

    try {
      const performancePreview = await withErrorHandling(
        () => raceAPIService.getPerformancePreview(raceUuid, playerUuid),
        'fetching performance preview',
        undefined,
        (errorState) => {
          // Handle preview fetch errors without blocking the UI
          console.warn('[RaceContainer] Performance preview error:', errorState.message);
        },
      );

      console.log('[RaceContainer] Performance preview loaded successfully');

      setState((prev) => ({
        ...prev,
        performancePreview,
      }));

      stopLoading(LOADING_KEYS.PERFORMANCE_PREVIEW);
    } catch (error) {
      // Non-critical error - log but don't block UI
      console.error('[RaceContainer] Failed to fetch performance preview:', error);
      stopLoading(LOADING_KEYS.PERFORMANCE_PREVIEW);
    }
  }, [raceUuid, playerUuid, startLoading, stopLoading]);

  /**
   * Debounced performance preview fetching to avoid excessive API calls
   * Requirements: 12.3
   */
  // Note: This can be used in future enhancements for real-time preview updates
  // const debouncedFetchPerformancePreview = useDebounce(fetchPerformancePreview, 300);

  /**
   * Handle boost selection
   * Requirements: 3.1, 3.2
   */
  const handleBoostSelection = useCallback(
    (boost: number) => {
      console.log('[RaceContainer] Boost selected:', boost);

      // Validate boost value is in valid range (0-4)
      if (boost < 0 || boost > 4) {
        console.warn('[RaceContainer] Invalid boost value:', boost);
        handleError(
          new Error('Invalid boost value. Please select a boost between 0 and 4.'),
          'boost selection',
        );
        return;
      }

      // Validate boost is available
      if (state.boostAvailability) {
        const isAvailable = state.boostAvailability.available_cards.includes(boost);

        if (!isAvailable) {
          console.warn('[RaceContainer] Boost not available:', boost);
          handleError(
            new Error('This boost card has already been used. Please select an available card.'),
            'boost selection',
          );
          return;
        }
      }

      // Check if action already submitted
      if (state.hasSubmittedThisTurn) {
        console.warn('[RaceContainer] Action already submitted for this turn');
        handleError(
          new Error('You have already submitted your action for this turn.'),
          'boost selection',
        );
        return;
      }

      // Update selected boost
      setState((prev) => ({
        ...prev,
        selectedBoost: boost,
        error: null, // Clear any previous errors
      }));

      console.log('[RaceContainer] Boost selection updated successfully');
    },
    [state.boostAvailability, state.hasSubmittedThisTurn, handleError],
  );

  /**
   * Handle turn phase change
   * Requirements: 4.2
   */
  const handleTurnPhaseChange = useCallback((turnPhase: TurnPhase) => {
    console.log('[RaceContainer] Turn phase changed:', turnPhase.turn_phase);

    // Update turn phase in state
    setState((prev) => ({
      ...prev,
      turnPhase,
    }));

    // Log phase transition for debugging
    console.log(`[RaceContainer] Phase transition detected: ${turnPhase.turn_phase}`);
    console.log(`[RaceContainer] Current lap: ${turnPhase.current_lap}`);
    console.log(`[RaceContainer] Lap characteristic: ${turnPhase.lap_characteristic}`);

    // TODO: Add UI notifications for phase changes (will be implemented in later tasks)
  }, []);

  /**
   * Check if race is complete
   * Requirements: 7.1, 7.2, 7.3, 7.4, 7.5
   * This will be fully implemented in subtask 3.4
   */
  const checkRaceCompletion = useCallback(
    (localView: LocalView): boolean => {
      // Find the player in the visible participants
      const playerParticipant = localView.visible_participants.find(
        (p) => p.player_uuid === playerUuid,
      );

      if (playerParticipant && playerParticipant.is_finished) {
        console.log('[RaceContainer] Player has finished the race!');
        return true;
      }

      return false;
    },
    [playerUuid],
  );

  /**
   * Handle turn completion with optimized batched API calls
   * Requirements: 4.3, 4.4, 4.5, 4.6, 5.1, 12.3
   */
  const handleTurnComplete = useCallback(async () => {
    console.log('[RaceContainer] Turn complete, fetching updated race state...');

    try {
      // Use batched API call for better performance
      const batchResult = await withErrorHandling(
        () =>
          raceAPIService.batchRaceData(raceUuid, playerUuid, {
            includeLocalView: true,
            includeBoostAvailability: true,
            includeLapHistory: true,
            includePerformancePreview: true, // Fetch for next turn
          }),
        'fetching updated race state',
      );

      const { localView, boostAvailability, lapHistory, performancePreview } = batchResult;

      console.log('[RaceContainer] Updated race state fetched successfully');

      // Check if race is complete
      const isRaceComplete = localView ? checkRaceCompletion(localView) : false;

      // Find player's final position if race is complete
      let finalPosition: number | null = null;
      if (isRaceComplete && localView) {
        const playerParticipant = localView.visible_participants.find(
          (p) => p.player_uuid === playerUuid,
        );
        finalPosition = playerParticipant?.position_in_sector ?? null;
      }

      // Update state atomically with all new data
      setState((prev) => ({
        ...prev,
        localView: localView || prev.localView,
        boostAvailability: boostAvailability || prev.boostAvailability,
        lapHistory: lapHistory || prev.lapHistory,
        performancePreview: performancePreview || prev.performancePreview,
        hasSubmittedThisTurn: false, // Reset for next turn
        selectedBoost: null, // Clear selection for next turn
        isPolling: false, // Stop polling
        isRaceComplete, // Update race completion status
        finalPosition, // Store final position
        error: null,
      }));

      // Stop polling loading state
      stopLoading(LOADING_KEYS.POLLING);

      console.log('[RaceContainer] State updated, ready for next turn');

      // If race is complete, trigger completion callback
      if (isRaceComplete) {
        console.log('[RaceContainer] Race complete, triggering completion callback');

        if (onRaceComplete && finalPosition !== null) {
          onRaceComplete(finalPosition);
        }
      }
    } catch (error) {
      const errorObj = error instanceof Error ? error : new Error(String(error));

      console.error('[RaceContainer] Failed to fetch updated race state:', errorObj);

      handleError(errorObj, 'fetching updated race state');

      // Stop polling on error
      setState((prev) => ({
        ...prev,
        isPolling: false,
      }));

      stopLoading(LOADING_KEYS.POLLING);
    }
  }, [raceUuid, playerUuid, checkRaceCompletion, onRaceComplete, handleError, stopLoading]);

  /**
   * Handle polling errors
   */
  const handlePollingError = useCallback((error: Error) => {
    console.error('[RaceContainer] Polling error:', error);

    // Don't show error to user for transient polling errors
    // Just log them for debugging
    // The polling will automatically retry
  }, []);

  /**
   * Handle max polling attempts reached
   */
  const handleMaxAttemptsReached = useCallback(() => {
    console.warn('[RaceContainer] Max polling attempts reached');

    handleError(
      new Error('Turn processing is taking longer than expected. Please refresh the page.'),
      'polling timeout',
    );

    // Stop polling
    setState((prev) => ({
      ...prev,
      isPolling: false,
    }));
  }, [handleError]);

  /**
   * Set up polling hook
   * Requirements: 4.1, 12.1
   */
  const { isPolling: pollingActive } = useRacePolling({
    raceUuid,
    enabled: state.isPolling,
    onTurnPhaseChange: handleTurnPhaseChange,
    onComplete: handleTurnComplete,
    onError: handlePollingError,
    onMaxAttemptsReached: handleMaxAttemptsReached,
  });

  /**
   * Submit turn action to backend
   * Requirements: 3.3, 3.4, 3.5, 3.6, 3.7
   */
  const submitTurnAction = useCallback(async () => {
    console.log('[RaceContainer] Submitting turn action...');

    // Validate boost selection exists
    if (state.selectedBoost === null) {
      handleError(new Error('Please select a boost card before submitting.'), 'turn submission');
      return;
    }

    // Validate boost is available
    if (state.boostAvailability) {
      const isAvailable = state.boostAvailability.available_cards.includes(state.selectedBoost);

      if (!isAvailable) {
        handleError(
          new Error('Selected boost card is not available. Please select a different card.'),
          'turn submission',
        );
        return;
      }
    }

    // Check if already submitted
    if (state.hasSubmittedThisTurn) {
      handleError(
        new Error('You have already submitted your action for this turn.'),
        'turn submission',
      );
      return;
    }

    // Set submitting state and start loading
    startLoading(LOADING_KEYS.SUBMIT_ACTION, 'action', {
      message: 'Submitting your boost selection...',
    });

    setState((prev) => ({
      ...prev,
      isSubmitting: true,
      error: null,
    }));

    try {
      // Submit action to backend with retry logic
      await withErrorHandling(
        () => raceAPIService.submitTurnAction(raceUuid, playerUuid, state.selectedBoost!),
        'submitting turn action',
        undefined,
        (errorState) => {
          // Show error notification during retries
          console.warn('[RaceContainer] Submission retry:', errorState.message);
          updateLoading(LOADING_KEYS.SUBMIT_ACTION, {
            message: `Retrying... ${errorState.message}`,
          });
        },
      );

      console.log('[RaceContainer] Turn action submitted successfully');

      // Update state to reflect successful submission and start polling
      setState((prev) => ({
        ...prev,
        isSubmitting: false,
        hasSubmittedThisTurn: true,
        isPolling: true, // Start polling for turn completion
        error: null,
      }));

      stopLoading(LOADING_KEYS.SUBMIT_ACTION);

      // Start polling indicator
      startLoading(LOADING_KEYS.POLLING, 'polling', {
        message: 'Waiting for turn to complete...',
      });

      console.log('[RaceContainer] Started polling for turn completion');
    } catch (error) {
      const errorObj = error instanceof Error ? error : new Error(String(error));

      console.error('[RaceContainer] Failed to submit turn action:', errorObj);

      // Update state with error
      setState((prev) => ({
        ...prev,
        isSubmitting: false,
      }));

      stopLoading(LOADING_KEYS.SUBMIT_ACTION);
      handleError(errorObj, 'submitting turn action');
    }
  }, [
    raceUuid,
    playerUuid,
    state.selectedBoost,
    state.boostAvailability,
    state.hasSubmittedThisTurn,
    handleError,
    startLoading,
    updateLoading,
    stopLoading,
  ]);

  /**
   * Initialize race on component mount
   */
  useEffect(() => {
    initializeRace();
  }, [initializeRace]);

  // Render initial loading state
  const initialLoadingState = getLoadingState(LOADING_KEYS.RACE_INIT);
  if (initialLoadingState?.isLoading) {
    return (
      <>
        <ToastContainer toasts={toasts} onRemoveToast={removeToast} />
        <RaceLoadingState
          type="initial"
          message={initialLoadingState.message}
          progress={initialLoadingState.progress}
          showSkeleton={true}
        />
      </>
    );
  }

  // Render error state
  if (state.error) {
    return (
      <>
        <ToastContainer toasts={toasts} onRemoveToast={removeToast} />
        <div className="flex items-center justify-center min-h-screen bg-gray-900">
          <div className="bg-red-900 border border-red-700 rounded-lg p-6 max-w-md">
            <h2 className="text-white text-xl font-bold mb-2">Error</h2>
            <p className="text-red-200 mb-4">{state.error.message}</p>
            <div className="flex gap-2">
              {state.error.retryable && (
                <button
                  onClick={() => {
                    clearError();
                    initializeRace();
                  }}
                  className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded"
                >
                  Retry
                </button>
              )}
              <button
                onClick={clearError}
                className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded"
              >
                Dismiss
              </button>
            </div>
          </div>
        </div>
      </>
    );
  }

  // Race completion UI - Lazy loaded for better performance (Requirements 12.5)
  if (state.isRaceComplete) {
    return (
      <>
        {/* Toast Notifications (Requirements 8.4) */}
        <ToastContainer toasts={toasts} onRemoveToast={removeToast} />

        <Suspense
          fallback={
            <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
              <RaceLoadingState
                type="initial"
                message="Loading race completion screen..."
                showSkeleton={true}
              />
            </div>
          }
        >
          <RaceCompletionScreen
            finalPosition={state.finalPosition}
            carData={cachedCarData}
            lapHistory={state.lapHistory}
            onReturnToLobby={onReturnToLobby}
            onViewDetails={() => {
              // Scroll to details or handle view details action
              console.log('[RaceContainer] View details requested');
            }}
          />
        </Suspense>
      </>
    );
  }

  // Main race UI - Lazy loaded for better performance (Requirements 12.5)
  return (
    <>
      {/* Toast Notifications (Requirements 8.4) */}
      <ToastContainer toasts={toasts} onRemoveToast={removeToast} />

      <Suspense
        fallback={
          <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
            <RaceLoadingState
              type="initial"
              message="Loading race interface..."
              showSkeleton={true}
            />
          </div>
        }
      >
        <RaceInterface
          // Race data
          carData={cachedCarData}
          performancePreview={cachedPerformancePreview}
          turnPhase={state.turnPhase}
          localView={state.localView}
          boostAvailability={cachedBoostAvailability}
          lapHistory={state.lapHistory}
          // UI state
          selectedBoost={state.selectedBoost}
          isSubmitting={state.isSubmitting}
          hasSubmittedThisTurn={state.hasSubmittedThisTurn}
          isPolling={pollingActive}
          // Loading states
          isLoadingPreview={isLoading(LOADING_KEYS.PERFORMANCE_PREVIEW)}
          isLoadingSubmit={isLoading(LOADING_KEYS.SUBMIT_ACTION)}
          isAnyLoading={isAnyLoading()}
          // Event handlers
          onBoostSelect={handleBoostSelection}
          onSubmitAction={submitTurnAction}
          // Player info
          raceUuid={raceUuid}
          playerUuid={playerUuid}
        />
      </Suspense>
    </>
  );
}

export default RaceContainer;
