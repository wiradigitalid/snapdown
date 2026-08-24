import React, { useState, useRef, useEffect, useCallback } from 'react';

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

type DragMode =
  | { type: 'move'; startX: number; startY: number; initialCrop: CropRect }
  | { type: 'resize'; handle: string; startX: number; startY: number; initialCrop: CropRect }
  | null;

export const CropOverlay: React.FC<CropOverlayProps> = ({
  imageWidth,
  imageHeight,
  onApplyCrop,
  onCancelCrop,
}) => {
  // Default to 80% center region
  const [crop, setCrop] = useState<CropRect>({
    x: 0.1,
    y: 0.1,
    width: 0.8,
    height: 0.8,
  });

  const overlayRef = useRef<HTMLDivElement>(null);
  const dragModeRef = useRef<DragMode>(null);

  // Clamping helper with minimum dimension (min 5% width/height or 20px)
  const clamp = (val: number, min: number, max: number) => Math.min(Math.max(val, min), max);

  const handlePointerDownMove = (e: React.PointerEvent) => {
    // Only drag with primary left button and ignore if clicking handles or buttons
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();

    dragModeRef.current = {
      type: 'move',
      startX: e.clientX,
      startY: e.clientY,
      initialCrop: { ...crop },
    };
  };

  const handlePointerDownHandle = (handle: string, e: React.PointerEvent) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();

    dragModeRef.current = {
      type: 'resize',
      handle,
      startX: e.clientX,
      startY: e.clientY,
      initialCrop: { ...crop },
    };
  };

  const handleGlobalPointerMove = useCallback((e: PointerEvent) => {
    if (!dragModeRef.current || !overlayRef.current) return;

    const rect = overlayRef.current.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) return;

    const deltaX = (e.clientX - dragModeRef.current.startX) / rect.width;
    const deltaY = (e.clientY - dragModeRef.current.startY) / rect.height;
    const init = dragModeRef.current.initialCrop;

    if (dragModeRef.current.type === 'move') {
      const maxX = 1 - init.width;
      const maxY = 1 - init.height;
      const newX = clamp(init.x + deltaX, 0, Math.max(0, maxX));
      const newY = clamp(init.y + deltaY, 0, Math.max(0, maxY));

      setCrop({
        x: newX,
        y: newY,
        width: init.width,
        height: init.height,
      });
    } else if (dragModeRef.current.type === 'resize') {
      const handle = dragModeRef.current.handle;
      const minW = Math.max(0.05, 20 / rect.width);
      const minH = Math.max(0.05, 20 / rect.height);

      let left = init.x;
      let top = init.y;
      let right = init.x + init.width;
      let bottom = init.y + init.height;

      // Horizontal resize
      if (handle.includes('w')) {
        left = clamp(init.x + deltaX, 0, right - minW);
      } else if (handle.includes('e')) {
        right = clamp(init.x + init.width + deltaX, left + minW, 1);
      }

      // Vertical resize
      if (handle.includes('n')) {
        top = clamp(init.y + deltaY, 0, bottom - minH);
      } else if (handle.includes('s')) {
        bottom = clamp(init.y + init.height + deltaY, top + minH, 1);
      }

      setCrop({
        x: left,
        y: top,
        width: Math.max(minW, right - left),
        height: Math.max(minH, bottom - top),
      });
    }
  }, []);

  const handleGlobalPointerUp = useCallback(() => {
    dragModeRef.current = null;
  }, []);

  useEffect(() => {
    window.addEventListener('pointermove', handleGlobalPointerMove);
    window.addEventListener('pointerup', handleGlobalPointerUp);
    return () => {
      window.removeEventListener('pointermove', handleGlobalPointerMove);
      window.removeEventListener('pointerup', handleGlobalPointerUp);
    };
  }, [handleGlobalPointerMove, handleGlobalPointerUp]);

  // Focus overlay automatically on mount so key events (Enter / Escape) work immediately
  useEffect(() => {
    overlayRef.current?.focus();
  }, []);

  // Keyboard handler for overlay component
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onCancelCrop();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      e.stopPropagation();
      onApplyCrop(crop);
    }
  };

  const pixelW = Math.max(1, Math.round(crop.width * imageWidth));
  const pixelH = Math.max(1, Math.round(crop.height * imageHeight));

  const isNearBottom = crop.y + crop.height > 0.82;
  const isNearTop = crop.y < 0.18;

  return (
    <div
      ref={overlayRef}
      data-testid="crop-mode-overlay"
      onKeyDown={handleKeyDown}
      style={{
        position: 'absolute',
        inset: 0,
        backgroundColor: 'transparent',
        zIndex: 50,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        userSelect: 'none',
        touchAction: 'none',
        overflow: 'visible',
      }}
      tabIndex={0}
    >
      {/* Active Crop Cutout Box */}
      <div
        data-testid="active-crop-boundary"
        onPointerDown={handlePointerDownMove}
        style={{
          position: 'absolute',
          left: `${crop.x * 100}%`,
          top: `${crop.y * 100}%`,
          width: `${crop.width * 100}%`,
          height: `${crop.height * 100}%`,
          border: '2px solid var(--color-overlay-ring)',
          boxShadow: '0 0 0 9999px var(--color-overlay-scrim)',
          cursor: 'move',
          boxSizing: 'border-box',
        }}
      >
        {/* 8 Resize Handles */}
        {(['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'] as const).map((handle) => (
          <div
            key={handle}
            data-testid={`crop-handle-${handle}`}
            onPointerDown={(e) => handlePointerDownHandle(handle, e)}
            style={{
              position: 'absolute',
              width: '12px',
              height: '12px',
              backgroundColor: 'var(--color-surface)',
              border: '2px solid var(--color-accent)',
              borderRadius: '2px',
              top: handle.includes('n') ? '-6px' : handle.includes('s') ? 'calc(100% - 6px)' : 'calc(50% - 6px)',
              left: handle.includes('w') ? '-6px' : handle.includes('e') ? 'calc(100% - 6px)' : 'calc(50% - 6px)',
              cursor: `${handle}-resize`,
              zIndex: 10,
            }}
          />
        ))}

        {/* Floating Dimension Tag & Action Controls */}
        <div
          data-testid="crop-action-toolbar"
          onPointerDown={(e) => e.stopPropagation()}
          style={{
            position: 'absolute',
            left: '50%',
            transform: 'translateX(-50%)',
            ...(isNearBottom && !isNearTop
              ? { top: '-52px' }
              : isNearBottom && isNearTop
              ? { bottom: '12px' }
              : { bottom: '-52px' }),
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--space-2)',
            backgroundColor: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            padding: 'var(--space-1) var(--space-3)',
            boxShadow: 'var(--shadow-lg)',
            whiteSpace: 'nowrap',
            zIndex: 20,
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
