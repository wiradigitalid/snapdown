import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { App } from '../App';
import { GeneralSection } from '../components/GeneralSection';
import * as settingsService from '../services/settings';

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  setVaultPath: vi.fn(),
  setQualityBudget: vi.fn(),
  getLatestFindingSize: vi.fn(),
  openVaultFolder: vi.fn(),
  getHotkeys: vi.fn(),
  setHotkey: vi.fn(),
  clearHotkey: vi.fn(),
  getStartupStatus: vi.fn(),
  setStartupStatus: vi.fn(),
  pickVaultFolder: vi.fn(),
}));

const mockDefaultHotkeys = {
  hotkeys: [
    {
      action: 'capture' as const,
      shortcut: 'CommandOrControl+Shift+S',
      is_registered: true,
      is_active: true,
    },
    {
      action: 'open_editor' as const,
      shortcut: 'CommandOrControl+Shift+E',
      is_registered: true,
      is_active: true,
    },
  ],
  startup_warnings: [],
};

const mockSettings = {
  vault_path: 'C:/Users/test/Vault',
  quality_budget: {
    named: 'auto' as const,
    max_long_edge: 1600,
    encoder_quality: 82,
  },
  latest_finding_size: null,
};

describe('Startup Registration UI & 3-State Control (W6-S5 / SCN-02 / BR-108 / BR-112)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(settingsService.getSettings).mockResolvedValue(mockSettings);
    vi.mocked(settingsService.getHotkeys).mockResolvedValue(mockDefaultHotkeys);
  });

  it('the_startup_toggle_renders_unknown_until_the_os_has_answered', async () => {
    // SCN-02 & BR-108: While backend read is pending, toggle renders indeterminate / Unknown
    let resolveStartupStatus!: (val: { enabled: boolean; state: 'on' | 'off' | 'unreadable' }) => void;
    const pendingPromise = new Promise<{ enabled: boolean; state: 'on' | 'off' | 'unreadable' }>((resolve) => {
      resolveStartupStatus = resolve;
    });
    vi.mocked(settingsService.getStartupStatus).mockReturnValue(pendingPromise);

    render(<App initialTab="settings" />);

    // Immediately on mount, before OS resolves:
    const toggle = screen.getByTestId('startup-toggle');
    expect(toggle).toBeInTheDocument();
    expect(toggle).toHaveAttribute('data-state', 'indeterminate');
    expect(toggle).toHaveAttribute('aria-checked', 'mixed');
    expect(toggle).toBeDisabled();

    // Now resolve the backend call
    resolveStartupStatus({ enabled: true, state: 'on' });

    await waitFor(() => {
      expect(toggle).toHaveAttribute('data-state', 'on');
      expect(toggle).toHaveAttribute('aria-checked', 'true');
      expect(toggle).not.toBeDisabled();
    });
  });

  it('the_startup_toggle_never_renders_a_definite_state_before_the_read_resolves', async () => {
    // BR-108 & state-machines.md § 1: Must never flash On or Off before read resolves
    let resolveStartupStatus!: (val: { enabled: boolean; state: 'off' }) => void;
    const pendingPromise = new Promise<{ enabled: boolean; state: 'off' }>((resolve) => {
      resolveStartupStatus = resolve;
    });
    vi.mocked(settingsService.getStartupStatus).mockReturnValue(pendingPromise);

    render(<App initialTab="settings" />);

    const toggle = screen.getByTestId('startup-toggle');
    // Must NOT be 'on' or 'off'
    expect(toggle.getAttribute('data-state')).toBe('indeterminate');
    expect(toggle.getAttribute('aria-checked')).toBe('mixed');
    expect(toggle).not.toHaveAttribute('data-state', 'on');
    expect(toggle).not.toHaveAttribute('data-state', 'off');

    resolveStartupStatus({ enabled: false, state: 'off' });

    await waitFor(() => {
      expect(toggle).toHaveAttribute('data-state', 'off');
      expect(toggle).toHaveAttribute('aria-checked', 'false');
    });
  });

  it('startup_toggle_shows_on_when_registered', async () => {
    vi.mocked(settingsService.getStartupStatus).mockResolvedValue({
      enabled: true,
      state: 'on',
    });

    render(<App initialTab="settings" />);

    await waitFor(() => {
      const toggle = screen.getByTestId('startup-toggle');
      expect(toggle).toHaveAttribute('data-state', 'on');
      expect(toggle).toHaveAttribute('aria-checked', 'true');
    });
  });

  it('startup_toggle_shows_off_when_not_registered', async () => {
    vi.mocked(settingsService.getStartupStatus).mockResolvedValue({
      enabled: false,
      state: 'off',
    });

    render(<App initialTab="settings" />);

    await waitFor(() => {
      const toggle = screen.getByTestId('startup-toggle');
      expect(toggle).toHaveAttribute('data-state', 'off');
      expect(toggle).toHaveAttribute('aria-checked', 'false');
    });
  });

  it('startup_toggle_shows_unreadable_with_retry_on_read_failure', async () => {
    // state-machines.md § 1: If read fails, shows Unreadable error state with Retry action
    vi.mocked(settingsService.getStartupStatus)
      .mockRejectedValueOnce(new Error('Windows Registry locked'))
      .mockResolvedValueOnce({ enabled: true, state: 'on' });

    render(<App initialTab="settings" />);

    await waitFor(() => {
      expect(screen.getByTestId('startup-unreadable-message')).toBeInTheDocument();
      expect(screen.getByText('Could not read Windows startup status')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Retry' })).toBeInTheDocument();
    });

    // Click Retry
    const retryBtn = screen.getByRole('button', { name: 'Retry' });
    fireEvent.click(retryBtn);

    await waitFor(() => {
      expect(settingsService.getStartupStatus).toHaveBeenCalledTimes(2);
      const toggle = screen.getByTestId('startup-toggle');
      expect(toggle).toHaveAttribute('data-state', 'on');
      expect(toggle).toHaveAttribute('aria-checked', 'true');
    });
  });

  it('isolated_general_section_unit_renders_all_four_states', async () => {
    const handleToggle = vi.fn();
    const handleRetry = vi.fn();

    // 1. Unknown state
    const { rerender } = render(
      <GeneralSection
        startupStatus="unknown"
        onToggleStartup={handleToggle}
        onRetryStartup={handleRetry}
      />
    );
    let toggle = screen.getByTestId('startup-toggle');
    expect(toggle).toHaveAttribute('data-state', 'indeterminate');
    expect(toggle).toHaveAttribute('aria-checked', 'mixed');
    expect(toggle).toBeDisabled();

    // 2. On state
    rerender(
      <GeneralSection
        startupStatus="on"
        onToggleStartup={handleToggle}
        onRetryStartup={handleRetry}
      />
    );
    toggle = screen.getByTestId('startup-toggle');
    expect(toggle).toHaveAttribute('data-state', 'on');
    expect(toggle).toHaveAttribute('aria-checked', 'true');
    expect(toggle).not.toBeDisabled();

    // 3. Off state
    rerender(
      <GeneralSection
        startupStatus="off"
        onToggleStartup={handleToggle}
        onRetryStartup={handleRetry}
      />
    );
    toggle = screen.getByTestId('startup-toggle');
    expect(toggle).toHaveAttribute('data-state', 'off');
    expect(toggle).toHaveAttribute('aria-checked', 'false');
    expect(toggle).not.toBeDisabled();

    // 4. Unreadable state
    rerender(
      <GeneralSection
        startupStatus="unreadable"
        onToggleStartup={handleToggle}
        onRetryStartup={handleRetry}
      />
    );
    expect(screen.getByTestId('startup-unreadable-message')).toBeInTheDocument();
    expect(screen.queryByTestId('startup-toggle')).not.toBeInTheDocument();

    const retryBtn = screen.getByRole('button', { name: 'Retry' });
    fireEvent.click(retryBtn);
    expect(handleRetry).toHaveBeenCalledTimes(1);
  });
});
