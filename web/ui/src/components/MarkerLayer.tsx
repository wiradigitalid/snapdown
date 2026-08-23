import React, { useRef, useState } from 'react';
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
  onAddMarker?: (x: number, y: number) => void;
  onUpdateMarkerPosition?: (markerId: string, x: number, y: number) => void;
  onSelectMarker?: (markerId: string) => void;
}

export const MarkerLayer: React.FC<MarkerLayerProps> = ({
  markers,
  selectedMarkerId,
  onAddMarker,
  onUpdateMarkerPosition,
  onSelectMarker,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [draggingMarkerId, setDraggingMarkerId] = useState<string | null>(null);

  const handleContainerClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (draggingMarkerId) return;
    if (!containerRef.current || !onAddMarker) return;

    // Check if clicked directly on container background
    if ((e.target as HTMLElement).getAttribute('data-marker-badge') === 'true') {
      return;
    }

    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.min(Math.max((e.clientX - rect.left) / rect.width, 0.0), 1.0);
    const y = Math.min(Math.max((e.clientY - rect.top) / rect.height, 0.0), 1.0);

    onAddMarker(x, y);
  };

  const handleMouseDownBadge = (e: React.MouseEvent, markerId: string) => {
    e.stopPropagation();
    setDraggingMarkerId(markerId);
    if (onSelectMarker) {
      onSelectMarker(markerId);
    }
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!draggingMarkerId || !containerRef.current || !onUpdateMarkerPosition) return;
    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.min(Math.max((e.clientX - rect.left) / rect.width, 0.0), 1.0);
    const y = Math.min(Math.max((e.clientY - rect.top) / rect.height, 0.0), 1.0);
    onUpdateMarkerPosition(draggingMarkerId, x, y);
  };

  const handleMouseUp = () => {
    setDraggingMarkerId(null);
  };

  return (
    <div
      ref={containerRef}
      data-testid="marker-layer"
      onClick={handleContainerClick}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      style={{
        position: 'relative',
        width: '100%',
        height: '100%',
        minHeight: '300px',
        backgroundColor: '#1e293b',
        cursor: 'crosshair',
        overflow: 'hidden',
        userSelect: 'none',
      }}
    >
      {markers.map((marker) => {
        const isSelected = marker.id === selectedMarkerId;
        const isDragging = marker.id === draggingMarkerId;

        return (
          <div
            key={marker.id}
            data-testid={`marker-badge-${marker.ordinal}`}
            data-marker-badge="true"
            onMouseDown={(e) => handleMouseDownBadge(e, marker.id)}
            style={{
              position: 'absolute',
              left: `${marker.x * 100}%`,
              top: `${marker.y * 100}%`,
              transform: 'translate(-50%, -50%)',
              cursor: isDragging ? 'grabbing' : 'grab',
              zIndex: isSelected ? 10 : 1,
            }}
          >
            <MarkerBadge number={marker.ordinal} isDragging={isDragging} />
          </div>
        );
      })}
    </div>
  );
};
