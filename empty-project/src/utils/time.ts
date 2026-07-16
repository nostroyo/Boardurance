// Time formatting helpers for the player game interface

/**
 * Format a whole-second duration as zero-padded MM:SS.
 *
 * Used by the multiplayer turn countdown: formatTime(45) -> "00:45",
 * formatTime(600) -> "10:00". Callers render the countdown only while
 * `seconds >= 0`, so negative input is out of contract.
 *
 * @param seconds - Non-negative duration in seconds
 * @returns Zero-padded MM:SS string
 */
export const formatTime = (seconds: number): string => {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
};
