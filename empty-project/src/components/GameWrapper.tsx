import { useParams } from 'react-router-dom';
import { PlayerGameProvider } from '../contexts/PlayerGameContext';
import { PlayerGameInterface } from './player-game-interface';
import { useAuthContext } from '../contexts/AuthContext';

function GameWrapper() {
  const { raceUuid } = useParams<{ raceUuid: string }>();
  const { user } = useAuthContext();

  if (!raceUuid) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-2">Invalid Race</h2>
          <p className="text-gray-300">Race UUID is required.</p>
        </div>
      </div>
    );
  }

  if (!user?.uuid) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-2xl font-bold mb-2">Authentication Required</h2>
          <p className="text-gray-300">Please log in to access the game.</p>
        </div>
      </div>
    );
  }

  return (
    <PlayerGameProvider>
      <PlayerGameInterface
        raceUuid={raceUuid}
        playerUuid={user.uuid}
        onRaceComplete={(position) => {
          console.log(`Race completed! Final position: ${position}`);
          // Could redirect to results page or show modal
        }}
        onError={(error) => {
          console.error('Game error:', error);
          // Could show error notification
        }}
      />
    </PlayerGameProvider>
  );
}

export default GameWrapper;
