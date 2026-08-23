import React from 'react';
import { Button } from './Button';

export interface ErrorStateProps {
  title?: string;
  message: string;
  actionLabel?: string;
  onAction?: () => void;
  className?: string;
  style?: React.CSSProperties;
}

export const ErrorState: React.FC<ErrorStateProps> = ({
  title = 'Something went wrong',
  message,
  actionLabel = 'Try Again',
  onAction,
  className = '',
  style,
}) => {
  return (
    <div
      data-testid="error-state"
      className={className}
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 'var(--space-6)',
        textAlign: 'center',
        gap: 'var(--space-2)',
        backgroundColor: 'var(--color-danger-bg)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
        ...style,
      }}
    >
      <div
        style={{
          width: '2.5rem',
          height: '2.5rem',
          borderRadius: '50%',
          backgroundColor: 'var(--color-danger)',
          color: 'var(--color-danger-text)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontWeight: 700,
          fontSize: 'var(--text-lg)',
          marginBottom: 'var(--space-1)',
        }}
      >
        !
      </div>
      <h3
        style={{
          margin: 0,
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-base)',
          fontWeight: 600,
          color: 'var(--color-text)',
        }}
      >
        {title}
      </h3>
      <p
        style={{
          margin: 0,
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-sm)',
          color: 'var(--color-text-muted)',
          maxWidth: '24rem',
        }}
      >
        {message}
      </p>
      {actionLabel && onAction && (
        <div style={{ marginTop: 'var(--space-3)' }}>
          <Button variant="primary" onClick={onAction}>
            {actionLabel}
          </Button>
        </div>
      )}
    </div>
  );
};
