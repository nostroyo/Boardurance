import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { RaceStatusPanel } from './RaceStatusPanel';
import type { Race } from '../../types/race';

describe('RaceStatusPanel', () => {
  const mockRace: Race = {
    uuid: 'race-123',
    name: 'Test Race',
    track: {
      uuid: 'track-123',
      name: 'Test Track',
      sectors: [],
    },
    participants: [],
    current_lap: 2,
    total_laps: 5,
    lap_characteristic: 'Straight',
    status: 'InProgress',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };

  it('renders race status information', () => {
    render(
      <RaceStatusPanel
        race={mockRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('Race Status')).toBeInTheDocument();
    expect(screen.getByText('Test Race')).toBeInTheDocument();
    expect(screen.getByText('2 / 5')).toBeInTheDocument();
    expect(screen.getByText('Straight')).toBeInTheDocument();
  });

  it('shows action required notification when waiting for player action', () => {
    render(
      <RaceStatusPanel
        race={mockRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('Action Required')).toBeInTheDocument();
    expect(screen.getByText('Submit your boost value to continue the race')).toBeInTheDocument();
  });

  it('shows processing notification when turn is processing', () => {
    render(
      <RaceStatusPanel race={mockRace} currentTurnPhase="Processing" hasSubmittedAction={true} />,
    );

    expect(screen.getByText('Processing lap results...')).toBeInTheDocument();
  });

  it('displays final lap indicator', () => {
    const finalLapRace = { ...mockRace, current_lap: 5 };

    render(
      <RaceStatusPanel
        race={finalLapRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByTitle('Final lap!')).toBeInTheDocument();
  });

  it('shows race finished notification', () => {
    const finishedRace = { ...mockRace, status: 'Finished' as const };

    render(
      <RaceStatusPanel race={finishedRace} currentTurnPhase="Complete" hasSubmittedAction={true} />,
    );

    expect(
      screen.getByText('Race has finished! Check your final position below.'),
    ).toBeInTheDocument();
  });

  it('displays turn phase status with correct color indicator', () => {
    render(
      <RaceStatusPanel
        race={mockRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('Turn Phase:')).toBeInTheDocument();
    expect(screen.getByText('WaitingForPlayers')).toBeInTheDocument();
  });

  it('shows lap characteristic icon for Straight', () => {
    render(
      <RaceStatusPanel
        race={mockRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('🏁')).toBeInTheDocument();
  });

  it('shows lap characteristic icon for Curve', () => {
    const curveRace = { ...mockRace, lap_characteristic: 'Curve' as const };

    render(
      <RaceStatusPanel
        race={curveRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('🌀')).toBeInTheDocument();
  });

  it('displays lap progress bar', () => {
    render(
      <RaceStatusPanel
        race={mockRace}
        currentTurnPhase="WaitingForPlayers"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('Lap Progress:')).toBeInTheDocument();
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
  });

  it('works with new backend API props', () => {
    render(
      <RaceStatusPanel
        currentLap={3}
        totalLaps={10}
        lapCharacteristic="Curve"
        raceStatus="InProgress"
        hasSubmittedAction={false}
      />,
    );

    expect(screen.getByText('3 / 10')).toBeInTheDocument();
    expect(screen.getByText('Curve')).toBeInTheDocument();
    expect(screen.getByText('🌀')).toBeInTheDocument();
  });
});
