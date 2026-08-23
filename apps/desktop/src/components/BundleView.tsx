import React, { useCallback, useEffect, useState } from 'react';
import { BundlesEditor, ConfirmDialog, Toast, BundleDetailDto } from '@snapdown/ui';
import {
  copyBundleToClipboard,
  deleteBundle,
  listBundles,
} from '../services/bundle';

export const BundleView: React.FC = () => {
  const [bundles, setBundles] = useState<BundleDetailDto[]>([]);
  const [selectedBundleId, setSelectedBundleId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [bundleToDelete, setBundleToDelete] = useState<BundleDetailDto | null>(null);
  const [isDeleting, setIsDeleting] = useState<boolean>(false);

  const fetchBundles = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const bundleList = await listBundles();
      setBundles(bundleList);
      setSelectedBundleId((current) => {
        if (bundleList.length > 0) {
          const sorted = [...bundleList].sort((a, b) => {
            const timeA = new Date(a.bundle.composed_at).getTime() || 0;
            const timeB = new Date(b.bundle.composed_at).getTime() || 0;
            return timeB - timeA;
          });
          if (!current || !sorted.some((b) => b.bundle.id === current)) {
            return sorted[0].bundle.id;
          }
          return current;
        }
        return null;
      });
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(msg);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBundles();
  }, [fetchBundles]);

  const handleCopyMarkdown = async (id: string) => {
    try {
      const text = await copyBundleToClipboard(id);
      if (navigator.clipboard) {
        await navigator.clipboard.writeText(text);
      }
      setToastMessage('Markdown copied to clipboard');
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setToastMessage(`Failed to copy markdown: ${msg}`);
    }
  };

  const handleDeleteClick = (id: string) => {
    const target = bundles.find((b) => b.bundle.id === id);
    if (target) {
      setBundleToDelete(target);
    }
  };

  const handleConfirmDelete = async () => {
    if (!bundleToDelete) return;
    setIsDeleting(true);
    try {
      await deleteBundle(bundleToDelete.bundle.id);
      setToastMessage(`Bundle "${bundleToDelete.bundle.name}" deleted`);
      setBundleToDelete(null);
      await fetchBundles();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setToastMessage(`Failed to delete bundle: ${msg}`);
    } finally {
      setIsDeleting(false);
    }
  };

  return (
    <div data-testid="bundle-view" style={{ width: '100%', height: '100%' }}>
      <BundlesEditor
        bundles={bundles}
        selectedBundleId={selectedBundleId}
        isLoading={isLoading}
        error={error}
        onSelectBundle={(id) => setSelectedBundleId(id)}
        onCopyMarkdown={handleCopyMarkdown}
        onDeleteBundle={handleDeleteClick}
        onRetry={fetchBundles}
      />

      {/* Accessible live region / toast for copy & delete feedback */}
      {toastMessage && (
        <Toast
          message={toastMessage}
          onDismiss={() => setToastMessage(null)}
          durationMs={3000}
        />
      )}

      {/* Confirmation Dialog for Bundle Deletion (FR-14) */}
      <ConfirmDialog
        isOpen={Boolean(bundleToDelete)}
        title="Delete Bundle"
        message={
          bundleToDelete
            ? `Are you sure you want to delete "${bundleToDelete.bundle.name}"? The bundle's markdown and image copies will be permanently deleted from the vault. Original findings will remain intact in your library.`
            : ''
        }
        confirmLabel="Delete Bundle"
        cancelLabel="Cancel"
        loading={isDeleting}
        onConfirm={handleConfirmDelete}
        onCancel={() => setBundleToDelete(null)}
      />
    </div>
  );
};
