import React, { useEffect, useState } from 'react';
import { Toast } from '@snapdown/ui';
import { VaultSection } from './components/VaultSection';
import { QualityBudgetSection } from './components/QualityBudgetSection';
import { HotkeySection } from './components/HotkeySection';
import {
  clearHotkey as apiClearHotkey,
  getHotkeys,
  getSettings,
  openVaultFolder,
  setHotkey as apiSetHotkey,
  setQualityBudget as apiSetQualityBudget,
  setVaultPath as apiSetVaultPath,
} from './services/settings';
import { HotkeyAction, HotkeySettingsDto, Settings } from './types/settings';

export const App: React.FC = () => {
  const [settings, setSettings] = useState<Settings>({
    vault_path: '',
    quality_budget: {
      max_long_edge: 1600,
      encoder_quality: 75,
    },
    latest_finding_size: null,
  });

  const [hotkeySettings, setHotkeySettings] = useState<HotkeySettingsDto>({
    hotkeys: [
      {
        action: 'capture',
        shortcut: 'CommandOrControl+Shift+S',
        is_registered: true,
        is_active: true,
      },
      {
        action: 'open_editor',
        shortcut: 'CommandOrControl+Shift+E',
        is_registered: true,
        is_active: true,
      },
    ],
    startup_warnings: [],
  });

  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;

    Promise.all([getSettings(), getHotkeys()])
      .then(([loadedSettings, loadedHotkeys]) => {
        if (isMounted) {
          setSettings(loadedSettings);
          setHotkeySettings(loadedHotkeys);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        console.error('Failed to load initial settings:', err);
        if (isMounted) {
          setIsLoading(false);
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const handleSaveVaultPath = async (newPath: string, migrate: boolean) => {
    const updatedPath = await apiSetVaultPath(newPath, migrate);
    setSettings((prev) => ({ ...prev, vault_path: updatedPath }));
    setToastMessage('Vault folder location updated successfully');
  };

  const handleSaveQualityBudget = async (maxLongEdge: number, encoderQuality: number) => {
    const updatedBudget = await apiSetQualityBudget(maxLongEdge, encoderQuality);
    setSettings((prev) => ({ ...prev, quality_budget: updatedBudget }));
    setToastMessage('Quality Budget saved successfully');
  };

  const handleSaveHotkey = async (action: HotkeyAction, shortcut: string) => {
    await apiSetHotkey(action, shortcut);
    const refreshed = await getHotkeys();
    setHotkeySettings(refreshed);
    setToastMessage(`Hotkey for ${action} updated successfully`);
  };

  const handleClearHotkey = async (action: HotkeyAction) => {
    await apiClearHotkey(action);
    const refreshed = await getHotkeys();
    setHotkeySettings(refreshed);
    setToastMessage(`Hotkey for ${action} cleared`);
  };

  const handleOpenExplorer = async () => {
    try {
      await openVaultFolder();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setToastMessage(`Failed to open folder: ${msg}`);
    }
  };

  return (
    <main
      data-testid="app-shell"
      style={{
        padding: 'var(--space-6)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-5)',
        maxWidth: '48rem',
        margin: '0 auto',
      }}
    >
      <header
        style={{
          borderBottom: '1px solid var(--color-border)',
          paddingBottom: 'var(--space-4)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div>
          <h1
            style={{
              margin: 0,
              fontSize: 'var(--text-xl)',
              fontWeight: 700,
              fontFamily: 'var(--font-ui)',
              color: 'var(--color-text)',
            }}
          >
            Snapdown Settings
          </h1>
          <p
            style={{
              margin: 'var(--space-1) 0 0 0',
              fontSize: 'var(--text-xs)',
              fontFamily: 'var(--font-ui)',
              color: 'var(--color-text-muted)',
            }}
          >
            Manage storage directory, image quality, and hotkey preferences.
          </p>
        </div>
      </header>

      <VaultSection
        vaultPath={settings.vault_path}
        onSaveVaultPath={handleSaveVaultPath}
        onOpenExplorer={handleOpenExplorer}
        disabled={isLoading}
      />

      <QualityBudgetSection
        qualityBudget={settings.quality_budget}
        latestFindingSize={settings.latest_finding_size}
        onSaveQualityBudget={handleSaveQualityBudget}
        disabled={isLoading}
      />

      <HotkeySection
        hotkeySettings={hotkeySettings}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
        disabled={isLoading}
      />

      {toastMessage && (
        <Toast
          message={toastMessage}
          onDismiss={() => setToastMessage(null)}
          durationMs={3000}
        />
      )}
    </main>
  );
};
