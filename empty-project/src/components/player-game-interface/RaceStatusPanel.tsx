import React, { useEffect, useRef } from 'react';
import type { Race, TurnPhase as OldTurnPhase } from '../../types/race';
import type { TurnPhase, TurnPhaseStatus, LapCharacteristic } from '../../types/race-api';

export interface RaceStatusPanelProps {
  race?: Race; // Legacy race object (optional for backward compatibility)
  turnPhase?: TurnPhase; // New backend API turn phase
  currentTurnPhase?: OldTurnPhase; // Legacy turn phase (optional for backward compatibility)
  hasSubmittedAction: boolean;
  timeRemaining?: number;
  // New props for backend API integration
  currentLap?: number;
  totalLaps?: number;
  lapCharacteristic?: LapCharacteristic;
  raceStatus?: 'NotStarted' | 'InProgress' | 'Completed';
  // Toast notification callbacks (Requirements 8.4)
  onPhaseChange?: (phase: TurnPhaseStatus) => void;
  onLapComplete?: (lap: number) => void;
  onActionSubmitted?: () => void;
}

export const RaceStatusPanel: React.FC<RaceStatusPanelProps> = ({
  race,
  turnPhase,
  currentTurnPhase,
  hasSubmittedAction,
  timeRemaining,
  currentLap,
  totalLaps,
  lapCharacteristic,
  raceStatus,
  onPhaseChange,
  onLapComplete,
  onActionSubmitted,
}) => {
  // Determine which data source to use (new API or legacy)
  const effectiveTurnPhase: TurnPhaseStatus =
    turnPhase?.turn_phase || currentTurnPhase || 'WaitingForPlayers';
  const effectiveCurrentLap = currentLap ?? turnPhase?.current_lap ?? race?.current_lap ?? 1;
  const effectiveTotalLaps = totalLaps ?? race?.total_laps ?? 1;
  const effectiveLapCharacteristic =
    lapCharacteristic ?? turnPhase?.lap_characteristic ?? race?.lap_characteristic ?? 'Straight';
  const effectiveRaceStatus =
    raceStatus ??
    (race?.status === 'Finished'
      ? 'Completed'
      : race?.status === 'InProgress'
        ? 'InProgress'
        : 'NotStarted');

  // Track previous values for change detection (Requirements 8.4)
  const prevTurnPhaseRef = useRef<TurnPhaseStatus>(effectiveTurnPhase);
  const prevLapRef = useRef<number>(effectiveCurrentLap);
  const prevSubmittedRef = useRef<boolean>(hasSubmittedAction);

  // Detect turn phase changes and trigger notification (Requirements 8.4)
  useEffect(() => {
    if (prevTurnPhaseRef.current !== effectiveTurnPhase) {
      if (onPhaseChange) {
        onPhaseChange(effectiveTurnPhase);
      }
      prevTurnPhaseRef.current = effectiveTurnPhase;
    }
  }, [effectiveTurnPhase, onPhaseChange]);

  // Detect lap completion and trigger notification (Requirements 8.4)
  useEffect(() => {
    if (prevLapRef.current !== effectiveCurrentLap && prevLapRef.current > 0) {
      if (onLapComplete) {
        onLapComplete(effectiveCurrentLap);
      }
    }
    prevLapRef.current = effectiveCurrentLap;
  }, [effectiveCurrentLap, onLapComplete]);

  // Detect action submission and trigger notification (Requirements 8.4)
  useEffect(() => {
    if (!prevSubmittedRef.current && hasSubmittedAction) {
      if (onActionSubmitted) {
        onActionSubmitted();
      }
    }
    prevSubmittedRef.current = hasSubmittedAction;
  }, [hasSubmittedAction, onActionSubmitted]);

  // Calculate lap progress percentage
  const getLapProgress = (): number => {
    if (effectiveTotalLaps === 0) return 0;
    return (effectiveCurrentLap / effectiveTotalLaps) * 100;
  };

  // Get turn phase color for indicator (Requirements 8.1)
  const getTurnPhaseColor = (): string => {
    switch (effectiveTurnPhase) {
      case 'WaitingForPlayers':
        return '#f59e0b'; // yellow/amber
      case 'AllSubmitted':
        return '#3b82f6'; // blue
      case 'Processing':
        return '#f97316'; // orange
      case 'Complete':
        return '#10b981'; // green
      default:
        return '#6b7280'; // gray
    }
  };

  // Get turn phase icon indicator (Requirements 8.1)
  const getTurnPhaseIcon = (): string => {
    switch (effectiveTurnPhase) {
      case 'WaitingForPlayers':
        return hasSubmittedAction ? '✓' : '⏳';
      case 'AllSubmitted':
        return '📋';
      case 'Processing':
        return '⚙️';
      case 'Complete':
        return '✅';
      default:
        return '❓';
    }
  };

  // Get turn phase description text (Requirements 8.1)
  const getTurnPhaseDescription = (): string => {
    switch (effectiveTurnPhase) {
      case 'WaitingForPlayers':
        return hasSubmittedAction ? 'Waiting for other players' : 'Waiting for player actions';
      case 'AllSubmitted':
        return 'All players submitted';
      case 'Processing':
        return 'Processing turn';
      case 'Complete':
        return 'Turn complete';
      default:
        return 'Unknown phase';
    }
  };

  // Get status message based on turn phase
  const getStatusMessage = (): string => {
    if (effectiveRaceStatus === 'Completed') {
      return 'Race Finished';
    }
    if (effectiveRaceStatus === 'NotStarted') {
      return 'Race Starting Soon';
    }

    switch (effectiveTurnPhase) {
      case 'WaitingForPlayers':
        return hasSubmittedAction ? 'Waiting for other players' : 'Submit your boost action';
      case 'AllSubmitted':
        return 'All players submitted - Processing soon';
      case 'Processing':
        return 'Processing turn results';
      case 'Complete':
        return 'Turn complete - Next turn starting';
      default:
        return 'Unknown status';
    }
  };

  // Get lap characteristic icon (Requirements 1.2)
  const getLapCharacteristicIcon = (): string => {
    return effectiveLapCharacteristic === 'Straight' ? '🏁' : '🌀';
  };

  // Check if this is the final lap
  const isFinalLap = (): boolean => {
    return effectiveCurrentLap === effectiveTotalLaps;
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700 shadow-lg">
      {/* Header with race name and status indicator */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-white">Race Status</h2>
        <div className="flex items-center space-x-2">
          <div
            className="w-3 h-3 rounded-full animate-pulse"
            style={{ backgroundColor: getTurnPhaseColor() }}
            aria-label={`Turn phase: ${effectiveTurnPhase}`}
          ></div>
          <span className="text-sm font-medium text-gray-200">{getStatusMessage()}</span>
        </div>
      </div>

      {/* Turn Phase Status Display (Requirements 1.5, 8.1) */}
      <div
        className="mb-4 p-3 rounded-lg border"
        style={{
          backgroundColor: `${getTurnPhaseColor()}20`,
          borderColor: getTurnPhaseColor(),
        }}
      >
        <div className="flex items-center space-x-3">
          <span className="text-2xl" role="img" aria-label="Turn phase icon">
            {getTurnPhaseIcon()}
          </span>
          <div className="flex-1">
            <div className="flex items-center space-x-2">
              <span className="font-semibold text-white">Turn Phase:</span>
              <span className="font-medium" style={{ color: getTurnPhaseColor() }}>
                {effectiveTurnPhase}
              </span>
            </div>
            <p className="text-sm text-gray-300 mt-1">{getTurnPhaseDescription()}</p>
          </div>
        </div>
      </div>

      {/* Main status grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        {/* Race name (if available) */}
        {race?.name && (
          <div>
            <span className="text-gray-400 block mb-1">Race:</span>
            <p className="font-medium text-white truncate" title={race.name}>
              {race.name}
            </p>
          </div>
        )}

        {/* Current lap / total laps (Requirements 1.2) */}
        <div>
          <span className="text-gray-400 block mb-1">Lap:</span>
          <p className="font-medium text-white text-lg">
            {effectiveCurrentLap} / {effectiveTotalLaps}
            {isFinalLap() && (
              <span className="text-yellow-400 ml-1" title="Final lap!">
                🏁
              </span>
            )}
          </p>
        </div>

        {/* Lap characteristic with icon (Requirements 1.2) */}
        <div>
          <span className="text-gray-400 block mb-1">Characteristic:</span>
          <p className="font-medium text-white">
            <span className="mr-1 text-xl">{getLapCharacteristicIcon()}</span>
            {effectiveLapCharacteristic}
          </p>
        </div>

        {/* Race status */}
        <div>
          <span className="text-gray-400 block mb-1">Race Status:</span>
          <p
            className={`font-medium ${
              effectiveRaceStatus === 'InProgress'
                ? 'text-green-400'
                : effectiveRaceStatus === 'Completed'
                  ? 'text-blue-400'
                  : effectiveRaceStatus === 'NotStarted'
                    ? 'text-yellow-400'
                    : 'text-red-400'
            }`}
          >
            {effectiveRaceStatus}
          </p>
        </div>
      </div>

      {/* Lap Progress Bar (Requirements 1.2) */}
      <div className="mt-4">
        <div className="flex items-center justify-between mb-2">
          <span className="text-sm text-gray-400">Lap Progress:</span>
          <span className="text-sm font-medium text-gray-300">{Math.round(getLapProgress())}%</span>
        </div>
        <div className="bg-gray-700 rounded-full h-3 overflow-hidden">
          <div
            className="bg-gradient-to-r from-blue-500 to-blue-400 h-3 rounded-full transition-all duration-500 ease-out"
            style={{ width: `${getLapProgress()}%` }}
            role="progressbar"
            aria-valuenow={getLapProgress()}
            aria-valuemin={0}
            aria-valuemax={100}
          ></div>
        </div>
      </div>

      {/* Time remaining (if provided) */}
      {timeRemaining !== undefined && timeRemaining > 0 && (
        <div className="mt-4 pt-4 border-t border-gray-700">
          <div className="flex items-center justify-between">
            <span className="text-sm text-gray-400">Time Remaining:</span>
            <span
              className={`text-lg font-bold ${
                timeRemaining < 10 ? 'text-red-400 animate-pulse' : 'text-blue-400'
              }`}
            >
              {timeRemaining}s
            </span>
          </div>
        </div>
      )}

      {/* Notification banner for action required */}
      {effectiveRaceStatus === 'InProgress' &&
        effectiveTurnPhase === 'WaitingForPlayers' &&
        !hasSubmittedAction && (
          <div className="mt-4 bg-yellow-600 bg-opacity-20 border border-yellow-500 rounded-lg p-3">
            <div className="flex items-center space-x-2">
              <span className="text-yellow-400 text-xl">⚠️</span>
              <div className="flex-1">
                <p className="text-yellow-200 font-medium text-sm">Action Required</p>
                <p className="text-yellow-300 text-xs">
                  Submit your boost value to continue the race
                </p>
              </div>
            </div>
          </div>
        )}

      {/* Processing notification */}
      {effectiveTurnPhase === 'Processing' && (
        <div className="mt-4 bg-orange-600 bg-opacity-20 border border-orange-500 rounded-lg p-3">
          <div className="flex items-center space-x-2">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-orange-400"></div>
            <p className="text-orange-200 text-sm">Processing lap results...</p>
          </div>
        </div>
      )}

      {/* Race finished notification */}
      {effectiveRaceStatus === 'Completed' && (
        <div className="mt-4 bg-blue-600 bg-opacity-20 border border-blue-500 rounded-lg p-3">
          <div className="flex items-center space-x-2">
            <span className="text-blue-400 text-xl">🏁</span>
            <p className="text-blue-200 font-medium text-sm">
              Race has finished! Check your final position below.
            </p>
          </div>
        </div>
      )}
    </div>
  );
};
