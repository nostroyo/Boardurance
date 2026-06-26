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

    // Modern top-down Formula One (13x21). Pointing "up": front wing -> nose ->
    // front tires -> sidepods/cockpit -> engine spine -> rear tires -> rear wing.
    // Values 2-5 are the player's livery gradient; 1 and 6-10 are constant neutrals.
    const pixelPattern = [
      [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
      [0, 0, 1, 4, 5, 6, 6, 6, 5, 4, 1, 0, 0],
      [0, 0, 1, 1, 0, 1, 5, 1, 0, 1, 1, 0, 0],
      [0, 0, 0, 0, 0, 1, 5, 1, 0, 0, 0, 0, 0],
      [0, 0, 0, 0, 1, 2, 5, 2, 1, 0, 0, 0, 0],
      [8, 7, 0, 0, 1, 2, 4, 2, 1, 0, 0, 7, 8],
      [7, 7, 10, 0, 1, 2, 5, 2, 1, 0, 10, 7, 7],
      [7, 8, 0, 1, 2, 3, 5, 3, 2, 1, 0, 8, 7],
      [0, 0, 0, 1, 2, 3, 4, 3, 2, 1, 0, 0, 0],
      [0, 0, 1, 2, 3, 9, 9, 9, 3, 2, 1, 0, 0],
      [0, 0, 1, 2, 3, 9, 6, 9, 3, 2, 1, 0, 0],
      [0, 0, 1, 2, 3, 3, 9, 3, 3, 2, 1, 0, 0],
      [0, 0, 0, 1, 2, 3, 5, 3, 2, 1, 0, 0, 0],
      [0, 0, 0, 1, 2, 4, 6, 4, 2, 1, 0, 0, 0],
      [7, 7, 10, 1, 2, 3, 5, 3, 2, 1, 10, 7, 7],
      [7, 8, 0, 1, 2, 3, 5, 3, 2, 1, 0, 8, 7],
      [8, 7, 0, 0, 1, 2, 5, 2, 1, 0, 0, 7, 8],
      [0, 0, 0, 0, 1, 2, 5, 2, 1, 0, 0, 0, 0],
      [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
      [0, 1, 3, 4, 5, 6, 6, 6, 5, 4, 3, 1, 0],
      [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
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
      // Grid is 13 cols x 21 rows, so width = 13 * pixelSize, height = 21 * pixelSize.
      case 'small':
        return {
          width: { base: 26, sm: 26 },
          height: { base: 42, sm: 42 },
          pixelSize: { base: 2, sm: 2 },
        };
      case 'large':
        return {
          width: { base: 52, sm: 52 },
          height: { base: 84, sm: 84 },
          pixelSize: { base: 4, sm: 4 },
        };
      default: // medium
        return {
          width: { base: 39, sm: 39 },
          height: { base: 63, sm: 63 },
          pixelSize: { base: 3, sm: 3 },
        };
    }
  };

  const dimensions = getSizeDimensions();

  // Get pixel color based on pattern value.
  // 2-5 use the player's livery gradient; 1 and 6-10 are constant neutrals chosen
  // to stay readable on the dark race UI.
  const getPixelColor = (value: number): string => {
    switch (value) {
      case 0:
        return 'transparent';
      case 1:
        return '#232838'; // silhouette / wing outline
      case 2:
        return spriteStyle.colors.secondary; // body shadow
      case 3:
        return spriteStyle.colors.primary; // body mid
      case 4:
        return spriteStyle.colors.highlight; // body light
      case 5:
        return spriteStyle.colors.accent; // livery sheen stripe
      case 6:
        return '#ffffff'; // white specular shine
      case 7:
        return '#4b5563'; // tire
      case 8:
        return '#d1d5db'; // tire sidewall / rim
      case 9:
        return '#1f2937'; // halo / cockpit (dark accent)
      case 10:
        return '#e5e7eb'; // suspension / metal
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

  // Render the pixel pattern as a grid of colored cells
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
      {/* Pixel-art car */}
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
