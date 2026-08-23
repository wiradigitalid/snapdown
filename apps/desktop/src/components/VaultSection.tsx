import React, { useState } from 'react';
import { Button, TextField, ConfirmDialog } from '@snapdown/ui';

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
          Location where capture screenshots and markdown notes are securely stored.
        </p>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-3)' }}>
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

        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Button
            variant="primary"
            onClick={handleApply}
            disabled={disabled || isSaving || inputValue.trim() === vaultPath}
            loading={isSaving}
          >
            Apply Change
          </Button>
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
