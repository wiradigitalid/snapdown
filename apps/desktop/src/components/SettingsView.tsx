import React from 'react';
import { GeneralSection } from './GeneralSection';
import { VaultSection } from './VaultSection';
import { QualityBudgetSection } from './QualityBudgetSection';
import { HotkeySection } from './HotkeySection';
import {
  HotkeyAction,
  HotkeySettingsDto,
  NamedBudget,
  ResolvedPair,
  Settings,
  StartupState,
} from '../types/settings';

export interface SettingsViewProps {
  settings: Settings;
  hotkeySettings: HotkeySettingsDto;
  startupStatus: StartupState;
  onSaveVaultPath: (newPath: string, migrate: boolean) => Promise<void>;
  onOpenExplorer: () => Promise<void>;
  onSaveQualityBudget: (budget: NamedBudget, advanced?: ResolvedPair | null) => Promise<void>;
  onSaveHotkey: (action: HotkeyAction, shortcut: string) => Promise<void>;
  onClearHotkey: (action: HotkeyAction) => Promise<void>;
  onToggleStartup: (enabled: boolean) => Promise<void>;
  onRetryStartup?: () => Promise<void> | void;
  disabled?: boolean;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  settings,
  hotkeySettings,
  startupStatus,
  onSaveVaultPath,
  onOpenExplorer,
  onSaveQualityBudget,
  onSaveHotkey,
  onClearHotkey,
  onToggleStartup,
  onRetryStartup,
  disabled = false,
}) => {
  return (
    <div
      data-testid="settings-view"
      style={{
        maxWidth: '56rem',
        margin: '0 auto',
        padding: 'var(--space-5)',
        display: 'flex',
        flexDirection: 'row',
        flexWrap: 'wrap',
        alignItems: 'flex-start',
        gap: 'var(--settings-group-gap)',
        boxSizing: 'border-box',
        width: '100%',
      }}
    >
      {/* Column A: Startup & Vault Folder */}
      <div
        data-testid="settings-column-a"
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--settings-group-gap)',
          flex: '1 1 var(--settings-column-min)',
          minWidth: 'min(100%, var(--settings-column-min))',
          boxSizing: 'border-box',
        }}
      >
        <GeneralSection
          startupStatus={startupStatus}
          onToggleStartup={onToggleStartup}
          onRetryStartup={onRetryStartup}
          disabled={disabled}
        />
        <VaultSection
          vaultPath={settings.vault_path}
          onSaveVaultPath={onSaveVaultPath}
          onOpenExplorer={onOpenExplorer}
          disabled={disabled}
        />
      </div>

      {/* Column B: Quality Budget & Hotkeys */}
      <div
        data-testid="settings-column-b"
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--settings-group-gap)',
          flex: '1 1 var(--settings-column-min)',
          minWidth: 'min(100%, var(--settings-column-min))',
          boxSizing: 'border-box',
        }}
      >
        <QualityBudgetSection
          qualityBudget={settings.quality_budget}
          latestFindingSize={settings.latest_finding_size}
          latestFinding={settings.latest_finding}
          onSaveQualityBudget={onSaveQualityBudget}
          disabled={disabled}
        />
        <HotkeySection
          hotkeySettings={hotkeySettings}
          onSaveHotkey={onSaveHotkey}
          onClearHotkey={onClearHotkey}
          disabled={disabled}
        />
      </div>
    </div>
  );
};
