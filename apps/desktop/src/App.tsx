import React, { useEffect, useState } from 'react';
import { Toast } from '@snapdown/ui';
import { VaultSection } from './components/VaultSection';
import { QualityBudgetSection } from './components/QualityBudgetSection';
import { HotkeySection } from './components/HotkeySection';
import { GeneralSection } from './components/GeneralSection';
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
import { HotkeyAction, HotkeySettingsDto, Settings } from './types/settings';

type NavigationTab = 'findings' | 'bundles' | 'agent-access' | 'settings';

export const App: React.FC<{ initialTab?: NavigationTab }> = ({ initialTab = 'settings' }) => {
  const [activeTab, setActiveTab] = useState<NavigationTab>(initialTab);
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

  return (
    <div
      data-testid="app-shell"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        width: '100vw',
        backgroundColor: 'var(--color-bg)',
        color: 'var(--color-text)',
        fontFamily: 'var(--font-ui)',
        overflow: 'hidden',
      }}
    >
      {/* Top App Header & Navigation */}
      <header
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          padding: 'var(--space-3) var(--space-5)',
          backgroundColor: 'var(--color-surface)',
          borderBottom: '1px solid var(--color-border)',
          flexShrink: 0,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-3)' }}>
          <div
            style={{
              width: '28px',
              height: '28px',
              borderRadius: 'var(--radius-sm)',
              backgroundColor: 'var(--color-accent)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--color-accent-text)',
              fontWeight: 700,
              fontSize: 'var(--text-sm)',
            }}
          >
            S
          </div>
          <span style={{ fontSize: 'var(--text-lg)', fontWeight: 700, letterSpacing: '-0.02em' }}>
            Snapdown
          </span>
        </div>

        {/* Tab Navigation */}
        <nav style={{ display: 'flex', gap: 'var(--space-1)' }}>
          {[
            { id: 'findings' as NavigationTab, label: 'Findings' },
            { id: 'bundles' as NavigationTab, label: 'Bundles' },
            { id: 'agent-access' as NavigationTab, label: 'Agent Access' },
            { id: 'settings' as NavigationTab, label: 'Settings' },
          ].map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  padding: 'var(--space-2) var(--space-4)',
                  borderRadius: 'var(--radius-md)',
                  border: 'none',
                  backgroundColor: isActive ? 'var(--color-accent)' : 'transparent',
                  color: isActive ? 'var(--color-accent-text)' : 'var(--color-text-muted)',
                  fontWeight: isActive ? 600 : 500,
                  fontSize: 'var(--text-sm)',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                }}
              >
                {tab.label}
              </button>
            );
          })}
        </nav>
      </header>

      {/* Main Content Area */}
      <main
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: activeTab === 'settings' ? 'var(--space-5)' : 0,
        }}
      >
        {activeTab === 'findings' && <FindingsView />}
        {activeTab === 'bundles' && <BundleView />}
        {activeTab === 'agent-access' && (
          <div style={{ padding: 'var(--space-5)', maxWidth: '56rem', margin: '0 auto' }}>
            <AgentAccessView />
          </div>
        )}
        {activeTab === 'settings' && (
          <div
            style={{
              maxWidth: '56rem',
              margin: '0 auto',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-4)',
            }}
          >
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 'var(--space-4)' }}>
              <GeneralSection
                runAtStartup={runAtStartup}
                onToggleStartup={handleToggleStartup}
                disabled={isLoading}
              />
              <QualityBudgetSection
                qualityBudget={settings.quality_budget}
                latestFindingSize={settings.latest_finding_size}
                onSaveQualityBudget={handleSaveQualityBudget}
                disabled={isLoading}
              />
            </div>

            <VaultSection
              vaultPath={settings.vault_path}
              onSaveVaultPath={handleSaveVaultPath}
              onOpenExplorer={handleOpenExplorer}
              disabled={isLoading}
            />

            <HotkeySection
              hotkeySettings={hotkeySettings}
              onSaveHotkey={handleSaveHotkey}
              onClearHotkey={handleClearHotkey}
              disabled={isLoading}
            />
          </div>
        )}
      </main>

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
