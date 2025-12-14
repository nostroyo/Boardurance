/**
 * RaceCompletionScreen - Lazy-loaded component for race completion
 *
 * This component is lazy-loaded to improve performance and only loads
 * when the race is actually completed.
 *
 * Requirements: 12.5
 */

import React, { Suspense, lazy } from 'react';
import type { CarData, LapHistory } from '../../types/race-api';
import { RaceLoadingState } from './RaceLoadingState';

// Lazy load the lap history panel
const LapHistoryPanel = lazy(() => import('./LapHistoryPanel'));

export interface RaceCompletionScreenProps {
  finalPosition: number | null;
  carData: CarData | null;
  lapHistory: LapHistory | null;
  onReturnToLobby?: () => void;
  onViewDetails?: () => void;
}

export const RaceCompletionScreen: React.FC<RaceCompletionScreenProps> = React.memo(
  ({ finalPosition, carData, lapHistory, onReturnToLobby, onViewDetails }) => {
    // Get position medal/color
    const getPositionStyle = (position: number | null) => {
      if (position === null) return { color: 'text-gray-400', medal: '🏁' };
      if (position === 1) return { color: 'text-yellow-400', medal: '🥇' };
      if (position === 2) return { color: 'text-gray-300', medal: '🥈' };
      if (position === 3) return { color: 'text-orange-400', medal: '🥉' };
      return { color: 'text-blue-400', medal: '🏁' };
    };

    const positionStyle = getPositionStyle(finalPosition);

    // Calculate race statistics
    const raceStats = React.useMemo(() => {
      if (!lapHistory || lapHistory.laps.length === 0) {
        return null;
      }

      const totalLaps = lapHistory.laps.length;
      const avgBoost = lapHistory.laps.reduce((sum, lap) => sum + lap.boost_used, 0) / totalLaps;
      const avgPerformance =
        lapHistory.laps.reduce((sum, lap) => sum + lap.final_value, 0) / totalLaps;
      const bestLap = lapHistory.laps.reduce((best, lap) =>
        lap.final_value > best.final_value ? lap : best,
      );
      const cyclesCompleted = lapHistory.cycle_summaries.length;

      return {
        totalLaps,
        avgBoost,
        avgPerformance,
        bestLap,
        cyclesCompleted,
      };
    }, [lapHistory]);

    return (
      <div className="min-h-screen bg-gray-900 text-white p-4">
        <div className="max-w-6xl mx-auto space-y-6">
          {/* Main Completion Banner */}
          <div className="bg-gradient-to-br from-green-900 to-blue-900 rounded-lg p-8 text-center">
            <div className="text-6xl mb-4">{positionStyle.medal}</div>
            <h1 className="text-5xl font-bold mb-4">Race Complete!</h1>

            {finalPosition !== null && (
              <p className="text-3xl mb-6">
                Final Position:{' '}
                <span className={`font-bold ${positionStyle.color}`}>#{finalPosition}</span>
              </p>
            )}

            {carData && (
              <div className="bg-gray-800/50 rounded-lg p-4 mb-6 inline-block">
                <h2 className="text-2xl font-semibold mb-2">🏎️ {carData.car.name}</h2>
                <p className="text-lg text-gray-300">Pilot: {carData.pilot.name}</p>
                <p className="text-md text-gray-400">
                  {carData.pilot.pilot_class} • {carData.pilot.rarity}
                </p>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex gap-4 justify-center flex-wrap">
              <button
                onClick={() => {
                  if (onReturnToLobby) {
                    onReturnToLobby();
                  } else {
                    // Fallback to direct navigation if no callback provided
                    window.location.href = '/game';
                  }
                }}
                className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-3 rounded-lg font-semibold text-lg transition-colors"
              >
                Return to Lobby
              </button>

              <button
                onClick={() => {
                  if (onViewDetails) {
                    onViewDetails();
                  } else {
                    // Scroll to details section
                    document.getElementById('race-details')?.scrollIntoView({ behavior: 'smooth' });
                  }
                }}
                className="bg-gray-600 hover:bg-gray-700 text-white px-8 py-3 rounded-lg font-semibold text-lg transition-colors"
              >
                View Details
              </button>

              <button
                onClick={() => window.location.reload()}
                className="bg-green-600 hover:bg-green-700 text-white px-8 py-3 rounded-lg font-semibold text-lg transition-colors"
              >
                Race Again
              </button>
            </div>
          </div>

          {/* Race Statistics Summary */}
          {raceStats && (
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
              <div className="bg-gray-800 rounded-lg p-4 text-center">
                <div className="text-2xl font-bold text-white">{raceStats.totalLaps}</div>
                <div className="text-sm text-gray-400">Total Laps</div>
              </div>

              <div className="bg-gray-800 rounded-lg p-4 text-center">
                <div className="text-2xl font-bold text-blue-400">
                  {raceStats.avgPerformance.toFixed(1)}
                </div>
                <div className="text-sm text-gray-400">Avg Performance</div>
              </div>

              <div className="bg-gray-800 rounded-lg p-4 text-center">
                <div className="text-2xl font-bold text-green-400">
                  {raceStats.avgBoost.toFixed(1)}
                </div>
                <div className="text-sm text-gray-400">Avg Boost</div>
              </div>

              <div className="bg-gray-800 rounded-lg p-4 text-center">
                <div className="text-2xl font-bold text-yellow-400">
                  {raceStats.bestLap.final_value}
                </div>
                <div className="text-sm text-gray-400">Best Lap</div>
              </div>

              <div className="bg-gray-800 rounded-lg p-4 text-center">
                <div className="text-2xl font-bold text-purple-400">
                  {raceStats.cyclesCompleted}
                </div>
                <div className="text-sm text-gray-400">Cycles</div>
              </div>
            </div>
          )}

          {/* Detailed Race Analysis - Lazy Loaded */}
          <div id="race-details">
            {lapHistory && lapHistory.laps.length > 0 ? (
              <Suspense
                fallback={
                  <div className="bg-gray-800 rounded-lg p-6">
                    <RaceLoadingState
                      type="data"
                      message="Loading detailed race analysis..."
                      showSkeleton={true}
                    />
                  </div>
                }
              >
                <LapHistoryPanel
                  lapHistory={lapHistory}
                  playerName={carData?.pilot.name || 'Player'}
                />
              </Suspense>
            ) : (
              <div className="bg-gray-800 rounded-lg p-6 text-center">
                <div className="text-4xl mb-4 text-gray-600">📊</div>
                <h3 className="text-xl font-semibold text-gray-400 mb-2">
                  No Detailed History Available
                </h3>
                <p className="text-gray-500">
                  Race data could not be loaded for detailed analysis.
                </p>
              </div>
            )}
          </div>

          {/* Share Results Section */}
          <div className="bg-gray-800 rounded-lg p-6 text-center">
            <h3 className="text-xl font-semibold text-white mb-4">Share Your Results</h3>
            <div className="flex gap-4 justify-center flex-wrap">
              <button
                onClick={() => {
                  const text = `Just finished a race in position #${finalPosition}! 🏁`;
                  if (navigator.share) {
                    navigator.share({ text });
                  } else {
                    navigator.clipboard.writeText(text);
                    alert('Results copied to clipboard!');
                  }
                }}
                className="bg-gray-700 hover:bg-gray-600 text-white px-6 py-2 rounded-lg transition-colors"
              >
                📱 Share Results
              </button>

              <button
                onClick={() => {
                  const raceData = {
                    position: finalPosition,
                    car: carData?.car.name,
                    pilot: carData?.pilot.name,
                    laps: raceStats?.totalLaps,
                    avgPerformance: raceStats?.avgPerformance.toFixed(1),
                  };
                  navigator.clipboard.writeText(JSON.stringify(raceData, null, 2));
                  alert('Race data copied to clipboard!');
                }}
                className="bg-gray-700 hover:bg-gray-600 text-white px-6 py-2 rounded-lg transition-colors"
              >
                📋 Copy Data
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  },
);

export default RaceCompletionScreen;
