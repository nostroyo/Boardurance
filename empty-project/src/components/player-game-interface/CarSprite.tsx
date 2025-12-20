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

    // Use solid Tailwind colors for better visibility
    const colorPalettes = [
      { primary: '#3B82F6', secondary: '#1E40AF', highlight: '#60A5FA', accent: '#DBEAFE' }, // Blue
      { primary: '#EF4444', secondary: '#B91C1C', highlight: '#F87171', accent: '#FEE2E2' }, // Red
      { primary: '#10B981', secondary: '#047857', highlight: '#34D399', accent: '#D1FAE5' }, // Green
      { primary: '#F59E0B', secondary: '#D97706', highlight: '#FBBF24', accent: '#FEF3C7' }, // Yellow
      { primary: '#8B5CF6', secondary: '#7C3AED', highlight: '#A78BFA', accent: '#EDE9FE' }, // Purple
      { primary: '#EC4899', secondary: '#BE185D', highlight: '#F472B6', accent: '#FCE7F3' }, // Pink
      { primary: '#06B6D4', secondary: '#0891B2', highlight: '#22D3EE', accent: '#CFFAFE' }, // Cyan
      { primary: '#84CC16', secondary: '#65A30D', highlight: '#A3E635', accent: '#ECFCCB' }, // Lime
    ];

    const colorIndex = hash % colorPalettes.length;
    const colors = colorPalettes[colorIndex];

    // Make player car more vibrant
    if (isPlayer) {
      colors.primary = '#FFD700'; // Gold for player
      colors.secondary = '#FFA500'; // Orange
      colors.highlight = '#FFFF00'; // Bright yellow
      colors.accent = '#FFF8DC'; // Cornsilk
    }

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
      colors,
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

  // Debug: Log colors to console in development
  if (typeof window !== 'undefined' && window.location.hostname === 'localhost') {
    console.log(`[CarSprite] ${participant.player_name} colors:`, spriteStyle.colors);
  }

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
        {row.map((pixel, colIndex) => {
          const pixelColor = getPixelColor(pixel);
          return (
            <div
              key={`${rowIndex}-${colIndex}`}
              className="border-0"
              style={{
                width: `${dimensions.pixelSize.base}px`,
                height: `${dimensions.pixelSize.base}px`,
                backgroundColor: pixelColor,
                // Ensure minimum visibility
                minWidth: '2px',
                minHeight: '2px',
                // Debug: Add a border to see if pixels are being rendered
                border: typeof window !== 'undefined' && window.location.hostname === 'localhost' && pixel !== 0 ? '1px solid rgba(255,255,255,0.1)' : 'none',
              }}
            />
          );
        })}
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
      {/* Debug info for development */}
      {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
        <div className="absolute -top-8 left-0 text-xs text-yellow-400 whitespace-nowrap z-10">
          {participant.player_name} ({dimensions.width.base}x{dimensions.height.base})
        </div>
      )}
      
      {/* 8-bit pixel car */}
      <div className="relative">
        {renderPixelPattern()}
        
        {/* Debug: Show a simple colored rectangle as fallback */}
        {typeof window !== 'undefined' && window.location.hostname === 'localhost' && (
          <div 
            className="absolute top-0 left-0 opacity-50"
            style={{
              width: `${dimensions.width.base}px`,
              height: `${dimensions.height.base}px`,
              backgroundColor: spriteStyle.colors.primary,
              border: '2px solid white',
            }}
          />
        )}
        
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