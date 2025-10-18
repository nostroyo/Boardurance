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
    <div className="min-h-screen bg-gray-900 p-6 relative overflow-hidden">
      {/* Racing Background Pattern */}
      <div className="absolute inset-0 opacity-5">
        <div className="absolute top-10 left-10 transform rotate-12">
          <svg width="200" height="80" viewBox="0 0 200 80" className="text-gray-600">
            <path d="M20 40 L40 20 L160 20 L180 40 L160 60 L40 60 Z" fill="currentColor" />
            <circle cx="50" cy="40" r="15" fill="none" stroke="currentColor" strokeWidth="2" />
            <circle cx="150" cy="40" r="15" fill="none" stroke="currentColor" strokeWidth="2" />
          </svg>
        </div>
        <div className="absolute top-32 right-20 transform -rotate-6">
          <svg width="150" height="60" viewBox="0 0 150 60" className="text-gray-600">
            <path d="M15 30 L30 15 L120 15 L135 30 L120 45 L30 45 Z" fill="currentColor" />
            <circle cx="40" cy="30" r="12" fill="none" stroke="currentColor" strokeWidth="2" />
            <circle cx="110" cy="30" r="12" fill="none" stroke="currentColor" strokeWidth="2" />
          </svg>
        </div>
        <div className="absolute bottom-20 left-1/3 transform rotate-3">
          <svg width="180" height="70" viewBox="0 0 180 70" className="text-gray-600">
            <path d="M18 35 L36 18 L144 18 L162 35 L144 52 L36 52 Z" fill="currentColor" />
            <circle cx="48" cy="35" r="14" fill="none" stroke="currentColor" strokeWidth="2" />
            <circle cx="132" cy="35" r="14" fill="none" stroke="currentColor" strokeWidth="2" />
          </svg>
        </div>
      </div>
      
      {/* Racing stripes decoration */}
      <div className="absolute top-0 left-0 w-full h-2 bg-gradient-to-r from-red-600 via-white to-red-600 opacity-20"></div>
      <div className="absolute bottom-0 left-0 w-full h-2 bg-gradient-to-r from-red-600 via-white to-red-600 opacity-20"></div>
      <div className="max-w-7xl mx-auto relative z-10">
        {/* Header */}
        <div className="mb-6 flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">
              TEAM: {player?.team_name}
            </h1>
            <div className="flex items-center space-x-4 text-gray-300 text-sm">
              <span className="flex items-center">
                <svg className="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Active Team
              </span>
              <span>•</span>
              <span>{player?.cars.length}/2 Cars</span>
              <span>•</span>
              <span>{player?.pilots.length} Pilots</span>
            </div>
          </div>
          <Link
            to="/dashboard"
            className="bg-red-600 hover:bg-red-700 text-white px-6 py-3 rounded-lg font-semibold transition duration-200 shadow-lg"
          >
            ← Back to Dashboard
          </Link>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Side - Cars */}
          <div className="lg:col-span-2 space-y-6">
            {player?.cars.map((car, index) => (
              <div key={car.uuid} className="bg-gray-800 rounded-lg shadow-2xl p-6 border-2 border-gray-600 relative overflow-hidden">
                {/* Car header with racing number */}
                <div className="flex items-center justify-between mb-4">
                  <h2 className="text-xl font-bold text-white flex items-center">
                    <span className="bg-red-600 text-white px-3 py-1 rounded-full text-lg font-bold mr-3">
                      {index + 1}
                    </span>
                    {car.name}
                  </h2>
                  <div className="text-gray-400 text-sm">
                    {car.is_equipped ? (
                      <span className="text-green-400 flex items-center">
                        <svg className="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                        Ready to Race
                      </span>
                    ) : (
                      <span className="text-yellow-400 flex items-center">
                        <svg className="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                        </svg>
                        Needs Setup
                      </span>
                    )}
                  </div>
                </div>
                
                <div className="grid grid-cols-2 gap-4">
                  {/* Left Side - Engine and Body stacked */}
                  <div className="space-y-4">
                    {/* Engine Section */}
                    <div className="border-2 border-gray-600 rounded-lg p-4 bg-gray-700 h-28 flex flex-col shadow-lg">
                      <h3 className="text-orange-400 font-semibold mb-2 text-sm flex items-center">
                        <svg className="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M12 2L13.09 8.26L22 9L13.09 9.74L12 16L10.91 9.74L2 9L10.91 8.26L12 2M12 4.5L11.5 7.5L8.5 8L11.5 8.5L12 11.5L12.5 8.5L15.5 8L12.5 7.5L12 4.5M7 18C7 16.9 6.1 16 5 16S3 16.9 3 18 3.9 20 5 20 7 19.1 7 18M21 18C21 16.9 20.1 16 19 16S17 16.9 17 18 17.9 20 19 20 21 19.1 21 18Z"/>
                        </svg>
                        Engine
                      </h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedEngine(car.engine_uuid) ? (
                          <div className="text-sm text-center">
                            <div className="font-medium text-white mb-1">{getAssignedEngine(car.engine_uuid)?.name}</div>
                            <div className="text-gray-300 text-xs">
                              {getAssignedEngine(car.engine_uuid)?.rarity} | 
                              S:{getAssignedEngine(car.engine_uuid)?.straight_value} C:{getAssignedEngine(car.engine_uuid)?.curve_value}
                            </div>
                          </div>
                        ) : (
                          <div className="text-gray-500 text-sm text-center">No engine assigned</div>
                        )}
                      </div>
                    </div>

                    {/* Body Section */}
                    <div className="border-2 border-gray-600 rounded-lg p-4 bg-gray-700 h-28 flex flex-col shadow-lg">
                      <h3 className="text-blue-400 font-semibold mb-2 text-sm flex items-center">
                        <svg className="w-4 h-4 mr-2" fill="currentColor" viewBox="0 0 24 24">
                          <path d="M18.92 6.01C18.72 5.42 18.16 5 17.5 5H6.5C5.84 5 5.28 5.42 5.08 6.01L3 12V20C3 20.55 3.45 21 4 21H5C5.55 21 6 20.55 6 20V19H18V20C18 20.55 18.45 21 19 21H20C20.55 21 21 20.55 21 20V12L18.92 6.01M6.5 16C5.67 16 5 15.33 5 14.5S5.67 13 6.5 13 8 13.67 8 14.5 7.33 16 6.5 16M17.5 16C16.67 16 16 15.33 16 14.5S16.67 13 17.5 13 19 13.67 19 14.5 18.33 16 17.5 16M5.81 10L6.5 7H17.5L18.19 10H5.81Z"/>
                        </svg>
                        Body
                      </h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedBody(car.body_uuid) ? (
                          <div className="text-sm text-center">
                            <div className="font-medium text-white mb-1">{getAssignedBody(car.body_uuid)?.name}</div>
                            <div className="text-gray-300 text-xs">
                              {getAssignedBody(car.body_uuid)?.rarity} | 
                              S:{getAssignedBody(car.body_uuid)?.straight_value} C:{getAssignedBody(car.body_uuid)?.curve_value}
                            </div>
                          </div>
                        ) : (
                          <div className="text-gray-500 text-sm text-center">No body assigned</div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Right Side - Pilots Section (full height) */}
                  <div className="border-2 border-gray-600 rounded-lg p-4 bg-gray-700 h-60 flex flex-col shadow-lg">
                    <h3 className="text-green-400 font-semibold mb-3 text-sm flex items-center">
                      <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                        <path d="M9 10h6"/>
                      </svg>
                      Pilots (3 max)
                    </h3>
                    <div className="flex-1 flex flex-col justify-center space-y-3">
                      {[0, 1, 2].map((slotIndex) => {
                        const assignedPilots = getAssignedPilots(car.pilot_uuids);
                        const pilot = assignedPilots[slotIndex];
                        return (
                          <div key={slotIndex} className={`flex items-center space-x-3 h-12 border rounded px-3 transition-all duration-200 ${
                            pilot 
                              ? 'border-green-500 bg-gray-600 shadow-md' 
                              : 'border-gray-500 bg-gray-800 border-dashed'
                          }`}>
                            {/* Simple Helmet Icon */}
                            <div className="w-6 h-6 flex-shrink-0">
                              {pilot ? (
                                <svg className="w-6 h-6 text-green-400" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                                  <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                                  <path d="M9 10h6"/>
                                </svg>
                              ) : (
                                <svg className="w-6 h-6 text-gray-500" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                                  <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                                  <path d="M9 10h6"/>
                                </svg>
                              )}
                            </div>
                            {/* Pilot Info */}
                            <div className="flex-1 min-w-0">
                              {pilot ? (
                                <div className="text-sm">
                                  <div className="font-medium text-white truncate">{pilot.name}</div>
                                  <div className="text-gray-300 text-xs">
                                    {pilot.pilot_class} | S:{pilot.performance.straight_value} C:{pilot.performance.curve_value}
                                  </div>
                                </div>
                              ) : (
                                <div className="text-gray-500 text-sm">Empty pilot slot</div>
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
            <div className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64">
              <h3 className="text-green-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                  <path d="M9 10h6"/>
                </svg>
                INVENTORY PILOTS
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailablePilots().map((pilot) => (
                  <div key={pilot.uuid} className="border border-gray-600 rounded p-3 bg-gray-700 h-16 flex flex-col justify-center hover:bg-gray-600 transition-colors cursor-pointer">
                    <div className="font-medium text-sm text-white">{pilot.name}</div>
                    <div className="text-xs text-gray-300">{pilot.pilot_class} - {pilot.rarity}</div>
                    <div className="text-xs text-gray-400">
                      S:{pilot.performance.straight_value} C:{pilot.performance.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailablePilots().length === 0 && (
                  <div className="text-gray-500 text-sm text-center h-16 flex items-center justify-center">All pilots assigned</div>
                )}
              </div>
            </div>

            {/* Inventory Bodies */}
            <div className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64">
              <h3 className="text-blue-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M18.92 6.01C18.72 5.42 18.16 5 17.5 5H6.5C5.84 5 5.28 5.42 5.08 6.01L3 12V20C3 20.55 3.45 21 4 21H5C5.55 21 6 20.55 6 20V19H18V20C18 20.55 18.45 21 19 21H20C20.55 21 21 20.55 21 20V12L18.92 6.01M6.5 16C5.67 16 5 15.33 5 14.5S5.67 13 6.5 13 8 13.67 8 14.5 7.33 16 6.5 16M17.5 16C16.67 16 16 15.33 16 14.5S16.67 13 17.5 13 19 13.67 19 14.5 18.33 16 17.5 16M5.81 10L6.5 7H17.5L18.19 10H5.81Z"/>
                </svg>
                Inventory Bodies
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableBodies().map((body) => (
                  <div key={body.uuid} className="border border-gray-600 rounded p-3 bg-gray-700 h-16 flex flex-col justify-center hover:bg-gray-600 transition-colors cursor-pointer">
                    <div className="font-medium text-sm text-white">{body.name}</div>
                    <div className="text-xs text-gray-300">{body.rarity}</div>
                    <div className="text-xs text-gray-400">
                      S:{body.straight_value} C:{body.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailableBodies().length === 0 && (
                  <div className="text-gray-500 text-sm text-center h-16 flex items-center justify-center">All bodies assigned</div>
                )}
              </div>
            </div>

            {/* Inventory Engines */}
            <div className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64">
              <h3 className="text-orange-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 2L13.09 8.26L22 9L13.09 9.74L12 16L10.91 9.74L2 9L10.91 8.26L12 2M12 4.5L11.5 7.5L8.5 8L11.5 8.5L12 11.5L12.5 8.5L15.5 8L12.5 7.5L12 4.5M7 18C7 16.9 6.1 16 5 16S3 16.9 3 18 3.9 20 5 20 7 19.1 7 18M21 18C21 16.9 20.1 16 19 16S17 16.9 17 18 17.9 20 19 20 21 19.1 21 18Z"/>
                </svg>
                Inventory Engines
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableEngines().map((engine) => (
                  <div key={engine.uuid} className="border border-gray-600 rounded p-3 bg-gray-700 h-16 flex flex-col justify-center hover:bg-gray-600 transition-colors cursor-pointer">
                    <div className="font-medium text-sm text-white">{engine.name}</div>
                    <div className="text-xs text-gray-300">{engine.rarity}</div>
                    <div className="text-xs text-gray-400">
                      S:{engine.straight_value} C:{engine.curve_value}
                    </div>
                  </div>
                ))}
                {getAvailableEngines().length === 0 && (
                  <div className="text-gray-500 text-sm text-center h-16 flex items-center justify-center">All engines assigned</div>
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