// UI state management types for the player game interface

import type { Race, RaceParticipant, LocalRaceView, TurnPhase, AnimationState } from './race';
import type { Car, Pilot, Engine, Body } from './player-assets';

// Main player game state
export interface PlayerGameState {
  // Race Data
  race: Race | null;
  localView: LocalRaceView;

  // Player Context
  playerUuid: string;
  playerParticipant: RaceParticipant | null;

  // Turn Management
  currentTurnPhase: TurnPhase;
  selectedBoost: number;
  hasSubmittedAction: boolean;

  // UI State
  isLoading: boolean;
  error: string | null;
  animationState: AnimationState;
}

// Component prop interfaces
export interface PlayerGameInterfaceProps {
  raceUuid: string;
  playerUuid: string;
  onRaceComplete?: (finalPosition: number) => void;
  onError?: (error: string) => void;
}

export interface RaceStatusPanelProps {
  race: Race;
  currentTurnPhase: TurnPhase;
  timeRemaining?: number;
}

export interface LocalSectorDisplayProps {
  visibleSectors: import('./race').Sector[];
  visibleParticipants: RaceParticipant[];
  playerSector: number;
  animationState: AnimationState;
}

export interface SectorCardProps {
  sector: import('./race').Sector;
  participants: RaceParticipant[];
  isPlayerSector: boolean;
  position: 'above' | 'center' | 'below';
}

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

export interface SimultaneousTurnControllerProps {
  currentTurnPhase: TurnPhase;
  selectedBoost: number;
  hasSubmitted: boolean;
  onBoostSelect: (boost: number) => void;
  onSubmitAction: () => Promise<void>;
  timeRemaining?: number;
}

export interface LocalSectorMovementProps {
  movements: import('./race').ParticipantMovement[];
  onAnimationComplete: () => void;
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
