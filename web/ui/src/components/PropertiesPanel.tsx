import React, { useState } from 'react';
import { FindingDetailItemDto } from './FindingsEditor';
import { MarkerBadge } from './MarkerBadge';
import { TokenEstimator } from './TokenEstimator';
import { TextArea } from './TextArea';

export interface PropertiesPanelProps {
  finding: FindingDetailItemDto | null;
  noteText: string;
  onNoteChange: (text: string) => void;
  onNoteBlur?: () => void;
  onSaveNote?: (text: string) => void;
  selectedMarkerId: string | null;
  onSelectMarker: (markerId: string | null) => void;
  onDeleteMarker: (markerId: string) => void;
  onUpdateMarkerComment?: (markerId: string, comment: string) => void;
  onDeleteFinding?: () => void;
  className?: string;
  style?: React.CSSProperties;
}

export const PropertiesPanel: React.FC<PropertiesPanelProps> = ({
  finding,
  noteText,
  onNoteChange,
  onNoteBlur,
  selectedMarkerId,
  onSelectMarker,
  onDeleteMarker,
  onUpdateMarkerComment,
  onDeleteFinding,
  className = '',
  style,
}) => {
  const [editingComments, setEditingComments] = useState<Record<string, string>>({});

  if (!finding) {
    return (
      <div
        data-testid="properties-panel"
        className={`properties-panel ${className}`.trim()}
        style={{
          width: '440px',
          minWidth: '440px',
          maxWidth: '440px',
          height: '100%',
          backgroundColor: 'var(--color-surface)',
          borderLeft: '1px solid var(--color-border)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: 'var(--color-text-muted)',
          fontSize: 'var(--text-sm)',
          fontFamily: 'var(--font-ui)',
          ...style,
        }}
      >
        No finding selected
      </div>
    );
  }

  const markers = finding.markers || [];
  const markerNotes = markers.map(
    (m) => editingComments[m.id] !== undefined ? editingComments[m.id] : (m.comment || '')
  );

  return (
    <div
      data-testid="properties-panel"
      className={`properties-panel ${className}`.trim()}
      style={{
        width: '440px',
        minWidth: '440px',
        maxWidth: '440px',
        height: '100%',
        backgroundColor: 'var(--color-surface)',
        borderLeft: '1px solid var(--color-border)',
        display: 'flex',
        flexDirection: 'column',
        boxSizing: 'border-box',
        overflow: 'hidden',
        flexShrink: 0,
        ...style,
      }}
    >
      {/* Panel Header */}
      <div
        style={{
          padding: 'var(--space-3) var(--space-4)',
          borderBottom: '1px solid var(--color-border)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          backgroundColor: 'var(--color-surface)',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
          <span style={{ fontSize: '1rem' }}>📝</span>
          <span
            style={{
              fontSize: 'var(--text-xs)',
              fontWeight: 800,
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
              color: 'var(--color-text)',
            }}
          >
            Notes & Markers
          </span>
        </div>
        <span
          data-testid="finding-id-pill"
          style={{
            fontSize: 'var(--text-2xs)',
            fontFamily: 'var(--font-mono)',
            backgroundColor: 'var(--color-surface-sunken)',
            padding: '2px 8px',
            borderRadius: 'var(--radius-sm)',
            color: 'var(--color-text-secondary)',
            fontWeight: 600,
          }}
        >
          {finding.finding.id.slice(0, 8)}…
        </span>
      </div>

      {/* Scrollable Body */}
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
        {/* Section 1: Fixed Height ~130px Observation Summary */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-1)' }}>
          <label
            htmlFor="observation-summary-input"
            style={{
              fontSize: 'var(--text-2xs)',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.04em',
              color: 'var(--color-text-muted)',
            }}
          >
            Observation Summary
          </label>
          <TextArea
            id="observation-summary-input"
            data-testid="observation-summary-textarea"
            value={noteText}
            onChange={(e) => onNoteChange(e.target.value)}
            onBlur={onNoteBlur}
            placeholder="Tuliskan ringkasan keseluruhan temuan di sini..."
            rows={5}
            style={{
              height: '130px',
              minHeight: '130px',
              maxHeight: '130px',
              fontFamily: 'var(--font-ui)',
              fontSize: 'var(--text-sm)',
              lineHeight: '1.5',
              resize: 'none',
            }}
          />
        </div>

        {/* Section 2: Step Marker Notes (Catatan langsung di tiap Step Marker 1, 2, 3...) */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              fontSize: 'var(--text-2xs)',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.04em',
              color: 'var(--color-text-muted)',
            }}
          >
            <span>Step Marker Notes ({markers.length})</span>
          </div>

          {markers.length === 0 ? (
            <div
              style={{
                padding: 'var(--space-3)',
                backgroundColor: 'var(--color-surface-sunken)',
                borderRadius: 'var(--radius-sm)',
                color: 'var(--color-text-muted)',
                fontSize: 'var(--text-xs)',
                fontStyle: 'italic',
                textAlign: 'center',
              }}
            >
              Klik pada gambar canvas untuk menempelkan marker nomor (1, 2, 3...)
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              {markers
                .slice()
                .sort((a, b) => a.ordinal - b.ordinal)
                .map((m) => {
                  const isSelected = m.id === selectedMarkerId;
                  const currentComment =
                    editingComments[m.id] !== undefined ? editingComments[m.id] : (m.comment || '');

                  return (
                    <div
                      key={m.id}
                      data-testid={`marker-note-row-${m.ordinal}`}
                      onClick={() => onSelectMarker(m.id)}
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 'var(--space-1)',
                        padding: 'var(--space-3)',
                        backgroundColor: isSelected
                          ? 'var(--color-accent-subtle)'
                          : 'var(--color-surface-sunken)',
                        border: isSelected
                          ? '1px solid var(--color-accent)'
                          : '1px solid var(--color-border)',
                        borderRadius: 'var(--radius-sm)',
                        transition: 'all 0.15s ease',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
                          <MarkerBadge number={m.ordinal} isSelected={isSelected} />
                          <span
                            style={{
                              fontSize: 'var(--text-xs)',
                              fontFamily: 'var(--font-mono)',
                              color: 'var(--color-text-secondary)',
                              fontWeight: 700,
                            }}
                          >
                            Marker #{m.ordinal}
                          </span>
                          <span
                            style={{
                              fontSize: 'var(--text-2xs)',
                              fontFamily: 'var(--font-mono)',
                              color: 'var(--color-text-muted)',
                            }}
                          >
                            ({Math.round(m.x * 100)}%, {Math.round(m.y * 100)}%)
                          </span>
                        </div>

                        <button
                          type="button"
                          data-testid={`delete-marker-btn-${m.ordinal}`}
                          onClick={(e) => {
                            e.stopPropagation();
                            onDeleteMarker(m.id);
                          }}
                          style={{
                            background: 'none',
                            border: 'none',
                            color: 'var(--color-text-muted)',
                            cursor: 'pointer',
                            padding: '2px 4px',
                            fontSize: 'var(--text-xs)',
                            lineHeight: 1,
                          }}
                          title={`Delete Marker ${m.ordinal}`}
                        >
                          🗑️
                        </button>
                      </div>

                      {/* Direct Editable Textarea per Step Marker */}
                      <textarea
                        data-testid={`marker-comment-input-${m.ordinal}`}
                        value={currentComment}
                        placeholder={`Tulis catatan untuk marker #${m.ordinal}...`}
                        onChange={(e) => {
                          const val = e.target.value;
                          setEditingComments((prev) => ({ ...prev, [m.id]: val }));
                        }}
                        onBlur={() => {
                          if (onUpdateMarkerComment && editingComments[m.id] !== undefined) {
                            onUpdateMarkerComment(m.id, editingComments[m.id]);
                          }
                        }}
                        rows={2}
                        style={{
                          width: '100%',
                          minHeight: '44px',
                          padding: 'var(--space-1) var(--space-2)',
                          fontFamily: 'var(--font-ui)',
                          fontSize: 'var(--text-xs)',
                          lineHeight: '1.4',
                          borderRadius: 'var(--radius-xs)',
                          border: '1px solid var(--color-border)',
                          backgroundColor: 'var(--color-surface)',
                          color: 'var(--color-text)',
                          resize: 'vertical',
                          boxSizing: 'border-box',
                          outline: 'none',
                        }}
                      />
                    </div>
                  );
                })}
            </div>
          )}
        </div>

        {/* Section 3: Live Multimodal Token Estimator */}
        <TokenEstimator
          imageWidth={finding.finding.image_width}
          imageHeight={finding.finding.image_height}
          summaryText={noteText}
          markerNotes={markerNotes}
        />
      </div>

      {/* Panel Footer: Delete Finding Action */}
      {onDeleteFinding && (
        <div
          style={{
            padding: 'var(--space-3) var(--space-4)',
            borderTop: '1px solid var(--color-border)',
            backgroundColor: 'var(--color-surface)',
          }}
        >
          <button
            type="button"
            data-testid="delete-current-finding-btn"
            onClick={onDeleteFinding}
            style={{
              width: '100%',
              padding: 'var(--space-2) 0',
              backgroundColor: 'transparent',
              color: 'var(--color-danger)',
              border: '1px solid var(--color-danger-bg)',
              borderRadius: 'var(--radius-sm)',
              fontWeight: 600,
              fontSize: 'var(--text-xs)',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 'var(--space-2)',
              transition: 'background-color 0.15s ease',
            }}
          >
            <span>🗑️</span>
            <span>Delete Screenshot from Queue</span>
          </button>
        </div>
      )}
    </div>
  );
};
