import React, { useRef, useState, useEffect, useCallback } from 'react';
import { MarkerBadge } from './MarkerBadge';
import { AnnotationType, VisualAnnotationItem, VisualCalloutAnnotation, VisualTextAnnotation } from './types/annotation';

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
  visualAnnotations?: VisualAnnotationItem[];
  activeTool?: AnnotationType;
  selectedMarkerId?: string | null;
  hoveredMarkerId?: string | null;
  selectedAnnotationId?: string | null;
  onAddMarker?: (x: number, y: number) => void;
  onUpdateMarkerPosition?: (markerId: string, x: number, y: number) => void;
  onSelectMarker?: (markerId: string | null) => void;
  onHoverMarker?: (markerId: string | null) => void;
  onDeleteMarker?: (markerId: string) => void;
  onMarkerContextMenu?: (marker: MarkerItem, e: React.MouseEvent) => void;
  onAddAnnotation?: (annotation: VisualAnnotationItem) => void;
  onUpdateAnnotation?: (annotation: VisualAnnotationItem) => void;
  onSelectAnnotation?: (id: string | null) => void;
  onDeleteAnnotation?: (id: string) => void;
  disabled?: boolean;
  style?: React.CSSProperties;
  className?: string;
}

export const MarkerLayer: React.FC<MarkerLayerProps> = ({
  markers,
  visualAnnotations = [],
  activeTool = 'marker',
  selectedMarkerId,
  hoveredMarkerId,
  selectedAnnotationId,
  onAddMarker,
  onUpdateMarkerPosition,
  onSelectMarker,
  onHoverMarker,
  onDeleteMarker,
  onMarkerContextMenu,
  onAddAnnotation,
  onUpdateAnnotation,
  onSelectAnnotation,
  onDeleteAnnotation,
  disabled = false,
  style,
  className = '',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [draggingMarkerId, setDraggingMarkerId] = useState<string | null>(null);

  // Drawing state for new visual elements (Shape, Blur, Arrow, Callout, Text)
  const [drawingStart, setDrawingStart] = useState<{ x: number; y: number } | null>(null);
  const [currentDraw, setCurrentDraw] = useState<{ x: number; y: number } | null>(null);

  // Dragging/Transforming existing visual elements
  const [dragMode, setDragMode] = useState<{
    id: string;
    type: 'move' | 'handle' | 'arrow-start' | 'arrow-end' | 'tail';
    handleIndex?: number;
    startX: number;
    startY: number;
    initialItem: VisualAnnotationItem;
  } | null>(null);

  // Inline editing state for Callout or Text
  const [editingTextId, setEditingTextId] = useState<string | null>(null);

  const calculateNormalizedCoords = useCallback((clientX: number, clientY: number) => {
    if (!containerRef.current) return { x: 0, y: 0 };
    const rect = containerRef.current.getBoundingClientRect();
    const width = rect.width || 1;
    const height = rect.height || 1;
    const x = Math.min(Math.max((clientX - rect.left) / width, 0.0), 1.0);
    const y = Math.min(Math.max((clientY - rect.top) / height, 0.0), 1.0);
    return { x, y };
  }, []);

  // Handle pointer interactions on canvas
  const handleContainerMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    if (disabled || e.button !== 0 || !containerRef.current) return;

    const target = e.target as HTMLElement;
    if (
      target.closest('[data-marker-badge="true"]') ||
      target.closest('[data-annotation-handle="true"]') ||
      target.closest('[data-annotation-item="true"]') ||
      target.closest('[data-annotation-floating-bar="true"]')
    ) {
      return;
    }

    const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);

    // Deselect active items when clicking canvas
    onSelectMarker?.(null);
    onSelectAnnotation?.(null);
    setEditingTextId(null);

    if (activeTool === 'marker') {
      onAddMarker?.(x, y);
    } else if (activeTool === 'text') {
      setDrawingStart({ x, y });
      setCurrentDraw({ x, y });
    } else {
      // Start drag to create Shape, Blur, Arrow, or Callout
      setDrawingStart({ x, y });
      setCurrentDraw({ x, y });
    }
  };

  const handleMouseDownBadge = (e: React.MouseEvent, markerId: string) => {
    if (disabled) return;
    e.stopPropagation();
    setDraggingMarkerId(markerId);
    onSelectMarker?.(markerId);
    onSelectAnnotation?.(null);
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

  // Keyboard navigation & deletion for selected annotations
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (disabled || editingTextId) return;
      if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedAnnotationId && onDeleteAnnotation) {
          e.preventDefault();
          onDeleteAnnotation(selectedAnnotationId);
          onSelectAnnotation?.(null);
        }
      } else if (e.key === 'Escape') {
        onSelectMarker?.(null);
        onSelectAnnotation?.(null);
        setEditingTextId(null);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [disabled, editingTextId, selectedAnnotationId, onDeleteAnnotation, onSelectAnnotation, onSelectMarker]);

  // Global mousemove & mouseup for dragging and drawing
  useEffect(() => {
    const handleGlobalMouseMove = (e: MouseEvent) => {
      const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);

      if (draggingMarkerId && onUpdateMarkerPosition) {
        onUpdateMarkerPosition(draggingMarkerId, x, y);
      } else if (drawingStart) {
        setCurrentDraw({ x, y });
      } else if (dragMode && onUpdateAnnotation) {
        const item = dragMode.initialItem;
        const dx = x - dragMode.startX;
        const dy = y - dragMode.startY;

        if (dragMode.type === 'move') {
          if (item.kind === 'arrow') {
            onUpdateAnnotation({
              ...item,
              startX: Math.min(Math.max(item.startX + dx, 0), 1),
              startY: Math.min(Math.max(item.startY + dy, 0), 1),
              endX: Math.min(Math.max(item.endX + dx, 0), 1),
              endY: Math.min(Math.max(item.endY + dy, 0), 1),
            });
          } else if (item.kind === 'callout') {
            onUpdateAnnotation({
              ...item,
              x: Math.min(Math.max(item.x + dx, 0), 1),
              y: Math.min(Math.max(item.y + dy, 0), 1),
              tailX: Math.min(Math.max(item.tailX + dx, 0), 1),
              tailY: Math.min(Math.max(item.tailY + dy, 0), 1),
            });
          } else {
            onUpdateAnnotation({
              ...item,
              x: Math.min(Math.max(item.x + dx, 0), 1),
              y: Math.min(Math.max(item.y + dy, 0), 1),
            });
          }
        } else if (dragMode.type === 'arrow-start' && item.kind === 'arrow') {
          onUpdateAnnotation({
            ...item,
            startX: x,
            startY: y,
          });
        } else if (dragMode.type === 'arrow-end' && item.kind === 'arrow') {
          onUpdateAnnotation({
            ...item,
            endX: x,
            endY: y,
          });
        } else if (dragMode.type === 'tail' && item.kind === 'callout') {
          onUpdateAnnotation({
            ...item,
            tailX: x,
            tailY: y,
          });
        } else if (dragMode.type === 'handle' && item.kind !== 'arrow') {
          // 8-point resize box
          const idx = dragMode.handleIndex ?? 0;
          let newX = item.x;
          let newY = item.y;
          let newW = item.width || 0.1;
          let newH = item.height || 0.1;

          if (idx === 0) { // Top-left
            newX = Math.min(item.x + dx, item.x + newW - 0.02);
            newY = Math.min(item.y + dy, item.y + newH - 0.02);
            newW = newW - (newX - item.x);
            newH = newH - (newY - item.y);
          } else if (idx === 1) { // Top-center
            newY = Math.min(item.y + dy, item.y + newH - 0.02);
            newH = newH - (newY - item.y);
          } else if (idx === 2) { // Top-right
            newY = Math.min(item.y + dy, item.y + newH - 0.02);
            newW = Math.max(newW + dx, 0.02);
            newH = newH - (newY - item.y);
          } else if (idx === 3) { // Mid-right
            newW = Math.max(newW + dx, 0.02);
          } else if (idx === 4) { // Bottom-right
            newW = Math.max(newW + dx, 0.02);
            newH = Math.max(newH + dy, 0.02);
          } else if (idx === 5) { // Bottom-center
            newH = Math.max(newH + dy, 0.02);
          } else if (idx === 6) { // Bottom-left
            newX = Math.min(item.x + dx, item.x + newW - 0.02);
            newW = newW - (newX - item.x);
            newH = Math.max(newH + dy, 0.02);
          } else if (idx === 7) { // Mid-left
            newX = Math.min(item.x + dx, item.x + newW - 0.02);
            newW = newW - (newX - item.x);
          }

          onUpdateAnnotation({
            ...item,
            x: Math.max(0, newX),
            y: Math.max(0, newY),
            width: Math.min(newW, 1.0 - newX),
            height: Math.min(newH, 1.0 - newY),
          });
        }
      }
    };

    const handleGlobalMouseUp = (e: MouseEvent) => {
      if (draggingMarkerId) {
        setDraggingMarkerId(null);
      }

      if (drawingStart && currentDraw && onAddAnnotation) {
        const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
        const dx = Math.abs(x - drawingStart.x);
        const dy = Math.abs(y - drawingStart.y);
        const dragDist = Math.hypot(x - drawingStart.x, y - drawingStart.y);

        // Require drag gesture (minimum movement) for shape, blur, arrow, callout, and text
        // Plain clicks without dragging should not create annotations (only marker tool creates on single click)
        const MIN_DRAG_THRESHOLD = 0.015;

        if (dragDist >= MIN_DRAG_THRESHOLD || dx >= MIN_DRAG_THRESHOLD || dy >= MIN_DRAG_THRESHOLD) {
          const minX = Math.min(drawingStart.x, x);
          const minY = Math.min(drawingStart.y, y);
          const width = Math.max(dx, 0.03);
          const height = Math.max(dy, 0.03);

          if (activeTool === 'shape') {
            const item: VisualAnnotationItem = {
              id: `shape-${Date.now()}`,
              kind: 'shape',
              x: minX,
              y: minY,
              width,
              height,
              strokeColor: 'var(--color-annotation-stroke)',
              strokeWidth: 3,
            };
            onAddAnnotation(item);
            onSelectAnnotation?.(item.id);
          } else if (activeTool === 'blur') {
            const item: VisualAnnotationItem = {
              id: `blur-${Date.now()}`,
              kind: 'blur',
              x: minX,
              y: minY,
              width,
              height,
              blurRadius: 10,
            };
            onAddAnnotation(item);
            onSelectAnnotation?.(item.id);
          } else if (activeTool === 'arrow') {
            const item: VisualAnnotationItem = {
              id: `arrow-${Date.now()}`,
              kind: 'arrow',
              startX: drawingStart.x,
              startY: drawingStart.y,
              endX: x,
              endY: y,
              color: 'var(--color-annotation-stroke)',
              strokeWidth: 4,
            };
            onAddAnnotation(item);
            onSelectAnnotation?.(item.id);
          } else if (activeTool === 'callout') {
            const boxW = Math.max(width, 0.16);
            const boxH = Math.max(height, 0.08);
            // Tail starts pointing to bottom right of the box
            const tailX = Math.min(minX + boxW + 0.06, 0.98);
            const tailY = Math.min(minY + boxH + 0.06, 0.98);

            const item: VisualAnnotationItem = {
              id: `callout-${Date.now()}`,
              kind: 'callout',
              x: minX,
              y: minY,
              width: boxW,
              height: boxH,
              tailX,
              tailY,
              text: 'Callout note...',
              fontSize: 14,
              fontFamily: 'Inter, sans-serif',
              fontWeight: '600',
              fontStyle: 'normal',
              bgColor: 'var(--color-annotation-callout-bg)',
              textColor: 'var(--color-annotation-callout-text)',
            };
            onAddAnnotation(item);
            onSelectAnnotation?.(item.id);
            setEditingTextId(item.id);
          } else if (activeTool === 'text') {
            const item: VisualAnnotationItem = {
              id: `text-${Date.now()}`,
              kind: 'text',
              x: minX,
              y: minY,
              width: Math.max(width, 0.18),
              height: Math.max(height, 0.06),
              text: 'Text comment...',
              fontSize: 16,
              fontFamily: 'Inter, sans-serif',
              fontWeight: '700',
              fontStyle: 'normal',
              textColor: 'var(--color-annotation-stroke)',
            };
            onAddAnnotation(item);
            onSelectAnnotation?.(item.id);
            setEditingTextId(item.id);
          }
        }
      }

      setDrawingStart(null);
      setCurrentDraw(null);
      setDragMode(null);
    };

    window.addEventListener('mousemove', handleGlobalMouseMove);
    window.addEventListener('mouseup', handleGlobalMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleGlobalMouseMove);
      window.removeEventListener('mouseup', handleGlobalMouseUp);
    };
  }, [
    draggingMarkerId,
    drawingStart,
    currentDraw,
    dragMode,
    activeTool,
    calculateNormalizedCoords,
    onUpdateMarkerPosition,
    onAddAnnotation,
    onUpdateAnnotation,
    onSelectAnnotation,
  ]);

  const selectedAnnotation = visualAnnotations.find((a) => a.id === selectedAnnotationId);

  return (
    <div
      ref={containerRef}
      data-testid="marker-layer"
      onMouseDown={handleContainerMouseDown}
      onClick={(e) => {
        // Support click for synthetic environments if onMouseDown didn't already trigger onAddMarker
        if (disabled || !containerRef.current) return;
        const target = e.target as HTMLElement;
        if (
          target.closest('[data-marker-badge="true"]') ||
          target.closest('[data-annotation-handle="true"]') ||
          target.closest('[data-annotation-item="true"]') ||
          target.closest('[data-annotation-floating-bar="true"]')
        ) {
          return;
        }
        if (activeTool === 'marker') {
          const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
          onAddMarker?.(x, y);
        }
      }}
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
      {/* SVG Canvas for Arrows, Callout tails, and preview vectors */}
      <svg
        style={{
          position: 'absolute',
          inset: 0,
          width: '100%',
          height: '100%',
          pointerEvents: 'none',
          zIndex: 4,
        }}
      >
        <defs>
          <marker
            id="arrowhead-red"
            markerWidth="8"
            markerHeight="8"
            refX="6"
            refY="4"
            orient="auto"
          >
            <polygon points="0 0, 8 4, 0 8" fill="var(--color-annotation-stroke)" />
          </marker>
        </defs>

        {/* Render Callout Tails as solid connected triangular polygons */}
        {visualAnnotations
          .filter((a): a is VisualAnnotationItem & { kind: 'callout' } => a.kind === 'callout')
          .map((callout) => {
            const tailPath = computeCalloutTailPolygon(callout);
            return (
              <polygon
                key={`tail-${callout.id}`}
                points={tailPath}
                fill="var(--color-annotation-callout-bg, var(--color-marker))"
                stroke="var(--color-annotation-callout-bg, var(--color-marker))"
                strokeWidth="1"
              />
            );
          })}

        {/* Render Arrows */}
        {visualAnnotations
          .filter((a): a is VisualAnnotationItem & { kind: 'arrow' } => a.kind === 'arrow')
          .map((arrow) => {
            const isSelected = arrow.id === selectedAnnotationId;
            return (
              <g key={arrow.id}>
                <line
                  x1={`${arrow.startX * 100}%`}
                  y1={`${arrow.startY * 100}%`}
                  x2={`${arrow.endX * 100}%`}
                  y2={`${arrow.endY * 100}%`}
                  stroke={arrow.color || 'var(--color-annotation-stroke)'}
                  strokeWidth={arrow.strokeWidth || 4}
                  markerEnd="url(#arrowhead-red)"
                  style={{
                    cursor: 'pointer',
                    pointerEvents: 'stroke',
                  }}
                  onMouseDown={(e) => {
                    e.stopPropagation();
                    onSelectAnnotation?.(arrow.id);
                    onSelectMarker?.(null);
                    setDragMode({
                      id: arrow.id,
                      type: 'move',
                      startX: arrow.startX,
                      startY: arrow.startY,
                      initialItem: arrow,
                    });
                  }}
                />
                {isSelected && (
                  <>
                    <circle
                      data-annotation-handle="true"
                      cx={`${arrow.startX * 100}%`}
                      cy={`${arrow.startY * 100}%`}
                      r="6"
                      fill="var(--color-annotation-handle-bg)"
                      stroke="var(--color-annotation-stroke)"
                      strokeWidth="2"
                      style={{ cursor: 'crosshair', pointerEvents: 'auto' }}
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDragMode({
                          id: arrow.id,
                          type: 'arrow-start',
                          startX: arrow.startX,
                          startY: arrow.startY,
                          initialItem: arrow,
                        });
                      }}
                    />
                    <circle
                      data-annotation-handle="true"
                      cx={`${arrow.endX * 100}%`}
                      cy={`${arrow.endY * 100}%`}
                      r="6"
                      fill="var(--color-annotation-handle-bg)"
                      stroke="var(--color-annotation-stroke)"
                      strokeWidth="2"
                      style={{ cursor: 'crosshair', pointerEvents: 'auto' }}
                      onMouseDown={(e) => {
                        e.stopPropagation();
                        setDragMode({
                          id: arrow.id,
                          type: 'arrow-end',
                          startX: arrow.endX,
                          startY: arrow.endY,
                          initialItem: arrow,
                        });
                      }}
                    />
                  </>
                )}
              </g>
            );
          })}

        {/* Live Drawing Preview for Arrow */}
        {drawingStart && currentDraw && activeTool === 'arrow' && (
          <line
            x1={`${drawingStart.x * 100}%`}
            y1={`${drawingStart.y * 100}%`}
            x2={`${currentDraw.x * 100}%`}
            y2={`${currentDraw.y * 100}%`}
            stroke="var(--color-annotation-stroke)"
            strokeWidth="3"
            strokeDasharray="4 4"
            markerEnd="url(#arrowhead-red)"
          />
        )}
      </svg>

      {/* Render Shapes (Rectangles) */}
      {visualAnnotations
        .filter((a): a is VisualAnnotationItem & { kind: 'shape' } => a.kind === 'shape')
        .map((shape) => {
          const isSelected = shape.id === selectedAnnotationId;
          return (
            <div
              key={shape.id}
              data-annotation-item="true"
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelectAnnotation?.(shape.id);
                onSelectMarker?.(null);
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: shape.id,
                  type: 'move',
                  startX: x,
                  startY: y,
                  initialItem: shape,
                });
              }}
              style={{
                position: 'absolute',
                left: `${shape.x * 100}%`,
                top: `${shape.y * 100}%`,
                width: `${shape.width * 100}%`,
                height: `${shape.height * 100}%`,
                border: `${shape.strokeWidth || 3}px solid ${shape.strokeColor || 'var(--color-annotation-stroke)'}`,
                backgroundColor: 'transparent',
                boxSizing: 'border-box',
                cursor: 'move',
                zIndex: isSelected ? 8 : 5,
              }}
            >
              {isSelected && render8PointHandles((handleIdx, e) => {
                e.stopPropagation();
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: shape.id,
                  type: 'handle',
                  handleIndex: handleIdx,
                  startX: x,
                  startY: y,
                  initialItem: shape,
                });
              })}
            </div>
          );
        })}

      {/* Render Blur Redaction Areas */}
      {visualAnnotations
        .filter((a): a is VisualAnnotationItem & { kind: 'blur' } => a.kind === 'blur')
        .map((blur) => {
          const isSelected = blur.id === selectedAnnotationId;
          return (
            <div
              key={blur.id}
              data-annotation-item="true"
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelectAnnotation?.(blur.id);
                onSelectMarker?.(null);
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: blur.id,
                  type: 'move',
                  startX: x,
                  startY: y,
                  initialItem: blur,
                });
              }}
              style={{
                position: 'absolute',
                left: `${blur.x * 100}%`,
                top: `${blur.y * 100}%`,
                width: `${blur.width * 100}%`,
                height: `${blur.height * 100}%`,
                backdropFilter: 'blur(10px) brightness(0.9)',
                WebkitBackdropFilter: 'blur(10px) brightness(0.9)',
                border: isSelected ? '2px dashed var(--color-primary)' : '1px solid var(--color-border)',
                backgroundColor: 'var(--color-annotation-fill)',
                boxSizing: 'border-box',
                cursor: 'move',
                zIndex: isSelected ? 8 : 5,
              }}
            >
              {isSelected && render8PointHandles((handleIdx, e) => {
                e.stopPropagation();
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: blur.id,
                  type: 'handle',
                  handleIndex: handleIdx,
                  startX: x,
                  startY: y,
                  initialItem: blur,
                });
              })}
            </div>
          );
        })}

      {/* Render Callout Bubbles with Triangular Pointer Tail */}
      {visualAnnotations
        .filter((a): a is VisualAnnotationItem & { kind: 'callout' } => a.kind === 'callout')
        .map((callout) => {
          const isSelected = callout.id === selectedAnnotationId;
          const isEditing = editingTextId === callout.id;
          return (
            <div
              key={callout.id}
              data-annotation-item="true"
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelectAnnotation?.(callout.id);
                onSelectMarker?.(null);
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: callout.id,
                  type: 'move',
                  startX: x,
                  startY: y,
                  initialItem: callout,
                });
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                setEditingTextId(callout.id);
              }}
              style={{
                position: 'absolute',
                left: `${callout.x * 100}%`,
                top: `${callout.y * 100}%`,
                width: `${callout.width * 100}%`,
                minHeight: `${callout.height * 100}%`,
                backgroundColor: callout.bgColor || 'var(--color-annotation-callout-bg, var(--color-marker))',
                color: callout.textColor || 'var(--color-annotation-callout-text, var(--color-marker-text))',
                borderRadius: 'var(--radius-md, 8px)',
                padding: '8px 12px',
                fontSize: `${callout.fontSize || 14}px`,
                fontFamily: callout.fontFamily || 'var(--font-ui)',
                fontWeight: (callout.fontWeight as React.CSSProperties['fontWeight']) || '600',
                fontStyle: callout.fontStyle || 'normal',
                boxShadow: '0 4px 14px var(--color-overlay-shadow-card)',
                boxSizing: 'border-box',
                cursor: 'move',
                zIndex: isSelected ? 12 : 7,
              }}
            >
              {isEditing ? (
                <textarea
                  autoFocus
                  defaultValue={callout.text}
                  onBlur={(e) => {
                    onUpdateAnnotation?.({
                      ...callout,
                      text: e.target.value,
                    });
                    setEditingTextId(null);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Escape') {
                      setEditingTextId(null);
                    }
                  }}
                  style={{
                    width: '100%',
                    height: '100%',
                    backgroundColor: 'transparent',
                    color: 'inherit',
                    border: 'none',
                    outline: 'none',
                    resize: 'none',
                    fontSize: 'inherit',
                    fontFamily: 'inherit',
                    fontWeight: 'inherit',
                    fontStyle: 'inherit',
                  }}
                />
              ) : (
                <div style={{ wordBreak: 'break-word', whiteSpace: 'pre-wrap' }}>{callout.text}</div>
              )}

              {/* Tail Point Interactive Handle */}
              {isSelected && (
                <div
                  data-annotation-handle="true"
                  onMouseDown={(e) => {
                    e.stopPropagation();
                    const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                    setDragMode({
                      id: callout.id,
                      type: 'tail',
                      startX: x,
                      startY: y,
                      initialItem: callout,
                    });
                  }}
                  style={{
                    position: 'absolute',
                    left: `${((callout.tailX - callout.x) / callout.width) * 100}%`,
                    top: `${((callout.tailY - callout.y) / callout.height) * 100}%`,
                    width: '14px',
                    height: '14px',
                    borderRadius: '50%',
                    backgroundColor: 'var(--color-annotation-handle-bg)',
                    border: '2.5px solid var(--color-annotation-stroke)',
                    boxShadow: '0 2px 6px var(--color-overlay-shadow-card)',
                    transform: 'translate(-50%, -50%)',
                    cursor: 'crosshair',
                    zIndex: 20,
                  }}
                />
              )}

              {isSelected && render8PointHandles((handleIdx, e) => {
                e.stopPropagation();
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: callout.id,
                  type: 'handle',
                  handleIndex: handleIdx,
                  startX: x,
                  startY: y,
                  initialItem: callout,
                });
              })}
            </div>
          );
        })}

      {/* Render Floating Text with 8-Point Bounding Resize Box */}
      {visualAnnotations
        .filter((a): a is VisualAnnotationItem & { kind: 'text' } => a.kind === 'text')
        .map((txt) => {
          const isSelected = txt.id === selectedAnnotationId;
          const isEditing = editingTextId === txt.id;
          return (
            <div
              key={txt.id}
              data-annotation-item="true"
              onMouseDown={(e) => {
                e.stopPropagation();
                onSelectAnnotation?.(txt.id);
                onSelectMarker?.(null);
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: txt.id,
                  type: 'move',
                  startX: x,
                  startY: y,
                  initialItem: txt,
                });
              }}
              onDoubleClick={(e) => {
                e.stopPropagation();
                setEditingTextId(txt.id);
              }}
              style={{
                position: 'absolute',
                left: `${txt.x * 100}%`,
                top: `${txt.y * 100}%`,
                width: `${txt.width * 100}%`,
                minHeight: `${txt.height * 100}%`,
                color: txt.textColor || 'var(--color-annotation-stroke)',
                fontSize: `${txt.fontSize || 16}px`,
                fontFamily: txt.fontFamily || 'var(--font-ui)',
                fontWeight: (txt.fontWeight as React.CSSProperties['fontWeight']) || '700',
                fontStyle: txt.fontStyle || 'normal',
                border: isSelected ? '1.5px dashed var(--color-primary)' : '1px solid transparent',
                backgroundColor: isSelected ? 'var(--color-annotation-fill)' : 'transparent',
                borderRadius: 'var(--radius-xs, 4px)',
                padding: '4px 6px',
                boxSizing: 'border-box',
                cursor: 'move',
                zIndex: isSelected ? 12 : 7,
              }}
            >
              {isEditing ? (
                <textarea
                  autoFocus
                  defaultValue={txt.text}
                  onBlur={(e) => {
                    onUpdateAnnotation?.({
                      ...txt,
                      text: e.target.value,
                    });
                    setEditingTextId(null);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Escape') {
                      setEditingTextId(null);
                    }
                  }}
                  style={{
                    width: '100%',
                    height: '100%',
                    backgroundColor: 'transparent',
                    color: 'inherit',
                    border: 'none',
                    outline: 'none',
                    resize: 'none',
                    fontSize: 'inherit',
                    fontFamily: 'inherit',
                    fontWeight: 'inherit',
                    fontStyle: 'inherit',
                  }}
                />
              ) : (
                <div style={{ wordBreak: 'break-word', whiteSpace: 'pre-wrap' }}>{txt.text}</div>
              )}

              {/* 8-Point Resize Handles for Bounding Box scaling of text area */}
              {isSelected && render8PointHandles((handleIdx, e) => {
                e.stopPropagation();
                const { x, y } = calculateNormalizedCoords(e.clientX, e.clientY);
                setDragMode({
                  id: txt.id,
                  type: 'handle',
                  handleIndex: handleIdx,
                  startX: x,
                  startY: y,
                  initialItem: txt,
                });
              })}
            </div>
          );
        })}

      {/* Floating Canvas Property Bar for Active Callout/Text */}
      {selectedAnnotation && (selectedAnnotation.kind === 'callout' || selectedAnnotation.kind === 'text') && (
        <div
          data-annotation-floating-bar="true"
          style={{
            position: 'absolute',
            left: `${selectedAnnotation.x * 100}%`,
            top: `calc(${selectedAnnotation.y * 100}% - 40px)`,
            transform: 'translateY(-100%)',
            backgroundColor: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
            boxShadow: 'var(--shadow-raised)',
            borderRadius: 'var(--radius-md, 8px)',
            padding: '4px 8px',
            display: 'flex',
            alignItems: 'center',
            gap: '6px',
            zIndex: 100,
            fontSize: 'var(--text-xs)',
          }}
        >
          {/* Font Family */}
          <select
            value={selectedAnnotation.fontFamily || 'Inter, sans-serif'}
            onChange={(e) => {
              onUpdateAnnotation?.({
                ...selectedAnnotation,
                fontFamily: e.target.value,
              } as VisualCalloutAnnotation | VisualTextAnnotation);
            }}
            style={{
              padding: '2px 6px',
              borderRadius: 'var(--radius-xs)',
              border: '1px solid var(--color-border)',
              backgroundColor: 'var(--color-surface-sunken)',
              color: 'var(--color-text)',
              fontSize: '11px',
              cursor: 'pointer',
            }}
          >
            <option value="Inter, sans-serif">Inter</option>
            <option value="'JetBrains Mono', monospace">JetBrains Mono</option>
            <option value="'Playfair Display', serif">Serif</option>
            <option value="'Plus Jakarta Sans', sans-serif">Jakarta Sans</option>
            <option value="Impact, sans-serif">Impact</option>
          </select>

          {/* Size - / + */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '2px' }}>
            <button
              type="button"
              onClick={() => {
                const current = selectedAnnotation.fontSize || 14;
                onUpdateAnnotation?.({
                  ...selectedAnnotation,
                  fontSize: Math.max(current - 2, 10),
                } as VisualCalloutAnnotation | VisualTextAnnotation);
              }}
              style={{
                width: '22px',
                height: '22px',
                border: '1px solid var(--color-border)',
                backgroundColor: 'var(--color-surface-sunken)',
                color: 'var(--color-text)',
                borderRadius: 'var(--radius-xs)',
                cursor: 'pointer',
                fontWeight: 700,
              }}
            >
              -
            </button>
            <span style={{ fontSize: '11px', minWidth: '24px', textAlign: 'center', fontWeight: 600 }}>
              {selectedAnnotation.fontSize || 14}
            </span>
            <button
              type="button"
              onClick={() => {
                const current = selectedAnnotation.fontSize || 14;
                onUpdateAnnotation?.({
                  ...selectedAnnotation,
                  fontSize: Math.min(current + 2, 48),
                } as VisualCalloutAnnotation | VisualTextAnnotation);
              }}
              style={{
                width: '22px',
                height: '22px',
                border: '1px solid var(--color-border)',
                backgroundColor: 'var(--color-surface-sunken)',
                color: 'var(--color-text)',
                borderRadius: 'var(--radius-xs)',
                cursor: 'pointer',
                fontWeight: 700,
              }}
            >
              +
            </button>
          </div>

          {/* Bold */}
          <button
            type="button"
            onClick={() => {
              const isBold = selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700';
              onUpdateAnnotation?.({
                ...selectedAnnotation,
                fontWeight: isBold ? 'normal' : 'bold',
              } as VisualCalloutAnnotation | VisualTextAnnotation);
            }}
            style={{
              width: '24px',
              height: '24px',
              borderRadius: 'var(--radius-xs)',
              border: '1px solid var(--color-border)',
              backgroundColor:
                selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700'
                  ? 'var(--color-primary)'
                  : 'var(--color-surface-sunken)',
              color:
                selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700'
                  ? 'var(--color-accent-text)'
                  : 'var(--color-text)',
              fontWeight: 800,
              cursor: 'pointer',
            }}
          >
            B
          </button>

          {/* Italic */}
          <button
            type="button"
            onClick={() => {
              const isItalic = selectedAnnotation.fontStyle === 'italic';
              onUpdateAnnotation?.({
                ...selectedAnnotation,
                fontStyle: isItalic ? 'normal' : 'italic',
              } as VisualCalloutAnnotation | VisualTextAnnotation);
            }}
            style={{
              width: '24px',
              height: '24px',
              borderRadius: 'var(--radius-xs)',
              border: '1px solid var(--color-border)',
              backgroundColor:
                selectedAnnotation.fontStyle === 'italic'
                  ? 'var(--color-primary)'
                  : 'var(--color-surface-sunken)',
              color:
                selectedAnnotation.fontStyle === 'italic'
                  ? 'var(--color-accent-text)'
                  : 'var(--color-text)',
              fontStyle: 'italic',
              fontWeight: 700,
              cursor: 'pointer',
            }}
          >
            I
          </button>

          <button
            type="button"
            onClick={() => onDeleteAnnotation?.(selectedAnnotation.id)}
            style={{
              background: 'none',
              border: 'none',
              color: 'var(--color-danger)',
              cursor: 'pointer',
              fontSize: '13px',
              padding: '0 4px',
            }}
            title="Delete element"
          >
            🗑️
          </button>
        </div>
      )}

      {/* Live Drawing Preview Box (for Shape / Blur / Callout / Text) */}
      {drawingStart && currentDraw && activeTool !== 'marker' && activeTool !== 'arrow' && (
        <div
          style={{
            position: 'absolute',
            left: `${Math.min(drawingStart.x, currentDraw.x) * 100}%`,
            top: `${Math.min(drawingStart.y, currentDraw.y) * 100}%`,
            width: `${Math.abs(currentDraw.x - drawingStart.x) * 100}%`,
            height: `${Math.abs(currentDraw.y - drawingStart.y) * 100}%`,
            border: activeTool === 'blur' ? '2px dashed var(--color-annotation-handle-border)' : '2px dashed var(--color-annotation-stroke)',
            backgroundColor: activeTool === 'blur' ? 'var(--color-annotation-blur-selection)' : 'var(--color-annotation-fill)',
            pointerEvents: 'none',
            zIndex: 15,
          }}
        />
      )}

      {/* Render Markers */}
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
            onContextMenu={(e) => {
              if (onMarkerContextMenu) {
                e.preventDefault();
                e.stopPropagation();
                onMarkerContextMenu(marker, e);
              }
            }}
            style={{
              position: 'absolute',
              left: `${marker.x * 100}%`,
              top: `${marker.y * 100}%`,
              transform: 'translate(-50%, -50%)',
              cursor: isDragging ? 'grabbing' : 'grab',
              zIndex: isDragging ? 30 : isSelected ? 25 : 10,
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

// Helper: Compute Callout triangular tail polygon SVG points relative to container
function computeCalloutTailPolygon(callout: VisualCalloutAnnotation): string {
  const boxLeft = callout.x * 100;
  const boxTop = callout.y * 100;
  const boxRight = (callout.x + callout.width) * 100;
  const boxBottom = (callout.y + callout.height) * 100;
  const boxCenterX = (boxLeft + boxRight) / 2;
  const boxCenterY = (boxTop + boxBottom) / 2;

  const tipX = callout.tailX * 100;
  const tipY = callout.tailY * 100;

  // Determine closest edge to the tail tip
  const distBottom = tipY - boxBottom;
  const distTop = boxTop - tipY;
  const distRight = tipX - boxRight;
  const distLeft = boxLeft - tipX;

  const maxDist = Math.max(distBottom, distTop, distRight, distLeft);

  let b1X = boxCenterX - 3;
  let b1Y = boxBottom;
  let b2X = boxCenterX + 3;
  let b2Y = boxBottom;

  if (maxDist === distBottom) {
    b1X = boxCenterX - 3;
    b1Y = boxBottom - 1;
    b2X = boxCenterX + 3;
    b2Y = boxBottom - 1;
  } else if (maxDist === distTop) {
    b1X = boxCenterX - 3;
    b1Y = boxTop + 1;
    b2X = boxCenterX + 3;
    b2Y = boxTop + 1;
  } else if (maxDist === distRight) {
    b1X = boxRight - 1;
    b1Y = boxCenterY - 3;
    b2X = boxRight - 1;
    b2Y = boxCenterY + 3;
  } else {
    b1X = boxLeft + 1;
    b1Y = boxCenterY - 3;
    b2X = boxLeft + 1;
    b2Y = boxCenterY + 3;
  }

  return `${b1X}%,${b1Y}% ${b2X}%,${b2Y}% ${tipX}%,${tipY}%`;
}

// Helper: 8-Point Resize Handles for Bounding Boxes
function render8PointHandles(onHandleMouseDown: (idx: number, e: React.MouseEvent) => void) {
  const positions = [
    { top: 0, left: 0, cursor: 'nwse-resize' },       // 0: Top-left
    { top: 0, left: '50%', cursor: 'ns-resize' },     // 1: Top-center
    { top: 0, left: '100%', cursor: 'nesw-resize' },   // 2: Top-right
    { top: '50%', left: '100%', cursor: 'ew-resize' }, // 3: Mid-right
    { top: '100%', left: '100%', cursor: 'nwse-resize' }, // 4: Bottom-right
    { top: '100%', left: '50%', cursor: 'ns-resize' },   // 5: Bottom-center
    { top: '100%', left: 0, cursor: 'nesw-resize' },     // 6: Bottom-left
    { top: '50%', left: 0, cursor: 'ew-resize' },       // 7: Mid-left
  ];

  return positions.map((pos, idx) => (
    <div
      key={idx}
      data-annotation-handle="true"
      onMouseDown={(e) => onHandleMouseDown(idx, e)}
      style={{
        position: 'absolute',
        top: pos.top,
        left: pos.left,
        width: '8px',
        height: '8px',
        borderRadius: '2px',
        backgroundColor: 'var(--color-annotation-handle-bg)',
        border: '1.5px solid var(--color-annotation-handle-border, var(--color-primary))',
        transform: 'translate(-50%, -50%)',
        cursor: pos.cursor,
        zIndex: 10,
      }}
    />
  ));
}
