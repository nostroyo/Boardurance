/**
 * useTurnCountdown - per-turn submission countdown for multiplayer races.
 *
 * The server is the time authority: each turn-phase poll delivers a fresh
 * `seconds_remaining` (clamped ≥ 0 server-side) and this hook re-syncs to it,
 * ticking down locally at 1 s between polls so a wrong client clock can never
 * skew the display by more than one poll interval.
 *
 * `onExpire` fires exactly once per armed countdown when it reaches zero
 * (whether by local ticking or because the server already reported zero).
 * It re-arms when a fresh positive value arrives — i.e. the next turn. This
 * is the AFK auto-advance trigger: the backend auto-plays the expired turn,
 * and the client must start polling to pick the result up.
 *
 * Returns the current remaining seconds, or `null` when there is no deadline
 * (solo races never show a countdown).
 */
import { useEffect, useRef, useState } from 'react';

export interface UseTurnCountdownOptions {
  /** Latest server-reported seconds until the deadline; null = no deadline. */
  secondsRemaining: number | null;
  /**
   * Identity of the poll payload the value came from (e.g. the turnPhase
   * object). Two consecutive turns can report the identical number (both
   * freshly armed at the same timeout), so the numeric value alone cannot
   * signal "a fresh server sync arrived" — the key does.
   */
  syncKey?: unknown;
  /** Fired once per armed countdown when it reaches zero. */
  onExpire?: () => void;
}

export function useTurnCountdown({
  secondsRemaining,
  syncKey,
  onExpire,
}: UseTurnCountdownOptions): number | null {
  const [remaining, setRemaining] = useState<number | null>(secondsRemaining);
  const expiredRef = useRef(false);
  // Keep the latest callback without retriggering the expiry effect.
  const onExpireRef = useRef(onExpire);
  onExpireRef.current = onExpire;

  // Re-sync on every fresh server payload; tick locally between polls.
  useEffect(() => {
    setRemaining(secondsRemaining);

    if (secondsRemaining == null) {
      expiredRef.current = false;
      return;
    }
    if (secondsRemaining > 0) {
      // A fresh positive value means a new (or still-running) turn: re-arm.
      expiredRef.current = false;
    } else {
      // Already at zero: nothing to tick down — the expiry effect below
      // handles the one-shot onExpire; no interval needed.
      return;
    }

    const id = setInterval(() => {
      setRemaining((prev) => (prev == null ? null : Math.max(0, prev - 1)));
    }, 1000);

    return () => clearInterval(id);
  }, [secondsRemaining, syncKey]);

  // Fire once per armed countdown when it hits zero.
  useEffect(() => {
    if (remaining === 0 && !expiredRef.current) {
      expiredRef.current = true;
      onExpireRef.current?.();
    }
  }, [remaining]);

  return remaining;
}
