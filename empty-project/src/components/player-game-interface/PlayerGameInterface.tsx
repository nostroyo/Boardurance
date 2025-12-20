import React, { useEffect, useCallback, useRef, useState } from 'react';
import type { PlayerGameInterfaceProps } from '../../types';
import { usePlayerGameContext } from '../../contexts/PlayerGameContext';
import { raceAPI, raceStatusUtils, raceErrorUtils } from '../../utils/raceAPI';
import type { LocalView, BoostAvailability, TurnPhaseStatus } from '../../types/race-api';

// Redesigned component imports
import { TrackDisplayRedesign } from './TrackDisplayRedesign';
import { BoostControlPanel } from './BoostControlPanel';

// Existing component imports
import { RaceStatusPanel } from './RaceStatusPanel';

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

  // State for redesigned interface data
  const [localView, setLocalView] = useState<LocalView | null>(null);
  const [boostAvailability, setBoostAvailability] = useState<BoostAvailability | null>(null);
  const [currentTurnPhase, setCurrentTurnPhase] = useState<TurnPhaseStatus>('WaitingForPlayers');

  // Fetch local view data from API with fallback to mock data
  const fetchLocalView = useCallback(async () => {
    try {
      const response = await raceAPI.getLocalView(raceUuid, playerUuid);
      if (response.success && response.data) {
        setLocalView(response.data as LocalView);
        console.log('[PlayerGameInterface] Local view loaded successfully:', response.data);
      } else {
        console.warn('[PlayerGameInterface] Local view API failed, using mock data:', response.error);
        // Fallback to mock data so user can see the interface
        const mockLocalView: LocalView = {
          center_sector: state.playerParticipant?.current_sector || 1,
          visible_sectors: [
            { id: 0, name: 'Start/Finish', min_value: 10, max_value: 20, slot_capacity: 5, sector_type: 'Start', current_occupancy: 2 },
            { id: 1, name: 'Sector 1', min_value: 15, max_value: 25, slot_capacity: 5, sector_type: 'Straight', current_occupancy: 3 },
            { id: 2, name: 'Sector 2', min_value: 12, max_value: 22, slot_capacity: 5, sector_type: 'Curve', current_occupancy: 1 },
            { id: 3, name: 'Sector 3', min_value: 18, max_value: 28, slot_capacity: 5, sector_type: 'Straight', current_occupancy: 2 },
            { id: 4, name: 'Sector 4', min_value: 14, max_value: 24, slot_capacity: 5, sector_type: 'Curve', current_occupancy: 4 },
          ],
          visible_participants: state.race?.participants.map((p, index) => ({
            player_uuid: p.player_uuid,
            player_name: `Player ${index + 1}`,
            car_name: `Car ${index + 1}`,
            current_sector: p.current_sector,
            position_in_sector: p.current_position_in_sector,
            total_value: p.total_value,
            current_lap: p.current_lap,
            is_finished: p.is_finished,
          })) || []
        };
        setLocalView(mockLocalView);
      }
    } catch (error) {
      console.error('[PlayerGameInterface] Failed to fetch local view:', error);
      // Still provide mock data on error
      const mockLocalView: LocalView = {
        center_sector: state.playerParticipant?.current_sector || 1,
        visible_sectors: [
          { id: 0, name: 'Start/Finish', min_value: 10, max_value: 20, slot_capacity: 5, sector_type: 'Start', current_occupancy: 2 },
          { id: 1, name: 'Sector 1', min_value: 15, max_value: 25, slot_capacity: 5, sector_type: 'Straight', current_occupancy: 3 },
          { id: 2, name: 'Sector 2', min_value: 12, max_value: 22, slot_capacity: 5, sector_type: 'Curve', current_occupancy: 1 },
        ],
        visible_participants: []
      };
      setLocalView(mockLocalView);
    }
  }, [raceUuid, playerUuid, state.playerParticipant, state.race?.participants]);

  // Fetch boost availability from API with fallback to mock data
  const fetchBoostAvailability = useCallback(async () => {
    try {
      const response = await raceAPI.getBoostAvailability(raceUuid, playerUuid);
      if (response.success && response.data) {
        setBoostAvailability(response.data as BoostAvailability);
        console.log('[PlayerGameInterface] Boost availability loaded successfully:', response.data);
      } else {
        console.warn('[PlayerGameInterface] Boost availability API failed, using mock data:', response.error);
        // Fallback to mock data so user can see the interface
        const mockBoostAvailability: BoostAvailability = {
          available_cards: [0, 1, 2, 3, 4, 5],
          hand_state: { '0': true, '1': true, '2': true, '3': true, '4': true, '5': true },
          current_cycle: 1,
          cycles_completed: 0,
          cards_remaining: 6,
          next_replenishment_at: null,
        };
        setBoostAvailability(mockBoostAvailability);
      }
    } catch (error) {
      console.error('[PlayerGameInterface] Failed to fetch boost availability:', error);
      // Still provide mock data on error
      const mockBoostAvailability: BoostAvailability = {
        available_cards: [0, 1, 2, 3, 4, 5],
        hand_state: { '0': true, '1': true, '2': true, '3': true, '4': true, '5': true },
        current_cycle: 1,
        cycles_completed: 0,
        cards_remaining: 6,
        next_replenishment_at: null,
      };
      setBoostAvailability(mockBoostAvailability);
    }
  }, [raceUuid, playerUuid]);

  // Enhanced race initialization with error recovery
  const initializeRaceWithRetry = useCallback(async () => {
    try {
      await actions.initializeRace(raceUuid, playerUuid);
      // Fetch additional data for redesigned interface
      await Promise.all([
        fetchLocalView(),
        fetchBoostAvailability()
      ]);
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
  }, [raceUuid, playerUuid, actions, fetchLocalView, fetchBoostAvailability]);

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
        
        // Update redesigned interface data
        await Promise.all([
          fetchLocalView(),
          fetchBoostAvailability()
        ]);

        // Check turn phase if race is in progress
        if (state.race?.status === 'InProgress') {
          const turnPhaseResponse = await raceAPI.getTurnPhase(raceUuid);
          if (turnPhaseResponse.success && turnPhaseResponse.data) {
            const newTurnPhase = (turnPhaseResponse.data as any).turn_phase;
            setCurrentTurnPhase(newTurnPhase);
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
  }, [state.race, raceUuid, actions, currentTurnPhase, state.hasSubmittedAction, fetchLocalView, fetchBoostAvailability]);

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
      {/* Main game interface container - Mobile responsive */}
      <div className="container mx-auto px-2 sm:px-4 py-3 sm:py-6 max-w-7xl">
        {/* Enhanced Race Status Panel - Top section - Mobile responsive */}
        <div className="mb-4 sm:mb-6">
          <RaceStatusPanel
            currentLap={state.race.current_lap}
            totalLaps={state.race.total_laps}
            lapCharacteristic={state.race.lap_characteristic}
            turnPhase={{
              turn_phase: state.currentTurnPhase,
              current_lap: state.race.current_lap,
              lap_characteristic: state.race.lap_characteristic,
              submitted_players: [], // TODO: Get from API
              pending_players: [], // TODO: Get from API
              total_active_players: state.race.participants.length
            }}
            raceStatus={
              state.race.status === 'Finished' 
                ? 'Completed' 
                : state.race.status === 'InProgress' 
                  ? 'InProgress' 
                  : 'NotStarted'
            }
            hasSubmittedAction={state.hasSubmittedAction}
          />
        </div>

        {/* Main game layout - Mobile-first responsive stacking */}
        <div className="flex flex-col lg:flex-row gap-4 sm:gap-6">
          {/* Track Display Section - Full width on mobile, 3/4 on desktop */}
          <div className="w-full lg:w-3/4 order-2 lg:order-1">
            {localView ? (
              <TrackDisplayRedesign
                localView={localView}
                playerUuid={playerUuid}
                animationState={state.animationState}
                onSectorClick={(sectorId) => {
                  console.log('Sector clicked:', sectorId);
                  // Future enhancement: show sector details
                }}
                onSlotClick={(sectorId, slotNumber) => {
                  console.log('Slot clicked:', sectorId, slotNumber);
                  // Future enhancement: show participant details
                }}
              />
            ) : (
              <div className="bg-gray-800 rounded-lg p-6 sm:p-8 text-center">
                <div className="animate-spin rounded-full h-8 w-8 sm:h-12 sm:w-12 border-b-2 border-blue-500 mx-auto mb-3 sm:mb-4"></div>
                <p className="text-gray-400 text-sm sm:text-base">Loading track view...</p>
              </div>
            )}
          </div>

          {/* Controls and Player Info Section - Full width on mobile, 1/4 on desktop */}
          <div className="w-full lg:w-1/4 order-1 lg:order-2 space-y-3 sm:space-y-4 lg:space-y-6">
            {/* Player Car Card - Mobile responsive */}
            <div className="bg-gray-800 rounded-lg p-3 sm:p-4 border border-gray-700">
              <h2 className="text-lg sm:text-xl font-bold mb-3 sm:mb-4">Your Car</h2>
              <div className="grid grid-cols-2 sm:grid-cols-1 gap-2 sm:gap-3 text-sm">
                <div className="bg-gray-700/50 p-2 rounded">
                  <span className="text-gray-400 block text-xs">Current Sector:</span>
                  <p className="font-medium text-sm sm:text-base">{state.playerParticipant?.current_sector}</p>
                </div>
                <div className="bg-gray-700/50 p-2 rounded">
                  <span className="text-gray-400 block text-xs">Position:</span>
                  <p className="font-medium text-sm sm:text-base">
                    {state.playerParticipant?.current_position_in_sector}
                  </p>
                </div>
                <div className="bg-gray-700/50 p-2 rounded">
                  <span className="text-gray-400 block text-xs">Current Lap:</span>
                  <p className="font-medium text-sm sm:text-base">{state.playerParticipant?.current_lap}</p>
                </div>
                <div className="bg-gray-700/50 p-2 rounded">
                  <span className="text-gray-400 block text-xs">Total Value:</span>
                  <p className="font-medium text-sm sm:text-base">{state.playerParticipant?.total_value}</p>
                </div>
              </div>
            </div>

            {/* Redesigned Boost Control Panel - Mobile responsive */}
            {raceStatusUtils.canSubmitActions(state.race) &&
            currentTurnPhase === 'WaitingForPlayers' ? (
              <BoostControlPanel
                selectedBoost={state.selectedBoost}
                availableBoosts={boostAvailability?.available_cards || [0, 1, 2, 3, 4, 5]}
                onBoostSelect={actions.selectBoost}
                onValidateTurn={actions.submitBoostAction}
                isSubmitting={state.isLoading}
                hasSubmitted={state.hasSubmittedAction}
                turnPhase={currentTurnPhase}
              />
            ) : state.hasSubmittedAction ? (
              <div className="bg-gray-800 rounded-lg p-3 sm:p-4 border border-gray-700">
                <div className="text-center py-4 sm:py-6">
                  <div className="text-green-400 text-3xl sm:text-4xl mb-2 sm:mb-3">✓</div>
                  <p className="text-green-400 font-medium text-base sm:text-lg mb-1 sm:mb-2">
                    Action Submitted Successfully
                  </p>
                  <p className="text-gray-400 text-sm mb-2 sm:mb-3">
                    Boost value: <span className="font-medium">{state.selectedBoost}</span>
                  </p>
                  <div className="bg-gray-700 rounded-lg p-2 sm:p-3">
                    <p className="text-gray-300 text-xs sm:text-sm">
                      {raceStatusUtils.getStatusMessage(
                        state.race,
                        currentTurnPhase,
                        state.hasSubmittedAction,
                      )}
                    </p>
                  </div>
                </div>
              </div>
            ) : state.race?.status === 'Finished' ? (
              <div className="bg-gray-800 rounded-lg p-3 sm:p-4 border border-gray-700">
                <div className="text-center py-4 sm:py-6">
                  <div className="text-blue-400 text-3xl sm:text-4xl mb-2 sm:mb-3">🏁</div>
                  <p className="text-blue-400 font-medium text-base sm:text-lg mb-1 sm:mb-2">Race Finished</p>
                  {state.playerParticipant?.finish_position && (
                    <p className="text-gray-300 text-sm sm:text-base">
                      Final Position:{' '}
                      <span className="font-bold text-yellow-400">
                        #{state.playerParticipant.finish_position}
                      </span>
                    </p>
                  )}
                </div>
              </div>
            ) : state.race?.status === 'Waiting' ? (
              <div className="bg-gray-800 rounded-lg p-3 sm:p-4 border border-gray-700">
                <div className="text-center py-4 sm:py-6">
                  <div className="text-yellow-400 text-3xl sm:text-4xl mb-2 sm:mb-3">⏳</div>
                  <p className="text-yellow-400 font-medium text-base sm:text-lg mb-1 sm:mb-2">Race Starting Soon</p>
                  <p className="text-gray-400 text-xs sm:text-sm">Waiting for race to begin...</p>
                </div>
              </div>
            ) : (
              <div className="bg-gray-800 rounded-lg p-3 sm:p-4 border border-gray-700">
                <div className="text-center py-4 sm:py-6">
                  <div className="text-gray-400 text-3xl sm:text-4xl mb-2 sm:mb-3">⏸️</div>
                  <p className="text-gray-400 font-medium text-sm sm:text-base">Turn actions not available</p>
                  <p className="text-gray-500 text-xs sm:text-sm mt-1 sm:mt-2">
                    Current phase: {currentTurnPhase}
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Enhanced error display with better UX - Mobile responsive */}
        {state.error && (
          <div className="fixed bottom-2 sm:bottom-4 right-2 sm:right-4 left-2 sm:left-auto max-w-full sm:max-w-md bg-red-600 text-white rounded-lg shadow-lg border border-red-500 z-50">
            <div className="p-3 sm:p-4">
              <div className="flex items-start space-x-2 sm:space-x-3">
                <div className="text-red-200 text-lg sm:text-xl flex-shrink-0">⚠️</div>
                <div className="flex-1 min-w-0">
                  <h4 className="font-medium mb-1 text-sm sm:text-base">Error</h4>
                  <p className="text-xs sm:text-sm text-red-100 break-words">
                    {raceErrorUtils.getUserFriendlyError(state.error)}
                  </p>
                  {raceErrorUtils.isRetryableError(state.error) && (
                    <button
                      onClick={handleRetry}
                      className="mt-2 text-xs bg-red-700 hover:bg-red-800 px-2 py-1 rounded transition-colors touch-manipulation"
                    >
                      Retry
                    </button>
                  )}
                </div>
                <button
                  onClick={actions.clearError}
                  className="text-red-200 hover:text-white transition-colors flex-shrink-0 touch-manipulation p-1"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Debug Info Panel - Show API status and data */}
        {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
          <div className="fixed bottom-20 right-4 bg-gray-800 rounded-lg p-3 text-xs max-w-sm border border-gray-600 z-30">
            <h4 className="font-bold mb-2 text-yellow-400">Debug Info</h4>
            <div className="space-y-1 text-gray-300">
              <div>Race Status: {state.race?.status}</div>
              <div>Turn Phase: {currentTurnPhase}</div>
              <div>Player Sector: {state.playerParticipant?.current_sector}</div>
              <div>Local View: {localView ? '✓ Loaded' : '✗ Missing'}</div>
              <div>Boost Availability: {boostAvailability ? '✓ Loaded' : '✗ Missing'}</div>
              <div>Can Submit: {raceStatusUtils.canSubmitActions(state.race) ? 'Yes' : 'No'}</div>
              <div>Has Submitted: {state.hasSubmittedAction ? 'Yes' : 'No'}</div>
              {localView && (
                <div>Visible Sectors: {localView.visible_sectors.length}</div>
              )}
              {boostAvailability && (
                <div>Available Boosts: [{boostAvailability.available_cards.join(', ')}]</div>
              )}
            </div>
          </div>
        )}

        {/* Connection status indicator - Mobile responsive */}
        {state.isLoading && state.race && (
          <div className="fixed bottom-2 sm:bottom-4 left-2 sm:left-4 bg-blue-600 text-white px-2 sm:px-3 py-1 sm:py-2 rounded-lg shadow-lg flex items-center space-x-1 sm:space-x-2 z-40">
            <div className="animate-spin rounded-full h-3 w-3 sm:h-4 sm:w-4 border-b-2 border-white flex-shrink-0"></div>
            <span className="text-xs sm:text-sm">Updating...</span>
          </div>
        )}

        {/* Animation overlay for turn processing - Mobile responsive */}
        {state.animationState.isAnimating && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40 p-4">
            <div className="bg-gray-800 rounded-lg p-4 sm:p-6 text-center border border-gray-700 max-w-sm w-full">
              <div className="text-blue-400 text-3xl sm:text-4xl mb-2 sm:mb-3">🏎️</div>
              <h3 className="text-lg sm:text-xl font-bold text-white mb-1 sm:mb-2">Processing Turn</h3>
              <p className="text-gray-300 text-xs sm:text-sm mb-3 sm:mb-4">Calculating race results...</p>
              <div className="w-24 sm:w-32 bg-gray-700 rounded-full h-1.5 sm:h-2 mx-auto">
                <div className="bg-blue-500 h-1.5 sm:h-2 rounded-full animate-pulse"></div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PlayerGameInterface;
