import { useState, useCallback } from 'react';
import type {
  Toast,
  ToastType,
  ToastAction,
} from '../components/player-game-interface/ToastNotification';

export const useToast = () => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback(
    (
      type: ToastType,
      title: string,
      message: string,
      options?: {
        duration?: number;
        actions?: ToastAction[];
        persistent?: boolean;
      },
    ) => {
      const id = `toast-${Date.now()}-${Math.random()}`;
      const newToast: Toast = {
        id,
        type,
        title,
        message,
        duration: options?.duration,
        actions: options?.actions,
        persistent: options?.persistent,
      };

      setToasts((prev) => [...prev, newToast]);
      return id;
    },
    [],
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const showSuccess = useCallback(
    (title: string, message: string, options?: { duration?: number; actions?: ToastAction[] }) => {
      return addToast('success', title, message, options);
    },
    [addToast],
  );

  const showInfo = useCallback(
    (title: string, message: string, options?: { duration?: number; actions?: ToastAction[] }) => {
      return addToast('info', title, message, options);
    },
    [addToast],
  );

  const showWarning = useCallback(
    (title: string, message: string, options?: { duration?: number; actions?: ToastAction[] }) => {
      return addToast('warning', title, message, options);
    },
    [addToast],
  );

  const showError = useCallback(
    (
      title: string,
      message: string,
      options?: { duration?: number; actions?: ToastAction[]; persistent?: boolean },
    ) => {
      return addToast('error', title, message, options);
    },
    [addToast],
  );

  // Enhanced error notification with retry options
  const showErrorWithRetry = useCallback(
    (
      title: string,
      message: string,
      onRetry: () => void,
      options?: {
        duration?: number;
        retryLabel?: string;
        showDismiss?: boolean;
        persistent?: boolean;
      },
    ) => {
      const actions: ToastAction[] = [
        {
          label: options?.retryLabel || 'Retry',
          action: onRetry,
          style: 'primary',
        },
      ];

      if (options?.showDismiss !== false) {
        actions.push({
          label: 'Dismiss',
          action: () => {}, // Will be handled by toast close
          style: 'secondary',
        });
      }

      return addToast('error', title, message, {
        duration: options?.duration,
        actions,
        persistent: options?.persistent,
      });
    },
    [addToast],
  );

  // Network error notification
  const showNetworkError = useCallback(
    (
      onRetry: () => void,
      options?: {
        message?: string;
        retryLabel?: string;
      },
    ) => {
      return showErrorWithRetry(
        'Connection Error',
        options?.message || 'Unable to connect to server. Please check your internet connection.',
        onRetry,
        {
          retryLabel: options?.retryLabel || 'Retry',
          persistent: true,
        },
      );
    },
    [showErrorWithRetry],
  );

  // API error notification
  const showAPIError = useCallback(
    (
      error: string,
      onRetry?: () => void,
      options?: {
        retryLabel?: string;
        showRetry?: boolean;
      },
    ) => {
      if (onRetry && options?.showRetry !== false) {
        return showErrorWithRetry('API Error', error, onRetry, {
          retryLabel: options?.retryLabel || 'Retry',
          persistent: true,
        });
      } else {
        return showError('API Error', error, {
          duration: 8000, // Longer duration for errors without retry
        });
      }
    },
    [showError, showErrorWithRetry],
  );

  // Turn event notifications (Requirements 8.4)
  const showTurnSubmitted = useCallback(() => {
    return showSuccess('Action Submitted', 'Your boost selection has been submitted successfully', {
      duration: 3000,
    });
  }, [showSuccess]);

  const showTurnCompleted = useCallback(() => {
    return showSuccess(
      'Turn Complete',
      'Turn has been processed. Your position has been updated.',
      { duration: 4000 },
    );
  }, [showSuccess]);

  const showLapFinished = useCallback(
    (lapNumber: number) => {
      return showSuccess('Lap Complete', `You have completed lap ${lapNumber}!`, {
        duration: 5000,
      });
    },
    [showSuccess],
  );

  const showRaceFinished = useCallback(
    (position: number) => {
      return showSuccess(
        'Race Complete!',
        `Congratulations! You finished in position ${position}.`,
        { duration: 10000 },
      );
    },
    [showSuccess],
  );

  // Clear all toasts
  const clearAllToasts = useCallback(() => {
    setToasts([]);
  }, []);

  return {
    toasts,
    addToast,
    removeToast,
    showSuccess,
    showInfo,
    showWarning,
    showError,
    showErrorWithRetry,
    showNetworkError,
    showAPIError,
    showTurnSubmitted,
    showTurnCompleted,
    showLapFinished,
    showRaceFinished,
    clearAllToasts,
  };
};
