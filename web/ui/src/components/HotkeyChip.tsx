import React, { useState } from 'react';

export type HotkeyChipState = 'bound' | 'listening' | 'unbound' | 'conflicted';

export interface HotkeyChipProps {
  id?: string;
  shortcut: string;
  state?: HotkeyChipState;
  disabled?: boolean;
  onRecord?: (shortcut: string) => void;
  onCancel?: () => void;
  'aria-label'?: string;
  className?: string;
  style?: React.CSSProperties;
}

const formatKeyCombination = (e: React.KeyboardEvent): string | null => {
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    return null; // Ignore pure modifier press
  }

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) {
    parts.push('CommandOrControl');
  }
  if (e.altKey) {
    parts.push('Alt');
  }
  if (e.shiftKey) {
    parts.push('Shift');
  }

  let key = e.key.toUpperCase();
  if (key === ' ') key = 'SPACE';
  if (key === 'ESCAPE') return null;

  parts.push(key);
  return parts.join('+');
};

export const HotkeyChip: React.FC<HotkeyChipProps> = ({
  id,
  shortcut,
  state: controlledState,
  disabled = false,
  onRecord,
  onCancel,
  'aria-label': ariaLabel,
  className = '',
  style,
}) => {
  const [internalListening, setInternalListening] = useState(false);

  const isListening = controlledState === 'listening' || internalListening;
  const computedState: HotkeyChipState = isListening
    ? 'listening'
    : controlledState || (shortcut ? 'bound' : 'unbound');

  const handleClick = () => {
    if (disabled) return;
    setInternalListening(true);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (disabled || !isListening) return;

    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      setInternalListening(false);
      if (onCancel) onCancel();
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    const combo = formatKeyCombination(e);
    if (combo) {
      setInternalListening(false);
      if (onRecord) {
        onRecord(combo);
      }
    }
  };

  const handleBlur = () => {
    if (isListening) {
      setInternalListening(false);
      if (onCancel) onCancel();
    }
  };

  let displayText = shortcut || 'Click to set';
  if (isListening) {
    displayText = 'Press keys… Esc to cancel';
  } else if (computedState === 'unbound') {
    displayText = 'Click to set';
  }

  return (
    <div
      id={id}
      tabIndex={disabled ? -1 : 0}
      role="button"
      aria-label={ariaLabel || `Hotkey ${shortcut || 'unbound'}`}
      aria-disabled={disabled}
      data-state={computedState}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      className={`hotkey-chip ${className}`.trim()}
      style={style}
    >
      {displayText}
    </div>
  );
};