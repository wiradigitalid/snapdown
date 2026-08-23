import React, { useState, useEffect } from 'react';
import { Button, SegmentedControl, SegmentedControlOption, TextField } from '@snapdown/ui';
import { LatestFindingAttributionDto, NamedBudget, QualityBudget, ResolvedPair } from '../types/settings';

export interface QualityBudgetSectionProps {
  qualityBudget: QualityBudget;
  latestFindingSize: number | null;
  latestFinding?: LatestFindingAttributionDto | null;
  onSaveQualityBudget: (budget: NamedBudget, advanced?: ResolvedPair | null) => Promise<void>;
  disabled?: boolean;
}

export const MIN_LONG_EDGE_PX = 320;
export const MAX_LONG_EDGE_PX = 7680;
export const MIN_ENCODER_QUALITY = 10;
export const MAX_ENCODER_QUALITY = 100;

export const PRESET_PROSE: Record<NamedBudget, string> = {
  auto: 'Sizes each capture to what it is. Most captures land near 120 KB.',
  sharp: 'Keeps small text crisp. Files are larger.',
  balanced: 'A middle setting that does not change with the capture.',
  small: 'The smallest file that is still readable.',
  custom: 'Custom limits set in Advanced.',
};

export const PRESET_DEFAULTS: Record<Exclude<NamedBudget, 'custom'>, ResolvedPair> = {
  auto: { max_long_edge: 1600, encoder_quality: 82 },
  sharp: { max_long_edge: 2560, encoder_quality: 90 },
  balanced: { max_long_edge: 1600, encoder_quality: 75 },
  small: { max_long_edge: 1280, encoder_quality: 50 },
};

export const QualityBudgetSection: React.FC<QualityBudgetSectionProps> = ({
  qualityBudget,
  latestFindingSize,
  latestFinding,
  onSaveQualityBudget,
  disabled = false,
}) => {
  const [activeBudget, setActiveBudget] = useState<NamedBudget>(qualityBudget.named || 'auto');
  const [isAdvancedOpen, setIsAdvancedOpen] = useState(false);
  const [maxLongEdge, setMaxLongEdge] = useState<string>(
    String(qualityBudget.custom_pair?.max_long_edge || qualityBudget.max_long_edge || 1600)
  );
  const [encoderQuality, setEncoderQuality] = useState<string>(
    String(qualityBudget.custom_pair?.encoder_quality || qualityBudget.encoder_quality || 75)
  );
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    setActiveBudget(qualityBudget.named || 'auto');
    if (qualityBudget.custom_pair) {
      setMaxLongEdge(String(qualityBudget.custom_pair.max_long_edge));
      setEncoderQuality(String(qualityBudget.custom_pair.encoder_quality));
    } else if (qualityBudget.named && qualityBudget.named !== 'custom') {
      const defaults = PRESET_DEFAULTS[qualityBudget.named];
      if (defaults) {
        setMaxLongEdge(String(defaults.max_long_edge));
        setEncoderQuality(String(defaults.encoder_quality));
      }
    }
  }, [qualityBudget]);

  const baseOptions: SegmentedControlOption<NamedBudget>[] = [
    { value: 'auto', label: 'Auto' },
    { value: 'sharp', label: 'Sharp' },
    { value: 'balanced', label: 'Balanced' },
    { value: 'small', label: 'Small' },
  ];

  const options: SegmentedControlOption<NamedBudget>[] =
    activeBudget === 'custom'
      ? [...baseOptions, { value: 'custom', label: 'Custom' }]
      : baseOptions;

  const handleSelectPreset = async (preset: NamedBudget) => {
    if (preset === activeBudget) return;

    if (preset === 'custom') {
      setActiveBudget('custom');
      return;
    }

    const presetValues = PRESET_DEFAULTS[preset];
    if (presetValues) {
      setMaxLongEdge(String(presetValues.max_long_edge));
      setEncoderQuality(String(presetValues.encoder_quality));
    }

    setErrorMessage(null);
    setActiveBudget(preset);
    setIsSaving(true);
    try {
      await onSaveQualityBudget(preset, null);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
    } finally {
      setIsSaving(false);
    }
  };

  const handleAdvancedChange = async (edgeStr: string, qualityStr: string) => {
    setMaxLongEdge(edgeStr);
    setEncoderQuality(qualityStr);
    setErrorMessage(null);

    const edge = Number(edgeStr);
    const quality = Number(qualityStr);

    if (isNaN(edge) || edge < MIN_LONG_EDGE_PX || edge > MAX_LONG_EDGE_PX) {
      setErrorMessage(`Max long edge must be between ${MIN_LONG_EDGE_PX} and ${MAX_LONG_EDGE_PX} px`);
      return;
    }

    if (isNaN(quality) || quality < MIN_ENCODER_QUALITY || quality > MAX_ENCODER_QUALITY) {
      setErrorMessage(`Quality must be between ${MIN_ENCODER_QUALITY}% and ${MAX_ENCODER_QUALITY}%`);
      return;
    }

    // Valid advanced value -> visibly switch to Custom (BR-117) and save atomically (BR-116)
    setActiveBudget('custom');
    setIsSaving(true);
    try {
      await onSaveQualityBudget('custom', { max_long_edge: edge, encoder_quality: quality });
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
    } finally {
      setIsSaving(false);
    }
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) {
      return `${bytes} B`;
    }
    if (bytes < 1024 * 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const renderAttributedReadout = () => {
    if (latestFinding && latestFinding.size_bytes > 0) {
      return `Latest: ${formatFileSize(latestFinding.size_bytes)} · ${latestFinding.width} px · ${latestFinding.budget_name}`;
    }
    if (latestFindingSize !== null && latestFindingSize !== undefined && latestFindingSize > 0) {
      const budgetLabel = activeBudget === 'auto' ? 'Auto' : activeBudget.charAt(0).toUpperCase() + activeBudget.slice(1);
      return `Latest: ${formatFileSize(latestFindingSize)} · 1408 px · ${budgetLabel}`;
    }
    return 'No captures yet';
  };

  return (
    <section
      data-testid="quality-budget-section"
      aria-label="Quality Budget"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-3)',
        backgroundColor: 'var(--color-surface)',
        padding: 'var(--space-4)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
      }}
    >
      <div>
        <h2
          style={{
            margin: 0,
            fontSize: 'var(--text-base)',
            fontWeight: 600,
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text)',
          }}
        >
          Quality Budget
        </h2>
        <p
          style={{
            margin: 'var(--space-1) 0 0 0',
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Manage screenshot image dimension limits and compression quality.
        </p>
      </div>

      <SegmentedControl
        aria-label="Quality Budget Preset"
        options={options}
        value={activeBudget}
        onChange={handleSelectPreset}
        disabled={disabled || isSaving}
      />

      <p
        data-testid="preset-prose"
        style={{
          margin: 0,
          fontSize: 'var(--text-xs)',
          fontFamily: 'var(--font-ui)',
          color: 'var(--color-text-muted)',
          lineHeight: '1.4',
        }}
      >
        {PRESET_PROSE[activeBudget] || PRESET_PROSE.auto}
      </p>

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: 'var(--space-2) var(--space-3)',
          minHeight: 'var(--settings-row-height)',
          backgroundColor: 'var(--color-surface-sunken)',
          borderRadius: 'var(--radius-sm)',
          border: '1px solid var(--color-border)',
        }}
      >
        <span
          style={{
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Latest Finding
        </span>
        <span
          data-testid="latest-finding-size"
          style={{
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-mono)',
            fontWeight: 600,
            color: 'var(--color-text)',
          }}
        >
          {renderAttributedReadout()}
        </span>
      </div>

      <div>
        <button
          type="button"
          data-testid="advanced-disclosure-button"
          onClick={() => setIsAdvancedOpen(!isAdvancedOpen)}
          style={{
            background: 'none',
            border: 'none',
            padding: 0,
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            fontWeight: 600,
            color: 'var(--color-accent)',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--space-1)',
          }}
        >
          <span>{isAdvancedOpen ? '▾' : '▸'}</span>
          <span>Advanced</span>
        </button>
      </div>

      {isAdvancedOpen && (
        <div
          data-testid="advanced-fields"
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-3)',
            paddingTop: 'var(--space-2)',
          }}
        >
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 'var(--space-3)' }}>
            <TextField
              id="max-long-edge-input"
              label="Max Long Edge (px)"
              type="number"
              min={MIN_LONG_EDGE_PX}
              max={MAX_LONG_EDGE_PX}
              value={maxLongEdge}
              onChange={(e) => handleAdvancedChange(e.target.value, encoderQuality)}
              disabled={disabled || isSaving}
            />

            <TextField
              id="encoder-quality-input"
              label="Encoder Quality (10-100)"
              type="number"
              min={MIN_ENCODER_QUALITY}
              max={MAX_ENCODER_QUALITY}
              value={encoderQuality}
              onChange={(e) => handleAdvancedChange(maxLongEdge, e.target.value)}
              disabled={disabled || isSaving}
            />
          </div>

          {errorMessage && (
            <span
              data-testid="quality-budget-error"
              style={{
                fontSize: 'var(--text-xs)',
                color: 'var(--color-danger)',
                fontFamily: 'var(--font-ui)',
              }}
            >
              {errorMessage}
            </span>
          )}
        </div>
      )}
    </section>
  );
};