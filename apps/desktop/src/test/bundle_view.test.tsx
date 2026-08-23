import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BundleView } from '../components/BundleView';
import * as bundleService from '../services/bundle';
import * as findingService from '../services/finding';

vi.mock('../services/bundle', () => ({
  listBundles: vi.fn(),
  createBundle: vi.fn(),
  getBundleDetail: vi.fn(),
  deleteBundle: vi.fn(),
}));

vi.mock('../services/finding', () => ({
  listFindings: vi.fn(),
}));

const mockBundles = [
  {
    bundle: {
      id: 'b-1',
      name: 'Sprint 1 Bundle',
      markdown: '# Sprint 1\n\n- Finding 1',
      markdown_path: 'bundles/b-1.md',
      composed_at: '2026-08-23T12:00:00Z',
    },
    items: [],
  },
];

describe('BundleView Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders bundle view and displays bundle preview', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);
    vi.mocked(findingService.listFindings).mockResolvedValue([]);

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('bundle-view')).toBeInTheDocument();
      expect(screen.getByTestId('bundle-item-b-1')).toBeInTheDocument();
      expect(screen.getAllByText('Sprint 1 Bundle').length).toBeGreaterThanOrEqual(1);
      expect(screen.getByTestId('bundle-markdown-preview')).toHaveTextContent('# Sprint 1');
    });
  });
});
