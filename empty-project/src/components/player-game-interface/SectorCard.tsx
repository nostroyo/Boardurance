import React from 'react';
import type { LocalView } from '../../types/race-api';
import { ParticipantList } from './ParticipantList';

export interface SectorCardProps {
  sector: LocalView['visible_sectors'][0];
  participants: LocalView['visible_participants'];
  isPlayerSector: boolean;
  playerUuid: string;
  position: 'above' | 'center' | 'below';
}

const SectorCardComponent: React.FC<SectorCardProps> = ({
  sector,
  participants,
  isPlayerSector,
  playerUuid,
  position,
}) => {
  // Get sector type styling with icons
  const getSectorTypeStyle = (): { bg: string; text: string; label: string; icon: string } => {
    switch (sector.sector_type) {
      case 'Start':
        return { bg: 'bg-green-600', text: 'text-white', label: 'Start', icon: '🏁' };
      case 'Finish':
        return { bg: 'bg-purple-600', text: 'text-white', label: 'Finish', icon: '🏆' };
      case 'Straight':
        return { bg: 'bg-blue-600', text: 'text-white', label: 'Straight', icon: '➡️' };
      case 'Curve':
        return { bg: 'bg-orange-600', text: 'text-white', label: 'Curve', icon: '🌀' };
      default:
        return { bg: 'bg-gray-600', text: 'text-white', label: 'Unknown', icon: '❓' };
    }
  };

  // Check if sector is at capacity
  const isAtCapacity = (): boolean => {
    if (sector.slot_capacity === null) return false;
    return sector.current_occupancy >= sector.slot_capacity;
  };

  // Get card styling based on player sector and position
  const getCardStyle = (): string => {
    if (isPlayerSector) {
      return 'bg-gradient-to-r from-blue-900 to-blue-800 border-blue-400 shadow-lg shadow-blue-500/30 ring-2 ring-blue-400/50';
    }

    // Gradient fade for non-player sectors
    if (position === 'above' || position === 'below') {
      return 'bg-gray-700 border-gray-600 hover:border-gray-500 transition-colors';
    }

    return 'bg-gray-700 border-gray-600 hover:border-gray-500 transition-colors';
  };

  const sectorTypeStyle = getSectorTypeStyle();

  return (
    <div
      className={`p-4 rounded-lg border transition-all duration-500 ease-in-out transform hover:shadow-lg ${getCardStyle()}`}
      role="region"
      aria-label={`Sector ${sector.id}: ${sector.name}`}
    >
      {/* Sector header */}
      <div className="flex justify-between items-center mb-3">
        <div className="flex items-center space-x-2">
          <h3 className="font-medium text-lg text-white">
            Sector {sector.id}: {sector.name}
          </h3>
          {isPlayerSector && (
            <span className="bg-blue-500 text-white text-xs px-3 py-1 rounded-full font-bold animate-pulse">
              🎯 YOUR SECTOR
            </span>
          )}
        </div>
        <div className="flex items-center space-x-2">
          <span
            className={`text-sm px-3 py-1 rounded-full flex items-center space-x-1 ${sectorTypeStyle.bg} ${sectorTypeStyle.text}`}
          >
            <span>{sectorTypeStyle.icon}</span>
            <span>{sectorTypeStyle.label}</span>
          </span>
        </div>
      </div>

      {/* Sector information grid */}
      <div className="grid grid-cols-2 gap-4 text-sm text-gray-300 mb-3">
        {/* Value range */}
        <div>
          <span className="text-gray-400 block mb-1">Value Range:</span>
          <p className="font-medium text-white">
            {sector.min_value} - {sector.max_value}
          </p>
        </div>

        {/* Capacity with visual indicator */}
        {sector.slot_capacity !== null && (
          <div>
            <span className="text-gray-400 block mb-1">Capacity:</span>
            <div className="flex items-center space-x-2">
              <p className="font-medium text-white">
                {sector.current_occupancy} / {sector.slot_capacity}
              </p>
              {/* Capacity bar */}
              <div className="flex-1 bg-gray-600 rounded-full h-2 min-w-[60px] overflow-hidden">
                <div
                  className={`h-2 rounded-full transition-all duration-700 ease-out ${
                    isAtCapacity() ? 'bg-red-500 animate-pulse' : 'bg-blue-500'
                  }`}
                  style={{
                    width: `${Math.min((sector.current_occupancy / sector.slot_capacity) * 100, 100)}%`,
                  }}
                />
              </div>
              {isAtCapacity() && <span className="text-red-400 text-xs font-bold">FULL</span>}
            </div>
          </div>
        )}

        {/* If no capacity limit */}
        {sector.slot_capacity === null && (
          <div>
            <span className="text-gray-400 block mb-1">Capacity:</span>
            <p className="font-medium text-white flex items-center space-x-1">
              <span>{sector.current_occupancy}</span>
              <span className="text-green-400 text-xs">∞ Unlimited</span>
            </p>
          </div>
        )}
      </div>

      {/* Participants list */}
      <ParticipantList participants={participants} playerUuid={playerUuid} />

      {/* Capacity warning */}
      {isAtCapacity() && (
        <div className="mt-3 bg-red-600 bg-opacity-20 border border-red-500 rounded p-2">
          <p className="text-red-300 text-xs">⚠️ This sector is at maximum capacity</p>
        </div>
      )}
    </div>
  );
};

export const SectorCard = React.memo(SectorCardComponent);
