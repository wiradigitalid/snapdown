import React, { useState } from 'react';
import { Button, EmptyState } from '@snapdown/ui';
import { cleanOrphans, OrphanScanReportDto, scanOrphans } from '../services/finding';

export interface OrphanReportViewProps {
  onBack?: () => void;
}

export const OrphanReportView: React.FC<OrphanReportViewProps> = ({ onBack }) => {
  const [report, setReport] = useState<OrphanScanReportDto | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [isCleaning, setIsCleaning] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const handleScan = async () => {
    setIsScanning(true);
    setStatusMsg(null);
    try {
      const res = await scanOrphans();
      setReport(res);
      if (res.orphan_files.length === 0 && res.missing_files.length === 0) {
        setStatusMsg('Vault is clean! No orphan or missing files detected.');
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Scan failed: ${msg}`);
    } finally {
      setIsScanning(false);
    }
  };

  const handleClean = async () => {
    if (!report || report.orphan_files.length === 0) return;
    setIsCleaning(true);
    try {
      const count = await cleanOrphans(report.orphan_files);
      setStatusMsg(`Successfully removed ${count} orphan file(s).`);
      await handleScan();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setStatusMsg(`Clean failed: ${msg}`);
    } finally {
      setIsCleaning(false);
    }
  };

  return (
    <div
      data-testid="orphan-report-view"
      style={{
        padding: 'var(--space-5)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
        maxWidth: '56rem',
        margin: '0 auto',
        width: '100%',
        height: '100%',
        boxSizing: 'border-box',
        overflowY: 'auto',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-3)' }}>
          {onBack && (
            <Button
              variant="secondary"
              data-testid="orphan-back-button"
              onClick={onBack}
            >
              ← Back to Findings
            </Button>
          )}
          <div>
            <h2 style={{ margin: 0, fontSize: 'var(--text-base)', fontWeight: 600, color: 'var(--color-text)' }}>
              Orphan Files Report
            </h2>
            <p style={{ margin: 'var(--space-1) 0 0 0', fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
              Detect files in vault that have no database reference, or missing files referenced by findings.
            </p>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Button variant="primary" onClick={handleScan} disabled={isScanning}>
            {isScanning ? 'Scanning...' : 'Scan Vault'}
          </Button>
          {report && report.orphan_files.length > 0 && (
            <Button variant="secondary" onClick={handleClean} disabled={isCleaning}>
              {isCleaning ? 'Cleaning...' : `Clean ${report.orphan_files.length} Orphan(s)`}
            </Button>
          )}
        </div>
      </div>

      {statusMsg && (
        <div
          data-testid="orphan-status-message"
          style={{
            padding: 'var(--space-2) var(--space-3)',
            backgroundColor: 'var(--color-neutral-bg)',
            borderRadius: 'var(--radius-sm)',
            fontSize: 'var(--text-sm)',
            color: 'var(--color-neutral-text)',
          }}
        >
          {statusMsg}
        </div>
      )}

      {report && (
        <div data-testid="orphan-summary-card" style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(4, 1fr)',
              gap: 'var(--space-3)',
              padding: 'var(--space-3)',
              backgroundColor: 'var(--color-bg)',
              border: '1px solid var(--color-border)',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-sm)',
              color: 'var(--color-text)',
            }}
          >
            <div><strong>Vault Files:</strong> {report.total_vault_files}</div>
            <div><strong>Referenced:</strong> {report.referenced_files}</div>
            <div><strong>Orphan Files:</strong> {report.orphan_files.length}</div>
            <div><strong>Missing Files:</strong> {report.missing_files.length}</div>
          </div>

          {report.orphan_files.length === 0 && report.missing_files.length === 0 ? (
            <EmptyState
              heading="Nothing orphaned"
              description="All files in the vault match database references cleanly."
            />
          ) : (
            <>
              {report.orphan_files.length > 0 && (
                <div>
                  <h4 style={{ margin: '0 0 var(--space-2) 0', fontSize: 'var(--text-sm)', color: 'var(--color-text)' }}>
                    Unreferenced Disk Files (Orphans)
                  </h4>
                  <ul data-testid="orphan-files-list" style={{ margin: 0, paddingLeft: 'var(--space-5)', fontSize: 'var(--text-xs)', color: 'var(--color-danger)' }}>
                    {report.orphan_files.map((file) => (
                      <li key={file}>{file}</li>
                    ))}
                  </ul>
                </div>
              )}

              {report.missing_files.length > 0 && (
                <div>
                  <h4 style={{ margin: '0 0 var(--space-2) 0', fontSize: 'var(--text-sm)', color: 'var(--color-text)' }}>
                    Missing Finding Files
                  </h4>
                  <ul data-testid="missing-files-list" style={{ margin: 0, paddingLeft: 'var(--space-5)', fontSize: 'var(--text-xs)', color: 'var(--color-warning-text)' }}>
                    {report.missing_files.map((file) => (
                      <li key={file}>{file}</li>
                    ))}
                  </ul>
                </div>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
};
