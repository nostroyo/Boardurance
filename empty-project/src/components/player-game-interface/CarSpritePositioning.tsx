import React, { useMemo } from 'react';
import type { LocalView } from '../../types/race-api';
import { CarSprite } from './CarSprite';

export interface CarSpritePositioningProps {
  sector: LocalView['visible_sectors'][0];
  participants: LocalView['visible_participants'];
  playerUuid: string;
  animationState?: 'idle' | 'moving' | 'highlighted';
  onCarClick?: (participantUuid: string) => void;
}

export interface PositionedCar {
  participant: LocalView['visible_participants'][0];
  position: number;
  isPlayer: boolean;
  gridPosition: {
    x: number;
    y: number;
  };
}

const CarSpritePositioningComponent: React.FC<CarSpritePositioningProps> = ({
  sector,
  participants,
  playerUuid,
  animationState = 'idle',
  onCarClick,
}) => {
  // Calculate grid position for a given slot number
  const calculateGridPosition = (slotNumber: number): { x: number; y: number } => {
    // Horizontal layout: positions 1-5 arranged left to right
    const slotsPerRow = 5;
    const slotWidth = 60; // Width including spacing
    
    // Center the grid horizontally
    const totalWidth = slotsPerRow * slotWidth;
    const startX = -totalWidth / 2 + slotWidth / 2;
    
    const x = startX + (slotNumber - 1) * slotWidth;
    const y = 0; // Single row for now
    
    return { x, y };
  };

  // Calculate positioned cars in this sector
  const positionedCars = useMemo((): PositionedCar[] => {
    const sectorParticipants = participants
      .filter((p) => p.current_sector === sector.id)
      .sort((a, b) => a.position_in_sector - b.position_in_sector);

    return sectorParticipants.map((participant) => {
      const isPlayer = participant.player_uuid === playerUuid;
      
      // Calculate grid position based on position_in_sector
      // Arrange cars in a 5-slot horizontal grid (positions 1-5)
      const position = participant.position_in_sector;
      const gridPosition = calculateGridPosition(position);

      return {
        participant,
        position,
        isPlayer,
        gridPosition,
      };
    });
  }, [participants, sector.id, playerUuid]);

  // Handle car overlap prevention
  const getCarPositionStyle = (car: PositionedCar, index: number): React.CSSProperties => {
    const basePosition = car.gridPosition;
    
    // Add slight offset for overlapping cars (shouldn't happen with proper positioning)
    const overlapOffset = index * 2;
    
    return {
      position: 'absolute',
      left: `calc(50% + ${basePosition.x}px)`,
      top: `calc(50% + ${basePosition.y + overlapOffset}px)`,
      transform: 'translate(-50%, -50%)',
      zIndex: car.isPlayer ? 20 : 10 + index,
      transition: 'all 0.5s cubic-bezier(0.4, 0, 0.2, 1)',
    };
  };

  // Get container dimensions based on number of cars
  const getContainerDimensions = (): { width: number; height: number } => {
    const minWidth = 300; // Minimum width for 5 slots
    const minHeight = 80; // Minimum height for car sprites
    
    return {
      width: Math.max(minWidth, positionedCars.length * 60),
      height: minHeight,
    };
  };

  const containerDimensions = getContainerDimensions();

  // Handle responsive scaling
  const getResponsiveCarSize = (): 'small' | 'medium' | 'large' => {
    // Use CSS media queries or container size to determine sprite size
    if (containerDimensions.width < 200) return 'small';
    if (containerDimensions.width > 400) return 'large';
    return 'medium';
  };

  const carSize = getResponsiveCarSize();

  return (
    <div
      className="relative mx-auto"
      style={{
        width: `${containerDimensions.width}px`,
        height: `${containerDimensions.height}px`,
      }}
    >
      {/* Position slot indicators (background) */}
      <div className="absolute inset-0 flex justify-center items-center">
        <div className="flex space-x-4">
          {[1, 2, 3, 4, 5].map((slotNumber) => {
            const hasCarInSlot = positionedCars.some((car) => car.position === slotNumber);
            return (
              <div
                key={slotNumber}
                className={`w-12 h-12 border-2 border-dashed rounded-lg flex items-center justify-center text-xs transition-all duration-300 ${
                  hasCarInSlot
                    ? 'border-gray-600 bg-gray-800/30'
                    : 'border-gray-700 bg-gray-900/20'
                }`}
              >
                {!hasCarInSlot && (
                  <span className="text-gray-500 font-mono">{slotNumber}</span>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Positioned car sprites */}
      {positionedCars.map((car, index) => (
        <div
          key={car.participant.player_uuid}
          style={getCarPositionStyle(car, index)}
          onClick={() => onCarClick?.(car.participant.player_uuid)}
          className="cursor-pointer"
        >
          <CarSprite
            participant={car.participant}
            isPlayer={car.isPlayer}
            size={carSize}
            animationState={car.isPlayer ? 'highlighted' : animationState}
          />
          
          {/* Position number indicator */}
          <div className="absolute -bottom-2 -right-2 w-4 h-4 bg-gray-800 border border-gray-600 rounded-full flex items-center justify-center text-xs text-gray-300 font-mono">
            {car.position}
          </div>
        </div>
      ))}

      {/* Sector capacity warning */}
      {sector.slot_capacity !== null && positionedCars.length >= sector.slot_capacity && (
        <div className="absolute -bottom-8 left-1/2 transform -translate-x-1/2 bg-red-600 bg-opacity-20 border border-red-500 rounded px-2 py-1">
          <span className="text-red-300 text-xs">⚠️ Sector Full</span>
        </div>
      )}
    </div>
  );
};

export const CarSpritePositioning = React.memo(CarSpritePositioningComponent);