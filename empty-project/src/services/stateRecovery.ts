/**
 * State Recovery Utilities
 * Handles state inconsistency detection and recovery
 * Requirements: 9.4 - Implement state inconsistency recovery
 */

import type { TurnPhase, LocalView, BoostAvailability } from '../types/race-api';
import { raceAPIService } from './raceAPI';

/**
 * State inconsistency types
 */
export type StateInconsistencyType =
  | 'turn_phase_mismatch'
  | 'boost_availability_mismatch'
  | 'player_position_mismatch'
  | 'lap_number_mismatch'
  | 'missing_data';

/**
 * State inconsistency details
 */
export interface StateInconsistency {
  type: StateInconsistencyType;
  description: string;
  expectedValue?: any;
  actualValue?: any;
  severity: 'low' | 'medium' | 'high';
}

/**
 * Recovery action types
 */
export type RecoveryAction =
  | 'refresh_all'
  | 'refresh_turn_phase'
  | 'refresh_boost_availability'
  | 'refresh_local_view'
  | 'reset_ui_state'
  | 'navigate_away';

/**
 * Recovery strategy
 */
export interface RecoveryStrategy {
  action: RecoveryAction;
  description: string;
  automatic: boolean; // Whether to execute automatically or ask user
}

/**
 * State validation result
 */
export interface StateValidationResult {
  isValid: boolean;
  inconsistencies: StateInconsistency[];
  recoveryStrategy?: RecoveryStrategy;
}

/**
 * Validate turn phase consistency
 */
export function validateTurnPhase(
  currentPhase: TurnPhase | null,
  expectedPhase?: string,
  hasSubmittedAction?: boolean,
): StateInconsistency[] {
  const inconsistencies: StateInconsistency[] = [];

  if (!currentPhase) {
    inconsistencies.push({
      type: 'missing_data',
      description: 'Turn phase data is missing',
      severity: 'high',
    });
    return inconsistencies;
  }

  // Check if player has submitted but phase is still WaitingForPlayers
  if (hasSubmittedAction && currentPhase.turn_phase === 'WaitingForPlayers') {
    // This might be normal if other players haven't submitted yet
    // Only flag as inconsistency if it's been too long
    inconsistencies.push({
      type: 'turn_phase_mismatch',
      description: 'Action submitted but turn phase still waiting for players',
      expectedValue: 'AllSubmitted or Processing',
      actualValue: currentPhase.turn_phase,
      severity: 'low',
    });
  }

  // Check for expected phase mismatch
  if (expectedPhase && currentPhase.turn_phase !== expectedPhase) {
    inconsistencies.push({
      type: 'turn_phase_mismatch',
      description: `Turn phase does not match expected value`,
      expectedValue: expectedPhase,
      actualValue: currentPhase.turn_phase,
      severity: 'medium',
    });
  }

  return inconsistencies;
}

/**
 * Validate boost availability consistency
 */
export function validateBoostAvailability(
  boostAvailability: BoostAvailability | null,
  selectedBoost?: number | null,
  hasSubmittedAction?: boolean,
): StateInconsistency[] {
  const inconsistencies: StateInconsistency[] = [];

  if (!boostAvailability) {
    inconsistencies.push({
      type: 'missing_data',
      description: 'Boost availability data is missing',
      severity: 'medium',
    });
    return inconsistencies;
  }

  // Check if selected boost is still available after submission
  if (hasSubmittedAction && selectedBoost !== null && selectedBoost !== undefined) {
    const isStillAvailable = boostAvailability.available_cards.includes(selectedBoost);
    if (isStillAvailable) {
      inconsistencies.push({
        type: 'boost_availability_mismatch',
        description: 'Submitted boost card is still marked as available',
        expectedValue: 'boost should be unavailable',
        actualValue: 'boost is still available',
        severity: 'medium',
      });
    }
  }

  // Validate boost hand state consistency
  const totalCards = Object.keys(boostAvailability.hand_state).length;
  const availableCards = boostAvailability.available_cards.length;
  const usedCards = totalCards - availableCards;

  if (usedCards < 0 || usedCards > 5) {
    inconsistencies.push({
      type: 'boost_availability_mismatch',
      description: 'Invalid boost hand state',
      expectedValue: '0-5 used cards',
      actualValue: `${usedCards} used cards`,
      severity: 'high',
    });
  }

  return inconsistencies;
}

/**
 * Validate local view consistency
 */
export function validateLocalView(
  localView: LocalView | null,
  playerUuid: string,
  expectedLap?: number,
): StateInconsistency[] {
  const inconsistencies: StateInconsistency[] = [];

  if (!localView) {
    inconsistencies.push({
      type: 'missing_data',
      description: 'Local view data is missing',
      severity: 'high',
    });
    return inconsistencies;
  }

  // Find player in visible participants
  const playerParticipant = localView.visible_participants.find(
    (p) => p.player_uuid === playerUuid,
  );

  if (!playerParticipant) {
    inconsistencies.push({
      type: 'player_position_mismatch',
      description: 'Player not found in local view',
      severity: 'high',
    });
    return inconsistencies;
  }

  // Check lap number consistency
  if (expectedLap && playerParticipant.current_lap !== expectedLap) {
    inconsistencies.push({
      type: 'lap_number_mismatch',
      description: 'Player lap number does not match expected value',
      expectedValue: expectedLap,
      actualValue: playerParticipant.current_lap,
      severity: 'medium',
    });
  }

  // Validate sector consistency
  const playerSector = playerParticipant.current_sector;
  const centerSector = localView.center_sector;
  const sectorDifference = Math.abs(playerSector - centerSector);

  if (sectorDifference > 2) {
    inconsistencies.push({
      type: 'player_position_mismatch',
      description: 'Player sector is outside expected local view range',
      expectedValue: `within 2 sectors of ${centerSector}`,
      actualValue: `sector ${playerSector}`,
      severity: 'medium',
    });
  }

  return inconsistencies;
}

/**
 * Determine recovery strategy based on inconsistencies
 */
export function determineRecoveryStrategy(
  inconsistencies: StateInconsistency[],
): RecoveryStrategy | null {
  if (inconsistencies.length === 0) {
    return null;
  }

  const highSeverityCount = inconsistencies.filter((i) => i.severity === 'high').length;
  const mediumSeverityCount = inconsistencies.filter((i) => i.severity === 'medium').length;

  // High severity issues require full refresh
  if (highSeverityCount > 0) {
    return {
      action: 'refresh_all',
      description: 'Critical state inconsistency detected. Refreshing all race data.',
      automatic: true,
    };
  }

  // Multiple medium severity issues
  if (mediumSeverityCount > 2) {
    return {
      action: 'refresh_all',
      description: 'Multiple state inconsistencies detected. Refreshing all race data.',
      automatic: true,
    };
  }

  // Single medium severity issue - targeted refresh
  if (mediumSeverityCount === 1) {
    const inconsistency = inconsistencies.find((i) => i.severity === 'medium')!;

    switch (inconsistency.type) {
      case 'turn_phase_mismatch':
        return {
          action: 'refresh_turn_phase',
          description: 'Turn phase inconsistency detected. Refreshing turn state.',
          automatic: true,
        };

      case 'boost_availability_mismatch':
        return {
          action: 'refresh_boost_availability',
          description: 'Boost availability inconsistency detected. Refreshing boost state.',
          automatic: true,
        };

      case 'player_position_mismatch':
      case 'lap_number_mismatch':
        return {
          action: 'refresh_local_view',
          description: 'Position inconsistency detected. Refreshing race position.',
          automatic: true,
        };

      default:
        return {
          action: 'refresh_all',
          description: 'State inconsistency detected. Refreshing race data.',
          automatic: true,
        };
    }
  }

  // Low severity issues - reset UI state
  return {
    action: 'reset_ui_state',
    description: 'Minor state inconsistency detected. Resetting UI state.',
    automatic: true,
  };
}

/**
 * Execute recovery action
 */
export async function executeRecovery(
  action: RecoveryAction,
  raceUuid: string,
  playerUuid: string,
): Promise<{
  turnPhase?: TurnPhase;
  localView?: LocalView;
  boostAvailability?: BoostAvailability;
}> {
  const result: any = {};

  switch (action) {
    case 'refresh_all':
      const [turnPhase, localView, boostAvailability] = await Promise.all([
        raceAPIService.getTurnPhase(raceUuid),
        raceAPIService.getLocalView(raceUuid, playerUuid),
        raceAPIService.getBoostAvailability(raceUuid, playerUuid),
      ]);

      result.turnPhase = turnPhase;
      result.localView = localView;
      result.boostAvailability = boostAvailability;
      break;

    case 'refresh_turn_phase':
      result.turnPhase = await raceAPIService.getTurnPhase(raceUuid);
      break;

    case 'refresh_boost_availability':
      result.boostAvailability = await raceAPIService.getBoostAvailability(raceUuid, playerUuid);
      break;

    case 'refresh_local_view':
      result.localView = await raceAPIService.getLocalView(raceUuid, playerUuid);
      break;

    case 'reset_ui_state':
      // This will be handled by the calling component
      break;

    case 'navigate_away':
      // This will be handled by the calling component
      break;
  }

  return result;
}

/**
 * Comprehensive state validation
 */
export function validateRaceState(
  turnPhase: TurnPhase | null,
  localView: LocalView | null,
  boostAvailability: BoostAvailability | null,
  playerUuid: string,
  uiState: {
    selectedBoost?: number | null;
    hasSubmittedAction?: boolean;
    expectedLap?: number;
    expectedPhase?: string;
  } = {},
): StateValidationResult {
  const allInconsistencies: StateInconsistency[] = [
    ...validateTurnPhase(turnPhase, uiState.expectedPhase, uiState.hasSubmittedAction),
    ...validateBoostAvailability(
      boostAvailability,
      uiState.selectedBoost,
      uiState.hasSubmittedAction,
    ),
    ...validateLocalView(localView, playerUuid, uiState.expectedLap),
  ];

  const isValid = allInconsistencies.length === 0;
  const recoveryStrategy = determineRecoveryStrategy(allInconsistencies);

  return {
    isValid,
    inconsistencies: allInconsistencies,
    recoveryStrategy: recoveryStrategy || undefined,
  };
}

/**
 * Auto-recovery hook for components
 */
export async function performAutoRecovery(
  validationResult: StateValidationResult,
  raceUuid: string,
  playerUuid: string,
  onRecoveryComplete?: (recoveredData: any) => void,
  onRecoveryError?: (error: Error) => void,
): Promise<boolean> {
  if (validationResult.isValid || !validationResult.recoveryStrategy) {
    return false;
  }

  const { recoveryStrategy } = validationResult;

  if (!recoveryStrategy.automatic) {
    return false;
  }

  try {
    console.log(`[StateRecovery] Executing recovery: ${recoveryStrategy.description}`);

    const recoveredData = await executeRecovery(recoveryStrategy.action, raceUuid, playerUuid);

    if (onRecoveryComplete) {
      onRecoveryComplete(recoveredData);
    }

    console.log('[StateRecovery] Recovery completed successfully');
    return true;
  } catch (error) {
    console.error('[StateRecovery] Recovery failed:', error);

    if (onRecoveryError) {
      onRecoveryError(error instanceof Error ? error : new Error(String(error)));
    }

    return false;
  }
}
