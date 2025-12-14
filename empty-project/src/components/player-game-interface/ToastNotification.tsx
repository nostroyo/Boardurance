import React, { useEffect, useState } from 'react';

export type ToastType = 'success' | 'info' | 'warning' | 'error';

export interface ToastAction {
  label: string;
  action: () => void;
  style?: 'primary' | 'secondary';
}

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message: string;
  duration?: number;
  actions?: ToastAction[];
  persistent?: boolean; // Don't auto-hide if true
}

interface ToastNotificationProps {
  toast: Toast;
  onClose: (id: string) => void;
}

export const ToastNotification: React.FC<ToastNotificationProps> = ({ toast, onClose }) => {
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    // Trigger entrance animation
    setIsVisible(true);

    // Auto-hide after duration (unless persistent)
    if (!toast.persistent) {
      const duration = toast.duration ?? 5000;
      const timer = setTimeout(() => {
        setIsVisible(false);
        setTimeout(() => onClose(toast.id), 300); // Wait for exit animation
      }, duration);

      return () => clearTimeout(timer);
    }
  }, [toast.id, toast.duration, toast.persistent, onClose]);

  const handleClose = () => {
    setIsVisible(false);
    setTimeout(() => onClose(toast.id), 300);
  };

  const getToastStyles = () => {
    switch (toast.type) {
      case 'success':
        return {
          bg: 'bg-green-600',
          border: 'border-green-500',
          icon: '✅',
          iconColor: 'text-green-400',
        };
      case 'info':
        return {
          bg: 'bg-blue-600',
          border: 'border-blue-500',
          icon: 'ℹ️',
          iconColor: 'text-blue-400',
        };
      case 'warning':
        return {
          bg: 'bg-yellow-600',
          border: 'border-yellow-500',
          icon: '⚠️',
          iconColor: 'text-yellow-400',
        };
      case 'error':
        return {
          bg: 'bg-red-600',
          border: 'border-red-500',
          icon: '❌',
          iconColor: 'text-red-400',
        };
    }
  };

  const styles = getToastStyles();

  return (
    <div
      className={`${styles.bg} bg-opacity-90 border ${styles.border} rounded-lg shadow-lg p-4 min-w-[300px] max-w-md transform transition-all duration-300 ${
        isVisible ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0'
      }`}
    >
      <div className="flex items-start space-x-3">
        <span className={`text-2xl ${styles.iconColor}`} role="img" aria-label={toast.type}>
          {styles.icon}
        </span>
        <div className="flex-1 min-w-0">
          <p className="text-white font-semibold text-sm">{toast.title}</p>
          <p className="text-gray-200 text-xs mt-1">{toast.message}</p>

          {/* Action buttons */}
          {toast.actions && toast.actions.length > 0 && (
            <div className="flex gap-2 mt-3">
              {toast.actions.map((action, index) => (
                <button
                  key={index}
                  onClick={() => {
                    action.action();
                    handleClose();
                  }}
                  className={`px-3 py-1 rounded text-xs font-semibold transition-colors ${
                    action.style === 'primary'
                      ? 'bg-blue-600 hover:bg-blue-700 text-white'
                      : 'bg-gray-600 hover:bg-gray-700 text-gray-200'
                  }`}
                >
                  {action.label}
                </button>
              ))}
            </div>
          )}
        </div>
        <button
          onClick={handleClose}
          className="text-white hover:text-gray-300 focus:outline-none flex-shrink-0"
          aria-label="Close notification"
        >
          <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
            <path
              fillRule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </div>
    </div>
  );
};
