import React, { useMemo } from 'react';
import type { PerformanceCalculatorProps } from '../../types/ui-state';
import { calculatePerformance } from '../../utils/performanceCalculation';

/**
 * PerformanceCalculator Component
 *
 * Displays real-time performance calculations with interactive boost simulation.
 * Shows base value calculation, sector ceiling application, and final value prediction.
 */
export const PerformanceCalculator: React.FC<PerformanceCalculatorProps> = ({
  pilot,
  engine,
  body,
  currentSector,
  lapCharacteristic,
  selectedBoost,
  onBoostChange,
}) => {
  // Calculate performance breakdown
  const performance = useMemo(() => {
    return calculatePerformance(
      pilot,
      engine,
      body,
      currentSector,
      lapCharacteristic as 'Straight' | 'Curve',
      selectedBoost,
    );
  }, [pilot, engine, body, currentSector, lapCharacteristic, selectedBoost]);

  // Check if base value exceeds sector ceiling
  const isBaseValueCapped = performance.baseValue > currentSector.max_value;

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
      <h3 className="text-lg font-bold mb-4 text-white">Performance Calculator</h3>

      {/* Lap Characteristic Indicator */}
      <div className="mb-4 p-3 bg-gray-700 rounded-lg">
        <div className="flex items-center justify-between">
          <span className="text-sm text-gray-400">Current Lap Type:</span>
          <div className="flex items-center space-x-2">
            <span
              className={`text-lg font-bold ${
                lapCharacteristic === 'Straight' ? 'text-blue-400' : 'text-purple-400'
              }`}
            >
              {lapCharacteristic === 'Straight' ? '→' : '↻'}
            </span>
            <span className="font-medium text-white">{lapCharacteristic}</span>
          </div>
        </div>
      </div>

      {/* Base Value Calculation */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-gray-300 mb-2">Base Performance</h4>
        <div className="space-y-2">
          {/* Engine Contribution */}
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Engine ({lapCharacteristic}):</span>
            <span className="font-medium text-green-400">+{performance.engineContribution}</span>
          </div>

          {/* Body Contribution */}
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Body ({lapCharacteristic}):</span>
            <span className="font-medium text-green-400">+{performance.bodyContribution}</span>
          </div>

          {/* Pilot Contribution */}
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">Pilot ({lapCharacteristic}):</span>
            <span className="font-medium text-green-400">+{performance.pilotContribution}</span>
          </div>

          {/* Base Value Total */}
          <div className="pt-2 border-t border-gray-600">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-300">Base Value:</span>
              <span className="text-lg font-bold text-white">{performance.baseValue}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Sector Ceiling Visualization */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-gray-300 mb-2">Sector Ceiling</h4>
        <div className="bg-gray-700 rounded-lg p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">Sector Max Value:</span>
            <span className="font-medium text-yellow-400">{currentSector.max_value}</span>
          </div>

          {/* Visual ceiling indicator */}
          <div className="relative h-2 bg-gray-600 rounded-full overflow-hidden">
            <div
              className={`absolute left-0 top-0 h-full transition-all duration-300 ${
                isBaseValueCapped ? 'bg-red-500' : 'bg-green-500'
              }`}
              style={{
                width: `${Math.min((performance.baseValue / currentSector.max_value) * 100, 100)}%`,
              }}
            />
          </div>

          {isBaseValueCapped && (
            <p className="text-xs text-red-400 mt-2">
              ⚠️ Base value exceeds sector ceiling and will be capped
            </p>
          )}

          {/* Capped Value Display */}
          <div className="flex items-center justify-between mt-3 pt-2 border-t border-gray-600">
            <span className="text-sm font-medium text-gray-300">After Ceiling:</span>
            <span className="text-lg font-bold text-white">{performance.sectorCappedValue}</span>
          </div>
        </div>
      </div>

      {/* Interactive Boost Simulation */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-gray-300 mb-2">Boost Simulation</h4>
        <div className="bg-gray-700 rounded-lg p-3">
          <label className="block text-xs text-gray-400 mb-2">Select Boost Value (0-4):</label>

          {/* Boost Selector Buttons */}
          <div className="grid grid-cols-6 gap-2 mb-3">
            {[0, 1, 2, 3, 4, 5].map((boost) => (
              <button
                key={boost}
                onClick={() => onBoostChange(boost)}
                className={`aspect-square rounded-lg border font-bold text-sm transition-all duration-200 ${
                  selectedBoost === boost
                    ? 'bg-blue-600 border-blue-500 text-white shadow-lg shadow-blue-500/30 scale-105'
                    : 'bg-gray-600 border-gray-500 text-gray-300 hover:bg-gray-500 hover:border-gray-400'
                }`}
              >
                {boost}
              </button>
            ))}
          </div>

          {/* Boost Value Display */}
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-400">Boost Addition:</span>
            <span className="font-medium text-blue-400">+{performance.boostValue}</span>
          </div>
        </div>
      </div>

      {/* Final Value Prediction */}
      <div className="bg-gradient-to-r from-blue-900/50 to-purple-900/50 rounded-lg p-4 border border-blue-700">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm font-medium text-gray-300">Final Performance Value:</span>
          <span className="text-2xl font-bold text-white">{performance.finalValue}</span>
        </div>

        {/* Performance Breakdown Summary */}
        <div className="text-xs text-gray-400 space-y-1">
          <div className="flex justify-between">
            <span>Capped Base:</span>
            <span>{performance.sectorCappedValue}</span>
          </div>
          <div className="flex justify-between">
            <span>Boost:</span>
            <span>+{performance.boostValue}</span>
          </div>
          <div className="flex justify-between pt-1 border-t border-gray-600 font-medium text-white">
            <span>Total:</span>
            <span>{performance.finalValue}</span>
          </div>
        </div>
      </div>

      {/* Performance Tips */}
      <div className="mt-4 p-3 bg-gray-700/50 rounded-lg">
        <p className="text-xs text-gray-400">
          💡 <span className="font-medium">Tip:</span>{' '}
          {isBaseValueCapped
            ? 'Your base performance exceeds the sector ceiling. Consider saving boost for higher sectors.'
            : 'Your base performance is within the sector ceiling. Boost will add directly to your final value.'}
        </p>
      </div>
    </div>
  );
};

export default PerformanceCalculator;
