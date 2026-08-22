import React, { TextareaHTMLAttributes, useEffect, useRef } from 'react';

export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  autoGrow?: boolean;
  invalid?: boolean;
  errorMessage?: string;
}

export const TextArea: React.FC<TextAreaProps> = ({
  label,
  autoGrow = true,
  invalid = false,
  errorMessage,
  value,
  disabled,
  style,
  onChange,
  ...props
}) => {
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    if (autoGrow && textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [value, autoGrow]);

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
      <textarea
        ref={textareaRef}
        disabled={disabled}
        value={value}
        onChange={(e) => {
          if (autoGrow && textareaRef.current) {
            textareaRef.current.style.height = 'auto';
            textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
          }
          if (onChange) {
            onChange(e);
          }
        }}
        style={{
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-sm)',
          padding: 'var(--space-2) var(--space-3)',
          borderRadius: 'var(--radius-sm)',
          border: `1px solid ${invalid ? 'var(--color-danger)' : 'var(--color-border)'}`,
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text)',
          outline: 'none',
          resize: autoGrow ? 'none' : 'vertical',
          opacity: disabled ? 0.6 : 1,
          ...style,
        }}
        {...props}
      />
      {invalid && errorMessage && (
        <span style={{ fontSize: 'var(--text-xs)', color: 'var(--color-danger)', fontFamily: 'var(--font-ui)' }}>
          {errorMessage}
        </span>
      )}
    </div>
  );
};
