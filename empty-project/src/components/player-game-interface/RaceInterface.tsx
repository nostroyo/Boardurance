/**
 * RaceInterface - Main race interface component
 *
 * This component can be lazy-loaded to improve performance.
 * It contains the main race UI components and interactions.
 *
 * Requirements: 12.5
 */

import React from 'react';
import type {
  CarData,
  PerformancePreview,
  TurnPhase,
  LocalView,
  BoostAvailability,
  LapHistory,
} from '../../types/race-api';
import { PerformancePreview as PerformancePreviewComponent } from './PerformancePreview';
import { PlayerCarCard } from './PlayerCarCard';
import { LocalSectorDisplay } from './LocalSectorDisplay';
import { RaceStatusPanel } from './RaceStatusPanel';
import { BoostSelector } from './BoostSelector';
import { RaceLoadingState, PollingIndicator } from './RaceLoadingState';

export interface RaceInterfaceProps {
  // Race data
  carData: CarData | null;
  performancePreview: PerformancePreview | null;
  turnPhase: TurnPhase | null;
  localView: LocalView | null;
  boostAvailability: BoostAvailability | null;
  lapHistory: LapHistory | null;

  // UI state
  selectedBoost: number | null;
  isSubmitting: boolean;
  hasSubmittedThisTurn: boolean;
  isPolling: boolean;

  // Loading states
  isLoadingPreview: boolean;
  isLoadingSubmit: boolean;
  isAnyLoading: boolean;

  // Event handlers
  onBoostSelect: (boost: number) => void;
  onSubmitAction: () => void;

  // Player info
  raceUuid: string;
  playerUuid: string;
}

export const RaceInterface: React.FC<RaceInterfaceProps> = React.memo(
  ({
    carData,
    performancePreview,
    turnPhase,
    localView,
    boostAvailability,
    lapHistory,
    selectedBoost,
    isSubmitting,
    hasSubmittedThisTurn,
    isPolling,
    isLoadingPreview,
    isLoadingSubmit,
    // isAnyLoading, // Currently unused but available for future enhancements
    onBoostSelect,
    onSubmitAction,
    raceUuid,
    playerUuid,
  }) => {
    return (
      <div className="min-h-screen bg-gray-900 text-white p-4">
        <div className="max-w-7xl mx-auto space-y-6">
          {/* Header */}
          <div className="text-center">
            <h1 className="text-3xl font-bold mb-2">🏁 Race in Progress</h1>
            <p className="text-gray-400">Race ID: {raceUuid.slice(-8)}</p>
          </div>

          {/* Race Status Panel */}
          {turnPhase && (
            <RaceStatusPanel
              currentLap={turnPhase.current_lap}
              totalLaps={10} // TODO: Get from race config
              lapCharacteristic={turnPhase.lap_characteristic}
              turnPhase={turnPhase}
              raceStatus="InProgress"
              hasSubmittedAction={hasSubmittedThisTurn}
            />
          )}

          {/* Main Race Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Left Column - Car Info and Performance */}
            <div className="space-y-6">
              {/* Player Car Card */}
              {carData && <PlayerCarCard carData={carData} lapHistory={lapHistory || undefined} />}

              {/* Performance Preview */}
              {isLoadingPreview ? (
                <div className="bg-gray-800 rounded-lg p-4">
                  <h2 className="text-xl font-semibold mb-4">Performance Preview</h2>
                  <RaceLoadingState type="data" message="Loading performance calculations..." />
                </div>
              ) : (
                performancePreview &&
                boostAvailability && (
                  <PerformancePreviewComponent
                    preview={performancePreview}
                    selectedBoost={selectedBoost}
                    onBoostSelect={onBoostSelect}
                    availableBoosts={boostAvailability.available_cards}
                  />
                )
              )}
            </div>

            {/* Center Column - Track View */}
            <div className="space-y-6">
              {/* Local Sector Display */}
              {localView && <LocalSectorDisplay localView={localView} playerUuid={playerUuid} />}
            </div>

            {/* Right Column - Actions and Status */}
            <div className="space-y-6">
              {/* Boost Selection and Submission */}
              <div className="bg-gray-800 rounded-lg p-4">
                <h2 className="text-xl font-semibold mb-4">Boost Selection</h2>

                {hasSubmittedThisTurn ? (
                  <div className="bg-green-900 border border-green-700 rounded p-4">
                    <p className="text-green-200 font-semibold">✓ Action Submitted</p>
                    <p className="text-green-300 text-sm mt-1">Waiting for turn to complete...</p>
                    <PollingIndicator isActive={isPolling} message="Processing turn..." />
                  </div>
                ) : isLoadingSubmit ? (
                  <div className="bg-blue-900 border border-blue-700 rounded p-4">
                    <RaceLoadingState type="action" message="Submitting your boost selection..." />
                  </div>
                ) : (
                  boostAvailability && (
                    <BoostSelector
                      selectedBoost={selectedBoost}
                      availableBoosts={boostAvailability.available_cards}
                      onBoostSelect={onBoostSelect}
                      onSubmit={onSubmitAction}
                      isSubmitting={isSubmitting}
                      hasSubmitted={hasSubmittedThisTurn}
                    />
                  )
                )}
              </div>

              {/* Boost Cycle Information */}
              {boostAvailability && (
                <div className="bg-gray-800 rounded-lg p-4">
                  <h3 className="text-lg font-semibold mb-3">Boost Cycle Status</h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-400">Current Cycle:</span>
                      <span className="text-white font-semibold">
                        {boostAvailability.current_cycle}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Cards Remaining:</span>
                      <span className="text-white font-semibold">
                        {boostAvailability.cards_remaining} / 5
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-400">Cycles Completed:</span>
                      <span className="text-white font-semibold">
                        {boostAvailability.cycles_completed}
                      </span>
                    </div>
                  </div>

                  {/* Progress Bar */}
                  <div className="mt-3">
                    <div className="flex justify-between text-xs text-gray-400 mb-1">
                      <span>Cycle Progress</span>
                      <span>{5 - boostAvailability.cards_remaining} / 5 used</span>
                    </div>
                    <div className="w-full bg-gray-600 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{
                          width: `${((5 - boostAvailability.cards_remaining) / 5) * 100}%`,
                        }}
                      />
                    </div>
                  </div>
                </div>
              )}

              {/* Turn Phase Status */}
              {turnPhase && (
                <div className="bg-gray-800 rounded-lg p-4">
                  <h3 className="text-lg font-semibold mb-3">Turn Status</h3>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <span className="text-gray-400">Phase:</span>
                      <span
                        className={`
                      px-3 py-1 rounded font-semibold text-sm
                      ${turnPhase.turn_phase === 'WaitingForPlayers' ? 'bg-yellow-900 text-yellow-200' : ''}
                      ${turnPhase.turn_phase === 'AllSubmitted' ? 'bg-blue-900 text-blue-200' : ''}
                      ${turnPhase.turn_phase === 'Processing' ? 'bg-orange-900 text-orange-200' : ''}
                      ${turnPhase.turn_phase === 'Complete' ? 'bg-green-900 text-green-200' : ''}
                    `}
                      >
                        {turnPhase.turn_phase}
                      </span>
                    </div>

                    <div className="text-sm text-gray-400">
                      <div>Active Players: {turnPhase.total_active_players}</div>
                      <div>Submitted: {turnPhase.submitted_players.length}</div>
                      <div>Pending: {turnPhase.pending_players.length}</div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Debug Info (Development Only) */}
          {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
            <div className="bg-gray-800 rounded-lg p-4 text-xs">
              <h3 className="text-sm font-semibold mb-2">Debug Info</h3>
              <div className="grid grid-cols-2 gap-4 text-gray-400">
                <div>
                  <div>Race UUID: {raceUuid}</div>
                  <div>Player UUID: {playerUuid.slice(-8)}</div>
                  <div>Selected Boost: {selectedBoost ?? 'None'}</div>
                </div>
                <div>
                  <div>Submitting: {isSubmitting ? 'Yes' : 'No'}</div>
                  <div>Submitted: {hasSubmittedThisTurn ? 'Yes' : 'No'}</div>
                  <div>Polling: {isPolling ? 'Yes' : 'No'}</div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    );
  },
);

export default RaceInterface;
