/**
 * DiagnosticPanel - Debug component to help identify race interface issues
 */

import React from 'react';
import type { LocalView, BoostAvailability, TurnPhase, CarData } from '../types/race-api';

interface DiagnosticPanelProps {
  localView: LocalView | null;
  boostAvailability: BoostAvailability | null;
  turnPhase: TurnPhase | null;
  carData: CarData | null;
  selectedBoost: number | null;
  raceUuid: string;
  playerUuid: string;
}

export const DiagnosticPanel: React.FC<DiagnosticPanelProps> = ({
  localView,
  boostAvailability,
  turnPhase,
  carData,
  selectedBoost,
  raceUuid,
  playerUuid,
}) => {
  const getDataStatus = (data: any, name: string) => {
    if (data === null || data === undefined) {
      return { status: '❌', color: 'text-red-400', message: `${name}: NULL/UNDEFINED` };
    }
    if (typeof data === 'object' && Object.keys(data).length === 0) {
      return { status: '⚠️', color: 'text-yellow-400', message: `${name}: EMPTY OBJECT` };
    }
    return { status: '✅', color: 'text-green-400', message: `${name}: LOADED` };
  };

  const localViewStatus = getDataStatus(localView, 'LocalView');
  const boostStatus = getDataStatus(boostAvailability, 'BoostAvailability');
  const turnPhaseStatus = getDataStatus(turnPhase, 'TurnPhase');
  const carDataStatus = getDataStatus(carData, 'CarData');

  return (
    <div className="fixed bottom-4 right-4 bg-black bg-opacity-90 text-white p-4 rounded-lg border border-gray-600 max-w-md z-50 text-xs">
      <h3 className="font-bold mb-2 text-yellow-400">🔍 DIAGNOSTIC PANEL</h3>
      
      {/* Data Status */}
      <div className="space-y-1 mb-3">
        <div className={localViewStatus.color}>{localViewStatus.status} {localViewStatus.message}</div>
        <div className={boostStatus.color}>{boostStatus.status} {boostStatus.message}</div>
        <div className={turnPhaseStatus.color}>{turnPhaseStatus.status} {turnPhaseStatus.message}</div>
        <div className={carDataStatus.color}>{carDataStatus.status} {carDataStatus.message}</div>
      </div>

      {/* Race Info */}
      <div className="border-t border-gray-600 pt-2 mb-3">
        <div className="text-gray-300">Race: {raceUuid.slice(-8)}</div>
        <div className="text-gray-300">Player: {playerUuid.slice(-8)}</div>
        <div className="text-gray-300">Selected Boost: {selectedBoost ?? 'None'}</div>
      </div>

      {/* Participants Count */}
      {localView && (
        <div className="border-t border-gray-600 pt-2 mb-3">
          <div className="text-blue-400">Participants: {localView.visible_participants.length}</div>
          <div className="text-blue-400">Sectors: {localView.visible_sectors.length}</div>
          <div className="text-blue-400">Center Sector: {localView.center_sector}</div>
        </div>
      )}

      {/* Boost Info */}
      {boostAvailability && (
        <div className="border-t border-gray-600 pt-2 mb-3">
          <div className="text-purple-400">Available Boosts: {boostAvailability.available_cards.join(', ')}</div>
          <div className="text-purple-400">Cards Remaining: {boostAvailability.cards_remaining}</div>
        </div>
      )}

      {/* Component Visibility Test */}
      <div className="border-t border-gray-600 pt-2">
        <div className="text-yellow-400 font-bold mb-1">Visibility Test:</div>
        <div className="bg-red-600 w-4 h-4 inline-block mr-2"></div>
        <span className="text-red-400">Red Square (Should be visible)</span>
        <br />
        <div className="bg-blue-600 w-4 h-4 inline-block mr-2 mt-1"></div>
        <span className="text-blue-400">Blue Square (Should be visible)</span>
      </div>
    </div>
  );
};

export default DiagnosticPanel;