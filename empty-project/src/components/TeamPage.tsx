import { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuthContext } from '../contexts/AuthContext';
import { apiUtils } from '../utils/auth';

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
  pilot_uuid?: string; // Single pilot assignment (backend structure)
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
  const [error, setError] = useState('');
  const [isSaving, setIsSaving] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [draggedItem, setDraggedItem] = useState<{type: 'engine' | 'body' | 'pilot', uuid: string} | null>(null);
  const navigate = useNavigate();
  const { user, isAuthenticated, logout, updateUser } = useAuthContext();

  useEffect(() => {
    // Redirect to login if not authenticated
    if (!isAuthenticated) {
      navigate('/login');
      return;
    }

    const fetchPlayerData = async () => {
      try {
        if (!user) {
          setError('No user data available');
          setLoading(false);
          return;
        }

        // Fetch complete player data from backend using UUID
        const result = await apiUtils.getPlayer(user.uuid);
        
        if (result.success && result.data) {
          setPlayer(result.data);
        } else {
          setError(result.error || 'Failed to load player data');
        }
      } catch (err) {
        console.error('Error loading player data:', err);
        setError('Failed to load player data');
      } finally {
        setLoading(false);
      }
    };

    fetchPlayerData();
  }, [isAuthenticated, user, navigate]);

  const getAssignedPilot = (pilotUuid?: string) => {
    return pilotUuid ? player?.pilots.find(p => p.uuid === pilotUuid) : undefined;
  };

  const getAssignedEngine = (engineUuid?: string) => {
    return player?.engines.find(e => e.uuid === engineUuid);
  };

  const getAssignedBody = (bodyUuid?: string) => {
    return player?.bodies.find(b => b.uuid === bodyUuid);
  };

  const getAvailablePilots = () => {
    const assignedPilotIds = player?.cars.map(car => car.pilot_uuid).filter(Boolean) || [];
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

  // Component assignment functions
  const assignEngineTocar = (carUuid: string, engineUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      // Remove engine from other cars first
      updatedPlayer.cars.forEach(car => {
        if (car.engine_uuid === engineUuid) {
          car.engine_uuid = undefined;
        }
      });
      // Assign to target car
      updatedPlayer.cars[carIndex].engine_uuid = engineUuid;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  const assignBodyToCar = (carUuid: string, bodyUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      // Remove body from other cars first
      updatedPlayer.cars.forEach(car => {
        if (car.body_uuid === bodyUuid) {
          car.body_uuid = undefined;
        }
      });
      // Assign to target car
      updatedPlayer.cars[carIndex].body_uuid = bodyUuid;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  const assignPilotToCar = (carUuid: string, pilotUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      // Remove pilot from other cars first
      updatedPlayer.cars.forEach(car => {
        if (car.pilot_uuid === pilotUuid) {
          car.pilot_uuid = undefined;
        }
      });
      // Assign to target car
      updatedPlayer.cars[carIndex].pilot_uuid = pilotUuid;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  const removeEngineFromCar = (carUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      updatedPlayer.cars[carIndex].engine_uuid = undefined;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  const removeBodyFromCar = (carUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      updatedPlayer.cars[carIndex].body_uuid = undefined;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  const removePilotFromCar = (carUuid: string) => {
    if (!player) return;
    
    const updatedPlayer = { ...player };
    const carIndex = updatedPlayer.cars.findIndex(car => car.uuid === carUuid);
    if (carIndex !== -1) {
      updatedPlayer.cars[carIndex].pilot_uuid = undefined;
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
  };

  // Drag and drop handlers
  const handleDragStart = (e: React.DragEvent, type: 'engine' | 'body' | 'pilot', uuid: string) => {
    setDraggedItem({ type, uuid });
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDropOnCar = (e: React.DragEvent, carUuid: string, slotType: 'engine' | 'body' | 'pilot') => {
    e.preventDefault();
    
    if (!draggedItem || draggedItem.type !== slotType) return;

    switch (slotType) {
      case 'engine':
        assignEngineTocar(carUuid, draggedItem.uuid);
        break;
      case 'body':
        assignBodyToCar(carUuid, draggedItem.uuid);
        break;
      case 'pilot':
        assignPilotToCar(carUuid, draggedItem.uuid);
        break;
    }
    
    setDraggedItem(null);
  };

  const handleDropOnInventory = (e: React.DragEvent) => {
    e.preventDefault();
    
    if (!draggedItem) return;

    // Find which car has this component and remove it
    if (player) {
      const updatedPlayer = { ...player };
      updatedPlayer.cars.forEach(car => {
        switch (draggedItem.type) {
          case 'engine':
            if (car.engine_uuid === draggedItem.uuid) {
              car.engine_uuid = undefined;
            }
            break;
          case 'body':
            if (car.body_uuid === draggedItem.uuid) {
              car.body_uuid = undefined;
            }
            break;
          case 'pilot':
            if (car.pilot_uuid === draggedItem.uuid) {
              car.pilot_uuid = undefined;
            }
            break;
        }
      });
      setPlayer(updatedPlayer);
      setHasChanges(true);
    }
    
    setDraggedItem(null);
  };

  // Save configuration to backend
  const saveConfiguration = async () => {
    if (!player || !hasChanges) return;
    
    setIsSaving(true);
    try {
      const result = await apiUtils.updatePlayerTeamName(player.uuid, player.team_name);
      
      if (result.success && result.data) {
        const updatedPlayer = result.data.player;
        
        // Update stored user data
        updateUser({ team_name: updatedPlayer.team_name });
        
        setPlayer(updatedPlayer);
        setHasChanges(false);
        console.log('Configuration saved successfully');
      } else {
        setError(result.error || 'Failed to save configuration');
      }
    } catch (err) {
      console.error('Error saving configuration:', err);
      setError('Error saving configuration');
    } finally {
      setIsSaving(false);
    }
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
          <div className="flex space-x-4">
            {hasChanges && (
              <button
                onClick={saveConfiguration}
                disabled={isSaving}
                className="bg-green-600 hover:bg-green-700 disabled:bg-green-400 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-semibold transition duration-200 shadow-lg flex items-center"
              >
                {isSaving ? (
                  <>
                    <svg className="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Saving...
                  </>
                ) : (
                  <>
                    <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    Save Configuration
                  </>
                )}
              </button>
            )}
            <Link
              to="/dashboard"
              className="bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-lg font-semibold transition duration-200 shadow-lg"
            >
              ← Dashboard
            </Link>
            <button
              onClick={async () => {
                await logout();
                navigate('/login');
              }}
              className="bg-red-600 hover:bg-red-700 text-white px-6 py-3 rounded-lg font-semibold transition duration-200 shadow-lg"
            >
              Logout
            </button>
          </div>
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
                    <div 
                      className="border-2 border-gray-600 rounded-lg p-4 bg-gray-700 h-28 flex flex-col shadow-lg transition-all duration-200"
                      onDragOver={handleDragOver}
                      onDrop={(e) => handleDropOnCar(e, car.uuid, 'engine')}
                    >
                      <h3 className="text-orange-400 font-semibold mb-2 text-sm flex items-center">
                        <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                          <rect x="4" y="8" width="16" height="10" rx="2"/>
                          <path d="M8 6v2M16 6v2"/>
                          <path d="M6 12h2M16 12h2"/>
                          <circle cx="8" cy="13" r="1"/>
                          <circle cx="12" cy="13" r="1"/>
                          <circle cx="16" cy="13" r="1"/>
                        </svg>
                        Engine
                      </h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedEngine(car.engine_uuid) ? (
                          <div 
                            className="text-sm text-center cursor-pointer hover:bg-gray-600 p-2 rounded transition-colors"
                            draggable
                            onDragStart={(e) => handleDragStart(e, 'engine', car.engine_uuid!)}
                            onClick={() => removeEngineFromCar(car.uuid)}
                            title="Drag to move or click to remove engine"
                          >
                            <div className="font-medium text-white mb-1">{getAssignedEngine(car.engine_uuid)?.name}</div>
                            <div className="text-gray-300 text-xs">
                              {getAssignedEngine(car.engine_uuid)?.rarity} | 
                              S:{getAssignedEngine(car.engine_uuid)?.straight_value} C:{getAssignedEngine(car.engine_uuid)?.curve_value}
                            </div>
                            <div className="text-orange-400 text-xs mt-1">Drag to move • Click to remove</div>
                          </div>
                        ) : (
                          <div className="text-gray-500 text-sm text-center border-2 border-dashed border-gray-500 rounded p-2">
                            Drop engine here
                          </div>
                        )}
                      </div>
                    </div>

                    {/* Body Section */}
                    <div 
                      className="border-2 border-gray-600 rounded-lg p-4 bg-gray-700 h-28 flex flex-col shadow-lg transition-all duration-200"
                      onDragOver={handleDragOver}
                      onDrop={(e) => handleDropOnCar(e, car.uuid, 'body')}
                    >
                      <h3 className="text-blue-400 font-semibold mb-2 text-sm flex items-center">
                        <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                          <path d="M7 17h10l2-6H5l2 6z"/>
                          <circle cx="7" cy="17" r="2"/>
                          <circle cx="17" cy="17" r="2"/>
                          <path d="M5 11V8a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v3"/>
                        </svg>
                        Body
                      </h3>
                      <div className="flex-1 flex flex-col justify-center">
                        {getAssignedBody(car.body_uuid) ? (
                          <div 
                            className="text-sm text-center cursor-pointer hover:bg-gray-600 p-2 rounded transition-colors"
                            draggable
                            onDragStart={(e) => handleDragStart(e, 'body', car.body_uuid!)}
                            onClick={() => removeBodyFromCar(car.uuid)}
                            title="Drag to move or click to remove body"
                          >
                            <div className="font-medium text-white mb-1">{getAssignedBody(car.body_uuid)?.name}</div>
                            <div className="text-gray-300 text-xs">
                              {getAssignedBody(car.body_uuid)?.rarity} | 
                              S:{getAssignedBody(car.body_uuid)?.straight_value} C:{getAssignedBody(car.body_uuid)?.curve_value}
                            </div>
                            <div className="text-blue-400 text-xs mt-1">Drag to move • Click to remove</div>
                          </div>
                        ) : (
                          <div className="text-gray-500 text-sm text-center border-2 border-dashed border-gray-500 rounded p-2">
                            Drop body here
                          </div>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Right Side - Pilots Section (3 slots) */}
                  <div className="border-2 border-gray-600 rounded-lg p-3 bg-gray-700 h-60 flex flex-col shadow-lg">
                    <h3 className="text-green-400 font-semibold mb-2 text-sm flex items-center">
                      <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                        <path d="M9 10h6"/>
                      </svg>
                      Pilots (3 Required)
                    </h3>
                    <div className="flex-1 space-y-2">
                      {[0, 1, 2].map((slotIndex) => {
                        const availablePilots = getAvailablePilots();
                        const assignedPilot = slotIndex === 0 ? getAssignedPilot(car.pilot_uuid) : null;
                        const pilot = assignedPilot || availablePilots[slotIndex - (assignedPilot ? 1 : 0)];
                        
                        return (
                          <div 
                            key={slotIndex}
                            className={`flex items-center space-x-2 h-12 border rounded px-2 transition-all duration-200 ${
                              pilot 
                                ? 'border-green-500 bg-gray-600 shadow-md' 
                                : 'border-gray-500 bg-gray-800 border-dashed'
                            }`}
                            onDragOver={handleDragOver}
                            onDrop={(e) => handleDropOnCar(e, car.uuid, 'pilot')}
                          >
                            {/* Pilot Number */}
                            <div className="w-6 h-6 flex-shrink-0 bg-green-600 rounded-full flex items-center justify-center text-white font-bold text-xs">
                              {slotIndex + 1}
                            </div>
                            
                            {/* Pilot Content */}
                            <div className="flex-1 min-w-0">
                              {pilot ? (
                                <div 
                                  className="cursor-move hover:bg-gray-500 p-1 rounded transition-colors"
                                  draggable
                                  onDragStart={(e) => handleDragStart(e, 'pilot', pilot.uuid)}
                                  onClick={() => slotIndex === 0 && assignedPilot ? removePilotFromCar(car.uuid) : null}
                                  title={slotIndex === 0 && assignedPilot ? "Drag to move or click to remove pilot" : "Drag to assign pilot"}
                                >
                                  <div className="font-medium text-xs text-white truncate">{pilot.name}</div>
                                  <div className="text-xs text-gray-300">{pilot.pilot_class}</div>
                                </div>
                              ) : (
                                <div className="text-gray-500 text-xs">
                                  <div>Empty Slot</div>
                                  <div className="text-xs">Drop pilot here</div>
                                </div>
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
            <div 
              className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64"
              onDragOver={handleDragOver}
              onDrop={handleDropOnInventory}
            >
              <h3 className="text-green-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path d="M12 3C8.5 3 6 5.5 6 9v3c0 1.5.5 3 1.5 4H7c-.5 0-1 .5-1 1v2c0 .5.5 1 1 1h10c.5 0 1-.5 1-1v-2c0-.5-.5-1-1-1h-.5c1-.5 1.5-2.5 1.5-4V9c0-3.5-2.5-6-6-6z"/>
                  <path d="M9 10h6"/>
                </svg>
                INVENTORY PILOTS
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailablePilots().map((pilot) => (
                  <div 
                    key={pilot.uuid} 
                    draggable
                    onDragStart={(e) => handleDragStart(e, 'pilot', pilot.uuid)}
                    className="border border-gray-600 rounded px-2 py-1 bg-gray-700 h-8 flex items-center hover:bg-gray-600 transition-colors cursor-move group"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-xs text-white truncate">{pilot.name}</div>
                    </div>
                    <div className="text-xs text-gray-400 ml-1">S:{pilot.performance.straight_value}</div>
                  </div>
                ))}
                {getAvailablePilots().length === 0 && (
                  <div className="text-gray-500 text-xs text-center h-8 flex items-center justify-center border-2 border-dashed border-gray-500 rounded">
                    No pilots available • Drop here to unassign
                  </div>
                )}
              </div>
            </div>

            {/* Inventory Bodies */}
            <div 
              className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64"
              onDragOver={handleDragOver}
              onDrop={handleDropOnInventory}
            >
              <h3 className="text-blue-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path d="M7 17h10l2-6H5l2 6z"/>
                  <circle cx="7" cy="17" r="2"/>
                  <circle cx="17" cy="17" r="2"/>
                  <path d="M5 11V8a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v3"/>
                </svg>
                INVENTORY BODIES
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableBodies().map((body) => (
                  <div 
                    key={body.uuid} 
                    draggable
                    onDragStart={(e) => handleDragStart(e, 'body', body.uuid)}
                    className="border border-gray-600 rounded px-2 py-1 bg-gray-700 h-8 flex items-center hover:bg-gray-600 transition-colors cursor-move group"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-xs text-white truncate">{body.name}</div>
                    </div>
                    <div className="text-xs text-gray-400 ml-1">S:{body.straight_value}</div>
                  </div>
                ))}
                {getAvailableBodies().length === 0 && (
                  <div className="text-gray-500 text-xs text-center h-8 flex items-center justify-center border-2 border-dashed border-gray-500 rounded">
                    All bodies assigned • Drop here to unassign
                  </div>
                )}
              </div>
            </div>

            {/* Inventory Engines */}
            <div 
              className="bg-gray-800 rounded-lg shadow-2xl p-4 border-2 border-gray-600 h-64"
              onDragOver={handleDragOver}
              onDrop={handleDropOnInventory}
            >
              <h3 className="text-orange-400 font-bold text-lg mb-4 flex items-center">
                <svg className="w-5 h-5 mr-2" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <rect x="4" y="8" width="16" height="10" rx="2"/>
                  <path d="M8 6v2M16 6v2"/>
                  <path d="M6 12h2M16 12h2"/>
                  <circle cx="8" cy="13" r="1"/>
                  <circle cx="12" cy="13" r="1"/>
                  <circle cx="16" cy="13" r="1"/>
                </svg>
                INVENTORY ENGINES
              </h3>
              <div className="space-y-2 h-48 overflow-y-auto">
                {getAvailableEngines().map((engine) => (
                  <div 
                    key={engine.uuid} 
                    draggable
                    onDragStart={(e) => handleDragStart(e, 'engine', engine.uuid)}
                    className="border border-gray-600 rounded px-2 py-1 bg-gray-700 h-8 flex items-center hover:bg-gray-600 transition-colors cursor-move group"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-xs text-white truncate">{engine.name}</div>
                    </div>
                    <div className="text-xs text-gray-400 ml-1">S:{engine.straight_value}</div>
                  </div>
                ))}
                {getAvailableEngines().length === 0 && (
                  <div className="text-gray-500 text-xs text-center h-8 flex items-center justify-center border-2 border-dashed border-gray-500 rounded">
                    All engines assigned • Drop here to unassign
                  </div>
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