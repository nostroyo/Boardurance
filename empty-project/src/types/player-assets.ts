// Player asset models for cars, pilots, engines, and bodies

export interface Car {
  uuid: string;
  name: string;
  pilot_uuid?: string;
  engine_uuid?: string;
  body_uuid?: string;
}

export interface Pilot {
  uuid: string;
  name: string;
  pilot_class: 'Rookie' | 'Veteran' | 'Elite' | 'Champion';
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  skills: {
    reaction_time: number;
    precision: number;
    focus: number;
    stamina: number;
  };
  performance: {
    straight_value: number;
    curve_value: number;
  };
}

export interface Engine {
  uuid: string;
  name: string;
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  straight_value: number;
  curve_value: number;
}

export interface Body {
  uuid: string;
  name: string;
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  straight_value: number;
  curve_value: number;
}

// Performance display models (data comes from backend)
export interface PerformanceBreakdown {
  engineContribution: number;
  bodyContribution: number;
  pilotContribution: number;
  baseValue: number;
  sectorCappedValue: number;
  boostValue: number;
  finalValue: number;
}

export interface LapPerformance {
  lapNumber: number;
  lapCharacteristic: 'Straight' | 'Curve';
  baseValue: number;
  boostUsed: number;
  finalValue: number;
  sectorMovement: 'Forward' | 'Backward' | 'Stay';
}

export interface RaceStatistics {
  totalLaps: number;
  averagePerformance: number;
  bestLap: LapPerformance | null;
  sectorsAdvanced: number;
  currentPosition: number;
}

// Validation functions for player asset data integrity
export const validateCar = (car: any): car is Car => {
  return (
    typeof car === 'object' &&
    car !== null &&
    typeof car.uuid === 'string' &&
    typeof car.name === 'string' &&
    (car.pilot_uuid === undefined || typeof car.pilot_uuid === 'string') &&
    (car.engine_uuid === undefined || typeof car.engine_uuid === 'string') &&
    (car.body_uuid === undefined || typeof car.body_uuid === 'string')
  );
};

export const validatePilot = (pilot: any): pilot is Pilot => {
  return (
    typeof pilot === 'object' &&
    pilot !== null &&
    typeof pilot.uuid === 'string' &&
    typeof pilot.name === 'string' &&
    ['Rookie', 'Veteran', 'Elite', 'Champion'].includes(pilot.pilot_class) &&
    ['Common', 'Uncommon', 'Rare', 'Epic', 'Legendary'].includes(pilot.rarity) &&
    validatePilotSkills(pilot.skills) &&
    validatePilotPerformance(pilot.performance)
  );
};

export const validatePilotSkills = (skills: any): boolean => {
  return (
    typeof skills === 'object' &&
    skills !== null &&
    typeof skills.reaction_time === 'number' &&
    skills.reaction_time >= 0 &&
    typeof skills.precision === 'number' &&
    skills.precision >= 0 &&
    typeof skills.focus === 'number' &&
    skills.focus >= 0 &&
    typeof skills.stamina === 'number' &&
    skills.stamina >= 0
  );
};

export const validatePilotPerformance = (performance: any): boolean => {
  return (
    typeof performance === 'object' &&
    performance !== null &&
    typeof performance.straight_value === 'number' &&
    performance.straight_value >= 0 &&
    typeof performance.curve_value === 'number' &&
    performance.curve_value >= 0
  );
};

export const validateEngine = (engine: any): engine is Engine => {
  return (
    typeof engine === 'object' &&
    engine !== null &&
    typeof engine.uuid === 'string' &&
    typeof engine.name === 'string' &&
    ['Common', 'Uncommon', 'Rare', 'Epic', 'Legendary'].includes(engine.rarity) &&
    typeof engine.straight_value === 'number' &&
    engine.straight_value >= 0 &&
    typeof engine.curve_value === 'number' &&
    engine.curve_value >= 0
  );
};

export const validateBody = (body: any): body is Body => {
  return (
    typeof body === 'object' &&
    body !== null &&
    typeof body.uuid === 'string' &&
    typeof body.name === 'string' &&
    ['Common', 'Uncommon', 'Rare', 'Epic', 'Legendary'].includes(body.rarity) &&
    typeof body.straight_value === 'number' &&
    body.straight_value >= 0 &&
    typeof body.curve_value === 'number' &&
    body.curve_value >= 0
  );
};

// Utility functions for display purposes (no calculations)
export const getRarityColor = (rarity: string): string => {
  const rarityColors: Record<string, string> = {
    Common: '#9CA3AF',
    Uncommon: '#10B981',
    Rare: '#3B82F6',
    Epic: '#8B5CF6',
    Legendary: '#F59E0B',
  };
  return rarityColors[rarity] || '#9CA3AF';
};

export const getPilotClassIcon = (pilotClass: string): string => {
  const classIcons: Record<string, string> = {
    Rookie: '🟢',
    Veteran: '🔵',
    Elite: '🟣',
    Champion: '🟡',
  };
  return classIcons[pilotClass] || '🟢';
};

export const formatPerformanceValue = (value: number): string => {
  return value.toFixed(1);
};
