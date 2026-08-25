import React from 'react';

export interface MarkerBadgeProps {
  number: number;
  isDragging?: boolean;
  isSelected?: boolean;
  isHovered?: boolean;
  style?: React.CSSProperties;
  className?: string;
  tabIndex?: number;
  onClick?: (e: React.MouseEvent<HTMLDivElement>) => void;
  onKeyDown?: (e: React.KeyboardEvent<HTMLDivElement>) => void;
  'aria-label'?: string;
}

export const MarkerBadge: React.FC<MarkerBadgeProps> = ({
  number,
  isDragging = false,
  isSelected = false,
  isHovered = false,
  style,
  className = '',
  tabIndex,
  onClick,
  onKeyDown,
  'aria-label': ariaLabel,
}) => {
  const displayNum = Number.isFinite(number)
    ? Math.min(Math.max(Math.round(number), 1), 99)
    : 1;

  const ringShadow = isSelected
    ? '0 0 0 3px var(--color-annotation-handle-bg), 0 2px 8px var(--color-overlay-shadow-card)'
    : isHovered
    ? '0 0 0 2px var(--color-annotation-handle-bg), 0 2px 6px var(--color-overlay-shadow-card)'
    : '0 2px 4px var(--color-overlay-shadow-card)';

  return (
    <div
      role="button"
      tabIndex={tabIndex}
      aria-label={ariaLabel || `Marker ${displayNum}`}
      onClick={onClick}
      onKeyDown={onKeyDown}
      className={`marker-badge ${className}`.trim()}
      style={{
        width: '1.75rem',
        height: '1.75rem',
        borderRadius: '50%',
        backgroundColor: 'var(--color-marker)',
        color: 'var(--color-marker-text)',
        boxShadow: ringShadow,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--text-xs)',
        fontWeight: 700,
        userSelect: 'none',
        opacity: isDragging ? 0.85 : 1,
        cursor: isDragging ? 'grabbing' : 'pointer',
        flexShrink: 0,
        outline: 'none',
        transition: 'box-shadow 0.15s ease, opacity 0.15s ease, transform 0.1s ease',
        ...style,
      }}
    >
      {displayNum}
    </div>
  );
};
