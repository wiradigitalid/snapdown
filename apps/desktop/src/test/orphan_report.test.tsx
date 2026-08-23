import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { OrphanReportView } from '../components/OrphanReportView';
import * as findingService from '../services/finding';

vi.mock('../services/finding', () => ({
  scanOrphans: vi.fn(),
  cleanOrphans: vi.fn(),
}));

describe('OrphanReportView Component (LC-030)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('orphan_report_displays_discrepancies_and_cleans', async () => {
    vi.mocked(findingService.scanOrphans).mockResolvedValue({
      total_vault_files: 5,
      referenced_files: 3,
      orphan_files: ['findings/orphan_1.png', 'findings/orphan_2.png'],
      missing_files: ['findings/missing_3.png'],
    });
    vi.mocked(findingService.cleanOrphans).mockResolvedValue(2);

    render(<OrphanReportView />);

    const scanBtn = screen.getByRole('button', { name: 'Scan Vault' });
    fireEvent.click(scanBtn);

    await waitFor(() => {
      expect(screen.getByTestId('orphan-summary-card')).toBeInTheDocument();
      expect(screen.getByText('findings/orphan_1.png')).toBeInTheDocument();
      expect(screen.getByText('findings/orphan_2.png')).toBeInTheDocument();
      expect(screen.getByText('findings/missing_3.png')).toBeInTheDocument();
    });

    const cleanBtn = screen.getByRole('button', { name: 'Clean 2 Orphan(s)' });
    fireEvent.click(cleanBtn);

    await waitFor(() => {
      expect(findingService.cleanOrphans).toHaveBeenCalledWith([
        'findings/orphan_1.png',
        'findings/orphan_2.png',
      ]);
    });
  });

  it('calls onBack when back button is clicked', () => {
    const onBack = vi.fn();
    render(<OrphanReportView onBack={onBack} />);

    const backBtn = screen.getByTestId('orphan-back-button');
    expect(backBtn).toBeInTheDocument();
    fireEvent.click(backBtn);
    expect(onBack).toHaveBeenCalledTimes(1);
  });
});
