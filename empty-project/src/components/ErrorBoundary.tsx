import React, { Component } from 'react';
import type { ErrorInfo, ReactNode } from 'react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary component to catch and handle React component errors
 * Requirements: 9.1 - Implement error boundaries for component errors
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log the error for debugging
    console.error('[ErrorBoundary] Component error caught:', error);
    console.error('[ErrorBoundary] Error info:', errorInfo);

    // Update state with error info
    this.setState({
      error,
      errorInfo,
    });

    // Call parent error handler if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  handleRetry = () => {
    // Reset error state to retry rendering
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <div className="min-h-screen bg-gray-900 flex items-center justify-center p-4">
          <div className="bg-red-900 border border-red-700 rounded-lg p-6 max-w-2xl w-full">
            <div className="flex items-center mb-4">
              <span className="text-red-400 text-2xl mr-3">⚠️</span>
              <h2 className="text-white text-xl font-bold">Something went wrong</h2>
            </div>

            <p className="text-red-200 mb-4">
              An unexpected error occurred in the application. This has been logged for debugging.
            </p>

            {this.state.error && (
              <div className="bg-red-800 border border-red-600 rounded p-3 mb-4">
                <p className="text-red-100 font-mono text-sm">{this.state.error.message}</p>
              </div>
            )}

            <div className="flex gap-3">
              <button
                onClick={this.handleRetry}
                className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded font-semibold transition-colors"
              >
                Try Again
              </button>
              <button
                onClick={() => window.location.reload()}
                className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded font-semibold transition-colors"
              >
                Reload Page
              </button>
              <button
                onClick={() => (window.location.href = '/')}
                className="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded font-semibold transition-colors"
              >
                Go Home
              </button>
            </div>

            {/* Development mode: Show error details */}
            {import.meta.env.DEV && this.state.errorInfo && (
              <details className="mt-4">
                <summary className="text-red-300 cursor-pointer hover:text-red-200">
                  Show Error Details (Development)
                </summary>
                <div className="mt-2 bg-gray-800 border border-gray-600 rounded p-3">
                  <pre className="text-gray-300 text-xs overflow-auto">
                    {this.state.error?.stack}
                  </pre>
                  <pre className="text-gray-400 text-xs overflow-auto mt-2">
                    {this.state.errorInfo.componentStack}
                  </pre>
                </div>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Higher-order component to wrap components with error boundary
 */
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  fallback?: ReactNode,
  onError?: (error: Error, errorInfo: ErrorInfo) => void,
) {
  const WrappedComponent = (props: P) => (
    <ErrorBoundary fallback={fallback} onError={onError}>
      <Component {...props} />
    </ErrorBoundary>
  );

  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;

  return WrappedComponent;
}

export default ErrorBoundary;
