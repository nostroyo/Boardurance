// Player asset models for cars, pilots, engines, and bodies

export interface Car {
  uuid: string;
  name: string;
  nft_mint_address?: string;
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
  nft_mint_address?: string;
}

export interface Engine {
  uuid: string;
  name: string;
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  straight_value: number;
  curve_value: number;
  nft_mint_address?: string;
}

export interface Body {
  uuid: string;
  name: string;
  rarity: 'Common' | 'Uncommon' | 'Rare' | 'Epic' | 'Legendary';
  straight_value: number;
  curve_value: number;
  nft_mint_address?: string;
}

// Performance calculation models
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
  lapCharacteristic: string;
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