import React, { useEffect, useRef } from 'react';

export interface CaptureNoteFieldProps {
  region: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  value: string;
  onChange: (value: string) => void;
  onSave: () => void;
  onCancel: () => void;
  disabled?: boolean;
}

const FIELD_BLOCK_H = 110;
const GAP = 32;

export const CaptureNoteField: React.FC<CaptureNoteFieldProps> = ({
  region,
  value,
  onChange,
  onSave,
  onCancel,
  disabled = false,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const isFullscreen =
    typeof window !== 'undefined' &&
    region.x === 0 &&
    region.y === 0 &&
    region.width >= window.innerWidth - 10 &&
    region.height >= window.innerHeight - 10;

  const shouldFlip =
    !isFullscreen &&
    typeof window !== 'undefined' &&
    region.y + region.height + FIELD_BLOCK_H + GAP > window.innerHeight;

  const anchor = isFullscreen ? 'center-bottom' : shouldFlip ? 'above' : 'below';

  const top = isFullscreen
    ? (typeof window !== 'undefined' ? window.innerHeight - FIELD_BLOCK_H - 48 : 800)
    : shouldFlip
    ? Math.max(16, region.y - FIELD_BLOCK_H - 8)
    : region.y + region.height + GAP;

  const left = isFullscreen
    ? (typeof window !== 'undefined' ? Math.max(16, (window.innerWidth - 380) / 2) : 100)
    : Math.min(
        typeof window !== 'undefined' ? window.innerWidth - 340 : region.x,
        Math.max(16, region.x)
      );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter') {
      if (!e.shiftKey) {
        e.preventDefault();
        e.stopPropagation();
        onSave();
      } else {
        e.stopPropagation();
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onCancel();
    }
  };

  return (
    <div
      data-testid="capture-note-container"
      data-anchor={anchor}
      onMouseDown={(e) => e.stopPropagation()}
      onMouseUp={(e) => e.stopPropagation()}
      onClick={(e) => e.stopPropagation()}
      style={{
        position: 'absolute',
        top: `${top}px`,
        left: `${left}px`,
        width: '320px',
        maxWidth: 'calc(100vw - 32px)',
        backgroundColor: 'var(--color-surface-raised)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
        boxShadow: 'var(--shadow-raised)',
        padding: 'var(--space-3)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-1)',
        zIndex: 20000,
        boxSizing: 'border-box',
      }}
    >
      <textarea
        ref={textareaRef}
        autoFocus
        disabled={disabled}
        data-testid="capture-note-field"
        placeholder="What is wrong here? (Optional — press Enter to save)"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        className="text-area-input"
        style={{
          width: '100%',
          minHeight: '60px',
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-sm)',
          color: 'var(--color-text)',
          backgroundColor: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: 'var(--radius-sm)',
          padding: 'var(--space-2) var(--space-3)',
          resize: 'vertical',
          boxSizing: 'border-box',
          outline: 'none',
        }}
      />
      <div
        data-testid="capture-note-hint"
        style={{
          fontFamily: 'var(--font-ui)',
          fontSize: 'var(--text-xs)',
          color: 'var(--color-text-muted)',
          userSelect: 'none',
        }}
      >
        Enter to save · Esc to cancel
      </div>
    </div>
  );
};