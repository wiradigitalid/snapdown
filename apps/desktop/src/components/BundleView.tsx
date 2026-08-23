import React, { useCallback, useEffect, useState } from 'react';
import { BundleComposer, Button } from '@snapdown/ui';
import {
  BundleDetailDto,
  copyBundleToClipboard,
  createBundle,
  deleteBundle,
  listBundles,
} from '../services/bundle';
import { FindingDetailDto, listFindings } from '../services/finding';

export const BundleView: React.FC = () => {
  const [bundles, setBundles] = useState<BundleDetailDto[]>([]);
  const [findings, setFindings] = useState<FindingDetailDto[]>([]);
  const [selectedBundleId, setSelectedBundleId] = useState<string | null>(null);
  const [showComposer, setShowComposer] = useState(false);
  const [copyFeedback, setCopyFeedback] = useState(false);

  const fetchAll = useCallback(async () => {
    try {
      const [bundleList, findingList] = await Promise.all([listBundles(), listFindings()]);
      setBundles(bundleList);
      setFindings(findingList);
      if (bundleList.length > 0 && (!selectedBundleId || !bundleList.some((b) => b.bundle.id === selectedBundleId))) {
        setSelectedBundleId(bundleList[0].bundle.id);
      }
    } catch (err) {
      console.error('Failed to load bundles/findings:', err);
    }
  }, [selectedBundleId]);

  useEffect(() => {
    fetchAll();
  }, [fetchAll]);

  const handleCreateBundle = async (name: string, findingIds: string[]) => {
    await createBundle({ name, finding_ids: findingIds });
    setShowComposer(false);
    await fetchAll();
  };

  const handleDeleteBundle = async (id: string) => {
    await deleteBundle(id);
    await fetchAll();
  };

  const handleCopyMarkdown = async (id: string) => {
    try {
      const text = await copyBundleToClipboard(id);
      if (navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      }
      setCopyFeedback(true);
      setTimeout(() => setCopyFeedback(false), 2000);
    } catch (err) {
      console.error('Failed to copy markdown:', err);
    }
  };

  const selectedBundle = bundles.find((b) => b.bundle.id === selectedBundleId);

  return (
    <div data-testid="bundle-view" style={{ padding: 'var(--space-4)', display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: 'var(--text-lg)', fontWeight: 600, color: 'var(--color-text)' }}>Documentation Bundles</h1>
          <p style={{ margin: 'var(--space-1) 0 0 0', fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
            Compose and export multi-finding markdown reviews.
          </p>
        </div>
        <Button variant="primary" onClick={() => setShowComposer((prev) => !prev)}>
          {showComposer ? 'Close Composer' : 'Compose New Bundle'}
        </Button>
      </div>

      {showComposer && (
        <BundleComposer
          findings={findings}
          onCreateBundle={handleCreateBundle}
          onCancel={() => setShowComposer(false)}
        />
      )}

      <div style={{ display: 'flex', gap: 'var(--space-4)', minHeight: '360px' }}>
        {/* Bundle List */}
        <div
          data-testid="bundle-list-pane"
          style={{
            width: '260px',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            padding: 'var(--space-3)',
            backgroundColor: 'var(--color-bg)',
          }}
        >
          <h3 style={{ margin: '0 0 var(--space-3) 0', fontSize: 'var(--text-sm)', fontWeight: 600, color: 'var(--color-text)' }}>
            Bundles ({bundles.length})
          </h3>
          {bundles.length === 0 ? (
            <p style={{ fontSize: 'var(--text-sm)', color: 'var(--color-text-muted)' }}>No bundles created yet.</p>
          ) : (
            <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: 'var(--space-2)' }}>
              {bundles.map((b) => {
                const isSelected = b.bundle.id === selectedBundleId;
                return (
                  <li
                    key={b.bundle.id}
                    data-testid={`bundle-item-${b.bundle.id}`}
                    onClick={() => setSelectedBundleId(b.bundle.id)}
                    style={{
                      padding: 'var(--space-2) var(--space-3)',
                      borderRadius: 'var(--radius-sm)',
                      cursor: 'pointer',
                      backgroundColor: isSelected ? 'var(--color-info-bg)' : 'var(--color-surface)',
                      border: isSelected ? '1px solid var(--color-accent)' : '1px solid var(--color-border)',
                    }}
                  >
                    <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)', color: isSelected ? 'var(--color-info-text)' : 'var(--color-text)' }}>
                      {b.bundle.name}
                    </div>
                    <div style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
                      {b.items.length} items • {b.bundle.composed_at}
                    </div>
                  </li>
                );
              })}
            </ul>
          )}
        </div>

        {/* Bundle Details & Markdown Preview */}
        <div
          data-testid="bundle-detail-pane"
          style={{
            flex: 1,
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            padding: 'var(--space-4)',
            backgroundColor: 'var(--color-surface)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-3)',
          }}
        >
          {selectedBundle ? (
            <>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2 style={{ margin: 0, fontSize: 'var(--text-base)', fontWeight: 600, color: 'var(--color-text)' }}>
                  {selectedBundle.bundle.name}
                </h2>
                <div style={{ display: 'flex', gap: 'var(--space-2)', alignItems: 'center' }}>
                  <Button variant="primary" onClick={() => handleCopyMarkdown(selectedBundle.bundle.id)}>
                    Copy Markdown
                  </Button>
                  {copyFeedback && (
                    <span data-testid="copy-feedback-msg" style={{ fontSize: 'var(--text-xs)', color: 'var(--color-success-text)', fontWeight: 600 }}>
                      Copied!
                    </span>
                  )}
                  <Button variant="secondary" onClick={() => handleDeleteBundle(selectedBundle.bundle.id)}>
                    Delete Bundle
                  </Button>
                </div>
              </div>

              <div style={{ fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
                Path: <code>{selectedBundle.bundle.markdown_path}</code>
              </div>

              <div>
                <h4 style={{ margin: '0 0 var(--space-2) 0', fontSize: 'var(--text-sm)', color: 'var(--color-text)' }}>Markdown Preview</h4>
                <pre
                  data-testid="bundle-markdown-preview"
                  style={{
                    backgroundColor: 'var(--color-surface-sunken)',
                    color: 'var(--color-text)',
                    padding: 'var(--space-3)',
                    borderRadius: 'var(--radius-sm)',
                    fontSize: 'var(--text-xs)',
                    fontFamily: 'var(--font-mono)',
                    whiteSpace: 'pre-wrap',
                    maxHeight: '300px',
                    overflowY: 'auto',
                    border: '1px solid var(--color-border)',
                  }}
                >
                  {selectedBundle.bundle.markdown}
                </pre>
              </div>
            </>
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--color-text-muted)' }}>
              Select a bundle to preview content.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
