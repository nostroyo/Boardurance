import React from 'react';
import { CarSprite } from './CarSprite';
import { BoostControlPanel } from './BoostControlPanel';
import type { LocalView } from '../../types/race-api';
import type { TurnPhase } from '../../types/race';

export const SimpleDemo: React.FC = () => {
  // Simple test participant
  const testParticipant: LocalView['visible_participants'][0] = {
    player_uuid: 'test-player',
    player_name: 'Test Player',
    car_name: 'Test Car',
    current_sector: 5,
    position_in_sector: 1,
    total_value: 10,
    current_lap: 1,
    is_finished: false,
  };

  const handleBoostSelect = (boost: number) => {
    console.log(`Boost ${boost} selected`);
  };

  const handleValidateTurn = () => {
    console.log('Turn validated');
  };

  return (
    <div className="p-8 bg-gray-900 min-h-screen">
      <div className="max-w-4xl mx-auto space-y-8">
        <h1 className="text-3xl font-bold text-white text-center">Simple Demo Test</h1>
        
        {/* Test Car Sprite */}
        <div className="bg-gray-800 p-6 rounded-lg">
          <h2 className="text-xl font-bold text-white mb-4">Car Sprite Test</h2>
          <div className="flex items-center justify-center space-x-4">
            <div className="text-center">
              <p className="text-gray-400 mb-2">Small Size</p>
              <CarSprite
                participant={testParticipant}
                isPlayer={true}
                size="small"
                animationState="highlighted"
              />
            </div>
            <div className="text-center">
              <p className="text-gray-400 mb-2">Medium Size</p>
              <CarSprite
                participant={testParticipant}
                isPlayer={true}
                size="medium"
                animationState="idle"
              />
            </div>
            <div className="text-center">
              <p className="text-gray-400 mb-2">Large Size</p>
              <CarSprite
                participant={testParticipant}
                isPlayer={true}
                size="large"
                animationState="moving"
              />
            </div>
          </div>
        </div>

        {/* Test Boost Control Panel */}
        <div className="bg-gray-800 p-6 rounded-lg">
          <h2 className="text-xl font-bold text-white mb-4">Boost Control Panel Test</h2>
          <div className="max-w-md mx-auto">
            <BoostControlPanel
              selectedBoost={null}
              availableBoosts={[0, 1, 2, 3, 4, 5]}
              onBoostSelect={handleBoostSelect}
              onValidateTurn={handleValidateTurn}
              isSubmitting={false}
              hasSubmitted={false}
              turnPhase={'WaitingForPlayers' as TurnPhase}
            />
          </div>
        </div>

        {/* Debug Info */}
        <div className="bg-gray-800 p-6 rounded-lg">
          <h2 className="text-xl font-bold text-white mb-4">Debug Info</h2>
          <pre className="text-gray-300 text-sm">
            {JSON.stringify(testParticipant, null, 2)}
          </pre>
        </div>
      </div>
    </div>
  );
};