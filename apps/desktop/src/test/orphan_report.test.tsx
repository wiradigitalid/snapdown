import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { OrphanReportView } from '../components/OrphanReportView';
import * as findingService from '../services/finding';

vi.mock('../services/finding', () => ({
  scanOrphans: vi.fn(),
  cleanOrphans: vi.fn(),
}));

describe('OrphanReportView Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('orphan_report_displays_discrepancies', async () => {
    vi.mocked(findingService.scanOrphans).mockResolvedValue({
      total_vault_files: 5,
      referenced_files: 3,
      orphan_files: ['findings/orphan_1.png', 'findings/orphan_2.png'],
      missing_files: ['findings/missing_3.png'],
    });

    render(<OrphanReportView />);

    const scanBtn = screen.getByRole('button', { name: 'Scan Vault' });
    fireEvent.click(scanBtn);

    await waitFor(() => {
      expect(screen.getByTestId('orphan-summary-card')).toBeInTheDocument();
      expect(screen.getByText('findings/orphan_1.png')).toBeInTheDocument();
      expect(screen.getByText('findings/orphan_2.png')).toBeInTheDocument();
      expect(screen.getByText('findings/missing_3.png')).toBeInTheDocument();
    });
  });
});
