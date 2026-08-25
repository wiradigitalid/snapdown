import React, { useState } from 'react';
import { FindingDetailItemDto } from './FindingsEditor';
import { MarkerBadge } from './MarkerBadge';
import { TokenEstimator } from './TokenEstimator';
import { TextArea } from './TextArea';
import { VisualAnnotationItem, VisualCalloutAnnotation, VisualTextAnnotation } from './types/annotation';

export interface PropertiesPanelProps {
  finding: FindingDetailItemDto | null;
  noteText: string;
  onNoteChange: (text: string) => void;
  onNoteBlur?: () => void;
  onSaveNote?: (text: string) => void;
  selectedMarkerId: string | null;
  selectedAnnotationId?: string | null;
  annotations?: VisualAnnotationItem[];
  onSelectMarker: (markerId: string | null) => void;
  onDeleteMarker: (markerId: string) => void;
  onUpdateMarkerComment?: (markerId: string, comment: string) => void;
  onUpdateAnnotation?: (annotation: VisualAnnotationItem) => void;
  onDeleteAnnotation?: (id: string) => void;
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
  selectedAnnotationId,
  annotations = [],
  onSelectMarker,
  onDeleteMarker,
  onUpdateMarkerComment,
  onUpdateAnnotation,
  onDeleteAnnotation,
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
  const selectedAnnotation = annotations.find((a) => a.id === selectedAnnotationId);

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
            Notes & Properties
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
        {/* Contextual Annotation Style Inspector (When a Text or Callout is Selected) */}
        {selectedAnnotation && (selectedAnnotation.kind === 'text' || selectedAnnotation.kind === 'callout') && (
          <div
            data-testid="annotation-style-inspector"
            style={{
              padding: 'var(--space-3)',
              backgroundColor: 'var(--color-surface-sunken)',
              border: '1px solid var(--color-border-strong, var(--color-primary))',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span
                style={{
                  fontSize: 'var(--text-xs)',
                  fontWeight: 800,
                  color: 'var(--color-text)',
                  textTransform: 'uppercase',
                  letterSpacing: '0.04em',
                }}
              >
                🎨 {selectedAnnotation.kind === 'callout' ? 'Callout Style' : 'Text Typography'}
              </span>
              <button
                type="button"
                onClick={() => onDeleteAnnotation?.(selectedAnnotation.id)}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--color-danger)',
                  cursor: 'pointer',
                  fontSize: 'var(--text-xs)',
                  fontWeight: 600,
                }}
              >
                Delete
              </button>
            </div>

            {/* Font Family Selector */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
              <label
                style={{
                  fontSize: 'var(--text-2xs)',
                  color: 'var(--color-text-muted)',
                  fontWeight: 700,
                }}
              >
                Font Family
              </label>
              <select
                value={selectedAnnotation.fontFamily || 'Inter, sans-serif'}
                onChange={(e) => {
                  if (onUpdateAnnotation) {
                    onUpdateAnnotation({
                      ...selectedAnnotation,
                      fontFamily: e.target.value,
                    } as VisualCalloutAnnotation | VisualTextAnnotation);
                  }
                }}
                style={{
                  padding: '4px 8px',
                  borderRadius: 'var(--radius-sm)',
                  border: '1px solid var(--color-border)',
                  backgroundColor: 'var(--color-surface)',
                  color: 'var(--color-text)',
                  fontSize: 'var(--text-xs)',
                  outline: 'none',
                  cursor: 'pointer',
                }}
              >
                <option value="Inter, sans-serif">Inter (Clean Modern)</option>
                <option value="'JetBrains Mono', monospace">JetBrains Mono (Code/Mono)</option>
                <option value="'Playfair Display', serif">Playfair Display (Editorial Serif)</option>
                <option value="'Plus Jakarta Sans', sans-serif">Plus Jakarta Sans (UI Sans)</option>
                <option value="Impact, sans-serif">Impact (Bold Heading)</option>
                <option value="'Comic Sans MS', cursive, sans-serif">Comic / Casual</option>
              </select>
            </div>

            {/* Font Size & Weight Row */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-3)' }}>
              <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: '4px' }}>
                <label
                  style={{
                    fontSize: 'var(--text-2xs)',
                    color: 'var(--color-text-muted)',
                    fontWeight: 700,
                  }}
                >
                  Font Size ({selectedAnnotation.fontSize || 14}px)
                </label>
                <input
                  type="range"
                  min={10}
                  max={36}
                  step={1}
                  value={selectedAnnotation.fontSize || 14}
                  onChange={(e) => {
                    if (onUpdateAnnotation) {
                      onUpdateAnnotation({
                        ...selectedAnnotation,
                        fontSize: Number(e.target.value),
                      } as VisualCalloutAnnotation | VisualTextAnnotation);
                    }
                  }}
                  style={{ width: '100%', cursor: 'pointer' }}
                />
              </div>

              {/* Bold & Italic Toggles */}
              <div style={{ display: 'flex', alignItems: 'flex-end', gap: '4px', height: '100%' }}>
                <button
                  type="button"
                  data-tooltip="Bold"
                  onClick={() => {
                    const isBold = selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700';
                    onUpdateAnnotation?.({
                      ...selectedAnnotation,
                      fontWeight: isBold ? 'normal' : 'bold',
                    } as VisualCalloutAnnotation | VisualTextAnnotation);
                  }}
                  style={{
                    width: '32px',
                    height: '32px',
                    borderRadius: 'var(--radius-sm)',
                    border: '1px solid var(--color-border)',
                    backgroundColor:
                      selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700'
                        ? 'var(--color-primary)'
                        : 'var(--color-surface)',
                    color:
                      selectedAnnotation.fontWeight === 'bold' || selectedAnnotation.fontWeight === '700'
                        ? 'var(--color-accent-text)'
                        : 'var(--color-text)',
                    fontWeight: 700,
                    cursor: 'pointer',
                  }}
                >
                  B
                </button>
                <button
                  type="button"
                  data-tooltip="Italic"
                  onClick={() => {
                    const isItalic = selectedAnnotation.fontStyle === 'italic';
                    onUpdateAnnotation?.({
                      ...selectedAnnotation,
                      fontStyle: isItalic ? 'normal' : 'italic',
                    } as VisualCalloutAnnotation | VisualTextAnnotation);
                  }}
                  style={{
                    width: '32px',
                    height: '32px',
                    borderRadius: 'var(--radius-sm)',
                    border: '1px solid var(--color-border)',
                    backgroundColor:
                      selectedAnnotation.fontStyle === 'italic'
                        ? 'var(--color-primary)'
                        : 'var(--color-surface)',
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
              </div>
            </div>
          </div>
        )}

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
                            padding: '4px',
                            fontSize: '12px',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                          }}
                          title="Hapus marker ini"
                        >
                          ✕
                        </button>
                      </div>

                      <input
                        type="text"
                        data-testid={`marker-comment-input-${m.ordinal}`}
                        value={currentComment}
                        placeholder={`Catatan spesifik untuk titik ${m.ordinal}...`}
                        onChange={(e) => {
                          const val = e.target.value;
                          setEditingComments((prev) => ({ ...prev, [m.id]: val }));
                        }}
                        onBlur={() => {
                          if (onUpdateMarkerComment && editingComments[m.id] !== undefined) {
                            onUpdateMarkerComment(m.id, editingComments[m.id]);
                          }
                        }}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            e.currentTarget.blur();
                          }
                        }}
                        style={{
                          width: '100%',
                          padding: '6px 8px',
                          fontSize: 'var(--text-xs)',
                          fontFamily: 'var(--font-ui)',
                          backgroundColor: 'var(--color-surface)',
                          color: 'var(--color-text)',
                          border: '1px solid var(--color-border)',
                          borderRadius: 'var(--radius-sm)',
                          outline: 'none',
                          boxSizing: 'border-box',
                        }}
                      />
                    </div>
                  );
                })}
            </div>
          )}
        </div>
      </div>

      {/* Token Estimator Footer */}
      <TokenEstimator
        imageWidth={finding.finding.image_width}
        imageHeight={finding.finding.image_height}
        summaryText={noteText}
        markerNotes={finding.markers.map((m) => m.comment)}
      />
    </div>
  );
};
