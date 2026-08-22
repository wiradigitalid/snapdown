import React, { InputHTMLAttributes, useEffect, useRef } from 'react';

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label?: string;
  indeterminate?: boolean;
}

export const Checkbox: React.FC<CheckboxProps> = ({
  label,
  indeterminate = false,
  checked,
  disabled,
  className = '',
  style,
  ...props
}) => {
  const checkboxRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (checkboxRef.current) {
      checkboxRef.current.indeterminate = indeterminate;
    }
  }, [indeterminate]);

  const inputClass = `checkbox-input ${className}`.trim();

  return (
    <label
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: 'var(--space-2)',
        cursor: disabled ? 'not-allowed' : 'pointer',
        fontFamily: 'var(--font-ui)',
        fontSize: 'var(--text-sm)',
        color: 'var(--color-text)',
        opacity: disabled ? 0.6 : 1,
        userSelect: 'none',
      }}
    >
      <input
        ref={checkboxRef}
        type="checkbox"
        checked={checked}
        disabled={disabled}
        className={inputClass}
        style={style}
        {...props}
      />
      {label && <span>{label}</span>}
    </label>
  );
};
