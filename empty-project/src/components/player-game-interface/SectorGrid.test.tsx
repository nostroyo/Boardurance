import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { SectorGrid } from './SectorGrid';
import type { LocalView } from '../../types/race-api';

// Mock data for testing
const mockSector: LocalView['visible_sectors'][0] = {
  id: 1,
  name: 'Test Sector',
  min_value: 10,
  max_value: 20,
  slot_capacity: 5,
  sector_type: 'Straight',
  current_occupancy: 2,
};

const mockParticipants: LocalView['visible_participants'] = [
  {
    player_uuid: 'player-1',
    player_name: 'Player One',
    car_name: 'Red Car',
    current_sector: 1,
    position_in_sector: 1,
    total_value: 15,
    current_lap: 1,
    is_finished: false,
  },
  {
    player_uuid: 'player-2',
    player_name: 'Player Two',
    car_name: 'Blue Car',
    current_sector: 1,
    position_in_sector: 3,
    total_value: 18,
    current_lap: 1,
    is_finished: false,
  },
];

describe('SectorGrid', () => {
  it('renders sector header with ID and name', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('Sector 1')).toBeInTheDocument();
    expect(screen.getByText('Test Sector')).toBeInTheDocument();
  });

  it('displays sector type badge correctly', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('Straight')).toBeInTheDocument();
    expect(screen.getByText('➡️')).toBeInTheDocument();
  });

  it('shows value range information', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('Value Range:')).toBeInTheDocument();
    expect(screen.getByText('10 - 20')).toBeInTheDocument();
  });

  it('displays capacity information correctly', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('Capacity:')).toBeInTheDocument();
    expect(screen.getByText('2 / 5')).toBeInTheDocument();
  });

  it('renders 5 position slots', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    // Should have 5 position slots (numbered 1-5)
    expect(screen.getByText('Position Slots:')).toBeInTheDocument();

    // Check for slot numbers in empty slots
    expect(screen.getByText('2')).toBeInTheDocument(); // Empty slot 2
    expect(screen.getByText('4')).toBeInTheDocument(); // Empty slot 4
    expect(screen.getByText('5')).toBeInTheDocument(); // Empty slot 5
  });

  it('highlights player sector correctly', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={true}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('🎯 YOU')).toBeInTheDocument();
  });

  it('shows occupied slots with car initials', () => {
    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    // Should show car initials for occupied slots
    expect(screen.getByText('R')).toBeInTheDocument(); // Red Car initial
    expect(screen.getByText('B')).toBeInTheDocument(); // Blue Car initial
  });

  it('calls onSectorClick when sector is clicked', () => {
    const mockOnSectorClick = vi.fn();

    render(
      <SectorGrid
        sector={mockSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
        onSectorClick={mockOnSectorClick}
      />,
    );

    const sectorElement = screen.getByRole('region', { name: /Sector 1: Test Sector/ });
    sectorElement.click();

    expect(mockOnSectorClick).toHaveBeenCalledWith(1);
  });

  it('shows capacity warning when sector is full', () => {
    const fullSector = {
      ...mockSector,
      current_occupancy: 5,
      slot_capacity: 5,
    };

    render(
      <SectorGrid
        sector={fullSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('FULL')).toBeInTheDocument();
    expect(screen.getByText('⚠️ This sector is at maximum capacity')).toBeInTheDocument();
  });

  it('handles unlimited capacity sectors', () => {
    const unlimitedSector = {
      ...mockSector,
      slot_capacity: null,
    };

    render(
      <SectorGrid
        sector={unlimitedSector}
        participants={mockParticipants}
        isPlayerSector={false}
        playerUuid="player-1"
      />,
    );

    expect(screen.getByText('∞')).toBeInTheDocument();
  });
});
