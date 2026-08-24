import React from 'react';

export interface StudioRibbonProps {
  onCaptureClick?: () => void;
  onOpenFileClick?: () => void;
  onPasteClick?: () => void;
  onAssembleBundle?: () => void;
  onCopyImage?: () => void;
  onShareBundle?: () => void;
  selectedFindingsCount?: number;
  className?: string;
  style?: React.CSSProperties;
}

export const StudioRibbon: React.FC<StudioRibbonProps> = ({
  onCaptureClick,
  onOpenFileClick,
  onPasteClick,
  onAssembleBundle,
  onCopyImage,
  onShareBundle,
  selectedFindingsCount = 0,
  className = '',
  style,
}) => {
  return (
    <div
      data-testid="studio-ribbon"
      className={`studio-ribbon ${className}`.trim()}
      style={{
        height: '56px',
        backgroundColor: 'var(--snagit-ribbon-bg)',
        borderBottom: '1px solid var(--snagit-ribbon-border)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0 var(--space-4)',
        boxSizing: 'border-box',
        userSelect: 'none',
        flexShrink: 0,
        ...style,
      }}
    >
      {/* LEFT ZONE: Input Actions (38x38 Icon Buttons) */}
      <div
        data-testid="ribbon-left-zone"
        style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}
      >
        <button
          type="button"
          data-testid="ribbon-capture-btn"
          data-tooltip="Capture Region (Ctrl+Shift+S)"
          onClick={onCaptureClick}
          style={{
            width: '38px',
            height: '38px',
            padding: 0,
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            fontSize: '1rem',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
            transition: 'background-color 0.15s ease, border-color 0.15s ease',
          }}
        >
          🔴
        </button>

        <button
          type="button"
          data-testid="ribbon-open-btn"
          data-tooltip="Open Image File"
          onClick={onOpenFileClick}
          style={{
            width: '38px',
            height: '38px',
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            fontSize: '1rem',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
          }}
        >
          📂
        </button>

        <button
          type="button"
          data-testid="ribbon-paste-btn"
          data-tooltip="Paste from Clipboard"
          onClick={onPasteClick}
          style={{
            width: '38px',
            height: '38px',
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            fontSize: '1rem',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
          }}
        >
          📥
        </button>
      </div>

      {/* RIGHT ZONE: Export & Assembly */}
      <div
        data-testid="ribbon-right-zone"
        style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}
      >
        <button
          type="button"
          data-testid="ribbon-assemble-btn"
          data-tooltip="Review & Assemble Bundle (Ctrl+B)"
          onClick={onAssembleBundle}
          style={{
            height: '38px',
            padding: '0 var(--space-4)',
            backgroundColor: 'var(--color-accent)',
            color: 'var(--color-accent-text)',
            border: 'none',
            borderRadius: 'var(--radius-sm)',
            fontWeight: 700,
            fontSize: 'var(--text-xs)',
            display: 'inline-flex',
            alignItems: 'center',
            gap: 'var(--space-2)',
            cursor: 'pointer',
            boxShadow: 'var(--shadow-sm)',
          }}
        >
          <span>📦</span>
          <span>Assemble {selectedFindingsCount > 0 ? `(${selectedFindingsCount})` : ''}</span>
        </button>

        <button
          type="button"
          data-testid="ribbon-copy-image-btn"
          data-tooltip="Copy Burned Image to Clipboard"
          onClick={onCopyImage}
          style={{
            width: '38px',
            height: '38px',
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            fontSize: '1rem',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            cursor: 'pointer',
          }}
        >
          📋
        </button>

        <button
          type="button"
          data-testid="ribbon-share-btn"
          data-tooltip="Share Bundle URL"
          onClick={onShareBundle}
          style={{
            height: '38px',
            padding: '0 var(--space-3)',
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            fontWeight: 600,
            fontSize: 'var(--text-xs)',
            display: 'inline-flex',
            alignItems: 'center',
            gap: 'var(--space-1)',
            cursor: 'pointer',
          }}
        >
          <span>Share ▼</span>
        </button>
      </div>
    </div>
  );
};
