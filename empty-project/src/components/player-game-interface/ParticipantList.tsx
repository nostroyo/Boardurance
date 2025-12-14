import React from 'react';
import type { LocalView } from '../../types/race-api';

export interface ParticipantListProps {
  participants: LocalView['visible_participants'];
  playerUuid: string;
}

// CSS animations for participant transitions
const participantAnimationStyles = `
  @keyframes slide-in {
    from { 
      opacity: 0; 
      transform: translateX(-10px); 
    }
    to { 
      opacity: 1; 
      transform: translateX(0); 
    }
  }
  
  @keyframes sector-highlight {
    0%, 100% { box-shadow: 0 0 0 rgba(59, 130, 246, 0); }
    50% { box-shadow: 0 0 15px rgba(59, 130, 246, 0.3); }
  }
  
  .animate-slide-in {
    animation: slide-in 0.4s ease-out;
  }
  
  .animate-sector-highlight {
    animation: sector-highlight 2s ease-in-out infinite;
  }
`;

const ParticipantListComponent: React.FC<ParticipantListProps> = ({ participants, playerUuid }) => {
  // Format player display name
  const getPlayerDisplayName = (participant: LocalView['visible_participants'][0]): string => {
    if (participant.player_uuid === playerUuid) {
      return 'You';
    }
    // Use player name if available, otherwise show last 4 characters of UUID
    return participant.player_name || `Player ${participant.player_uuid.slice(-4)}`;
  };

  // Get position badge color
  const getPositionBadgeColor = (position: number): string => {
    if (position === 1) return 'bg-yellow-500 text-gray-900'; // Gold
    if (position === 2) return 'bg-gray-400 text-gray-900'; // Silver
    if (position === 3) return 'bg-orange-600 text-white'; // Bronze
    return 'bg-gray-500 text-white';
  };

  // Empty state
  if (participants.length === 0) {
    return (
      <div className="text-center py-4 text-gray-400 text-sm bg-gray-800 rounded">
        No participants in this sector
      </div>
    );
  }

  return (
    <div>
      {/* Inject CSS animations */}
      <style>{participantAnimationStyles}</style>
      <span className="text-gray-400 text-sm block mb-2">
        Participants ({participants.length}):
      </span>
      <div className="grid grid-cols-1 gap-2">
        {participants.map((participant, index) => {
          const isPlayer = participant.player_uuid === playerUuid;
          const position = index + 1;

          return (
            <div
              key={participant.player_uuid}
              style={{ animationDelay: `${index * 50}ms` }}
              className={`flex items-center justify-between p-3 rounded-lg text-sm transition-all duration-300 transform hover:scale-[1.02] animate-slide-in ${
                isPlayer
                  ? 'bg-gradient-to-r from-blue-800 to-blue-700 border border-blue-500 shadow-lg ring-1 ring-blue-400/50 animate-sector-highlight'
                  : 'bg-gray-600 hover:bg-gray-550 border border-gray-500 hover:border-gray-400'
              }`}
              role="listitem"
              aria-label={`${getPlayerDisplayName(participant)} - Position ${position}`}
            >
              {/* Left side: Position and name */}
              <div className="flex items-center space-x-3">
                <span
                  className={`w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold ${getPositionBadgeColor(
                    position,
                  )}`}
                  aria-label={`Position ${position}`}
                >
                  {position}
                </span>
                <div>
                  <div className="flex items-center space-x-2">
                    <div
                      className={`${
                        isPlayer ? 'text-blue-100 font-bold' : 'text-gray-200 font-medium'
                      }`}
                    >
                      {isPlayer && '👤 '}
                      {getPlayerDisplayName(participant)}
                    </div>
                    {participant.is_finished && (
                      <span className="text-xs text-green-400 bg-green-900/30 px-2 py-0.5 rounded-full">
                        ✓ Finished
                      </span>
                    )}
                  </div>
                  <div className={`text-xs ${isPlayer ? 'text-blue-300' : 'text-gray-400'}`}>
                    🏎️ {participant.car_name}
                  </div>
                </div>
              </div>

              {/* Right side: Stats */}
              <div className="text-right">
                <div className={`font-bold ${isPlayer ? 'text-blue-100' : 'text-white'}`}>
                  ⚡ {participant.total_value}
                </div>
                <div className={`text-xs ${isPlayer ? 'text-blue-300' : 'text-gray-400'}`}>
                  🏁 Lap {participant.current_lap} | #️⃣ Pos {participant.position_in_sector}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export const ParticipantList = React.memo(ParticipantListComponent);
