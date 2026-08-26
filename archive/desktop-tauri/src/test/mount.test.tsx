import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Root } from '../main';

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: vi.fn(),
}));

vi.mock('../services/settings', () => ({
  getSettings: vi.fn().mockResolvedValue({
    vault_path: 'C:/Users/test/Vault',
    quality_budget: { named: 'balanced', max_long_edge: 1600, encoder_quality: 75 },
    latest_finding_size: null,
  }),
  getHotkeys: vi.fn().mockResolvedValue({ hotkeys: [], startup_warnings: [] }),
  getStartupStatus: vi.fn().mockResolvedValue({ enabled: false }),
}));

vi.mock('../services/capture', () => ({
  captureScreenRegion: vi.fn(),
  triggerOverlay: vi.fn(),
  dismissOverlay: vi.fn(),
}));

describe('Root Mount Decision (BUG-4)', () => {
  const originalLocation = window.location;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getCurrentWebviewWindow).mockImplementation(() => {
      throw new Error('Not running inside Tauri window');
    });
  });

  afterEach(() => {
    Object.defineProperty(window, 'location', {
      writable: true,
      value: originalLocation,
    });
  });

  it('mounts CaptureOverlay when Tauri window label is overlay', async () => {
    vi.mocked(getCurrentWebviewWindow).mockReturnValue({
      label: 'overlay',
    } as unknown as WebviewWindow);

    Object.defineProperty(window, 'location', {
      writable: true,
      value: {
        ...originalLocation,
        search: '',
      },
    });

    render(<Root />);

    expect(screen.getByTestId('capture-overlay')).toBeInTheDocument();
    expect(screen.queryByTestId('editor-shell')).not.toBeInTheDocument();
    expect(screen.queryByTestId('app-shell')).not.toBeInTheDocument();
  });

  it('mounts CaptureOverlay when URL query has overlay=true', async () => {
    Object.defineProperty(window, 'location', {
      writable: true,
      value: {
        ...originalLocation,
        search: '?overlay=true',
      },
    });

    render(<Root />);

    expect(screen.getByTestId('capture-overlay')).toBeInTheDocument();
    expect(screen.queryByTestId('editor-shell')).not.toBeInTheDocument();
    expect(screen.queryByTestId('app-shell')).not.toBeInTheDocument();
  });

  it('mounts App (Editor shell) when URL query is empty and window is main', async () => {
    vi.mocked(getCurrentWebviewWindow).mockReturnValue({
      label: 'main',
    } as unknown as WebviewWindow);

    Object.defineProperty(window, 'location', {
      writable: true,
      value: {
        ...originalLocation,
        search: '',
      },
    });

    render(<Root />);

    await waitFor(() => {
      expect(screen.getByTestId('app-shell')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('capture-overlay')).not.toBeInTheDocument();
  });

  it('mounts App (Editor shell) when URL query has other parameters without overlay=true', async () => {
    Object.defineProperty(window, 'location', {
      writable: true,
      value: {
        ...originalLocation,
        search: '?view=settings&tab=general',
      },
    });

    render(<Root />);

    await waitFor(() => {
      expect(screen.getByTestId('app-shell')).toBeInTheDocument();
    });
    expect(screen.queryByTestId('capture-overlay')).not.toBeInTheDocument();
  });
});
