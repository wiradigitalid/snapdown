import React from 'react';
import { Button } from './Button';

export interface EmptyStateProps {
  heading: string;
  description: string;
  actionLabel?: string;
  onAction?: () => void;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  heading,
  description,
  actionLabel,
  onAction,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 'var(--space-6)',
        textAlign: 'center',
        gap: 'var(--space-2)',
      }}
    >
      <h3
        style={{
          margin: 0,
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-lg)',
          fontWeight: 600,
          color: 'var(--color-text)',
        }}
      >
        {heading}
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
        {description}
      </p>
      {actionLabel && onAction && (
        <div style={{ marginTop: 'var(--space-4)' }}>
          <Button variant="primary" onClick={onAction}>
            {actionLabel}
          </Button>
        </div>
      )}
    </div>
  );
};
