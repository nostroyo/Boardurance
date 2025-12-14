/**
 * LapHistoryPanel - Lazy-loaded component for displaying detailed lap history
 *
 * This component is lazy-loaded to improve initial page load performance.
 * It provides detailed lap-by-lap analysis and performance trends.
 *
 * Requirements: 12.5
 */

import React from 'react';
import type { LapHistory } from '../../types/race-api';

export interface LapHistoryPanelProps {
  lapHistory: LapHistory;
  playerName?: string;
}

export const LapHistoryPanel: React.FC<LapHistoryPanelProps> = React.memo(
  ({ lapHistory, playerName = 'Player' }) => {
    // Find best lap
    const bestLap = lapHistory.laps.reduce(
      (best, lap) => (lap.final_value > best.final_value ? lap : best),
      lapHistory.laps[0],
    );

    // Calculate statistics
    const avgBoost =
      lapHistory.laps.reduce((sum, lap) => sum + lap.boost_used, 0) / lapHistory.laps.length;
    const avgPerformance =
      lapHistory.laps.reduce((sum, lap) => sum + lap.final_value, 0) / lapHistory.laps.length;

    // Movement type icons
    const getMovementIcon = (movementType: string) => {
      switch (movementType.toLowerCase()) {
        case 'forward':
          return '↗️';
        case 'backward':
          return '↙️';
        case 'stay':
          return '➡️';
        default:
          return '❓';
      }
    };

    // Get lap characteristic icon
    const getLapCharacteristicIcon = (characteristic: string) => {
      return characteristic === 'Straight' ? '🏁' : '🌀';
    };

    return (
      <div className="bg-gray-800 rounded-lg p-6 space-y-6">
        {/* Header */}
        <div className="border-b border-gray-700 pb-4">
          <h2 className="text-2xl font-bold text-white mb-2">📈 Lap History Analysis</h2>
          <p className="text-gray-400">{playerName}'s Performance Overview</p>
        </div>

        {/* Performance Statistics */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="bg-gradient-to-r from-yellow-900/30 to-orange-900/30 border border-yellow-700/50 rounded-lg p-4">
            <div className="text-yellow-400 text-sm font-medium mb-1">🏆 Best Lap</div>
            <div className="text-2xl font-bold text-white">{bestLap.final_value}</div>
            <div className="text-xs text-gray-400">
              Lap {bestLap.lap_number} ({bestLap.lap_characteristic})
            </div>
          </div>

          <div className="bg-gradient-to-r from-blue-900/30 to-purple-900/30 border border-blue-700/50 rounded-lg p-4">
            <div className="text-blue-400 text-sm font-medium mb-1">📊 Avg Performance</div>
            <div className="text-2xl font-bold text-white">{avgPerformance.toFixed(1)}</div>
            <div className="text-xs text-gray-400">Across {lapHistory.laps.length} laps</div>
          </div>

          <div className="bg-gradient-to-r from-green-900/30 to-teal-900/30 border border-green-700/50 rounded-lg p-4">
            <div className="text-green-400 text-sm font-medium mb-1">🚀 Avg Boost</div>
            <div className="text-2xl font-bold text-white">{avgBoost.toFixed(1)}</div>
            <div className="text-xs text-gray-400">Boost usage rate</div>
          </div>

          <div className="bg-gradient-to-r from-purple-900/30 to-pink-900/30 border border-purple-700/50 rounded-lg p-4">
            <div className="text-purple-400 text-sm font-medium mb-1">🔄 Cycles</div>
            <div className="text-2xl font-bold text-white">{lapHistory.cycle_summaries.length}</div>
            <div className="text-xs text-gray-400">Boost cycles completed</div>
          </div>
        </div>

        {/* Boost Usage Distribution */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-white">Boost Usage Distribution</h3>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex space-x-1 h-8 bg-gray-600 rounded overflow-hidden mb-3">
              {[0, 1, 2, 3, 4].map((boostValue) => {
                const count = lapHistory.laps.filter((lap) => lap.boost_used === boostValue).length;
                const percentage = (count / lapHistory.laps.length) * 100;
                const colors = [
                  'bg-gray-500',
                  'bg-green-500',
                  'bg-blue-500',
                  'bg-purple-500',
                  'bg-red-500',
                ];

                return (
                  <div
                    key={boostValue}
                    className={`${colors[boostValue]} transition-all duration-300 flex items-center justify-center text-white text-xs font-bold`}
                    style={{ width: `${percentage}%` }}
                    title={`Boost ${boostValue}: ${count} times (${percentage.toFixed(1)}%)`}
                  >
                    {percentage > 10 ? boostValue : ''}
                  </div>
                );
              })}
            </div>
            <div className="grid grid-cols-5 gap-2 text-xs">
              {[0, 1, 2, 3, 4].map((boostValue) => {
                const count = lapHistory.laps.filter((lap) => lap.boost_used === boostValue).length;
                const percentage = (count / lapHistory.laps.length) * 100;

                return (
                  <div key={boostValue} className="text-center">
                    <div className="text-white font-semibold">Boost {boostValue}</div>
                    <div className="text-gray-400">{count} times</div>
                    <div className="text-gray-500">{percentage.toFixed(1)}%</div>
                  </div>
                );
              })}
            </div>
          </div>
        </div>

        {/* Cycle Summaries */}
        {lapHistory.cycle_summaries.length > 0 && (
          <div className="space-y-3">
            <h3 className="text-lg font-semibold text-white">Boost Cycle Analysis</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              {lapHistory.cycle_summaries.map((cycle) => (
                <div key={cycle.cycle_number} className="bg-gray-700 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-lg font-semibold text-white">
                      Cycle {cycle.cycle_number}
                    </span>
                    <span className="text-lg font-bold text-blue-400">
                      {cycle.average_boost.toFixed(1)} avg
                    </span>
                  </div>

                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Cards Used:</span>
                      <span className="text-white font-mono">[{cycle.cards_used.join(', ')}]</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Laps in Cycle:</span>
                      <span className="text-white">{cycle.laps_in_cycle.length}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Lap Range:</span>
                      <span className="text-white">
                        {Math.min(...cycle.laps_in_cycle)} - {Math.max(...cycle.laps_in_cycle)}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Detailed Lap History */}
        <div className="space-y-3">
          <h3 className="text-lg font-semibold text-white">Lap-by-Lap Performance</h3>
          <div className="space-y-2 max-h-96 overflow-y-auto">
            {lapHistory.laps.map((lap) => (
              <div
                key={lap.lap_number}
                className={`bg-gray-700 rounded-lg p-4 hover:bg-gray-600 transition-colors ${
                  lap.lap_number === bestLap.lap_number ? 'ring-2 ring-yellow-500/50' : ''
                }`}
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center space-x-3">
                    <span className="text-lg font-semibold text-white">Lap {lap.lap_number}</span>
                    <span className="text-sm text-gray-400">
                      {getLapCharacteristicIcon(lap.lap_characteristic)} {lap.lap_characteristic}
                    </span>
                    <span className="text-xs bg-gray-600 px-2 py-1 rounded">
                      Cycle {lap.boost_cycle}
                    </span>
                    {lap.lap_number === bestLap.lap_number && (
                      <span className="text-xs bg-yellow-600 text-yellow-100 px-2 py-1 rounded font-bold">
                        🏆 BEST
                      </span>
                    )}
                  </div>
                  <span className="text-xl font-bold text-white">{lap.final_value}</span>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm mb-3">
                  <div>
                    <span className="text-gray-400">Base Value:</span>
                    <div className="text-white font-semibold">{lap.base_value}</div>
                  </div>
                  <div>
                    <span className="text-gray-400">Boost Used:</span>
                    <div
                      className={`font-semibold ${
                        lap.boost_used === 0
                          ? 'text-gray-400'
                          : lap.boost_used <= 2
                            ? 'text-green-400'
                            : 'text-red-400'
                      }`}
                    >
                      +{lap.boost_used}
                    </div>
                  </div>
                  <div>
                    <span className="text-gray-400">Movement:</span>
                    <div className="text-white font-semibold">
                      {getMovementIcon(lap.movement_type)} {lap.movement_type}
                    </div>
                  </div>
                  <div>
                    <span className="text-gray-400">Sectors:</span>
                    <div className="text-white font-semibold">
                      {lap.from_sector} → {lap.to_sector}
                    </div>
                  </div>
                </div>

                {/* Performance bar */}
                <div className="space-y-1">
                  <div className="flex justify-between text-xs text-gray-400">
                    <span>Performance</span>
                    <span>{lap.final_value} / 100</span>
                  </div>
                  <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-300"
                      style={{ width: `${Math.min((lap.final_value / 100) * 100, 100)}%` }}
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  },
);

export default LapHistoryPanel;
