/**
 * Notification Service
 * Centralized service for managing user notifications
 * Requirements: 8.4 - Show error notifications with retry options, Display success messages for actions
 */

import type { ErrorState } from './errorHandling';
import type { ToastAction } from '../components/player-game-interface/ToastNotification';

/**
 * Notification types for different race events
 */
export type RaceEventType =
  | 'turn_submitted'
  | 'turn_completed'
  | 'lap_finished'
  | 'race_finished'
  | 'phase_changed'
  | 'boost_replenished'
  | 'error_occurred'
  | 'recovery_completed';

/**
 * Race event notification data
 */
export interface RaceEventNotification {
  type: RaceEventType;
  title: string;
  message: string;
  data?: any;
  actions?: ToastAction[];
  persistent?: boolean;
  duration?: number;
}

/**
 * Notification service interface
 */
export interface NotificationService {
  showSuccess: (title: string, message: string, options?: { duration?: number }) => string;
  showInfo: (title: string, message: string, options?: { duration?: number }) => string;
  showWarning: (title: string, message: string, options?: { duration?: number }) => string;
  showError: (
    title: string,
    message: string,
    options?: { duration?: number; persistent?: boolean },
  ) => string;
  showErrorWithRetry: (
    title: string,
    message: string,
    onRetry: () => void,
    options?: { retryLabel?: string },
  ) => string;
  showNetworkError: (onRetry: () => void, options?: { message?: string }) => string;
  showAPIError: (error: string, onRetry?: () => void, options?: { showRetry?: boolean }) => string;
  clearAll: () => void;
}

/**
 * Create race event notifications
 */
export function createRaceEventNotification(
  type: RaceEventType,
  data?: any,
): RaceEventNotification {
  switch (type) {
    case 'turn_submitted':
      return {
        type,
        title: 'Action Submitted',
        message: 'Your boost selection has been submitted successfully',
        duration: 3000,
      };

    case 'turn_completed':
      return {
        type,
        title: 'Turn Complete',
        message: 'Turn has been processed. Your position has been updated.',
        duration: 4000,
      };

    case 'lap_finished':
      return {
        type,
        title: 'Lap Complete',
        message: `You have completed lap ${data?.lapNumber || 'N/A'}!`,
        duration: 5000,
      };

    case 'race_finished':
      return {
        type,
        title: 'Race Complete!',
        message: `Congratulations! You finished in position ${data?.position || 'N/A'}.`,
        duration: 10000,
      };

    case 'phase_changed':
      const phaseMessages = {
        WaitingForPlayers: 'Waiting for player actions',
        AllSubmitted: 'All players have submitted their actions',
        Processing: 'Processing turn results...',
        Complete: 'Turn processing complete',
      };

      return {
        type,
        title: 'Turn Phase Update',
        message: phaseMessages[data?.phase as keyof typeof phaseMessages] || 'Turn phase changed',
        duration: 3000,
      };

    case 'boost_replenished':
      return {
        type,
        title: 'Boost Cards Replenished',
        message: `Your boost hand has been replenished for cycle ${data?.cycle || 'N/A'}`,
        duration: 4000,
      };

    case 'error_occurred':
      return {
        type,
        title: 'Error',
        message: data?.message || 'An unexpected error occurred',
        persistent: true,
        actions: data?.actions,
      };

    case 'recovery_completed':
      return {
        type,
        title: 'State Synchronized',
        message: 'Race state has been synchronized successfully',
        duration: 3000,
      };

    default:
      return {
        type,
        title: 'Notification',
        message: 'Race event occurred',
        duration: 3000,
      };
  }
}

/**
 * Create error notification from ErrorState
 */
export function createErrorNotification(
  errorState: ErrorState,
  onRetry?: () => void,
  onRefresh?: () => void,
  onNavigate?: () => void,
): RaceEventNotification {
  const actions: ToastAction[] = [];

  // Add appropriate actions based on error type and retryability
  if (errorState.retryable && onRetry) {
    actions.push({
      label: 'Retry',
      action: onRetry,
      style: 'primary',
    });
  }

  if (errorState.type === 'state' && onRefresh) {
    actions.push({
      label: 'Refresh',
      action: onRefresh,
      style: 'primary',
    });
  }

  if (errorState.type === 'api' && errorState.message.includes('not found') && onNavigate) {
    actions.push({
      label: 'Return to Lobby',
      action: onNavigate,
      style: 'primary',
    });
  }

  // Always add dismiss option
  actions.push({
    label: 'Dismiss',
    action: () => {}, // Will be handled by toast close
    style: 'secondary',
  });

  return {
    type: 'error_occurred',
    title: getErrorTitle(errorState.type),
    message: errorState.message,
    persistent: true,
    actions,
  };
}

/**
 * Get user-friendly error title based on error type
 */
function getErrorTitle(errorType: string): string {
  switch (errorType) {
    case 'network':
      return 'Connection Error';
    case 'api':
      return 'Server Error';
    case 'validation':
      return 'Invalid Input';
    case 'state':
      return 'State Error';
    default:
      return 'Error';
  }
}

/**
 * Notification manager for race events
 */
export class RaceNotificationManager {
  private notificationService: NotificationService;
  private activeNotifications: Map<string, string> = new Map();

  constructor(notificationService: NotificationService) {
    this.notificationService = notificationService;
  }

  /**
   * Show race event notification
   */
  showRaceEvent(
    type: RaceEventType,
    data?: any,
    options?: {
      replaceExisting?: boolean;
      customMessage?: string;
    },
  ): string {
    // Remove existing notification of same type if requested
    if (options?.replaceExisting) {
      const existingId = this.activeNotifications.get(type);
      if (existingId) {
        // Note: We can't remove individual toasts with current implementation
        // This would need to be added to the toast service
      }
    }

    const notification = createRaceEventNotification(type, data);

    // Override message if provided
    if (options?.customMessage) {
      notification.message = options.customMessage;
    }

    let notificationId: string;

    if (notification.actions && notification.actions.length > 0) {
      // Show error with actions
      notificationId = this.notificationService.showError(
        notification.title,
        notification.message,
        {
          duration: notification.duration,
          persistent: notification.persistent,
        },
      );
    } else {
      // Show appropriate notification type
      switch (type) {
        case 'turn_submitted':
        case 'turn_completed':
        case 'lap_finished':
        case 'race_finished':
        case 'boost_replenished':
        case 'recovery_completed':
          notificationId = this.notificationService.showSuccess(
            notification.title,
            notification.message,
            { duration: notification.duration },
          );
          break;

        case 'phase_changed':
          notificationId = this.notificationService.showInfo(
            notification.title,
            notification.message,
            { duration: notification.duration },
          );
          break;

        case 'error_occurred':
          notificationId = this.notificationService.showError(
            notification.title,
            notification.message,
            {
              duration: notification.duration,
              persistent: notification.persistent,
            },
          );
          break;

        default:
          notificationId = this.notificationService.showInfo(
            notification.title,
            notification.message,
            { duration: notification.duration },
          );
          break;
      }
    }

    // Track active notification
    this.activeNotifications.set(type, notificationId);
    return notificationId;
  }

  /**
   * Show error notification with appropriate actions
   */
  showError(
    errorState: ErrorState,
    actions?: {
      onRetry?: () => void;
      onRefresh?: () => void;
      onNavigate?: () => void;
    },
  ): string {
    const notification = createErrorNotification(
      errorState,
      actions?.onRetry,
      actions?.onRefresh,
      actions?.onNavigate,
    );

    const notificationId = this.notificationService.showError(
      notification.title,
      notification.message,
      {
        persistent: notification.persistent,
      },
    );

    this.activeNotifications.set('error_occurred', notificationId);
    return notificationId;
  }

  /**
   * Show network error with retry
   */
  showNetworkError(onRetry: () => void, customMessage?: string): string {
    const notificationId = this.notificationService.showNetworkError(onRetry, {
      message: customMessage,
    });

    this.activeNotifications.set('error_occurred', notificationId);
    return notificationId;
  }

  /**
   * Show API error with optional retry
   */
  showAPIError(error: string, onRetry?: () => void, options?: { showRetry?: boolean }): string {
    const notificationId = this.notificationService.showAPIError(error, onRetry, options);

    this.activeNotifications.set('error_occurred', notificationId);
    return notificationId;
  }

  /**
   * Clear all notifications
   */
  clearAll(): void {
    this.notificationService.clearAll();
    this.activeNotifications.clear();
  }

  /**
   * Get active notification count
   */
  getActiveCount(): number {
    return this.activeNotifications.size;
  }
}

/**
 * Create notification manager with toast service
 */
export function createNotificationManager(
  notificationService: NotificationService,
): RaceNotificationManager {
  return new RaceNotificationManager(notificationService);
}
