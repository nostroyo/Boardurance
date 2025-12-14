/**
 * Error Handling Utilities for Race API
 * Provides error categorization, retry logic, and user-friendly error messages
 */

/**
 * Error categories for different types of failures
 */
export type ErrorCategory = 'network' | 'api' | 'validation' | 'state';

/**
 * Structured error state for UI display
 */
export interface ErrorState {
  type: ErrorCategory;
  message: string;
  retryable: boolean;
  retryCount: number;
  originalError?: Error;
}

/**
 * Retry configuration for exponential backoff
 */
export interface RetryConfig {
  maxRetries: number;
  baseDelay: number; // milliseconds
  maxDelay: number; // milliseconds
}

/**
 * Default retry configuration
 */
export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 5,
  baseDelay: 1000, // 1 second
  maxDelay: 8000, // 8 seconds
};

/**
 * Categorize error based on error message and type
 */
export function categorizeError(error: Error | string): ErrorCategory {
  const errorMessage = typeof error === 'string' ? error : error.message;
  const lowerMessage = errorMessage.toLowerCase();

  // Network errors
  if (
    lowerMessage.includes('network') ||
    lowerMessage.includes('fetch') ||
    lowerMessage.includes('connection') ||
    lowerMessage.includes('timeout') ||
    lowerMessage.includes('dns') ||
    lowerMessage.includes('failed to fetch')
  ) {
    return 'network';
  }

  // Validation errors
  if (
    lowerMessage.includes('invalid') ||
    lowerMessage.includes('validation') ||
    lowerMessage.includes('required') ||
    lowerMessage.includes('must be') ||
    (lowerMessage.includes('boost') && lowerMessage.includes('available'))
  ) {
    return 'validation';
  }

  // State inconsistency errors
  if (
    lowerMessage.includes('state') ||
    lowerMessage.includes('inconsistent') ||
    lowerMessage.includes('unexpected') ||
    lowerMessage.includes('phase') ||
    lowerMessage.includes('already submitted')
  ) {
    return 'state';
  }

  // Default to API error
  return 'api';
}

/**
 * Determine if an error is retryable
 */
export function isRetryableError(errorType: ErrorCategory): boolean {
  // Network errors and some state errors are retryable
  return errorType === 'network' || errorType === 'state';
}

/**
 * Create structured error state from an error
 */
export function createErrorState(
  error: Error | string,
  context: string,
  retryCount: number = 0,
): ErrorState {
  const errorObj = typeof error === 'string' ? new Error(error) : error;
  const type = categorizeError(errorObj);
  const retryable = isRetryableError(type);
  const message = formatErrorMessage(errorObj, type, context);

  return {
    type,
    message,
    retryable,
    retryCount,
    originalError: errorObj,
  };
}

/**
 * Format error message for user display
 */
export function formatErrorMessage(
  error: Error | string,
  type: ErrorCategory,
  _context: string,
): string {
  const errorMessage = typeof error === 'string' ? error : error.message;

  // Map of specific error messages to user-friendly versions
  const errorMappings: Record<string, string> = {
    'Failed to fetch': 'Connection lost. Please check your internet connection.',
    'Network error': 'Connection lost. Retrying...',
    'Connection timeout': 'Request timed out. Retrying...',
    'Server unavailable': 'Server is temporarily unavailable. Retrying...',
    'Race not found': 'This race no longer exists or has been removed.',
    'Player not in race': 'You are not registered for this race.',
    'Invalid boost value': 'Please select a valid boost card (0-4).',
    'Boost card not available': 'This boost card has already been used.',
    'Action already submitted': 'You have already submitted your action for this turn.',
    'Race not in progress': 'This race is not currently active.',
    'Turn phase mismatch': 'Race state has changed. Refreshing...',
    '404': 'Resource not found.',
    '409': 'Conflict with current race state.',
    '500': 'Server error. Please try again.',
  };

  // Check for exact matches
  for (const [key, message] of Object.entries(errorMappings)) {
    if (errorMessage.includes(key)) {
      return message;
    }
  }

  // Category-specific default messages
  switch (type) {
    case 'network':
      return 'Connection lost. Retrying...';
    case 'validation':
      return `Invalid input: ${errorMessage}`;
    case 'state':
      return 'Race state has changed. Synchronizing...';
    case 'api':
      return `Error: ${errorMessage}`;
    default:
      return 'An unexpected error occurred. Please try again.';
  }
}

/**
 * Calculate delay for exponential backoff
 */
export function calculateBackoffDelay(
  retryCount: number,
  config: RetryConfig = DEFAULT_RETRY_CONFIG,
): number {
  const delay = config.baseDelay * Math.pow(2, retryCount);
  return Math.min(delay, config.maxDelay);
}

/**
 * Retry a function with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  config: RetryConfig = DEFAULT_RETRY_CONFIG,
  onRetry?: (attempt: number, error: Error) => void,
): Promise<T> {
  let lastError: Error;

  for (let attempt = 0; attempt <= config.maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      // Don't retry if we've exhausted attempts
      if (attempt >= config.maxRetries) {
        break;
      }

      // Check if error is retryable
      const errorType = categorizeError(lastError);
      if (!isRetryableError(errorType)) {
        throw lastError;
      }

      // Call retry callback if provided
      if (onRetry) {
        onRetry(attempt + 1, lastError);
      }

      // Wait before retrying
      const delay = calculateBackoffDelay(attempt, config);
      await new Promise((resolve) => setTimeout(resolve, delay));
    }
  }

  throw lastError!;
}

/**
 * Wrap an API call with error handling and retry logic
 * @param apiCall - The API function to call
 * @param context - Context string for error messages (e.g., "fetching car data")
 * @param config - Retry configuration
 * @param onError - Optional callback for error notifications
 */
export async function withErrorHandling<T>(
  apiCall: () => Promise<T>,
  context: string,
  config: RetryConfig = DEFAULT_RETRY_CONFIG,
  onError?: (errorState: ErrorState) => void,
): Promise<T> {
  try {
    return await retryWithBackoff(apiCall, config, (attempt, error) => {
      if (onError) {
        const errorState = createErrorState(error, context, attempt);
        onError(errorState);
      }
    });
  } catch (error) {
    const errorState = createErrorState(
      error instanceof Error ? error : new Error(String(error)),
      context,
      config.maxRetries,
    );

    if (onError) {
      onError(errorState);
    }

    // Re-throw the error for the caller to handle
    throw error;
  }
}

/**
 * Check if error requires navigation away from race
 */
export function requiresNavigation(errorType: ErrorCategory, errorMessage: string): boolean {
  const lowerMessage = errorMessage.toLowerCase();

  return (
    errorType === 'api' &&
    (lowerMessage.includes('race not found') ||
      lowerMessage.includes('player not in race') ||
      lowerMessage.includes('404'))
  );
}

/**
 * Get appropriate action for error
 */
export function getErrorAction(
  errorState: ErrorState,
): 'retry' | 'refresh' | 'navigate' | 'dismiss' {
  if (requiresNavigation(errorState.type, errorState.message)) {
    return 'navigate';
  }

  if (errorState.type === 'state') {
    return 'refresh';
  }

  if (errorState.retryable && errorState.retryCount < DEFAULT_RETRY_CONFIG.maxRetries) {
    return 'retry';
  }

  return 'dismiss';
}

/**
 * Get user-friendly action text
 */
export function getActionText(action: 'retry' | 'refresh' | 'navigate' | 'dismiss'): string {
  const actionTexts: Record<string, string> = {
    retry: 'Retry',
    refresh: 'Refresh',
    navigate: 'Return to Lobby',
    dismiss: 'Dismiss',
  };

  return actionTexts[action] || 'OK';
}
