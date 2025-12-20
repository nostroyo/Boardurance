import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { PositionSlot } from './PositionSlot';
import type { LocalView } from '../../types/race-api';

// Mock participant data
const mockParticipant: LocalView['visible_participants'][0] = {
  player_uuid: 'player-1',
  player_name: 'Test Player',
  car_name: 'Red Racer',
  current_sector: 1,
  position_in_sector: 1,
  total_value: 15,
  current_lap: 1,
  is_finished: false,
};

describe('PositionSlot', () => {
  it('renders empty slot with slot number', () => {
    render(<PositionSlot slotNumber={3} isOccupied={false} isPlayerSlot={false} />);

    expect(screen.getByText('3')).toBeInTheDocument();
    expect(screen.getByLabelText('Position 3 - Empty')).toBeInTheDocument();
  });

  it('renders occupied slot with car initial', () => {
    render(
      <PositionSlot
        slotNumber={1}
        participant={mockParticipant}
        isOccupied={true}
        isPlayerSlot={false}
      />,
    );

    expect(screen.getByText('R')).toBeInTheDocument(); // Red Racer initial
    expect(screen.getByLabelText('Test Player - Red Racer')).toBeInTheDocument();
  });

  it('highlights player slot correctly', () => {
    render(
      <PositionSlot
        slotNumber={1}
        participant={mockParticipant}
        isOccupied={true}
        isPlayerSlot={true}
      />,
    );

    const slot = screen.getByLabelText('Test Player - Red Racer (You)');
    expect(slot).toBeInTheDocument();

    // Should have player indicator dot
    const playerIndicator = slot.querySelector('.bg-blue-400');
    expect(playerIndicator).toBeInTheDocument();
  });

  it('shows slot number indicator for occupied slots', () => {
    render(
      <PositionSlot
        slotNumber={2}
        participant={mockParticipant}
        isOccupied={true}
        isPlayerSlot={false}
      />,
    );

    // Should show slot number in bottom-left corner
    expect(screen.getByText('2')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const mockOnClick = vi.fn();

    render(
      <PositionSlot slotNumber={1} isOccupied={false} isPlayerSlot={false} onClick={mockOnClick} />,
    );

    const slot = screen.getByLabelText('Position 1 - Empty');
    fireEvent.click(slot);

    expect(mockOnClick).toHaveBeenCalledTimes(1);
  });

  it('handles keyboard navigation', () => {
    const mockOnClick = vi.fn();

    render(
      <PositionSlot slotNumber={1} isOccupied={false} isPlayerSlot={false} onClick={mockOnClick} />,
    );

    const slot = screen.getByLabelText('Position 1 - Empty');

    // Test Enter key
    fireEvent.keyDown(slot, { key: 'Enter' });
    expect(mockOnClick).toHaveBeenCalledTimes(1);

    // Test Space key
    fireEvent.keyDown(slot, { key: ' ' });
    expect(mockOnClick).toHaveBeenCalledTimes(2);
  });

  it('applies custom className', () => {
    render(
      <PositionSlot
        slotNumber={1}
        isOccupied={false}
        isPlayerSlot={false}
        className="custom-class"
      />,
    );

    const slot = screen.getByLabelText('Position 1 - Empty');
    expect(slot).toHaveClass('custom-class');
  });

  it('handles participant without player name', () => {
    const participantWithoutName = {
      ...mockParticipant,
      player_name: null,
    };

    render(
      <PositionSlot
        slotNumber={1}
        participant={participantWithoutName}
        isOccupied={true}
        isPlayerSlot={false}
      />,
    );

    expect(screen.getByLabelText('Unknown Player - Red Racer')).toBeInTheDocument();
  });
});
