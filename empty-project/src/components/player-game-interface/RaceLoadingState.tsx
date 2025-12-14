import React from 'react';
import { LoadingSpinner } from '../LoadingSpinner';
import { SkeletonLoader, SkeletonCard } from '../SkeletonLoader';

interface RaceLoadingStateProps {
  type: 'initial' | 'action' | 'data' | 'polling';
  message?: string;
  progress?: number; // 0-100 for progress bar
  showSkeleton?: boolean;
}

/**
 * Comprehensive loading state component for race interface
 * Requirements: 3.4 - Show loading spinner during initial race load, Display loading state during action submission, Show polling status indicator
 */
export const RaceLoadingState: React.FC<RaceLoadingStateProps> = ({
  type,
  message,
  progress,
  showSkeleton = false,
}) => {
  const getLoadingConfig = () => {
    switch (type) {
      case 'initial':
        return {
          title: 'Loading Race',
          defaultMessage: 'Initializing race data...',
          spinnerSize: 'xl' as const,
          showProgress: true,
        };

      case 'action':
        return {
          title: 'Submitting Action',
          defaultMessage: 'Submitting your boost selection...',
          spinnerSize: 'lg' as const,
          showProgress: false,
        };

      case 'data':
        return {
          title: 'Loading Data',
          defaultMessage: 'Fetching race information...',
          spinnerSize: 'md' as const,
          showProgress: false,
        };

      case 'polling':
        return {
          title: 'Processing Turn',
          defaultMessage: 'Waiting for turn to complete...',
          spinnerSize: 'md' as const,
          showProgress: false,
          pulsing: true,
        };

      default:
        return {
          title: 'Loading',
          defaultMessage: 'Please wait...',
          spinnerSize: 'md' as const,
          showProgress: false,
        };
    }
  };

  const config = getLoadingConfig();
  const displayMessage = message || config.defaultMessage;

  // Full screen loading for initial load
  if (type === 'initial') {
    return (
      <div className="min-h-screen bg-gray-900 flex items-center justify-center">
        <div className="text-center max-w-md">
          <LoadingSpinner size={config.spinnerSize} color="blue" className="mx-auto mb-6" />
          <h2 className="text-white text-2xl font-bold mb-2">{config.title}</h2>
          <p className="text-gray-300 mb-6">{displayMessage}</p>

          {config.showProgress && progress !== undefined && (
            <div className="w-full bg-gray-700 rounded-full h-2 mb-4">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
              />
            </div>
          )}

          {showSkeleton && (
            <div className="mt-8 space-y-4">
              <SkeletonCard />
              <SkeletonCard />
            </div>
          )}
        </div>
      </div>
    );
  }

  // Inline loading states
  return (
    <div className="flex items-center justify-center p-4">
      <div className="flex items-center gap-3">
        <LoadingSpinner
          size={config.spinnerSize}
          color="blue"
          className={config.pulsing ? 'opacity-75' : ''}
        />
        <div>
          <p className="text-white font-semibold">{config.title}</p>
          <p className="text-gray-400 text-sm">{displayMessage}</p>
        </div>
      </div>
    </div>
  );
};

/**
 * Polling indicator component
 */
export const PollingIndicator: React.FC<{
  isActive: boolean;
  message?: string;
  attempts?: number;
  maxAttempts?: number;
}> = ({ isActive, message = 'Polling for updates...', attempts, maxAttempts }) => {
  if (!isActive) return null;

  return (
    <div className="flex items-center gap-2 text-blue-400 text-sm">
      <LoadingSpinner size="sm" color="blue" />
      <span>{message}</span>
      {attempts !== undefined && maxAttempts !== undefined && (
        <span className="text-gray-500">
          ({attempts}/{maxAttempts})
        </span>
      )}
    </div>
  );
};

/**
 * Action loading overlay
 */
export const ActionLoadingOverlay: React.FC<{
  isVisible: boolean;
  message?: string;
  onCancel?: () => void;
}> = ({ isVisible, message = 'Processing...', onCancel }) => {
  if (!isVisible) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 max-w-sm w-full mx-4">
        <div className="text-center">
          <LoadingSpinner size="lg" color="blue" className="mx-auto mb-4" />
          <h3 className="text-white text-lg font-semibold mb-2">Processing</h3>
          <p className="text-gray-300 mb-4">{message}</p>

          {onCancel && (
            <button
              onClick={onCancel}
              className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded transition-colors"
            >
              Cancel
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

/**
 * Data loading placeholder
 */
export const DataLoadingPlaceholder: React.FC<{
  type: 'card' | 'list' | 'table' | 'chart';
  count?: number;
  className?: string;
}> = ({ type, count = 1, className = '' }) => {
  const renderSkeleton = () => {
    switch (type) {
      case 'card':
        return Array.from({ length: count }).map((_, index) => (
          <SkeletonCard key={index} className="mb-4" />
        ));

      case 'list':
        return (
          <div className="space-y-3">
            {Array.from({ length: count }).map((_, index) => (
              <div key={index} className="flex items-center gap-3">
                <SkeletonLoader variant="circular" width={40} height={40} />
                <div className="flex-1">
                  <SkeletonLoader height="1rem" width="60%" className="mb-2" />
                  <SkeletonLoader height="0.75rem" width="40%" />
                </div>
              </div>
            ))}
          </div>
        );

      case 'table':
        return (
          <div className="space-y-2">
            {Array.from({ length: count }).map((_, index) => (
              <div key={index} className="flex gap-4">
                <SkeletonLoader width="25%" height="2rem" />
                <SkeletonLoader width="20%" height="2rem" />
                <SkeletonLoader width="20%" height="2rem" />
                <SkeletonLoader width="15%" height="2rem" />
              </div>
            ))}
          </div>
        );

      case 'chart':
        return (
          <div className="space-y-4">
            <SkeletonLoader height="200px" />
            <div className="flex justify-between">
              {Array.from({ length: 5 }).map((_, index) => (
                <SkeletonLoader key={index} width="15%" height="1rem" />
              ))}
            </div>
          </div>
        );

      default:
        return <SkeletonCard />;
    }
  };

  return <div className={`animate-pulse ${className}`}>{renderSkeleton()}</div>;
};

export default RaceLoadingState;
