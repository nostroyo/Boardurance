/**
 * useRacePolling — turn-advancement detection (multiplayer turn sync).
 *
 * The polled phase alone can never end a multiplayer wait: after a turn
 * resolves, pending actions clear and the phase snaps back to
 * "WaitingForPlayers". The hook therefore completes when the polled
 * `turns_taken` exceeds the baseline captured at submission (or on the
 * legacy race-finished "Complete" phase).
 */
import { renderHook, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useRacePolling } from './useRacePolling';
import { raceAPIService } from '../services/raceAPI';
import type { TurnPhase } from '../types/race-api';

vi.mock('../services/raceAPI', () => ({
  raceAPIService: {
    getTurnPhase: vi.fn(),
  },
}));

const mockGetTurnPhase = vi.mocked(raceAPIService.getTurnPhase);

function makePhase(overrides: Partial<TurnPhase> = {}): TurnPhase {
  return {
    turn_phase: 'WaitingForPlayers',
    current_lap: 1,
    total_laps: 3,
    lap_characteristic: 'Straight',
    submitted_players: [],
    pending_players: ['other-player'],
    total_active_players: 2,
    turns_taken: 3,
    turn_deadline: null,
    seconds_remaining: null,
    ...overrides,
  };
}

function renderPolling(baselineTurn: number | null) {
  const onComplete = vi.fn();
  const onTurnPhaseChange = vi.fn();
  const utils = renderHook(() =>
    useRacePolling({
      raceUuid: 'race-1',
      enabled: true,
      baselineTurn,
      onTurnPhaseChange,
      onComplete,
    }),
  );
  return { onComplete, onTurnPhaseChange, ...utils };
}

describe('useRacePolling turn-advancement detection', () => {
  beforeEach(() => {
    mockGetTurnPhase.mockReset();
  });

  it('fires onComplete when polled turns_taken exceeds the baseline', async () => {
    mockGetTurnPhase.mockResolvedValue(makePhase({ turns_taken: 4 }));

    const { onComplete, unmount } = renderPolling(3);

    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    unmount();
  });

  it('does not fire onComplete while turns_taken equals the baseline (phase flapping)', async () => {
    // Same turn, still waiting — even though the phase "changed" vs the
    // hook's initial null phase, the turn has not advanced.
    mockGetTurnPhase.mockResolvedValue(makePhase({ turns_taken: 3 }));

    const { onComplete, onTurnPhaseChange, unmount } = renderPolling(3);

    await waitFor(() => expect(mockGetTurnPhase).toHaveBeenCalled());
    await waitFor(() => expect(onTurnPhaseChange).toHaveBeenCalled());
    expect(onComplete).not.toHaveBeenCalled();
    unmount();
  });

  it('still fires onComplete on the race-finished Complete phase without a baseline', async () => {
    mockGetTurnPhase.mockResolvedValue(makePhase({ turn_phase: 'Complete', turns_taken: 3 }));

    const { onComplete, unmount } = renderPolling(null);

    await waitFor(() => expect(onComplete).toHaveBeenCalledTimes(1));
    unmount();
  });
});
