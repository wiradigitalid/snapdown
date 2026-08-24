import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { BundleComposer } from '../components/BundleComposer';

const mockFindings = [
  {
    finding: {
      id: 'f1',
      image_path: 'findings/finding-1.png',
      image_width: 1920,
      image_height: 1080,
      captured_at: '2026-08-23T10:00:00Z',
      source_monitor: 'DISPLAY1',
      region: '0,0,1920,1080',
    },
    note: {
      id: 'n1',
      finding_id: 'f1',
      body: 'Note 1',
      updated_at: '2026-08-23T10:00:00Z',
    },
    markers: [],
  },
  {
    finding: {
      id: 'f2',
      image_path: 'findings/finding-2.png',
      image_width: 800,
      image_height: 600,
      captured_at: '2026-08-23T11:00:00Z',
      source_monitor: 'DISPLAY1',
      region: '0,0,800,600',
    },
    note: {
      id: 'n2',
      finding_id: 'f2',
      body: 'Note 2',
      updated_at: '2026-08-23T11:00:00Z',
    },
    markers: [],
  },
];

describe('BundleComposer Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders selected findings with visual preview and triggers onCreateBundle', async () => {
    const onCreateBundle = vi.fn().mockResolvedValue(undefined);

    render(
      <BundleComposer
        findings={mockFindings}
        onCreateBundle={onCreateBundle}
      />
    );

    const titleInput = screen.getByLabelText('Bundle Title');
    fireEvent.change(titleInput, { target: { value: 'Alpha Release Review' } });

    // Verify row items are listed
    expect(screen.getByTestId('bundle-finding-row-f1')).toBeInTheDocument();
    expect(screen.getByTestId('bundle-finding-row-f2')).toBeInTheDocument();

    // Verify preview modes
    expect(screen.getByTestId('preview-mode-visual-btn')).toBeInTheDocument();
    expect(screen.getByTestId('preview-mode-md-btn')).toBeInTheDocument();

    const submitBtn = screen.getByRole('button', { name: 'Create Bundle' });
    fireEvent.click(submitBtn);

    await waitFor(() => {
      expect(onCreateBundle).toHaveBeenCalledWith('Alpha Release Review', ['f1', 'f2']);
    });
  });
});
