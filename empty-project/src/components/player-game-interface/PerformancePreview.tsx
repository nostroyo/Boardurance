import React from 'react';
import type { PerformancePreview as PerformancePreviewType } from '../../types/race-api';

/**
 * PerformancePreview Component
 *
 * Displays performance calculations and boost options from backend.
 * Shows base performance breakdown, boost options with predictions,
 * and boost cycle information.
 *
 * Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7
 */

interface PerformancePreviewProps {
  preview: PerformancePreviewType;
  selectedBoost: number | null;
  onBoostSelect: (boost: number) => void;
  availableBoosts: number[];
}

const PerformancePreviewComponent: React.FC<PerformancePreviewProps> = ({
  preview,
  selectedBoost,
  onBoostSelect,
  availableBoosts,
}) => {
  const { base_performance, boost_options, boost_cycle_info } = preview;

  /**
   * Get movement probability emoji indicator
   */
  const getMovementIcon = (probability: string): string => {
    switch (probability) {
      case 'MoveUp':
        return '⬆️';
      case 'Stay':
        return '⚪';
      case 'MoveDown':
        return '⬇️';
      default:
        return '❓';
    }
  };

  /**
   * Get lap characteristic icon
   */
  const getLapCharacteristicIcon = (characteristic: string): string => {
    return characteristic === 'Straight' ? '🏁' : '🌀';
  };

  return (
    <div className="bg-white rounded-lg shadow-md p-6 space-y-6">
      {/* Header */}
      <div className="border-b pb-4">
        <h2 className="text-2xl font-bold text-gray-800">Performance Preview</h2>
        <p className="text-sm text-gray-600 mt-1">
          {getLapCharacteristicIcon(base_performance.lap_characteristic)}{' '}
          {base_performance.lap_characteristic} Lap
        </p>
      </div>

      {/* Base Performance Breakdown */}
      <div className="space-y-3">
        <h3 className="text-lg font-semibold text-gray-700">Base Performance Breakdown</h3>

        <div className="bg-gray-50 rounded-lg p-4 space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-gray-600">Engine Contribution:</span>
            <span className="font-semibold text-blue-600">
              {base_performance.engine_contribution}
            </span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-600">Body Contribution:</span>
            <span className="font-semibold text-green-600">
              {base_performance.body_contribution}
            </span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-600">Pilot Contribution:</span>
            <span className="font-semibold text-purple-600">
              {base_performance.pilot_contribution}
            </span>
          </div>

          <div className="border-t pt-2 mt-2">
            <div className="flex justify-between items-center">
              <span className="font-medium text-gray-700">Base Total:</span>
              <span className="font-bold text-gray-900">{base_performance.base_value}</span>
            </div>
          </div>
        </div>

        {/* Sector Ceiling Information */}
        <div className="bg-yellow-50 rounded-lg p-4 space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-gray-700">Sector Ceiling:</span>
            <span className="font-semibold text-yellow-700">{base_performance.sector_ceiling}</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="font-medium text-gray-700">Capped Base Value:</span>
            <span className="font-bold text-yellow-800">{base_performance.capped_base_value}</span>
          </div>
        </div>
      </div>

      {/* Boost Options */}
      <div className="space-y-3">
        <h3 className="text-lg font-semibold text-gray-700">Boost Options</h3>

        <div className="grid grid-cols-1 gap-3">
          {boost_options.map((option) => {
            const isAvailable = availableBoosts.includes(option.boost_value);
            const isSelected = selectedBoost === option.boost_value;

            return (
              <button
                key={option.boost_value}
                onClick={() => isAvailable && onBoostSelect(option.boost_value)}
                disabled={!isAvailable}
                className={`
                  p-4 rounded-lg border-2 transition-all duration-200
                  ${
                    isSelected
                      ? 'border-blue-500 bg-blue-50 shadow-md'
                      : 'border-gray-300 bg-white hover:border-gray-400'
                  }
                  ${
                    !isAvailable
                      ? 'opacity-50 cursor-not-allowed bg-gray-100'
                      : 'cursor-pointer hover:shadow-md'
                  }
                `}
              >
                <div className="flex items-center justify-between">
                  {/* Boost Value */}
                  <div className="flex items-center space-x-3">
                    <div
                      className={`
                      w-12 h-12 rounded-full flex items-center justify-center font-bold text-xl
                      ${isSelected ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700'}
                    `}
                    >
                      {option.boost_value}
                    </div>

                    <div className="text-left">
                      <div className="font-semibold text-gray-900">
                        Final Value: {option.final_value}
                      </div>
                      <div className="text-sm text-gray-600">
                        {getMovementIcon(option.movement_probability)} {option.movement_probability}
                      </div>
                    </div>
                  </div>

                  {/* Availability Badge */}
                  <div>
                    {isAvailable ? (
                      <span className="px-3 py-1 bg-green-100 text-green-700 rounded-full text-sm font-medium">
                        Available
                      </span>
                    ) : (
                      <span className="px-3 py-1 bg-red-100 text-red-700 rounded-full text-sm font-medium">
                        Used
                      </span>
                    )}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Boost Cycle Information */}
      <div className="space-y-3">
        <h3 className="text-lg font-semibold text-gray-700">Boost Cycle Information</h3>

        <div className="bg-indigo-50 rounded-lg p-4 space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-gray-700">Current Cycle:</span>
            <span className="font-semibold text-indigo-700">{boost_cycle_info.current_cycle}</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-gray-700">Cards Remaining:</span>
            <span className="font-semibold text-indigo-700">
              {boost_cycle_info.cards_remaining} / 5
            </span>
          </div>

          {/* Progress Bar */}
          <div className="space-y-1">
            <div className="flex justify-between text-sm text-gray-600">
              <span>Cycle Progress</span>
              <span>{5 - boost_cycle_info.cards_remaining} / 5 used</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-3">
              <div
                className="bg-indigo-600 h-3 rounded-full transition-all duration-300"
                style={{
                  width: `${((5 - boost_cycle_info.cards_remaining) / 5) * 100}%`,
                }}
              />
            </div>
          </div>

          {/* Next Replenishment - Calculate based on cycle */}
          {boost_cycle_info.cards_remaining === 0 && (
            <div className="pt-2 border-t border-indigo-200">
              <div className="flex justify-between items-center">
                <span className="text-gray-700">Next Replenishment:</span>
                <span className="font-semibold text-indigo-700">Next Cycle</span>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export const PerformancePreview = React.memo(PerformancePreviewComponent);
export default PerformancePreview;
