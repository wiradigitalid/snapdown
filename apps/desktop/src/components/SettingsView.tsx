import React, { useState, useEffect } from 'react';
import appIconSrc from '../assets/app-icon.png';
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

export type SettingsTab = 'general' | 'agent-bridge' | 'about';

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
  initialTab?: SettingsTab;
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
  initialTab = 'general',
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>(initialTab);

  // Keyboard shortcut Esc to close / back to editor
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && onClose) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [onClose]);

  const tabs: { id: SettingsTab; label: string; icon: string }[] = [
    { id: 'general', label: 'General & Quality', icon: '⚙️' },
    { id: 'agent-bridge', label: 'Local Agent Bridge', icon: '🤖' },
    { id: 'about', label: 'About', icon: 'ℹ️' },
  ];

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
        alignContent: 'flex-start',
        gap: 'var(--settings-group-gap)',
        boxSizing: 'border-box',
        width: '100%',
        height: '100%',
        overflowY: 'auto',
      }}
    >
      {/* Top Navigation Bar with Tabs & Back Button */}
      <div
        data-testid="settings-tabs-header"
        style={{
          width: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          borderBottom: '1px solid var(--color-border)',
          paddingBottom: 'var(--space-3)',
          marginBottom: 'var(--space-1)',
          flexShrink: 0,
          gap: 'var(--space-2)',
        }}
      >
        {/* Horizontal Navigation Pills */}
        <div
          role="tablist"
          aria-label="Settings Categories"
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: 'var(--space-1)',
            backgroundColor: 'var(--color-surface-sunken)',
            padding: '3px',
            borderRadius: 'var(--radius-md)',
            border: '1px solid var(--color-border)',
          }}
        >
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                type="button"
                role="tab"
                id={`settings-tab-btn-${tab.id}`}
                aria-selected={isActive}
                aria-controls={`settings-tabpanel-${tab.id}`}
                data-testid={`settings-tab-${tab.id}`}
                onClick={() => setActiveTab(tab.id)}
                style={{
                  display: 'inline-flex',
                  alignItems: 'center',
                  gap: 'var(--space-1)',
                  padding: 'var(--space-1) var(--space-3)',
                  borderRadius: 'var(--radius-sm)',
                  border: 'none',
                  backgroundColor: isActive ? 'var(--color-surface)' : 'transparent',
                  color: isActive ? 'var(--color-text)' : 'var(--color-text-muted)',
                  fontWeight: isActive ? 700 : 500,
                  fontSize: 'var(--text-xs)',
                  cursor: 'pointer',
                  boxShadow: isActive ? 'var(--shadow-sm)' : 'none',
                  transition: 'all 0.15s ease',
                }}
              >
                <span>{tab.icon}</span>
                <span>{tab.label}</span>
              </button>
            );
          })}
        </div>

        {/* Prominent Back to Editor Button */}
        {onClose && (
          <button
            type="button"
            data-testid="settings-close-btn"
            aria-label="Back to Snapdown Editor"
            title="Back to Snapdown Editor (Esc)"
            onClick={onClose}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: 'var(--space-2)',
              padding: 'var(--space-1) var(--space-3)',
              backgroundColor: 'var(--color-surface-sunken)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-md)',
              fontSize: 'var(--text-xs)',
              fontWeight: 600,
              color: 'var(--color-text)',
              cursor: 'pointer',
              transition: 'all 0.15s ease',
              flexShrink: 0,
            }}
          >
            <span>←</span>
            <span>Back to Editor</span>
          </button>
        )}
      </div>

      {/* TAB 1: General & Quality (2-Column Unified Preferences) */}
      {activeTab === 'general' && (
        <>
          {/* Column A: Startup & Vault Storage */}
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

      {/* TAB 2: Local Agent Bridge & Token Management (Top-Aligned Full Region) */}
      {activeTab === 'agent-bridge' && (
        <div
          id="settings-tabpanel-agent-bridge"
          role="tabpanel"
          aria-labelledby="settings-tab-btn-agent-bridge"
          data-testid="settings-tabpanel-agent-bridge"
          style={{
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'flex-start',
            alignItems: 'stretch',
            gap: 'var(--space-4)',
            width: '100%',
            boxSizing: 'border-box',
          }}
        >
          {/* Bridge Status Summary Card */}
          <div
            style={{
              padding: 'var(--space-4)',
              backgroundColor: 'var(--color-surface)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-md)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
              width: '100%',
              boxSizing: 'border-box',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <span style={{ fontWeight: 700, fontSize: 'var(--text-sm)', color: 'var(--color-text)' }}>
                Local Loopback Bridge
              </span>
              <span
                style={{
                  color: 'var(--color-success-text)',
                  backgroundColor: 'var(--color-success-bg)',
                  padding: '2px 8px',
                  borderRadius: 'var(--radius-sm)',
                  fontWeight: 800,
                  fontSize: 'var(--text-2xs)',
                }}
              >
                🟢 Port 3849 Active
              </span>
            </div>
            <p style={{ margin: 0, fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
              Provides stdio & HTTP bridge for Claude Code, Codex, and OpenCode AI agents to observe and capture UI states.
            </p>
          </div>

          {/* Access Key Management Panel */}
          <div style={{ width: '100%', boxSizing: 'border-box' }}>
            {agentAccessContent}
          </div>
        </div>
      )}

      {/* TAB 3: About Snapdown & Architecture (Full Height Region, Equal Margins) */}
      {activeTab === 'about' && (
        <div
          id="settings-tabpanel-about"
          role="tabpanel"
          aria-labelledby="settings-tab-btn-about"
          data-testid="settings-tabpanel-about"
          style={{
            width: '100%',
            minHeight: '480px',
            flex: '1 1 100%',
            padding: 'var(--space-8) var(--space-6)',
            backgroundColor: 'var(--color-surface)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-md)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            textAlign: 'center',
            gap: 'var(--space-4)',
            boxSizing: 'border-box',
            justifyContent: 'center',
          }}
        >
          <img
            src={appIconSrc}
            alt="Snapdown Logo"
            style={{
              width: '72px',
              height: '72px',
              borderRadius: 'var(--radius-lg)',
              objectFit: 'contain',
              boxShadow: 'var(--shadow-md)',
            }}
          />
          <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-1)', alignItems: 'center' }}>
            <h2 style={{ margin: 0, fontSize: 'var(--text-xl)', fontWeight: 800, color: 'var(--color-text)', letterSpacing: '-0.02em' }}>
              Snapdown
            </h2>
            <span
              style={{
                fontSize: 'var(--text-xs)',
                color: 'var(--color-text-muted)',
                fontFamily: 'var(--font-mono)',
                backgroundColor: 'var(--color-surface-sunken)',
                padding: '2px 8px',
                borderRadius: 'var(--radius-xs)',
              }}
            >
              Version 1.4.0 (Tauri v2 · x64 Windows 11)
            </span>
          </div>
          <p style={{ margin: 0, maxWidth: '34rem', fontSize: 'var(--text-sm)', color: 'var(--color-text-secondary)', lineHeight: 1.6 }}>
            Visual UI/UX Observation & Multimodal Handoff Tool built for AI Coding Agents. Developed under WDI Method by Wira Digital Indonesia.
          </p>
          <div style={{ display: 'flex', gap: 'var(--space-2)', marginTop: 'var(--space-2)' }}>
            <span style={{ fontSize: 'var(--text-2xs)', color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}>
              Port 3849 · Vault Integration · AES-256 GCM Keyring
            </span>
          </div>
        </div>
      )}
    </div>
  );
};
