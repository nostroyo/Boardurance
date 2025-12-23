import React, { useState, useEffect } from 'react';
import type { TurnPhase } from '../../types/race';

export interface BoostControlPanelProps {
  selectedBoost: number | null;
  availableBoosts: number[];
  onBoostSelect: (boost: number) => void;
  onValidateTurn: () => void;
  isSubmitting: boolean;
  hasSubmitted: boolean;
  turnPhase: TurnPhase;
}

export interface BoostButtonState {
  value: number;
  available: boolean;
  selected: boolean;
  used: boolean;
}

/**
 * BoostControlPanel Component
 *
 * Prominent boost control panel with visible boost buttons (0-5) and validate turn button.
 * Designed for the race interface redesign with enhanced visibility and user experience.
 *
 * Requirements: 3.1, 3.2, 3.3, 7.1, 7.2
 * Subtask 4.1 Requirements: 3.3, 3.4, 3.5, 7.3, 7.4, 7.5
 */
export const BoostControlPanel: React.FC<BoostControlPanelProps> = ({
  selectedBoost,
  availableBoosts,
  onBoostSelect,
  onValidateTurn,
  isSubmitting,
  hasSubmitted,
  turnPhase,
}) => {
  const [showConfirmation, setShowConfirmation] = useState(false);
  const [selectionFeedback, setSelectionFeedback] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  // Boost values 0-4 as specified in requirements
  const boostValues = [0, 1, 2, 3, 4];

  // Reset states when turn phase changes or submission completes
  useEffect(() => {
    if (turnPhase !== 'WaitingForPlayers') {
      setShowConfirmation(false);
      setSelectionFeedback(null);
      setValidationError(null);
    }
  }, [turnPhase]);

  // Clear feedback after a delay
  useEffect(() => {
    if (selectionFeedback) {
      const timer = setTimeout(() => {
        setSelectionFeedback(null);
      }, 2000);
      return () => clearTimeout(timer);
    }
  }, [selectionFeedback]);

  // Handle boost button click with immediate visual feedback
  const handleBoostClick = (boost: number) => {
    if (hasSubmitted || isSubmitting || turnPhase !== 'WaitingForPlayers') return;

    // Clear any previous validation errors
    setValidationError(null);

    // Check if boost is available
    if (!availableBoosts.includes(boost)) {
      setSelectionFeedback(`Boost ${boost} is not available (already used)`);
      return;
    }

    // Provide immediate feedback for selection
    if (selectedBoost === boost) {
      setSelectionFeedback(`Boost ${boost} is already selected`);
    } else {
      onBoostSelect(boost);
      setSelectionFeedback(`Boost ${boost} selected`);
    }
  };

  // Handle validate turn with enhanced error handling
  const handleValidateClick = () => {
    if (selectedBoost === null) {
      setValidationError('Please select a boost value first');
      return;
    }

    if (!availableBoosts.includes(selectedBoost)) {
      setValidationError('Selected boost is not available');
      return;
    }

    setValidationError(null);
    setShowConfirmation(true);
  };

  // Handle confirmation with loading state
  const handleConfirmValidate = () => {
    setIsValidating(true);
    setValidationError(null);

    try {
      onValidateTurn();
      setShowConfirmation(false);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to validate turn';
      setValidationError(errorMessage);
    } finally {
      setIsValidating(false);
    }
  };

  const handleCancelValidate = () => {
    setShowConfirmation(false);
    setValidationError(null);
  };

  const getBoostButtonState = (boost: number): BoostButtonState => {
    return {
      value: boost,
      available: availableBoosts.includes(boost),
      selected: selectedBoost === boost,
      used: !availableBoosts.includes(boost),
    };
  };

  const canValidateTurn =
    selectedBoost !== null &&
    availableBoosts.includes(selectedBoost) &&
    !hasSubmitted &&
    !isSubmitting &&
    !isValidating &&
    turnPhase === 'WaitingForPlayers';

  const isInteractionDisabled =
    hasSubmitted || isSubmitting || isValidating || turnPhase !== 'WaitingForPlayers';

  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-3 sm:p-4 lg:p-6 space-y-4 sm:space-y-5 lg:space-y-6 w-full max-w-full">
      {/* Panel Header - Mobile responsive */}
      <div className="text-center">
        <h3 className="text-lg sm:text-xl font-bold text-white mb-1 sm:mb-2">Boost Control</h3>
        <p className="text-xs sm:text-sm text-gray-400">Select your boost value for this turn</p>
      </div>

      {/* Boost Value Buttons Grid - Mobile-first responsive */}
      <div className="space-y-2 sm:space-y-3">
        <label className="block text-sm font-medium text-gray-200">Boost Value (0-4):</label>

        <div className="grid grid-cols-3 sm:grid-cols-6 gap-2 sm:gap-3">
          {boostValues.map((boost) => {
            const buttonState = getBoostButtonState(boost);
            const isDisabled = isInteractionDisabled || !buttonState.available;

            return (
              <div key={boost} className="relative">
                <button
                  onClick={() => handleBoostClick(boost)}
                  disabled={isDisabled}
                  className={`
                    w-full aspect-square rounded-lg border-2 font-bold text-base sm:text-lg 
                    transition-all duration-200 relative touch-manipulation
                    min-h-[48px] sm:min-h-[56px] lg:min-h-[64px]
                    ${
                      buttonState.selected
                        ? 'bg-blue-600 border-blue-400 text-white shadow-lg shadow-blue-500/40 scale-105 sm:scale-110 ring-2 ring-blue-300'
                        : buttonState.available && !isInteractionDisabled
                          ? 'bg-gray-700 border-gray-500 text-gray-200 hover:bg-gray-600 hover:border-gray-400 active:scale-95 hover:scale-105 hover:shadow-md'
                          : 'bg-gray-800 border-gray-600 text-gray-500 cursor-not-allowed opacity-60'
                    }
                  `}
                  aria-label={`Select boost value ${boost}`}
                  aria-pressed={buttonState.selected}
                  aria-disabled={isDisabled}
                >
                  {boost}
                </button>

                {/* "Used" indicator for unavailable boosts - Mobile responsive */}
                {buttonState.used && (
                  <div className="absolute -top-1 -right-1 bg-red-600 text-white text-[9px] sm:text-[10px] px-1 sm:px-1.5 py-0.5 rounded-full font-medium shadow-lg">
                    Used
                  </div>
                )}

                {/* Selection indicator */}
                {buttonState.selected && (
                  <div className="absolute -bottom-1 left-1/2 transform -translate-x-1/2">
                    <div className="w-1.5 h-1.5 sm:w-2 sm:h-2 bg-blue-400 rounded-full"></div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Selected Boost Display - Mobile responsive */}
      {selectedBoost !== null && (
        <div className="bg-gray-700 rounded-lg p-2 sm:p-3">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
            <span className="text-sm text-gray-300">Selected Boost:</span>
            <div className="flex items-center space-x-2">
              <span className="text-lg font-bold text-blue-400">{selectedBoost}</span>
              {availableBoosts.includes(selectedBoost) ? (
                <span className="text-green-400 text-xs">✓ Available</span>
              ) : (
                <span className="text-red-400 text-xs">✗ Not Available</span>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Selection Feedback Toast - Mobile responsive */}
      {selectionFeedback && (
        <div className="bg-blue-900/30 border border-blue-700 rounded-lg p-2 text-center animate-pulse">
          <p className="text-xs sm:text-sm text-blue-400">{selectionFeedback}</p>
        </div>
      )}

      {/* Available Boosts Info - Mobile responsive */}
      <div className="text-xs text-gray-500 bg-gray-700 rounded p-2">
        <div className="flex flex-col sm:flex-row sm:items-center gap-1">
          <span className="font-medium">Available boosts:</span>
          {availableBoosts.length > 0 ? (
            <span className="text-gray-400">{availableBoosts.join(', ')}</span>
          ) : (
            <span className="text-red-400">None available</span>
          )}
        </div>
      </div>

      {/* Validate Turn Button - Mobile responsive with touch-friendly sizing */}
      {!showConfirmation ? (
        <button
          onClick={handleValidateClick}
          disabled={!canValidateTurn}
          className={`
            w-full px-4 sm:px-6 py-3 sm:py-4 rounded-lg font-bold text-base sm:text-lg 
            transition-all duration-200 touch-manipulation
            flex items-center justify-center space-x-2 sm:space-x-3
            min-h-[48px] sm:min-h-[56px]
            ${
              canValidateTurn
                ? 'bg-green-600 hover:bg-green-700 active:bg-green-800 text-white shadow-lg hover:shadow-green-500/40 active:scale-95 hover:scale-105'
                : 'bg-gray-700 text-gray-500 cursor-not-allowed opacity-60'
            }
          `}
          aria-label="Validate turn with selected boost"
        >
          <span>Validate Turn</span>
          {selectedBoost !== null && <span>({selectedBoost})</span>}
          <span className="text-lg sm:text-xl">🚀</span>
        </button>
      ) : (
        /* Confirmation Dialog - Mobile responsive */
        <div className="space-y-3">
          <div className="bg-yellow-900/30 border border-yellow-700 rounded-lg p-3 sm:p-4">
            <div className="flex items-start space-x-2 sm:space-x-3">
              <span className="text-yellow-400 text-lg sm:text-xl flex-shrink-0">⚠️</span>
              <div className="min-w-0 flex-1">
                <p className="font-medium text-yellow-400 mb-1 text-sm sm:text-base">Confirm Turn Validation</p>
                <p className="text-xs sm:text-sm text-gray-300 mb-2">
                  You are about to validate your turn with boost value:{' '}
                  <span className="font-bold text-yellow-400">{selectedBoost}</span>
                </p>
                <p className="text-xs text-gray-400">
                  This action cannot be changed after submission.
                </p>
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
            <button
              onClick={handleCancelValidate}
              disabled={isSubmitting || isValidating}
              className="bg-gray-600 hover:bg-gray-700 active:bg-gray-800 disabled:bg-gray-700 disabled:cursor-not-allowed px-3 sm:px-4 py-3 rounded-lg font-medium transition-colors touch-manipulation min-h-[48px]"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirmValidate}
              disabled={isSubmitting || isValidating}
              className="bg-green-600 hover:bg-green-700 active:bg-green-800 disabled:bg-gray-600 disabled:cursor-not-allowed px-3 sm:px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2 touch-manipulation min-h-[48px]"
            >
              {isSubmitting || isValidating ? (
                <>
                  <div className="animate-spin rounded-full h-3 w-3 sm:h-4 sm:w-4 border-b-2 border-white"></div>
                  <span className="text-sm sm:text-base">Validating...</span>
                </>
              ) : (
                <span className="text-sm sm:text-base">Confirm</span>
              )}
            </button>
          </div>
        </div>
      )}

      {/* Turn Submitted State - Mobile responsive */}
      {hasSubmitted && (
        <div className="bg-green-900/30 border border-green-700 rounded-lg p-3 sm:p-4 text-center">
          <div className="text-green-400 text-2xl sm:text-3xl mb-2">✓</div>
          <p className="text-green-400 font-medium mb-1 text-sm sm:text-base">Turn Validated</p>
          <p className="text-gray-300 text-xs sm:text-sm mb-2">
            Boost value: <span className="font-medium text-green-400">{selectedBoost}</span>
          </p>
          <p className="text-gray-400 text-xs">Waiting for other players...</p>
        </div>
      )}

      {/* Turn Phase Status - Mobile responsive */}
      {turnPhase !== 'WaitingForPlayers' && !hasSubmitted && (
        <div className="bg-gray-700 border border-gray-600 rounded-lg p-3 text-center">
          <p className="text-gray-400 text-xs sm:text-sm">Turn actions not available</p>
          <p className="text-gray-500 text-xs mt-1">Current phase: {turnPhase}</p>
        </div>
      )}

      {/* Validation Error Message - Mobile responsive */}
      {validationError && (
        <div className="bg-red-900/30 border border-red-700 rounded-lg p-3">
          <div className="flex items-start space-x-2">
            <span className="text-red-400 text-base sm:text-lg flex-shrink-0">⚠️</span>
            <div className="min-w-0 flex-1">
              <p className="text-xs sm:text-sm font-medium text-red-400 mb-1">Validation Error</p>
              <p className="text-xs text-gray-400 break-words">{validationError}</p>
            </div>
          </div>
        </div>
      )}

      {/* Invalid Selection Warning - Mobile responsive */}
      {selectedBoost !== null &&
        !availableBoosts.includes(selectedBoost) &&
        !hasSubmitted &&
        !validationError && (
          <div className="bg-red-900/30 border border-red-700 rounded-lg p-3">
            <div className="flex items-start space-x-2">
              <span className="text-red-400 text-base sm:text-lg flex-shrink-0">⚠️</span>
              <div className="min-w-0 flex-1">
                <p className="text-xs sm:text-sm font-medium text-red-400 mb-1">Invalid Selection</p>
                <p className="text-xs text-gray-400">
                  This boost card has already been used in the current cycle
                </p>
              </div>
            </div>
          </div>
        )}
    </div>
  );
};

export default BoostControlPanel;
