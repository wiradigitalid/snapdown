import React from 'react';

export interface MarkerBadgeProps {
  number: number;
  isDragging?: boolean;
  style?: React.CSSProperties;
}

export const MarkerBadge: React.FC<MarkerBadgeProps> = ({
  number,
  isDragging = false,
  style,
}) => {
  const displayNum = Number.isFinite(number)
    ? Math.min(Math.max(Math.round(number), 1), 99)
    : 1;

  return (
    <div
      style={{
        width: '1.5rem',
        height: '1.5rem',
        borderRadius: '50%',
        backgroundColor: 'var(--color-marker)',
        color: 'var(--color-marker-text)',
        boxShadow: `0 0 0 2px var(--color-marker-ring)`,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontFamily: 'var(--font-mono)',
        fontSize: 'var(--text-xs)',
        fontWeight: 700,
        userSelect: 'none',
        opacity: isDragging ? 0.75 : 1,
        cursor: isDragging ? 'grabbing' : 'default',
        flexShrink: 0,
        ...style,
      }}
    >
      {displayNum}
    </div>
  );
};
