import { describe, it, expect } from 'vitest';
import { formatTime } from './time';

describe('formatTime', () => {
  it('zero-pads both minutes and seconds (MM:SS)', () => {
    // Pins the same output as the countdown in RaceStatusPanel /
    // RaceContainer.multiplayer.test.tsx (seconds_remaining: 45 -> "00:45").
    expect(formatTime(45)).toBe('00:45');
    expect(formatTime(5)).toBe('00:05');
  });

  it('rolls seconds into minutes', () => {
    expect(formatTime(60)).toBe('01:00');
    expect(formatTime(75)).toBe('01:15');
    expect(formatTime(600)).toBe('10:00');
  });

  it('handles zero (deadline passed, resolving on next poll)', () => {
    expect(formatTime(0)).toBe('00:00');
  });
});
