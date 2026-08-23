import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
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
    },
    note: {
      id: 'note-1',
      finding_id: '018f2345-6789-7abc-8def-0123456789aa',
      body: 'Initial note description',
      updated_at: '2026-08-23T10:00:00Z',
    },
    markers: [
      {
        id: 'm1',
        finding_id: '018f2345-6789-7abc-8def-0123456789aa',
        ordinal: 1,
        x: 0.5,
        y: 0.5,
        comment: 'Badge 1',
      },
    ],
  },
];

describe('FindingsEditor Component (Screen 3 & 4)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('findings_editor_renders_list_and_details', () => {
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
    expect(screen.getByTestId('finding-item-018f2345-6789-7abc-8def-0123456789aa')).toBeInTheDocument();
    expect(screen.getByText('Finding Details')).toBeInTheDocument();
    expect(screen.getByText('1920 × 1080 px')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Initial note description')).toBeInTheDocument();
  });

  it('note_inline_editor_persists_markdown', async () => {
    const onSelectFinding = vi.fn();
    const onSaveNote = vi.fn().mockResolvedValue(undefined);

    render(
      <FindingsEditor
        findings={mockFindings}
        selectedFindingId="018f2345-6789-7abc-8def-0123456789aa"
        onSelectFinding={onSelectFinding}
        onSaveNote={onSaveNote}
      />
    );

    const textarea = screen.getByLabelText('Note');
    fireEvent.change(textarea, { target: { value: 'Updated markdown notes with **bold**' } });

    const saveBtn = screen.getByRole('button', { name: 'Save Note' });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(onSaveNote).toHaveBeenCalledWith(
        '018f2345-6789-7abc-8def-0123456789aa',
        'Updated markdown notes with **bold**'
      );
      expect(screen.getByTestId('save-success-indicator')).toBeInTheDocument();
    });
  });
});
