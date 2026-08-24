import React from 'react';

export interface TokenBreakdown {
  imageTokens: number;
  textTokens: number;
  totalTokens: number;
  imageDimensions?: { width: number; height: number };
  characterCount: number;
}

export interface TokenEstimatorProps {
  imageWidth?: number;
  imageHeight?: number;
  summaryText?: string;
  markerNotes?: string[];
  totalFindingCount?: number;
  className?: string;
  style?: React.CSSProperties;
}

export function calculateTokens(
  width: number = 0,
  height: number = 0,
  summary: string = '',
  markerNotes: string[] = []
): TokenBreakdown {
  // Vision model grid calculation: ceil((width * height) / 750)
  const imageTokens = width > 0 && height > 0 ? Math.ceil((width * height) / 750) : 0;

  // Text character calculation: ceil(total_chars / 3.8)
  const totalText = `${summary} ${markerNotes.join(' ')}`.trim();
  const characterCount = totalText.length;
  const textTokens = characterCount > 0 ? Math.ceil(characterCount / 3.8) : 0;

  return {
    imageTokens,
    textTokens,
    totalTokens: imageTokens + textTokens,
    imageDimensions: width > 0 && height > 0 ? { width, height } : undefined,
    characterCount,
  };
}

export const TokenEstimator: React.FC<TokenEstimatorProps> = ({
  imageWidth = 1600,
  imageHeight = 900,
  summaryText = '',
  markerNotes = [],
  totalFindingCount,
  className = '',
  style,
}) => {
  const breakdown = calculateTokens(imageWidth, imageHeight, summaryText, markerNotes);

  return (
    <div
      data-testid="token-estimator"
      className={`token-estimator ${className}`.trim()}
      style={{
        backgroundColor: 'var(--color-surface-sunken)',
        border: '1px solid var(--color-border)',
        borderRadius: 'var(--radius-sm)',
        padding: 'var(--space-3)',
        fontSize: 'var(--text-xs)',
        fontFamily: 'var(--font-ui)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-2)',
        ...style,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          fontWeight: 700,
          color: 'var(--color-text-secondary)',
          textTransform: 'uppercase',
          letterSpacing: '0.04em',
        }}
      >
        <span>🪙 Estimated LLM Cost</span>
        <span
          data-testid="total-tokens-badge"
          style={{
            fontFamily: 'var(--font-mono)',
            color: 'var(--color-accent)',
            fontWeight: 800,
          }}
        >
          ~{breakdown.totalTokens.toLocaleString()} tk
        </span>
      </div>

      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--space-1)',
          color: 'var(--color-text-muted)',
          fontFamily: 'var(--font-mono)',
          fontSize: 'var(--text-2xs)',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span>🖼️ Image ({imageWidth}×{imageHeight})</span>
          <span>~{breakdown.imageTokens} tk</span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span>📝 Notes ({breakdown.characterCount} chars)</span>
          <span>~{breakdown.textTokens} tk</span>
        </div>
        {totalFindingCount !== undefined && totalFindingCount > 1 && (
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              paddingTop: 'var(--space-1)',
              borderTop: '1px dashed var(--color-border)',
              color: 'var(--color-text)',
              fontWeight: 600,
            }}
          >
            <span>Total Bundle ({totalFindingCount} items)</span>
            <span>~{(breakdown.totalTokens * totalFindingCount).toLocaleString()} tk</span>
          </div>
        )}
      </div>
    </div>
  );
};
