import React from 'react';

export interface ToggleProps extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onChange'> {
  id?: string;
  checked?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
  'aria-label'?: string;
  'aria-labelledby'?: string;
  className?: string;
  style?: React.CSSProperties;
  'data-testid'?: string;
}

export const Toggle: React.FC<ToggleProps> = ({
  id,
  checked = false,
  indeterminate = false,
  disabled = false,
  onChange,
  'aria-label': ariaLabel,
  'aria-labelledby': ariaLabelledBy,
  className = '',
  style,
  'data-testid': dataTestId,
  ...props
}) => {
  const state = indeterminate ? 'indeterminate' : checked ? 'on' : 'off';
  const ariaChecked = indeterminate ? 'mixed' : checked;

  const handleClick = (e: React.MouseEvent<HTMLButtonElement>) => {
    if (disabled || !onChange) return;
    onChange(!checked);
    props.onClick?.(e);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return;
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      if (onChange) {
        onChange(!checked);
      }
    }
    props.onKeyDown?.(e);
  };

  return (
    <button
      id={id}
      type="button"
      role="switch"
      aria-checked={ariaChecked}
      aria-label={ariaLabel}
      aria-labelledby={ariaLabelledBy}
      aria-disabled={disabled}
      disabled={disabled}
      data-state={state}
      data-disabled={disabled ? 'true' : 'false'}
      data-testid={dataTestId}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={`toggle-switch ${className}`.trim()}
      style={style}
    >
      <span className="toggle-thumb" aria-hidden="true" />
    </button>
  );
};
