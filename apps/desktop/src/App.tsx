import React, { useEffect, useState, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import {
  Toast,
  BundlesDrawer,
  BundleComposer,
  FindingDetailItemDto,
  BundleDetailDto,
} from '@snapdown/ui';
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
import { listBundles, createBundle, deleteBundle } from './services/bundle';
import { listFindings, FindingDetailDto } from './services/finding';
import {
  HotkeyAction,
  HotkeySettingsDto,
  NamedBudget,
  ResolvedPair,
  Settings,
  StartupState,
} from './types/settings';

export const App: React.FC<{ initialTab?: NavigationTab }> = ({ initialTab = 'findings' }) => {
  const [activeTab, setActiveTab] = useState<NavigationTab>(initialTab);
  const [isHistoryDrawerOpen, setIsHistoryDrawerOpen] = useState(false);
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);
  const [isComposerModalOpen, setIsComposerModalOpen] = useState(false);
  const [composerFindings, setComposerFindings] = useState<FindingDetailItemDto[]>([]);
  const [savedBundles, setSavedBundles] = useState<BundleDetailDto[]>([]);
  const [activeFinding, setActiveFinding] = useState<FindingDetailDto | null>(null);

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

  const [startupStatus, setStartupStatus] = useState<StartupState>('unknown');
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const refreshBundles = useCallback(async () => {
    try {
      const data = await listBundles();
      setSavedBundles(data);
    } catch {
      // Ignored
    }
  }, []);

  useEffect(() => {
    refreshBundles();
  }, [refreshBundles]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    try {
      listen<string>('switch-tab', (event) => {
        if (['findings', 'bundles', 'agent-access', 'settings'].includes(event.payload)) {
          setActiveTab(event.payload as NavigationTab);
        }
      })
        .then((fn) => {
          unlisten = fn;
        })
        .catch(() => {});

      listen('capture-completed', () => {
        setActiveTab('findings');
      }).catch(() => {});
    } catch {
      // Ignored outside Tauri
    }

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

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
          const val = startupRes.value;
          setStartupStatus(val.state ?? (val.enabled ? 'on' : 'off'));
        } else {
          console.error('Failed to load startup status:', startupRes.reason);
          setStartupStatus('unreadable');
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
      const nextState = res.state ?? (res.enabled ? 'on' : 'off');
      setStartupStatus(nextState);
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
        setStartupStatus(current.state ?? (current.enabled ? 'on' : 'off'));
      } catch {
        setStartupStatus('unreadable');
      }
    }
  };

  const handleRetryStartup = async () => {
    setStartupStatus('unknown');
    try {
      const res = await getStartupStatus();
      setStartupStatus(res.state ?? (res.enabled ? 'on' : 'off'));
    } catch (err: unknown) {
      console.error('Failed to retry startup status read:', err);
      setStartupStatus('unreadable');
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

  const handleComposeModal = async (selectedIds: string[]) => {
    try {
      const currentFindings = await listFindings();
      // Filter findings to strictly match what was selected on filmstrip
      const targetFindings = selectedIds.length > 0
        ? currentFindings.filter((f) => selectedIds.includes(f.finding.id))
        : (activeFinding ? [activeFinding] : currentFindings.slice(0, 1));

      setComposerFindings(targetFindings);
      setIsComposerModalOpen(true);
    } catch (err) {
      console.error('Failed to prepare bundle compose:', err);
    }
  };

  const handleCreateBundle = async (name: string, selectedIds: string[]) => {
    await createBundle({ name, finding_ids: selectedIds });
    setIsComposerModalOpen(false);
    await refreshBundles();
    setToastMessage(`Bundle "${name}" assembled and saved!`);
  };

  const handleCopyMarkdown = async (bundleId: string) => {
    const target = savedBundles.find((b) => b.bundle.id === bundleId);
    if (target?.bundle?.markdown) {
      await navigator.clipboard.writeText(target.bundle.markdown);
      setToastMessage('Bundle Markdown copied to clipboard');
    }
  };

  const handleDeleteBundle = async (bundleId: string) => {
    await deleteBundle(bundleId);
    await refreshBundles();
    setToastMessage('Bundle deleted successfully');
  };

  const activeFileName = activeFinding?.finding?.image_path
    ? activeFinding.finding.image_path.split(/[/\\\\]/).pop() || activeFinding.finding.image_path
    : 'No finding selected';

  return (
    <div data-testid="app-shell" style={{ width: '100vw', height: '100vh', overflow: 'hidden' }}>
      <EditorShell
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onCaptureClick={handleCaptureClick}
        onOpenHistory={() => {
          refreshBundles();
          setIsHistoryDrawerOpen(true);
        }}
        onOpenSettings={() => setIsSettingsModalOpen(true)}
        activeFindingTitle={activeFileName}
      >
        {activeTab === 'findings' && (
          <FindingsView
            onCompose={handleComposeModal}
            onActiveFindingChange={setActiveFinding}
          />
        )}
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
            startupStatus={startupStatus}
            onSaveVaultPath={handleSaveVaultPath}
            onOpenExplorer={handleOpenExplorer}
            onSaveQualityBudget={handleSaveQualityBudget}
            onSaveHotkey={handleSaveHotkey}
            onClearHotkey={handleClearHotkey}
            onToggleStartup={handleToggleStartup}
            onRetryStartup={handleRetryStartup}
            disabled={isLoading}
          />
        )}
      </EditorShell>

      {/* 3-Column Bundle Review & Assembly Modal (SPEC-05) */}
      {isComposerModalOpen && (
        <div
          data-testid="bundle-composer-modal-backdrop"
          onClick={() => setIsComposerModalOpen(false)}
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'var(--color-overlay-scrim)',
            backdropFilter: 'blur(4px)',
            zIndex: 300,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 'var(--space-4)',
          }}
        >
          <div onClick={(e) => e.stopPropagation()}>
            <BundleComposer
              findings={composerFindings}
              onCreateBundle={handleCreateBundle}
              onCancel={() => setIsComposerModalOpen(false)}
            />
          </div>
        </div>
      )}

      {/* Saved Bundles History Drawer (SPEC-06) */}
      <BundlesDrawer
        isOpen={isHistoryDrawerOpen}
        onClose={() => setIsHistoryDrawerOpen(false)}
        bundles={savedBundles}
        onSelectBundle={() => {
          setIsHistoryDrawerOpen(false);
          setActiveTab('bundles');
        }}
        onCopyMarkdown={handleCopyMarkdown}
        onDeleteBundle={handleDeleteBundle}
      />

      {/* Settings Modal (State 6) */}
      {isSettingsModalOpen && (
        <div
          data-testid="settings-modal-backdrop"
          onClick={() => setIsSettingsModalOpen(false)}
          style={{
            position: 'fixed',
            inset: 0,
            backgroundColor: 'var(--color-overlay-scrim)',
            backdropFilter: 'blur(4px)',
            zIndex: 300,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 'var(--space-4)',
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              backgroundColor: 'var(--color-surface)',
              borderRadius: 'var(--radius-lg)',
              boxShadow: 'var(--shadow-modal)',
              border: '1px solid var(--color-border)',
              maxHeight: '90vh',
              overflowY: 'auto',
              width: '100%',
              maxWidth: '56rem',
            }}
          >
            <SettingsView
              settings={settings}
              hotkeySettings={hotkeySettings}
              startupStatus={startupStatus}
              onSaveVaultPath={handleSaveVaultPath}
              onOpenExplorer={handleOpenExplorer}
              onSaveQualityBudget={handleSaveQualityBudget}
              onSaveHotkey={handleSaveHotkey}
              onClearHotkey={handleClearHotkey}
              onToggleStartup={handleToggleStartup}
              onRetryStartup={handleRetryStartup}
              onClose={() => setIsSettingsModalOpen(false)}
              agentAccessContent={<AgentAccessView />}
              disabled={isLoading}
            />
          </div>
        </div>
      )}

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
