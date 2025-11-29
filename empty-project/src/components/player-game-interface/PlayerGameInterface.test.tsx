import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { PlayerGameInterface } from './PlayerGameInterface';
import { PlayerGameProvider } from '../../contexts/PlayerGameContext';

// Mock the race API
jest.mock('../../utils/raceAPI', () => ({
  raceAPI: {
    getRace: jest.fn(),
    getTurnPhase: jest.fn(),
    processRaceTurn: jest.fn(),
  },
  raceStatusUtils: {
    canSubmitActions: jest.fn(() => true),
    getStatusMessage: jest.fn(() => 'Test status'),
    getTurnPhaseColor: jest.fn(() => '#10B981'),
    getLapProgress: jest.fn(() => 50),
    isFinalLap: jest.fn(() => false),
    getLapCharacteristicIcon: jest.fn(() => '🏁'),
  },
  raceErrorUtils: {
    isRetryableError: jest.fn(() => true),
    getUserFriendlyError: jest.fn((error) => error),
    requiresUserAction: jest.fn(() => false),
  },
}));

// Mock race data
const mockRace = {
  uuid: 'test-race-uuid',
  name: 'Test Race',
  track: {
    uuid: 'test-track-uuid',
    name: 'Test Track',
    sectors: [
      { id: 0, name: 'Start', min_value: 0, max_value: 10, slot_capacity: null, sector_type: 'Start' },
      { id: 1, name: 'Sector 1', min_value: 5, max_value: 15, slot_capacity: 5, sector_type: 'Straight' },
      { id: 2, name: 'Sector 2', min_value: 10, max_value: 20, slot_capacity: 5, sector_type: 'Curve' },
      { id: 3, name: 'Sector 3', min_value: 15, max_value: 25, slot_capacity: 5, sector_type: 'Straight' },
      { id: 4, name: 'Finish', min_value: 20, max_value: 30, slot_capacity: null, sector_type: 'Finish' },
    ],
  },
  participants: [
    {
      player_uuid: 'test-player-uuid',
      car_uuid: 'test-car-uuid',
      pilot_uuid: 'test-pilot-uuid',
      current_sector: 2,
      current_position_in_sector: 1,
      current_lap: 1,
      total_value: 100,
      is_finished: false,
      finish_position: null,
    },
  ],
  current_lap: 1,
  total_laps: 3,
  lap_characteristic: 'Straight',
  status: 'InProgress',
  created_at: '2024-01-01T00:00:00Z',
  updated_at: '2024-01-01T00:00:00Z',
};

const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <PlayerGameProvider>{children}</PlayerGameProvider>
);

describe('PlayerGameInterface', () => {
  beforeEach(() => {
    // Reset mocks
    jest.clearAllMocks();
    
    // Setup default mock responses
    const { raceAPI } = require('../../utils/raceAPI');
    raceAPI.getRace.mockResolvedValue({ success: true, data: mockRace });
    raceAPI.getTurnPhase.mockResolvedValue({ success: true, data: 'WaitingForPlayers' });
  });

  test('renders loading state initially', () => {
    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
        />
      </TestWrapper>
    );

    expect(screen.getByText('Loading race data...')).toBeInTheDocument();
  });

  test('renders race interface after loading', async () => {
    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
        />
      </TestWrapper>
    );

    // Wait for race data to load
    await waitFor(() => {
      expect(screen.getByText('Test Race')).toBeInTheDocument();
    });

    // Check if main sections are rendered
    expect(screen.getByText('Race Status')).toBeInTheDocument();
    expect(screen.getByText('Local Sector View')).toBeInTheDocument();
    expect(screen.getByText('Your Car')).toBeInTheDocument();
    expect(screen.getByText('Turn Actions')).toBeInTheDocument();
  });

  test('displays local sector view correctly', async () => {
    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
        />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Test Race')).toBeInTheDocument();
    });

    // Should show 5 sectors (current ±2)
    expect(screen.getByText('Sector 0: Start')).toBeInTheDocument();
    expect(screen.getByText('Sector 1: Sector 1')).toBeInTheDocument();
    expect(screen.getByText('Sector 2: Sector 2')).toBeInTheDocument(); // Player's sector
    expect(screen.getByText('Sector 3: Sector 3')).toBeInTheDocument();
    expect(screen.getByText('Sector 4: Finish')).toBeInTheDocument();

    // Player's sector should be highlighted
    expect(screen.getByText('YOUR SECTOR')).toBeInTheDocument();
  });

  test('displays boost selection interface', async () => {
    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
        />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Test Race')).toBeInTheDocument();
    });

    // Should show boost selection buttons
    expect(screen.getByText('Select Boost Value (0-5):')).toBeInTheDocument();
    
    // Check all boost buttons are present
    for (let i = 0; i <= 5; i++) {
      expect(screen.getByRole('button', { name: i.toString() })).toBeInTheDocument();
    }

    expect(screen.getByRole('button', { name: /Submit Boost/ })).toBeInTheDocument();
  });

  test('handles error state correctly', async () => {
    const { raceAPI } = require('../../utils/raceAPI');
    raceAPI.getRace.mockResolvedValue({ success: false, error: 'Network error' });

    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
          onError={jest.fn()}
        />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Error Loading Race')).toBeInTheDocument();
    });

    expect(screen.getByText('Network error')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
  });

  test('handles player not in race', async () => {
    const raceWithoutPlayer = {
      ...mockRace,
      participants: [], // No participants
    };

    const { raceAPI } = require('../../utils/raceAPI');
    raceAPI.getRace.mockResolvedValue({ success: true, data: raceWithoutPlayer });

    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
        />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(screen.getByText('Not Participating')).toBeInTheDocument();
    });

    expect(screen.getByText('You are not registered as a participant in this race.')).toBeInTheDocument();
  });

  test('calls onRaceComplete when race finishes', async () => {
    const finishedRace = {
      ...mockRace,
      status: 'Finished',
      participants: [
        {
          ...mockRace.participants[0],
          is_finished: true,
          finish_position: 2,
        },
      ],
    };

    const { raceAPI } = require('../../utils/raceAPI');
    raceAPI.getRace.mockResolvedValue({ success: true, data: finishedRace });

    const onRaceComplete = jest.fn();

    render(
      <TestWrapper>
        <PlayerGameInterface
          raceUuid="test-race-uuid"
          playerUuid="test-player-uuid"
          onRaceComplete={onRaceComplete}
        />
      </TestWrapper>
    );

    await waitFor(() => {
      expect(onRaceComplete).toHaveBeenCalledWith(2);
    });
  });
});