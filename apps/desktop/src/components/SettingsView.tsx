import React, { useState } from 'react';
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

export type SettingsTab = 'general' | 'hotkeys' | 'agent-bridge' | 'about';

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
  onClose?: () => void;
  agentAccessContent?: React.ReactNode;
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
  onClose,
  agentAccessContent,
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');

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
      {/* Tab bar spanning 100% width */}
      <div
        data-testid="settings-tabs-header"
        style={{
          width: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          borderBottom: '1px solid var(--color-border)',
          paddingBottom: 'var(--space-2)',
          marginBottom: 'var(--space-2)',
        }}
      >
        <div style={{ display: 'flex', gap: 'var(--space-1)', backgroundColor: 'var(--color-surface-sunken)', padding: '2px', borderRadius: 'var(--radius-md)' }}>
          {[
            { id: 'general', label: '⚙️ General & Quality' },
            { id: 'hotkeys', label: '⌨️ Hotkeys' },
            { id: 'agent-bridge', label: '🤖 Local Agent Bridge' },
            { id: 'about', label: 'ℹ️ About' },
          ].map((tab) => (
            <button
              key={tab.id}
              type="button"
              data-testid={`settings-tab-${tab.id}`}
              onClick={() => setActiveTab(tab.id as SettingsTab)}
              style={{
                padding: 'var(--space-1) var(--space-3)',
                borderRadius: 'var(--radius-sm)',
                border: 'none',
                backgroundColor: activeTab === tab.id ? 'var(--color-surface)' : 'transparent',
                color: activeTab === tab.id ? 'var(--color-text)' : 'var(--color-text-muted)',
                fontWeight: activeTab === tab.id ? 700 : 500,
                fontSize: 'var(--text-xs)',
                cursor: 'pointer',
                boxShadow: activeTab === tab.id ? 'var(--shadow-sm)' : 'none',
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {onClose && (
          <button
            type="button"
            data-testid="settings-close-btn"
            onClick={onClose}
            style={{
              background: 'none',
              border: 'none',
              fontSize: '1rem',
              color: 'var(--color-text-muted)',
              cursor: 'pointer',
            }}
          >
            ✕
          </button>
        )}
      </div>

      {/* TAB 1: General & Quality (Height-packed 2 columns) */}
      {activeTab === 'general' && (
        <>
          {/* Column A: Startup & Vault */}
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
        </>
      )}

      {/* TAB 2: Shortcuts & Hotkeys Detail */}
      {activeTab === 'hotkeys' && (
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 'var(--space-3)' }}>
          <HotkeySection
            hotkeySettings={hotkeySettings}
            onSaveHotkey={onSaveHotkey}
            onClearHotkey={onClearHotkey}
            disabled={disabled}
          />
        </div>
      )}

      {/* TAB 3: Local Agent Bridge & Access Token */}
      {activeTab === 'agent-bridge' && (
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: 'var(--space-3)' }}>
          <div
            style={{
              padding: 'var(--space-4)',
              backgroundColor: 'var(--color-surface)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontWeight: 700, fontSize: 'var(--text-sm)' }}>Local Loopback Bridge</span>
              <span style={{ color: 'var(--color-success-text)', backgroundColor: 'var(--color-success-bg)', padding: '2px 8px', borderRadius: 'var(--radius-sm)', fontWeight: 800, fontSize: 'var(--text-2xs)' }}>
                🟢 Port 3849 Active
              </span>
            </div>
            <p style={{ margin: 0, fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
              Provides stdio & HTTP bridge for Claude Code, Codex, and OpenCode AI agents.
            </p>
          </div>
          {agentAccessContent}
        </div>
      )}

      {/* TAB 4: About & System Info */}
      {activeTab === 'about' && (
        <div
          style={{
            width: '100%',
            padding: 'var(--space-6)',
            backgroundColor: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            textAlign: 'center',
            gap: 'var(--space-3)',
          }}
        >
          <div style={{ width: '48px', height: '48px', borderRadius: 'var(--radius-md)', backgroundColor: 'var(--color-accent)', color: 'var(--color-accent-text)', fontSize: '1.5rem', display: 'flex', alignItems: 'center', justifyContent: 'center', fontWeight: 800 }}>
            ⚡
          </div>
          <div>
            <h2 style={{ margin: 0, fontSize: 'var(--text-lg)', fontWeight: 800, color: 'var(--color-text)' }}>
              Snapdown Studio
            </h2>
            <span style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}>
              Version 1.4.0 (Tauri v2 · x64 Windows 11)
            </span>
          </div>
          <p style={{ margin: 0, maxWidth: '32rem', fontSize: 'var(--text-sm)', color: 'var(--color-text-secondary)', lineHeight: 1.5 }}>
            Visual UI/UX Observation & Multimodal Handoff Tool built for AI Coding Agents. Developed under WDI Method by Wira Digital Indonesia.
          </p>
        </div>
      )}
    </div>
  );
};
