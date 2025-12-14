import React, { useEffect, useCallback, useRef } from 'react';
import type { PlayerGameInterfaceProps } from '../../types';
import { usePlayerGameContext } from '../../contexts/PlayerGameContext';
import { raceAPI, raceStatusUtils, raceErrorUtils } from '../../utils/raceAPI';

// Component imports (to be created in subsequent tasks)
// import { RaceStatusPanel } from './RaceStatusPanel';
// import { LocalSectorDisplay } from './LocalSectorDisplay';
// import { PlayerCarCard } from './PlayerCarCard';
// import { PerformanceCalculator } from './PerformanceCalculator';
// import { SimultaneousTurnController } from './SimultaneousTurnController';

const PlayerGameInterface: React.FC<PlayerGameInterfaceProps> = ({
  raceUuid,
  playerUuid,
  onRaceComplete,
  onError,
}) => {
  const { state, actions } = usePlayerGameContext();
  const pollingRef = useRef<number | null>(null);
  const retryCountRef = useRef(0);
  const maxRetries = 3;

  // Enhanced race initialization with error recovery
  const initializeRaceWithRetry = useCallback(async () => {
    try {
      await actions.initializeRace(raceUuid, playerUuid);
      retryCountRef.current = 0; // Reset retry count on success
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to initialize race';

      if (retryCountRef.current < maxRetries && raceErrorUtils.isRetryableError(errorMessage)) {
        retryCountRef.current++;
        const delay = 1000 * Math.pow(2, retryCountRef.current - 1); // Exponential backoff

        setTimeout(() => {
          initializeRaceWithRetry();
        }, delay);
      } else {
        actions.setError(raceErrorUtils.getUserFriendlyError(errorMessage));
      }
    }
  }, [raceUuid, playerUuid, actions]);

  // Initialize race on component mount
  useEffect(() => {
    initializeRaceWithRetry();
  }, [initializeRaceWithRetry]);

  // Enhanced real-time race data polling with turn phase synchronization
  useEffect(() => {
    if (!state.race || state.race.status === 'Finished' || state.race.status === 'Cancelled') {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
      return;
    }

    // Start polling for race updates
    pollingRef.current = setInterval(async () => {
      try {
        // Update race data
        await actions.updateRaceData();

        // Check turn phase if race is in progress
        if (state.race?.status === 'InProgress') {
          const turnPhaseResponse = await raceAPI.getTurnPhase(raceUuid);
          if (turnPhaseResponse.success && turnPhaseResponse.data) {
            const newTurnPhase = turnPhaseResponse.data as any;
            if (newTurnPhase !== state.currentTurnPhase) {
              // Turn phase changed - reset submission status if new turn started
              if (newTurnPhase === 'WaitingForPlayers' && state.hasSubmittedAction) {
                actions.setError(null); // Clear any previous errors
              }
            }
          }
        }
      } catch (error) {
        console.error('Polling error:', error);
        // Don't set error for polling failures to avoid disrupting UI
      }
    }, 2000); // Poll every 2 seconds

    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [state.race, raceUuid, actions, state.currentTurnPhase, state.hasSubmittedAction]);

  // Handle race completion with enhanced feedback
  useEffect(() => {
    if (state.race?.status === 'Finished' && state.playerParticipant?.finish_position) {
      // Stop polling when race is finished
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }

      if (onRaceComplete) {
        onRaceComplete(state.playerParticipant.finish_position);
      }
    }
  }, [state.race?.status, state.playerParticipant?.finish_position, onRaceComplete]);

  // Enhanced error handling with user-friendly messages
  useEffect(() => {
    if (state.error) {
      const userFriendlyError = raceErrorUtils.getUserFriendlyError(state.error);
      if (onError) {
        onError(userFriendlyError);
      }
    }
  }, [state.error, onError]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, []);

  // Enhanced retry function with exponential backoff
  const handleRetry = useCallback(() => {
    actions.clearError();
    retryCountRef.current = 0;
    initializeRaceWithRetry();
  }, [actions, initializeRaceWithRetry]);

  // Enhanced loading state with progress indication
  if (state.isLoading && !state.race) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-lg mb-2">Loading race data...</p>
          {retryCountRef.current > 0 && (
            <p className="text-sm text-gray-400">
              Retry attempt {retryCountRef.current} of {maxRetries}
            </p>
          )}
        </div>
      </div>
    );
  }

  // Enhanced error state with detailed feedback and recovery options
  if (state.error && !state.race) {
    const isRetryable = raceErrorUtils.isRetryableError(state.error);
    const requiresAction = raceErrorUtils.requiresUserAction(state.error);

    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center max-w-md mx-auto px-4">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h2 className="text-2xl font-bold mb-2">Error Loading Race</h2>
          <p className="text-gray-300 mb-4">{raceErrorUtils.getUserFriendlyError(state.error)}</p>

          <div className="space-y-3">
            {isRetryable && (
              <button
                onClick={handleRetry}
                disabled={state.isLoading}
                className="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-6 py-2 rounded-lg font-medium transition-colors w-full"
              >
                {state.isLoading ? 'Retrying...' : 'Retry'}
              </button>
            )}

            {requiresAction && (
              <p className="text-sm text-yellow-400">
                This error requires manual intervention. Please check your race participation
                status.
              </p>
            )}

            <button
              onClick={() => window.location.reload()}
              className="bg-gray-600 hover:bg-gray-700 px-6 py-2 rounded-lg font-medium transition-colors w-full"
            >
              Refresh Page
            </button>
          </div>

          {retryCountRef.current >= maxRetries && (
            <p className="text-sm text-red-400 mt-4">
              Maximum retry attempts reached. Please check your connection and try refreshing the
              page.
            </p>
          )}
        </div>
      </div>
    );
  }

  // No race data state
  if (!state.race) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-gray-500 text-6xl mb-4">🏁</div>
          <h2 className="text-2xl font-bold mb-2">Race Not Found</h2>
          <p className="text-gray-300 mb-4">
            Unable to load race data. The race may not exist or may have been removed.
          </p>
          <button
            onClick={handleRetry}
            className="bg-blue-600 hover:bg-blue-700 px-6 py-2 rounded-lg font-medium transition-colors"
          >
            Try Again
          </button>
        </div>
      </div>
    );
  }

  // Player not in race state with helpful information
  if (!state.playerParticipant) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-yellow-500 text-6xl mb-4">👤</div>
          <h2 className="text-2xl font-bold mb-2">Not Participating</h2>
          <p className="text-gray-300 mb-4">
            You are not registered as a participant in this race.
          </p>
          <div className="text-sm text-gray-400 space-y-1">
            <p>Race: {state.race.name}</p>
            <p>Status: {state.race.status}</p>
            <p>Participants: {state.race.participants.length}</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Main game interface container */}
      <div className="container mx-auto px-4 py-6">
        {/* Enhanced Race Status Panel - Top section */}
        <div className="mb-6">
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-xl font-bold">Race Status</h2>
              <div className="flex items-center space-x-2">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{
                    backgroundColor: raceStatusUtils.getTurnPhaseColor(state.currentTurnPhase),
                  }}
                ></div>
                <span className="text-sm font-medium">
                  {raceStatusUtils.getStatusMessage(
                    state.race,
                    state.currentTurnPhase,
                    state.hasSubmittedAction,
                  )}
                </span>
              </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-5 gap-4 text-sm">
              <div>
                <span className="text-gray-400">Race:</span>
                <p className="font-medium truncate">{state.race.name}</p>
              </div>
              <div>
                <span className="text-gray-400">Lap:</span>
                <p className="font-medium">
                  {state.race.current_lap} / {state.race.total_laps}
                  {raceStatusUtils.isFinalLap(state.race) && (
                    <span className="text-yellow-400 ml-1">🏁</span>
                  )}
                </p>
              </div>
              <div>
                <span className="text-gray-400">Characteristic:</span>
                <p className="font-medium">
                  {raceStatusUtils.getLapCharacteristicIcon(state.race.lap_characteristic)}{' '}
                  {state.race.lap_characteristic}
                </p>
              </div>
              <div>
                <span className="text-gray-400">Race Status:</span>
                <p
                  className={`font-medium ${
                    state.race.status === 'InProgress'
                      ? 'text-green-400'
                      : state.race.status === 'Finished'
                        ? 'text-blue-400'
                        : state.race.status === 'Waiting'
                          ? 'text-yellow-400'
                          : 'text-red-400'
                  }`}
                >
                  {state.race.status}
                </p>
              </div>
              <div>
                <span className="text-gray-400">Progress:</span>
                <div className="flex items-center space-x-2">
                  <div className="flex-1 bg-gray-700 rounded-full h-2">
                    <div
                      className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                      style={{ width: `${raceStatusUtils.getLapProgress(state.race)}%` }}
                    ></div>
                  </div>
                  <span className="text-xs font-medium">
                    {Math.round(raceStatusUtils.getLapProgress(state.race))}%
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Main game layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Enhanced Left column - Local Sector Display with improved calculations */}
          <div className="lg:col-span-2">
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-bold">Local Sector View</h2>
                <div className="text-sm text-gray-400">
                  Showing sectors {Math.max(0, state.localView.centerSector - 2)} -{' '}
                  {Math.min(state.race.track.sectors.length - 1, state.localView.centerSector + 2)}
                </div>
              </div>

              <div className="space-y-3">
                {state.localView.visibleSectors.map((sector) => {
                  const isPlayerSector = sector.id === state.playerParticipant?.current_sector;
                  const participantsInSector = state.localView.visibleParticipants
                    .filter((p) => p.current_sector === sector.id)
                    .sort((a, b) => a.current_position_in_sector - b.current_position_in_sector);

                  // Calculate position relative to player sector for visual emphasis
                  const relativePosition = sector.id - state.localView.centerSector;
                  const positionClass =
                    relativePosition === 0 ? 'center' : relativePosition < 0 ? 'above' : 'below';

                  return (
                    <div
                      key={sector.id}
                      className={`p-4 rounded-lg border transition-all duration-300 ${
                        isPlayerSector
                          ? 'bg-blue-900 border-blue-500 shadow-lg shadow-blue-500/20'
                          : positionClass === 'above' || positionClass === 'below'
                            ? 'bg-gray-700 border-gray-600 opacity-90'
                            : 'bg-gray-700 border-gray-600'
                      }`}
                    >
                      <div className="flex justify-between items-center mb-3">
                        <div className="flex items-center space-x-2">
                          <h3 className="font-medium text-lg">
                            Sector {sector.id}: {sector.name}
                          </h3>
                          {isPlayerSector && (
                            <span className="bg-blue-600 text-white text-xs px-2 py-1 rounded-full font-medium">
                              YOUR SECTOR
                            </span>
                          )}
                        </div>
                        <div className="flex items-center space-x-2">
                          <span
                            className={`text-sm px-2 py-1 rounded ${
                              sector.sector_type === 'Start'
                                ? 'bg-green-600 text-white'
                                : sector.sector_type === 'Finish'
                                  ? 'bg-purple-600 text-white'
                                  : sector.sector_type === 'Straight'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-orange-600 text-white'
                            }`}
                          >
                            {sector.sector_type}
                          </span>
                        </div>
                      </div>

                      <div className="grid grid-cols-2 gap-4 text-sm text-gray-300 mb-3">
                        <div>
                          <span className="text-gray-400">Value Range:</span>
                          <p className="font-medium">
                            {sector.min_value} - {sector.max_value}
                          </p>
                        </div>
                        {sector.slot_capacity && (
                          <div>
                            <span className="text-gray-400">Capacity:</span>
                            <p className="font-medium">
                              {participantsInSector.length} / {sector.slot_capacity}
                              {participantsInSector.length >= sector.slot_capacity && (
                                <span className="text-red-400 ml-1">FULL</span>
                              )}
                            </p>
                          </div>
                        )}
                      </div>

                      {participantsInSector.length > 0 ? (
                        <div>
                          <span className="text-gray-400 text-sm block mb-2">
                            Participants ({participantsInSector.length}):
                          </span>
                          <div className="grid grid-cols-1 gap-2">
                            {participantsInSector.map((participant, i) => (
                              <div
                                key={participant.player_uuid}
                                className={`flex items-center justify-between p-2 rounded text-sm ${
                                  participant.player_uuid === playerUuid
                                    ? 'bg-blue-800 border border-blue-600'
                                    : 'bg-gray-600'
                                }`}
                              >
                                <div className="flex items-center space-x-2">
                                  <span className="w-6 h-6 bg-gray-500 rounded-full flex items-center justify-center text-xs font-bold">
                                    {i + 1}
                                  </span>
                                  <span
                                    className={
                                      participant.player_uuid === playerUuid
                                        ? 'text-blue-200 font-medium'
                                        : 'text-gray-200'
                                    }
                                  >
                                    {participant.player_uuid === playerUuid
                                      ? 'You'
                                      : `Player ${participant.player_uuid.slice(-4)}`}
                                  </span>
                                </div>
                                <div className="text-right">
                                  <div className="font-medium">
                                    Value: {participant.total_value}
                                  </div>
                                  <div className="text-xs text-gray-400">
                                    Lap: {participant.current_lap}
                                  </div>
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      ) : (
                        <div className="text-center py-4 text-gray-400 text-sm">
                          No participants in this sector
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {/* Local view navigation hint */}
              <div className="mt-4 text-center text-xs text-gray-500">
                Showing your current sector ± 2 sectors ({state.localView.visibleSectors.length}{' '}
                total)
              </div>
            </div>
          </div>

          {/* Right column - Player info and controls */}
          <div className="space-y-6">
            {/* Player Car Card */}
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <h2 className="text-xl font-bold mb-4">Your Car</h2>
              <div className="space-y-2 text-sm">
                <div>
                  <span className="text-gray-400">Current Sector:</span>
                  <p className="font-medium">{state.playerParticipant.current_sector}</p>
                </div>
                <div>
                  <span className="text-gray-400">Position in Sector:</span>
                  <p className="font-medium">
                    {state.playerParticipant.current_position_in_sector}
                  </p>
                </div>
                <div>
                  <span className="text-gray-400">Current Lap:</span>
                  <p className="font-medium">{state.playerParticipant.current_lap}</p>
                </div>
                <div>
                  <span className="text-gray-400">Total Value:</span>
                  <p className="font-medium">{state.playerParticipant.total_value}</p>
                </div>
              </div>
            </div>

            {/* Enhanced Turn Controller with better state management */}
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-bold">Turn Actions</h2>
                <div className="text-sm text-gray-400">Phase: {state.currentTurnPhase}</div>
              </div>

              {raceStatusUtils.canSubmitActions(state.race) &&
              state.currentTurnPhase === 'WaitingForPlayers' &&
              !state.hasSubmittedAction ? (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-3">
                      Select Boost Value (0-5):
                    </label>
                    <div className="grid grid-cols-6 gap-2">
                      {[0, 1, 2, 3, 4, 5].map((boost) => (
                        <button
                          key={boost}
                          onClick={() => actions.selectBoost(boost)}
                          className={`aspect-square rounded-lg border font-bold text-lg transition-all duration-200 ${
                            state.selectedBoost === boost
                              ? 'bg-blue-600 border-blue-500 text-white shadow-lg shadow-blue-500/30 scale-105'
                              : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500'
                          }`}
                        >
                          {boost}
                        </button>
                      ))}
                    </div>
                    <div className="mt-2 text-xs text-gray-400">
                      Selected boost:{' '}
                      <span className="font-medium text-blue-400">{state.selectedBoost}</span>
                    </div>
                  </div>

                  {/* Performance preview */}
                  <div className="bg-gray-700 rounded-lg p-3">
                    <div className="text-sm font-medium mb-2">Performance Preview:</div>
                    <div className="text-xs text-gray-300 space-y-1">
                      <div>Base performance will be calculated from your car stats</div>
                      <div>
                        Sector ceiling:{' '}
                        {state.localView.visibleSectors.find(
                          (s) => s.id === state.playerParticipant?.current_sector,
                        )?.max_value || 'N/A'}
                      </div>
                      <div>Boost addition: +{state.selectedBoost}</div>
                    </div>
                  </div>

                  <button
                    onClick={actions.submitBoostAction}
                    disabled={state.isLoading}
                    className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2"
                  >
                    {state.isLoading ? (
                      <>
                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                        <span>Submitting...</span>
                      </>
                    ) : (
                      <>
                        <span>Submit Boost ({state.selectedBoost})</span>
                        <span>🚀</span>
                      </>
                    )}
                  </button>
                </div>
              ) : state.hasSubmittedAction ? (
                <div className="text-center py-6">
                  <div className="text-green-400 text-4xl mb-3">✓</div>
                  <p className="text-green-400 font-medium text-lg mb-2">
                    Action Submitted Successfully
                  </p>
                  <p className="text-gray-400 text-sm mb-3">
                    Boost value: <span className="font-medium">{state.selectedBoost}</span>
                  </p>
                  <div className="bg-gray-700 rounded-lg p-3">
                    <p className="text-gray-300 text-sm">
                      {raceStatusUtils.getStatusMessage(
                        state.race,
                        state.currentTurnPhase,
                        state.hasSubmittedAction,
                      )}
                    </p>
                  </div>
                </div>
              ) : state.race.status === 'Finished' ? (
                <div className="text-center py-6">
                  <div className="text-blue-400 text-4xl mb-3">🏁</div>
                  <p className="text-blue-400 font-medium text-lg mb-2">Race Finished</p>
                  {state.playerParticipant?.finish_position && (
                    <p className="text-gray-300">
                      Final Position:{' '}
                      <span className="font-bold text-yellow-400">
                        #{state.playerParticipant.finish_position}
                      </span>
                    </p>
                  )}
                </div>
              ) : state.race.status === 'Waiting' ? (
                <div className="text-center py-6">
                  <div className="text-yellow-400 text-4xl mb-3">⏳</div>
                  <p className="text-yellow-400 font-medium text-lg mb-2">Race Starting Soon</p>
                  <p className="text-gray-400 text-sm">Waiting for race to begin...</p>
                </div>
              ) : (
                <div className="text-center py-6">
                  <div className="text-gray-400 text-4xl mb-3">⏸️</div>
                  <p className="text-gray-400 font-medium">Turn actions not available</p>
                  <p className="text-gray-500 text-sm mt-2">
                    Current phase: {state.currentTurnPhase}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Enhanced error display with better UX */}
        {state.error && (
          <div className="fixed bottom-4 right-4 max-w-md bg-red-600 text-white rounded-lg shadow-lg border border-red-500 z-50">
            <div className="p-4">
              <div className="flex items-start space-x-3">
                <div className="text-red-200 text-xl">⚠️</div>
                <div className="flex-1">
                  <h4 className="font-medium mb-1">Error</h4>
                  <p className="text-sm text-red-100">
                    {raceErrorUtils.getUserFriendlyError(state.error)}
                  </p>
                  {raceErrorUtils.isRetryableError(state.error) && (
                    <button
                      onClick={handleRetry}
                      className="mt-2 text-xs bg-red-700 hover:bg-red-800 px-2 py-1 rounded transition-colors"
                    >
                      Retry
                    </button>
                  )}
                </div>
                <button
                  onClick={actions.clearError}
                  className="text-red-200 hover:text-white transition-colors"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Connection status indicator */}
        {state.isLoading && state.race && (
          <div className="fixed bottom-4 left-4 bg-blue-600 text-white px-3 py-2 rounded-lg shadow-lg flex items-center space-x-2">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
            <span className="text-sm">Updating...</span>
          </div>
        )}

        {/* Animation overlay for turn processing */}
        {state.animationState.isAnimating && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40">
            <div className="bg-gray-800 rounded-lg p-6 text-center border border-gray-700">
              <div className="text-blue-400 text-4xl mb-3">🏎️</div>
              <h3 className="text-xl font-bold text-white mb-2">Processing Turn</h3>
              <p className="text-gray-300 text-sm">Calculating race results...</p>
              <div className="mt-4 w-32 bg-gray-700 rounded-full h-2 mx-auto">
                <div className="bg-blue-500 h-2 rounded-full animate-pulse"></div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PlayerGameInterface;
