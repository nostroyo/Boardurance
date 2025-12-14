import React, { useMemo } from 'react';
import type { LocalView } from '../../types/race-api';
import type { AnimationState } from '../../types/race';
import { SectorCard } from './SectorCard';

// Custom CSS animations for sector transitions
const sectorAnimationStyles = `
  @keyframes bounce-subtle {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-2px); }
  }
  
  @keyframes slide-in {
    from { 
      opacity: 0; 
      transform: translateX(-20px); 
    }
    to { 
      opacity: 1; 
      transform: translateX(0); 
    }
  }
  
  @keyframes sector-highlight {
    0%, 100% { box-shadow: 0 0 0 rgba(59, 130, 246, 0); }
    50% { box-shadow: 0 0 20px rgba(59, 130, 246, 0.5); }
  }
  
  .animate-bounce-subtle {
    animation: bounce-subtle 2s ease-in-out infinite;
  }
  
  .animate-slide-in {
    animation: slide-in 0.5s ease-out;
  }
  
  .animate-sector-highlight {
    animation: sector-highlight 1.5s ease-in-out infinite;
  }
`;

export interface LocalSectorDisplayProps {
  localView: LocalView;
  playerUuid: string;
  animationState?: AnimationState;
  onSectorClick?: (sectorId: number) => void;
}

const LocalSectorDisplayComponent: React.FC<LocalSectorDisplayProps> = ({
  localView,
  playerUuid,
  animationState,
  onSectorClick,
}) => {
  const {
    center_sector: playerSector,
    visible_sectors: visibleSectors,
    visible_participants: visibleParticipants,
  } = localView;

  // Calculate position relative to player sector for visual emphasis
  const getPositionClass = (sectorId: number): 'above' | 'center' | 'below' => {
    const relativePosition = sectorId - playerSector;
    if (relativePosition === 0) return 'center';
    if (relativePosition < 0) return 'above';
    return 'below';
  };

  // Get opacity for gradient fade effect
  const getOpacity = (sectorId: number): number => {
    const distance = Math.abs(sectorId - playerSector);
    if (distance === 0) return 1.0;
    if (distance === 1) return 0.95;
    return 0.85;
  };

  // Get participants for a specific sector
  const getParticipantsInSector = (sectorId: number) => {
    return visibleParticipants
      .filter((p) => p.current_sector === sectorId)
      .sort((a, b) => a.position_in_sector - b.position_in_sector);
  };

  // Memoize expensive calculations for performance
  const memoizedSortedSectors = useMemo(() => {
    return [...visibleSectors].sort((a, b) => {
      const distA = Math.abs(a.id - playerSector);
      const distB = Math.abs(b.id - playerSector);
      if (distA !== distB) return distA - distB;
      return a.id - b.id;
    });
  }, [visibleSectors, playerSector]);

  const memoizedSectorRange = useMemo(() => {
    if (visibleSectors.length === 0) return 'No sectors';
    const minSector = Math.min(...visibleSectors.map((s) => s.id));
    const maxSector = Math.max(...visibleSectors.map((s) => s.id));
    return `${minSector} - ${maxSector}`;
  }, [visibleSectors]);

  return (
    <div className="bg-gray-800 rounded-lg p-4 border border-gray-700 shadow-lg">
      {/* Inject custom CSS animations */}
      <style>{sectorAnimationStyles}</style>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-bold text-white">Local Sector View</h2>
        <div className="text-sm text-gray-400">Showing sectors {memoizedSectorRange}</div>
      </div>

      {/* Animation overlay indicator */}
      {animationState?.isAnimating && (
        <div className="mb-4 bg-purple-600 bg-opacity-20 border border-purple-500 rounded-lg p-3">
          <div className="flex items-center space-x-2">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-purple-400"></div>
            <p className="text-purple-200 text-sm">Animating sector movements...</p>
          </div>
        </div>
      )}

      {/* Sector cards */}
      <div className="space-y-3">
        {memoizedSortedSectors.map((sector, index) => {
          const isPlayerSector = sector.id === playerSector;
          const participantsInSector = getParticipantsInSector(sector.id);
          const positionClass = getPositionClass(sector.id);
          const opacity = getOpacity(sector.id);

          return (
            <div
              key={sector.id}
              style={{
                opacity,
                animationDelay: `${index * 100}ms`,
              }}
              className={`transition-all duration-500 ease-in-out transform hover:scale-[1.02] ${
                animationState?.isAnimating ? 'animate-pulse' : ''
              } ${isPlayerSector ? 'animate-bounce-subtle' : ''}`}
              onClick={() => onSectorClick?.(sector.id)}
            >
              <SectorCard
                sector={sector}
                participants={participantsInSector}
                isPlayerSector={isPlayerSector}
                playerUuid={playerUuid}
                position={positionClass}
              />
            </div>
          );
        })}
      </div>

      {/* Footer info */}
      <div className="mt-4 text-center text-xs text-gray-500">
        Showing your current sector ± 2 sectors ({visibleSectors.length} total)
      </div>

      {/* Empty state */}
      {visibleSectors.length === 0 && (
        <div className="text-center py-8">
          <div className="text-gray-500 text-4xl mb-2">🏁</div>
          <p className="text-gray-400">No sectors to display</p>
        </div>
      )}
    </div>
  );
};

export const LocalSectorDisplay = React.memo(LocalSectorDisplayComponent);
