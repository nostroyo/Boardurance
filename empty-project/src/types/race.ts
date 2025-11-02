// Core race data models for the player game interface

export interface Race {
  uuid: string;
  name: string;
  track: Track;
  participants: RaceParticipant[];
  current_lap: number;
  total_laps: number;
  lap_characteristic: string;
  status: 'Waiting' | 'InProgress' | 'Finished' | 'Cancelled';
  created_at: string;
  updated_at: string;
}

export interface Track {
  uuid: string;
  name: string;
  sectors: Sector[];
}

export interface Sector {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: string;
}

export interface RaceParticipant {
  player_uuid: string;
  car_uuid: string;
  pilot_uuid: string;
  current_sector: number;
  current_position_in_sector: number;
  current_lap: number;
  total_value: number;
  is_finished: boolean;
  finish_position: number | null;
}

// Local race view for player interface
export interface LocalRaceView {
  centerSector: number; // Player's current sector
  visibleSectors: Sector[]; // 5 sectors (center ±2)
  visibleParticipants: RaceParticipant[];
}

// Turn phase management
export type TurnPhase = 'WaitingForPlayers' | 'AllSubmitted' | 'Processing' | 'Complete';

export const TurnPhase = {
  WaitingForPlayers: 'WaitingForPlayers' as const,
  AllSubmitted: 'AllSubmitted' as const,
  Processing: 'Processing' as const,
  Complete: 'Complete' as const
} as const;

// Movement tracking for animations
export interface ParticipantMovement {
  participantUuid: string;
  movementType: 'Forward' | 'Backward' | 'Stay';
  fromSector: number;
  toSector: number;
  fromPosition: number;
  toPosition: number;
}

export interface AnimationState {
  isAnimating: boolean;
  movements: ParticipantMovement[];
  duration: number;
}