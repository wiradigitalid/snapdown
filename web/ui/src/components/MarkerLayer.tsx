import React, { useRef, useState, useEffect, useCallback } from 'react';
import { MarkerBadge } from './MarkerBadge';

export interface MarkerItem {
  id: string;
  finding_id: string;
  ordinal: number;
  x: number; // 0.0 .. 1.0
  y: number; // 0.0 .. 1.0
  comment: string;
}

export interface MarkerLayerProps {
  markers: MarkerItem[];
  selectedMarkerId?: string | null;
  hoveredMarkerId?: string | null;
  onAddMarker?: (x: number, y: number) => void;
  onUpdateMarkerPosition?: (markerId: string, x: number, y: number) => void;
  onSelectMarker?: (markerId: string | null) => void;
  onHoverMarker?: (markerId: string | null) => void;
  onDeleteMarker?: (markerId: string) => void;
  disabled?: boolean;
  style?: React.CSSProperties;
  className?: string;
}

export const MarkerLayer: React.FC<MarkerLayerProps> = ({
  markers,
  selectedMarkerId,
  hoveredMarkerId,
  onAddMarker,
  onUpdateMarkerPosition,
  onSelectMarker,
  onHoverMarker,
  onDeleteMarker,
  disabled = false,
  style,
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [draggingMarkerId, setDraggingMarkerId] = useState<string | null>(null);

  const calculateNormalizedCoords = useCallback((clientX: number, clientY: number) => {
    if (!containerRef.current) return { x: 0, y: 0 };
    const rect = containerRef.current.getBoundingClientRect();
    const width = rect.width || 1;
    const height = rect.height || 1;
    const x = Math.min(Math.max((clientX - rect.left) / width, 0.0), 1.0);
    const y = Math.min(Math.max((clientY - rect.top) / height, 0.0), 1.0);
    return { x, y };
  }, []);

  const handleContainerClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (disabled || draggingMarkerId || !containerRef.current) return;

    // Check if clicked directly on marker badge element
    const target = e.target as HTMLElement;
    if (target.closest('[data-marker-badge="true"]')) {
      return;
    }

    // Clicking blank canvas deselects any active marker
    if (onSelectMarker) {
      onSelectMarker(null);
    }

    if (onAddMarker) {
      const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
      onAddMarker(x, y);
    }
  };

  const handleMouseDownBadge = (e: React.MouseEvent, markerId: string) => {
    if (disabled) return;
    e.stopPropagation();
    setDraggingMarkerId(markerId);
    if (onSelectMarker) {
      onSelectMarker(markerId);
    }
  };

  const handleBadgeKeyDown = (e: React.KeyboardEvent<HTMLDivElement>, marker: MarkerItem) => {
    if (disabled) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      e.stopPropagation();
      onSelectMarker?.(marker.id);
    } else if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      e.stopPropagation();
      onDeleteMarker?.(marker.id);
    }
  };

  useEffect(() => {
    if (!draggingMarkerId) return;

    const handleGlobalMouseMove = (e: MouseEvent) => {
      if (!draggingMarkerId || !containerRef.current || !onUpdateMarkerPosition) return;
      const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
      onUpdateMarkerPosition(draggingMarkerId, x, y);
    };

    const handleGlobalMouseUp = () => {
      setDraggingMarkerId(null);
    };

    window.addEventListener('mousemove', handleGlobalMouseMove);
    window.addEventListener('mouseup', handleGlobalMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleGlobalMouseMove);
      window.removeEventListener('mouseup', handleGlobalMouseUp);
    };
  }, [draggingMarkerId, calculateNormalizedCoords, onUpdateMarkerPosition]);

  return (
    <div
      ref={containerRef}
      data-testid="marker-layer"
      onClick={handleContainerClick}
      className={`marker-layer ${className}`.trim()}
      style={{
        position: 'absolute',
        inset: 0,
        width: '100%',
        height: '100%',
        cursor: disabled ? 'default' : 'crosshair',
        overflow: 'hidden',
        userSelect: 'none',
        pointerEvents: disabled ? 'none' : 'auto',
        ...style,
      }}
    >
      {markers.map((marker) => {
        const isSelected = marker.id === selectedMarkerId;
        const isHovered = marker.id === hoveredMarkerId;
        const isDragging = marker.id === draggingMarkerId;

        return (
          <div
            key={marker.id}
            data-testid={`marker-badge-${marker.ordinal}`}
            data-marker-id={marker.id}
            data-marker-badge="true"
            onMouseDown={(e) => handleMouseDownBadge(e, marker.id)}
            onMouseEnter={() => onHoverMarker?.(marker.id)}
            onMouseLeave={() => onHoverMarker?.(null)}
            style={{
              position: 'absolute',
              left: `${marker.x * 100}%`,
              top: `${marker.y * 100}%`,
              transform: 'translate(-50%, -50%)',
              cursor: isDragging ? 'grabbing' : 'grab',
              zIndex: isDragging ? 20 : isSelected ? 10 : 1,
            }}
          >
            <MarkerBadge
              number={marker.ordinal}
              isDragging={isDragging}
              isSelected={isSelected}
              isHovered={isHovered}
              tabIndex={disabled ? -1 : 0}
              onKeyDown={(e) => handleBadgeKeyDown(e, marker)}
            />
          </div>
        );
      })}
    </div>
  );
};
