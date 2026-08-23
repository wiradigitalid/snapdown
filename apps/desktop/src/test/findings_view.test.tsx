import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { FindingsView } from '../components/FindingsView';
import * as findingService from '../services/finding';
import * as settingsService from '../services/settings';

vi.mock('../services/finding', () => ({
  listFindings: vi.fn(),
  getFindingDetail: vi.fn(),
  saveNote: vi.fn(),
  deleteFinding: vi.fn(),
  addMarker: vi.fn(),
  updateMarker: vi.fn(),
  deleteMarker: vi.fn(),
  scanOrphans: vi.fn(),
  cleanOrphans: vi.fn(),
}));

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  getHotkeys: vi.fn(),
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
      body: '1. Button misaligned\n2. Contrast issue\n3. Spacing bug',
      updated_at: '2026-08-23T10:00:00Z',
    },
    markers: [
      {
        id: 'm1',
        finding_id: 'fid-1',
        ordinal: 1,
        x: 0.2,
        y: 0.3,
        comment: 'Marker 1',
      },
      {
        id: 'm2',
        finding_id: 'fid-1',
        ordinal: 2,
        x: 0.5,
        y: 0.5,
        comment: 'Marker 2',
      },
      {
        id: 'm3',
        finding_id: 'fid-1',
        ordinal: 3,
        x: 0.8,
        y: 0.7,
        comment: 'Marker 3',
      },
    ],
  },
];

describe('FindingsView Composition Tests (BUG-5, BUG-6, SCN-04)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: '/path/to/vault',
      quality_budget: { named: 'balanced', max_long_edge: 1600, encoder_quality: 75 },
      latest_finding_size: 188416,
    });
    vi.mocked(settingsService.getHotkeys).mockResolvedValue({
      hotkeys: [
        {
          action: 'capture',
          shortcut: 'CommandOrControl+Shift+S',
          is_registered: true,
          is_active: true,
        },
      ],
      startup_warnings: [],
    });
    vi.mocked(findingService.listFindings).mockResolvedValue(mockFindings);
  });

  it('BUG-5 composition: renders screenshot image with convertFileSrc and mounts MarkerLayer canvas directly over it', async () => {
    render(<FindingsView />);

    await waitFor(() => {
      expect(screen.getByTestId('findings-view')).toBeInTheDocument();
      expect(screen.getByTestId('findings-editor')).toBeInTheDocument();
    });

    // 1. Assert screenshot <img> element is mounted in DOM with resolved convertFileSrc URL
    const img = screen.getByTestId('finding-image');
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute(
      'src',
      'asset://localhost/%2Fpath%2Fto%2Fvault%2Ffindings%2Ffinding-1.png'
    );

    // 2. Assert MarkerLayer is mounted directly over the image in the canvas viewport
    const markerLayer = screen.getByTestId('marker-layer');
    expect(markerLayer).toBeInTheDocument();

    // 3. Assert marker badges 1, 2, 3 are rendered
    expect(screen.getByTestId('marker-badge-1')).toBeInTheDocument();
    expect(screen.getByTestId('marker-badge-2')).toBeInTheDocument();
    expect(screen.getByTestId('marker-badge-3')).toBeInTheDocument();

    // 4. Assert readout is present
    expect(screen.getByTestId('canvas-readout')).toHaveTextContent('1920 × 1080 px');
  });

  it('composition: clicking canvas places new marker and appends numbered line to note', async () => {
    vi.mocked(findingService.addMarker).mockResolvedValue({
      id: 'm4',
      finding_id: 'fid-1',
      ordinal: 4,
      x: 0.6,
      y: 0.6,
      comment: 'Marker 4',
    });
    vi.mocked(findingService.saveNote).mockResolvedValue(undefined);

    render(<FindingsView />);

    await waitFor(() => {
      expect(screen.getByTestId('marker-layer')).toBeInTheDocument();
    });

    const layer = screen.getByTestId('marker-layer');
    vi.spyOn(layer, 'getBoundingClientRect').mockReturnValue({
      left: 0,
      top: 0,
      width: 1000,
      height: 600,
      right: 1000,
      bottom: 600,
      x: 0,
      y: 0,
      toJSON: () => {},
    });

    // Click at 60%, 60%
    fireEvent.click(layer, { clientX: 600, clientY: 360 });

    await waitFor(() => {
      expect(findingService.addMarker).toHaveBeenCalledWith(
        'fid-1',
        expect.any(String),
        0.6,
        0.6,
        'Marker 4'
      );
      expect(findingService.saveNote).toHaveBeenCalledWith(
        'fid-1',
        '1. Button misaligned\n2. Contrast issue\n3. Spacing bug\n4. Marker 4'
      );
    });
  });

  it('composition: deleting marker calls deleteMarker and renumbers remaining note lines contiguously', async () => {
    vi.mocked(findingService.deleteMarker).mockResolvedValue(undefined);
    vi.mocked(findingService.saveNote).mockResolvedValue(undefined);

    render(<FindingsView />);

    await waitFor(() => {
      expect(screen.getByTestId('delete-marker-button-2')).toBeInTheDocument();
    });

    // Delete marker 2
    const deleteBtn = screen.getByTestId('delete-marker-button-2');
    fireEvent.click(deleteBtn);

    await waitFor(() => {
      expect(findingService.deleteMarker).toHaveBeenCalledWith('fid-1', 'm2');
      // Line 2 removed and line 3 renumbered to line 2
      expect(findingService.saveNote).toHaveBeenCalledWith(
        'fid-1',
        '1. Button misaligned\n2. Spacing bug'
      );
    });
  });

  it('BUG-6 composition: missing image renders warning panel and transitions to OrphanReportView on click', async () => {
    render(<FindingsView />);

    await waitFor(() => {
      expect(screen.getByTestId('finding-image')).toBeInTheDocument();
    });

    // Trigger image load error to enter image-missing state
    const img = screen.getByTestId('finding-image');
    fireEvent.error(img);

    await waitFor(() => {
      expect(screen.getByTestId('canvas-image-missing')).toBeInTheDocument();
      expect(screen.getByText('Image file missing')).toBeInTheDocument();
      expect(screen.getByTestId('open-orphan-report-button')).toBeInTheDocument();
    });

    // Click "Open Orphan Report" button
    const openOrphanBtn = screen.getByTestId('open-orphan-report-button');
    fireEvent.click(openOrphanBtn);

    // Assert OrphanReportView is now mounted
    await waitFor(() => {
      expect(screen.getByTestId('orphan-report-view')).toBeInTheDocument();
      expect(screen.getByText('Orphan Files Report')).toBeInTheDocument();
      expect(screen.getByTestId('orphan-back-button')).toBeInTheDocument();
    });

    // Click "Back to Findings" button
    const backBtn = screen.getByTestId('orphan-back-button');
    fireEvent.click(backBtn);

    // Assert return to FindingsView
    await waitFor(() => {
      expect(screen.getByTestId('findings-editor')).toBeInTheDocument();
    });
  });
});
