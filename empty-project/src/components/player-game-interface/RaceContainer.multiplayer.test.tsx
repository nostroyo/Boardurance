/**
 * RaceContainer — multiplayer turn-sync wiring.
 *
 * Pins the three container-level behaviors of the race-ui delta:
 * 1. The countdown from the polled `seconds_remaining` reaches the status UI.
 * 2. A `WaitingForPlayers` submit stores the response's `turns_taken` as the
 *    polling baseline, and a later poll with a larger counter triggers the
 *    full state refresh (the "NextTurnExecuted" moment).
 * 3. AFK auto-advance: a countdown at zero without a submission starts
 *    polling, so the backend's auto-played turn is picked up.
 */
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { RaceContainer } from './RaceContainer';
import { raceAPIService } from '../../services/raceAPI';
import type {
  BoostAvailability,
  CarData,
  LapHistory,
  LocalView,
  PerformancePreview,
  TurnPhase,
} from '../../types/race-api';

vi.mock('../../services/raceAPI', () => ({
  raceAPIService: {
    getCarData: vi.fn(),
    getLocalView: vi.fn(),
    getTurnPhase: vi.fn(),
    getBoostAvailability: vi.fn(),
    getLapHistory: vi.fn(),
    getPerformancePreview: vi.fn(),
    getRace: vi.fn(),
    batchRaceData: vi.fn(),
    submitTurnAction: vi.fn(),
    pitStop: vi.fn(),
  },
}));

const api = vi.mocked(raceAPIService);

const RACE = 'race-uuid-1';
const ME = 'player-me';
const OTHER = 'player-other';

const carData: CarData = {
  car: { uuid: 'car-1', name: 'Test Car' },
  pilot: {
    uuid: 'pilot-1',
    name: 'Test Pilot',
    pilot_class: 'AllRounder',
    rarity: 'Rookie',
    skills: { reaction_time: 6, precision: 6, focus: 6, stamina: 6 },
    performance: { straight_value: 8, curve_value: 5 },
  },
  engine: {
    uuid: 'eng-1',
    name: 'Test Engine',
    rarity: 'Common',
    straight_value: 7,
    curve_value: 5,
  },
  body: { uuid: 'body-1', name: 'Test Body', rarity: 'Common', straight_value: 5, curve_value: 7 },
};

const localView: LocalView = {
  center_sector: 0,
  total_sectors: 2,
  visible_sectors: [
    {
      id: 0,
      name: 'Start',
      min_value: 0,
      max_value: 10,
      slot_capacity: null,
      sector_type: 'Start',
      current_occupancy: 2,
    },
  ],
  visible_participants: [
    {
      player_uuid: ME,
      player_name: 'Me',
      car_name: 'Test Car',
      current_sector: 0,
      position_in_sector: 0,
      total_value: 0,
      current_lap: 1,
      is_finished: false,
    },
    {
      player_uuid: OTHER,
      player_name: 'Other',
      car_name: 'Other Car',
      current_sector: 0,
      position_in_sector: 1,
      total_value: 0,
      current_lap: 1,
      is_finished: false,
    },
  ],
};

const boostAvailability: BoostAvailability = {
  available_cards: [0, 2, 3, 4],
  hand_state: { '1': 0, '2': 2, '3': 2, '4': 1 },
  tyre_type: 'Medium',
  pit_stops_completed: 0,
  cards_remaining: 5,
};

const lapHistory = { laps: [] } as unknown as LapHistory;

const performancePreview = {
  base_performance: {
    engine_contribution: 7,
    body_contribution: 5,
    pilot_contribution: 8,
    base_value: 20,
    sector_ceiling: 10,
    capped_base_value: 10,
    lap_characteristic: 'Straight',
  },
  boost_options: [],
  boost_cycle_info: {
    tyre_type: 'Medium',
    pit_stops_completed: 0,
    cards_remaining: 5,
    available_cards: [0, 2, 3, 4],
  },
} as unknown as PerformancePreview;

function makeTurnPhase(overrides: Partial<TurnPhase> = {}): TurnPhase {
  return {
    turn_phase: 'WaitingForPlayers',
    current_lap: 1,
    total_laps: 3,
    lap_characteristic: 'Straight',
    submitted_players: [],
    pending_players: [ME, OTHER],
    total_active_players: 2,
    turns_taken: 0,
    turn_deadline: 1_000_060,
    seconds_remaining: 45,
    ...overrides,
  };
}

const raceObject = {
  uuid: RACE,
  status: 'InProgress',
  participants: [
    { player_uuid: ME, is_finished: false, finish_position: null },
    { player_uuid: OTHER, is_finished: false, finish_position: null },
  ],
};

function installDefaultMocks(turnPhaseImpl: () => TurnPhase) {
  api.getCarData.mockResolvedValue(carData);
  api.getLocalView.mockResolvedValue(localView);
  api.getBoostAvailability.mockResolvedValue(boostAvailability);
  api.getLapHistory.mockResolvedValue(lapHistory);
  api.getPerformancePreview.mockResolvedValue(performancePreview);
  api.getRace.mockResolvedValue(raceObject as never);
  api.batchRaceData.mockResolvedValue({
    localView,
    boostAvailability,
    lapHistory,
    performancePreview,
  } as never);
  api.getTurnPhase.mockImplementation(async () => turnPhaseImpl());
}

describe('RaceContainer multiplayer turn sync', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders the MM:SS countdown from the polled seconds_remaining', async () => {
    installDefaultMocks(() => makeTurnPhase({ seconds_remaining: 45 }));

    render(<RaceContainer raceUuid={RACE} playerUuid={ME} />);

    await waitFor(() => expect(screen.getByText(/00:45/)).toBeInTheDocument(), {
      timeout: 5000,
    });
  });

  it('stores the submit baseline and refreshes when the polled counter advances', async () => {
    let submitted = false;
    installDefaultMocks(() => makeTurnPhase({ turns_taken: submitted ? 1 : 0 }));
    api.submitTurnAction.mockImplementation(async () => {
      submitted = true;
      return {
        success: true,
        message: 'Action submitted successfully',
        turn_phase: 'WaitingForPlayers',
        players_submitted: 1,
        total_players: 2,
        turns_taken: 0,
      };
    });

    render(<RaceContainer raceUuid={RACE} playerUuid={ME} />);

    // Drive the real submit UI: select boost 2 → Validate Turn → Confirm.
    const boostButton = await screen.findByLabelText('Select boost value 2', undefined, {
      timeout: 5000,
    });
    fireEvent.click(boostButton);
    fireEvent.click(screen.getByLabelText('Validate turn with selected boost'));
    fireEvent.click(screen.getByText('Confirm'));

    await waitFor(() => expect(api.submitTurnAction).toHaveBeenCalledTimes(1), {
      timeout: 5000,
    });

    // The poller (baseline 0) sees turns_taken 1 and triggers the full
    // refresh — the client-side "NextTurnExecuted".
    await waitFor(() => expect(api.batchRaceData).toHaveBeenCalled(), {
      timeout: 8000,
    });
  });

  it('starts polling on countdown expiry without a submission (AFK auto-advance)', async () => {
    let polls = 0;
    installDefaultMocks(() => {
      polls += 1;
      // Initial load reports an already-expired deadline; subsequent polls
      // report the backend's auto-played turn.
      return polls === 1
        ? makeTurnPhase({ seconds_remaining: 0, turns_taken: 0 })
        : makeTurnPhase({ seconds_remaining: 60, turns_taken: 1 });
    });

    render(<RaceContainer raceUuid={RACE} playerUuid={ME} />);

    // Without any user action, the expired countdown must start polling and
    // the auto-played turn must refresh the race state.
    await waitFor(() => expect(api.batchRaceData).toHaveBeenCalled(), {
      timeout: 8000,
    });
    expect(api.submitTurnAction).not.toHaveBeenCalled();
  });
});
