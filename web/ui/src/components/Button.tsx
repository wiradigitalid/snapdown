import React, { ButtonHTMLAttributes } from 'react';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger';
  loading?: boolean;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = 'secondary',
  loading = false,
  disabled,
  style,
  ...props
}) => {
  const isPrimary = variant === 'primary';
  const isDanger = variant === 'danger';

  const baseStyle: React.CSSProperties = {
    fontFamily: 'var(--font-ui)',
    fontSize: 'var(--text-sm)',
    padding: 'var(--space-2) var(--space-4)',
    borderRadius: 'var(--radius-sm)',
    border: '1px solid transparent',
    cursor: disabled || loading ? 'not-allowed' : 'pointer',
    opacity: disabled || loading ? 0.6 : 1,
    backgroundColor: isPrimary
      ? 'var(--color-accent)'
      : isDanger
      ? 'var(--color-danger)'
      : 'var(--color-surface-raised)',
    color: isPrimary
      ? 'var(--color-accent-text)'
      : isDanger
      ? 'var(--color-danger-text)'
      : 'var(--color-text)',
    borderColor: isPrimary || isDanger ? 'transparent' : 'var(--color-border)',
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 'var(--space-2)',
    outline: 'none',
    transition: 'background-color 0.15s ease, border-color 0.15s ease',
    ...style,
  };

  return (
    <button disabled={disabled || loading} style={baseStyle} {...props}>
      {loading && <span>...</span>}
      {children}
    </button>
  );
};
