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
  className = '',
  style,
  ...props
}) => {
  const currentLength = typeof value === 'string' ? value.length : 0;
  const inputClass = `text-field-input ${invalid ? 'invalid' : ''} ${className}`.trim();

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
        className={inputClass}
        style={style}
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
