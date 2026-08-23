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
        padding: '16px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        display: 'flex',
        flexDirection: 'column',
        gap: '16px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>Compose Bundle</h2>
        {onCancel && (
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
        )}
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <label htmlFor="bundle-title-input" style={{ fontSize: '13px', fontWeight: 500 }}>
          Bundle Title
        </label>
        <TextField
          id="bundle-title-input"
          value={bundleName}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setBundleName(e.target.value)}
          placeholder="e.g. Sprint Review & Defect Report"
        />
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <label style={{ fontSize: '13px', fontWeight: 500 }}>
            Select Findings ({selectedIds.length} of {findings.length})
          </label>
          <div style={{ display: 'flex', gap: '8px' }}>
            <Button variant="secondary" onClick={handleSelectAll}>
              Select All
            </Button>
            <Button variant="secondary" onClick={handleClearSelection}>
              Clear
            </Button>
          </div>
        </div>

        {findings.length === 0 ? (
          <p style={{ fontSize: '13px', color: '#64748b' }}>No findings available to bundle.</p>
        ) : (
          <div
            style={{
              maxHeight: '260px',
              overflowY: 'auto',
              display: 'flex',
              flexDirection: 'column',
              gap: '6px',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              padding: '8px',
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
                    gap: '10px',
                    padding: '8px',
                    backgroundColor: checked ? '#f0f9ff' : '#ffffff',
                    border: '1px solid #e2e8f0',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  <Checkbox
                    id={`cb-${f.finding.id}`}
                    checked={checked}
                    onChange={() => toggleFinding(f.finding.id)}
                  />
                  <div style={{ flex: 1 }}>
                    <div style={{ fontSize: '13px', fontWeight: 500, color: '#1e293b' }}>
                      {f.finding.image_path}
                    </div>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>
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
        <div data-testid="composer-error-msg" style={{ color: '#ef4444', fontSize: '13px' }}>
          {errorMsg}
        </div>
      )}

      <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
        <Button variant="primary" onClick={handleCreate} disabled={isSubmitting}>
          {isSubmitting ? 'Composing...' : 'Create Bundle'}
        </Button>
      </div>
    </div>
  );
};
