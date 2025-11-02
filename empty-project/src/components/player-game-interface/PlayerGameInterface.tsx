import React, { useEffect } from 'react';
import type { PlayerGameInterfaceProps } from '../../types';
import { usePlayerGameContext } from '../../contexts/PlayerGameContext';

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
  onError
}) => {
  const { state, actions } = usePlayerGameContext();

  // Initialize race on component mount
  useEffect(() => {
    actions.initializeRace(raceUuid, playerUuid);
  }, [raceUuid, playerUuid, actions]);

  // Handle race completion
  useEffect(() => {
    if (state.race?.status === 'Finished' && state.playerParticipant?.finish_position && onRaceComplete) {
      onRaceComplete(state.playerParticipant.finish_position);
    }
  }, [state.race?.status, state.playerParticipant?.finish_position, onRaceComplete]);

  // Handle errors
  useEffect(() => {
    if (state.error && onError) {
      onError(state.error);
    }
  }, [state.error, onError]);

  // Loading state
  if (state.isLoading && !state.race) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-lg">Loading race data...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (state.error && !state.race) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <div className="text-red-500 text-6xl mb-4">⚠️</div>
          <h2 className="text-2xl font-bold mb-2">Error Loading Race</h2>
          <p className="text-gray-300 mb-4">{state.error}</p>
          <button
            onClick={() => actions.initializeRace(raceUuid, playerUuid)}
            className="bg-blue-600 hover:bg-blue-700 px-6 py-2 rounded-lg font-medium transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  // No race data
  if (!state.race) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-2">Race Not Found</h2>
          <p className="text-gray-300">Unable to load race data.</p>
        </div>
      </div>
    );
  }

  // Player not in race
  if (!state.playerParticipant) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-gray-900 text-white">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-2">Not Participating</h2>
          <p className="text-gray-300">You are not participating in this race.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* Main game interface container */}
      <div className="container mx-auto px-4 py-6">
        
        {/* Race Status Panel - Top section */}
        <div className="mb-6">
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h2 className="text-xl font-bold mb-2">Race Status Panel</h2>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <span className="text-gray-400">Race:</span>
                <p className="font-medium">{state.race.name}</p>
              </div>
              <div>
                <span className="text-gray-400">Lap:</span>
                <p className="font-medium">{state.race.current_lap} / {state.race.total_laps}</p>
              </div>
              <div>
                <span className="text-gray-400">Characteristic:</span>
                <p className="font-medium">{state.race.lap_characteristic}</p>
              </div>
              <div>
                <span className="text-gray-400">Status:</span>
                <p className="font-medium">{state.race.status}</p>
              </div>
            </div>
          </div>
        </div>

        {/* Main game layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          
          {/* Left column - Local Sector Display */}
          <div className="lg:col-span-2">
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <h2 className="text-xl font-bold mb-4">Local Sector View</h2>
              <div className="space-y-2">
                {state.localView.visibleSectors.map((sector) => {
                  const isPlayerSector = sector.id === state.playerParticipant?.current_sector;
                  const participantsInSector = state.localView.visibleParticipants.filter(
                    p => p.current_sector === sector.id
                  );
                  
                  return (
                    <div
                      key={sector.id}
                      className={`p-3 rounded border ${
                        isPlayerSector 
                          ? 'bg-blue-900 border-blue-500' 
                          : 'bg-gray-700 border-gray-600'
                      }`}
                    >
                      <div className="flex justify-between items-center mb-2">
                        <h3 className="font-medium">
                          Sector {sector.id}: {sector.name}
                        </h3>
                        <span className="text-sm text-gray-400">
                          {sector.sector_type}
                        </span>
                      </div>
                      <div className="text-sm text-gray-300 mb-2">
                        Range: {sector.min_value} - {sector.max_value}
                        {sector.slot_capacity && ` | Capacity: ${sector.slot_capacity}`}
                      </div>
                      {participantsInSector.length > 0 && (
                        <div className="text-sm">
                          <span className="text-gray-400">Participants: </span>
                          {participantsInSector.map((p, i) => (
                            <span key={p.player_uuid} className={
                              p.player_uuid === playerUuid ? 'text-blue-400 font-medium' : 'text-gray-300'
                            }>
                              {p.player_uuid === playerUuid ? 'You' : `Player ${i + 1}`}
                              {i < participantsInSector.length - 1 && ', '}
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                  );
                })}
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
                  <p className="font-medium">{state.playerParticipant.current_position_in_sector}</p>
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

            {/* Turn Controller */}
            <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
              <h2 className="text-xl font-bold mb-4">Boost Selection</h2>
              
              {state.race.status === 'InProgress' && !state.hasSubmittedAction ? (
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">
                      Select Boost (0-5):
                    </label>
                    <div className="flex space-x-2">
                      {[0, 1, 2, 3, 4, 5].map(boost => (
                        <button
                          key={boost}
                          onClick={() => actions.selectBoost(boost)}
                          className={`w-10 h-10 rounded border font-medium transition-colors ${
                            state.selectedBoost === boost
                              ? 'bg-blue-600 border-blue-500 text-white'
                              : 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600'
                          }`}
                        >
                          {boost}
                        </button>
                      ))}
                    </div>
                  </div>
                  
                  <button
                    onClick={actions.submitBoostAction}
                    disabled={state.isLoading}
                    className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-4 py-2 rounded font-medium transition-colors"
                  >
                    {state.isLoading ? 'Submitting...' : 'Submit Boost'}
                  </button>
                </div>
              ) : state.hasSubmittedAction ? (
                <div className="text-center py-4">
                  <div className="text-green-400 text-2xl mb-2">✓</div>
                  <p className="text-green-400 font-medium">Action Submitted</p>
                  <p className="text-gray-400 text-sm">Waiting for other players...</p>
                </div>
              ) : (
                <div className="text-center py-4">
                  <p className="text-gray-400">Race not active</p>
                </div>
              )}
            </div>

          </div>
        </div>

        {/* Error display */}
        {state.error && (
          <div className="fixed bottom-4 right-4 bg-red-600 text-white px-4 py-2 rounded-lg shadow-lg">
            <div className="flex items-center space-x-2">
              <span>⚠️</span>
              <span>{state.error}</span>
              <button
                onClick={actions.clearError}
                className="ml-2 text-red-200 hover:text-white"
              >
                ✕
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PlayerGameInterface;