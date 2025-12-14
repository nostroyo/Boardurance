import React, { useState } from 'react';
import type { CarData, LapHistory } from '../../types/race-api';
import { getRarityColor, getPilotClassIcon } from '../../types/player-assets';

type TabType = 'overview' | 'engine' | 'body' | 'pilot' | 'history';

/**
 * Updated PlayerCarCard props to use backend API data structure
 */
export interface PlayerCarCardProps {
  carData: CarData;
  lapHistory?: LapHistory;
}

/**
 * PlayerCarCard Component
 *
 * Displays detailed car specifications, pilot information, and performance history.
 * Features a tabbed interface for organizing different information categories.
 * Now integrates with backend car-data endpoint.
 *
 * Performance optimized with React.memo for expensive renders.
 */
const PlayerCarCardComponent: React.FC<PlayerCarCardProps> = ({ carData, lapHistory }) => {
  const [activeTab, setActiveTab] = useState<TabType>('overview');

  // Tab configuration
  const tabs: { id: TabType; label: string; icon: string }[] = [
    { id: 'overview', label: 'Overview', icon: '📊' },
    { id: 'engine', label: 'Engine', icon: '⚙️' },
    { id: 'body', label: 'Body', icon: '🏎️' },
    { id: 'pilot', label: 'Pilot', icon: '👤' },
    { id: 'history', label: 'History', icon: '📈' },
  ];

  return (
    <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
      {/* Header */}
      <div className="bg-gradient-to-r from-blue-900 to-purple-900 p-4">
        <h2 className="text-xl font-bold text-white mb-1">{carData.car.name}</h2>
        <p className="text-sm text-gray-300">
          {carData.pilot.name} • {getPilotClassIcon(carData.pilot.pilot_class)}{' '}
          {carData.pilot.pilot_class}
        </p>
      </div>

      {/* Tab Navigation */}
      <div className="flex border-b border-gray-700 bg-gray-900">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 px-3 py-3 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'bg-gray-800 text-white border-b-2 border-blue-500'
                : 'text-gray-400 hover:text-gray-300 hover:bg-gray-800/50'
            }`}
          >
            <span className="mr-1">{tab.icon}</span>
            <span className="hidden sm:inline">{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="p-4">
        {activeTab === 'overview' && <OverviewTab carData={carData} />}

        {activeTab === 'engine' && <EngineTab engine={carData.engine} />}

        {activeTab === 'body' && <BodyTab body={carData.body} />}

        {activeTab === 'pilot' && <PilotTab pilot={carData.pilot} />}

        {activeTab === 'history' && <HistoryTab lapHistory={lapHistory} />}
      </div>
    </div>
  );
};

// Overview Tab Component
const OverviewTab: React.FC<{
  carData: CarData;
}> = ({ carData }) => {
  const { pilot, engine, body } = carData;
  const totalStraight =
    engine.straight_value + body.straight_value + pilot.performance.straight_value;
  const totalCurve = engine.curve_value + body.curve_value + pilot.performance.curve_value;

  return (
    <div className="space-y-4">
      {/* Performance Summary */}
      <div>
        <h3 className="text-sm font-medium text-gray-400 mb-2">Performance</h3>
        <div className="space-y-2">
          <div className="flex items-center justify-between p-2 bg-gray-700 rounded">
            <span className="text-sm text-gray-300">→ Straight Performance:</span>
            <span className="font-bold text-blue-400">{totalStraight}</span>
          </div>
          <div className="flex items-center justify-between p-2 bg-gray-700 rounded">
            <span className="text-sm text-gray-300">↻ Curve Performance:</span>
            <span className="font-bold text-purple-400">{totalCurve}</span>
          </div>
          <div className="flex items-center justify-between p-2 bg-gray-700 rounded">
            <span className="text-sm text-gray-300">Average:</span>
            <span className="font-bold text-white">
              {Math.round((totalStraight + totalCurve) / 2)}
            </span>
          </div>
        </div>
      </div>

      {/* Component Summary */}
      <div>
        <h3 className="text-sm font-medium text-gray-400 mb-2">Components</h3>
        <div className="space-y-2 text-sm">
          <div className="flex items-center justify-between">
            <span className="text-gray-300">⚙️ {engine.name}</span>
            <span
              className="px-2 py-1 rounded text-xs font-medium"
              style={{
                backgroundColor: getRarityColor(engine.rarity) + '20',
                color: getRarityColor(engine.rarity),
              }}
            >
              {engine.rarity}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-300">🏎️ {body.name}</span>
            <span
              className="px-2 py-1 rounded text-xs font-medium"
              style={{
                backgroundColor: getRarityColor(body.rarity) + '20',
                color: getRarityColor(body.rarity),
              }}
            >
              {body.rarity}
            </span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-gray-300">👤 {pilot.name}</span>
            <span
              className="px-2 py-1 rounded text-xs font-medium"
              style={{
                backgroundColor: getRarityColor(pilot.rarity) + '20',
                color: getRarityColor(pilot.rarity),
              }}
            >
              {pilot.rarity}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

// Engine Tab Component
const EngineTab: React.FC<{ engine: CarData['engine'] }> = ({ engine }) => {
  return (
    <div className="space-y-4">
      {/* Engine Header */}
      <div className="text-center pb-4 border-b border-gray-700">
        <div className="text-4xl mb-2">⚙️</div>
        <h3 className="text-lg font-bold text-white">{engine.name}</h3>
        <span
          className="inline-block px-3 py-1 rounded-full text-sm font-medium mt-2"
          style={{
            backgroundColor: getRarityColor(engine.rarity) + '20',
            color: getRarityColor(engine.rarity),
          }}
        >
          {engine.rarity}
        </span>
      </div>

      {/* Engine Stats */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Performance Stats</h4>
        <div className="space-y-3">
          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-300">→ Straight Value</span>
              <span className="text-xl font-bold text-blue-400">{engine.straight_value}</span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500"
                style={{ width: `${(engine.straight_value / 100) * 100}%` }}
              />
            </div>
          </div>

          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-300">↻ Curve Value</span>
              <span className="text-xl font-bold text-purple-400">{engine.curve_value}</span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-purple-500"
                style={{ width: `${(engine.curve_value / 100) * 100}%` }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* NFT Info */}
      {engine.nft_mint_address && (
        <div className="bg-gray-700/50 rounded-lg p-3">
          <div className="text-xs text-gray-400 mb-1">NFT Mint Address</div>
          <div className="text-xs text-gray-300 font-mono break-all">{engine.nft_mint_address}</div>
        </div>
      )}
    </div>
  );
};

// Body Tab Component
const BodyTab: React.FC<{ body: CarData['body'] }> = ({ body }) => {
  return (
    <div className="space-y-4">
      {/* Body Header */}
      <div className="text-center pb-4 border-b border-gray-700">
        <div className="text-4xl mb-2">🏎️</div>
        <h3 className="text-lg font-bold text-white">{body.name}</h3>
        <span
          className="inline-block px-3 py-1 rounded-full text-sm font-medium mt-2"
          style={{
            backgroundColor: getRarityColor(body.rarity) + '20',
            color: getRarityColor(body.rarity),
          }}
        >
          {body.rarity}
        </span>
      </div>

      {/* Body Stats */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Performance Stats</h4>
        <div className="space-y-3">
          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-300">→ Straight Value</span>
              <span className="text-xl font-bold text-blue-400">{body.straight_value}</span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500"
                style={{ width: `${(body.straight_value / 100) * 100}%` }}
              />
            </div>
          </div>

          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm text-gray-300">↻ Curve Value</span>
              <span className="text-xl font-bold text-purple-400">{body.curve_value}</span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-purple-500"
                style={{ width: `${(body.curve_value / 100) * 100}%` }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* NFT Info */}
      {body.nft_mint_address && (
        <div className="bg-gray-700/50 rounded-lg p-3">
          <div className="text-xs text-gray-400 mb-1">NFT Mint Address</div>
          <div className="text-xs text-gray-300 font-mono break-all">{body.nft_mint_address}</div>
        </div>
      )}
    </div>
  );
};

// Pilot Tab Component - Enhanced with detailed skills breakdown
const PilotTab: React.FC<{ pilot: CarData['pilot'] }> = ({ pilot }) => {
  // Skill icons mapping
  const skillIcons: Record<string, string> = {
    reaction_time: '⚡',
    precision: '🎯',
    focus: '🧠',
    stamina: '💪',
  };

  // Skill descriptions
  const skillDescriptions: Record<string, string> = {
    reaction_time: 'Quick response to race conditions',
    precision: 'Accuracy in boost timing and control',
    focus: 'Mental concentration during races',
    stamina: 'Endurance for long races',
  };

  return (
    <div className="space-y-4">
      {/* Pilot Header */}
      <div className="text-center pb-4 border-b border-gray-700">
        <div className="text-4xl mb-2">{getPilotClassIcon(pilot.pilot_class)}</div>
        <h3 className="text-lg font-bold text-white">{pilot.name}</h3>
        <div className="flex items-center justify-center space-x-2 mt-2">
          <span
            className="px-3 py-1 rounded-full text-sm font-medium"
            style={{
              backgroundColor: getRarityColor(pilot.rarity) + '20',
              color: getRarityColor(pilot.rarity),
            }}
          >
            {pilot.rarity}
          </span>
          <span className="px-3 py-1 rounded-full text-sm font-medium bg-gray-700 text-gray-300">
            {pilot.pilot_class}
          </span>
        </div>
      </div>

      {/* Enhanced Pilot Skills Breakdown */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Skills Breakdown</h4>
        <div className="space-y-3">
          {Object.entries(pilot.skills).map(([skill, value]) => (
            <div key={skill} className="bg-gray-700 rounded-lg p-3">
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center space-x-2">
                  <span className="text-lg">{skillIcons[skill] || '📊'}</span>
                  <div>
                    <span className="text-sm font-medium text-white capitalize">
                      {skill.replace('_', ' ')}
                    </span>
                    <div className="text-xs text-gray-400">{skillDescriptions[skill]}</div>
                  </div>
                </div>
                <span className="text-lg font-bold text-white">{value}</span>
              </div>
              <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-green-500 to-blue-500 transition-all duration-300"
                  style={{ width: `${Math.min((value / 100) * 100, 100)}%` }}
                />
              </div>
              <div className="flex justify-between text-xs text-gray-500 mt-1">
                <span>0</span>
                <span>100</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Enhanced Performance Values */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Performance Values</h4>
        <div className="space-y-3">
          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-2">
                <span className="text-lg">🏁</span>
                <span className="text-sm font-medium text-gray-300">Straight Performance</span>
              </div>
              <span className="text-xl font-bold text-blue-400">
                {pilot.performance.straight_value}
              </span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-blue-500 transition-all duration-300"
                style={{
                  width: `${Math.min((pilot.performance.straight_value / 50) * 100, 100)}%`,
                }}
              />
            </div>
          </div>

          <div className="bg-gray-700 rounded-lg p-3">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-2">
                <span className="text-lg">🌀</span>
                <span className="text-sm font-medium text-gray-300">Curve Performance</span>
              </div>
              <span className="text-xl font-bold text-purple-400">
                {pilot.performance.curve_value}
              </span>
            </div>
            <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
              <div
                className="h-full bg-purple-500 transition-all duration-300"
                style={{ width: `${Math.min((pilot.performance.curve_value / 50) * 100, 100)}%` }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Pilot Class and Rarity Information */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Pilot Information</h4>
        <div className="bg-gray-700 rounded-lg p-3 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-300">Class</span>
            <div className="flex items-center space-x-1">
              <span>{getPilotClassIcon(pilot.pilot_class)}</span>
              <span className="text-sm font-medium text-white">{pilot.pilot_class}</span>
            </div>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-300">Rarity</span>
            <span
              className="px-2 py-1 rounded text-xs font-medium"
              style={{
                backgroundColor: getRarityColor(pilot.rarity) + '20',
                color: getRarityColor(pilot.rarity),
              }}
            >
              {pilot.rarity}
            </span>
          </div>
        </div>
      </div>

      {/* NFT Info */}
      {pilot.nft_mint_address && (
        <div className="bg-gray-700/50 rounded-lg p-3">
          <div className="text-xs text-gray-400 mb-1">NFT Mint Address</div>
          <div className="text-xs text-gray-300 font-mono break-all">{pilot.nft_mint_address}</div>
        </div>
      )}
    </div>
  );
};

// History Tab Component - Enhanced with backend lap history data
const HistoryTab: React.FC<{ lapHistory?: LapHistory }> = ({ lapHistory }) => {
  if (!lapHistory || lapHistory.laps.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="text-4xl mb-2 text-gray-600">📊</div>
        <p className="text-gray-400">No lap history available yet</p>
        <p className="text-sm text-gray-500 mt-1">Complete laps to see your performance history</p>
      </div>
    );
  }

  // Find best lap
  const bestLap = lapHistory.laps.reduce(
    (best, lap) => (lap.final_value > best.final_value ? lap : best),
    lapHistory.laps[0],
  );

  // Calculate average boost usage
  const avgBoost =
    lapHistory.laps.reduce((sum, lap) => sum + lap.boost_used, 0) / lapHistory.laps.length;

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

  return (
    <div className="space-y-4">
      {/* Performance Summary */}
      <div className="grid grid-cols-2 gap-3">
        {/* Best Lap Highlight */}
        <div className="bg-gradient-to-r from-yellow-900/30 to-orange-900/30 border border-yellow-700/50 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-yellow-400">🏆 Best Lap</span>
            <span className="text-xs text-gray-400">Lap {bestLap.lap_number}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-300">
              {bestLap.lap_characteristic === 'Straight' ? '🏁' : '🌀'} {bestLap.lap_characteristic}
            </span>
            <span className="text-xl font-bold text-white">{bestLap.final_value}</span>
          </div>
        </div>

        {/* Average Boost */}
        <div className="bg-gradient-to-r from-blue-900/30 to-purple-900/30 border border-blue-700/50 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-blue-400">📈 Avg Boost</span>
            <span className="text-xs text-gray-400">{lapHistory.laps.length} laps</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-300">Usage</span>
            <span className="text-xl font-bold text-white">{avgBoost.toFixed(1)}</span>
          </div>
        </div>
      </div>

      {/* Boost Usage Patterns */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Boost Usage Patterns</h4>
        <div className="bg-gray-700 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-300">Boost Distribution</span>
          </div>
          <div className="flex space-x-1 h-6 bg-gray-600 rounded overflow-hidden">
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
                  className={`${colors[boostValue]} transition-all duration-300`}
                  style={{ width: `${percentage}%` }}
                  title={`Boost ${boostValue}: ${count} times (${percentage.toFixed(1)}%)`}
                />
              );
            })}
          </div>
          <div className="flex justify-between text-xs text-gray-400 mt-1">
            <span>0</span>
            <span>1</span>
            <span>2</span>
            <span>3</span>
            <span>4</span>
          </div>
        </div>
      </div>

      {/* Cycle Summaries */}
      {lapHistory.cycle_summaries.length > 0 && (
        <div>
          <h4 className="text-sm font-medium text-gray-400 mb-3">Cycle Summaries</h4>
          <div className="space-y-2 max-h-32 overflow-y-auto">
            {lapHistory.cycle_summaries.map((cycle) => (
              <div key={cycle.cycle_number} className="bg-gray-700 rounded-lg p-2">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium text-white">Cycle {cycle.cycle_number}</span>
                  <span className="text-sm font-bold text-blue-400">
                    {cycle.average_boost.toFixed(1)} avg
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs text-gray-400">
                  <span>Cards: [{cycle.cards_used.join(', ')}]</span>
                  <span>{cycle.laps_in_cycle.length} laps</span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Lap-by-Lap Performance Chart */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Lap-by-Lap Performance</h4>
        <div className="space-y-2 max-h-64 overflow-y-auto">
          {lapHistory.laps.map((lap) => (
            <div
              key={lap.lap_number}
              className="bg-gray-700 rounded-lg p-3 hover:bg-gray-600 transition-colors"
            >
              <div className="flex items-center justify-between mb-2">
                <div className="flex items-center space-x-2">
                  <span className="text-sm font-medium text-white">Lap {lap.lap_number}</span>
                  <span className="text-xs text-gray-400">
                    {lap.lap_characteristic === 'Straight' ? '🏁' : '🌀'} {lap.lap_characteristic}
                  </span>
                  <span className="text-xs bg-gray-600 px-2 py-1 rounded">
                    Cycle {lap.boost_cycle}
                  </span>
                </div>
                <span className="text-lg font-bold text-white">{lap.final_value}</span>
              </div>

              <div className="flex items-center justify-between text-xs text-gray-400 mb-2">
                <div className="flex items-center space-x-3">
                  <span>Base: {lap.base_value}</span>
                  <span
                    className={`font-medium ${
                      lap.boost_used === 0
                        ? 'text-gray-400'
                        : lap.boost_used <= 2
                          ? 'text-green-400'
                          : 'text-red-400'
                    }`}
                  >
                    Boost: +{lap.boost_used}
                  </span>
                </div>
                <div className="flex items-center space-x-2">
                  <span>
                    Sector {lap.from_sector} → {lap.to_sector}
                  </span>
                  <span className="text-lg">{getMovementIcon(lap.movement_type)}</span>
                </div>
              </div>

              {/* Performance bar */}
              <div className="h-2 bg-gray-600 rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-blue-500 to-purple-500 transition-all duration-300"
                  style={{ width: `${Math.min((lap.final_value / 100) * 100, 100)}%` }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export const PlayerCarCard = React.memo(PlayerCarCardComponent);
export default PlayerCarCard;
