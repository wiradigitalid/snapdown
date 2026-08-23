import React, { useMemo } from 'react';
import { Button } from './Button';
import { Badge } from './Badge';
import { EmptyState } from './EmptyState';
import { ErrorState } from './ErrorState';

export interface BundleDto {
  id: string;
  name: string;
  markdown: string;
  markdown_path: string;
  composed_at: string;
}

export interface BundleItemDto {
  id: string;
  bundle_id: string;
  finding_id: string;
  position: number;
  image_path: string;
  note_first_line?: string;
  is_missing?: boolean;
}

export interface BundleDetailDto {
  bundle: BundleDto;
  items: BundleItemDto[];
}

export interface BundlesEditorProps {
  bundles: BundleDetailDto[];
  selectedBundleId: string | null;
  isLoading?: boolean;
  error?: string | null;
  onSelectBundle: (id: string) => void;
  onCopyMarkdown: (bundleId: string) => Promise<void> | void;
  onPublishBundle?: (bundleId: string) => void;
  onDeleteBundle?: (bundleId: string) => Promise<void> | void;
  onRetry?: () => void;
}

export function formatComposedDate(isoString: string): string {
  try {
    const d = new Date(isoString);
    if (isNaN(d.getTime())) return isoString;
    const day = d.getDate();
    const month = d.toLocaleString('en-US', { month: 'short' });
    return `${day} ${month}`;
  } catch {
    return isoString;
  }
}

export const BundlesEditor: React.FC<BundlesEditorProps> = ({
  bundles,
  selectedBundleId,
  isLoading = false,
  error = null,
  onSelectBundle,
  onCopyMarkdown,
  onPublishBundle,
  onDeleteBundle,
  onRetry,
}) => {
  const sortedBundles = useMemo(() => {
    return [...bundles].sort((a, b) => {
      const timeA = new Date(a.bundle.composed_at).getTime() || 0;
      const timeB = new Date(b.bundle.composed_at).getTime() || 0;
      return timeB - timeA;
    });
  }, [bundles]);

  const selectedBundle = useMemo(() => {
    return bundles.find((b) => b.bundle.id === selectedBundleId) || null;
  }, [bundles, selectedBundleId]);

  const sortedItems = useMemo(() => {
    if (!selectedBundle) return [];
    return [...selectedBundle.items].sort((a, b) => (a.position || 0) - (b.position || 0));
  }, [selectedBundle]);

  if (error) {
    return (
      <div
        data-testid="bundles-error-state"
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          width: '100%',
          padding: 'var(--space-6)',
          backgroundColor: 'var(--color-bg)',
        }}
      >
        <ErrorState
          title="The Library could not be read"
          message={error}
          actionLabel="Retry"
          onAction={onRetry}
        />
      </div>
    );
  }

  if (!isLoading && bundles.length === 0) {
    return (
      <div
        data-testid="bundles-empty-state"
        style={{
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          height: '100%',
          width: '100%',
          backgroundColor: 'var(--color-bg)',
          padding: 'var(--space-6)',
        }}
      >
        <EmptyState
          heading="No bundles yet"
          description="Select findings on the Findings tab and choose Compose."
        />
      </div>
    );
  }

  return (
    <div
      data-testid="bundles-editor"
      style={{
        display: 'flex',
        flexDirection: 'row',
        height: '100%',
        width: '100%',
        backgroundColor: 'var(--color-bg)',
        overflow: 'hidden',
      }}
    >
      {/* COLUMN 1: Bundle List (240px) */}
      <div
        data-testid="bundle-list-pane"
        style={{
          width: 'var(--bundle-list-width, 240px)',
          minWidth: 'var(--bundle-list-width, 240px)',
          maxWidth: 'var(--bundle-list-width, 240px)',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          borderRight: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-surface)',
          overflowY: 'auto',
        }}
      >
        <div
          style={{
            padding: 'var(--space-3) var(--space-4)',
            borderBottom: '1px solid var(--color-border)',
          }}
        >
          <h3
            style={{
              margin: 0,
              fontFamily: 'var(--font-ui)',
              fontSize: 'var(--text-sm)',
              fontWeight: 600,
              color: 'var(--color-text)',
            }}
          >
            Bundles ({bundles.length})
          </h3>
        </div>

        {isLoading ? (
          <div
            data-testid="bundle-list-loading-skeleton"
            style={{
              padding: 'var(--space-3)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
            }}
          >
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                data-testid="bundle-skeleton-row"
                style={{
                  height: '52px',
                  borderRadius: 'var(--radius-sm)',
                  backgroundColor: 'var(--color-surface-sunken)',
                  opacity: 0.6,
                }}
              />
            ))}
          </div>
        ) : (
          <ul
            role="listbox"
            aria-label="Bundles list"
            style={{
              listStyle: 'none',
              padding: 'var(--space-2)',
              margin: 0,
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-1)',
              flex: 1,
              overflowY: 'auto',
            }}
          >
            {sortedBundles.map((b) => {
              const isSelected = b.bundle.id === selectedBundleId;
              const itemCountText = `${b.items.length} ${b.items.length === 1 ? 'item' : 'items'}`;
              const composedDateText = formatComposedDate(b.bundle.composed_at);

              return (
                <li
                  key={b.bundle.id}
                  role="option"
                  aria-selected={isSelected}
                  tabIndex={0}
                  data-testid={`bundle-item-${b.bundle.id}`}
                  onClick={() => onSelectBundle(b.bundle.id)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      onSelectBundle(b.bundle.id);
                    }
                  }}
                  style={{
                    padding: 'var(--space-2) var(--space-3)',
                    borderRadius: 'var(--radius-sm)',
                    cursor: 'pointer',
                    backgroundColor: isSelected ? 'var(--color-info-bg)' : 'transparent',
                    border: isSelected
                      ? '1px solid var(--color-accent)'
                      : '1px solid transparent',
                    transition: 'background-color 0.15s ease',
                  }}
                >
                  <div
                    style={{
                      fontFamily: 'var(--font-ui)',
                      fontWeight: isSelected ? 600 : 500,
                      fontSize: 'var(--text-sm)',
                      color: isSelected ? 'var(--color-info-text)' : 'var(--color-text)',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {b.bundle.name}
                  </div>
                  <div
                    style={{
                      fontFamily: 'var(--font-ui)',
                      fontSize: 'var(--text-xs)',
                      color: 'var(--color-text-muted)',
                      marginTop: 'var(--space-0)',
                    }}
                  >
                    {itemCountText} · {composedDateText}
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>

      {/* COLUMN 2: Markdown Preview (Flex Growing Centre) */}
      <div
        data-testid="bundle-preview-pane"
        style={{
          flex: 1,
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          backgroundColor: 'var(--color-surface-sunken)',
          overflow: 'hidden',
          minWidth: 0,
        }}
      >
        {selectedBundle ? (
          <div
            role="region"
            aria-label="Markdown Preview"
            data-testid="bundle-markdown-preview"
            style={{
              flex: 1,
              height: '100%',
              overflowY: 'auto',
              padding: 'var(--space-4)',
              boxSizing: 'border-box',
              userSelect: 'text',
              cursor: 'default',
              fontFamily: 'var(--font-mono)',
              fontSize: 'var(--text-sm)',
              lineHeight: 'var(--preview-line-height, 1.55)',
              color: 'var(--color-text)',
              whiteSpace: 'pre-wrap',
              wordBreak: 'break-word',
            }}
          >
            {selectedBundle.bundle.markdown}
          </div>
        ) : (
          <div
            data-testid="bundle-preview-empty"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              color: 'var(--color-text-muted)',
              fontFamily: 'var(--font-ui)',
              fontSize: 'var(--text-sm)',
            }}
          >
            Select a bundle to preview content.
          </div>
        )}
      </div>

      {/* COLUMN 3: Item List (280px) */}
      <div
        data-testid="bundle-items-pane"
        style={{
          width: 'var(--item-list-width, 280px)',
          minWidth: 'var(--item-list-width, 280px)',
          maxWidth: 'var(--item-list-width, 280px)',
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          borderLeft: '1px solid var(--color-border)',
          backgroundColor: 'var(--color-surface)',
        }}
      >
        {/* Header */}
        <div
          style={{
            padding: 'var(--space-3) var(--space-4)',
            borderBottom: '1px solid var(--color-border)',
          }}
        >
          <h4
            style={{
              margin: 0,
              fontFamily: 'var(--font-ui)',
              fontSize: 'var(--text-sm)',
              fontWeight: 600,
              color: 'var(--color-text)',
            }}
          >
            Findings ({selectedBundle?.items.length || 0})
          </h4>
        </div>

        {/* Items List */}
        <div
          style={{
            flex: 1,
            overflowY: 'auto',
            padding: 'var(--space-2)',
          }}
        >
          {selectedBundle ? (
            <ul
              data-testid="bundle-item-list"
              style={{
                listStyle: 'none',
                padding: 0,
                margin: 0,
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--space-1)',
              }}
            >
              {sortedItems.map((item, idx) => {
                const position = item.position || idx + 1;
                const label = item.note_first_line || item.image_path || `Finding ${position}`;
                const isMissing = Boolean(item.is_missing);

                return (
                  <li
                    key={item.id || `item-${idx}`}
                    data-testid={`bundle-item-row-${item.id || idx}`}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      gap: 'var(--space-2)',
                      padding: 'var(--space-2) var(--space-3)',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: 'var(--color-surface-sunken)',
                      fontSize: 'var(--text-xs)',
                    }}
                  >
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 'var(--space-2)',
                        minWidth: 0,
                        flex: 1,
                      }}
                    >
                      <span
                        style={{
                          fontWeight: 700,
                          color: 'var(--color-text-muted)',
                          fontFamily: 'var(--font-mono)',
                          flexShrink: 0,
                        }}
                      >
                        {position}
                      </span>
                      <span
                        style={{
                          color: 'var(--color-text)',
                          fontFamily: 'var(--font-ui)',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                        }}
                        title={label}
                      >
                        {label}
                      </span>
                    </div>
                    {isMissing && (
                      <Badge variant="warning" data-testid="item-missing-badge">
                        Missing
                      </Badge>
                    )}
                  </li>
                );
              })}
            </ul>
          ) : (
            <div
              style={{
                padding: 'var(--space-4)',
                textAlign: 'center',
                color: 'var(--color-text-muted)',
                fontSize: 'var(--text-xs)',
              }}
            >
              No bundle selected
            </div>
          )}
        </div>

        {/* Pinned Action Footer */}
        {selectedBundle && (
          <div
            data-testid="bundle-action-footer"
            style={{
              padding: 'var(--space-3)',
              borderTop: '1px solid var(--color-border)',
              backgroundColor: 'var(--color-surface)',
              display: 'flex',
              flexDirection: 'column',
              gap: 'var(--space-2)',
            }}
          >
            <Button
              variant="primary"
              data-testid="copy-markdown-btn"
              onClick={() => onCopyMarkdown(selectedBundle.bundle.id)}
              style={{ width: '100%' }}
            >
              Copy Markdown
            </Button>

            <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
              <Button
                variant="secondary"
                data-testid="publish-bundle-btn"
                onClick={() => onPublishBundle && onPublishBundle(selectedBundle.bundle.id)}
                disabled={true}
                style={{ flex: 1 }}
                title="Publishing is frozen (DEC-005)"
              >
                Publish
              </Button>

              <Button
                variant="danger"
                data-testid="delete-bundle-btn"
                onClick={() => onDeleteBundle && onDeleteBundle(selectedBundle.bundle.id)}
                style={{ flex: 1 }}
              >
                Delete
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
