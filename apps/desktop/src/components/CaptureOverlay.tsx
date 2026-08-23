import React, { useCallback, useEffect, useState } from 'react';
import { captureScreenRegion, dismissOverlay } from '../services/capture';

export interface CaptureOverlayProps {
  onCaptureComplete?: (result: {
    image_path: string;
    image_width: number;
    image_height: number;
    region: string;
  }) => void;
  onDismiss?: () => void;
}

interface DragState {
  startX: number;
  startY: number;
  currentX: number;
  currentY: number;
  isDragging: boolean;
}

export const CaptureOverlay: React.FC<CaptureOverlayProps> = ({
  onCaptureComplete,
  onDismiss,
}) => {
  const [drag, setDrag] = useState<DragState>({
    startX: 0,
    startY: 0,
    currentX: 0,
    currentY: 0,
    isDragging: false,
  });
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        try {
          await dismissOverlay();
        } catch {
          // ignore
        }
        if (onDismiss) onDismiss();
      }
    },
    [onDismiss]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (e.button !== 0) return; // Left mouse button only
    setErrorMsg(null);
    setDrag({
      startX: e.clientX,
      startY: e.clientY,
      currentX: e.clientX,
      currentY: e.clientY,
      isDragging: true,
    });
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!drag.isDragging) return;
    setDrag((prev) => ({
      ...prev,
      currentX: e.clientX,
      currentY: e.clientY,
    }));
  };

  const handleMouseUp = async () => {
    if (!drag.isDragging) return;
    const x = Math.min(drag.startX, drag.currentX);
    const y = Math.min(drag.startY, drag.currentY);
    const width = Math.abs(drag.currentX - drag.startX);
    const height = Math.abs(drag.currentY - drag.startY);

    setDrag((prev) => ({ ...prev, isDragging: false }));

    // BR-31: Refuse regions smaller than 8x8 pixels
    if (width < 8 || height < 8) {
      setErrorMsg('Region must be at least 8x8 pixels');
      return;
    }

    try {
      const res = await captureScreenRegion({
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
      });
      if (onCaptureComplete) {
        onCaptureComplete(res);
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMsg(msg);
    }
  };

  const boxX = Math.min(drag.startX, drag.currentX);
  const boxY = Math.min(drag.startY, drag.currentY);
  const boxWidth = Math.abs(drag.currentX - drag.startX);
  const boxHeight = Math.abs(drag.currentY - drag.startY);

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
        backgroundColor: 'rgba(0, 0, 0, 0.4)',
        cursor: 'crosshair',
        zIndex: 9999,
        userSelect: 'none',
      }}
    >
      {drag.isDragging && boxWidth > 0 && boxHeight > 0 && (
        <div
          data-testid="selection-box"
          style={{
            position: 'absolute',
            left: `${boxX}px`,
            top: `${boxY}px`,
            width: `${boxWidth}px`,
            height: `${boxHeight}px`,
            border: '2px solid var(--color-primary, #3b82f6)',
            backgroundColor: 'rgba(59, 130, 246, 0.15)',
            pointerEvents: 'none',
          }}
        >
          <span
            data-testid="dimensions-readout"
            style={{
              position: 'absolute',
              bottom: '-24px',
              left: '0px',
              backgroundColor: '#1e293b',
              color: '#f8fafc',
              padding: '2px 6px',
              fontSize: '12px',
              borderRadius: '4px',
              whiteSpace: 'nowrap',
            }}
          >
            {boxWidth} × {boxHeight} px
          </span>
        </div>
      )}

      {errorMsg && (
        <div
          data-testid="capture-error-toast"
          style={{
            position: 'absolute',
            top: '20px',
            left: '50%',
            transform: 'translateX(-50%)',
            backgroundColor: '#ef4444',
            color: '#ffffff',
            padding: '8px 16px',
            borderRadius: '6px',
            fontSize: '14px',
          }}
        >
          {errorMsg}
        </div>
      )}
    </div>
  );
};
