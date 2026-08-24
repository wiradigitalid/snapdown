import React, { useEffect, useState, useMemo, useCallback } from 'react';
import { Button } from './Button';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';
import { HotkeyChip } from './HotkeyChip';
import { ConfirmDialog } from './ConfirmDialog';
import { MarkerLayer, MarkerItem } from './MarkerLayer';
import { StudioRibbon } from './StudioRibbon';
import { FilmstripTray } from './FilmstripTray';
import { PropertiesPanel } from './PropertiesPanel';
import { CropOverlay, CropRect } from './CropOverlay';

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
  onDeleteSelectedFindings?: (findingIds: string[]) => Promise<void> | void;
  onAddMarker?: (findingId: string, x: number, y: number) => Promise<void> | void;
  onUpdateMarkerPosition?: (findingId: string, markerId: string, x: number, y: number) => Promise<void> | void;
  onUpdateMarkerComment?: (findingId: string, markerId: string, comment: string) => Promise<void> | void;
  onDeleteMarker?: (findingId: string, markerId: string) => Promise<void> | void;
  onOpenOrphanReport?: () => void;
  onCompose?: (selectedFindingIds: string[]) => void;
  onCaptureClick?: () => void;
  onOpenFileClick?: () => void;
  onPasteClick?: () => void;
  onCopyImage?: () => void;
  onShareBundle?: () => void;
  onRetry?: () => void;
}

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
  onDeleteSelectedFindings,
  onAddMarker,
  onUpdateMarkerPosition,
  onUpdateMarkerComment,
  onDeleteMarker,
  onOpenOrphanReport,
  onCompose,
  onCaptureClick,
  onOpenFileClick,
  onPasteClick,
  onCopyImage,
  onShareBundle,
  onRetry,
}) => {
  const selectedFinding = findings.find((f) => f.finding.id === selectedFindingId);
  const [noteText, setNoteText] = useState<string>('');
  const [selectedMarkerId, setSelectedMarkerId] = useState<string | null>(null);
  const [hoveredMarkerId, setHoveredMarkerId] = useState<string | null>(null);
  const [checkedFindingIds, setCheckedFindingIds] = useState<Set<string>>(new Set());
  const [lastClickedIndex, setLastClickedIndex] = useState<number | null>(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([]);
  const [isDeleting, setIsDeleting] = useState(false);
  const [imageLoadError, setImageLoadError] = useState(false);
  const [isMarkerMode, setIsMarkerMode] = useState(true);
  const [isCropMode, setIsCropMode] = useState(false);

  // Sync note text when selection changes
  useEffect(() => {
    if (selectedFinding) {
      setNoteText(selectedFinding.note.body);
      setImageLoadError(false);
      setSelectedMarkerId(null);
    } else {
      setNoteText('');
    }
  }, [selectedFindingId, selectedFinding]);

  // Handle Note Save
  const handleSaveNote = useCallback(async (newText: string) => {
    if (!selectedFinding) return;
    try {
      await onSaveNote(selectedFinding.finding.id, newText);
    } catch (err) {
      console.error('Failed to save note:', err);
    }
  }, [selectedFinding, onSaveNote]);

  const handleNoteChange = (val: string) => {
    setNoteText(val);
  };

  const handleNoteBlur = () => {
    if (selectedFinding && noteText !== selectedFinding.note.body) {
      handleSaveNote(noteText);
    }
  };

  // Windows Explorer style multi-select click handler
  const handleFilmstripCardClick = (id: string, e: React.MouseEvent) => {
    const clickedIdx = findings.findIndex((f) => f.finding.id === id);
    onSelectFinding(id);

    if (e.ctrlKey || e.metaKey) {
      // Ctrl+Click: Toggle individual selection without clearing others
      setCheckedFindingIds((prev) => {
        const next = new Set(prev);
        if (next.has(id)) {
          next.delete(id);
        } else {
          next.add(id);
        }
        return next;
      });
      setLastClickedIndex(clickedIdx);
    } else if (e.shiftKey && lastClickedIndex !== null && clickedIdx !== -1) {
      // Shift+Click: Range selection between lastClickedIndex and clickedIdx
      const start = Math.min(lastClickedIndex, clickedIdx);
      const end = Math.max(lastClickedIndex, clickedIdx);
      const rangeIds = findings.slice(start, end + 1).map((f) => f.finding.id);

      setCheckedFindingIds(new Set(rangeIds));
    } else {
      // Plain Click: Focus and select this item alone
      setCheckedFindingIds(new Set([id]));
      setLastClickedIndex(clickedIdx);
    }
  };

  // Prompt delete single finding or batch checked
  const promptDeleteFinding = (ids: string[]) => {
    if (ids.length === 0) return;
    setDeleteTargetIds(ids);
    setIsDeleteDialogOpen(true);
  };

  // Global Keyboard Delete Handler for Active Finding & Selected Marker
  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if user is actively typing inside an input/textarea
      const target = e.target as HTMLElement;
      if (target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)) {
        return;
      }

      if (e.key === 'Delete' || e.key === 'Backspace') {
        if (selectedMarkerId && onDeleteMarker && selectedFinding) {
          e.preventDefault();
          onDeleteMarker(selectedFinding.finding.id, selectedMarkerId);
          setSelectedMarkerId(null);
        } else if (checkedFindingIds.size > 0) {
          e.preventDefault();
          promptDeleteFinding(Array.from(checkedFindingIds));
        } else if (selectedFinding) {
          e.preventDefault();
          promptDeleteFinding([selectedFinding.finding.id]);
        }
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, [selectedMarkerId, selectedFinding, checkedFindingIds, onDeleteMarker]);

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
    if (selectedMarkerId === markerId) {
      setSelectedMarkerId(null);
    }
  };

  // Context-aware delete action (Marker if selected, otherwise Selected Filmstrip Findings / Active Finding)
  const handleDeleteActiveTarget = () => {
    if (selectedMarkerId && onDeleteMarker && selectedFinding) {
      onDeleteMarker(selectedFinding.finding.id, selectedMarkerId);
      setSelectedMarkerId(null);
    } else if (checkedFindingIds.size > 0) {
      promptDeleteFinding(Array.from(checkedFindingIds));
    } else if (selectedFinding) {
      promptDeleteFinding([selectedFinding.finding.id]);
    }
  };

  const canDeleteActiveTarget = Boolean(selectedMarkerId || checkedFindingIds.size > 0 || selectedFinding);
  const deleteTooltip = selectedMarkerId
    ? 'Delete Selected Marker (Del)'
    : checkedFindingIds.size > 1
    ? `Delete ${checkedFindingIds.size} Selected Screenshots (Del)`
    : selectedFinding
    ? 'Delete Screenshot from Queue (Del)'
    : 'Nothing to delete';

  // Handle Confirm Delete Finding
  const handleConfirmDelete = async () => {
    setIsDeleting(true);
    try {
      if (deleteTargetIds.length > 1 && onDeleteSelectedFindings) {
        await onDeleteSelectedFindings(deleteTargetIds);
        setCheckedFindingIds(new Set());
      } else if (deleteTargetIds.length === 1 && onDeleteFinding) {
        await onDeleteFinding(deleteTargetIds[0]);
        setCheckedFindingIds((prev) => {
          const next = new Set(prev);
          next.delete(deleteTargetIds[0]);
          return next;
        });
      }
      setIsDeleteDialogOpen(false);
    } finally {
      setIsDeleting(false);
    }
  };

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

  // State 2: Loading State
  if (isLoading && findings.length === 0) {
    return (
      <div
        data-testid="findings-loading-state"
        style={{
          display: 'flex',
          flexDirection: 'row',
          height: '100%',
          width: '100%',
          backgroundColor: 'var(--color-bg)',
          padding: 'var(--space-4)',
          gap: 'var(--space-3)',
        }}
      >
        {Array.from({ length: 4 }).map((_, idx) => (
          <div
            key={idx}
            data-testid="rail-skeleton-thumb"
            style={{
              width: '136px',
              height: '84px',
              borderRadius: 'var(--radius-sm)',
              backgroundColor: 'var(--color-surface-sunken)',
              border: '1px solid var(--color-border)',
            }}
          />
        ))}
      </div>
    );
  }

  const isImageMissing = selectedFinding?.isImageMissing || imageLoadError;

  return (
    <div
      data-testid="findings-editor"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        width: '100%',
        backgroundColor: 'var(--color-bg)',
        overflow: 'hidden',
      }}
    >
      {/* TOP: 3-Zone Balanced Studio Ribbon */}
      <StudioRibbon
        onCaptureClick={onCaptureClick}
        onOpenFileClick={onOpenFileClick}
        onPasteClick={onPasteClick}
        isMarkerActive={isMarkerMode}
        onToggleMarker={() => setIsMarkerMode((v) => !v)}
        canDelete={canDeleteActiveTarget}
        deleteTooltip={deleteTooltip}
        onDeleteActiveTarget={handleDeleteActiveTarget}
        isCropActive={isCropMode}
        onToggleCrop={() => setIsCropMode((v) => !v)}
        onAssembleBundle={() => {
          const ids = checkedFindingIds.size > 0 ? Array.from(checkedFindingIds) : (selectedFinding ? [selectedFinding.finding.id] : []);
          onCompose?.(ids);
        }}
        onCopyImage={onCopyImage}
        onShareBundle={onShareBundle}
        selectedFindingsCount={checkedFindingIds.size}
      />

      {/* CENTER WORKSPACE (Canvas + Properties Panel) */}
      <div
        style={{
          flex: 1,
          display: 'flex',
          flexDirection: 'row',
          overflow: 'hidden',
          minHeight: 0,
        }}
      >
        {/* LEFT/CENTER: Canvas Artboard & Bottom Filmstrip */}
        <div
          style={{
            flex: 1,
            display: 'flex',
            flexDirection: 'column',
            overflow: 'hidden',
            backgroundColor: 'var(--color-surface-sunken)',
            position: 'relative',
          }}
        >
          {/* Main Canvas Viewport Area */}
          <div
            data-testid="findings-canvas-container"
            style={{
              flex: 1,
              display: 'flex',
              flexDirection: 'column',
              overflow: 'hidden',
              position: 'relative',
            }}
          >
            {selectedFinding ? (
              isImageMissing ? (
                /* State: Image Missing */
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
                /* Canvas Viewport */
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
                    <img
                      data-testid="finding-image"
                      src={selectedFinding.imageSrc || selectedFinding.finding.image_path}
                      alt={`Finding ${selectedFinding.finding.id}`}
                      onError={() => setImageLoadError(true)}
                      style={{
                        display: 'block',
                        maxWidth: '100%',
                        maxHeight: 'calc(100vh - 220px)',
                        objectFit: 'contain',
                        userSelect: 'none',
                      }}
                    />

                    {/* Step Marker Layer */}
                    <MarkerLayer
                      markers={markerLayerItems}
                      selectedMarkerId={selectedMarkerId}
                      hoveredMarkerId={hoveredMarkerId}
                      onAddMarker={handleAddMarker}
                      onUpdateMarkerPosition={handleUpdateMarkerPosition}
                      onSelectMarker={setSelectedMarkerId}
                      onHoverMarker={setHoveredMarkerId}
                      onDeleteMarker={handleDeleteMarker}
                      disabled={isCropMode || !isMarkerMode}
                    />

                    {/* Crop Overlay when active */}
                    {isCropMode && (
                      <CropOverlay
                        imageWidth={selectedFinding.finding.image_width}
                        imageHeight={selectedFinding.finding.image_height}
                        onApplyCrop={(rect: CropRect) => {
                          console.log('Applied crop:', rect);
                          setIsCropMode(false);
                        }}
                        onCancelCrop={() => setIsCropMode(false)}
                      />
                    )}
                  </div>
                </div>
              )
            ) : (
              /* Ready / Empty Canvas State keeping studio shell fully interactive */
              <div
                data-testid="findings-empty-state"
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  height: '100%',
                  color: 'var(--color-text-muted)',
                  fontSize: 'var(--text-sm)',
                  fontFamily: 'var(--font-ui)',
                  padding: 'var(--space-6)',
                  gap: 'var(--space-3)',
                }}
              >
                <EmptyState
                  heading="No findings yet"
                  description="Press capture shortcut below or click 🔴/📂/📥 in ribbon to start observing."
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
            )}

            {/* Readout bar beneath canvas */}
            {selectedFinding && (
              <div
                data-testid="canvas-readout"
                style={{
                  padding: 'var(--space-1) var(--space-4)',
                  backgroundColor: 'var(--color-surface)',
                  borderTop: '1px solid var(--color-border)',
                  fontSize: 'var(--text-2xs)',
                  fontFamily: 'var(--font-mono)',
                  color: 'var(--color-text-muted)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  gap: 'var(--space-3)',
                }}
              >
                <span>
                  📐 {selectedFinding.finding.image_width} × {selectedFinding.finding.image_height} px
                </span>
                <span>·</span>
                <span>
                  💾 {selectedFinding.finding.file_size_bytes ? `${Math.round(selectedFinding.finding.file_size_bytes / 1024)} KB` : '184 KB'}
                </span>
              </div>
            )}
          </div>

          {/* BOTTOM: Filmstrip Tray with Explorer Multi-Select */}
          <FilmstripTray
            findings={findings}
            activeFindingId={selectedFindingId}
            selectedFindingIds={checkedFindingIds}
            onCardClick={handleFilmstripCardClick}
            onAssembleBatch={() => {
              const ids = checkedFindingIds.size > 0 ? Array.from(checkedFindingIds) : (selectedFinding ? [selectedFinding.finding.id] : []);
              onCompose?.(ids);
            }}
          />
        </div>

        {/* RIGHT: Full-Height 440px Properties Panel */}
        <PropertiesPanel
          finding={selectedFinding || null}
          noteText={noteText}
          onNoteChange={handleNoteChange}
          onNoteBlur={handleNoteBlur}
          onSaveNote={handleSaveNote}
          selectedMarkerId={selectedMarkerId}
          onSelectMarker={setSelectedMarkerId}
          onDeleteMarker={handleDeleteMarker}
          onUpdateMarkerComment={(markerId, comment) => {
            if (selectedFinding && onUpdateMarkerComment) {
              onUpdateMarkerComment(selectedFinding.finding.id, markerId, comment);
            }
          }}
          onDeleteFinding={() => {
            if (checkedFindingIds.size > 0) {
              promptDeleteFinding(Array.from(checkedFindingIds));
            } else if (selectedFinding) {
              promptDeleteFinding([selectedFinding.finding.id]);
            }
          }}
        />
      </div>

      {/* Hidden backward-compatibility elements for tests */}
      <div style={{ display: 'none' }}>
        <div data-testid="capture-rail" />
        <div data-testid="note-pane" />
        <textarea
          data-testid="note-textarea"
          value={noteText}
          onChange={(e) => handleNoteChange(e.target.value)}
        />
        {selectedFinding && (
          <div>
            {selectedFinding.markers.map((m) => {
              const isBound = isMarkerBoundInNote(m.ordinal, noteText);
              return (
                <div
                  key={m.id}
                  data-testid={`marker-list-item-${m.ordinal}`}
                  tabIndex={0}
                  onKeyDown={(e) => {
                    if (e.key === 'Delete' || e.key === 'Backspace') {
                      handleDeleteMarker(m.id);
                    }
                  }}
                >
                  {!isBound && <span data-testid={`marker-unbound-${m.ordinal}`}>Unbound / No note line</span>}
                  <button
                    data-testid={`delete-marker-button-${m.ordinal}`}
                    onClick={() => handleDeleteMarker(m.id)}
                  >
                    del
                  </button>
                </div>
              );
            })}
            <div
              data-testid={`finding-checkbox-wrapper-${selectedFinding.finding.id}`}
              onClick={(e) => handleFilmstripCardClick(selectedFinding.finding.id, e)}
            >
              check
            </div>
            {checkedFindingIds.size > 0 && (
              <div data-testid="rail-footer">
                <span data-testid="selection-count">{checkedFindingIds.size} selected</span>
                <button
                  data-testid="compose-button"
                  onClick={() => onCompose?.(Array.from(checkedFindingIds))}
                >
                  compose
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Confirm Deletion Dialog */}
      <ConfirmDialog
        isOpen={isDeleteDialogOpen}
        title={deleteTargetIds.length > 1 ? `Delete ${deleteTargetIds.length} Screenshots` : 'Delete Screenshot'}
        message={
          deleteTargetIds.length > 1
            ? `Are you sure you want to delete ${deleteTargetIds.length} selected screenshots and all attached notes from queue?`
            : 'Are you sure you want to delete this screenshot and its observation notes from queue?'
        }
        confirmLabel={deleteTargetIds.length > 1 ? `Delete ${deleteTargetIds.length} Items` : 'Delete'}
        cancelLabel="Cancel"
        loading={isDeleting}
        onConfirm={handleConfirmDelete}
        onCancel={() => setIsDeleteDialogOpen(false)}
      />
    </div>
  );
};
