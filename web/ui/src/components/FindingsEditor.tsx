import React, { useEffect, useState } from 'react';
import { Button } from './Button';
import { TextArea } from './TextArea';
import { TextField } from './TextField';
import { MarkerLayer } from './MarkerLayer';

export interface FindingItemDto {
  id: string;
  image_path: string;
  image_width: number;
  image_height: number;
  captured_at: string;
  source_monitor: string;
  region: string;
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
}

export interface FindingsEditorProps {
  findings: FindingDetailItemDto[];
  selectedFindingId: string | null;
  onSelectFinding: (id: string) => void;
  onSaveNote: (findingId: string, noteBody: string) => Promise<void>;
  onDeleteFinding?: (findingId: string) => Promise<void>;
  onAddMarker?: (findingId: string, x: number, y: number) => Promise<void>;
  onUpdateMarker?: (findingId: string, markerId: string, x: number, y: number, comment: string) => Promise<void>;
  onDeleteMarker?: (findingId: string, markerId: string) => Promise<void>;
}

export const FindingsEditor: React.FC<FindingsEditorProps> = ({
  findings,
  selectedFindingId,
  onSelectFinding,
  onSaveNote,
  onDeleteFinding,
  onAddMarker,
  onUpdateMarker,
  onDeleteMarker,
}) => {
  const selectedFinding = findings.find((f) => f.finding.id === selectedFindingId);
  const [noteText, setNoteText] = useState<string>('');
  const [selectedMarkerId, setSelectedMarkerId] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    if (selectedFinding) {
      setNoteText(selectedFinding.note.body);
      setSaveSuccess(false);
      if (selectedFinding.markers.length > 0) {
        setSelectedMarkerId(selectedFinding.markers[0].id);
      } else {
        setSelectedMarkerId(null);
      }
    }
  }, [selectedFindingId, selectedFinding]);

  const handleSave = async () => {
    if (!selectedFinding) return;
    setIsSaving(true);
    try {
      await onSaveNote(selectedFinding.finding.id, noteText);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 2000);
    } finally {
      setIsSaving(false);
    }
  };

  const handleAddMarker = async (x: number, y: number) => {
    if (!selectedFinding || !onAddMarker) return;
    await onAddMarker(selectedFinding.finding.id, x, y);
  };

  const handleUpdateMarkerPosition = async (markerId: string, x: number, y: number) => {
    if (!selectedFinding || !onUpdateMarker) return;
    const m = selectedFinding.markers.find((item) => item.id === markerId);
    if (!m) return;
    await onUpdateMarker(selectedFinding.finding.id, markerId, x, y, m.comment);
  };

  const handleCommentChange = async (markerId: string, newComment: string) => {
    if (!selectedFinding || !onUpdateMarker) return;
    const m = selectedFinding.markers.find((item) => item.id === markerId);
    if (!m) return;
    await onUpdateMarker(selectedFinding.finding.id, markerId, m.x, m.y, newComment);
  };

  const handleDeleteMarker = async (markerId: string) => {
    if (!selectedFinding || !onDeleteMarker) return;
    await onDeleteMarker(selectedFinding.finding.id, markerId);
  };

  return (
    <div
      data-testid="findings-editor"
      style={{
        display: 'flex',
        height: '100%',
        minHeight: '400px',
        border: '1px solid var(--color-border, #e2e8f0)',
        borderRadius: '8px',
        overflow: 'hidden',
      }}
    >
      {/* Sidebar Finding List */}
      <div
        data-testid="findings-sidebar"
        style={{
          width: '280px',
          borderRight: '1px solid var(--color-border, #e2e8f0)',
          overflowY: 'auto',
          backgroundColor: 'var(--color-bg-subtle, #f8fafc)',
          padding: '12px',
        }}
      >
        <h3
          style={{
            margin: '0 0 12px 0',
            fontSize: '14px',
            fontWeight: 600,
            color: 'var(--color-text, #1e293b)',
          }}
        >
          Findings ({findings.length})
        </h3>
        {findings.length === 0 ? (
          <p
            data-testid="empty-findings-message"
            style={{ fontSize: '13px', color: 'var(--color-text-muted, #64748b)' }}
          >
            No findings captured yet.
          </p>
        ) : (
          <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: '8px' }}>
            {findings.map((f) => {
              const isSelected = f.finding.id === selectedFindingId;
              return (
                <li
                  key={f.finding.id}
                  data-testid={`finding-item-${f.finding.id}`}
                  onClick={() => onSelectFinding(f.finding.id)}
                  style={{
                    padding: '8px 12px',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    backgroundColor: isSelected ? 'var(--color-primary-subtle, #e0f2fe)' : '#ffffff',
                    border: isSelected ? '1px solid var(--color-primary, #3b82f6)' : '1px solid var(--color-border, #e2e8f0)',
                  }}
                >
                  <div style={{ fontWeight: 500, fontSize: '13px', color: '#1e293b' }}>
                    {f.finding.image_path}
                  </div>
                  <div style={{ fontSize: '11px', color: '#64748b', marginTop: '4px' }}>
                    {f.finding.image_width} × {f.finding.image_height} px • {f.markers.length} markers
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {/* Main Detail Pane */}
      <div
        data-testid="findings-detail-pane"
        style={{
          flex: 1,
          padding: '16px',
          overflowY: 'auto',
          backgroundColor: '#ffffff',
          display: 'flex',
          flexDirection: 'column',
          gap: '16px',
        }}
      >
        {selectedFinding ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h2 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>
                Finding Details
              </h2>
              {onDeleteFinding && (
                <Button
                  variant="secondary"
                  onClick={() => onDeleteFinding(selectedFinding.finding.id)}
                >
                  Delete Finding
                </Button>
              )}
            </div>

            {/* Canvas Marker Layer */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <label style={{ fontSize: '13px', fontWeight: 500 }}>
                Screenshot & Markers
              </label>
              <div style={{ width: '100%', height: '240px', borderRadius: '6px', overflow: 'hidden' }}>
                <MarkerLayer
                  markers={selectedFinding.markers}
                  selectedMarkerId={selectedMarkerId}
                  onAddMarker={handleAddMarker}
                  onUpdateMarkerPosition={handleUpdateMarkerPosition}
                  onSelectMarker={(mId) => setSelectedMarkerId(mId)}
                />
              </div>
            </div>

            {/* Marker List & Annotations */}
            {selectedFinding.markers.length > 0 && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                <label style={{ fontSize: '13px', fontWeight: 500 }}>
                  Marker Annotations (Single-Sequence)
                </label>
                <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                  {selectedFinding.markers.map((m) => (
                    <div
                      key={m.id}
                      data-testid={`marker-row-${m.ordinal}`}
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                        padding: '6px 10px',
                        backgroundColor: '#f8fafc',
                        border: '1px solid #e2e8f0',
                        borderRadius: '6px',
                      }}
                    >
                      <span style={{ fontWeight: 700, fontSize: '12px', minWidth: '24px' }}>
                        #{m.ordinal}
                      </span>
                      <TextField
                        id={`marker-comment-input-${m.ordinal}`}
                        value={m.comment}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => handleCommentChange(m.id, e.target.value)}
                        placeholder={`Annotation for badge ${m.ordinal}`}
                        style={{ flex: 1 }}
                      />
                      {onDeleteMarker && (
                        <Button
                          variant="secondary"
                          onClick={() => handleDeleteMarker(m.id)}
                        >
                          Remove
                        </Button>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Metadata */}
            <div
              data-testid="finding-metadata"
              style={{
                fontSize: '12px',
                color: '#475569',
                backgroundColor: '#f1f5f9',
                padding: '8px 12px',
                borderRadius: '6px',
              }}
            >
              <div><strong>ID:</strong> {selectedFinding.finding.id}</div>
              <div><strong>Captured at:</strong> {selectedFinding.finding.captured_at}</div>
              <div><strong>Resolution:</strong> {selectedFinding.finding.image_width} × {selectedFinding.finding.image_height} px</div>
              <div><strong>Region:</strong> {selectedFinding.finding.region}</div>
            </div>

            {/* Note Editor */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <label htmlFor="note-body-textarea" style={{ fontSize: '13px', fontWeight: 500 }}>
                Note
              </label>
              <TextArea
                id="note-body-textarea"
                value={noteText}
                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setNoteText(e.target.value)}
                placeholder="Enter finding note..."
                rows={4}
              />
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Button
                  variant="primary"
                  onClick={handleSave}
                  disabled={isSaving}
                >
                  {isSaving ? 'Saving...' : 'Save Note'}
                </Button>
                {saveSuccess && (
                  <span data-testid="save-success-indicator" style={{ fontSize: '12px', color: '#16a34a' }}>
                    Saved!
                  </span>
                )}
              </div>
            </div>
          </div>
        ) : (
          <div
            data-testid="no-finding-selected"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: '#94a3b8',
              fontSize: '14px',
            }}
          >
            Select a finding from the sidebar to inspect details.
          </div>
        )}
      </div>
    </div>
  );
};
