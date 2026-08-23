import React, { useState } from 'react';
import { Button } from './Button';
import { Checkbox } from './Checkbox';
import { TextField } from './TextField';
import { FindingDetailItemDto } from './FindingsEditor';

export interface BundleComposerProps {
  findings: FindingDetailItemDto[];
  onCreateBundle: (name: string, selectedFindingIds: string[]) => Promise<void>;
  onCancel?: () => void;
}

export const BundleComposer: React.FC<BundleComposerProps> = ({
  findings,
  onCreateBundle,
  onCancel,
}) => {
  const [bundleName, setBundleName] = useState('');
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const toggleFinding = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]
    );
  };

  const handleSelectAll = () => {
    setSelectedIds(findings.map((f) => f.finding.id));
  };

  const handleClearSelection = () => {
    setSelectedIds([]);
  };

  const handleCreate = async () => {
    if (bundleName.trim().length === 0) {
      setErrorMsg('Please enter a bundle name');
      return;
    }
    if (selectedIds.length === 0) {
      setErrorMsg('Please select at least one finding');
      return;
    }

    setIsSubmitting(true);
    setErrorMsg(null);
    try {
      await onCreateBundle(bundleName.trim(), selectedIds);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMsg(msg);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div
      data-testid="bundle-composer"
      style={{
        padding: 'var(--space-4)',
        backgroundColor: 'var(--color-surface)',
        border: '1px solid var(--color-border)',
        borderRadius: 'var(--radius-md)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2 style={{ margin: 0, fontSize: 'var(--text-base)', fontWeight: 600, color: 'var(--color-text)' }}>
          Compose Bundle
        </h2>
        {onCancel && (
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
        )}
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
        <label htmlFor="bundle-title-input" style={{ fontSize: 'var(--text-sm)', fontWeight: 500, color: 'var(--color-text)' }}>
          Bundle Title
        </label>
        <TextField
          id="bundle-title-input"
          value={bundleName}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setBundleName(e.target.value)}
          placeholder="e.g. Sprint Review & Defect Report"
        />
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <label style={{ fontSize: 'var(--text-sm)', fontWeight: 500, color: 'var(--color-text)' }}>
            Select Findings ({selectedIds.length} of {findings.length})
          </label>
          <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
            <Button variant="secondary" onClick={handleSelectAll}>
              Select All
            </Button>
            <Button variant="secondary" onClick={handleClearSelection}>
              Clear
            </Button>
          </div>
        </div>

        {findings.length === 0 ? (
          <p style={{ fontSize: 'var(--text-sm)', color: 'var(--color-text-muted)' }}>No findings available to bundle.</p>
        ) : (
          <div
            style={{
              maxHeight: '260px',
              overflowY: 'auto',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-1)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-sm)',
              padding: 'var(--space-2)',
            }}
          >
            {findings.map((f) => {
              const checked = selectedIds.includes(f.finding.id);
              return (
                <div
                  key={f.finding.id}
                  data-testid={`bundle-finding-row-${f.finding.id}`}
                  onClick={() => toggleFinding(f.finding.id)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 'var(--space-2)',
                    padding: 'var(--space-2)',
                    backgroundColor: checked ? 'var(--color-info-bg)' : 'var(--color-surface)',
                    border: '1px solid var(--color-border)',
                    borderRadius: 'var(--radius-sm)',
                    cursor: 'pointer',
                  }}
                >
                  <Checkbox
                    id={`cb-${f.finding.id}`}
                    checked={checked}
                    onChange={() => toggleFinding(f.finding.id)}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ fontSize: 'var(--text-sm)', fontWeight: 500, color: checked ? 'var(--color-info-text)' : 'var(--color-text)' }}>
                      {f.finding.image_path}
                    </div>
                    <div style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
                      {f.note.body ? f.note.body.slice(0, 60) : 'No note text'}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {errorMsg && (
        <div data-testid="composer-error-msg" style={{ color: 'var(--color-danger)', fontSize: 'var(--text-sm)' }}>
          {errorMsg}
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 'var(--space-2)' }}>
        <Button variant="primary" onClick={handleCreate} disabled={isSubmitting}>
          {isSubmitting ? 'Composing...' : 'Create Bundle'}
        </Button>
      </div>
    </div>
  );
};
