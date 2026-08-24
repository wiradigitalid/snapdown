import React from 'react';
import { BundleDetailDto, formatComposedDate } from './BundlesEditor';

export interface BundlesDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  bundles: BundleDetailDto[];
  onSelectBundle: (id: string) => void;
  onCopyMarkdown: (id: string) => void;
  onDeleteBundle?: (id: string) => void;
  className?: string;
  style?: React.CSSProperties;
}

export const BundlesDrawer: React.FC<BundlesDrawerProps> = ({
  isOpen,
  onClose,
  bundles,
  onSelectBundle,
  onCopyMarkdown,
  onDeleteBundle,
  className = '',
  style,
}) => {
  if (!isOpen) return null;

  return (
    <div
      data-testid="bundles-drawer-backdrop"
      onClick={onClose}
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'var(--color-overlay-scrim)',
        backdropFilter: 'blur(3px)',
        zIndex: 250,
        display: 'flex',
        justifyContent: 'flex-end',
      }}
    >
      <div
        data-testid="bundles-drawer"
        className={`bundles-drawer ${className}`.trim()}
        onClick={(e) => e.stopPropagation()}
        style={{
          width: '380px',
          maxWidth: '100vw',
          height: '100%',
          backgroundColor: 'var(--color-surface)',
          borderLeft: '1px solid var(--color-border)',
          boxShadow: 'var(--shadow-xl)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          ...style,
        }}
      >
        {/* Drawer Header */}
        <div
          style={{
            padding: 'var(--space-3) var(--space-4)',
            borderBottom: '1px solid var(--color-border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            backgroundColor: 'var(--color-surface)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
            <span style={{ fontSize: '1.1rem' }}>📚</span>
            <h3 style={{ margin: 0, fontSize: 'var(--text-sm)', fontWeight: 800, color: 'var(--color-text)' }}>
              Saved Bundles History
            </h3>
          </div>
          <button
            type="button"
            data-testid="close-bundles-drawer-btn"
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
        </div>

        {/* Drawer List */}
        <div
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: 'var(--space-3)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-2)',
          }}
        >
          {bundles.length === 0 ? (
            <div style={{ padding: 'var(--space-6)', textAlign: 'center', color: 'var(--color-text-muted)', fontSize: 'var(--text-xs)' }}>
              No saved bundles found in vault.
            </div>
          ) : (
            bundles.map((b) => (
              <div
                key={b.bundle.id}
                data-testid={`drawer-bundle-card-${b.bundle.id}`}
                style={{
                  padding: 'var(--space-3)',
                  backgroundColor: 'var(--color-surface-sunken)',
                  border: '1px solid var(--color-border)',
                  borderRadius: 'var(--radius-sm)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 'var(--space-2)',
                }}
              >
                <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', gap: 'var(--space-2)' }}>
                  <span style={{ fontWeight: 700, fontSize: 'var(--text-xs)', color: 'var(--color-text)' }}>
                    {b.bundle.name}
                  </span>
                  <span style={{ fontSize: 'var(--text-2xs)', color: 'var(--color-text-muted)', fontFamily: 'var(--font-mono)' }}>
                    {formatComposedDate(b.bundle.composed_at)}
                  </span>
                </div>

                <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--color-text-muted)', display: 'flex', gap: 'var(--space-2)' }}>
                  <span>📦 {b.items.length} items</span>
                  <span>·</span>
                  <span>MD Generated</span>
                </div>

                <div style={{ display: 'flex', gap: 'var(--space-2)', marginTop: 'var(--space-1)' }}>
                  <button
                    type="button"
                    data-testid={`drawer-copy-md-btn-${b.bundle.id}`}
                    onClick={() => onCopyMarkdown(b.bundle.id)}
                    style={{
                      flex: 1,
                      padding: 'var(--space-1) var(--space-2)',
                      backgroundColor: 'var(--color-surface)',
                      border: '1px solid var(--color-border)',
                      borderRadius: 'var(--radius-xs)',
                      fontSize: 'var(--text-2xs)',
                      fontWeight: 600,
                      color: 'var(--color-text)',
                      cursor: 'pointer',
                    }}
                  >
                    📋 Copy MD
                  </button>

                  <button
                    type="button"
                    data-testid={`drawer-view-btn-${b.bundle.id}`}
                    onClick={() => onSelectBundle(b.bundle.id)}
                    style={{
                      flex: 1,
                      padding: 'var(--space-1) var(--space-2)',
                      backgroundColor: 'var(--color-accent)',
                      border: 'none',
                      borderRadius: 'var(--radius-xs)',
                      fontSize: 'var(--text-2xs)',
                      fontWeight: 700,
                      color: 'var(--color-accent-text)',
                      cursor: 'pointer',
                    }}
                  >
                    👁️ View
                  </button>

                  {onDeleteBundle && (
                    <button
                      type="button"
                      data-testid={`drawer-delete-btn-${b.bundle.id}`}
                      onClick={() => onDeleteBundle(b.bundle.id)}
                      style={{
                        padding: 'var(--space-1) var(--space-2)',
                        backgroundColor: 'transparent',
                        border: '1px solid var(--color-danger-bg)',
                        borderRadius: 'var(--radius-xs)',
                        fontSize: 'var(--text-2xs)',
                        color: 'var(--color-danger)',
                        cursor: 'pointer',
                      }}
                    >
                      🗑️
                    </button>
                  )}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
};
