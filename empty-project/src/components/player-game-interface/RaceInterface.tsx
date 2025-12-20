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
import { TrackDisplayRedesign } from './TrackDisplayRedesign';
import { RaceStatusPanel } from './RaceStatusPanel';
import { BoostControlPanel } from './BoostControlPanel';
import { RaceLoadingState, PollingIndicator } from './RaceLoadingState';
import DiagnosticPanel from '../DiagnosticPanel';

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

          {/* Simplified Layout - Track Display and Boost Controls */}
          <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
            {/* Track Display - Takes up 3/4 of the width */}
            <div className="lg:col-span-3 space-y-6">
              {/* Redesigned Track Display */}
              {localView ? (
                <TrackDisplayRedesign
                  localView={localView}
                  playerUuid={playerUuid}
                  animationState={undefined} // TODO: Add animation state support
                  onSectorClick={(sectorId) => {
                    console.log('Sector clicked:', sectorId);
                    // Future enhancement: show sector details
                  }}
                  onSlotClick={(sectorId, slotNumber) => {
                    console.log('Slot clicked:', sectorId, slotNumber);
                    // Future enhancement: show participant details
                  }}
                />
              ) : (
                <div className="bg-gray-800 rounded-lg p-8 text-center">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                  <p className="text-gray-400">Loading track view...</p>
                </div>
              )}
            </div>

            {/* Boost Controls - Takes up 1/4 of the width */}
            <div className="lg:col-span-1 space-y-6">
              {/* Player Car Info */}
              {carData && (
                <div className="bg-gray-800 rounded-lg p-4">
                  <h3 className="text-lg font-semibold mb-3">Your Car</h3>
                  <div className="space-y-2 text-sm">
                    <div><span className="text-gray-400">Car:</span> {carData.car.name}</div>
                    <div><span className="text-gray-400">Pilot:</span> {carData.pilot.name}</div>
                    <div><span className="text-gray-400">Engine:</span> {carData.engine.name}</div>
                    <div><span className="text-gray-400">Body:</span> {carData.body.name}</div>
                  </div>
                </div>
              )}

              {/* Redesigned Boost Control Panel */}
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
                  boostAvailability &&
                  turnPhase && (
                    <BoostControlPanel
                      selectedBoost={selectedBoost}
                      availableBoosts={boostAvailability.available_cards}
                      onBoostSelect={onBoostSelect}
                      onValidateTurn={onSubmitAction}
                      isSubmitting={isSubmitting}
                      hasSubmitted={hasSubmittedThisTurn}
                      turnPhase={turnPhase.turn_phase}
                    />
                  )
                )}
              </div>

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
                      <div>Lap: {turnPhase.current_lap}</div>
                      <div>Characteristic: {turnPhase.lap_characteristic}</div>
                      <div>Active Players: {turnPhase.total_active_players}</div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Debug Info (Development Only) */}
          {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
            <div className="bg-gray-800 rounded-lg p-4 text-xs">
              <h3 className="text-sm font-semibold mb-2 text-yellow-400">Debug Info - RaceInterface</h3>
              <div className="grid grid-cols-2 gap-4 text-gray-400">
                <div>
                  <div>Race UUID: {raceUuid}</div>
                  <div>Player UUID: {playerUuid.slice(-8)}</div>
                  <div>Selected Boost: {selectedBoost ?? 'None'}</div>
                  <div>Car Data: {carData ? '✓ Loaded' : '✗ Missing'}</div>
                  <div>Local View: {localView ? '✓ Loaded' : '✗ Missing'}</div>
                  <div>Boost Availability: {boostAvailability ? '✓ Loaded' : '✗ Missing'}</div>
                </div>
                <div>
                  <div>Turn Phase: {turnPhase ? '✓ Loaded' : '✗ Missing'}</div>
                  <div>Submitting: {isSubmitting ? 'Yes' : 'No'}</div>
                  <div>Submitted: {hasSubmittedThisTurn ? 'Yes' : 'No'}</div>
                  <div>Polling: {isPolling ? 'Yes' : 'No'}</div>
                  {localView && (
                    <>
                      <div>Visible Sectors: {localView.visible_sectors.length}</div>
                      <div>Visible Participants: {localView.visible_participants.length}</div>
                    </>
                  )}
                </div>
              </div>
              {localView && localView.visible_participants.length > 0 && (
                <div className="mt-2 pt-2 border-t border-gray-600">
                  <div className="text-yellow-400 font-semibold mb-1">Participants:</div>
                  {localView.visible_participants.map((p, i) => (
                    <div key={i} className="text-xs">
                      {p.player_name} - Sector {p.current_sector}, Pos {p.position_in_sector}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Floating Diagnostic Panel */}
          {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
            <DiagnosticPanel
              localView={localView}
              boostAvailability={boostAvailability}
              turnPhase={turnPhase}
              carData={carData}
              selectedBoost={selectedBoost}
              raceUuid={raceUuid}
              playerUuid={playerUuid}
            />
          )}
        </div>
      </div>
    );
  },
);

export default RaceInterface;
