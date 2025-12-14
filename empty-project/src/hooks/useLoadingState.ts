import { useState, useCallback, useRef } from 'react';

/**
 * Loading state types
 */
export type LoadingStateType = 'initial' | 'action' | 'data' | 'polling' | 'refresh';

/**
 * Loading state configuration
 */
export interface LoadingState {
  isLoading: boolean;
  type: LoadingStateType;
  message?: string;
  progress?: number;
  startTime?: number;
}

/**
 * Loading state manager hook
 * Requirements: 3.4 - Show loading spinner during initial race load, Display loading state during action submission, Show polling status indicator
 */
export const useLoadingState = () => {
  const [loadingStates, setLoadingStates] = useState<Map<string, LoadingState>>(new Map());
  const timeoutRefs = useRef<Map<string, number>>(new Map());

  /**
   * Start loading state
   */
  const startLoading = useCallback(
    (
      key: string,
      type: LoadingStateType,
      options?: {
        message?: string;
        progress?: number;
        timeout?: number; // Auto-stop after timeout (ms)
      },
    ) => {
      const newState: LoadingState = {
        isLoading: true,
        type,
        message: options?.message,
        progress: options?.progress,
        startTime: Date.now(),
      };

      setLoadingStates((prev) => new Map(prev).set(key, newState));

      // Set auto-stop timeout if specified
      if (options?.timeout) {
        const timeoutId = setTimeout(() => {
          stopLoading(key);
        }, options.timeout);

        timeoutRefs.current.set(key, timeoutId);
      }
    },
    [],
  );

  /**
   * Update loading state
   */
  const updateLoading = useCallback(
    (key: string, updates: Partial<Pick<LoadingState, 'message' | 'progress'>>) => {
      setLoadingStates((prev) => {
        const current = prev.get(key);
        if (!current) return prev;

        const updated = { ...current, ...updates };
        return new Map(prev).set(key, updated);
      });
    },
    [],
  );

  /**
   * Stop loading state
   */
  const stopLoading = useCallback((key: string) => {
    setLoadingStates((prev) => {
      const newMap = new Map(prev);
      newMap.delete(key);
      return newMap;
    });

    // Clear timeout if exists
    const timeoutId = timeoutRefs.current.get(key);
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutRefs.current.delete(key);
    }
  }, []);

  /**
   * Stop all loading states
   */
  const stopAllLoading = useCallback(() => {
    setLoadingStates(new Map());

    // Clear all timeouts
    timeoutRefs.current.forEach((timeoutId) => clearTimeout(timeoutId));
    timeoutRefs.current.clear();
  }, []);

  /**
   * Check if specific loading state is active
   */
  const isLoading = useCallback(
    (key: string): boolean => {
      return loadingStates.get(key)?.isLoading ?? false;
    },
    [loadingStates],
  );

  /**
   * Get specific loading state
   */
  const getLoadingState = useCallback(
    (key: string): LoadingState | undefined => {
      return loadingStates.get(key);
    },
    [loadingStates],
  );

  /**
   * Check if any loading state is active
   */
  const isAnyLoading = useCallback((): boolean => {
    return Array.from(loadingStates.values()).some((state) => state.isLoading);
  }, [loadingStates]);

  /**
   * Get all active loading states
   */
  const getActiveLoadingStates = useCallback((): LoadingState[] => {
    return Array.from(loadingStates.values()).filter((state) => state.isLoading);
  }, [loadingStates]);

  /**
   * Get loading duration for a specific key
   */
  const getLoadingDuration = useCallback(
    (key: string): number => {
      const state = loadingStates.get(key);
      if (!state?.startTime) return 0;
      return Date.now() - state.startTime;
    },
    [loadingStates],
  );

  return {
    // State management
    startLoading,
    updateLoading,
    stopLoading,
    stopAllLoading,

    // State queries
    isLoading,
    getLoadingState,
    isAnyLoading,
    getActiveLoadingStates,
    getLoadingDuration,

    // Raw state for advanced usage
    loadingStates: Array.from(loadingStates.entries()),
  };
};

/**
 * Predefined loading state keys for consistency
 */
export const LOADING_KEYS = {
  RACE_INIT: 'race_init',
  CAR_DATA: 'car_data',
  PERFORMANCE_PREVIEW: 'performance_preview',
  TURN_PHASE: 'turn_phase',
  LOCAL_VIEW: 'local_view',
  BOOST_AVAILABILITY: 'boost_availability',
  LAP_HISTORY: 'lap_history',
  SUBMIT_ACTION: 'submit_action',
  POLLING: 'polling',
  REFRESH: 'refresh',
} as const;

/**
 * Async operation wrapper with loading state
 */
export const useAsyncWithLoading = () => {
  const { startLoading, stopLoading, updateLoading, isLoading } = useLoadingState();

  const executeWithLoading = useCallback(
    async <T>(
      key: string,
      asyncFn: () => Promise<T>,
      options?: {
        type?: LoadingStateType;
        message?: string;
        onProgress?: (progress: number) => void;
        timeout?: number;
      },
    ): Promise<T> => {
      try {
        startLoading(key, options?.type || 'data', {
          message: options?.message,
          timeout: options?.timeout,
        });

        // Set up progress callback if provided
        if (options?.onProgress) {
          const progressInterval = setInterval(() => {
            // Simulate progress - in real implementation, this would come from the async operation
            const duration = Date.now();
            const progress = Math.min(90, ((duration % 3000) / 3000) * 100);
            updateLoading(key, { progress });
            options.onProgress!(progress);
          }, 100);

          try {
            const result = await asyncFn();
            clearInterval(progressInterval);
            return result;
          } catch (error) {
            clearInterval(progressInterval);
            throw error;
          }
        } else {
          return await asyncFn();
        }
      } finally {
        stopLoading(key);
      }
    },
    [startLoading, stopLoading, updateLoading],
  );

  return {
    executeWithLoading,
    isLoading,
    startLoading,
    stopLoading,
    updateLoading,
  };
};

export default useLoadingState;
