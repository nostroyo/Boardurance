import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import { vi } from 'vitest';
import { PlayerGameInterface } from './PlayerGameInterface';
import { PlayerGameProvider } from '../../contexts/PlayerGameContext';

import React from 'react';
import { render, screen } from '@testing-library/react';
import { vi } from 'vitest';
import PlayerGameInterface from './PlayerGameInterface';
import { PlayerGameProvider } from '../../contexts/PlayerGameContext';

// Simple mock for the race API
vi.mock('../../utils/raceAPI', () => ({
  raceAPI: {
    getRace: vi.fn().mockResolvedValue({ success: false, error: 'Mock not configured' }),
    getTurnPhase: vi.fn().mockResolvedValue({ success: false, error: 'Mock not configured' }),
    processRaceTurn: vi.fn().mockResolvedValue({ success: false, error: 'Mock not configured' }),
  },
  raceStatusUtils: {
    canSubmitActions: vi.fn(() => false),
    getStatusMessage: vi.fn(() => 'Mock status'),
    getTurnPhaseColor: vi.fn(() => '#10B981'),
    getLapProgress: vi.fn(() => 0),
    isFinalLap: vi.fn(() => false),
    getLapCharacteristicIcon: vi.fn(() => '🏁'),
  },
  raceErrorUtils: {
    isRetryableError: vi.fn(() => true),
    getUserFriendlyError: vi.fn((error) => error || 'Unknown error'),
    requiresUserAction: vi.fn(() => false),
  },
}));

const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <PlayerGameProvider>{children}</PlayerGameProvider>
);

describe('PlayerGameInterface Integration', () => {
  test('renders loading state initially', () => {
    render(
      <TestWrapper>
        <PlayerGameInterface raceUuid="test-race-uuid" playerUuid="test-player-uuid" />
      </TestWrapper>,
    );

    expect(screen.getByText('Loading race data...')).toBeInTheDocument();
  });

  test('renders error state when race loading fails', async () => {
    render(
      <TestWrapper>
        <PlayerGameInterface raceUuid="test-race-uuid" playerUuid="test-player-uuid" />
      </TestWrapper>,
    );

    // Should eventually show error state
    expect(await screen.findByText('Error Loading Race')).toBeInTheDocument();
  });

  test('renders redesigned components structure', () => {
    // Test that the component structure includes the redesigned components
    const { container } = render(
      <TestWrapper>
        <PlayerGameInterface raceUuid="test-race-uuid" playerUuid="test-player-uuid" />
      </TestWrapper>,
    );

    // Check that the main container exists
    expect(container.querySelector('.min-h-screen.bg-gray-900')).toBeInTheDocument();
  });
});
