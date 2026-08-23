import React from 'react';

export interface ToggleProps {
  id?: string;
  checked?: boolean;
  indeterminate?: boolean;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
  'aria-label'?: string;
  'aria-labelledby'?: string;
  className?: string;
  style?: React.CSSProperties;
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
}) => {
  const state = indeterminate ? 'indeterminate' : checked ? 'on' : 'off';
  const ariaChecked = indeterminate ? 'mixed' : checked;

  const handleClick = () => {
    if (disabled || !onChange) return;
    onChange(!checked);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return;
    if (e.key === ' ' || e.key === 'Enter') {
      e.preventDefault();
      if (onChange) {
        onChange(!checked);
      }
    }
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
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      className={`toggle-switch ${className}`.trim()}
      style={style}
    >
      <span className="toggle-thumb" aria-hidden="true" />
    </button>
  );
};
