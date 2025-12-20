/**
 * TestRaceInterface - Simple test component to verify sprites and buttons are working
 */

import React from 'react';
import { CarSprite } from './player-game-interface/CarSprite';
import { BoostControlPanel } from './player-game-interface/BoostControlPanel';
import type { LocalView } from '../types/race-api';

const TestRaceInterface: React.FC = () => {
  // Mock participant data for testing
  const mockParticipant: LocalView['visible_participants'][0] = {
    player_uuid: 'test-player-1',
    player_name: 'Test Player',
    car_name: 'Test Car',
    current_sector: 1,
    position_in_sector: 1,
    total_value: 45,
    current_lap: 1,
    is_finished: false,
  };

  const mockParticipant2: LocalView['visible_participants'][0] = {
    player_uuid: 'test-player-2',
    player_name: 'Other Player',
    car_name: 'Other Car',
    current_sector: 1,
    position_in_sector: 2,
    total_value: 42,
    current_lap: 1,
    is_finished: false,
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <div className="max-w-4xl mx-auto space-y-8">
        <h1 className="text-3xl font-bold text-center">🧪 Race Interface Test</h1>
        
        {/* Car Sprites Test */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-xl font-bold mb-4">Car Sprites Test</h2>
          <div className="space-y-4">
            <div>
              <h3 className="text-lg mb-2">Player Car (You)</h3>
              <CarSprite 
                participant={mockParticipant}
                isPlayer={true}
                size="large"
                animationState="highlighted"
              />
            </div>
            
            <div>
              <h3 className="text-lg mb-2">Other Player Car</h3>
              <CarSprite 
                participant={mockParticipant2}
                isPlayer={false}
                size="large"
                animationState="idle"
              />
            </div>
            
            <div>
              <h3 className="text-lg mb-2">Different Sizes</h3>
              <div className="flex items-center space-x-4">
                <div className="text-center">
                  <p className="text-sm mb-2">Small</p>
                  <CarSprite 
                    participant={mockParticipant}
                    isPlayer={true}
                    size="small"
                  />
                </div>
                <div className="text-center">
                  <p className="text-sm mb-2">Medium</p>
                  <CarSprite 
                    participant={mockParticipant}
                    isPlayer={true}
                    size="medium"
                  />
                </div>
                <div className="text-center">
                  <p className="text-sm mb-2">Large</p>
                  <CarSprite 
                    participant={mockParticipant}
                    isPlayer={true}
                    size="large"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Boost Control Panel Test */}
        <div className="bg-gray-800 rounded-lg p-6">
          <h2 className="text-xl font-bold mb-4">Boost Control Panel Test</h2>
          <BoostControlPanel
            selectedBoost={null}
            availableBoosts={[0, 1, 2, 3, 4]}
            onBoostSelect={(boost) => console.log('Boost selected:', boost)}
            onValidateTurn={() => console.log('Turn validated')}
            isSubmitting={false}
            hasSubmitted={false}
            turnPhase="WaitingForPlayers"
          />
        </div>

        {/* Status */}
        <div className="bg-green-900 border border-green-700 rounded-lg p-4 text-center">
          <p className="text-green-200 font-bold">✓ Test Components Loaded</p>
          <p className="text-green-300 text-sm mt-1">
            If you can see car sprites above and boost buttons (0-5), the components are working correctly.
          </p>
        </div>
      </div>
    </div>
  );
};

export default TestRaceInterface;