import React, { useEffect } from 'react';

export interface ToastProps {
  message: string;
  actionLabel?: string;
  onAction?: () => void;
  durationMs?: number;
  onDismiss: () => void;
}

export const Toast: React.FC<ToastProps> = ({
  message,
  actionLabel,
  onAction,
  durationMs = 3000,
  onDismiss,
}) => {
  useEffect(() => {
    if (durationMs <= 0) return;
    const timer = setTimeout(() => {
      onDismiss();
    }, durationMs);
    return () => clearTimeout(timer);
  }, [durationMs, onDismiss]);

  return (
    <div
      role="status"
      aria-live="polite"
      tabIndex={-1}
      style={{
        position: 'fixed',
        bottom: 'var(--space-5)',
        right: 'var(--space-5)',
        zIndex: 'var(--z-toast)',
        backgroundColor: 'var(--color-surface-raised)',
        color: 'var(--color-text)',
        border: '1px solid var(--color-border)',
        boxShadow: 'var(--shadow-raised)',
        borderRadius: 'var(--radius-md)',
        padding: 'var(--space-3) var(--space-4)',
        display: 'flex',
        alignItems: 'center',
        gap: 'var(--space-3)',
        fontFamily: 'var(--font-ui)',
        fontSize: 'var(--text-sm)',
        pointerEvents: 'none',
      }}
    >
      <span>{message}</span>
      {actionLabel && onAction && (
        <button
          type="button"
          onClick={onAction}
          style={{
            background: 'none',
            border: 'none',
            color: 'var(--color-accent)',
            fontFamily: 'var(--font-ui)',
            fontSize: 'var(--text-sm)',
            fontWeight: 600,
            cursor: 'pointer',
            padding: 0,
            pointerEvents: 'auto',
          }}
        >
          {actionLabel}
        </button>
      )}
    </div>
  );
};
