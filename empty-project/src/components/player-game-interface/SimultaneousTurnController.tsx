import React, { useState, useEffect } from 'react';
import type { SimultaneousTurnControllerProps } from '../../types';
import { BoostSelector } from './BoostSelector';

/**
 * SimultaneousTurnController Component
 *
 * Manages player action submission during turn phases with boost selection,
 * confirmation, loading states, and error handling.
 *
 * Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 6.1, 6.5
 */
export const SimultaneousTurnController: React.FC<SimultaneousTurnControllerProps> = ({
  currentTurnPhase,
  selectedBoost,
  hasSubmitted,
  onBoostSelect,
  onSubmitAction,
  timeRemaining,
}) => {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [showConfirmation, setShowConfirmation] = useState(false);

  // Reset states when turn phase changes
  useEffect(() => {
    if (currentTurnPhase === 'WaitingForPlayers' && hasSubmitted) {
      setIsSubmitting(false);
      setSubmitError(null);
      setShowConfirmation(false);
    }
  }, [currentTurnPhase, hasSubmitted]);

  // Handle boost submission with error handling
  const handleSubmit = async () => {
    if (isSubmitting || hasSubmitted) return;

    setIsSubmitting(true);
    setSubmitError(null);

    try {
      await onSubmitAction();
      setShowConfirmation(false);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to submit action';
      setSubmitError(errorMessage);
    } finally {
      setIsSubmitting(false);
    }
  };

  // Handle retry after error
  const handleRetry = () => {
    setSubmitError(null);
    handleSubmit();
  };

  // Format time remaining for display
  const formatTimeRemaining = (seconds?: number): string => {
    if (seconds === undefined) return '';

    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  // Determine if actions can be submitted
  const canSubmitActions = currentTurnPhase === 'WaitingForPlayers' && !hasSubmitted;

  // Render turn phase status indicator
  const renderTurnPhaseStatus = () => {
    const phaseConfig = {
      WaitingForPlayers: {
        color: 'text-green-400',
        bgColor: 'bg-green-900/20',
        borderColor: 'border-green-800',
        icon: '⏳',
        label: 'Waiting for Players',
      },
      AllSubmitted: {
        color: 'text-yellow-400',
        bgColor: 'bg-yellow-900/20',
        borderColor: 'border-yellow-800',
        icon: '✓',
        label: 'All Submitted',
      },
      Processing: {
        color: 'text-blue-400',
        bgColor: 'bg-blue-900/20',
        borderColor: 'border-blue-800',
        icon: '⚙️',
        label: 'Processing Turn',
      },
      Complete: {
        color: 'text-purple-400',
        bgColor: 'bg-purple-900/20',
        borderColor: 'border-purple-800',
        icon: '🏁',
        label: 'Turn Complete',
      },
    };

    const config = phaseConfig[currentTurnPhase];

    return (
      <div
        className={`flex items-center justify-between p-3 rounded-lg border ${config.bgColor} ${config.borderColor}`}
      >
        <div className="flex items-center space-x-2">
          <span className="text-xl">{config.icon}</span>
          <span className={`font-medium ${config.color}`}>{config.label}</span>
        </div>
        {timeRemaining !== undefined && timeRemaining > 0 && (
          <div className="text-sm text-gray-400">
            Time:{' '}
            <span className="font-mono font-medium">{formatTimeRemaining(timeRemaining)}</span>
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="space-y-4">
      {/* Turn Phase Status Display */}
      {renderTurnPhaseStatus()}

      {/* Action Submission Interface */}
      {canSubmitActions ? (
        <div className="space-y-4">
          {/* Boost Selector */}
          <BoostSelector
            selectedBoost={selectedBoost}
            onBoostSelect={onBoostSelect}
            disabled={isSubmitting || hasSubmitted}
            showPreview={true}
            previewValue={selectedBoost}
          />

          {/* Submit Button */}
          {!showConfirmation ? (
            <button
              onClick={() => setShowConfirmation(true)}
              disabled={isSubmitting || hasSubmitted}
              className="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-4 py-3 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2"
            >
              <span>Submit Boost ({selectedBoost})</span>
              <span>🚀</span>
            </button>
          ) : (
            <div className="space-y-2">
              <div className="bg-yellow-900/20 border border-yellow-800 rounded-lg p-3 text-sm text-yellow-400">
                <p className="font-medium mb-1">Confirm Action</p>
                <p className="text-xs text-gray-400">
                  You are about to submit boost value:{' '}
                  <span className="font-bold text-yellow-400">{selectedBoost}</span>
                </p>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <button
                  onClick={() => setShowConfirmation(false)}
                  disabled={isSubmitting}
                  className="bg-gray-600 hover:bg-gray-700 disabled:bg-gray-700 disabled:cursor-not-allowed px-4 py-2 rounded-lg font-medium transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSubmit}
                  disabled={isSubmitting}
                  className="bg-green-600 hover:bg-green-700 disabled:bg-gray-600 disabled:cursor-not-allowed px-4 py-2 rounded-lg font-medium transition-colors flex items-center justify-center space-x-2"
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

          {/* Error Display with Retry */}
          {submitError && (
            <div className="bg-red-900/20 border border-red-800 rounded-lg p-3">
              <div className="flex items-start space-x-2">
                <span className="text-red-400 text-lg">⚠️</span>
                <div className="flex-1">
                  <p className="text-sm font-medium text-red-400 mb-1">Submission Failed</p>
                  <p className="text-xs text-gray-400 mb-2">{submitError}</p>
                  <button
                    onClick={handleRetry}
                    disabled={isSubmitting}
                    className="text-xs bg-red-700 hover:bg-red-800 disabled:bg-gray-700 disabled:cursor-not-allowed px-3 py-1 rounded transition-colors"
                  >
                    {isSubmitting ? 'Retrying...' : 'Retry'}
                  </button>
                </div>
              </div>
            </div>
          )}

          {/* Submission Guidelines */}
          <div className="text-xs text-gray-500 space-y-1">
            <p>• Select your boost value carefully - it cannot be changed after submission</p>
            <p>• Higher boost values provide more performance but use limited resources</p>
            <p>• Wait for all players to submit before turn processing begins</p>
          </div>
        </div>
      ) : hasSubmitted ? (
        /* Submitted State */
        <div className="text-center py-6">
          <div className="text-green-400 text-4xl mb-3">✓</div>
          <p className="text-green-400 font-medium text-lg mb-2">Action Submitted Successfully</p>
          <p className="text-gray-400 text-sm mb-3">
            Boost value: <span className="font-medium text-green-400">{selectedBoost}</span>
          </p>
          <div className="bg-gray-700 rounded-lg p-3">
            <p className="text-gray-300 text-sm">
              Waiting for other players to submit their actions...
            </p>
            {timeRemaining !== undefined && timeRemaining > 0 && (
              <p className="text-gray-400 text-xs mt-2">
                Time remaining: {formatTimeRemaining(timeRemaining)}
              </p>
            )}
          </div>
        </div>
      ) : currentTurnPhase === 'Processing' ? (
        /* Processing State */
        <div className="text-center py-6">
          <div className="text-blue-400 text-4xl mb-3">⚙️</div>
          <p className="text-blue-400 font-medium text-lg mb-2">Processing Turn</p>
          <p className="text-gray-400 text-sm mb-3">Calculating race results...</p>
          <div className="w-32 bg-gray-700 rounded-full h-2 mx-auto">
            <div className="bg-blue-500 h-2 rounded-full animate-pulse"></div>
          </div>
        </div>
      ) : currentTurnPhase === 'AllSubmitted' ? (
        /* All Submitted State */
        <div className="text-center py-6">
          <div className="text-yellow-400 text-4xl mb-3">✓</div>
          <p className="text-yellow-400 font-medium text-lg mb-2">All Actions Submitted</p>
          <p className="text-gray-400 text-sm">Turn processing will begin shortly...</p>
        </div>
      ) : (
        /* Other States */
        <div className="text-center py-6">
          <div className="text-gray-400 text-4xl mb-3">⏸️</div>
          <p className="text-gray-400 font-medium">Turn actions not available</p>
          <p className="text-gray-500 text-sm mt-2">Current phase: {currentTurnPhase}</p>
        </div>
      )}
    </div>
  );
};

export default SimultaneousTurnController;
