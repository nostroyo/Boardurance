import { useParams } from 'react-router-dom';
import { RaceContainer } from './player-game-interface/RaceContainer';
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
    <RaceContainer
      raceUuid={raceUuid}
      playerUuid={user.uuid}
      onRaceComplete={(_position) => {
        // Could redirect to results page or show modal
      }}
      onError={(_error) => {
        // Could show error notification
      }}
      onReturnToLobby={() => {
        // TODO: Navigate back to lobby
      }}
    />
  );
}

export default GameWrapper;
