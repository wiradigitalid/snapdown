import React, { useCallback, useEffect, useState } from 'react';
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

export const CaptureOverlay: React.FC<CaptureOverlayProps> = ({
  onCaptureComplete,
  onDismiss,
}) => {
  const [phase, setPhase] = useState<Phase>('armed');
  const [note, setNote] = useState<string>('');
  const [pendingRegion, setPendingRegion] = useState<RegionRect | null>(null);
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
    if (phase === 'narrating' || phase === 'saving') return;
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
    if (!drag.isDragging) return;
    setDrag((prev) => ({
      ...prev,
      currentX: e.clientX,
      currentY: e.clientY,
    }));
  };

  const handleMouseUp = () => {
    if (!drag.isDragging) return;
    const x = Math.min(drag.startX, drag.currentX);
    const y = Math.min(drag.startY, drag.currentY);
    const width = Math.abs(drag.currentX - drag.startX);
    const height = Math.abs(drag.currentY - drag.startY);

    setDrag((prev) => ({ ...prev, isDragging: false }));

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
            }}
          >
            {currentBox.width} × {currentBox.height} px
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
