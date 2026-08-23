import React, { useEffect, useState } from 'react';
import { Button } from './Button';
import { TextArea } from './TextArea';

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
}

export const FindingsEditor: React.FC<FindingsEditorProps> = ({
  findings,
  selectedFindingId,
  onSelectFinding,
  onSaveNote,
  onDeleteFinding,
}) => {
  const selectedFinding = findings.find((f) => f.finding.id === selectedFindingId);
  const [noteText, setNoteText] = useState<string>('');
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  useEffect(() => {
    if (selectedFinding) {
      setNoteText(selectedFinding.note.body);
      setSaveSuccess(false);
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

  return (
    <div
      data-testid="findings-editor"
      style={{
        display: 'flex',
        height: '100%',
        minHeight: '400px',
        border: '1px solid var(--color-border)',
        borderRadius: 'var(--radius-md)',
        overflow: 'hidden',
      }}
    >
      {/* Sidebar Finding List */}
      <div
        data-testid="findings-sidebar"
        style={{
          width: '280px',
          borderRight: '1px solid var(--color-border)',
          overflowY: 'auto',
          backgroundColor: 'var(--color-bg)',
          padding: 'var(--space-3)',
        }}
      >
        <h3
          style={{
            margin: '0 0 var(--space-3) 0',
            fontSize: 'var(--text-sm)',
            fontWeight: 600,
            color: 'var(--color-text)',
          }}
        >
          Findings ({findings.length})
        </h3>
        {findings.length === 0 ? (
          <p
            data-testid="empty-findings-message"
            style={{ fontSize: 'var(--text-sm)', color: 'var(--color-text-muted)' }}
          >
            No findings captured yet.
          </p>
        ) : (
          <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
            {findings.map((f) => {
              const isSelected = f.finding.id === selectedFindingId;
              return (
                <li
                  key={f.finding.id}
                  data-testid={`finding-item-${f.finding.id}`}
                  onClick={() => onSelectFinding(f.finding.id)}
                  style={{
                    padding: 'var(--space-2) var(--space-3)',
                    borderRadius: 'var(--radius-sm)',
                    cursor: 'pointer',
                    backgroundColor: isSelected ? 'var(--color-info-bg)' : 'var(--color-surface)',
                    border: isSelected ? '1px solid var(--color-accent)' : '1px solid var(--color-border)',
                  }}
                >
                  <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)', color: isSelected ? 'var(--color-info-text)' : 'var(--color-text)' }}>
                    {f.finding.image_path}
                  </div>
                  <div style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)', marginTop: 'var(--space-1)' }}>
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
          padding: 'var(--space-4)',
          overflowY: 'auto',
          backgroundColor: 'var(--color-surface)',
        }}
      >
        {selectedFinding ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <h2 style={{ margin: 0, fontSize: 'var(--text-base)', fontWeight: 600, color: 'var(--color-text)' }}>
                Finding Details
              </h2>
              {onDeleteFinding && (
                <Button
                  variant="secondary"
                  onClick={() => onDeleteFinding(selectedFinding.finding.id)}
                >
                  Delete
                </Button>
              )}
            </div>

            {/* Metadata */}
            <div
              data-testid="finding-metadata"
              style={{
                fontSize: 'var(--text-xs)',
                color: 'var(--color-neutral-text)',
                backgroundColor: 'var(--color-neutral-bg)',
                padding: 'var(--space-2) var(--space-3)',
                borderRadius: 'var(--radius-sm)',
              }}
            >
              <div><strong>ID:</strong> {selectedFinding.finding.id}</div>
              <div><strong>Captured at:</strong> {selectedFinding.finding.captured_at}</div>
              <div><strong>Resolution:</strong> {selectedFinding.finding.image_width} × {selectedFinding.finding.image_height} px</div>
              <div><strong>Region:</strong> {selectedFinding.finding.region}</div>
            </div>

            {/* Note Editor */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              <label htmlFor="note-body-textarea" style={{ fontSize: 'var(--text-sm)', fontWeight: 500, color: 'var(--color-text)' }}>
                Note
              </label>
              <TextArea
                id="note-body-textarea"
                value={noteText}
                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setNoteText(e.target.value)}
                placeholder="Enter finding note..."
                rows={5}
              />
              <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
                <Button
                  variant="primary"
                  onClick={handleSave}
                  disabled={isSaving}
                >
                  {isSaving ? 'Saving...' : 'Save Note'}
                </Button>
                {saveSuccess && (
                  <span data-testid="save-success-indicator" style={{ fontSize: 'var(--text-xs)', color: 'var(--color-success-text)', fontWeight: 600 }}>
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
              color: 'var(--color-text-muted)',
              fontSize: 'var(--text-sm)',
            }}
          >
            Select a finding from the sidebar to inspect details.
          </div>
        )}
      </div>
    </div>
  );
};
