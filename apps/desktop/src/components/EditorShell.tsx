import React from 'react';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export type NavigationTab = 'findings' | 'bundles' | 'agent-access' | 'settings';

export interface EditorShellProps {
  activeTab: NavigationTab;
  onTabChange: (tab: NavigationTab) => void;
  onCaptureClick?: () => void;
  onOpenHistory?: () => void;
  onOpenSettings?: () => void;
  activeFindingTitle?: string;
  children: React.ReactNode;
}

interface NavItem {
  id: NavigationTab;
  label: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: 'findings', label: 'Findings' },
  { id: 'bundles', label: 'Bundles' },
  { id: 'agent-access', label: 'Agent Access' },
  { id: 'settings', label: 'Settings' },
];

export const EditorShell: React.FC<EditorShellProps> = ({
  activeTab,
  onTabChange,
  onCaptureClick,
  onOpenHistory,
  onOpenSettings,
  activeFindingTitle = 'Studio Workspace',
  children,
}) => {
  const handleMinimize = async () => {
    try {
      const appWindow = getCurrentWebviewWindow();
      await appWindow.minimize();
    } catch {
      // Ignored outside Tauri
    }
  };

  const handleMaximize = async () => {
    try {
      const appWindow = getCurrentWebviewWindow();
      await appWindow.toggleMaximize();
    } catch {
      // Ignored outside Tauri
    }
  };

  const handleClose = async () => {
    try {
      const appWindow = getCurrentWebviewWindow();
      await appWindow.close();
    } catch {
      // Ignored outside Tauri
    }
  };

  return (
    <div
      data-testid="editor-shell"
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
      {/* 34px Frameless Desktop Titlebar (SPEC-01 / FR-SHELL-1) */}
      <header
        data-testid="studio-titlebar"
        style={{
          height: '34px',
          minHeight: '34px',
          backgroundColor: 'var(--snagit-ribbon-bg)',
          borderBottom: '1px solid var(--color-border)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 0 0 var(--space-3)',
          fontSize: 'var(--text-xs)',
          userSelect: 'none',
          boxSizing: 'border-box',
          flexShrink: 0,
        }}
      >
        {/* Left Drag Region: Logo, Brand & Active Finding Pill */}
        <div
          data-tauri-drag-region="true"
          style={{
            flex: 1,
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--space-2)',
            cursor: 'default',
            overflow: 'hidden',
          }}
        >
          <div
            style={{
              width: '18px',
              height: '18px',
              borderRadius: 'var(--radius-xs)',
              backgroundColor: 'var(--color-accent)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--color-accent-text)',
              fontWeight: 800,
              fontSize: '0.65rem',
            }}
          >
            ⚡
          </div>
          <span style={{ fontWeight: 800, color: 'var(--color-text)', letterSpacing: '-0.02em' }}>
            Snapdown Studio
          </span>
          <span style={{ color: 'var(--color-border-strong)' }}>|</span>
          <span
            data-testid="titlebar-finding-pill"
            style={{
              color: 'var(--color-text-muted)',
              fontSize: 'var(--text-2xs)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {activeFindingTitle}
          </span>
        </div>

        {/* Right Action Buttons & Window Controls */}
        <div style={{ display: 'flex', alignItems: 'center', height: '100%' }}>
          {onOpenHistory && (
            <button
              type="button"
              data-testid="titlebar-history-btn"
              onClick={onOpenHistory}
              style={{
                height: '24px',
                padding: '0 var(--space-2)',
                fontSize: 'var(--text-2xs)',
                fontWeight: 700,
                backgroundColor: 'transparent',
                border: '1px solid var(--color-border)',
                borderRadius: 'var(--radius-xs)',
                color: 'var(--color-text)',
                cursor: 'pointer',
                marginRight: 'var(--space-2)',
                display: 'inline-flex',
                alignItems: 'center',
                gap: '4px',
              }}
            >
              <span>📚</span>
              <span>Bundles</span>
            </button>
          )}

          {onOpenSettings && (
            <button
              type="button"
              data-testid="titlebar-settings-btn"
              onClick={onOpenSettings}
              style={{
                height: '24px',
                padding: '0 var(--space-2)',
                fontSize: 'var(--text-2xs)',
                fontWeight: 700,
                backgroundColor: 'transparent',
                border: '1px solid var(--color-border)',
                borderRadius: 'var(--radius-xs)',
                color: 'var(--color-text)',
                cursor: 'pointer',
                marginRight: 'var(--space-2)',
                display: 'inline-flex',
                alignItems: 'center',
                gap: '4px',
              }}
            >
              <span>⚙️</span>
              <span>Settings</span>
            </button>
          )}

          {/* 44px OS Window Controls */}
          <button
            type="button"
            className="win-control-btn"
            data-testid="win-minimize-btn"
            title="Minimize"
            onClick={handleMinimize}
          >
            🗕
          </button>
          <button
            type="button"
            className="win-control-btn"
            data-testid="win-maximize-btn"
            title="Maximize"
            onClick={handleMaximize}
          >
            🗖
          </button>
          <button
            type="button"
            className="win-control-btn btn-close"
            data-testid="win-close-btn"
            title="Close"
            onClick={handleClose}
          >
            ✕
          </button>
        </div>
      </header>

      {/* Main Studio Viewport */}
      <main
        role="tabpanel"
        id={`panel-${activeTab}`}
        aria-labelledby={`tab-${activeTab}`}
        style={{
          flex: 1,
          height: 'calc(100vh - 34px)',
          overflow: 'hidden',
          backgroundColor: 'var(--color-bg)',
          color: 'var(--color-text)',
          display: 'flex',
          flexDirection: 'column',
        }}
      >
        {children}
      </main>

      {/* Accessible Nav Rail (Visually hidden for screen readers / programmatic tab control) */}
      <div
        style={{
          position: 'absolute',
          width: '1px',
          height: '1px',
          padding: 0,
          margin: '-1px',
          overflow: 'hidden',
          clip: 'rect(0, 0, 0, 0)',
          whiteSpace: 'nowrap',
          border: 0,
        }}
      >
        <nav
          data-testid="navigation-rail"
          aria-label="Main Navigation"
          style={{ width: '200px' }}
        >
          <span>Snapdown</span>
          <div role="tablist">
            {NAV_ITEMS.map((item) => {
              const isActive = activeTab === item.id;
              return (
                <button
                  key={item.id}
                  role="tab"
                  id={`tab-${item.id}`}
                  aria-selected={isActive}
                  className="nav-rail-item"
                  onClick={() => onTabChange(item.id)}
                  style={{
                    backgroundColor: isActive ? 'var(--color-accent)' : 'transparent',
                    borderLeftWidth: '4px',
                    borderLeftStyle: 'solid',
                    borderLeftColor: isActive ? 'var(--color-accent-text)' : 'transparent',
                  }}
                >
                  {item.label}
                </button>
              );
            })}
          </div>
          <button
            type="button"
            className="rail-capture-btn"
            data-testid="rail-capture-btn"
            onClick={onCaptureClick}
          >
            Capture
          </button>
        </nav>
      </div>
    </div>
  );
};
