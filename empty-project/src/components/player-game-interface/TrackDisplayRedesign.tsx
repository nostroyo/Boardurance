import React, { useMemo, useRef, useEffect } from 'react';
import type { LocalView } from '../../types/race-api';
import type { AnimationState } from '../../types/race';
import { SectorGrid } from './SectorGrid';

export interface TrackDisplayRedesignProps {
  localView: LocalView;
  playerUuid: string;
  animationState?: AnimationState;
  onSectorClick?: (sectorId: number) => void;
  onSlotClick?: (sectorId: number, slotNumber: number) => void;
}

const TrackDisplayRedesignComponent: React.FC<TrackDisplayRedesignProps> = ({
  localView,
  playerUuid,
  animationState,
  onSectorClick,
  onSlotClick,
}) => {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const playerSectorRef = useRef<HTMLDivElement>(null);

  const {
    center_sector: playerSector,
    visible_sectors: visibleSectors,
    visible_participants: visibleParticipants,
  } = localView;

  // Filter and sort sectors for linear arrangement - max 2 before and after player sector (Requirements 1.1, 1.5)
  const sortedSectors = useMemo(() => {
    const allSectors = [...visibleSectors].sort((a, b) => a.id - b.id);

    // Find player sector index
    const playerSectorIndex = allSectors.findIndex((sector) => sector.id === playerSector);

    if (playerSectorIndex === -1) {
      // If player sector not found, return all sectors (fallback)
      return allSectors;
    }

    // Get max 2 sectors before and after player sector
    const startIndex = Math.max(0, playerSectorIndex - 2);
    const endIndex = Math.min(allSectors.length, playerSectorIndex + 3); // +3 because slice is exclusive

    return allSectors.slice(startIndex, endIndex);
  }, [visibleSectors, playerSector]);

  // Smooth scrolling/centering on player sector (Requirements 1.5)
  useEffect(() => {
    if (playerSectorRef.current && scrollContainerRef.current) {
      const playerElement = playerSectorRef.current;
      const container = scrollContainerRef.current;

      // Calculate the position to center the player sector
      const containerHeight = container.clientHeight;
      const elementTop = playerElement.offsetTop;
      const elementHeight = playerElement.clientHeight;

      // Center the player sector in the viewport
      const scrollTop = elementTop - containerHeight / 2 + elementHeight / 2;

      // Check if scrollTo method is available (for test environment compatibility)
      if (typeof container.scrollTo === 'function') {
        container.scrollTo({
          top: scrollTop,
          behavior: 'smooth',
        });
      } else {
        // Fallback for test environments
        container.scrollTop = scrollTop;
      }
    }
  }, [playerSector, sortedSectors]);

  // Get sector capacity indicator styling (Requirements 2.4)
  const getSectorCapacityStyle = (
    sector: LocalView['visible_sectors'][0],
  ): {
    color: string;
    bgColor: string;
    icon: string;
  } => {
    if (sector.slot_capacity === null) {
      return { color: 'text-green-400', bgColor: 'bg-green-900/20', icon: '∞' };
    }

    const occupancyRatio = sector.current_occupancy / sector.slot_capacity;

    if (occupancyRatio >= 1.0) {
      return { color: 'text-red-400', bgColor: 'bg-red-900/20', icon: '🚫' };
    } else if (occupancyRatio >= 0.8) {
      return { color: 'text-yellow-400', bgColor: 'bg-yellow-900/20', icon: '⚠️' };
    } else {
      return { color: 'text-green-400', bgColor: 'bg-green-900/20', icon: '✓' };
    }
  };

  // Get value range display styling (Requirements 2.4)
  const getValueRangeStyle = (sector: LocalView['visible_sectors'][0]): string => {
    const range = sector.max_value - sector.min_value;

    if (range <= 2) {
      return 'text-red-400'; // Narrow range - high precision required
    } else if (range <= 4) {
      return 'text-yellow-400'; // Medium range
    } else {
      return 'text-green-400'; // Wide range - more forgiving
    }
  };

  return (
    <div className="bg-gray-900 rounded-lg border border-gray-700 shadow-xl overflow-hidden w-full max-w-full">
      {/* Header - Mobile-first responsive layout */}
      <div className="bg-gray-800 px-3 sm:px-4 lg:px-6 py-2 sm:py-3 lg:py-4 border-b border-gray-700">
        <div className="flex flex-col space-y-2 sm:space-y-0 sm:flex-row sm:items-center sm:justify-between">
          <h2 className="text-base sm:text-lg lg:text-xl font-bold text-white flex items-center space-x-2">
            <span className="text-lg sm:text-xl lg:text-2xl">🏁</span>
            <span className="truncate">Track View</span>
          </h2>
          <div className="text-xs sm:text-sm text-gray-400 flex flex-wrap items-center gap-1 sm:gap-2">
            <span className="hidden sm:inline whitespace-nowrap">
              Showing {sortedSectors.length} sectors
            </span>
            <span className="hidden sm:inline">•</span>
            <span className="whitespace-nowrap">Player in sector {playerSector}</span>
            <span className="hidden lg:inline">•</span>
            <span className="hidden lg:inline whitespace-nowrap">Max 2 before/after</span>
          </div>
        </div>
      </div>

      {/* Animation overlay indicator - Mobile responsive */}
      {animationState?.isAnimating && (
        <div className="bg-purple-600 bg-opacity-20 border-b border-purple-500 px-3 sm:px-4 lg:px-6 py-2 sm:py-3">
          <div className="flex items-center space-x-2">
            <div className="animate-spin rounded-full h-3 w-3 sm:h-4 sm:w-4 border-b-2 border-purple-400 flex-shrink-0"></div>
            <p className="text-purple-200 text-xs sm:text-sm truncate">
              Processing sector movements...
            </p>
          </div>
        </div>
      )}

      {/* Linear sector arrangement container - Adaptive height and scrolling */}
      <div
        ref={scrollContainerRef}
        className="h-64 sm:h-80 md:h-96 lg:h-[28rem] overflow-y-auto scrollbar-thin scrollbar-thumb-gray-600 scrollbar-track-gray-800"
      >
        <div className="p-2 sm:p-3 lg:p-4 space-y-2 sm:space-y-3 lg:space-y-4">
          {sortedSectors.map((sector) => {
            const isPlayerSector = sector.id === playerSector;
            const capacityStyle = getSectorCapacityStyle(sector);
            const valueRangeStyle = getValueRangeStyle(sector);

            return (
              <div
                key={sector.id}
                ref={isPlayerSector ? playerSectorRef : undefined}
                className={`transition-all duration-500 ease-in-out ${
                  animationState?.isAnimating ? 'animate-pulse' : ''
                }`}
              >
                {/* Sector capacity and value range indicators - Mobile responsive */}
                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 mb-2 px-1 sm:px-2">
                  {/* Capacity indicator */}
                  <div
                    className={`flex items-center space-x-1 sm:space-x-2 px-2 sm:px-3 py-1 rounded-full text-xs font-medium ${capacityStyle.bgColor} ${capacityStyle.color} flex-shrink-0`}
                  >
                    <span className="text-xs sm:text-sm">{capacityStyle.icon}</span>
                    <span className="whitespace-nowrap">
                      {sector.slot_capacity !== null
                        ? `${sector.current_occupancy}/${sector.slot_capacity}`
                        : `${sector.current_occupancy} cars`}
                    </span>
                  </div>

                  {/* Value range indicator */}
                  <div
                    className={`px-2 sm:px-3 py-1 rounded-full text-xs font-medium bg-gray-800 ${valueRangeStyle} flex-shrink-0`}
                  >
                    <span className="whitespace-nowrap">
                      Range: {sector.min_value}-{sector.max_value}
                    </span>
                  </div>
                </div>

                {/* Sector grid component */}
                <SectorGrid
                  sector={sector}
                  participants={visibleParticipants}
                  isPlayerSector={isPlayerSector}
                  playerUuid={playerUuid}
                  onSectorClick={onSectorClick}
                  onSlotClick={onSlotClick}
                  animationState={
                    isPlayerSector ? 'highlighted' : animationState?.isAnimating ? 'moving' : 'idle'
                  }
                />
              </div>
            );
          })}
        </div>
      </div>

      {/* Footer with navigation hints - Mobile responsive */}
      <div className="bg-gray-800 px-3 sm:px-4 lg:px-6 py-2 sm:py-3 border-t border-gray-700">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-2 sm:gap-0 text-xs text-gray-400">
          <div className="flex flex-wrap items-center gap-2 sm:gap-4">
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-blue-400 rounded-full flex-shrink-0"></div>
              <span className="whitespace-nowrap">Your sector</span>
            </div>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-green-400 rounded-full flex-shrink-0"></div>
              <span className="whitespace-nowrap">Available capacity</span>
            </div>
            <div className="flex items-center space-x-1">
              <div className="w-2 h-2 bg-red-400 rounded-full flex-shrink-0"></div>
              <span className="whitespace-nowrap">Full capacity</span>
            </div>
          </div>
          <div className="text-xs text-gray-500 sm:text-gray-400">
            <span className="hidden sm:inline">
              Scroll to view all sectors • Click sectors for details
            </span>
            <span className="sm:hidden">Tap sectors for details</span>
          </div>
        </div>
      </div>

      {/* Empty state - Mobile responsive */}
      {sortedSectors.length === 0 && (
        <div className="flex flex-col items-center justify-center py-8 sm:py-12 lg:py-16 text-center px-4">
          <div className="text-gray-500 text-4xl sm:text-5xl lg:text-6xl mb-2 sm:mb-4">🏁</div>
          <h3 className="text-lg sm:text-xl font-semibold text-gray-400 mb-1 sm:mb-2">
            No Track Data
          </h3>
          <p className="text-gray-500 text-sm max-w-xs">Waiting for race data to load...</p>
        </div>
      )}
    </div>
  );
};

export const TrackDisplayRedesign = React.memo(TrackDisplayRedesignComponent);
