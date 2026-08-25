import React from 'react';
import { AnnotationType } from './types/annotation';

export interface StudioRibbonProps {
  onCaptureClick?: () => void;
  onOpenFileClick?: () => void;
  onPasteClick?: () => void;
  activeTool?: AnnotationType;
  onSelectTool?: (tool: AnnotationType) => void;
  isMarkerActive?: boolean;
  onToggleMarker?: () => void;
  isCropActive?: boolean;
  onToggleCrop?: () => void;
  onAssembleBundle?: () => void;
  onCopyImage?: () => void;
  onShareBundle?: () => void;
  onUndo?: () => void;
  onRedo?: () => void;
  canUndo?: boolean;
  canRedo?: boolean;
  selectedFindingsCount?: number;
  className?: string;
  style?: React.CSSProperties;
}

export const StudioRibbon: React.FC<StudioRibbonProps> = ({
  onCaptureClick,
  onOpenFileClick,
  onPasteClick,
  activeTool = 'marker',
  onSelectTool,
  isMarkerActive = false,
  onToggleMarker,
  isCropActive = false,
  onToggleCrop,
  onAssembleBundle,
  onCopyImage,
  onShareBundle,
  onUndo,
  onRedo,
  canUndo = false,
  canRedo = false,
  selectedFindingsCount = 0,
  className = '',
  style,
}) => {
  const tools: { id: AnnotationType; label: string; icon: string; tooltip: string }[] = [
    { id: 'marker', label: 'Marker', icon: '🔴', tooltip: 'Numbered Marker (1)' },
    { id: 'shape', label: 'Shape', icon: '▢', tooltip: 'Outline Box (2)' },
    { id: 'callout', label: 'Callout', icon: '💬', tooltip: 'Callout Bubble (3)' },
    { id: 'blur', label: 'Blur', icon: '░', tooltip: 'Blur Redaction (4)' },
    { id: 'arrow', label: 'Arrow', icon: '↗', tooltip: 'Directional Arrow (5)' },
    { id: 'text', label: 'Text', icon: 'T', tooltip: 'Floating Text (6)' },
  ];

  return (
    <div
      data-testid="studio-ribbon"
      className={`studio-ribbon ${className}`.trim()}
      style={{
        height: '56px',
        backgroundColor: 'var(--color-surface)',
        borderBottom: '1px solid var(--color-border)',
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

      {/* CENTER ZONE: Annotation Tools Palette */}
      <div
        data-testid="ribbon-center-zone"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--space-1)',
          backgroundColor: 'var(--color-surface-sunken)',
          padding: '4px',
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--color-border)',
        }}
      >
        {/* Legacy marker button for backward-compat tests */}
        <button
          type="button"
          data-testid="ribbon-marker-btn"
          data-tooltip="Insert Step Marker (1, 2, 3...)"
          onClick={onToggleMarker}
          style={{ display: 'none' }}
        >
          🔢
        </button>

        {tools.map((t) => {
          const isActive = activeTool === t.id && !isCropActive;
          return (
            <button
              key={t.id}
              type="button"
              data-testid={`ribbon-tool-${t.id}`}
              data-tooltip={t.tooltip}
              onClick={() => {
                if (isCropActive && onToggleCrop) onToggleCrop();
                onSelectTool?.(t.id);
                if (t.id === 'marker' && onToggleMarker && !isMarkerActive) {
                  onToggleMarker();
                }
              }}
              style={{
                height: '32px',
                padding: '0 10px',
                display: 'inline-flex',
                alignItems: 'center',
                gap: '6px',
                backgroundColor: isActive ? 'var(--color-accent)' : 'transparent',
                color: isActive ? 'var(--color-accent-text)' : 'var(--color-text)',
                border: 'none',
                borderRadius: 'var(--radius-sm)',
                fontSize: 'var(--text-xs)',
                fontWeight: isActive ? 700 : 500,
                cursor: 'pointer',
                transition: 'all 0.12s ease',
              }}
            >
              <span style={{ fontSize: '13px' }}>{t.icon}</span>
              <span>{t.label}</span>
            </button>
          );
        })}

        <div style={{ width: '1px', height: '20px', backgroundColor: 'var(--color-border)', margin: '0 2px' }} />

        {/* Crop tool */}
        <button
          type="button"
          data-testid="ribbon-crop-btn"
          data-tooltip="Crop Region"
          onClick={onToggleCrop}
          style={{
            height: '32px',
            padding: '0 10px',
            display: 'inline-flex',
            alignItems: 'center',
            gap: '6px',
            backgroundColor: isCropActive ? 'var(--color-accent)' : 'transparent',
            color: isCropActive ? 'var(--color-accent-text)' : 'var(--color-text)',
            border: 'none',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-xs)',
            fontWeight: isCropActive ? 700 : 500,
            cursor: 'pointer',
          }}
        >
          <span>✂️</span>
          <span>Crop</span>
        </button>

        <div style={{ width: '1px', height: '20px', backgroundColor: 'var(--color-border)', margin: '0 2px' }} />

        {/* Undo / Redo */}
        <button
          type="button"
          data-testid="ribbon-undo-btn"
          data-tooltip="Undo (Ctrl+Z)"
          disabled={!canUndo}
          onClick={onUndo}
          style={{
            width: '32px',
            height: '32px',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'transparent',
            color: canUndo ? 'var(--color-text)' : 'var(--color-text-muted)',
            border: 'none',
            borderRadius: 'var(--radius-sm)',
            cursor: canUndo ? 'pointer' : 'default',
            opacity: canUndo ? 1 : 0.4,
          }}
        >
          ↶
        </button>
        <button
          type="button"
          data-testid="ribbon-redo-btn"
          data-tooltip="Redo (Ctrl+Y)"
          disabled={!canRedo}
          onClick={onRedo}
          style={{
            width: '32px',
            height: '32px',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'transparent',
            color: canRedo ? 'var(--color-text)' : 'var(--color-text-muted)',
            border: 'none',
            borderRadius: 'var(--radius-sm)',
            cursor: canRedo ? 'pointer' : 'default',
            opacity: canRedo ? 1 : 0.4,
          }}
        >
          ↷
        </button>
      </div>

      {/* RIGHT ZONE: Primary Output Actions */}
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
