import React, { useMemo } from 'react';
import type { LocalView } from '../../types/race-api';
import { PositionSlot } from './PositionSlot';

export interface SectorGridProps {
  sector: LocalView['visible_sectors'][0];
  participants: LocalView['visible_participants'];
  isPlayerSector: boolean;
  playerUuid: string;
  onSectorClick?: (sectorId: number) => void;
  onSlotClick?: (sectorId: number, slotNumber: number) => void;
  animationState?: 'idle' | 'moving' | 'highlighted';
}

const SectorGridComponent: React.FC<SectorGridProps> = ({
  sector,
  participants,
  isPlayerSector,
  playerUuid,
  onSectorClick,
  onSlotClick,
  animationState = 'idle',
}) => {
  // Get participants in this sector sorted by position
  const sectorParticipants = useMemo(() => {
    return participants
      .filter((p) => p.current_sector === sector.id)
      .sort((a, b) => a.position_in_sector - b.position_in_sector);
  }, [participants, sector.id]);

  // Create position slots (1-5 numbered slots)
  const positionSlots = useMemo(() => {
    const slots = [];
    const maxSlots = 5; // Fixed 5 slots per sector as per requirements

    for (let i = 1; i <= maxSlots; i++) {
      const participant = sectorParticipants.find((p) => p.position_in_sector === i);
      const isPlayerSlot = participant?.player_uuid === playerUuid;

      slots.push({
        slotNumber: i,
        participant,
        isOccupied: !!participant,
        isPlayerSlot,
      });
    }

    return slots;
  }, [sectorParticipants, playerUuid]);

  // Get sector container styling
  const getSectorContainerStyle = (): string => {
    const baseStyle =
      'p-4 rounded-lg border transition-all duration-300 cursor-pointer hover:shadow-lg';

    if (isPlayerSector) {
      return `${baseStyle} bg-gradient-to-r from-blue-900 to-blue-800 border-blue-400 shadow-lg shadow-blue-500/30 ring-2 ring-blue-400/50`;
    }

    return `${baseStyle} bg-gray-800 border-gray-700 hover:border-gray-600`;
  };

  // Check if sector is at capacity
  const isAtCapacity = (): boolean => {
    if (sector.slot_capacity === null) return false;
    return sector.current_occupancy >= sector.slot_capacity;
  };

  return (
    <div
      className={getSectorContainerStyle()}
      onClick={() => onSectorClick?.(sector.id)}
      role="region"
      aria-label={`Sector ${sector.id}: ${sector.name}`}
    >
      {/* Sector Header - Mobile responsive */}
      <div className="flex flex-col sm:flex-row sm:justify-between sm:items-center mb-3 sm:mb-4 gap-2">
        <div className="flex items-center space-x-2 sm:space-x-3">
          <h3 className="font-bold text-base sm:text-lg text-white">Sector {sector.id}</h3>
          <span className="text-gray-300 text-sm truncate">{sector.name}</span>
          {isPlayerSector && (
            <span className="bg-blue-500 text-white text-xs px-2 py-1 rounded-full font-bold animate-pulse whitespace-nowrap">
              🎯 YOU
            </span>
          )}
        </div>
      </div>

      {/* Sector Information - Mobile-first grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-4 text-sm text-gray-300 mb-3 sm:mb-4">
        {/* Value Range */}
        <div className="bg-gray-700/50 p-2 sm:p-3 rounded">
          <span className="text-gray-400 block mb-1 text-xs sm:text-sm">Value Range:</span>
          <p className="font-medium text-white text-sm sm:text-base">
            {sector.min_value} - {sector.max_value}
          </p>
        </div>

        {/* Capacity */}
        <div className="bg-gray-700/50 p-2 sm:p-3 rounded">
          <span className="text-gray-400 block mb-1 text-xs sm:text-sm">Capacity:</span>
          {sector.slot_capacity !== null ? (
            <div className="flex items-center space-x-2">
              <p className="font-medium text-white text-sm sm:text-base">
                {sector.current_occupancy} / {sector.slot_capacity}
              </p>
              {/* Capacity indicator - Responsive sizing */}
              <div className="flex-1 bg-gray-600 rounded-full h-1.5 sm:h-2 min-w-[30px] sm:min-w-[40px] overflow-hidden">
                <div
                  className={`h-1.5 sm:h-2 rounded-full transition-all duration-500 ${
                    isAtCapacity() ? 'bg-red-500 animate-pulse' : 'bg-blue-500'
                  }`}
                  style={{
                    width: `${Math.min((sector.current_occupancy / sector.slot_capacity) * 100, 100)}%`,
                  }}
                />
              </div>
              {isAtCapacity() && <span className="text-red-400 text-xs font-bold">FULL</span>}
            </div>
          ) : (
            <p className="font-medium text-white flex items-center space-x-1 text-sm sm:text-base">
              <span>{sector.current_occupancy}</span>
              <span className="text-green-400 text-xs">∞</span>
            </p>
          )}
        </div>
      </div>

      {/* Position Slots Grid - Mobile responsive */}
      <div className="mb-3 sm:mb-4">
        <div className="text-gray-400 text-xs sm:text-sm mb-2">Position Slots:</div>
        <div className="flex flex-wrap sm:flex-nowrap gap-1 sm:gap-2 justify-center">
          {positionSlots.map((slot) => (
            <PositionSlot
              key={slot.slotNumber}
              slotNumber={slot.slotNumber}
              participant={slot.participant}
              isOccupied={slot.isOccupied}
              isPlayerSlot={slot.isPlayerSlot}
              animationState={slot.isPlayerSlot ? 'highlighted' : animationState}
              onClick={() => onSlotClick?.(sector.id, slot.slotNumber)}
            />
          ))}
        </div>
      </div>

      {/* Capacity Warning - Mobile responsive */}
      {isAtCapacity() && (
        <div className="bg-red-600 bg-opacity-20 border border-red-500 rounded p-2">
          <p className="text-red-300 text-xs">⚠️ This sector is at maximum capacity</p>
        </div>
      )}
    </div>
  );
};

export const SectorGrid = React.memo(SectorGridComponent);
