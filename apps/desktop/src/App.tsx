import React, { useEffect, useState } from 'react';
import { Toast } from '@snapdown/ui';
import { EditorShell, NavigationTab } from './components/EditorShell';
import { SettingsView } from './components/SettingsView';
import { FindingsView } from './components/FindingsView';
import { BundleView } from './components/BundleView';
import { AgentAccessView } from './components/AgentAccessView';
import {
  clearHotkey as apiClearHotkey,
  getHotkeys,
  getSettings,
  getStartupStatus,
  openVaultFolder,
  setHotkey as apiSetHotkey,
  setQualityBudget as apiSetQualityBudget,
  setStartupStatus as apiSetStartupStatus,
  setVaultPath as apiSetVaultPath,
} from './services/settings';
import { triggerOverlay } from './services/capture';
import { HotkeyAction, HotkeySettingsDto, NamedBudget, ResolvedPair, Settings } from './types/settings';

export const App: React.FC<{ initialTab?: NavigationTab }> = ({ initialTab = 'settings' }) => {
  const [activeTab, setActiveTab] = useState<NavigationTab>(initialTab);
  const [settings, setSettings] = useState<Settings>({
    vault_path: '',
    quality_budget: {
      named: 'auto',
      prose: 'Sizes each capture to what it is. Most captures land near 120 KB.',
      max_long_edge: 1600,
      encoder_quality: 82,
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

  const [runAtStartup, setRunAtStartup] = useState<boolean>(true);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let isMounted = true;

    Promise.allSettled([getSettings(), getHotkeys(), getStartupStatus()])
      .then(([settingsRes, hotkeysRes, startupRes]) => {
        if (!isMounted) return;

        if (settingsRes.status === 'fulfilled') {
          setSettings(settingsRes.value);
        } else {
          console.error('Failed to load settings:', settingsRes.reason);
        }

        if (hotkeysRes.status === 'fulfilled') {
          setHotkeySettings(hotkeysRes.value);
        } else {
          console.error('Failed to load hotkeys:', hotkeysRes.reason);
        }

        if (startupRes.status === 'fulfilled') {
          setRunAtStartup(startupRes.value.enabled);
        } else {
          console.error('Failed to load startup status:', startupRes.reason);
        }

        setIsLoading(false);
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

  const handleSaveQualityBudget = async (budget: NamedBudget, advanced?: ResolvedPair | null) => {
    const updatedBudget = await apiSetQualityBudget(budget, advanced);
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

  const handleToggleStartup = async (enabled: boolean) => {
    try {
      const res = await apiSetStartupStatus(enabled);
      setRunAtStartup(res.enabled);
      setToastMessage(
        res.enabled
          ? 'Snapdown will run at Windows startup'
          : 'Snapdown startup registration removed'
      );
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setToastMessage(`Failed to update startup registration: ${msg}`);
      try {
        const current = await getStartupStatus();
        setRunAtStartup(current.enabled);
      } catch {
        setRunAtStartup(!enabled);
      }
    }
  };

  const handleOpenExplorer = async () => {
    try {
      await openVaultFolder();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setToastMessage(`Failed to open folder: ${msg}`);
    }
  };

  const handleCaptureClick = async () => {
    try {
      await triggerOverlay();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error('Failed to trigger capture overlay:', msg);
    }
  };

  return (
    <div data-testid="app-shell" style={{ width: '100vw', height: '100vh', overflow: 'hidden' }}>
      <EditorShell
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onCaptureClick={handleCaptureClick}
      >
        {activeTab === 'findings' && <FindingsView />}
        {activeTab === 'bundles' && <BundleView />}
        {activeTab === 'agent-access' && (
          <div style={{ padding: 'var(--space-5)', maxWidth: '56rem', margin: '0 auto' }}>
            <AgentAccessView />
          </div>
        )}
        {activeTab === 'settings' && (
          <SettingsView
            settings={settings}
            hotkeySettings={hotkeySettings}
            runAtStartup={runAtStartup}
            onSaveVaultPath={handleSaveVaultPath}
            onOpenExplorer={handleOpenExplorer}
            onSaveQualityBudget={handleSaveQualityBudget}
            onSaveHotkey={handleSaveHotkey}
            onClearHotkey={handleClearHotkey}
            onToggleStartup={handleToggleStartup}
            disabled={isLoading}
          />
        )}
      </EditorShell>

      {toastMessage && (
        <Toast
          message={toastMessage}
          onDismiss={() => setToastMessage(null)}
          durationMs={3000}
        />
      )}
    </div>
  );
};