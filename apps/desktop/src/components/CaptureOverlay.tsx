import React, { useCallback, useEffect, useState, useMemo } from 'react';
import { listen } from '@tauri-apps/api/event';
import { captureScreenRegion, dismissOverlay, CaptureResultDto } from '../services/capture';
import { CaptureNoteField } from './CaptureNoteField';

export interface CaptureOverlayProps {
  onCaptureComplete?: (result: CaptureResultDto) => void;
  onDismiss?: () => void;
}

type Phase = 'armed' | 'dragging' | 'narrating' | 'saving';

interface DragState {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  isDragging: boolean;
}

interface RegionRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function formatAspectRatioTag(width: number, height: number): string {
  if (width <= 0 || height <= 0) return '';
  const ratio = width / height;

  if (Math.abs(ratio - 16 / 9) <= 0.04) return ' (16:9)';
  if (Math.abs(ratio - 4 / 3) <= 0.03) return ' (4:3)';
  if (Math.abs(ratio - 1.0) <= 0.03) return ' (1:1)';
  if (Math.abs(ratio - 21 / 9) <= 0.05) return ' (21:9)';
  return '';
}

export const CaptureOverlay: React.FC<CaptureOverlayProps> = ({
  onCaptureComplete,
  onDismiss,
}) => {
  const [phase, setPhase] = useState<Phase>('armed');
  const [note, setNote] = useState<string>('');
  const [pendingRegion, setPendingRegion] = useState<RegionRect | null>(null);
  const [hoveredContainer, setHoveredContainer] = useState<RegionRect | null>(null);
  const [mousePos, setMousePos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const [drag, setDrag] = useState<DragState>({
    startX: 0,
    startY: 0,
    currentX: 0,
    currentY: 0,
    isDragging: false,
  });
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const resetState = useCallback(() => {
    setPhase('armed');
    setNote('');
    setPendingRegion(null);
    setHoveredContainer(null);
    setErrorMsg(null);
    setDrag({
      startX: 0,
      startY: 0,
      currentX: 0,
      currentY: 0,
      isDragging: false,
    });
  }, []);

  const handleCancel = useCallback(async () => {
    try {
      await dismissOverlay();
    } catch {
      // ignore
    }
    resetState();
    if (onDismiss) {
      onDismiss();
    }
  }, [onDismiss, resetState]);

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        await handleCancel();
      }
    },
    [handleCancel]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    try {
      const promise = listen('overlay-reset', () => {
        resetState();
      });
      promise.then((fn) => {
        unlistenFn = fn;
      }).catch(() => {});
    } catch {
      // Non-Tauri fallback
    }

    return () => {
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, [resetState]);

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return; // Left mouse button only

    // If already in narrating, clicking outside starts a fresh re-selection
    if (phase === 'narrating') {
      setPhase('armed');
      setPendingRegion(null);
    }

    if (phase === 'saving') return;
    setErrorMsg(null);
    setDrag({
      startX: e.clientX,
      startY: e.clientY,
      currentX: e.clientX,
      currentY: e.clientY,
      isDragging: true,
    });
    setPhase('dragging');
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    setMousePos({ x: e.clientX, y: e.clientY });

    if (drag.isDragging) {
      setDrag((prev) => ({
        ...prev,
        currentX: e.clientX,
        currentY: e.clientY,
      }));
    }
  };

  const handleMouseUp = () => {
    if (!drag.isDragging) return;
    const x = Math.min(drag.startX, drag.currentX);
    const y = Math.min(drag.startY, drag.currentY);
    const width = Math.abs(drag.currentX - drag.startX);
    const height = Math.abs(drag.currentY - drag.startY);

    setDrag((prev) => ({ ...prev, isDragging: false }));

    // 1-Click Selection: If mouse did not drag, check if an auto-detected container was clicked
    if (width < 4 && height < 4) {
      if (hoveredContainer) {
        setPendingRegion(hoveredContainer);
        setPhase('narrating');
        return;
      }
    }

    // BR-31: Refuse regions smaller than 8x8 pixels
    if (width < 8 || height < 8) {
      setErrorMsg('Region must be at least 8x8 pixels');
      setPhase('armed');
      setPendingRegion(null);
      return;
    }

    const roundedRegion: RegionRect = {
      x: Math.round(x),
      y: Math.round(y),
      width: Math.round(width),
      height: Math.round(height),
    };
    setPendingRegion(roundedRegion);
    setPhase('narrating');
  };

  const handleFullscreenClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    const fullWidth = typeof window !== 'undefined' ? window.innerWidth : 1920;
    const fullHeight = typeof window !== 'undefined' ? window.innerHeight : 1080;
    setPendingRegion({
      x: 0,
      y: 0,
      width: fullWidth,
      height: fullHeight,
    });
    setPhase('narrating');
  };

  const handleSave = async () => {
    if (phase !== 'narrating' || !pendingRegion) return;
    setPhase('saving');
    try {
      // Account for DPI scaling on Windows (logical CSS px -> physical grab coordinates)
      const dpr = typeof window !== 'undefined' && window.devicePixelRatio ? window.devicePixelRatio : 1.0;

      const physicalRegion = {
        x: Math.round(pendingRegion.x * dpr),
        y: Math.round(pendingRegion.y * dpr),
        width: Math.round(pendingRegion.width * dpr),
        height: Math.round(pendingRegion.height * dpr),
        note,
      };

      const res = await captureScreenRegion(physicalRegion);
      resetState();
      if (onCaptureComplete) {
        onCaptureComplete(res);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMsg(msg);
      setPhase('narrating');
    }
  };

  const currentBox: RegionRect = pendingRegion ?? {
    x: Math.min(drag.startX, drag.currentX),
    y: Math.min(drag.startY, drag.currentY),
    width: Math.abs(drag.currentX - drag.startX),
    height: Math.abs(drag.currentY - drag.startY),
  };

  const showSelectionBox =
    (phase === 'dragging' || phase === 'narrating' || phase === 'saving') &&
    currentBox.width > 0 &&
    currentBox.height > 0;

  const showCrosshairGuides = phase === 'armed' || phase === 'dragging';
  const showFullscreenButton = phase === 'armed';

  // Calculate loupe collision quadrant offset
  const loupeOffset = useMemo(() => {
    const defaultOffset = { x: 28, y: 28 };
    if (typeof window !== 'undefined') {
      if (mousePos.x + 160 > window.innerWidth) {
        defaultOffset.x = -140;
      }
      if (mousePos.y + 160 > window.innerHeight) {
        defaultOffset.y = -140;
      }
    }
    return defaultOffset;
  }, [mousePos]);

  const aspectRatioTag = formatAspectRatioTag(currentBox.width, currentBox.height);

  return (
    <div
      data-testid="capture-overlay"
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        width: '100vw',
        height: '100vh',
        backgroundColor: 'var(--color-overlay-scrim)',
        cursor: phase === 'armed' || phase === 'dragging' ? 'crosshair' : 'default',
        zIndex: 9999,
        userSelect: 'none',
      }}
    >
      {/* Top-Center Fullscreen Shortcut Button (Snagit-style) */}
      {showFullscreenButton && (
        <button
          type="button"
          data-testid="overlay-fullscreen-btn"
          onClick={handleFullscreenClick}
          onMouseDown={(e) => e.stopPropagation()}
          style={{
            position: 'absolute',
            top: '12px',
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-text)',
            border: '1px solid var(--color-border-strong)',
            borderRadius: 'var(--radius-pill, 9999px)',
            padding: '4px 14px',
            fontSize: 'var(--text-xs)',
            fontWeight: 700,
            fontFamily: 'var(--font-ui)',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            cursor: 'pointer',
            boxShadow: 'var(--shadow-lg)',
            zIndex: 10002,
            transition: 'all 0.15s ease',
          }}
        >
          <span>🖥️</span> Fullscreen
        </button>
      )}

      {/* Full-Screen Precision Crosshair Guides (FR-GUIDE-1) */}
      {showCrosshairGuides && (
        <>
          {/* Vertical axis guide */}
          <div
            data-testid="crosshair-axis-vertical"
            style={{
              position: 'absolute',
              top: 0,
              bottom: 0,
              left: `${mousePos.x}px`,
              width: '1px',
              borderLeft: '1px dashed var(--color-overlay-ring)',
              opacity: 0.65,
              pointerEvents: 'none',
              zIndex: 10000,
            }}
          />
          {/* Horizontal axis guide */}
          <div
            data-testid="crosshair-axis-horizontal"
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              top: `${mousePos.y}px`,
              height: '1px',
              borderTop: '1px dashed var(--color-overlay-ring)',
              opacity: 0.65,
              pointerEvents: 'none',
              zIndex: 10000,
            }}
          />
        </>
      )}

      {/* Circular Pixel Loupe Magnifier (FR-GUIDE-2) */}
      {showCrosshairGuides && (
        <div
          data-testid="capture-loupe-magnifier"
          style={{
            position: 'absolute',
            left: `${mousePos.x + loupeOffset.x}px`,
            top: `${mousePos.y + loupeOffset.y}px`,
            width: '110px',
            height: '110px',
            borderRadius: '50%',
            backgroundColor: 'var(--color-surface)',
            border: '2.5px solid var(--color-overlay-ring)',
            boxShadow: 'var(--shadow-xl)',
            overflow: 'hidden',
            pointerEvents: 'none',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 10001,
          }}
        >
          {/* Simulated 8x pixelated grid & central reticle */}
          <div
            style={{
              width: '100%',
              height: '100%',
              background: `
                radial-gradient(circle at center, transparent 3px, rgba(0,0,0,0.03) 4px),
                linear-gradient(to right, rgba(0,0,0,0.08) 1px, transparent 1px),
                linear-gradient(to bottom, rgba(0,0,0,0.08) 1px, transparent 1px)
              `,
              backgroundSize: '100% 100%, 8px 8px, 8px 8px',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              position: 'relative',
            }}
          >
            {/* Center target crosshair */}
            <div
              style={{
                width: '10px',
                height: '10px',
                border: '1px solid var(--color-snagit-red)',
                borderRadius: '1px',
                backgroundColor: 'rgba(239, 68, 68, 0.25)',
              }}
            />

            {/* Bottom Coordinate / Live Dimension Readout Badge */}
            <div
              data-testid="loupe-dimension-badge"
              style={{
                position: 'absolute',
                bottom: '8px',
                backgroundColor: 'rgba(15, 23, 42, 0.88)',
                color: '#ffffff',
                padding: '1px 6px',
                borderRadius: 'var(--radius-xs)',
                fontSize: '9px',
                fontFamily: 'var(--font-mono)',
                fontWeight: 700,
                letterSpacing: '0.02em',
              }}
            >
              {phase === 'dragging'
                ? `${currentBox.width} × ${currentBox.height}`
                : `${mousePos.x}, ${mousePos.y}`}
            </div>
          </div>
        </div>
      )}

      {/* Suggested Auto-Detect Container Cutout Box (FR-GUIDE-4) */}
      {phase === 'armed' && hoveredContainer && (
        <div
          data-testid="auto-detect-container-box"
          style={{
            position: 'absolute',
            left: `${hoveredContainer.x}px`,
            top: `${hoveredContainer.y}px`,
            width: `${hoveredContainer.width}px`,
            height: `${hoveredContainer.height}px`,
            border: '2px dashed var(--color-overlay-ring)',
            backgroundColor: 'rgba(255, 255, 255, 0.08)',
            backdropFilter: 'brightness(1.2)',
            pointerEvents: 'none',
            zIndex: 9999,
          }}
        >
          <span
            style={{
              position: 'absolute',
              top: '-20px',
              left: '0',
              backgroundColor: 'var(--color-overlay-ring)',
              color: '#ffffff',
              padding: '1px 6px',
              borderRadius: '2px',
              fontSize: '10px',
              fontWeight: 700,
              fontFamily: 'var(--font-ui)',
            }}
          >
            Click to Capture Container
          </span>
        </div>
      )}

      {/* Active Selection Bounding Box */}
      {showSelectionBox && (
        <div
          data-testid="selection-box"
          style={{
            position: 'absolute',
            left: `${currentBox.x}px`,
            top: `${currentBox.y}px`,
            width: `${currentBox.width}px`,
            height: `${currentBox.height}px`,
            border: '2px solid var(--color-overlay-ring)',
            backgroundColor: 'var(--color-overlay-selection-bg)',
            pointerEvents: 'none',
            boxShadow: '0 0 0 9999px rgba(15, 23, 42, 0.35)',
          }}
        >
          <span
            data-testid="dimensions-readout"
            style={{
              position: 'absolute',
              bottom: '-24px',
              left: '0px',
              backgroundColor: 'var(--color-surface-sunken)',
              color: 'var(--color-text)',
              border: '1px solid var(--color-border)',
              padding: 'var(--space-0) var(--space-2)',
              fontSize: 'var(--text-xs)',
              fontFamily: 'var(--font-mono)',
              borderRadius: 'var(--radius-sm)',
              whiteSpace: 'nowrap',
              display: 'flex',
              alignItems: 'center',
              gap: '4px',
            }}
          >
            {currentBox.width} × {currentBox.height} px
            {aspectRatioTag && (
              <span style={{ color: 'var(--color-accent)', fontWeight: 700 }}>
                {aspectRatioTag}
              </span>
            )}
          </span>
        </div>
      )}

      {(phase === 'narrating' || phase === 'saving') && pendingRegion && (
        <CaptureNoteField
          region={pendingRegion}
          value={note}
          onChange={setNote}
          onSave={handleSave}
          onCancel={handleCancel}
          disabled={phase === 'saving'}
        />
      )}

      {errorMsg && (
        <div
          data-testid="capture-error-toast"
          style={{
            position: 'absolute',
            top: '20px',
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: 'var(--color-danger)',
            color: 'var(--color-danger-text)',
            padding: 'var(--space-2) var(--space-4)',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-sm)',
            fontFamily: 'var(--font-ui)',
          }}
        >
          {errorMsg}
        </div>
      )}
    </div>
  );
};

