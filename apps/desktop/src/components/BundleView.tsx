import React, { useCallback, useEffect, useState } from 'react';
import { BundleComposer, Button } from '@snapdown/ui';
import { BundleDetailDto, createBundle, deleteBundle, listBundles } from '../services/bundle';
import { FindingDetailDto, listFindings } from '../services/finding';

export const BundleView: React.FC = () => {
  const [bundles, setBundles] = useState<BundleDetailDto[]>([]);
  const [findings, setFindings] = useState<FindingDetailDto[]>([]);
  const [selectedBundleId, setSelectedBundleId] = useState<string | null>(null);
  const [showComposer, setShowComposer] = useState(false);

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

  const selectedBundle = bundles.find((b) => b.bundle.id === selectedBundleId);

  return (
    <div data-testid="bundle-view" style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ margin: 0, fontSize: '18px', fontWeight: 600 }}>Documentation Bundles</h1>
          <p style={{ margin: '4px 0 0 0', fontSize: '12px', color: '#64748b' }}>
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

      <div style={{ display: 'flex', gap: '16px', minHeight: '360px' }}>
        {/* Bundle List */}
        <div
          data-testid="bundle-list-pane"
          style={{
            width: '260px',
            border: '1px solid #e2e8f0',
            borderRadius: '6px',
            padding: '12px',
            backgroundColor: '#f8fafc',
          }}
        >
          <h3 style={{ margin: '0 0 12px 0', fontSize: '14px', fontWeight: 600 }}>
            Bundles ({bundles.length})
          </h3>
          {bundles.length === 0 ? (
            <p style={{ fontSize: '13px', color: '#64748b' }}>No bundles created yet.</p>
          ) : (
            <ul style={{ listStyle: 'none', padding: 0, margin: 0, display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {bundles.map((b) => {
                const isSelected = b.bundle.id === selectedBundleId;
                return (
                  <li
                    key={b.bundle.id}
                    data-testid={`bundle-item-${b.bundle.id}`}
                    onClick={() => setSelectedBundleId(b.bundle.id)}
                    style={{
                      padding: '8px 12px',
                      borderRadius: '6px',
                      cursor: 'pointer',
                      backgroundColor: isSelected ? '#e0f2fe' : '#ffffff',
                      border: isSelected ? '1px solid #3b82f6' : '1px solid #e2e8f0',
                    }}
                  >
                    <div style={{ fontWeight: 500, fontSize: '13px' }}>{b.bundle.name}</div>
                    <div style={{ fontSize: '11px', color: '#64748b' }}>
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
            border: '1px solid #e2e8f0',
            borderRadius: '6px',
            padding: '16px',
            backgroundColor: '#ffffff',
            display: 'flex',
            flexDirection: 'column',
            gap: '12px',
          }}
        >
          {selectedBundle ? (
            <>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <h2 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>
                  {selectedBundle.bundle.name}
                </h2>
                <Button variant="secondary" onClick={() => handleDeleteBundle(selectedBundle.bundle.id)}>
                  Delete Bundle
                </Button>
              </div>

              <div style={{ fontSize: '12px', color: '#64748b' }}>
                Path: <code>{selectedBundle.bundle.markdown_path}</code>
              </div>

              <div>
                <h4 style={{ margin: '0 0 6px 0', fontSize: '13px' }}>Markdown Preview</h4>
                <pre
                  data-testid="bundle-markdown-preview"
                  style={{
                    backgroundColor: '#f1f5f9',
                    padding: '12px',
                    borderRadius: '6px',
                    fontSize: '12px',
                    fontFamily: 'var(--font-mono, monospace)',
                    whiteSpace: 'pre-wrap',
                    maxHeight: '300px',
                    overflowY: 'auto',
                  }}
                >
                  {selectedBundle.bundle.markdown}
                </pre>
              </div>
            </>
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#94a3b8' }}>
              Select a bundle to preview content.
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
