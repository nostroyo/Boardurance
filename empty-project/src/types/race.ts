// Core race data models for the player game interface

export interface Race {
  uuid: string;
  name: string;
  track: Track;
  participants: RaceParticipant[];
  current_lap: number;
  total_laps: number;
  lap_characteristic: 'Straight' | 'Curve';
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
  sector_type: 'Start' | 'Straight' | 'Curve' | 'Finish';
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
  Complete: 'Complete' as const,
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

// Validation functions for race data integrity
export const validateRace = (race: any): race is Race => {
  return (
    typeof race === 'object' &&
    race !== null &&
    typeof race.uuid === 'string' &&
    typeof race.name === 'string' &&
    validateTrack(race.track) &&
    Array.isArray(race.participants) &&
    race.participants.every(validateRaceParticipant) &&
    typeof race.current_lap === 'number' &&
    race.current_lap >= 1 &&
    typeof race.total_laps === 'number' &&
    race.total_laps >= 1 &&
    (race.lap_characteristic === 'Straight' || race.lap_characteristic === 'Curve') &&
    ['Waiting', 'InProgress', 'Finished', 'Cancelled'].includes(race.status) &&
    typeof race.created_at === 'string' &&
    typeof race.updated_at === 'string'
  );
};

export const validateTrack = (track: any): track is Track => {
  return (
    typeof track === 'object' &&
    track !== null &&
    typeof track.uuid === 'string' &&
    typeof track.name === 'string' &&
    Array.isArray(track.sectors) &&
    track.sectors.every(validateSector)
  );
};

export const validateSector = (sector: any): sector is Sector => {
  return (
    typeof sector === 'object' &&
    sector !== null &&
    typeof sector.id === 'number' &&
    typeof sector.name === 'string' &&
    typeof sector.min_value === 'number' &&
    typeof sector.max_value === 'number' &&
    sector.min_value <= sector.max_value &&
    (sector.slot_capacity === null || typeof sector.slot_capacity === 'number') &&
    ['Start', 'Straight', 'Curve', 'Finish'].includes(sector.sector_type)
  );
};

export const validateRaceParticipant = (participant: any): participant is RaceParticipant => {
  return (
    typeof participant === 'object' &&
    participant !== null &&
    typeof participant.player_uuid === 'string' &&
    typeof participant.car_uuid === 'string' &&
    typeof participant.pilot_uuid === 'string' &&
    typeof participant.current_sector === 'number' &&
    participant.current_sector >= 0 &&
    typeof participant.current_position_in_sector === 'number' &&
    participant.current_position_in_sector >= 0 &&
    typeof participant.current_lap === 'number' &&
    participant.current_lap >= 1 &&
    typeof participant.total_value === 'number' &&
    participant.total_value >= 0 &&
    typeof participant.is_finished === 'boolean' &&
    (participant.finish_position === null || typeof participant.finish_position === 'number')
  );
};

// Utility functions for local view calculations (player sector ±2)
export const calculateLocalView = (race: Race, playerUuid: string): LocalRaceView | null => {
  const playerParticipant = race.participants.find((p) => p.player_uuid === playerUuid);

  if (!playerParticipant) {
    return null;
  }

  const centerSector = playerParticipant.current_sector;
  const visibleSectorIds = getVisibleSectorIds(centerSector, race.track.sectors.length);

  const visibleSectors = race.track.sectors.filter((sector) =>
    visibleSectorIds.includes(sector.id),
  );

  const visibleParticipants = race.participants.filter((participant) =>
    visibleSectorIds.includes(participant.current_sector),
  );

  return {
    centerSector,
    visibleSectors,
    visibleParticipants,
  };
};

export const getVisibleSectorIds = (centerSector: number, totalSectors: number): number[] => {
  const visibleIds: number[] = [];

  // Add center sector ±2 (5 sectors total)
  for (let offset = -2; offset <= 2; offset++) {
    const sectorId = centerSector + offset;

    // Handle wrapping for circular tracks
    if (sectorId < 0) {
      visibleIds.push(totalSectors + sectorId);
    } else if (sectorId >= totalSectors) {
      visibleIds.push(sectorId - totalSectors);
    } else {
      visibleIds.push(sectorId);
    }
  }

  return visibleIds;
};

export const getPlayerParticipant = (race: Race, playerUuid: string): RaceParticipant | null => {
  return race.participants.find((p) => p.player_uuid === playerUuid) || null;
};

export const getSectorById = (track: Track, sectorId: number): Sector | null => {
  return track.sectors.find((s) => s.id === sectorId) || null;
};

export const getParticipantsInSector = (race: Race, sectorId: number): RaceParticipant[] => {
  return race.participants.filter((p) => p.current_sector === sectorId);
};
