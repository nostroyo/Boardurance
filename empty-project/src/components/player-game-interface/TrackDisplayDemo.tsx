import React, { useState } from 'react';
import { TrackDisplayRedesign } from './TrackDisplayRedesign';
import { BoostControlPanel } from './BoostControlPanel';
import type { LocalView } from '../../types/race-api';
import type { AnimationState, TurnPhase } from '../../types/race';

// Demo component to test TrackDisplayRedesign functionality
export const TrackDisplayDemo: React.FC = () => {
  const [isAnimating, setIsAnimating] = useState(false);
  const [selectedBoost, setSelectedBoost] = useState<number | null>(null);
  const [hasSubmitted, setHasSubmitted] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Available boost values for demo
  const availableBoosts = [0, 1, 2, 3, 4, 5];
  const turnPhase: TurnPhase = hasSubmitted ? 'Processing' : 'WaitingForPlayers';

  // Mock data for testing the TrackDisplayRedesign component
  const mockLocalView: LocalView = {
    center_sector: 5,
    visible_sectors: [
      {
        id: 1,
        name: 'Sector 1',
        min_value: 5,
        max_value: 9,
        slot_capacity: 3,
        sector_type: 'Start',
        current_occupancy: 1,
      },
      {
        id: 2,
        name: 'Sector 2',
        min_value: 7,
        max_value: 11,
        slot_capacity: 4,
        sector_type: 'Straight',
        current_occupancy: 0,
      },
      {
        id: 3,
        name: 'Sector 3',
        min_value: 8,
        max_value: 12,
        slot_capacity: 4,
        sector_type: 'Straight',
        current_occupancy: 2,
      },
      {
        id: 4,
        name: 'Sector 4',
        min_value: 6,
        max_value: 10,
        slot_capacity: 3,
        sector_type: 'Curve',
        current_occupancy: 3,
      },
      {
        id: 5,
        name: 'Player Sector',
        min_value: 10,
        max_value: 15,
        slot_capacity: 5,
        sector_type: 'Straight',
        current_occupancy: 2,
      },
      {
        id: 6,
        name: 'Sector 6',
        min_value: 7,
        max_value: 11,
        slot_capacity: null, // Unlimited capacity
        sector_type: 'Curve',
        current_occupancy: 1,
      },
      {
        id: 7,
        name: 'Sector 7',
        min_value: 9,
        max_value: 13,
        slot_capacity: 2,
        sector_type: 'Straight',
        current_occupancy: 0,
      },
      {
        id: 8,
        name: 'Sector 8',
        min_value: 11,
        max_value: 16,
        slot_capacity: 4,
        sector_type: 'Finish',
        current_occupancy: 0,
      },
      {
        id: 9,
        name: 'Sector 9',
        min_value: 8,
        max_value: 12,
        slot_capacity: 3,
        sector_type: 'Straight',
        current_occupancy: 0,
      },
    ],
    visible_participants: [
      {
        player_uuid: 'player-1',
        player_name: 'Alice',
        car_name: 'Lightning Bolt',
        current_sector: 3,
        position_in_sector: 1,
        total_value: 10,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-2',
        player_name: 'Bob',
        car_name: 'Thunder Strike',
        current_sector: 3,
        position_in_sector: 2,
        total_value: 9,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-3',
        player_name: 'Charlie',
        car_name: 'Speed Demon',
        current_sector: 4,
        position_in_sector: 1,
        total_value: 11,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-4',
        player_name: 'Diana',
        car_name: 'Wind Runner',
        current_sector: 4,
        position_in_sector: 2,
        total_value: 8,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-5',
        player_name: 'Eve',
        car_name: 'Storm Chaser',
        current_sector: 4,
        position_in_sector: 3,
        total_value: 12,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'test-player',
        player_name: 'You',
        car_name: 'Your Car',
        current_sector: 5,
        position_in_sector: 1,
        total_value: 13,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-6',
        player_name: 'Frank',
        car_name: 'Rocket Racer',
        current_sector: 5,
        position_in_sector: 2,
        total_value: 7,
        current_lap: 2,
        is_finished: false,
      },
      {
        player_uuid: 'player-7',
        player_name: 'Grace',
        car_name: 'Turbo Boost',
        current_sector: 6,
        position_in_sector: 1,
        total_value: 14,
        current_lap: 2,
        is_finished: false,
      },
    ],
  };

  const mockAnimationState: AnimationState = {
    isAnimating,
    movements: [],
    duration: 1000,
  };

  const handleSectorClick = (sectorId: number) => {
    console.log(`Sector ${sectorId} clicked`);
  };

  const handleSlotClick = (sectorId: number, slotNumber: number) => {
    console.log(`Slot ${slotNumber} in sector ${sectorId} clicked`);
  };

  const handleBoostSelect = (boost: number) => {
    setSelectedBoost(boost);
    console.log(`Boost ${boost} selected`);
  };

  const handleValidateTurn = () => {
    if (selectedBoost === null) return;
    
    setIsSubmitting(true);
    console.log(`Validating turn with boost ${selectedBoost}`);
    
    // Simulate API call
    setTimeout(() => {
      setIsSubmitting(false);
      setHasSubmitted(true);
      console.log('Turn validated successfully');
    }, 2000);
  };

  const resetDemo = () => {
    setSelectedBoost(null);
    setHasSubmitted(false);
    setIsSubmitting(false);
    setIsAnimating(false);
  };

  const toggleAnimation = () => {
    setIsAnimating(!isAnimating);
    if (!isAnimating) {
      // Auto-stop animation after 3 seconds
      setTimeout(() => setIsAnimating(false), 3000);
    }
  };

  return (
    <div className="p-6 bg-gray-900 min-h-screen">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="text-center">
          <h1 className="text-3xl font-bold text-white mb-4">
            Race Interface Redesign Demo
          </h1>
          <p className="text-gray-400 mb-6">
            Testing the new bird's eye view track display with boost controls
          </p>
          
          <div className="flex justify-center space-x-4">
            <button
              onClick={toggleAnimation}
              className={`px-6 py-2 rounded-lg font-medium transition-colors ${
                isAnimating
                  ? 'bg-red-600 hover:bg-red-700 text-white'
                  : 'bg-blue-600 hover:bg-blue-700 text-white'
              }`}
            >
              {isAnimating ? 'Stop Animation' : 'Test Animation'}
            </button>
            
            <button
              onClick={resetDemo}
              className="px-6 py-2 rounded-lg font-medium bg-gray-600 hover:bg-gray-700 text-white transition-colors"
            >
              Reset Demo
            </button>
          </div>
        </div>

        {/* Main Interface Layout */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Track Display - Takes up 2 columns on large screens */}
          <div className="lg:col-span-2">
            <TrackDisplayRedesign
              localView={mockLocalView}
              playerUuid="test-player"
              animationState={mockAnimationState}
              onSectorClick={handleSectorClick}
              onSlotClick={handleSlotClick}
            />
          </div>

          {/* Boost Control Panel - Takes up 1 column */}
          <div className="lg:col-span-1">
            <BoostControlPanel
              selectedBoost={selectedBoost}
              availableBoosts={availableBoosts}
              onBoostSelect={handleBoostSelect}
              onValidateTurn={handleValidateTurn}
              isSubmitting={isSubmitting}
              hasSubmitted={hasSubmitted}
              turnPhase={turnPhase}
            />
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <h3 className="text-lg font-semibold text-white mb-3">Demo Features</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h4 className="text-md font-medium text-blue-400 mb-2">Track Display</h4>
              <ul className="text-gray-300 text-sm space-y-1">
                <li>✅ Limited sector view (max 2 before/after player)</li>
                <li>✅ Player sector centering (sector 5 highlighted)</li>
                <li>✅ 8-bit car sprites in position slots</li>
                <li>✅ Sector capacity indicators</li>
                <li>✅ Value range display</li>
                <li>✅ Smooth scrolling and animations</li>
              </ul>
            </div>
            <div>
              <h4 className="text-md font-medium text-green-400 mb-2">Boost Controls</h4>
              <ul className="text-gray-300 text-sm space-y-1">
                <li>✅ Prominent boost selection buttons (0-5)</li>
                <li>✅ Visual feedback on selection</li>
                <li>✅ Validate turn button</li>
                <li>✅ Loading states and confirmation</li>
                <li>✅ Turn submission workflow</li>
                <li>✅ Boost availability tracking</li>
              </ul>
            </div>
          </div>
        </div>

        {/* Debug Information */}
        <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
          <h3 className="text-lg font-semibold text-white mb-3">Debug Information</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h4 className="text-md font-medium text-yellow-400 mb-2">Participants in Sector 5</h4>
              <div className="text-gray-300 text-sm">
                {mockLocalView.visible_participants
                  .filter(p => p.current_sector === 5)
                  .map(p => (
                    <div key={p.player_uuid} className="mb-1">
                      {p.player_name} ({p.player_uuid}) - Position {p.position_in_sector}
                    </div>
                  ))
                }
              </div>
            </div>
            <div>
              <h4 className="text-md font-medium text-yellow-400 mb-2">Player UUID</h4>
              <div className="text-gray-300 text-sm">
                Current: "test-player"
              </div>
              <h4 className="text-md font-medium text-yellow-400 mb-2 mt-4">Center Sector</h4>
              <div className="text-gray-300 text-sm">
                {mockLocalView.center_sector}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};