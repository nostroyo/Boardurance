/**
 * useDebouncedCallback - Custom hook for debouncing function calls
 *
 * This hook creates a debounced version of a callback function that delays
 * execution until after the specified delay has passed since the last call.
 *
 * Features:
 * - Configurable delay
 * - Automatic cleanup on unmount
 * - Cancel pending calls
 * - Immediate execution option
 *
 * Requirements: 12.3
 */

import { useCallback, useRef, useEffect } from 'react';

export interface UseDebouncedCallbackOptions {
  delay: number;
  leading?: boolean; // Execute immediately on first call
  trailing?: boolean; // Execute after delay (default: true)
}

/**
 * Custom hook for debouncing callback functions
 */
export function useDebouncedCallback<T extends (...args: any[]) => any>(
  callback: T,
  options: UseDebouncedCallbackOptions,
): {
  debouncedCallback: T;
  cancel: () => void;
  flush: () => void;
} {
  const { delay, leading = false, trailing = true } = options;

  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastCallTimeRef = useRef<number>(0);
  const lastArgsRef = useRef<Parameters<T> | undefined>(undefined);
  const hasLeadingExecutedRef = useRef<boolean>(false);

  /**
   * Cancel any pending debounced call
   */
  const cancel = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    hasLeadingExecutedRef.current = false;
  }, []);

  /**
   * Execute the callback immediately with the last arguments
   */
  const flush = useCallback(() => {
    if (timeoutRef.current && lastArgsRef.current) {
      cancel();
      callback(...lastArgsRef.current);
    }
  }, [callback, cancel]);

  /**
   * The debounced callback function
   */
  const debouncedCallback = useCallback(
    ((...args: Parameters<T>) => {
      const now = Date.now();
      lastArgsRef.current = args;
      lastCallTimeRef.current = now;

      // Handle leading edge execution
      if (leading && !hasLeadingExecutedRef.current) {
        hasLeadingExecutedRef.current = true;
        callback(...args);

        // If trailing is false, we're done
        if (!trailing) {
          return;
        }
      }

      // Cancel any existing timeout
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      // Set up new timeout for trailing execution
      if (trailing) {
        timeoutRef.current = setTimeout(() => {
          // Only execute if this is still the latest call
          if (lastArgsRef.current && lastArgsRef.current === args) {
            callback(...args);
          }
          hasLeadingExecutedRef.current = false;
          timeoutRef.current = null;
        }, delay);
      }
    }) as T,
    [callback, delay, leading, trailing],
  );

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      cancel();
    };
  }, [cancel]);

  return {
    debouncedCallback,
    cancel,
    flush,
  };
}

/**
 * Simplified debounced callback hook with default options
 */
export function useDebounce<T extends (...args: any[]) => any>(callback: T, delay: number): T {
  const { debouncedCallback } = useDebouncedCallback(callback, {
    delay,
    trailing: true,
  });

  return debouncedCallback;
}
