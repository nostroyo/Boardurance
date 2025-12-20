import React from 'react';
import type { LocalView } from '../../types/race-api';
import { CarSprite } from './CarSprite';

export interface PositionSlotProps {
  slotNumber: number;
  participant?: LocalView['visible_participants'][0];
  isOccupied: boolean;
  isPlayerSlot: boolean;
  onClick?: () => void;
  className?: string;
  animationState?: 'idle' | 'moving' | 'highlighted';
}

const PositionSlotComponent: React.FC<PositionSlotProps> = ({
  slotNumber,
  participant,
  isOccupied,
  isPlayerSlot,
  onClick,
  className = '',
  animationState = 'idle',
}) => {
  // Get slot styling based on state - Mobile responsive
  const getSlotStyle = (): string => {
    const baseStyle =
      'relative w-12 h-12 sm:w-14 sm:h-14 lg:w-16 lg:h-16 border-2 rounded-lg transition-all duration-300 cursor-pointer flex items-center justify-center text-sm font-bold overflow-hidden touch-manipulation';

    if (isOccupied && participant) {
      if (isPlayerSlot) {
        return `${baseStyle} bg-blue-900/30 border-blue-400 text-white shadow-lg shadow-blue-500/30 ring-2 ring-blue-400/50 hover:bg-blue-800/40 active:scale-95`;
      }
      return `${baseStyle} bg-gray-800/50 border-gray-400 text-white hover:bg-gray-700/60 active:scale-95`;
    }

    // Empty slot styling
    return `${baseStyle} bg-gray-800 border-gray-600 border-dashed text-gray-400 hover:border-gray-500 hover:bg-gray-700 active:scale-95`;
  };

  // Get slot content
  const getSlotContent = (): React.ReactNode => {
    if (isOccupied && participant) {
      // Debug logging
      if (typeof window !== 'undefined' && window.location.hostname === 'localhost') {
        console.log(`[PositionSlot] Rendering CarSprite for ${participant.player_name} in slot ${slotNumber}`);
      }
      
      // Show 8-bit car sprite with debug wrapper
      return (
        <div className="relative">
          {/* Debug indicator */}
          {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
            <div className="absolute -top-6 left-0 bg-green-500 text-black text-[8px] px-1 rounded z-10">
              {participant.player_name}
            </div>
          )}
          <CarSprite
            participant={participant}
            isPlayer={isPlayerSlot}
            size="small"
            animationState={isPlayerSlot ? 'highlighted' : animationState}
          />
        </div>
      );
    }

    // Empty slot shows slot number
    return <span className="text-xs">{slotNumber}</span>;
  };

  // Get hover tooltip content
  const getTooltipContent = (): string => {
    if (isOccupied && participant) {
      const playerName = participant.player_name || 'Unknown Player';
      return `${playerName} - ${participant.car_name}${isPlayerSlot ? ' (You)' : ''}`;
    }
    return `Position ${slotNumber} - Empty`;
  };

  return (
    <div
      className={`${getSlotStyle()} ${className}`}
      onClick={onClick}
      title={getTooltipContent()}
      role="button"
      tabIndex={0}
      aria-label={getTooltipContent()}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick?.();
        }
      }}
    >
      {getSlotContent()}

      {/* Slot number indicator for occupied slots - Mobile responsive */}
      {isOccupied && participant && (
        <div className="absolute -bottom-0.5 sm:-bottom-1 -left-0.5 sm:-left-1 w-3 h-3 sm:w-4 sm:h-4 bg-gray-800 border border-gray-600 rounded-full flex items-center justify-center text-[9px] sm:text-xs text-gray-300">
          {slotNumber}
        </div>
      )}
    </div>
  );
};

export const PositionSlot = React.memo(PositionSlotComponent);
