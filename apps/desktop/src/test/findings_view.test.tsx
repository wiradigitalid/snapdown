import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { FindingsView } from '../components/FindingsView';
import * as findingService from '../services/finding';

vi.mock('../services/finding', () => ({
  listFindings: vi.fn(),
  getFindingDetail: vi.fn(),
  saveNote: vi.fn(),
  deleteFinding: vi.fn(),
}));

const mockFindings = [
  {
    finding: {
      id: 'fid-1',
      image_path: 'findings/finding-1.png',
      image_width: 1920,
      image_height: 1080,
      captured_at: '2026-08-23T10:00:00Z',
      source_monitor: 'DISPLAY1',
      region: '0,0,1920,1080',
    },
    note: {
      id: 'note-1',
      finding_id: 'fid-1',
      body: 'Sample note content',
      updated_at: '2026-08-23T10:00:00Z',
    },
    markers: [],
  },
];

describe('FindingsView Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders findings view and loads findings from service', async () => {
    vi.mocked(findingService.listFindings).mockResolvedValue(mockFindings);

    render(<FindingsView />);

    await waitFor(() => {
      expect(screen.getByTestId('findings-view')).toBeInTheDocument();
      expect(screen.getByTestId('finding-item-fid-1')).toBeInTheDocument();
      expect(screen.getByDisplayValue('Sample note content')).toBeInTheDocument();
    });
  });
});
