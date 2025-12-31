import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CarSpritePositioning } from './CarSpritePositioning';
import type { LocalView } from '../../types/race-api';

// Mock sector data
const mockSector: LocalView['visible_sectors'][0] = {
  id: 1,
  name: 'Test Sector',
  min_value: 10,
  max_value: 20,
  slot_capacity: 5,
  sector_type: 'Straight',
  current_occupancy: 2,
};

// Mock participants
const mockParticipants: LocalView['visible_participants'] = [
  {
    player_uuid: 'player-1',
    player_name: 'Player 1',
    car_name: 'Car 1',
    current_sector: 1,
    position_in_sector: 1,
    total_value: 100,
    current_lap: 1,
    is_finished: false,
  },
  {
    player_uuid: 'player-2',
    player_name: 'Player 2',
    car_name: 'Car 2',
    current_sector: 1,
    position_in_sector: 3,
    total_value: 120,
    current_lap: 1,
    is_finished: false,
  },
  {
    player_uuid: 'player-3',
    player_name: 'Player 3',
    car_name: 'Car 3',
    current_sector: 2, // Different sector
    position_in_sector: 1,
    total_value: 90,
    current_lap: 1,
    is_finished: false,
  },
];

describe('CarSpritePositioning', () => {
  it('renders positioned cars in correct slots', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    // Should show 5 position slots
    const slots = screen.getAllByText(/[1-5]/);
    expect(slots.length).toBeGreaterThanOrEqual(5);

    // Should render car sprites for participants in this sector
    const carSprites = screen.getAllByRole('img');
    expect(carSprites).toHaveLength(2); // Only 2 participants in sector 1
  });

  it('filters participants by sector correctly', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    // Should only show participants in sector 1
    expect(screen.getByLabelText(/Car sprite for Player 1/)).toBeInTheDocument();
    expect(screen.getByLabelText(/Car sprite for Player 2/)).toBeInTheDocument();
    expect(screen.queryByLabelText(/Car sprite for Player 3/)).not.toBeInTheDocument();
  });

  it('highlights player car correctly', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    const playerSprite = screen.getByLabelText(/Car sprite for Player 1/);
    expect(playerSprite.parentElement).toHaveStyle({ zIndex: '20' });
  });

  it('positions cars without overlap', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    const carContainers = screen.getAllByRole('img').map((img) => img.parentElement);

    // Each car should have a unique position
    const positions = carContainers.map((container) => {
      const style = window.getComputedStyle(container!);
      return `${style.left}-${style.top}`;
    });

    const uniquePositions = new Set(positions);
    expect(uniquePositions.size).toBe(carContainers.length);
  });

  it('handles car click events', () => {
    const onCarClick = vi.fn();

    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
        onCarClick={onCarClick}
      />,
    );

    const carSprite = screen.getByLabelText(/Car sprite for Player 1/);
    fireEvent.click(carSprite.parentElement!);

    expect(onCarClick).toHaveBeenCalledWith('player-1');
  });

  it('shows sector capacity warning when full', () => {
    const fullSector = { ...mockSector, slot_capacity: 2, current_occupancy: 2 };

    render(
      <CarSpritePositioning
        sector={fullSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText(/Sector Full/)).toBeInTheDocument();
  });

  it('adapts car size based on container width', () => {
    // Test with small container (fewer participants)
    const { rerender } = render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants.slice(0, 1)}
        playerUuid="player-1"
      />,
    );

    let carSprite = screen.getByRole('img');
    expect(carSprite).toHaveStyle({ width: '32px' }); // Medium size (default)

    // Test with more participants (larger container)
    rerender(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    carSprite = screen.getAllByRole('img')[0];
    expect(carSprite).toHaveStyle({ width: '32px' }); // Still medium size
  });

  it('shows position numbers for each car', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={mockParticipants}
        playerUuid="player-1"
      />,
    );

    // Should show position indicators
    expect(screen.getByText('1')).toBeInTheDocument(); // Player 1 in position 1
    expect(screen.getByText('3')).toBeInTheDocument(); // Player 2 in position 3
  });

  it('handles empty sector correctly', () => {
    render(
      <CarSpritePositioning
        sector={mockSector}
        participants={[]} // No participants
        playerUuid="player-1"
      />,
    );

    // Should show empty slots
    const slots = screen.getAllByText(/[1-5]/);
    expect(slots.length).toBe(5); // All 5 slots should be empty and show numbers

    // Should not show any car sprites
    expect(screen.queryByRole('img')).not.toBeInTheDocument();
  });
});
