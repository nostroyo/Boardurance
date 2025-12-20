import React, { useMemo } from 'react';
import type { LocalView } from '../../types/race-api';

export interface CarSpriteProps {
  participant: LocalView['visible_participants'][0];
  isPlayer: boolean;
  size?: 'small' | 'medium' | 'large';
  animationState?: 'idle' | 'moving' | 'highlighted';
  className?: string;
}

export interface SpriteStyle {
  colors: {
    primary: string;
    secondary: string;
    highlight: string;
    accent: string;
  };
  pixelPattern: number[][];
  animations: {
    idle: string;
    moving: string;
    highlighted: string;
  };
}

const CarSpriteComponent: React.FC<CarSpriteProps> = ({
  participant,
  isPlayer,
  size = 'medium',
  animationState = 'idle',
  className = '',
}) => {
  // Generate unique color scheme based on player UUID
  const spriteStyle = useMemo((): SpriteStyle => {
    const hash = participant.player_uuid
      .split('')
      .reduce((acc, char) => acc + char.charCodeAt(0), 0);

    // Generate distinct colors for each player
    const hue = (hash * 137.508) % 360; // Golden angle for good distribution
    const saturation = isPlayer ? 80 : 60;
    const lightness = isPlayer ? 55 : 45;

    const primary = `hsl(${hue}, ${saturation}%, ${lightness}%)`;
    const secondary = `hsl(${(hue + 30) % 360}, ${saturation - 10}%, ${lightness - 10}%)`;
    const highlight = `hsl(${hue}, ${saturation + 20}%, ${lightness + 20}%)`;
    const accent = `hsl(${(hue + 180) % 360}, ${saturation}%, ${lightness + 10}%)`;

    // 8-bit car pixel pattern (8x6 pixels)
    const pixelPattern = [
      [0, 0, 1, 1, 1, 1, 0, 0],
      [0, 1, 2, 2, 2, 2, 1, 0],
      [1, 2, 3, 2, 2, 3, 2, 1],
      [1, 2, 2, 2, 2, 2, 2, 1],
      [0, 1, 1, 1, 1, 1, 1, 0],
      [0, 0, 4, 0, 0, 4, 0, 0],
    ];

    const animations = {
      idle: 'animate-pulse',
      moving: 'animate-bounce',
      highlighted: 'animate-ping',
    };

    return {
      colors: { primary, secondary, highlight, accent },
      pixelPattern,
      animations,
    };
  }, [participant.player_uuid, isPlayer]);

  // Get size dimensions - Mobile responsive
  const getSizeDimensions = () => {
    switch (size) {
      case 'small':
        return { 
          width: { base: 20, sm: 24 }, 
          height: { base: 15, sm: 18 }, 
          pixelSize: { base: 2.5, sm: 3 } 
        };
      case 'large':
        return { 
          width: { base: 32, sm: 40 }, 
          height: { base: 24, sm: 30 }, 
          pixelSize: { base: 4, sm: 5 } 
        };
      default: // medium
        return { 
          width: { base: 26, sm: 32 }, 
          height: { base: 20, sm: 24 }, 
          pixelSize: { base: 3.25, sm: 4 } 
        };
    }
  };

  const dimensions = getSizeDimensions();

  // Get pixel color based on pattern value
  const getPixelColor = (value: number): string => {
    switch (value) {
      case 0:
        return 'transparent';
      case 1:
        return spriteStyle.colors.secondary;
      case 2:
        return spriteStyle.colors.primary;
      case 3:
        return spriteStyle.colors.highlight;
      case 4:
        return spriteStyle.colors.accent;
      default:
        return 'transparent';
    }
  };

  // Get container styling
  const getContainerStyle = (): string => {
    const baseStyle = 'relative inline-block transition-all duration-300';
    const animationClass = spriteStyle.animations[animationState];
    
    let playerStyle = '';
    if (isPlayer) {
      playerStyle = 'ring-2 ring-blue-400 ring-opacity-50 shadow-lg shadow-blue-500/30';
    }

    return `${baseStyle} ${animationClass} ${playerStyle}`;
  };

  // Render the 8-bit pixel pattern - Responsive sizing with better visibility
  const renderPixelPattern = () => {
    return spriteStyle.pixelPattern.map((row, rowIndex) => (
      <div key={rowIndex} className="flex">
        {row.map((pixel, colIndex) => (
          <div
            key={`${rowIndex}-${colIndex}`}
            className="border-0"
            style={{
              width: `${dimensions.pixelSize.base}px`,
              height: `${dimensions.pixelSize.base}px`,
              backgroundColor: getPixelColor(pixel),
              imageRendering: 'pixelated',
              // Ensure minimum visibility
              minWidth: '2px',
              minHeight: '2px',
            }}
          />
        ))}
      </div>
    ));
  };

  return (
    <div
      className={`${getContainerStyle()} ${className}`}
      style={{
        width: `${dimensions.width.base}px`,
        height: `${dimensions.height.base}px`,
      }}
      title={`${participant.player_name || 'Unknown Player'} - ${participant.car_name}${isPlayer ? ' (You)' : ''}`}
      role="img"
      aria-label={`Car sprite for ${participant.player_name || 'Unknown Player'}`}
    >
      {/* 8-bit pixel car */}
      <div className="relative">
        {renderPixelPattern()}
        
        {/* Player indicator overlay - Mobile responsive */}
        {isPlayer && (
          <div className="absolute -top-0.5 sm:-top-1 -right-0.5 sm:-right-1 w-2 h-2 sm:w-3 sm:h-3 bg-blue-400 rounded-full border border-white animate-pulse" />
        )}
        
        {/* Car name label (optional, for debugging) - Mobile responsive */}
        {size === 'large' && (
          <div className="absolute -bottom-5 sm:-bottom-6 left-1/2 transform -translate-x-1/2 text-[10px] sm:text-xs text-gray-300 whitespace-nowrap">
            {participant.car_name}
          </div>
        )}
      </div>
    </div>
  );
};

export const CarSprite = React.memo(CarSpriteComponent);