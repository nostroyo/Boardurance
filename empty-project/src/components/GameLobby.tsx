import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuthContext } from '../contexts/AuthContext';

interface Race {
  uuid: string;
  name: string;
  status: string;
  participants: number;
  maxParticipants: number;
  current_lap: number;
  total_laps: number;
}

function GameLobby() {
  const { user } = useAuthContext();
  const [races, setRaces] = useState<Race[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Fetch available races
    const fetchRaces = async () => {
      try {
        const response = await fetch('http://localhost:3000/api/v1/races');
        if (response.ok) {
          const data = await response.json();
          setRaces(data || []);
        } else {
          setError('Failed to load races');
        }
      } catch (err) {
        setError('Network error while loading races');
      } finally {
        setLoading(false);
      }
    };

    fetchRaces();
  }, []);

  const joinRace = async (raceUuid: string) => {
    if (!user?.uuid) {
      setError('User not authenticated');
      return;
    }

    try {
      console.log('Fetching player data for:', user.uuid);
      
      // Get player's cars to select one for the race
      const playerResponse = await fetch(`http://localhost:3000/api/v1/players/${user.uuid}`, {
        credentials: 'include',
      });

      if (!playerResponse.ok) {
        const errorText = await playerResponse.text();
        console.error('Failed to load player data:', errorText);
        setError(`Failed to load player data: ${playerResponse.status}`);
        return;
      }

      const playerData = await playerResponse.json();
      console.log('Player data:', playerData);
      
      // Use the first car that has pilots assigned
      const carWithPilots = playerData.cars?.find((car: any) => car.pilot_uuids && car.pilot_uuids.length === 3);
      
      if (!carWithPilots) {
        console.error('No car with pilots found. Cars:', playerData.cars);
        setError('No car with 3 pilots found. Please assign pilots to your car first.');
        return;
      }

      console.log('Using car:', carWithPilots.uuid, 'with pilots:', carWithPilots.pilot_uuids);

      // Select the first pilot as the active pilot
      const pilotUuid = carWithPilots.pilot_uuids[0];

      const joinData = {
        player_uuid: user.uuid,
        car_uuid: carWithPilots.uuid,
        pilot_uuid: pilotUuid,
      };

      console.log('Joining race with data:', joinData);

      const response = await fetch(`http://localhost:3000/api/v1/races/${raceUuid}/join`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify(joinData),
      });

      console.log('Join race response status:', response.status);

      if (response.ok) {
        console.log('Successfully joined race');
        // Refresh races list
        window.location.reload();
      } else {
        const errorText = await response.text();
        console.error('Failed to join race:', errorText);
        try {
          const errorData = JSON.parse(errorText);
          setError(errorData.error || 'Failed to join race');
        } catch {
          setError(`Failed to join race: ${response.status} - ${errorText}`);
        }
      }
    } catch (err) {
      console.error('Error joining race:', err);
      setError(`Network error while joining race: ${err instanceof Error ? err.message : String(err)}`);
    }
  };

  const createTestRace = async () => {
    try {
      const testRaceData = {
        name: `Test Race ${Date.now()}`,
        track_name: "Test Track",
        total_laps: 3,
        sectors: [
          { id: 0, name: "Start", min_value: 0, max_value: 10, slot_capacity: null, sector_type: "Start" },
          { id: 1, name: "Sector 1", min_value: 5, max_value: 15, slot_capacity: 5, sector_type: "Straight" },
          { id: 2, name: "Sector 2", min_value: 10, max_value: 20, slot_capacity: 5, sector_type: "Curve" },
          { id: 3, name: "Sector 3", min_value: 15, max_value: 25, slot_capacity: 5, sector_type: "Straight" },
          { id: 4, name: "Finish", min_value: 20, max_value: 30, slot_capacity: null, sector_type: "Finish" }
        ]
      };

      const response = await fetch('http://localhost:3000/api/v1/races', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(testRaceData),
      });

      if (response.ok) {
        const data = await response.json();
        // Refresh races list
        window.location.reload();
      } else {
        setError('Failed to create race');
      }
    } catch (err) {
      setError('Network error while creating race');
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
          <p className="text-lg">Loading races...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <div className="container mx-auto px-4 py-8">
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold mb-4">🏁 Racing Lobby</h1>
          <p className="text-gray-300 text-lg">Welcome, {user?.team_name}! Choose your race.</p>
        </div>

        {error && (
          <div className="bg-red-600 text-white p-4 rounded-lg mb-6 text-center">
            {error}
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Available Races */}
          <div className="lg:col-span-2">
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h2 className="text-2xl font-bold mb-4">Available Races</h2>
              
              {races.length === 0 ? (
                <div className="text-center py-8">
                  <div className="text-gray-400 text-6xl mb-4">🏎️</div>
                  <p className="text-gray-400 text-lg mb-4">No races available</p>
                  <p className="text-gray-500">Create a new race to get started!</p>
                </div>
              ) : (
                <div className="space-y-4">
                  {races.map((race) => (
                    <div key={race.uuid} className="bg-gray-700 rounded-lg p-4 border border-gray-600">
                      <div className="flex justify-between items-center mb-2">
                        <h3 className="text-xl font-semibold">{race.name}</h3>
                        <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                          race.status === 'Waiting' ? 'bg-yellow-600 text-yellow-100' :
                          race.status === 'InProgress' ? 'bg-green-600 text-green-100' :
                          race.status === 'Finished' ? 'bg-blue-600 text-blue-100' :
                          'bg-gray-600 text-gray-100'
                        }`}>
                          {race.status}
                        </span>
                      </div>
                      
                      <div className="grid grid-cols-2 gap-4 text-sm text-gray-300 mb-4">
                        <div>
                          <span className="text-gray-400">Participants:</span>
                          <span className="ml-2 font-medium">{race.participants?.length || 0}</span>
                        </div>
                        <div>
                          <span className="text-gray-400">Lap:</span>
                          <span className="ml-2 font-medium">{race.current_lap} / {race.total_laps}</span>
                        </div>
                      </div>
                      
                      <div className="flex space-x-3">
                        {race.status === 'Waiting' && (
                          <button 
                            onClick={() => joinRace(race.uuid)}
                            className="bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded font-medium transition-colors"
                          >
                            Join Race
                          </button>
                        )}
                        {race.status === 'InProgress' && (
                          <Link
                            to={`/game/${race.uuid}`}
                            className="bg-green-600 hover:bg-green-700 px-4 py-2 rounded font-medium transition-colors"
                          >
                            Enter Race
                          </Link>
                        )}
                        <button className="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded font-medium transition-colors">
                          View Details
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Actions Panel */}
          <div className="space-y-6">
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h2 className="text-xl font-bold mb-4">Quick Actions</h2>
              
              <div className="space-y-4">
                <button
                  onClick={createTestRace}
                  className="w-full bg-green-600 hover:bg-green-700 px-4 py-3 rounded-lg font-medium transition-colors"
                >
                  Create Test Race
                </button>
                
                <button className="w-full bg-blue-600 hover:bg-blue-700 px-4 py-3 rounded-lg font-medium transition-colors">
                  Quick Match
                </button>
                
                <Link
                  to="/team"
                  className="block w-full bg-purple-600 hover:bg-purple-700 px-4 py-3 rounded-lg font-medium transition-colors text-center"
                >
                  Manage Team
                </Link>
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h2 className="text-xl font-bold mb-4">Player Stats</h2>
              
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-400">Team:</span>
                  <span className="font-medium">{user?.team_name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Races Won:</span>
                  <span className="font-medium">0</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Total Races:</span>
                  <span className="font-medium">0</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">Rank:</span>
                  <span className="font-medium">Rookie</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default GameLobby;