import React, { useEffect, useState, useMemo, useCallback } from 'react';
import { Button } from './Button';
import { TextArea } from './TextArea';
import { Checkbox } from './Checkbox';
import { Badge } from './Badge';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';
import { HotkeyChip } from './HotkeyChip';
import { ConfirmDialog } from './ConfirmDialog';
import { MarkerBadge } from './MarkerBadge';
import { MarkerLayer, MarkerItem } from './MarkerLayer';

export interface FindingItemDto {
  id: string;
  image_path: string;
  image_width: number;
  image_height: number;
  captured_at: string;
  source_monitor: string;
  region: string;
  file_size_bytes?: number | null;
}

export interface NoteItemDto {
  id: string;
  finding_id: string;
  body: string;
  updated_at: string;
}

export interface MarkerItemDto {
  id: string;
  finding_id: string;
  ordinal: number;
  x: number;
  y: number;
  comment: string;
}

export interface FindingDetailItemDto {
  finding: FindingItemDto;
  note: NoteItemDto;
  markers: MarkerItemDto[];
  imageSrc?: string;
  isImageMissing?: boolean;
}

export interface FindingsEditorProps {
  findings: FindingDetailItemDto[];
  selectedFindingId: string | null;
  isLoading?: boolean;
  error?: string | null;
  captureHotkey?: string;
  onSelectFinding: (id: string) => void;
  onSaveNote: (findingId: string, noteBody: string) => Promise<void> | void;
  onDeleteFinding?: (findingId: string) => Promise<void> | void;
  onAddMarker?: (findingId: string, x: number, y: number) => Promise<void> | void;
  onUpdateMarkerPosition?: (findingId: string, markerId: string, x: number, y: number) => Promise<void> | void;
  onDeleteMarker?: (findingId: string, markerId: string) => Promise<void> | void;
  onOpenOrphanReport?: () => void;
  onCompose?: (selectedFindingIds: string[]) => void;
  onRetry?: () => void;
}

function formatFileSize(bytes?: number | null): string {
  if (bytes === null || bytes === undefined || isNaN(bytes)) {
    return '184 KB';
  }
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatTimestamp(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch {
    return isoString;
  }
}

// Checks if note body contains a line starting with `${ordinal}.`
export function isMarkerBoundInNote(ordinal: number, noteBody: string): boolean {
  const linePattern = new RegExp(`^\\s*${ordinal}\\.`, 'm');
  return linePattern.test(noteBody);
}

export const FindingsEditor: React.FC<FindingsEditorProps> = ({
  findings,
  selectedFindingId,
  isLoading = false,
  error = null,
  captureHotkey = 'CommandOrControl+Shift+S',
  onSelectFinding,
  onSaveNote,
  onDeleteFinding,
  onAddMarker,
  onUpdateMarkerPosition,
  onDeleteMarker,
  onOpenOrphanReport,
  onCompose,
  onRetry,
}) => {
  const selectedFinding = findings.find((f) => f.finding.id === selectedFindingId);
  const [noteText, setNoteText] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [selectedMarkerId, setSelectedMarkerId] = useState<string | null>(null);
  const [hoveredMarkerId, setHoveredMarkerId] = useState<string | null>(null);
  const [checkedFindingIds, setCheckedFindingIds] = useState<Set<string>>(new Set());
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [imageLoadError, setImageLoadError] = useState(false);

  // Sync note text when selection changes
  useEffect(() => {
    if (selectedFinding) {
      setNoteText(selectedFinding.note.body);
      setSaveSuccess(false);
      setImageLoadError(false);
      setSelectedMarkerId(null);
    } else {
      setNoteText('');
    }
  }, [selectedFindingId, selectedFinding]);

  // Handle Note Auto-Save / Save button
  const handleSaveNote = useCallback(async (newText: string) => {
    if (!selectedFinding) return;
    setIsSaving(true);
    try {
      await onSaveNote(selectedFinding.finding.id, newText);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } catch (err) {
      console.error('Failed to save note:', err);
    } finally {
      setIsSaving(false);
    }
  }, [selectedFinding, onSaveNote]);

  const handleNoteChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    setNoteText(val);
  };

  const handleNoteBlur = () => {
    if (selectedFinding && noteText !== selectedFinding.note.body) {
      handleSaveNote(noteText);
    }
  };

  // Toggle multi-select checkbox for finding
  const handleToggleCheck = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    setCheckedFindingIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  // Handle Add Marker
  const handleAddMarker = (x: number, y: number) => {
    if (!selectedFinding || !onAddMarker) return;
    onAddMarker(selectedFinding.finding.id, x, y);
  };

  // Handle Update Marker Position
  const handleUpdateMarkerPosition = (markerId: string, x: number, y: number) => {
    if (!selectedFinding || !onUpdateMarkerPosition) return;
    onUpdateMarkerPosition(selectedFinding.finding.id, markerId, x, y);
  };

  // Handle Delete Marker
  const handleDeleteMarker = (markerId: string) => {
    if (!selectedFinding || !onDeleteMarker) return;
    onDeleteMarker(selectedFinding.finding.id, markerId);
  };

  // Handle Confirm Delete Finding
  const handleConfirmDelete = async () => {
    if (!selectedFinding || !onDeleteFinding) return;
    setIsDeleting(true);
    try {
      await onDeleteFinding(selectedFinding.finding.id);
      setIsDeleteDialogOpen(false);
    } finally {
      setIsDeleting(false);
    }
  };

  // Sorted findings (newest first)
  const sortedFindings = useMemo(() => {
    return [...findings].sort((a, b) => {
      const timeA = new Date(a.finding.captured_at).getTime();
      const timeB = new Date(b.finding.captured_at).getTime();
      return timeB - timeA;
    });
  }, [findings]);

  // Marker items adapted for MarkerLayer
  const markerLayerItems: MarkerItem[] = useMemo(() => {
    if (!selectedFinding) return [];
    return selectedFinding.markers.map((m) => ({
      id: m.id,
      finding_id: m.finding_id,
      ordinal: m.ordinal,
      x: m.x,
      y: m.y,
      comment: m.comment,
    }));
  }, [selectedFinding]);

  // State 1: Error State
  if (error) {
    return (
      <div
        data-testid="findings-error-state"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          width: '100%',
          padding: 'var(--space-6)',
          backgroundColor: 'var(--color-bg)',
        }}
      >
        <ErrorState
          title="The Library could not be read"
          message={error}
          actionLabel="Retry"
          onAction={onRetry}
        />
      </div>
    );
  }

  // State 2: Empty State (3 columns collapse to centered EmptyState teaching hotkey)
  if (!isLoading && findings.length === 0) {
    return (
      <div
        data-testid="findings-empty-state"
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          width: '100%',
          backgroundColor: 'var(--color-bg)',
          padding: 'var(--space-6)',
        }}
      >
        <EmptyState
          heading="No findings yet"
          description="Capture screenshots with the shortcut below to inspect findings."
          illustration={
            <div style={{ marginTop: 'var(--space-2)' }}>
              <HotkeyChip
                shortcut={captureHotkey}
                disabled
                aria-label={`Capture shortcut: ${captureHotkey}`}
              />
            </div>
          }
        />
      </div>
    );
  }

  const isImageMissing = selectedFinding?.isImageMissing || imageLoadError;

  return (
    <div
      data-testid="findings-editor"
      style={{
        display: 'flex',
        flexDirection: 'row',
        height: '100%',
        width: '100%',
        backgroundColor: 'var(--color-bg)',
        overflow: 'hidden',
      }}
    >
      {/* COLUMN 1: Capture Rail (200px) */}
      <div
        data-testid="capture-rail"
        style={{
          width: '200px',
          minWidth: '200px',
          maxWidth: '200px',
          height: '100%',
          borderRight: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-surface)',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}
      >
        {/* Rail Header */}
        <div
          style={{
            padding: 'var(--space-3) var(--space-4)',
            borderBottom: '1px solid var(--color-border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <span
            style={{
              fontSize: 'var(--text-xs)',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
              color: 'var(--color-text-muted)',
            }}
          >
            Captures ({isLoading ? '…' : findings.length})
          </span>
        </div>

        {/* Rail Items List */}
        <div
          data-testid="capture-rail-list"
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: 'var(--space-3) var(--space-3)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-3)',
          }}
        >
          {isLoading ? (
            /* Loading State: Skeleton rail placeholders */
            Array.from({ length: 4 }).map((_, idx) => (
              <div
                key={idx}
                data-testid="rail-skeleton-thumb"
                style={{
                  width: '176px',
                  height: '100px',
                  borderRadius: 'var(--radius-sm)',
                  backgroundColor: 'var(--color-surface-sunken)',
                  border: '1px solid var(--color-border)',
                  animation: 'pulse 1.5s infinite ease-in-out',
                }}
              />
            ))
          ) : (
            sortedFindings.map((f) => {
              const isSelected = f.finding.id === selectedFindingId;
              const isChecked = checkedFindingIds.has(f.finding.id);

              return (
                <div
                  key={f.finding.id}
                  data-testid={`finding-item-${f.finding.id}`}
                  onClick={() => onSelectFinding(f.finding.id)}
                  style={{
                    width: '176px',
                    borderRadius: 'var(--radius-sm)',
                    border: isSelected
                      ? '2px solid var(--color-accent)'
                      : '1px solid var(--color-border)',
                    backgroundColor: isSelected
                      ? 'var(--color-info-bg)'
                      : 'var(--color-surface)',
                    cursor: 'pointer',
                    position: 'relative',
                    overflow: 'hidden',
                    display: 'flex',
                    flexDirection: 'column',
                    transition: 'all 0.15s ease',
                  }}
                >
                  {/* Thumbnail / Image Preview container */}
                  <div
                    style={{
                      width: '100%',
                      height: '96px',
                      backgroundColor: 'var(--color-surface-sunken)',
                      position: 'relative',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      overflow: 'hidden',
                    }}
                  >
                    {f.imageSrc ? (
                      <img
                        src={f.imageSrc}
                        alt={`Capture ${f.finding.id}`}
                        style={{
                          width: '100%',
                          height: '100%',
                          objectFit: 'cover',
                        }}
                      />
                    ) : (
                      <div
                        style={{
                          fontSize: 'var(--text-xs)',
                          color: 'var(--color-text-muted)',
                          fontFamily: 'var(--font-mono)',
                        }}
                      >
                        {f.finding.image_width} × {f.finding.image_height}
                      </div>
                    )}

                    {/* Checkbox overlay on top-left */}
                    <div
                      data-testid={`finding-checkbox-wrapper-${f.finding.id}`}
                      onClick={(e) => handleToggleCheck(f.finding.id, e)}
                      style={{
                        position: 'absolute',
                        top: 'var(--space-1)',
                        left: 'var(--space-1)',
                        zIndex: 5,
                        backgroundColor: 'var(--color-surface)',
                        borderRadius: 'var(--radius-sm)',
                        padding: '2px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        boxShadow: 'var(--shadow-raised)',
                      }}
                    >
                      <Checkbox
                        data-testid={`finding-checkbox-${f.finding.id}`}
                        checked={isChecked}
                        onChange={() => {}}
                        aria-label={`Select finding ${f.finding.id}`}
                      />
                    </div>

                    {/* Markers count badge on top-right */}
                    {f.markers.length > 0 && (
                      <div
                        style={{
                          position: 'absolute',
                          top: 'var(--space-1)',
                          right: 'var(--space-1)',
                          zIndex: 5,
                        }}
                      >
                        <Badge variant="warning">{f.markers.length}</Badge>
                      </div>
                    )}
                  </div>

                  {/* Thumbnail Footer info */}
                  <div
                    style={{
                      padding: 'var(--space-1) var(--space-2)',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      fontSize: 'var(--text-xs)',
                      color: isSelected ? 'var(--color-info-text)' : 'var(--color-text-muted)',
                      fontFamily: 'var(--font-mono)',
                    }}
                  >
                    <span>{formatTimestamp(f.finding.captured_at)}</span>
                    <span>{f.finding.image_width}×{f.finding.image_height}</span>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Pinned Rail Footer for Multi-Select Compose Bridge (FR-9) */}
        {checkedFindingIds.size > 0 && (
          <div
            data-testid="rail-footer"
            style={{
              padding: 'var(--space-3)',
              borderTop: '1px solid var(--color-border)',
              backgroundColor: 'var(--color-surface)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
            }}
          >
            <span
              data-testid="selection-count"
              style={{
                fontSize: 'var(--text-xs)',
                fontWeight: 600,
                color: 'var(--color-text)',
              }}
            >
              {checkedFindingIds.size} selected
            </span>
            <Button
              variant="primary"
              data-testid="compose-button"
              onClick={() => onCompose?.(Array.from(checkedFindingIds))}
              style={{ width: '100%' }}
            >
              Compose →
            </Button>
          </div>
        )}
      </div>

      {/* COLUMN 2: Canvas (Flex Growing Center) */}
      <div
        data-testid="findings-canvas-container"
        style={{
          flex: 1,
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          backgroundColor: 'var(--color-surface-sunken)',
          overflow: 'hidden',
          position: 'relative',
        }}
      >
        {selectedFinding ? (
          isImageMissing ? (
            /* State 5: Image Missing State */
            <div
              data-testid="canvas-image-missing"
              style={{
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: 'var(--space-6)',
              }}
            >
              <div
                style={{
                  backgroundColor: 'var(--color-warning-bg)',
                  border: '1px solid var(--color-border)',
                  borderRadius: 'var(--radius-md)',
                  padding: 'var(--space-6)',
                  maxWidth: '32rem',
                  textAlign: 'center',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: 'var(--space-3)',
                }}
              >
                <div
                  style={{
                    fontSize: 'var(--text-lg)',
                    fontWeight: 700,
                    color: 'var(--color-warning-text)',
                  }}
                >
                  Image file missing
                </div>
                <p
                  style={{
                    margin: 0,
                    fontSize: 'var(--text-sm)',
                    color: 'var(--color-text-muted)',
                    wordBreak: 'break-all',
                    fontFamily: 'var(--font-mono)',
                  }}
                >
                  {selectedFinding.finding.image_path}
                </p>
                {onOpenOrphanReport && (
                  <div style={{ marginTop: 'var(--space-2)' }}>
                    <Button
                      variant="primary"
                      data-testid="open-orphan-report-button"
                      onClick={onOpenOrphanReport}
                    >
                      Open Orphan Report
                    </Button>
                  </div>
                )}
              </div>
            </div>
          ) : (
            /* Populated Canvas: Screenshot + MarkerLayer Overlay */
            <div
              style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                height: '100%',
                overflow: 'hidden',
              }}
            >
              {/* Main Canvas Viewport */}
              <div
                data-testid="canvas-viewport"
                style={{
                  flex: 1,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  padding: 'var(--space-4)',
                  overflow: 'auto',
                  position: 'relative',
                }}
              >
                <div
                  style={{
                    position: 'relative',
                    display: 'inline-block',
                    maxWidth: '100%',
                    maxHeight: '100%',
                    boxShadow: 'var(--shadow-raised)',
                    borderRadius: 'var(--radius-sm)',
                    overflow: 'hidden',
                    lineHeight: 0,
                  }}
                >
                  {/* Finding Screenshot Image */}
                  <img
                    data-testid="finding-image"
                    src={selectedFinding.imageSrc || selectedFinding.finding.image_path}
                    alt={`Finding ${selectedFinding.finding.id}`}
                    onError={() => setImageLoadError(true)}
                    style={{
                      display: 'block',
                      maxWidth: '100%',
                      maxHeight: 'calc(100vh - 140px)',
                      objectFit: 'contain',
                      userSelect: 'none',
                    }}
                  />

                  {/* MarkerLayer Mounted Directly Over Screenshot */}
                  <MarkerLayer
                    markers={markerLayerItems}
                    selectedMarkerId={selectedMarkerId}
                    hoveredMarkerId={hoveredMarkerId}
                    onAddMarker={handleAddMarker}
                    onUpdateMarkerPosition={handleUpdateMarkerPosition}
                    onSelectMarker={setSelectedMarkerId}
                    onHoverMarker={setHoveredMarkerId}
                    onDeleteMarker={handleDeleteMarker}
                  />
                </div>
              </div>

              {/* Readout bar beneath canvas: Dimensions & stored byte size */}
              <div
                data-testid="canvas-readout"
                style={{
                  padding: 'var(--space-2) var(--space-4)',
                  backgroundColor: 'var(--color-surface)',
                  borderTop: '1px solid var(--color-border)',
                  fontSize: 'var(--text-xs)',
                  fontFamily: 'var(--font-mono)',
                  color: 'var(--color-text-muted)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 'var(--space-2)',
                }}
              >
                <span>
                  {selectedFinding.finding.image_width} × {selectedFinding.finding.image_height} px
                </span>
                <span>·</span>
                <span>{formatFileSize(selectedFinding.finding.file_size_bytes)}</span>
              </div>
            </div>
          )
        ) : (
          /* State 4: Nothing Selected State */
          <div
            data-testid="no-finding-selected"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: 'var(--color-text-muted)',
              fontSize: 'var(--text-sm)',
              fontFamily: 'var(--font-ui)',
            }}
          >
            Select a finding from the rail to inspect details.
          </div>
        )}
      </div>

      {/* COLUMN 3: Note Pane (320px) */}
      <div
        data-testid="note-pane"
        style={{
          width: '320px',
          minWidth: '320px',
          maxWidth: '320px',
          height: '100%',
          borderLeft: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-surface)',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}
      >
        {selectedFinding ? (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              height: '100%',
              overflow: 'hidden',
            }}
          >
            {/* Note Pane Header */}
            <div
              style={{
                padding: 'var(--space-3) var(--space-4)',
                borderBottom: '1px solid var(--color-border)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <h3
                style={{
                  margin: 0,
                  fontSize: 'var(--text-sm)',
                  fontWeight: 600,
                  color: 'var(--color-text)',
                }}
              >
                Finding Note
              </h3>
              {saveSuccess && (
                <span
                  data-testid="save-success-indicator"
                  style={{
                    fontSize: 'var(--text-xs)',
                    color: 'var(--color-success-text)',
                    fontWeight: 600,
                  }}
                >
                  Saved!
                </span>
              )}
            </div>

            {/* Scrollable Content: Note editor and Marker List */}
            <div
              style={{
                flex: 1,
                overflowY: 'auto',
                padding: 'var(--space-4)',
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--space-4)',
              }}
            >
              {/* Note Textarea */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
                <label
                  htmlFor="note-body-textarea"
                  style={{
                    fontSize: 'var(--text-xs)',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                    letterSpacing: '0.05em',
                    color: 'var(--color-text-muted)',
                  }}
                >
                  Note
                </label>
                <TextArea
                  id="note-body-textarea"
                  data-testid="note-textarea"
                  value={noteText}
                  onChange={handleNoteChange}
                  onBlur={handleNoteBlur}
                  placeholder="1. What is wrong here..."
                  rows={6}
                  autoGrow={false}
                />
                <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                  <Button
                    variant="primary"
                    data-testid="save-note-button"
                    onClick={() => handleSaveNote(noteText)}
                    disabled={isSaving}
                  >
                    {isSaving ? 'Saving...' : 'Save Note'}
                  </Button>
                </div>
              </div>

              {/* Marker List (Keyboard Accessibility & SCN-04 Unbound Reporting) */}
              <div
                data-testid="note-markers-section"
                style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}
              >
                <div
                  style={{
                    fontSize: 'var(--text-xs)',
                    fontWeight: 600,
                    textTransform: 'uppercase',
                    letterSpacing: '0.05em',
                    color: 'var(--color-text-muted)',
                    display: 'flex',
                    justifyContent: 'space-between',
                  }}
                >
                  <span>Markers ({selectedFinding.markers.length})</span>
                </div>

                {selectedFinding.markers.length === 0 ? (
                  <p
                    data-testid="no-markers-hint"
                    style={{
                      margin: 0,
                      fontSize: 'var(--text-xs)',
                      color: 'var(--color-text-muted)',
                      fontStyle: 'italic',
                    }}
                  >
                    Click the screenshot to place a numbered marker.
                  </p>
                ) : (
                  <div
                    data-testid="note-markers-list"
                    style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}
                  >
                    {selectedFinding.markers
                      .slice()
                      .sort((a, b) => a.ordinal - b.ordinal)
                      .map((marker) => {
                        const isBound = isMarkerBoundInNote(marker.ordinal, noteText);
                        const isSelected = marker.id === selectedMarkerId;
                        const isHovered = marker.id === hoveredMarkerId;

                        return (
                          <div
                            key={marker.id}
                            tabIndex={0}
                            role="listitem"
                            data-testid={`marker-list-item-${marker.ordinal}`}
                            onClick={() => setSelectedMarkerId(marker.id)}
                            onMouseEnter={() => setHoveredMarkerId(marker.id)}
                            onMouseLeave={() => setHoveredMarkerId(null)}
                            onKeyDown={(e) => {
                              if (e.key === 'Delete' || e.key === 'Backspace') {
                                e.preventDefault();
                                handleDeleteMarker(marker.id);
                              } else if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                setSelectedMarkerId(marker.id);
                              }
                            }}
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              justifyContent: 'space-between',
                              padding: 'var(--space-2)',
                              borderRadius: 'var(--radius-sm)',
                              border: isSelected
                                ? '1px solid var(--color-accent)'
                                : '1px solid var(--color-border)',
                              backgroundColor: isHovered || isSelected
                                ? 'var(--color-surface-sunken)'
                                : 'var(--color-surface)',
                              outline: 'none',
                              cursor: 'pointer',
                              gap: 'var(--space-2)',
                            }}
                          >
                            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
                              <MarkerBadge
                                number={marker.ordinal}
                                isSelected={isSelected}
                                isHovered={isHovered}
                              />
                              <div style={{ display: 'flex', flexDirection: 'column' }}>
                                <span
                                  style={{
                                    fontSize: 'var(--text-xs)',
                                    fontFamily: 'var(--font-mono)',
                                    color: 'var(--color-text)',
                                  }}
                                >
                                  {marker.comment || `(${Math.round(marker.x * 100)}%, ${Math.round(marker.y * 100)}%)`}
                                </span>
                                {!isBound && (
                                  <span
                                    data-testid={`marker-unbound-${marker.ordinal}`}
                                    style={{
                                      fontSize: 'var(--text-xs)',
                                      color: 'var(--color-warning-text)',
                                      fontWeight: 500,
                                    }}
                                  >
                                    Unbound / No note line
                                  </span>
                                )}
                              </div>
                            </div>

                            <button
                              type="button"
                              data-testid={`delete-marker-button-${marker.ordinal}`}
                              onClick={(e) => {
                                e.stopPropagation();
                                handleDeleteMarker(marker.id);
                              }}
                              aria-label={`Delete marker ${marker.ordinal}`}
                              style={{
                                background: 'none',
                                border: 'none',
                                color: 'var(--color-text-muted)',
                                cursor: 'pointer',
                                padding: 'var(--space-1)',
                                borderRadius: 'var(--radius-sm)',
                                fontSize: 'var(--text-sm)',
                                lineHeight: 1,
                              }}
                            >
                              ✕
                            </button>
                          </div>
                        );
                      })}
                  </div>
                )}
              </div>
            </div>

            {/* Note Pane Footer: Delete Finding action */}
            {onDeleteFinding && (
              <div
                style={{
                  padding: 'var(--space-3) var(--space-4)',
                  borderTop: '1px solid var(--color-border)',
                  backgroundColor: 'var(--color-surface)',
                }}
              >
                <Button
                  variant="danger"
                  data-testid="delete-finding-button"
                  onClick={() => setIsDeleteDialogOpen(true)}
                  style={{ width: '100%' }}
                >
                  Delete Finding
                </Button>
              </div>
            )}
          </div>
        ) : (
          <div
            style={{
              padding: 'var(--space-4)',
              color: 'var(--color-text-muted)',
              fontSize: 'var(--text-sm)',
            }}
          >
            No finding selected.
          </div>
        )}
      </div>

      {/* Single Confirmation Dialog for Finding Deletion (FR-13) */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title="Delete Finding"
        message="Are you sure you want to delete this finding? The screenshot image and its notes will be permanently removed."
        confirmLabel="Delete Finding"
        cancelLabel="Cancel"
        loading={isDeleting}
        onConfirm={handleConfirmDelete}
        onCancel={() => setIsDeleteDialogOpen(false)}
      />
    </div>
  );
};
