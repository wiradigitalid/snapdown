import React, { useState } from 'react';
import { Button, TextField } from '@snapdown/ui';
import { QualityBudget } from '../types/settings';

export interface QualityBudgetSectionProps {
  qualityBudget: QualityBudget;
  latestFindingSize: number | null;
  onSaveQualityBudget: (maxLongEdge: number, encoderQuality: number) => Promise<void>;
  disabled?: boolean;
}

export const MIN_LONG_EDGE_PX = 320;
export const MAX_LONG_EDGE_PX = 7680;
export const MIN_ENCODER_QUALITY = 10;
export const MAX_ENCODER_QUALITY = 100;

export const QualityBudgetSection: React.FC<QualityBudgetSectionProps> = ({
  qualityBudget,
  latestFindingSize,
  onSaveQualityBudget,
  disabled = false,
}) => {
  const [maxLongEdge, setMaxLongEdge] = useState<string>(String(qualityBudget.max_long_edge));
  const [encoderQuality, setEncoderQuality] = useState<string>(String(qualityBudget.encoder_quality));
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  React.useEffect(() => {
    setMaxLongEdge(String(qualityBudget.max_long_edge));
    setEncoderQuality(String(qualityBudget.encoder_quality));
  }, [qualityBudget]);

  const validate = (): { edge: number; quality: number } | null => {
    const edge = Number(maxLongEdge);
    const quality = Number(encoderQuality);

    if (isNaN(edge) || edge < MIN_LONG_EDGE_PX || edge > MAX_LONG_EDGE_PX) {
      setErrorMessage(`Max long edge must be between ${MIN_LONG_EDGE_PX} and ${MAX_LONG_EDGE_PX} px`);
      return null;
    }

    if (isNaN(quality) || quality < MIN_ENCODER_QUALITY || quality > MAX_ENCODER_QUALITY) {
      setErrorMessage(`Quality must be between ${MIN_ENCODER_QUALITY}% and ${MAX_ENCODER_QUALITY}%`);
      return null;
    }

    return { edge, quality };
  };

  const handleSave = async () => {
    const validated = validate();
    if (!validated) return;

    setErrorMessage(null);
    setIsSaving(true);
    try {
      await onSaveQualityBudget(validated.edge, validated.quality);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
    } finally {
      setIsSaving(false);
    }
  };

  const formatFileSize = (bytes: number | null): string => {
    if (bytes === null || bytes === undefined) {
      return 'No captures yet';
    }
    if (bytes < 1024) {
      return `${bytes} B`;
    }
    if (bytes < 1024 * 1024) {
      return `${(bytes / 1024).toFixed(1)} KB`;
    }
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const isChanged =
    Number(maxLongEdge) !== qualityBudget.max_long_edge ||
    Number(encoderQuality) !== qualityBudget.encoder_quality;

  return (
    <section
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
        backgroundColor: 'var(--color-surface)',
        padding: 'var(--space-5)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
      }}
    >
      <div>
        <h2
          style={{
            margin: 0,
            fontSize: 'var(--text-lg)',
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
          Manage screenshot image dimension limits and compression quality to balance visual fidelity and storage footprint.
        </p>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 'var(--space-3)' }}>
        <TextField
          id="max-long-edge-input"
          label="Max Long Edge (px)"
          type="number"
          min={MIN_LONG_EDGE_PX}
          max={MAX_LONG_EDGE_PX}
          value={maxLongEdge}
          onChange={(e) => {
            setMaxLongEdge(e.target.value);
            setErrorMessage(null);
          }}
          disabled={disabled || isSaving}
        />

        <TextField
          id="encoder-quality-input"
          label="Encoder Quality (10-100)"
          type="number"
          min={MIN_ENCODER_QUALITY}
          max={MAX_ENCODER_QUALITY}
          value={encoderQuality}
          onChange={(e) => {
            setEncoderQuality(e.target.value);
            setErrorMessage(null);
          }}
          disabled={disabled || isSaving}
        />
      </div>

      {errorMessage && (
        <span
          style={{
            fontSize: 'var(--text-xs)',
            color: 'var(--color-danger)',
            fontFamily: 'var(--font-ui)',
          }}
        >
          {errorMessage}
        </span>
      )}

      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: 'var(--space-3)',
          backgroundColor: 'var(--color-surface-raised)',
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
          Latest Finding Size
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
          {formatFileSize(latestFindingSize)}
        </span>
      </div>

      <div style={{ display: 'flex' }}>
        <Button
          variant="primary"
          onClick={handleSave}
          disabled={disabled || isSaving || !isChanged}
          loading={isSaving}
        >
          Save Quality Budget
        </Button>
      </div>
    </section>
  );
};
