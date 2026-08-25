import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { CaptureOverlay } from '../components/CaptureOverlay';
import * as captureService from '../services/capture';

vi.mock('../services/capture', () => ({
  captureScreenRegion: vi.fn(),
  triggerOverlay: vi.fn(),
  dismissOverlay: vi.fn(),
}));

describe('CaptureOverlay Component (Screen 1 & 2)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('the_overlay_asks_for_a_note_before_it_writes_a_finding', async () => {
    render(<CaptureOverlay />);

    const overlay = screen.getByTestId('capture-overlay');
    expect(overlay).toBeInTheDocument();

    // Mouse drag from (50, 50) to (250, 200)
    fireEvent.mouseDown(overlay, { clientX: 50, clientY: 50, button: 0 });
    fireEvent.mouseMove(overlay, { clientX: 250, clientY: 200 });

    const selectionBox = screen.getByTestId('selection-box');
    expect(selectionBox).toBeInTheDocument();
    expect(selectionBox).toHaveStyle({
      left: '50px',
      top: '50px',
      width: '200px',
      height: '150px',
    });

    const readout = screen.getByTestId('dimensions-readout');
    expect(readout).toHaveTextContent('200 × 150 px (4:3)');

    // Mouse up commits selection and enters Narrating
    fireEvent.mouseUp(overlay);

    expect(captureService.captureScreenRegion).not.toHaveBeenCalled();

    const noteField = screen.getByTestId('capture-note-field');
    expect(noteField).toBeInTheDocument();
    expect(noteField).toHaveFocus();

    const hint = screen.getByTestId('capture-note-hint');
    expect(hint).toHaveTextContent('Enter to save · Esc to cancel');
  });

  it('enter_saves_the_note_with_the_finding', async () => {
    const onCaptureComplete = vi.fn();
    vi.mocked(captureService.captureScreenRegion).mockResolvedValue({
      image_path: 'findings/capture_test.png',
      image_width: 200,
      image_height: 150,
      source_monitor: 'DISPLAY1',
      region: '50,50,200,150',
    });

    render(<CaptureOverlay onCaptureComplete={onCaptureComplete} />);

    const overlay = screen.getByTestId('capture-overlay');
    fireEvent.mouseDown(overlay, { clientX: 50, clientY: 50, button: 0 });
    fireEvent.mouseMove(overlay, { clientX: 250, clientY: 200 });
    fireEvent.mouseUp(overlay);

    const noteField = screen.getByTestId('capture-note-field');
    fireEvent.change(noteField, { target: { value: 'the CTA is unreadable' } });

    // Shift+Enter inserts newline without saving
    fireEvent.keyDown(noteField, { key: 'Enter', shiftKey: true });
    expect(captureService.captureScreenRegion).not.toHaveBeenCalled();

    // Enter saves
    fireEvent.keyDown(noteField, { key: 'Enter', shiftKey: false });

    await waitFor(() => {
      expect(captureService.captureScreenRegion).toHaveBeenCalledTimes(1);
      expect(captureService.captureScreenRegion).toHaveBeenCalledWith({
        x: 50,
        y: 50,
        width: 200,
        height: 150,
        note: 'the CTA is unreadable',
      });
      expect(onCaptureComplete).toHaveBeenCalledWith({
        image_path: 'findings/capture_test.png',
        image_width: 200,
        image_height: 150,
        source_monitor: 'DISPLAY1',
        region: '50,50,200,150',
      });
    });
  });

  it('esc_cancels_the_capture_and_writes_no_finding', async () => {
    const onDismiss = vi.fn();
    vi.mocked(captureService.dismissOverlay).mockResolvedValue();

    render(<CaptureOverlay onDismiss={onDismiss} />);

    const overlay = screen.getByTestId('capture-overlay');
    fireEvent.mouseDown(overlay, { clientX: 50, clientY: 50, button: 0 });
    fireEvent.mouseMove(overlay, { clientX: 250, clientY: 200 });
    fireEvent.mouseUp(overlay);

    const noteField = screen.getByTestId('capture-note-field');
    fireEvent.change(noteField, { target: { value: 'some note' } });

    fireEvent.keyDown(noteField, { key: 'Escape' });

    await waitFor(() => {
      expect(captureService.dismissOverlay).toHaveBeenCalledTimes(1);
      expect(onDismiss).toHaveBeenCalledTimes(1);
    });

    expect(captureService.captureScreenRegion).not.toHaveBeenCalled();
  });

  it('refuses selection smaller than 8x8 pixels (BR-31)', async () => {
    render(<CaptureOverlay />);

    const overlay = screen.getByTestId('capture-overlay');

    // Tiny drag (4x4)
    fireEvent.mouseDown(overlay, { clientX: 10, clientY: 10, button: 0 });
    fireEvent.mouseMove(overlay, { clientX: 14, clientY: 14 });
    fireEvent.mouseUp(overlay);

    expect(screen.getByText('Region must be at least 8x8 pixels')).toBeInTheDocument();
    expect(captureService.captureScreenRegion).not.toHaveBeenCalled();
  });

  it('renders precision crosshairs, loupe magnifier, and fullscreen button in armed phase (SPEC-w9)', () => {
    render(<CaptureOverlay />);

    expect(screen.getByTestId('overlay-fullscreen-btn')).toBeInTheDocument();
    expect(screen.getByTestId('crosshair-axis-vertical')).toBeInTheDocument();
    expect(screen.getByTestId('crosshair-axis-horizontal')).toBeInTheDocument();
    expect(screen.getByTestId('capture-loupe-magnifier')).toBeInTheDocument();
  });

  it('clicking fullscreen button selects viewport and enters narrating phase (SPEC-w9)', () => {
    render(<CaptureOverlay />);

    const fullscreenBtn = screen.getByTestId('overlay-fullscreen-btn');
    fireEvent.click(fullscreenBtn);

    const noteField = screen.getByTestId('capture-note-field');
    expect(noteField).toBeInTheDocument();
    expect(screen.queryByTestId('overlay-fullscreen-btn')).not.toBeInTheDocument();
  });

  it('dismisses overlay on Escape key press (BR-32)', async () => {
    const onDismiss = vi.fn();
    vi.mocked(captureService.dismissOverlay).mockResolvedValue();

    render(<CaptureOverlay onDismiss={onDismiss} />);

    fireEvent.keyDown(window, { key: 'Escape' });

    await waitFor(() => {
      expect(captureService.dismissOverlay).toHaveBeenCalledTimes(1);
      expect(onDismiss).toHaveBeenCalledTimes(1);
    });
  });
});