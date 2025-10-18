import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

interface Player {
  uuid: string;
  email: string;
  team_name: string;
  cars: Car[];
  pilots: Pilot[];
  engines: Engine[];
  bodies: Body[];
}

interface Car {
  uuid: string;
  name: string;
  pilot_uuids: string[]; // Array of up to 3 pilots
  engine_uuid?: string;
  body_uuid?: string;
  is_equipped: boolean;
}

interface Pilot {
  uuid: string;
  name: string;
  pilot_class: string;
  rarity: string;
  performance: {
    straight_value: number;
    curve_value: number;
  };
}

interface Engine {
  uuid: string;
  name: string;
  rarity: string;
  straight_value: number;
  curve_value: number;
}

interface Body {
  uuid: string;
  name: string;
  rarity: string;
  straight_value: number;
  curve_value: number;
}

function TeamPage() {
  const [player, setPlayer] = useState<Player | null>(null);
  const [loading, setLoading] = useState(true);
  const [error] = useState('');

  useEffect(() => {
    // For demo purposes, we'll use mock data
    // In a real app, you'd fetch this from your API
    const mockPlayer: Player = {
      uuid: "player-123",
      email: "player@example.com",
      team_name: "Lightning Racers",
      cars: [
        {
          uuid: "car-1",
          name: "Thunder Bolt",
          pilot_uuids: ["pilot-1", "pilot-2"],
          engine_uuid: "engine-1",
          body_uuid: "body-1",
          is_equipped: true
        },
        {
          uuid: "car-2", 
          name: "Speed Demon",
          pilot_uuids: [],
          engine_uuid: undefined,
          body_uuid: undefined,
          is_equipped: false
        }
      ],
      pilots: [
        {
          uuid: "pilot-1",
          name: "Alex Thunder",
          pilot_class: "Speedster",
          rarity: "Rare",
          performance: { straight_value: 85, curve_value: 70 }
        },
        {
          uuid: "pilot-2",
          name: "Sarah Velocity",
          pilot_class: "Technician", 
          rarity: "Epic",
          performance: { straight_value: 75, curve_value: 90 }
        },
        {
          uuid: "pilot-3",
          name: "Mike Drift",
          pilot_class: "Racer",
          rarity: "Common",
          performance: { straight_value: 70, curve_value: 80 }
        }
      ],
      engines: [
        {
          uuid: "engine-1",
          name: "Turbo V8",
          rarity: "Rare",
          straight_value: 90,
          curve_value: 60
        },
        {
          uuid: "engine-2", 
          name: "Electric Motor",
          rarity: "Epic",
          straight_value: 85,
          curve_value: 85
        }
      ],
      bodies: [
        {
          uuid: "body-1",
          name: "Aerodynamic Frame",
          rarity: "Rare", 
          straight_value: 70,
          curve_value: 95
        },
        {
          uuid: "body-2",
          name: "Heavy Chassis",
          rarity: "Common",
          straight_value: 85,
          curve_value: 65
        }
      ]
    };

    setTimeout(() => {
      setPlayer(mockPlayer);
      setLoading(false);
    }, 500);
  }, []);

  const getAssignedPilots = (pilotUuids: string[]) => {
    return pilotUuids.map(uuid => player?.pilots.find(p => p.uuid === uuid)).filter(Boolean) as Pilot[];
  };

  const getAssignedEngine = (engineUuid?: string) => {
    return player?.engines.find(e => e.uuid === engineUuid);
  };

  const getAssignedBody = (bodyUuid?: string) => {
    return player?.bodies.find(b => b.uuid === bodyUuid);
  };

  const getAvailablePilots = () => {
    const assignedPilotIds = player?.cars.flatMap(car => car.pilot_uuids) || [];
    return player?.pilots.filter(pilot => !assignedPilotIds.includes(pilot.uuid)) || [];
  };

  const getAvailableEngines = () => {
    const assignedEngineIds = player?.cars.map(car => car.engine_uuid).filter(Boolean) || [];
    return player?.engines.filter(engine => !assignedEngineIds.includes(engine.uuid)) || [];
  };

  const getAvailableBodies = () => {
    const assignedBodyIds = player?.cars.map(car => car.body_uuid).filter(Boolean) || [];
    return player?.bodies.filter(body => !assignedBodyIds.includes(body.uuid)) || [];
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-xl">Loading team data...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center">
        <div className="text-red-600">{error}</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-100 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-6 flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold text-gray-800">
              TEAM: {player?.team_name}
            </h1>
          </div>
          <Link
            to="/dashboard"
            className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg"
          >
            Back to Dashboard
          </Link>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Side - Cars */}
          <div className="lg:col-span-2 space-y-6">
            {player?.cars.map((car, index) => (
              <div key={car.uuid} className="bg-white rounded-lg shadow-lg p-6 border-2 border-gray-300">
                <h2 className="text-xl font-bold text-red-600 mb-4">
                  CAR{index + 1}: {car.name}
                </h2>
                
                <div className="grid grid-cols-2 gap-4">
                  {/* Left Side - Engine and Body stacked */}
                  <div className="space-y-4">
                    {/* Engine Section */}
                    <div className="border-2 border-gray-300 rounded-lg p-4 bg-gray-50 h-28 flex flex-col">
                      <h3 className="text-gray-700 font-semibold mb-2 text-sm">Engine</h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedEngine(car.engine_uuid) ? (
                          <div className="text-sm text-center">
                            <div className="font-medium text-gray-800 mb-1">{getAssignedEngine(car.engine_uuid)?.name}</div>
                            <div className="text-gray-600 text-xs">
                              {getAssignedEngine(car.engine_uuid)?.rarity} | 
                              S:{getAssignedEngine(car.engine_uuid)?.straight_value} C:{getAssignedEngine(car.engine_uuid)?.curve_value}
                            </div>
                          </div>
                        ) : (
                          <div className="text-gray-400 text-sm text-center">No engine assigned</div>
                        )}
                      </div>
                    </div>

                    {/* Body Section */}
                    <div className="border-2 border-gray-300 rounded-lg p-4 bg-gray-50 h-28 flex flex-col">
                      <h3 className="text-gray-700 font-semibold mb-2 text-sm">Body</h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedBody(car.body_uuid) ? (
                          <div className="text-sm text-center">
                            <div className="font-medium text-gray-800 mb-1">{getAssignedBody(car.body_uuid)?.name}</div>
                            <div className="text-gray-600 text-xs">
                              {getAssignedBody(car.body_uuid)?.rarity} | 
                              S:{getAssignedBody(car.body_uuid)?.straight_value} C:{getAssignedBody(car.body_uuid)?.curve_value}
                            </div>
                          </div>
                        ) : (
                          <div className="text-gray-400 text-sm text-center">No body assigned</div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Right Side - Pilots Section (full height) */}
                  <div className="border-2 border-gray-300 rounded-lg p-4 bg-gray-50 h-60 flex flex-col">
                    <h3 className="text-gray-700 font-semibold mb-3 text-sm">Pilots (3 max)</h3>
                    <div className="flex-1 flex flex-col justify-center space-y-3">
                      {[0, 1, 2].map((slotIndex) => {
                        const assignedPilots = getAssignedPilots(car.pilot_uuids);
                        const pilot = assignedPilots[slotIndex];
                        return (
                          <div key={slotIndex} className="flex items-center space-x-3 h-12 border border-gray-200 rounded px-3 bg-white">
                            {/* Helmet Icon */}
                            <div className="w-8 h-8 flex-shrink-0">
                              {pilot ? (
                                <svg className="w-8 h-8 text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                                  <path fillRule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clipRule="evenodd" />
                                </svg>
                              ) : (
                                <svg className="w-8 h-8 text-gray-300" fill="currentColor" viewBox="0 0 20 20">
                                  <path fillRule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clipRule="evenodd" />
                                </svg>
                              )}
                            </div>
                            {/* Pilot Info */}
                            <div className="flex-1 min-w-0">
                              {pilot ? (
                                <div className="text-sm">
                                  <div className="font-medium text-gray-800 truncate">{pilot.name}</div>
                                  <div className="text-gray-500 text-xs">
                                    {pilot.pilot_class} | S:{pilot.performance.straight_value} C:{pilot.performance.curve_value}
                                  </div>
                                </div>
                              ) : (
                                <div className="text-gray-400 text-sm">Empty pilot slot</div>
                              )}
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Right Side - Inventory */}
          <div className="space-y-4">
            {/* Inventory Pilots */}
            <div className="bg-white rounded-lg shadow-lg p-4 border-2 border-gray-300 h-64">
              <h3 className="text-gray-700 font-bold text-lg mb-4">INVENTORY PILOTS</h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailablePilots().map((pilot) => (
                  <div key={pilot.uuid} className="border border-gray-300 rounded p-3 bg-gray-50 h-16 flex flex-col justify-center">
                    <div className="font-medium text-sm text-gray-800">{pilot.name}</div>
                    <div className="text-xs text-gray-600">{pilot.pilot_class} - {pilot.rarity}</div>
                    <div className="text-xs text-gray-500">
                      S:{pilot.performance.straight_value} C:{pilot.performance.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailablePilots().length === 0 && (
                  <div className="text-gray-400 text-sm text-center h-16 flex items-center justify-center">All pilots assigned</div>
                )}
              </div>
            </div>

            {/* Inventory Bodies */}
            <div className="bg-white rounded-lg shadow-lg p-4 border-2 border-gray-300 h-64">
              <h3 className="text-gray-700 font-bold text-lg mb-4">Inventory Bodies</h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableBodies().map((body) => (
                  <div key={body.uuid} className="border border-gray-300 rounded p-3 bg-gray-50 h-16 flex flex-col justify-center">
                    <div className="font-medium text-sm text-gray-800">{body.name}</div>
                    <div className="text-xs text-gray-600">{body.rarity}</div>
                    <div className="text-xs text-gray-500">
                      S:{body.straight_value} C:{body.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailableBodies().length === 0 && (
                  <div className="text-gray-400 text-sm text-center h-16 flex items-center justify-center">All bodies assigned</div>
                )}
              </div>
            </div>

            {/* Inventory Engines */}
            <div className="bg-white rounded-lg shadow-lg p-4 border-2 border-gray-300 h-64">
              <h3 className="text-gray-700 font-bold text-lg mb-4">Inventory Engines</h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableEngines().map((engine) => (
                  <div key={engine.uuid} className="border border-gray-300 rounded p-3 bg-gray-50 h-16 flex flex-col justify-center">
                    <div className="font-medium text-sm text-gray-800">{engine.name}</div>
                    <div className="text-xs text-gray-600">{engine.rarity}</div>
                    <div className="text-xs text-gray-500">
                      S:{engine.straight_value} C:{engine.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailableEngines().length === 0 && (
                  <div className="text-gray-400 text-sm text-center h-16 flex items-center justify-center">All engines assigned</div>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default TeamPage;