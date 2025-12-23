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
    nft_mint_address: string | null;
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
    nft_mint_address: string | null;
  };
  engine: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
    nft_mint_address: string | null;
  };
  body: {
    uuid: string;
    name: string;
    rarity: string;
    straight_value: number;
    curve_value: number;
    nft_mint_address: string | null;
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
    current_cycle: number;
    cycles_completed: number;
    cards_remaining: number;
    available_cards: number[];
  };
}

// ============================================================================
// Turn Phase Types (from /turn-phase endpoint)
// ============================================================================

export type TurnPhaseStatus = 'WaitingForPlayers' | 'AllSubmitted' | 'Processing' | 'Complete';

export interface TurnPhase {
  turn_phase: TurnPhaseStatus;
  current_lap: number;
  lap_characteristic: LapCharacteristic;
  submitted_players: string[];
  pending_players: string[];
  total_active_players: number;
}

// ============================================================================
// Local View Types (from /local-view endpoint)
// ============================================================================

export interface LocalView {
  center_sector: number;
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

export interface BoostAvailability {
  available_cards: number[];
  hand_state: Record<string, boolean>;
  current_cycle: number;
  cycles_completed: number;
  cards_remaining: number;
  next_replenishment_at: number | null;
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
  turn_phase: string; // "WaitingForPlayers", "Processing"
  players_submitted: number;
  total_players: number;
}
