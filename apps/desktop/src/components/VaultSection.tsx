import React, { useState } from 'react';
import { Button, TextField, ConfirmDialog } from '@snapdown/ui';
import { pickVaultFolder } from '../services/settings';

export interface VaultSectionProps {
  vaultPath: string;
  onSaveVaultPath: (newPath: string, migrate: boolean) => Promise<void>;
  onOpenExplorer: () => Promise<void>;
  disabled?: boolean;
}

export const VaultSection: React.FC<VaultSectionProps> = ({
  vaultPath,
  onSaveVaultPath,
  onOpenExplorer,
  disabled = false,
}) => {
  const [inputValue, setInputValue] = useState(vaultPath);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [showConfirm, setShowConfirm] = useState(false);
  const [pendingPath, setPendingPath] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  React.useEffect(() => {
    setInputValue(vaultPath);
  }, [vaultPath]);

  const handleBrowse = async () => {
    try {
      const selected = await pickVaultFolder();
      if (selected && selected.trim() !== '') {
        setInputValue(selected);
        setErrorMessage(null);
        if (selected !== vaultPath) {
          setPendingPath(selected);
          setShowConfirm(true);
        }
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(`Failed to open folder picker: ${msg}`);
    }
  };

  const handleApply = () => {
    const trimmed = inputValue.trim();
    if (!trimmed) {
      setErrorMessage('Vault path cannot be empty');
      return;
    }
    if (trimmed === vaultPath) {
      setErrorMessage(null);
      return;
    }
    setErrorMessage(null);
    setPendingPath(trimmed);
    setShowConfirm(true);
  };

  const handleConfirmMigration = async (migrate: boolean) => {
    if (!pendingPath) return;
    setShowConfirm(false);
    setIsSaving(true);
    try {
      await onSaveVaultPath(pendingPath, migrate);
      setErrorMessage(null);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessage(msg);
      setInputValue(vaultPath);
    } finally {
      setIsSaving(false);
      setPendingPath(null);
    }
  };

  return (
    <section
      data-testid="vault-section"
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
          Vault Folder
        </h2>
        <p
          style={{
            margin: 'var(--space-1) 0 0 0',
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Location where capture screenshots and markdown notes are stored.
        </p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
        <div style={{ display: 'flex', gap: 'var(--space-2)', alignItems: 'flex-end' }}>
          <div style={{ flex: 1 }}>
            <TextField
              id="vault-path-input"
              label="Vault Path"
              value={inputValue}
              onChange={(e) => {
                setInputValue(e.target.value);
                setErrorMessage(null);
              }}
              invalid={Boolean(errorMessage)}
              errorMessage={errorMessage || undefined}
              disabled={disabled || isSaving}
            />
          </div>
          <Button
            variant="secondary"
            onClick={handleBrowse}
            disabled={disabled || isSaving}
          >
            Browse...
          </Button>
          <Button
            variant="primary"
            onClick={handleApply}
            disabled={disabled || isSaving || inputValue.trim() === vaultPath}
            loading={isSaving}
          >
            Apply
          </Button>
        </div>

        <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Button
            variant="secondary"
            onClick={onOpenExplorer}
            disabled={disabled || isSaving}
          >
            Open in Explorer
          </Button>
        </div>
      </div>

      <ConfirmDialog
        isOpen={showConfirm}
        title="Move Existing Files?"
        message="Would you like to move all existing capture files from the current Vault to the new location?"
        confirmLabel="Move Files"
        cancelLabel="Leave Files"
        onConfirm={() => handleConfirmMigration(true)}
        onCancel={() => handleConfirmMigration(false)}
      />
    </section>
  );
};
