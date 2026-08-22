import React, { InputHTMLAttributes } from 'react';

export interface TextFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  invalid?: boolean;
  errorMessage?: string;
  showCharCount?: boolean;
  maxLength?: number;
}

export const TextField: React.FC<TextFieldProps> = ({
  label,
  invalid = false,
  errorMessage,
  showCharCount = false,
  maxLength,
  value,
  disabled,
  style,
  ...props
}) => {
  const currentLength = typeof value === 'string' ? value.length : 0;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-1)', width: '100%' }}>
      {label && (
        <label
          style={{
            fontFamily: 'var(--font-ui)',
            fontSize: 'var(--text-xs)',
            color: 'var(--color-text-muted)',
          }}
        >
          {label}
        </label>
      )}
      <input
        disabled={disabled}
        value={value}
        maxLength={maxLength}
        style={{
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-sm)',
          padding: 'var(--space-2) var(--space-3)',
          borderRadius: 'var(--radius-sm)',
          border: `1px solid ${invalid ? 'var(--color-danger)' : 'var(--color-border)'}`,
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text)',
          outline: 'none',
          opacity: disabled ? 0.6 : 1,
          ...style,
        }}
        {...props}
      />
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        {invalid && errorMessage ? (
          <span style={{ fontSize: 'var(--text-xs)', color: 'var(--color-danger)', fontFamily: 'var(--font-ui)' }}>
            {errorMessage}
          </span>
        ) : <span />}
        {showCharCount && maxLength !== undefined && (
          <span style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}>
            {currentLength}/{maxLength}
          </span>
        )}
      </div>
    </div>
  );
};
