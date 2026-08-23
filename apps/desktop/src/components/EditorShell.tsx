import React from 'react';

export type NavigationTab = 'findings' | 'bundles' | 'agent-access' | 'settings';

export interface EditorShellProps {
  activeTab: NavigationTab;
  onTabChange: (tab: NavigationTab) => void;
  onCaptureClick?: () => void;
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
  children,
}) => {
  return (
    <div
      data-testid="editor-shell"
      style={{
        display: 'flex',
        flexDirection: 'row',
        height: '100vh',
        width: '100vw',
        backgroundColor: 'var(--color-bg)',
        color: 'var(--color-text)',
        fontFamily: 'var(--font-ui)',
        overflow: 'hidden',
      }}
    >
      {/* 200px Left Navigation Rail (LC-028) */}
      <nav
        data-testid="navigation-rail"
        aria-label="Main Navigation"
        style={{
          width: '200px',
          minWidth: '200px',
          maxWidth: '200px',
          height: '100%',
          backgroundColor: 'var(--color-surface)',
          borderRight: '1px solid var(--color-border)',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}
      >
        {/* Product Brand Header */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--space-3)',
            padding: 'var(--space-4) var(--space-4) var(--space-3)',
          }}
        >
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
              flexShrink: 0,
            }}
          >
            S
          </div>
          <span
            style={{
              fontSize: 'var(--text-lg)',
              fontWeight: 700,
              letterSpacing: '-0.02em',
              color: 'var(--color-text)',
            }}
          >
            Snapdown
          </span>
        </div>

        {/* Primary Surface Tabs */}
        <div
          role="tablist"
          aria-orientation="vertical"
          style={{
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-1)',
            padding: 'var(--space-2) var(--space-2)',
          }}
        >
          {NAV_ITEMS.map((item) => {
            const isActive = activeTab === item.id;
            return (
              <button
                key={item.id}
                role="tab"
                id={`tab-${item.id}`}
                aria-selected={isActive}
                aria-controls={`panel-${item.id}`}
                data-testid={`nav-item-${item.id}`}
                onClick={() => onTabChange(item.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  width: '100%',
                  textAlign: 'left',
                  padding: 'var(--space-2) var(--space-3)',
                  borderRadius: 'var(--radius-md)',
                  borderTop: 'none',
                  borderRight: 'none',
                  borderBottom: 'none',
                  borderLeftWidth: '4px',
                  borderLeftStyle: 'solid',
                  borderLeftColor: isActive
                    ? 'var(--color-accent-text)'
                    : 'transparent',
                  backgroundColor: isActive ? 'var(--color-accent)' : 'transparent',
                  color: isActive ? 'var(--color-accent-text)' : 'var(--color-text-muted)',
                  fontWeight: isActive ? 600 : 500,
                  fontSize: 'var(--text-sm)',
                  cursor: 'pointer',
                  transition: 'all 0.15s ease',
                  outline: 'none',
                }}
              >
                {item.label}
              </button>
            );
          })}
        </div>

        {/* Pinned Capture Action Button at Rail Foot */}
        <div
          style={{
            marginTop: 'auto',
            padding: 'var(--space-3)',
            borderTop: '1px solid var(--color-border)',
          }}
        >
          <button
            type="button"
            data-testid="rail-capture-btn"
            onClick={onCaptureClick}
            style={{
              width: '100%',
              padding: 'var(--space-2) var(--space-3)',
              borderRadius: 'var(--radius-md)',
              border: 'none',
              backgroundColor: 'var(--color-accent)',
              color: 'var(--color-accent-text)',
              fontWeight: 600,
              fontSize: 'var(--text-sm)',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 'var(--space-2)',
              transition: 'opacity 0.15s ease',
            }}
          >
            <span>?</span>
            <span>Capture</span>
          </button>
        </div>
      </nav>

      {/* Main Content Area */}
      <main
        role="tabpanel"
        id={`panel-${activeTab}`}
        aria-labelledby={`tab-${activeTab}`}
        style={{
          flex: 1,
          height: '100%',
          overflowY: 'auto',
          backgroundColor: 'var(--color-bg)',
          color: 'var(--color-text)',
        }}
      >
        {children}
      </main>
    </div>
  );
};
