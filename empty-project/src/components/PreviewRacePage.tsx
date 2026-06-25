/**
 * PreviewRacePage - DEV-ONLY throwaway page for reviewing the race UI / car sprite
 * without a backend. Renders the real RaceInterface with mock data.
 * Route: /preview-race  (remove this file and its route when done reviewing.)
 */

import RaceInterface from './player-game-interface/RaceInterface';
import type { LocalView } from '../types/race-api';

const PLAYER_UUID = 'player-gold-0001';

const mockLocalView: LocalView = {
  center_sector: 2,
  visible_sectors: [
    {
      id: 1,
      name: 'Start / Finish Straight',
      min_value: 10,
      max_value: 25,
      slot_capacity: 5,
      sector_type: 'Straight',
      current_occupancy: 2,
    },
    {
      id: 2,
      name: 'Esses',
      min_value: 8,
      max_value: 18,
      slot_capacity: 5,
      sector_type: 'Corner',
      current_occupancy: 3,
    },
    {
      id: 3,
      name: 'Back Straight',
      min_value: 12,
      max_value: 30,
      slot_capacity: null,
      sector_type: 'Straight',
      current_occupancy: 1,
    },
  ],
  visible_participants: [
    {
      player_uuid: PLAYER_UUID,
      player_name: 'You',
      car_name: 'Gold Arrow',
      current_sector: 2,
      position_in_sector: 1,
      total_value: 142,
      current_lap: 3,
      is_finished: false,
    },
    {
      player_uuid: 'rival-blue-22',
      player_name: 'Verstappen',
      car_name: 'Blue Bolt',
      current_sector: 2,
      position_in_sector: 0,
      total_value: 150,
      current_lap: 3,
      is_finished: false,
    },
    {
      player_uuid: 'rival-red-77',
      player_name: 'Leclerc',
      car_name: 'Rosso Corsa',
      current_sector: 2,
      position_in_sector: 3,
      total_value: 138,
      current_lap: 3,
      is_finished: false,
    },
    {
      player_uuid: 'rival-green-04',
      player_name: 'Norris',
      car_name: 'Papaya... no, Green',
      current_sector: 1,
      position_in_sector: 2,
      total_value: 120,
      current_lap: 3,
      is_finished: false,
    },
    {
      player_uuid: 'rival-purple-19',
      player_name: 'Hamilton',
      car_name: 'Violet Streak',
      current_sector: 1,
      position_in_sector: 4,
      total_value: 118,
      current_lap: 3,
      is_finished: false,
    },
    {
      player_uuid: 'rival-cyan-31',
      player_name: 'Piastri',
      car_name: 'Cyan Comet',
      current_sector: 3,
      position_in_sector: 2,
      total_value: 95,
      current_lap: 3,
      is_finished: false,
    },
  ],
};

export default function PreviewRacePage() {
  return (
    <RaceInterface
      carData={null}
      performancePreview={null}
      turnPhase={null}
      localView={mockLocalView}
      boostAvailability={null}
      lapHistory={null}
      selectedBoost={null}
      isSubmitting={false}
      hasSubmittedThisTurn={false}
      isPolling={false}
      isLoadingPreview={false}
      isLoadingSubmit={false}
      isAnyLoading={false}
      onBoostSelect={() => {}}
      onSubmitAction={() => {}}
      raceUuid="preview-race-uuid-0001"
      playerUuid={PLAYER_UUID}
    />
  );
}
