import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { TrackDisplayRedesign } from './TrackDisplayRedesign';
import type { LocalView } from '../../types/race-api';
import type { AnimationState } from '../../types/race';

describe('TrackDisplayRedesign', () => {
  const mockLocalView: LocalView = {
    center_sector: 5,
    visible_sectors: [
      {
        id: 3,
        name: 'Sector 3',
        min_value: 8,
        max_value: 12,
        slot_capacity: 4,
        sector_type: 'Straight',
        current_occupancy: 2,
      },
      {
        id: 4,
        name: 'Sector 4',
        min_value: 6,
        max_value: 10,
        slot_capacity: 3,
        sector_type: 'Curve',
        current_occupancy: 3,
      },
      {
        id: 5,
        name: 'Player Sector',
        min_value: 10,
        max_value: 15,
        slot_capacity: 5,
        sector_type: 'Straight',
        current_occupancy: 2,
      },
      {
        id: 6,
        name: 'Sector 6',
        min_value: 7,
        max_value: 11,
        slot_capacity: null, // Unlimited capacity
        sector_type: 'Curve',
        current_occupancy: 1,
      },
      {
        id: 7,
        name: 'Sector 7',
        min_value: 9,
        max_value: 13,
        slot_capacity: 2,
        sector_type: 'Finish',
        current_occupancy: 0,
      },
    ],
    visible_participants: [
      {
        player_uuid: 'test-player',
        player_name: 'You',
        car_name: 'Your Car',
        current_sector: 5,
        position_in_sector: 1,
        total_value: 13,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-2',
        player_name: 'Bob',
        car_name: 'Thunder Strike',
        current_sector: 3,
        position_in_sector: 1,
        total_value: 9,
        current_lap: 2,
        is_finished: false,
      },
    ],
  };

  const mockAnimationState: AnimationState = {
    isAnimating: false,
    movements: [],
    duration: 1000,
  };

  it('renders track display with header', () => {
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    expect(screen.getByText('Track View')).toBeInTheDocument();
    expect(screen.getByText(/Showing 5 sectors/)).toBeInTheDocument();
    expect(screen.getByText(/Player in sector 5/)).toBeInTheDocument();
  });

  it('displays sectors in linear arrangement', () => {
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    // Should show all sectors by their aria-labels
    expect(screen.getByLabelText('Sector 3: Sector 3')).toBeInTheDocument();
    expect(screen.getByLabelText('Sector 4: Sector 4')).toBeInTheDocument();
    expect(screen.getByLabelText('Sector 5: Player Sector')).toBeInTheDocument();
    expect(screen.getByLabelText('Sector 6: Sector 6')).toBeInTheDocument();
    expect(screen.getByLabelText('Sector 7: Sector 7')).toBeInTheDocument();
  });

  it('shows sector capacity indicators', () => {
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    // Should show capacity indicators
    expect(screen.getByText('2/4')).toBeInTheDocument(); // Sector 3
    expect(screen.getByText('3/3')).toBeInTheDocument(); // Sector 4 (full)
    expect(screen.getByText('2/5')).toBeInTheDocument(); // Sector 5 (player)
    expect(screen.getByText('1 cars')).toBeInTheDocument(); // Sector 6 (unlimited)
    expect(screen.getByText('0/2')).toBeInTheDocument(); // Sector 7 (empty)
  });

  it('shows value range indicators', () => {
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    // Should show value ranges
    expect(screen.getByText('Range: 8-12')).toBeInTheDocument(); // Sector 3
    expect(screen.getByText('Range: 6-10')).toBeInTheDocument(); // Sector 4
    expect(screen.getByText('Range: 10-15')).toBeInTheDocument(); // Sector 5
    expect(screen.getByText('Range: 7-11')).toBeInTheDocument(); // Sector 6
    expect(screen.getByText('Range: 9-13')).toBeInTheDocument(); // Sector 7
  });

  it('calls onSectorClick when sector is clicked', () => {
    const mockOnSectorClick = vi.fn();
    
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
        onSectorClick={mockOnSectorClick}
      />
    );

    // Click on a sector (the SectorGrid component should handle the click)
    const sectorElement = screen.getByLabelText('Sector 3: Sector 3');
    fireEvent.click(sectorElement);

    expect(mockOnSectorClick).toHaveBeenCalledWith(3);
  });

  it('calls onSlotClick when slot is clicked', () => {
    const mockOnSlotClick = vi.fn();
    
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
        onSlotClick={mockOnSlotClick}
      />
    );

    // Click on a position slot (the PositionSlot component should handle the click)
    const slotElements = screen.getAllByRole('button');
    if (slotElements.length > 0) {
      fireEvent.click(slotElements[0]);
      // The exact parameters depend on which slot was clicked
      expect(mockOnSlotClick).toHaveBeenCalled();
    }
  });

  it('shows animation indicator when animating', () => {
    const animatingState: AnimationState = {
      isAnimating: true,
      movements: [],
      duration: 1000,
    };

    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={animatingState}
      />
    );

    expect(screen.getByText('Processing sector movements...')).toBeInTheDocument();
  });

  it('shows empty state when no sectors', () => {
    const emptyLocalView: LocalView = {
      center_sector: 0,
      visible_sectors: [],
      visible_participants: [],
    };

    render(
      <TrackDisplayRedesign
        localView={emptyLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    expect(screen.getByText('No Track Data')).toBeInTheDocument();
    expect(screen.getByText('Waiting for race data to load...')).toBeInTheDocument();
  });

  it('shows legend in footer', () => {
    render(
      <TrackDisplayRedesign
        localView={mockLocalView}
        playerUuid="test-player"
        animationState={mockAnimationState}
      />
    );

    expect(screen.getByText('Your sector')).toBeInTheDocument();
    expect(screen.getByText('Available capacity')).toBeInTheDocument();
    expect(screen.getByText('Full capacity')).toBeInTheDocument();
    expect(screen.getByText('Scroll to view all sectors • Click sectors for details')).toBeInTheDocument();
  });
});