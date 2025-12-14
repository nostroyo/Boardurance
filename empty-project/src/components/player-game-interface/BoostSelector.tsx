import React, { useState } from 'react';

export interface BoostSelectorProps {
  selectedBoost: number | null;
  availableBoosts?: number[];
  onBoostSelect: (boost: number) => void;
  onSubmit?: () => void;
  isSubmitting?: boolean;
  hasSubmitted?: boolean;
  disabled?: boolean;
  showPreview?: boolean;
  previewValue?: number | null;
}

/**
 * BoostSelector Component
 *
 * Interactive boost value selector (0-4 range) with availability validation,
 * submission controls, and confirmation dialog.
 *
 * Requirements: 3.1, 3.2, 3.3, 3.4, 3.6, 3.7
 */
export const BoostSelector: React.FC<BoostSelectorProps> = ({
  selectedBoost,
  availableBoosts,
  onBoostSelect,
  onSubmit,
  isSubmitting,
  hasSubmitted,
}) => {
  const [showConfirmation, setShowConfirmation] = useState(false);
  const boostValues = [0, 1, 2, 3, 4];

  const handleBoostClick = (boost: number) => {
    if (hasSubmitted || isSubmitting) return;

    // Only allow selection of available boosts
    if (!availableBoosts?.includes(boost)) {
      return;
    }

    onBoostSelect(boost);
  };

  const handleSubmitClick = () => {
    if (selectedBoost === null || !availableBoosts?.includes(selectedBoost)) {
      return;
    }
    setShowConfirmation(true);
  };

  const handleConfirmSubmit = () => {
    setShowConfirmation(false);
    onSubmit?.();
  };

  const handleCancelSubmit = () => {
    setShowConfirmation(false);
  };

  const isBoostAvailable = (boost: number): boolean => {
    return availableBoosts?.includes(boost) ?? false;
  };

  const canSubmit =
    selectedBoost !== null &&
    availableBoosts?.includes(selectedBoost) &&
    !hasSubmitted &&
    !isSubmitting;

  return (
    <div className="space-y-4">
      <label className="block text-sm font-medium text-gray-200">Select Boost Value (0-4):</label>

      {/* Boost value buttons */}
      <div className="grid grid-cols-5 gap-2">
        {boostValues.map((boost) => {
          const isSelected = selectedBoost === boost;
          const isAvailable = isBoostAvailable(boost);
          const isDisabled = hasSubmitted || isSubmitting || !isAvailable;

          return (
            <div key={boost} className="relative">
              <button
                onClick={() => handleBoostClick(boost)}
                disabled={isDisabled}
                className={`
                  w-full aspect-square rounded-lg border font-bold text-lg 
                  transition-all duration-200 relative
                  ${
                    isSelected
                      ? 'bg-blue-600 border-blue-500 text-white shadow-lg shadow-blue-500/30 scale-105'
                      : isAvailable && !hasSubmitted && !isSubmitting
                        ? 'bg-gray-700 border-gray-600 text-gray-300 hover:bg-gray-600 hover:border-gray-500 hover:scale-105'
                        : 'bg-gray-800 border-gray-700 text-gray-600 cursor-not-allowed opacity-50'
                  }
                `}
                aria-label={`Select boost value ${boost}`}
                aria-pressed={isSelected}
                aria-disabled={isDisabled}
              >
                {boost}
              </button>

              {/* "Already Used" badge */}
              {!isAvailable && (
                <div className="absolute -top-1 -right-1 bg-red-600 text-white text-[10px] px-1.5 py-0.5 rounded-full font-medium shadow-lg">
                  Used
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Selected boost display */}
      {selectedBoost !== null && (
        <div className="flex items-center justify-between text-sm">
          <span className="text-gray-400">
            Selected boost: <span className="font-medium text-blue-400">{selectedBoost}</span>
          </span>
          {availableBoosts?.includes(selectedBoost) ? (
            <span className="text-green-400 text-xs">✓ Available</span>
          ) : (
            <span className="text-red-400 text-xs">✗ Not Available</span>
          )}
        </div>
      )}

      {/* Available boosts info */}
      <div className="text-xs text-gray-500">
        Available boosts:{' '}
        {(availableBoosts?.length ?? 0) > 0 ? availableBoosts?.join(', ') : 'None'}
      </div>

      {/* Submit button or confirmation dialog */}
      {!showConfirmation ? (
        <button
          onClick={handleSubmitClick}
          disabled={!canSubmit}
          className={`
            w-full px-4 py-3 rounded-lg font-medium transition-all duration-200
            flex items-center justify-center space-x-2
            ${
              canSubmit
                ? 'bg-green-600 hover:bg-green-700 text-white shadow-lg hover:shadow-green-500/30'
                : 'bg-gray-700 text-gray-500 cursor-not-allowed'
            }
          `}
          aria-label="Submit boost selection"
        >
          <span>Submit Boost</span>
          {selectedBoost !== null && <span>({selectedBoost})</span>}
          <span>🚀</span>
        </button>
      ) : (
        <div className="space-y-2">
          <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-3">
            <p className="font-medium text-yellow-400 text-sm mb-1">Confirm Action</p>
            <p className="text-xs text-gray-400">
              You are about to submit boost value:{' '}
              <span className="font-bold text-yellow-400">{selectedBoost}</span>
            </p>
            <p className="text-xs text-gray-500 mt-1">
              This action cannot be changed after submission.
            </p>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={handleCancelSubmit}
              disabled={isSubmitting}
              className="bg-gray-600 hover:bg-gray-700 disabled:bg-gray-700 disabled:cursor-not-allowed px-4 py-2 rounded-lg font-medium transition-colors text-sm"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirmSubmit}
              disabled={isSubmitting}
              className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-4 py-2 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2 text-sm"
            >
              {isSubmitting ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                  <span>Submitting...</span>
                </>
              ) : (
                <span>Confirm</span>
              )}
            </button>
          </div>
        </div>
      )}

      {/* Action Submitted state */}
      {hasSubmitted && (
        <div className="bg-green-900/20 border border-green-800 rounded-lg p-4 text-center">
          <div className="text-green-400 text-3xl mb-2">✓</div>
          <p className="text-green-400 font-medium mb-1">Action Submitted</p>
          <p className="text-gray-400 text-sm">
            Boost value: <span className="font-medium text-green-400">{selectedBoost}</span>
          </p>
          <p className="text-gray-500 text-xs mt-2">Waiting for other players...</p>
        </div>
      )}

      {/* Validation message */}
      {selectedBoost !== null && !availableBoosts?.includes(selectedBoost) && !hasSubmitted && (
        <div className="text-xs text-red-400 bg-red-900/20 border border-red-800 rounded px-3 py-2">
          ⚠️ This boost card has already been used in the current cycle
        </div>
      )}
    </div>
  );
};

export default BoostSelector;
