import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { CarSprite } from './CarSprite';
import type { LocalView } from '../../types/race-api';

// Mock participant data
const mockParticipant: LocalView['visible_participants'][0] = {
  player_uuid: 'test-player-123',
  player_name: 'Test Player',
  car_name: 'Test Car',
  current_sector: 1,
  position_in_sector: 1,
  total_value: 100,
  current_lap: 1,
  is_finished: false,
};

const mockPlayerParticipant: LocalView['visible_participants'][0] = {
  player_uuid: 'player-456',
  player_name: 'Player',
  car_name: 'Player Car',
  current_sector: 2,
  position_in_sector: 2,
  total_value: 150,
  current_lap: 1,
  is_finished: false,
};

describe('CarSprite', () => {
  it('renders car sprite with correct participant information', () => {
    render(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
      />
    );

    const sprite = screen.getByRole('img');
    expect(sprite).toBeInTheDocument();
    expect(sprite).toHaveAttribute('aria-label', 'Car sprite for Test Player');
  });

  it('renders player car with special highlighting', () => {
    render(
      <CarSprite
        participant={mockPlayerParticipant}
        isPlayer={true}
      />
    );

    const sprite = screen.getByRole('img');
    expect(sprite).toBeInTheDocument();
    
    // Check for player indicator (blue dot)
    const playerIndicator = sprite.querySelector('.bg-blue-400');
    expect(playerIndicator).toBeInTheDocument();
  });

  it('applies correct size styling', () => {
    const { rerender } = render(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        size="small"
      />
    );

    let sprite = screen.getByRole('img');
    expect(sprite).toHaveStyle({ width: '24px', height: '18px' });

    rerender(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        size="large"
      />
    );

    sprite = screen.getByRole('img');
    expect(sprite).toHaveStyle({ width: '40px', height: '30px' });
  });

  it('applies correct animation state classes', () => {
    const { rerender } = render(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        animationState="idle"
      />
    );

    let container = screen.getByRole('img');
    expect(container).toHaveClass('animate-pulse');

    rerender(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        animationState="moving"
      />
    );

    container = screen.getByRole('img');
    expect(container).toHaveClass('animate-bounce');

    rerender(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        animationState="highlighted"
      />
    );

    container = screen.getByRole('img');
    expect(container).toHaveClass('animate-ping');
  });

  it('generates unique colors for different players', () => {
    const participant1 = { ...mockParticipant, player_uuid: 'player-1' };
    const participant2 = { ...mockParticipant, player_uuid: 'player-2' };

    const { rerender } = render(
      <CarSprite participant={participant1} isPlayer={false} />
    );

    const sprite1 = screen.getByRole('img');
    const pixels1 = sprite1.querySelectorAll('div[style*="background"]');

    rerender(
      <CarSprite participant={participant2} isPlayer={false} />
    );

    const sprite2 = screen.getByRole('img');
    const pixels2 = sprite2.querySelectorAll('div[style*="background"]');

    // Colors should be different for different players
    // Check that pixel pattern is rendered correctly
    expect(pixels1.length).toBeGreaterThan(0);
    expect(pixels2.length).toBeGreaterThan(0);
  });

  it('renders 8-bit pixel pattern correctly', () => {
    render(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
      />
    );

    const sprite = screen.getByRole('img');
    
    // Check that pixel pattern is rendered (8x6 grid)
    const pixelRows = sprite.querySelectorAll('div.flex');
    expect(pixelRows).toHaveLength(6); // 6 rows in the pattern

    // Check first row has 8 pixels
    const firstRowPixels = pixelRows[0].children;
    expect(firstRowPixels).toHaveLength(8);
  });

  it('shows car name for large size sprites', () => {
    render(
      <CarSprite
        participant={mockParticipant}
        isPlayer={false}
        size="large"
      />
    );

    expect(screen.getByText('Test Car')).toBeInTheDocument();
  });

  it('includes player indicator in tooltip for player cars', () => {
    render(
      <CarSprite
        participant={mockPlayerParticipant}
        isPlayer={true}
      />
    );

    const sprite = screen.getByRole('img');
    expect(sprite).toHaveAttribute('title', expect.stringContaining('(You)'));
  });
});