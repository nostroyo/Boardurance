import React from 'react';
import { SectorGrid } from './SectorGrid';
import type { LocalView } from '../../types/race-api';

// Demo component to showcase the new SectorGrid component
export const SectorGridDemo: React.FC = () => {
  // Mock data for demonstration
  const mockSector: LocalView['visible_sectors'][0] = {
    id: 1,
    name: 'Demo Sector',
    min_value: 10,
    max_value: 20,
    slot_capacity: 5,
    sector_type: 'Straight',
    current_occupancy: 3,
  };

  const mockParticipants: LocalView['visible_participants'] = [
    {
      player_uuid: 'player-1',
      player_name: 'Alice',
      car_name: 'Red Lightning',
      current_sector: 1,
      position_in_sector: 1,
      total_value: 15,
      current_lap: 1,
      is_finished: false,
    },
    {
      player_uuid: 'player-2',
      player_name: 'Bob',
      car_name: 'Blue Thunder',
      current_sector: 1,
      position_in_sector: 3,
      total_value: 18,
      current_lap: 1,
      is_finished: false,
    },
    {
      player_uuid: 'player-3',
      player_name: 'Charlie',
      car_name: 'Green Machine',
      current_sector: 1,
      position_in_sector: 5,
      total_value: 12,
      current_lap: 1,
      is_finished: false,
    },
  ];

  return (
    <div className="p-8 bg-gray-900 min-h-screen">
      <h1 className="text-2xl font-bold text-white mb-6">SectorGrid Component Demo</h1>
      
      <div className="max-w-2xl">
        <SectorGrid
          sector={mockSector}
          participants={mockParticipants}
          isPlayerSector={true}
          playerUuid="player-1"
          onSectorClick={(sectorId) => console.log('Sector clicked:', sectorId)}
          onSlotClick={(sectorId, slotNumber) => console.log('Slot clicked:', sectorId, slotNumber)}
        />
      </div>
      
      <div className="mt-8 text-gray-400 text-sm">
        <p>This demo shows the new SectorGrid component with:</p>
        <ul className="list-disc list-inside mt-2 space-y-1">
          <li>Linear sector layout with position slots (1-5)</li>
          <li>8-bit style car sprites (showing car initials)</li>
          <li>Player sector highlighting (blue border and "YOU" badge)</li>
          <li>Sector information (value range, capacity)</li>
          <li>Interactive position slots with hover effects</li>
        </ul>
      </div>
    </div>
  );
};