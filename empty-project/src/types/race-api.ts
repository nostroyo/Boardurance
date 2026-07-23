/**
 * TypeScript interfaces for Race API responses
 * These types match the backend API response schemas from the Rust backend
 */

// ============================================================================
// Car Data Types (from /car-data endpoint)
// ============================================================================

export interface CarData {
  car: {
    uuid: string;
    name: string;
  };
  pilot: {
    uuid: string;
    name: string;
    pilot_class: string;
    rarity: string;
    skills: {
      reaction_time: number;
      precision: number;
      focus: number;
      stamina: number;
    };
    performance: {
      straight_value: number;
      curve_value: number;
    };
  };
  engine: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
  };
  body: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
  };
}

// ============================================================================
// Performance Preview Types (from /performance-preview endpoint)
// ============================================================================

export type LapCharacteristic = 'Straight' | 'Curve';
export type MovementProbability = 'MoveUp' | 'Stay' | 'MoveDown';

export interface PerformancePreview {
  base_performance: {
    engine_contribution: number;
    body_contribution: number;
    pilot_contribution: number;
    base_value: number;
    sector_ceiling: number;
    capped_base_value: number;
    lap_characteristic: LapCharacteristic;
  };
  boost_options: Array<{
    boost_value: number;
    is_available: boolean;
    final_value: number;
    movement_probability: MovementProbability;
  }>;
  boost_cycle_info: {
    tyre_type: TyreType;
    pit_stops_completed: number;
    cards_remaining: number;
    available_cards: number[];
  };
}

// ============================================================================
// Turn Phase Types (from /turn-phase endpoint)
// ============================================================================

export type TurnPhaseStatus =
  | 'WaitingForPlayers'
  | 'AllSubmitted'
  | 'Processing'
  | 'TurnProcessed'
  | 'Complete';

export interface TurnPhase {
  turn_phase: TurnPhaseStatus;
  current_lap: number;
  total_laps: number;
  lap_characteristic: LapCharacteristic;
  submitted_players: string[];
  pending_players: string[];
  total_active_players: number;
  /**
   * Turn counter — increments exactly once per processed turn. The waiting
   * client detects "my turn executed" by this exceeding the baseline captured
   * at submission (`current_lap` saturates at `total_laps`, so it cannot
   * signal the final turn).
   */
  turns_taken: number;
  /** Armed submission deadline (Unix epoch seconds); null for solo races. */
  turn_deadline: number | null;
  /** Server-computed seconds until the deadline, clamped ≥ 0; null when no deadline. */
  seconds_remaining: number | null;
}

// ============================================================================
// Local View Types (from /local-view endpoint)
// ============================================================================

export interface LocalView {
  center_sector: number;
  /**
   * Total sectors on the track; used to number the lead sector as "Sector 1".
   * Optional for backward-compatible test fixtures — the backend always sends it.
   */
  total_sectors?: number;
  visible_sectors: Array<{
    id: number;
    name: string;
    min_value: number;
    max_value: number;
    slot_capacity: number | null;
    sector_type: string;
    current_occupancy: number;
  }>;
  visible_participants: Array<{
    player_uuid: string;
    player_name: string | null;
    car_name: string;
    current_sector: number;
    position_in_sector: number;
    total_value: number;
    current_lap: number;
    is_finished: boolean;
  }>;
}

// ============================================================================
// Boost Availability Types (from /boost-availability endpoint)
// ============================================================================

/** Tyre type chosen at race entry / pit stop, defining the boost card pool. */
export type TyreType = 'Soft' | 'Medium' | 'Hard';

export interface BoostAvailability {
  available_cards: number[];
  /** Remaining count per boost card value (keys "1".."4"). */
  hand_state: Record<string, number>;
  tyre_type: TyreType;
  pit_stops_completed: number;
  cards_remaining: number;
}

// ============================================================================
// Lap History Types (from /lap-history endpoint)
// ============================================================================

export interface LapHistory {
  laps: Array<{
    lap_number: number;
    lap_characteristic: string;
    boost_used: number;
    boost_cycle: number;
    base_value: number;
    final_value: number;
    from_sector: number;
    to_sector: number;
    movement_type: string;
  }>;
  cycle_summaries: Array<{
    cycle_number: number;
    cards_used: number[];
    laps_in_cycle: number[];
    average_boost: number;
  }>;
}

// ============================================================================
// Submit Action Types (for POST /submit-action endpoint)
// ============================================================================

export interface SubmitActionRequest {
  player_uuid: string;
  boost_value: number;
}

export interface SubmitActionResponse {
  success: boolean;
  message: string;
  turn_phase: string; // "WaitingForPlayers", "Processing", "TurnProcessed"
  players_submitted: number;
  total_players: number;
  /** Turn counter after this submission — baseline for turn-advancement detection. */
  turns_taken: number;
}

// ============================================================================
// Solo Race Types (for POST /races/solo endpoint)
// ============================================================================

/** Response from POST /races/solo. Only `race.uuid` is needed to route into play. */
export interface CreateSoloRaceResponse {
  race: { uuid: string } & Record<string, unknown>;
  message: string;
}
