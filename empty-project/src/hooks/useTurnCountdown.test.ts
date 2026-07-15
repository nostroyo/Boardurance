/**
 * useTurnCountdown — per-turn countdown for multiplayer races.
 *
 * The server is the time authority: every poll delivers a fresh
 * `seconds_remaining` and the hook re-syncs to it, ticking down locally at
 * 1 s between polls so client clock skew never accumulates. `onExpire` fires
 * once per armed countdown when it reaches zero — the AFK auto-advance
 * trigger (the backend will auto-play the turn; the client must start
 * polling to pick it up).
 */
import { renderHook, act } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useTurnCountdown } from './useTurnCountdown';

describe('useTurnCountdown', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('ticks down locally at one-second intervals', () => {
    const { result } = renderHook(() => useTurnCountdown({ secondsRemaining: 45 }));
    expect(result.current).toBe(45);

    act(() => vi.advanceTimersByTime(2000));
    expect(result.current).toBe(43);
  });

  it('re-syncs to every fresh server value', () => {
    const { result, rerender } = renderHook(
      ({ s }: { s: number | null }) => useTurnCountdown({ secondsRemaining: s }),
      { initialProps: { s: 45 as number | null } },
    );
    act(() => vi.advanceTimersByTime(5000));
    expect(result.current).toBe(40);

    rerender({ s: 30 });
    expect(result.current).toBe(30);
  });

  it('stays null without a deadline (solo races)', () => {
    const { result } = renderHook(() => useTurnCountdown({ secondsRemaining: null }));
    expect(result.current).toBeNull();

    act(() => vi.advanceTimersByTime(3000));
    expect(result.current).toBeNull();
  });

  it('clamps at zero and never goes negative', () => {
    const { result } = renderHook(() => useTurnCountdown({ secondsRemaining: 1 }));
    act(() => vi.advanceTimersByTime(5000));
    expect(result.current).toBe(0);
  });

  it('fires onExpire exactly once when the countdown reaches zero', () => {
    const onExpire = vi.fn();
    renderHook(() => useTurnCountdown({ secondsRemaining: 2, onExpire }));

    act(() => vi.advanceTimersByTime(2000));
    expect(onExpire).toHaveBeenCalledTimes(1);

    act(() => vi.advanceTimersByTime(3000));
    expect(onExpire).toHaveBeenCalledTimes(1);
  });

  it('fires onExpire when the server already reports zero', () => {
    const onExpire = vi.fn();
    renderHook(() => useTurnCountdown({ secondsRemaining: 0, onExpire }));
    expect(onExpire).toHaveBeenCalledTimes(1);
  });

  it('re-arms after a fresh positive server value (next turn)', () => {
    const onExpire = vi.fn();
    const { rerender } = renderHook(
      ({ s }: { s: number | null }) => useTurnCountdown({ secondsRemaining: s, onExpire }),
      { initialProps: { s: 1 as number | null } },
    );
    act(() => vi.advanceTimersByTime(1000));
    expect(onExpire).toHaveBeenCalledTimes(1);

    rerender({ s: 60 });
    act(() => vi.advanceTimersByTime(60_000));
    expect(onExpire).toHaveBeenCalledTimes(2);
  });

  it('never fires onExpire without a deadline', () => {
    const onExpire = vi.fn();
    renderHook(() => useTurnCountdown({ secondsRemaining: null, onExpire }));
    act(() => vi.advanceTimersByTime(10_000));
    expect(onExpire).not.toHaveBeenCalled();
  });
});
