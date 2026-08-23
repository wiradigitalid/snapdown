import React, { useState } from 'react';
import { Button } from '@snapdown/ui';
import { cleanOrphans, OrphanScanReportDto, scanOrphans } from '../services/finding';

export const OrphanReportView: React.FC = () => {
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
    <div data-testid="orphan-report-view" style={{ padding: '16px', display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h2 style={{ margin: 0, fontSize: '16px', fontWeight: 600 }}>Orphan Files Report</h2>
          <p style={{ margin: '4px 0 0 0', fontSize: '12px', color: '#64748b' }}>
            Detect files in vault that have no database reference, or missing files referenced by findings.
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
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
            padding: '8px 12px',
            backgroundColor: '#f1f5f9',
            borderRadius: '6px',
            fontSize: '13px',
            color: '#334155',
          }}
        >
          {statusMsg}
        </div>
      )}

      {report && (
        <div data-testid="orphan-summary-card" style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(4, 1fr)',
              gap: '12px',
              padding: '12px',
              backgroundColor: '#f8fafc',
              border: '1px solid #e2e8f0',
              borderRadius: '6px',
              fontSize: '13px',
            }}
          >
            <div><strong>Vault Files:</strong> {report.total_vault_files}</div>
            <div><strong>Referenced:</strong> {report.referenced_files}</div>
            <div><strong>Orphan Files:</strong> {report.orphan_files.length}</div>
            <div><strong>Missing Files:</strong> {report.missing_files.length}</div>
          </div>

          {report.orphan_files.length > 0 && (
            <div>
              <h4 style={{ margin: '0 0 8px 0', fontSize: '13px' }}>Unreferenced Disk Files (Orphans)</h4>
              <ul data-testid="orphan-files-list" style={{ margin: 0, paddingLeft: '20px', fontSize: '12px', color: '#dc2626' }}>
                {report.orphan_files.map((file) => (
                  <li key={file}>{file}</li>
                ))}
              </ul>
            </div>
          )}

          {report.missing_files.length > 0 && (
            <div>
              <h4 style={{ margin: '0 0 8px 0', fontSize: '13px' }}>Missing Finding Files</h4>
              <ul data-testid="missing-files-list" style={{ margin: 0, paddingLeft: '20px', fontSize: '12px', color: '#d97706' }}>
                {report.missing_files.map((file) => (
                  <li key={file}>{file}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
