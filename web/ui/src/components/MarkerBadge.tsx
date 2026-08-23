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

  const ringThickness = isSelected || isHovered ? '3px' : '2px';

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
        boxShadow: `0 0 0 ${ringThickness} var(--color-marker-ring)`,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--text-xs)',
        fontWeight: 700,
        userSelect: 'none',
        opacity: isDragging ? 0.8 : 1,
        cursor: isDragging ? 'grabbing' : 'pointer',
        flexShrink: 0,
        outline: 'none',
        transition: 'box-shadow 0.15s ease, opacity 0.15s ease',
        ...style,
      }}
    >
      {displayNum}
    </div>
  );
};
