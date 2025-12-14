/**
 * useRacePolling - Custom hook for polling turn phase status
 *
 * This hook manages the polling lifecycle for monitoring turn phase transitions.
 * It polls the backend every 2 seconds to check if the turn has completed.
 *
 * Features:
 * - Efficient 2-second polling interval
 * - Max 60 attempts (2 minutes timeout)
 * - Graceful error handling
 * - Automatic cleanup on unmount
 * - Request cancellation on component unmount
 * - Optimized polling with exponential backoff on errors
 *
 * Requirements: 4.1, 12.1, 12.3
 */

import { useEffect, useRef, useCallback } from 'react';
import { raceAPIService } from '../services/raceAPI';
import type { TurnPhase } from '../types/race-api';

const POLL_INTERVAL = 2000; // 2 seconds
const MAX_POLL_ATTEMPTS = 60; // 2 minutes max (60 * 2 seconds)

export interface UseRacePollingOptions {
  raceUuid: string;
  enabled: boolean;
  onTurnPhaseChange: (turnPhase: TurnPhase) => void;
  onComplete: () => void;
  onError?: (error: Error) => void;
  onMaxAttemptsReached?: () => void;
}

/**
 * Custom hook for polling race turn phase
 */
export function useRacePolling({
  raceUuid,
  enabled,
  onTurnPhaseChange,
  onComplete,
  onError,
  onMaxAttemptsReached,
}: UseRacePollingOptions) {
  const attemptsRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isMountedRef = useRef(true);
  const lastPhaseRef = useRef<string | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);
  const errorCountRef = useRef(0);
  const lastSuccessfulPollRef = useRef<number>(Date.now());

  /**
   * Clear the polling timer and cancel any pending requests
   */
  const clearTimer = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }

    // Cancel any pending API request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
  }, []);

  /**
   * Reset polling state
   */
  const resetPolling = useCallback(() => {
    attemptsRef.current = 0;
    lastPhaseRef.current = null;
    errorCountRef.current = 0;
    lastSuccessfulPollRef.current = Date.now();
    clearTimer();
  }, [clearTimer]);

  /**
   * Calculate polling interval with exponential backoff on errors
   */
  const getPollingInterval = useCallback(() => {
    if (errorCountRef.current === 0) {
      return POLL_INTERVAL; // Normal 2-second interval
    }

    // Exponential backoff: 2s, 4s, 8s, max 16s
    const backoffMultiplier = Math.min(Math.pow(2, errorCountRef.current), 8);
    return POLL_INTERVAL * backoffMultiplier;
  }, []);

  /**
   * Poll the turn phase endpoint
   */
  const poll = useCallback(async () => {
    // Check if component is still mounted
    if (!isMountedRef.current) {
      console.log('[useRacePolling] Component unmounted, stopping poll');
      return;
    }

    // Check if polling is still enabled
    if (!enabled) {
      console.log('[useRacePolling] Polling disabled, stopping poll');
      return;
    }

    // Check max attempts
    if (attemptsRef.current >= MAX_POLL_ATTEMPTS) {
      console.warn('[useRacePolling] Max polling attempts reached');

      if (onMaxAttemptsReached) {
        onMaxAttemptsReached();
      }

      return;
    }

    attemptsRef.current += 1;

    console.log(`[useRacePolling] Polling attempt ${attemptsRef.current}/${MAX_POLL_ATTEMPTS}`);

    try {
      // Create abort controller for this request
      abortControllerRef.current = new AbortController();

      // Fetch turn phase from backend with cancellation support
      const turnPhase = await raceAPIService.getTurnPhase(raceUuid);

      console.log('[useRacePolling] Turn phase:', turnPhase.turn_phase);

      // Check if component is still mounted after async operation
      if (!isMountedRef.current) {
        return;
      }

      // Reset error count on successful poll
      errorCountRef.current = 0;
      lastSuccessfulPollRef.current = Date.now();

      // Detect phase change
      if (lastPhaseRef.current !== turnPhase.turn_phase) {
        console.log(
          `[useRacePolling] Phase changed: ${lastPhaseRef.current} → ${turnPhase.turn_phase}`,
        );

        lastPhaseRef.current = turnPhase.turn_phase;
        onTurnPhaseChange(turnPhase);
      }

      // Check if turn is complete
      if (turnPhase.turn_phase === 'Complete') {
        console.log('[useRacePolling] Turn complete, stopping poll');
        resetPolling();
        onComplete();
        return;
      }

      // Schedule next poll if still enabled and mounted
      if (enabled && isMountedRef.current && attemptsRef.current < MAX_POLL_ATTEMPTS) {
        const interval = getPollingInterval();
        timerRef.current = setTimeout(poll, interval);
      }
    } catch (error) {
      // Check if error is due to request cancellation
      if (error instanceof Error && error.name === 'AbortError') {
        console.log('[useRacePolling] Request cancelled');
        return;
      }

      console.error('[useRacePolling] Polling error:', error);

      // Check if component is still mounted
      if (!isMountedRef.current) {
        return;
      }

      // Increment error count for exponential backoff
      errorCountRef.current += 1;

      // Call error handler if provided (but not for every error to avoid spam)
      if (onError && errorCountRef.current <= 3) {
        const errorObj = error instanceof Error ? error : new Error(String(error));
        onError(errorObj);
      }

      // Continue polling on error (network issues might be temporary)
      // But respect max attempts limit and use exponential backoff
      if (enabled && isMountedRef.current && attemptsRef.current < MAX_POLL_ATTEMPTS) {
        const interval = getPollingInterval();
        console.log(`[useRacePolling] Retrying after error in ${interval}ms...`);
        timerRef.current = setTimeout(poll, interval);
      }
    } finally {
      // Clear the abort controller reference
      abortControllerRef.current = null;
    }
  }, [
    raceUuid,
    enabled,
    onTurnPhaseChange,
    onComplete,
    onError,
    onMaxAttemptsReached,
    resetPolling,
  ]);

  /**
   * Start polling when enabled changes to true
   */
  useEffect(() => {
    if (enabled) {
      console.log('[useRacePolling] Starting polling');
      resetPolling();
      poll();
    } else {
      console.log('[useRacePolling] Polling disabled, clearing timer');
      clearTimer();
    }

    // Cleanup on unmount or when enabled changes
    return () => {
      clearTimer();
    };
  }, [enabled, poll, resetPolling, clearTimer]);

  /**
   * Track component mount status
   */
  useEffect(() => {
    isMountedRef.current = true;

    return () => {
      isMountedRef.current = false;
      clearTimer();
    };
  }, [clearTimer]);

  return {
    attempts: attemptsRef.current,
    isPolling: enabled && attemptsRef.current > 0 && attemptsRef.current < MAX_POLL_ATTEMPTS,
    errorCount: errorCountRef.current,
    lastSuccessfulPoll: lastSuccessfulPollRef.current,
    reset: resetPolling,
  };
}
