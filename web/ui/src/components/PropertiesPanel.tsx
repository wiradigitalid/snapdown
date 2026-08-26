import React, { useState, useEffect } from 'react';
import { FindingDetailItemDto } from './FindingsEditor';
import { MarkerBadge } from './MarkerBadge';
import { TokenEstimator } from './TokenEstimator';
import { TextArea } from './TextArea';
import {
  VisualAnnotationItem,
  VisualCalloutAnnotation,
  VisualTextAnnotation,
} from './types/annotation';

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

export type PropertiesTab = 'notes' | 'properties';

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
  const [activeTab, setActiveTab] = useState<PropertiesTab>('notes');
  const [editingComments, setEditingComments] = useState<Record<string, string>>({});

  const selectedAnnotation = annotations.find((a) => a.id === selectedAnnotationId);

  // An element is considered to have configurable properties if it is selected and has typography/style options
  const hasElementProperties = Boolean(
    selectedAnnotation &&
      (selectedAnnotation.kind === 'text' || selectedAnnotation.kind === 'callout')
  );

  // Auto-switch to Properties tab when an editable element is selected, and fallback to Notes if deselected
  useEffect(() => {
    if (hasElementProperties) {
      setActiveTab('properties');
    } else {
      setActiveTab('notes');
    }
  }, [hasElementProperties, selectedAnnotationId]);

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
    (m) => (editingComments[m.id] !== undefined ? editingComments[m.id] : m.comment || '')
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
      {/* Top Segmented Tab Navigation Header */}
      <div
        data-testid="properties-tabs"
        style={{
          display: 'flex',
          borderBottom: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-surface-sunken)',
          padding: '0 var(--space-3)',
          gap: 'var(--space-2)',
          flexShrink: 0,
        }}
      >
        <button
          type="button"
          data-testid="tab-notes-btn"
          onClick={() => setActiveTab('notes')}
          style={{
            flex: 1,
            padding: '12px var(--space-2)',
            backgroundColor: 'transparent',
            color: activeTab === 'notes' ? 'var(--color-accent)' : 'var(--color-text-muted)',
            border: 'none',
            borderBottom: activeTab === 'notes' ? '2.5px solid var(--color-accent)' : '2.5px solid transparent',
            fontSize: 'var(--text-xs)',
            fontWeight: 600,
            cursor: 'pointer',
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            transition: 'color 0.15s ease, border-color 0.15s ease',
          }}
        >
          <span>📋</span>
          <span>Notes</span>
        </button>

        <button
          type="button"
          data-testid="tab-element-properties-btn"
          disabled={!hasElementProperties}
          onClick={() => {
            if (hasElementProperties) {
              setActiveTab('properties');
            }
          }}
          style={{
            flex: 1,
            padding: '12px var(--space-2)',
            backgroundColor: 'transparent',
            color:
              !hasElementProperties
                ? 'var(--color-text-dim)'
                : activeTab === 'properties'
                ? 'var(--color-accent)'
                : 'var(--color-text-muted)',
            border: 'none',
            borderBottom:
              activeTab === 'properties' && hasElementProperties
                ? '2.5px solid var(--color-accent)'
                : '2.5px solid transparent',
            fontSize: 'var(--text-xs)',
            fontWeight: 600,
            cursor: hasElementProperties ? 'pointer' : 'not-allowed',
            opacity: hasElementProperties ? 1 : 0.4,
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: '8px',
            transition: 'color 0.15s ease, border-color 0.15s ease, opacity 0.15s ease',
          }}
          title={
            hasElementProperties
              ? 'Atur properti elemen aktif'
              : 'Aktif saat elemen teks atau callout disorot'
          }
        >
          <span>🎨</span>
          <span>Properties</span>
        </button>
      </div>

      {/* Tab Content Body (Scrollable) */}
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
        {activeTab === 'notes' ? (
          /* TAB 1: NOTES (Observation summary, step marker notes, estimated llm cost) */
          <>
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
                        editingComments[m.id] !== undefined ? editingComments[m.id] : m.comment || '';

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
                                  color: 'var(--color-text-dim)',
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

            {/* Section 3: Estimated LLM Cost Token Estimator */}
            <TokenEstimator
              imageWidth={finding.finding.image_width}
              imageHeight={finding.finding.image_height}
              summaryText={noteText}
              markerNotes={markerNotes}
            />
          </>
        ) : (
          /* TAB 2: PROPERTIES */
          <div data-testid="element-properties-tab" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
            {selectedAnnotation && (selectedAnnotation.kind === 'text' || selectedAnnotation.kind === 'callout') ? (
              /* Active Text / Callout Inspector */
              <div
                data-testid="annotation-style-inspector"
                style={{
                  padding: 'var(--space-4)',
                  backgroundColor: 'var(--color-surface-sunken)',
                  border: '1px solid var(--color-border)',
                  borderRadius: 'var(--radius-md)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 'var(--space-4)',
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
                    🎨 {selectedAnnotation.kind === 'callout' ? 'Callout Properties' : 'Text Typography'}
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
                    Delete Element
                  </button>
                </div>

                {/* Font Family Selector */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
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
                      padding: '6px 8px',
                      borderRadius: 'var(--radius-sm)',
                      border: '1px solid var(--color-border)',
                      backgroundColor: 'var(--color-surface)',
                      color: 'var(--color-text)',
                      fontSize: 'var(--text-xs)',
                      outline: 'none',
                      cursor: 'pointer',
                    }}
                  >
                    <option value="Inter, sans-serif">Inter (Modern Clean)</option>
                    <option value="'JetBrains Mono', monospace">JetBrains Mono (Code/Mono)</option>
                    <option value="'Playfair Display', serif">Playfair Display (Serif)</option>
                    <option value="'Plus Jakarta Sans', sans-serif">Plus Jakarta Sans (UI)</option>
                    <option value="Impact, sans-serif">Impact (Bold Heading)</option>
                    <option value="'Comic Sans MS', cursive, sans-serif">Casual / Sketch</option>
                  </select>
                </div>

                {/* Font Size Slider & Readout */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <label
                      style={{
                        fontSize: 'var(--text-2xs)',
                        color: 'var(--color-text-muted)',
                        fontWeight: 700,
                      }}
                    >
                      Font Size
                    </label>
                    <span style={{ fontSize: 'var(--text-xs)', fontFamily: 'var(--font-mono)', fontWeight: 700 }}>
                      {selectedAnnotation.fontSize || 14}px
                    </span>
                  </div>
                  <input
                    type="range"
                    min={10}
                    max={48}
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

                {/* Style Toggles (Bold / Italic) with High-Contrast Active State Indication */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                  <label
                    style={{
                      fontSize: 'var(--text-2xs)',
                      color: 'var(--color-text-muted)',
                      fontWeight: 700,
                    }}
                  >
                    Font Weight & Style
                  </label>
                  <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
                    {(() => {
                      const isBold =
                        selectedAnnotation.fontWeight === 'bold' ||
                        selectedAnnotation.fontWeight === '700';
                      return (
                        <button
                          type="button"
                          onClick={() => {
                            onUpdateAnnotation?.({
                              ...selectedAnnotation,
                              fontWeight: isBold ? 'normal' : 'bold',
                            } as VisualCalloutAnnotation | VisualTextAnnotation);
                          }}
                          style={{
                            flex: 1,
                            height: '36px',
                            borderRadius: 'var(--radius-sm)',
                            border: isBold
                              ? '1.5px solid var(--color-accent)'
                              : '1px solid var(--color-border)',
                            backgroundColor: isBold
                              ? 'var(--color-accent)'
                              : 'var(--color-surface)',
                            color: isBold
                              ? 'var(--color-accent-text)'
                              : 'var(--color-text)',
                            fontWeight: 800,
                            cursor: 'pointer',
                            display: 'inline-flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            gap: '6px',
                            boxShadow: isBold ? 'var(--shadow-sm)' : 'none',
                            transition: 'all 0.15s ease',
                          }}
                        >
                          <span>{isBold ? '✓' : ''}</span>
                          <span>Bold</span>
                        </button>
                      );
                    })()}

                    {(() => {
                      const isItalic = selectedAnnotation.fontStyle === 'italic';
                      return (
                        <button
                          type="button"
                          onClick={() => {
                            onUpdateAnnotation?.({
                              ...selectedAnnotation,
                              fontStyle: isItalic ? 'normal' : 'italic',
                            } as VisualCalloutAnnotation | VisualTextAnnotation);
                          }}
                          style={{
                            flex: 1,
                            height: '36px',
                            borderRadius: 'var(--radius-sm)',
                            border: isItalic
                              ? '1.5px solid var(--color-accent)'
                              : '1px solid var(--color-border)',
                            backgroundColor: isItalic
                              ? 'var(--color-accent)'
                              : 'var(--color-surface)',
                            color: isItalic
                              ? 'var(--color-accent-text)'
                              : 'var(--color-text)',
                            fontStyle: 'italic',
                            fontWeight: 700,
                            cursor: 'pointer',
                            display: 'inline-flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            gap: '6px',
                            boxShadow: isItalic ? 'var(--shadow-sm)' : 'none',
                            transition: 'all 0.15s ease',
                          }}
                        >
                          <span>{isItalic ? '✓' : ''}</span>
                          <span>Italic</span>
                        </button>
                      );
                    })()}
                  </div>
                </div>
              </div>
            ) : (
              /* No Configurable Element Selected Empty State */
              <div
                data-testid="element-properties-empty-state"
                style={{
                  padding: 'var(--space-6)',
                  backgroundColor: 'var(--color-surface-sunken)',
                  borderRadius: 'var(--radius-md)',
                  border: '1px dashed var(--color-border)',
                  textAlign: 'center',
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: 'var(--space-2)',
                  color: 'var(--color-text-muted)',
                }}
              >
                <span style={{ fontSize: '1.5rem' }}>🎯</span>
                <span style={{ fontWeight: 700, color: 'var(--color-text)' }}>
                  Tidak ada elemen dengan properti
                </span>
                <span style={{ fontSize: 'var(--text-xs)' }}>
                  Sorot elemen Teks atau Callout pada canvas untuk mengubah font family, ukuran font, atau gaya teks.
                </span>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
