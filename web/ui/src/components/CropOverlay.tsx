import React, { useState } from 'react';

export interface CropRect {
  x: number; // fractional 0..1
  y: number; // fractional 0..1
  width: number; // fractional 0..1
  height: number; // fractional 0..1
}

export interface CropOverlayProps {
  imageWidth: number;
  imageHeight: number;
  onApplyCrop: (crop: CropRect) => void;
  onCancelCrop: () => void;
}

export const CropOverlay: React.FC<CropOverlayProps> = ({
  imageWidth,
  imageHeight,
  onApplyCrop,
  onCancelCrop,
}) => {
  // Default to 80% center region
  const [crop] = useState<CropRect>({
    x: 0.1,
    y: 0.1,
    width: 0.8,
    height: 0.8,
  });

  const pixelW = Math.round(crop.width * imageWidth);
  const pixelH = Math.round(crop.height * imageHeight);

  return (
    <div
      data-testid="crop-mode-overlay"
      style={{
        position: 'absolute',
        inset: 0,
        backgroundColor: 'var(--color-overlay-scrim)',
        zIndex: 50,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        userSelect: 'none',
      }}
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          e.preventDefault();
          onCancelCrop();
        } else if (e.key === 'Enter') {
          e.preventDefault();
          onApplyCrop(crop);
        }
      }}
    >
      {/* Active Crop Cutout Box */}
      <div
        data-testid="active-crop-boundary"
        style={{
          position: 'absolute',
          left: `${crop.x * 100}%`,
          top: `${crop.y * 100}%`,
          width: `${crop.width * 100}%`,
          height: `${crop.height * 100}%`,
          border: '2px solid var(--color-overlay-ring)',
          boxShadow: '0 0 0 9999px var(--color-overlay-scrim)',
          cursor: 'move',
        }}
      >
        {/* 8 Resize Handles */}
        {['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'].map((handle) => (
          <div
            key={handle}
            data-testid={`crop-handle-${handle}`}
            style={{
              position: 'absolute',
              width: '10px',
              height: '10px',
              backgroundColor: 'var(--color-surface)',
              border: '2px solid var(--color-accent)',
              borderRadius: '2px',
              top: handle.includes('n') ? '-5px' : handle.includes('s') ? 'calc(100% - 5px)' : 'calc(50% - 5px)',
              left: handle.includes('w') ? '-5px' : handle.includes('e') ? 'calc(100% - 5px)' : 'calc(50% - 5px)',
              cursor: `${handle}-resize`,
            }}
          />
        ))}

        {/* Floating Dimension Tag & Action Controls */}
        <div
          style={{
            position: 'absolute',
            bottom: '-48px',
            left: '50%',
            transform: 'translateX(-50%)',
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--space-2)',
            backgroundColor: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            padding: 'var(--space-1) var(--space-3)',
            boxShadow: 'var(--shadow-lg)',
            whiteSpace: 'nowrap',
          }}
        >
          <span
            style={{
              fontSize: 'var(--text-xs)',
              fontFamily: 'var(--font-mono)',
              color: 'var(--color-text-secondary)',
              marginRight: 'var(--space-2)',
            }}
          >
            {pixelW} × {pixelH} px
          </span>
          <button
            type="button"
            data-testid="apply-crop-button"
            onClick={() => onApplyCrop(crop)}
            style={{
              padding: 'var(--space-1) var(--space-3)',
              backgroundColor: 'var(--color-accent)',
              color: 'var(--color-accent-text)',
              border: 'none',
              borderRadius: 'var(--radius-xs)',
              fontSize: 'var(--text-xs)',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            ✓ Apply Crop (Enter)
          </button>
          <button
            type="button"
            data-testid="cancel-crop-button"
            onClick={onCancelCrop}
            style={{
              padding: 'var(--space-1) var(--space-2)',
              backgroundColor: 'transparent',
              color: 'var(--color-text-muted)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-xs)',
              fontSize: 'var(--text-xs)',
              cursor: 'pointer',
            }}
          >
            ✕ Cancel (Esc)
          </button>
        </div>
      </div>
    </div>
  );
};
