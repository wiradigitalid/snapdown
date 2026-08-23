import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { FindingsEditor } from '../components/FindingsEditor';

const mockFindings = [
  {
    finding: {
      id: '018f2345-6789-7abc-8def-0123456789aa',
      image_path: 'findings/finding-1.png',
      image_width: 1920,
      image_height: 1080,
      captured_at: '2026-08-23T10:00:00Z',
      source_monitor: 'DISPLAY1',
      region: '0,0,1920,1080',
      file_size_bytes: 188416,
    },
    note: {
      id: 'note-1',
      finding_id: '018f2345-6789-7abc-8def-0123456789aa',
      body: '1. First observation\n2. Second observation\n3. Third observation',
      updated_at: '2026-08-23T10:00:00Z',
    },
    markers: [
      {
        id: 'm1',
        finding_id: '018f2345-6789-7abc-8def-0123456789aa',
        ordinal: 1,
        x: 0.25,
        y: 0.3,
        comment: 'Badge 1',
      },
      {
        id: 'm2',
        finding_id: '018f2345-6789-7abc-8def-0123456789aa',
        ordinal: 2,
        x: 0.5,
        y: 0.5,
        comment: 'Badge 2',
      },
      {
        id: 'm3',
        finding_id: '018f2345-6789-7abc-8def-0123456789aa',
        ordinal: 3,
        x: 0.75,
        y: 0.7,
        comment: 'Badge 3',
      },
    ],
    imageSrc: 'asset://localhost/findings/finding-1.png',
  },
];

describe('FindingsEditor Component (LC-006 / LC-007)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders_three_column_layout_when_populated', () => {
    const onSelectFinding = vi.fn();
    const onSaveNote = vi.fn();

    render(
      <FindingsEditor
        findings={mockFindings}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onSelectFinding={onSelectFinding}
        onSaveNote={onSaveNote}
      />
    );

    expect(screen.getByTestId('findings-editor')).toBeInTheDocument();
    expect(screen.getByTestId('capture-rail')).toBeInTheDocument();
    expect(screen.getByTestId('findings-canvas-container')).toBeInTheDocument();
    expect(screen.getByTestId('note-pane')).toBeInTheDocument();

    // Canvas checks
    const img = screen.getByTestId('finding-image');
    expect(img).toBeInTheDocument();
    expect(img).toHaveAttribute('src', 'asset://localhost/findings/finding-1.png');
    expect(screen.getByTestId('marker-layer')).toBeInTheDocument();
    expect(screen.getByTestId('canvas-readout')).toHaveTextContent('1920 × 1080 px');

    // Note pane checks
    expect(screen.getByTestId('note-textarea')).toHaveValue(
      '1. First observation\n2. Second observation\n3. Third observation'
    );
    expect(screen.getByTestId('marker-list-item-1')).toBeInTheDocument();
    expect(screen.getByTestId('marker-list-item-2')).toBeInTheDocument();
    expect(screen.getByTestId('marker-list-item-3')).toBeInTheDocument();
  });

  it('empty_state_collapses_to_centered_empty_state_with_hotkey_chip', () => {
    render(
      <FindingsEditor
        findings={[]}
        selectedFindingId={null}
        isLoading={false}
        captureHotkey="CommandOrControl+Shift+S"
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    expect(screen.getByTestId('findings-empty-state')).toBeInTheDocument();
    expect(screen.getByText('No findings yet')).toBeInTheDocument();
    expect(screen.getByText('CommandOrControl+Shift+S')).toBeInTheDocument();
  });

  it('loading_state_renders_skeleton_placeholders', () => {
    render(
      <FindingsEditor
        findings={[]}
        selectedFindingId={null}
        isLoading={true}
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    expect(screen.getAllByTestId('rail-skeleton-thumb')).toHaveLength(4);
  });

  it('error_state_renders_centered_error_with_retry', () => {
    const onRetry = vi.fn();
    render(
      <FindingsEditor
        findings={[]}
        selectedFindingId={null}
        error="Database connection failed"
        onRetry={onRetry}
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    expect(screen.getByTestId('findings-error-state')).toBeInTheDocument();
    expect(screen.getByText('The Library could not be read')).toBeInTheDocument();
    const retryBtn = screen.getByRole('button', { name: 'Retry' });
    fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('image_missing_state_renders_warning_panel_and_orphan_report_button', () => {
    const onOpenOrphanReport = vi.fn();
    const missingFinding = [
      {
        ...mockFindings[0],
        isImageMissing: true,
      },
    ];

    render(
      <FindingsEditor
        findings={missingFinding}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onOpenOrphanReport={onOpenOrphanReport}
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    expect(screen.getByTestId('canvas-image-missing')).toBeInTheDocument();
    expect(screen.getByText('Image file missing')).toBeInTheDocument();
    expect(screen.getByText('findings/finding-1.png')).toBeInTheDocument();

    const orphanBtn = screen.getByTestId('open-orphan-report-button');
    fireEvent.click(orphanBtn);
    expect(onOpenOrphanReport).toHaveBeenCalledTimes(1);
  });

  it('scn04_deleting_note_line_reports_marker_unbound_in_note_pane', async () => {
    const onSaveNote = vi.fn().mockResolvedValue(undefined);

    render(
      <FindingsEditor
        findings={mockFindings}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onSelectFinding={vi.fn()}
        onSaveNote={onSaveNote}
      />
    );

    // Initial state: all markers bound
    expect(screen.queryByTestId('marker-unbound-1')).not.toBeInTheDocument();
    expect(screen.queryByTestId('marker-unbound-2')).not.toBeInTheDocument();
    expect(screen.queryByTestId('marker-unbound-3')).not.toBeInTheDocument();

    // User deletes line 2 from the note textarea
    const textarea = screen.getByTestId('note-textarea');
    fireEvent.change(textarea, {
      target: { value: '1. First observation\n3. Third observation' },
    });

    // Marker 2 is now reported unbound in the note pane
    expect(screen.getByTestId('marker-unbound-2')).toBeInTheDocument();
    expect(screen.getByTestId('marker-unbound-2')).toHaveTextContent('Unbound / No note line');

    // All markers still remain on canvas!
    expect(screen.getByTestId('marker-badge-1')).toBeInTheDocument();
    expect(screen.getByTestId('marker-badge-2')).toBeInTheDocument();
    expect(screen.getByTestId('marker-badge-3')).toBeInTheDocument();
  });

  it('marker_list_keyboard_navigation_and_deletion', () => {
    const onDeleteMarker = vi.fn();

    render(
      <FindingsEditor
        findings={mockFindings}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onDeleteMarker={onDeleteMarker}
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    // Click delete marker 2
    const deleteBtn = screen.getByTestId('delete-marker-button-2');
    fireEvent.click(deleteBtn);
    expect(onDeleteMarker).toHaveBeenCalledWith('018f2345-6789-7abc-8def-0123456789aa', 'm2');

    // Keyboard delete on marker list item 3
    const listItem3 = screen.getByTestId('marker-list-item-3');
    fireEvent.keyDown(listItem3, { key: 'Delete' });
    expect(onDeleteMarker).toHaveBeenCalledWith('018f2345-6789-7abc-8def-0123456789aa', 'm3');
  });

  it('multi_select_compose_footer_toggles_and_bridges_to_compose', () => {
    const onCompose = vi.fn();

    render(
      <FindingsEditor
        findings={mockFindings}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onCompose={onCompose}
        onSelectFinding={vi.fn()}
        onSaveNote={vi.fn()}
      />
    );

    // Initial: no items checked, footer hidden
    expect(screen.queryByTestId('rail-footer')).not.toBeInTheDocument();

    // Check finding checkbox
    const checkboxWrapper = screen.getByTestId('finding-checkbox-wrapper-018f2345-6789-7abc-8def-0123456789aa');
    fireEvent.click(checkboxWrapper);

    // Footer visible with count and Compose button
    expect(screen.getByTestId('rail-footer')).toBeInTheDocument();
    expect(screen.getByTestId('selection-count')).toHaveTextContent('1 selected');

    const composeBtn = screen.getByTestId('compose-button');
    fireEvent.click(composeBtn);
    expect(onCompose).toHaveBeenCalledWith(['018f2345-6789-7abc-8def-0123456789aa']);
  });
});
