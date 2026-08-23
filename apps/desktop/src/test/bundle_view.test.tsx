import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { BundleView } from '../components/BundleView';
import * as bundleService from '../services/bundle';

vi.mock('../services/bundle', () => ({
  listBundles: vi.fn(),
  createBundle: vi.fn(),
  getBundleDetail: vi.fn(),
  deleteBundle: vi.fn(),
  copyBundleToClipboard: vi.fn(),
}));

const mockBundles = [
  {
    bundle: {
      id: 'b-1',
      name: 'Sprint 1 Review',
      markdown: '# Sprint 1\n\n![](./img/f-01.png)\n\n1. Button alignment issue',
      markdown_path: 'bundles/b-1.md',
      composed_at: '2026-08-23T12:00:00Z',
    },
    items: [
      {
        id: 'item-1',
        bundle_id: 'b-1',
        finding_id: 'f-1',
        position: 1,
        image_path: 'findings/f-1.png',
        note_first_line: 'Button alignment issue',
      },
    ],
  },
  {
    bundle: {
      id: 'b-2',
      name: 'Sprint 2 Review',
      markdown: '# Sprint 2\n\n2. Navbar overflow',
      markdown_path: 'bundles/b-2.md',
      composed_at: '2026-08-24T10:00:00Z',
    },
    items: [
      {
        id: 'item-2',
        bundle_id: 'b-2',
        finding_id: 'f-2',
        position: 1,
        image_path: 'findings/f-2.png',
        note_first_line: 'Navbar overflow',
      },
    ],
  },
];

describe('BundleView Container Component (W6-S8)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders bundle view with loaded bundles and auto-selects first bundle', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('bundle-view')).toBeInTheDocument();
      expect(screen.getByTestId('bundles-editor')).toBeInTheDocument();
      expect(screen.getByTestId('bundle-item-b-2')).toBeInTheDocument();
      expect(screen.getByTestId('bundle-item-b-1')).toBeInTheDocument();
    });

    const preview = screen.getByTestId('bundle-markdown-preview');
    expect(preview).toBeInTheDocument();
    expect(preview).toHaveAttribute('role', 'region');
    expect(preview).toHaveAttribute('aria-label', 'Markdown Preview');
  });

  it('copies markdown to clipboard and announces result with toast', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);
    vi.mocked(bundleService.copyBundleToClipboard).mockResolvedValue(
      '# Sprint 2\n\n2. Navbar overflow'
    );

    const writeTextMock = vi.fn().mockResolvedValue(undefined);
    Object.assign(navigator, {
      clipboard: {
        writeText: writeTextMock,
      },
    });

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('copy-markdown-btn')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId('copy-markdown-btn'));

    await waitFor(() => {
      expect(bundleService.copyBundleToClipboard).toHaveBeenCalledWith('b-2');
      expect(writeTextMock).toHaveBeenCalledWith('# Sprint 2\n\n2. Navbar overflow');
      expect(screen.getByText('Markdown copied to clipboard')).toBeInTheDocument();
    });
  });

  it('opens delete confirmation dialog and deletes bundle on confirm', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);
    vi.mocked(bundleService.deleteBundle).mockResolvedValue(undefined);

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('delete-bundle-btn')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId('delete-bundle-btn'));

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Delete Bundle' })).toBeInTheDocument();
      expect(
        screen.getByText(/The bundle's markdown and image copies will be permanently deleted from the vault/)
      ).toBeInTheDocument();
      expect(
        screen.getByText(/Original findings will remain intact in your library/)
      ).toBeInTheDocument();
    });

    // Click confirm in modal
    const confirmButtons = screen.getAllByRole('button', { name: 'Delete Bundle' });
    const modalDeleteBtn = confirmButtons[confirmButtons.length - 1];
    fireEvent.click(modalDeleteBtn);

    await waitFor(() => {
      expect(bundleService.deleteBundle).toHaveBeenCalledWith('b-2');
    });
  });

  it('renders honest empty state when no bundles exist', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue([]);

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('bundles-empty-state')).toBeInTheDocument();
      expect(screen.getByText('No bundles yet')).toBeInTheDocument();
      expect(
        screen.getByText('Select findings on the Findings tab and choose Compose.')
      ).toBeInTheDocument();
    });

    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('renders error state and retries fetching when service fails', async () => {
    vi.mocked(bundleService.listBundles).mockRejectedValueOnce(new Error('Library database locked'));

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('bundles-error-state')).toBeInTheDocument();
      expect(screen.getByText('The Library could not be read')).toBeInTheDocument();
      expect(screen.getByText('Library database locked')).toBeInTheDocument();
    });

    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);
    fireEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.getByTestId('bundles-editor')).toBeInTheDocument();
      expect(screen.getByTestId('bundle-item-b-1')).toBeInTheDocument();
    });
  });

  it('switches selected bundle when item in list is clicked', async () => {
    vi.mocked(bundleService.listBundles).mockResolvedValue(mockBundles);

    render(<BundleView />);

    await waitFor(() => {
      expect(screen.getByTestId('bundle-item-b-1')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId('bundle-item-b-1'));

    await waitFor(() => {
      expect(screen.getByTestId('bundle-markdown-preview')).toHaveTextContent('# Sprint 1');
      expect(screen.getByText('Button alignment issue')).toBeInTheDocument();
    });
  });
});
