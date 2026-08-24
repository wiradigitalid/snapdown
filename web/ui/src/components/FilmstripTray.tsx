import React from 'react';
import { FindingDetailItemDto } from './FindingsEditor';
import { Checkbox } from './Checkbox';

export interface FilmstripTrayProps {
  findings: FindingDetailItemDto[];
  activeFindingId: string | null;
  selectedFindingIds: Set<string>;
  onSelectActiveFinding: (id: string) => void;
  onToggleSelectFinding: (id: string, e: React.MouseEvent) => void;
  onAssembleBatch: () => void;
  className?: string;
  style?: React.CSSProperties;
}

export function formatFilmstripTime(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch {
    return isoString;
  }
}

export const FilmstripTray: React.FC<FilmstripTrayProps> = ({
  findings,
  activeFindingId,
  selectedFindingIds,
  onSelectActiveFinding,
  onToggleSelectFinding,
  onAssembleBatch,
  className = '',
  style,
}) => {
  return (
    <div
      data-testid="filmstrip-tray"
      className={`filmstrip-tray ${className}`.trim()}
      style={{
        height: '110px',
        backgroundColor: 'var(--snagit-tray-bg)',
        borderTop: '1px solid var(--color-border)',
        display: 'flex',
        alignItems: 'center',
        padding: '0 var(--space-3)',
        gap: 'var(--space-3)',
        overflowX: 'auto',
        overflowY: 'hidden',
        boxSizing: 'border-box',
        userSelect: 'none',
        flexShrink: 0,
        ...style,
      }}
    >
      {/* Left Docked Batcher Banner */}
      <div
        data-testid="filmstrip-batcher-banner"
        style={{
          width: '130px',
          minWidth: '130px',
          height: '84px',
          backgroundColor: 'var(--color-surface)',
          border: '1px solid var(--color-border)',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          padding: 'var(--space-2)',
          gap: 'var(--space-1)',
          flexShrink: 0,
        }}
      >
        <span
          data-testid="filmstrip-selected-count"
          style={{
            fontSize: 'var(--text-xs)',
            fontWeight: 700,
            color: 'var(--color-text)',
          }}
        >
          {selectedFindingIds.size} selected
        </span>
        <button
          type="button"
          data-testid="filmstrip-assemble-btn"
          onClick={onAssembleBatch}
          disabled={selectedFindingIds.size === 0}
          style={{
            width: '100%',
            padding: 'var(--space-1) 0',
            backgroundColor: selectedFindingIds.size > 0 ? 'var(--color-accent)' : 'var(--color-surface-sunken)',
            color: selectedFindingIds.size > 0 ? 'var(--color-accent-text)' : 'var(--color-text-dim)',
            border: 'none',
            borderRadius: 'var(--radius-xs)',
            fontSize: 'var(--text-2xs)',
            fontWeight: 700,
            cursor: selectedFindingIds.size > 0 ? 'pointer' : 'not-allowed',
            transition: 'background-color 0.15s ease',
          }}
        >
          📦 Assemble
        </button>
      </div>

      {/* Horizontal Filmstrip Cards */}
      <div
        data-testid="filmstrip-scroll-area"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 'var(--space-3)',
          height: '100%',
          flex: 1,
          overflowX: 'auto',
          overflowY: 'hidden',
          paddingRight: 'var(--space-4)',
        }}
      >
        {findings.map((f) => {
          const isActive = f.finding.id === activeFindingId;
          const isSelected = selectedFindingIds.has(f.finding.id);

          return (
            <div
              key={f.finding.id}
              data-testid={`filmstrip-card-${f.finding.id}`}
              onClick={() => onSelectActiveFinding(f.finding.id)}
              style={{
                width: '136px',
                minWidth: '136px',
                height: '84px',
                backgroundColor: 'var(--color-surface)',
                border: isActive
                  ? '2px solid var(--color-accent)'
                  : '1px solid var(--color-border)',
                borderRadius: 'var(--radius-sm)',
                cursor: 'pointer',
                position: 'relative',
                display: 'flex',
                flexDirection: 'column',
                overflow: 'hidden',
                boxShadow: isActive ? 'var(--shadow-md)' : 'none',
                transition: 'all 0.15s ease',
              }}
            >
              {/* Thumbnail Container */}
              <div
                style={{
                  width: '100%',
                  height: '58px',
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
                    alt={`Thumb ${f.finding.id}`}
                    style={{
                      width: '100%',
                      height: '100%',
                      objectFit: 'cover',
                    }}
                  />
                ) : (
                  <span
                    style={{
                      fontSize: 'var(--text-2xs)',
                      fontFamily: 'var(--font-mono)',
                      color: 'var(--color-text-dim)',
                    }}
                  >
                    {f.finding.image_width}×{f.finding.image_height}
                  </span>
                )}

                {/* Top-Left Selection Checkbox */}
                <div
                  data-testid={`filmstrip-card-checkbox-${f.finding.id}`}
                  onClick={(e) => onToggleSelectFinding(f.finding.id, e)}
                  style={{
                    position: 'absolute',
                    top: '2px',
                    left: '2px',
                    zIndex: 10,
                    backgroundColor: 'var(--color-surface)',
                    borderRadius: '2px',
                    padding: '1px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    boxShadow: 'var(--shadow-sm)',
                  }}
                >
                  <Checkbox
                    checked={isSelected}
                    onChange={() => {}}
                    aria-label={`Select item ${f.finding.id}`}
                  />
                </div>

                {/* Top-Right Step Marker Count Badge */}
                {f.markers.length > 0 && (
                  <div
                    style={{
                      position: 'absolute',
                      top: '2px',
                      right: '2px',
                      zIndex: 10,
                      backgroundColor: 'var(--color-marker)',
                      color: 'var(--color-marker-text)',
                      borderRadius: 'var(--radius-full)',
                      padding: '1px 5px',
                      fontSize: 'var(--text-2xs)',
                      fontWeight: 800,
                      fontFamily: 'var(--font-mono)',
                    }}
                  >
                    🟡 {f.markers.length}
                  </div>
                )}

                {/* Floating Active Editing Tag */}
                {isActive && (
                  <div
                    data-testid="active-editing-pill"
                    style={{
                      position: 'absolute',
                      bottom: '2px',
                      left: '50%',
                      transform: 'translateX(-50%)',
                      backgroundColor: 'var(--color-success-bg)',
                      color: 'var(--color-success-text)',
                      fontSize: '0.6rem',
                      fontWeight: 800,
                      padding: '1px 6px',
                      borderRadius: 'var(--radius-full)',
                      whiteSpace: 'nowrap',
                      zIndex: 10,
                    }}
                  >
                    ● Editing
                  </div>
                )}
              </div>

              {/* Card Footer Info */}
              <div
                style={{
                  height: '26px',
                  padding: '0 var(--space-2)',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  fontSize: 'var(--text-2xs)',
                  fontFamily: 'var(--font-mono)',
                  color: isActive ? 'var(--color-accent)' : 'var(--color-text-muted)',
                  borderTop: '1px solid var(--color-border)',
                }}
              >
                <span>{formatFilmstripTime(f.finding.captured_at)}</span>
                <span>{f.finding.image_width}×{f.finding.image_height}</span>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
