// Performance calculation engine for player game interface
// Implements base value calculation, sector ceiling application, and boost integration

import type { Pilot, Engine, Body, PerformanceBreakdown } from '../types/player-assets';
import type { Sector } from '../types/race';

/**
 * Calculate complete performance breakdown for a player's car
 *
 * @param pilot - Player's pilot
 * @param engine - Car's engine
 * @param body - Car's body
 * @param sector - Current sector
 * @param lapCharacteristic - Current lap characteristic (Straight or Curve)
 * @param boost - Selected boost value (0-5)
 * @returns Complete performance breakdown with all calculations
 */
export const calculatePerformance = (
  pilot: Pilot,
  engine: Engine,
  body: Body,
  sector: Sector,
  lapCharacteristic: 'Straight' | 'Curve',
  boost: number,
): PerformanceBreakdown => {
  // Validate boost value
  const validBoost = Math.max(0, Math.min(5, boost));

  // Get stat values based on lap characteristic
  const engineValue = lapCharacteristic === 'Straight' ? engine.straight_value : engine.curve_value;

  const bodyValue = lapCharacteristic === 'Straight' ? body.straight_value : body.curve_value;

  const pilotValue =
    lapCharacteristic === 'Straight'
      ? pilot.performance.straight_value
      : pilot.performance.curve_value;

  // Calculate base value (sum of all components)
  const baseValue = engineValue + bodyValue + pilotValue;

  // Apply sector ceiling to base value
  const sectorCappedValue = Math.min(baseValue, sector.max_value);

  // Calculate final value (capped base + boost)
  const finalValue = sectorCappedValue + validBoost;

  return {
    engineContribution: engineValue,
    bodyContribution: bodyValue,
    pilotContribution: pilotValue,
    baseValue,
    sectorCappedValue,
    boostValue: validBoost,
    finalValue,
  };
};

/**
 * Calculate base value only (without boost)
 *
 * @param pilot - Player's pilot
 * @param engine - Car's engine
 * @param body - Car's body
 * @param lapCharacteristic - Current lap characteristic
 * @returns Base value before sector ceiling and boost
 */
export const calculateBaseValue = (
  pilot: Pilot,
  engine: Engine,
  body: Body,
  lapCharacteristic: 'Straight' | 'Curve',
): number => {
  const engineValue = lapCharacteristic === 'Straight' ? engine.straight_value : engine.curve_value;

  const bodyValue = lapCharacteristic === 'Straight' ? body.straight_value : body.curve_value;

  const pilotValue =
    lapCharacteristic === 'Straight'
      ? pilot.performance.straight_value
      : pilot.performance.curve_value;

  return engineValue + bodyValue + pilotValue;
};

/**
 * Apply sector ceiling to a base value
 *
 * @param baseValue - Calculated base value
 * @param sector - Current sector
 * @returns Capped value based on sector maximum
 */
export const applySectorCeiling = (baseValue: number, sector: Sector): number => {
  return Math.min(baseValue, sector.max_value);
};

/**
 * Calculate final value with boost
 *
 * @param cappedBaseValue - Base value after sector ceiling
 * @param boost - Boost value (0-5)
 * @returns Final performance value
 */
export const calculateFinalValue = (cappedBaseValue: number, boost: number): number => {
  const validBoost = Math.max(0, Math.min(5, boost));
  return cappedBaseValue + validBoost;
};

/**
 * Get performance value for a specific characteristic
 *
 * @param engine - Car's engine
 * @param body - Car's body
 * @param pilot - Player's pilot
 * @param characteristic - Straight or Curve
 * @returns Performance value for the characteristic
 */
export const getCharacteristicValue = (
  engine: Engine,
  body: Body,
  pilot: Pilot,
  characteristic: 'Straight' | 'Curve',
): number => {
  const engineValue = characteristic === 'Straight' ? engine.straight_value : engine.curve_value;

  const bodyValue = characteristic === 'Straight' ? body.straight_value : body.curve_value;

  const pilotValue =
    characteristic === 'Straight'
      ? pilot.performance.straight_value
      : pilot.performance.curve_value;

  return engineValue + bodyValue + pilotValue;
};

/**
 * Validate boost value is within acceptable range
 *
 * @param boost - Boost value to validate
 * @returns True if boost is valid (0-5)
 */
export const isValidBoost = (boost: number): boolean => {
  return Number.isInteger(boost) && boost >= 0 && boost <= 5;
};

/**
 * Get boost validation error message
 *
 * @param boost - Boost value to validate
 * @returns Error message or null if valid
 */
export const getBoostValidationError = (boost: number): string | null => {
  if (!Number.isFinite(boost)) {
    return 'Boost value must be a number';
  }

  if (!Number.isInteger(boost)) {
    return 'Boost value must be a whole number';
  }

  if (boost < 0) {
    return 'Boost value cannot be negative';
  }

  if (boost > 5) {
    return 'Boost value cannot exceed 5';
  }

  return null;
};
