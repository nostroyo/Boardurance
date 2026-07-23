// UI state management types for the player game interface

import type { Car, Pilot, Engine, Body } from './player-assets';

// Component prop interfaces
// PlayerCarCardProps moved to component file to use backend API types

export interface PerformanceCalculatorProps {
  car: Car;
  pilot: Pilot;
  engine: Engine;
  body: Body;
  currentSector: import('./race').Sector;
  lapCharacteristic: string;
  selectedBoost: number;
  onBoostChange: (boost: number) => void;
}

// Error handling types
export interface ErrorRecoveryStrategy {
  errorType: string;
  retryAttempts: number;
  retryDelay: number;
  fallbackAction: () => void;
  userMessage: string;
}

// API response types
export interface APIResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

// Action types for boost submission
export interface BoostAction {
  player_uuid: string;
  boost_value: number;
}

export interface TurnActionRequest {
  actions: BoostAction[];
}
