import React from 'react';
import { ToastNotification } from './ToastNotification';
import type { Toast } from './ToastNotification';

interface ToastContainerProps {
  toasts: Toast[];
  onRemoveToast: (id: string) => void;
}

export const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onRemoveToast }) => {
  return (
    <div className="fixed top-4 right-4 z-50 space-y-2">
      {toasts.map((toast) => (
        <ToastNotification key={toast.id} toast={toast} onClose={onRemoveToast} />
      ))}
    </div>
  );
};
