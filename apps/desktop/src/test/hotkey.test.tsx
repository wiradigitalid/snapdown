import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { HotkeySection } from '../components/HotkeySection';
import { HotkeySettingsDto } from '../types/settings';

describe('HotkeySection and Hotkey rows (W6-S6 / UC-15 / BR-26 / BR-27 / BR-113 / BR-114)', () => {
  const mockDefaultHotkeys: HotkeySettingsDto = {
    hotkeys: [
      {
        action: 'capture',
        shortcut: 'CommandOrControl+Shift+S',
        is_registered: true,
        is_active: true,
      },
      {
        action: 'open_editor',
        shortcut: 'CommandOrControl+Shift+E',
        is_registered: true,
        is_active: true,
      },
    ],
    startup_warnings: [],
  };

  const handleSaveHotkey = vi.fn();
  const handleClearHotkey = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('a_listening_chip_stops_listening_when_focus_leaves_it', async () => {
    // SCN-03 / UC-15 / EXPERIENCE.md: A chip left listening when focus leaves it (blur)
    // ceases listening immediately and restores prior state, preventing keystroke leakage.
    render(
      <HotkeySection
        hotkeySettings={mockDefaultHotkeys}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    const captureChip = screen.getByRole('button', { name: 'Record shortcut for Capture Region' });
    expect(captureChip).toHaveAttribute('data-state', 'bound');
    expect(captureChip).toHaveTextContent('CommandOrControl+Shift+S');

    // Click to start listening
    fireEvent.click(captureChip);
    expect(captureChip).toHaveAttribute('data-state', 'listening');
    expect(captureChip).toHaveTextContent('Press keys… Esc to cancel');

    // Focus leaves chip (blur event)
    fireEvent.blur(captureChip);

    // Ceases listening and restores prior bound shortcut
    expect(captureChip).toHaveAttribute('data-state', 'bound');
    expect(captureChip).toHaveTextContent('CommandOrControl+Shift+S');
  });

  it('a_snapdown_internal_conflict_is_worded_differently_from_an_os_conflict', async () => {
    // BR-27 vs BR-26: Internal conflict (another Snapdown action) and OS conflict
    // have completely different remedies and MUST be worded distinctly.
    const internalError = new Error('Another Snapdown action already uses this combination');
    const osError = new Error('This combination is already held by Windows or another application');

    handleSaveHotkey
      .mockRejectedValueOnce(internalError)
      .mockRejectedValueOnce(osError);

    const { rerender } = render(
      <HotkeySection
        hotkeySettings={mockDefaultHotkeys}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    // 1. Attempt internal conflict on open_editor
    const editorChip = screen.getByRole('button', { name: 'Record shortcut for Open Workspace / Editor' });
    fireEvent.click(editorChip);
    fireEvent.keyDown(editorChip, { key: 's', ctrlKey: true, shiftKey: true });

    const saveEditorBtn = screen.getByRole('button', { name: 'Save Open Workspace / Editor' });
    fireEvent.click(saveEditorBtn);

    await waitFor(() => {
      expect(
        screen.getByText('Another Snapdown action already uses this combination')
      ).toBeInTheDocument();
    });

    const internalMsg = screen.getByText('Another Snapdown action already uses this combination').textContent;

    // 2. Attempt OS conflict on capture
    rerender(
      <HotkeySection
        hotkeySettings={mockDefaultHotkeys}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    const captureChip = screen.getByRole('button', { name: 'Record shortcut for Capture Region' });
    fireEvent.click(captureChip);
    fireEvent.keyDown(captureChip, { key: 'z', ctrlKey: true, altKey: true });

    const saveCaptureBtn = screen.getByRole('button', { name: 'Save Capture Region' });
    fireEvent.click(saveCaptureBtn);

    await waitFor(() => {
      expect(
        screen.getByText('This combination is already held by Windows or another application')
      ).toBeInTheDocument();
    });

    const osMsg = screen.getByText('This combination is already held by Windows or another application').textContent;

    expect(internalMsg).not.toEqual(osMsg);
  });

  it('a_cleared_hotkey_reads_disabled_rather_than_empty', () => {
    // BR-113: A cleared hotkey displays Disabled badge and Click to set unbound chip,
    // clearly distinguishing deliberate disabling from an uninitialized or broken input.
    const clearedSettings: HotkeySettingsDto = {
      hotkeys: [
        {
          action: 'capture',
          shortcut: '',
          is_registered: false,
          is_active: false,
        },
        {
          action: 'open_editor',
          shortcut: 'CommandOrControl+Shift+E',
          is_registered: true,
          is_active: true,
        },
      ],
      startup_warnings: [],
    };

    render(
      <HotkeySection
        hotkeySettings={clearedSettings}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    const captureRow = screen.getByTestId('hotkey-row-capture');
    expect(captureRow).toBeInTheDocument();

    // Badge reads 'Disabled' in words
    const badge = screen.getByTestId('status-badge-capture');
    expect(badge).toHaveTextContent('Disabled');

    // Chip displays 'Click to set' with unbound state, not empty or broken
    const chip = screen.getByRole('button', { name: 'Record shortcut for Capture Region' });
    expect(chip).toHaveAttribute('data-state', 'unbound');
    expect(chip).toHaveTextContent('Click to set');
  });

  it('a_startup_registration_failure_carries_a_badge_before_the_reviewer_acts', () => {
    // DESIGN.md & BR-26 & BR-114: If startup registration fails because another app grabbed
    // the shortcut, the row carries a warning badge and explanatory message under the control
    // BEFORE the Reviewer interacts with it, preserving the stored setting.
    const startupFailureSettings: HotkeySettingsDto = {
      hotkeys: [
        {
          action: 'capture',
          shortcut: 'CommandOrControl+Shift+S',
          is_registered: false,
          is_active: false,
          startup_error: "Failed to register shortcut for action 'capture' at startup: combination is already held by Windows or another application",
        },
        {
          action: 'open_editor',
          shortcut: 'CommandOrControl+Shift+E',
          is_registered: true,
          is_active: true,
        },
      ],
      startup_warnings: [
        "Failed to register shortcut for action 'capture' at startup: combination is already held by Windows or another application",
      ],
    };

    render(
      <HotkeySection
        hotkeySettings={startupFailureSettings}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    // Warning badge is present on the row before any interaction
    const warningBadge = screen.getByTestId('startup-warning-badge-capture');
    expect(warningBadge).toBeInTheDocument();
    expect(warningBadge).toHaveTextContent('Conflict');

    // Explanatory error message is rendered under the control
    const errorMsg = screen.getByTestId('startup-error-capture');
    expect(errorMsg).toBeInTheDocument();
    expect(errorMsg).toHaveTextContent(/Failed to register shortcut for action 'capture' at startup/);

    // Chip is in conflicted state displaying the stored combination
    const chip = screen.getByRole('button', { name: 'Record shortcut for Capture Region' });
    expect(chip).toHaveAttribute('data-state', 'conflicted');
    expect(chip).toHaveTextContent('CommandOrControl+Shift+S');
  });

  it('the_active_and_disabled_badges_carry_a_word_not_only_a_colour', () => {
    // EXPERIENCE.md & NFR-16: Accessibility floor requires status badges to carry
    // explicit text words (Active / Disabled) and semantic token classes (badge-success / badge-neutral).
    const mixedSettings: HotkeySettingsDto = {
      hotkeys: [
        {
          action: 'capture',
          shortcut: 'CommandOrControl+Shift+S',
          is_registered: true,
          is_active: true,
        },
        {
          action: 'open_editor',
          shortcut: '',
          is_registered: false,
          is_active: false,
        },
      ],
      startup_warnings: [],
    };

    render(
      <HotkeySection
        hotkeySettings={mixedSettings}
        onSaveHotkey={handleSaveHotkey}
        onClearHotkey={handleClearHotkey}
      />
    );

    const activeBadgeContainer = screen.getByTestId('status-badge-capture');
    const activeBadge = activeBadgeContainer.querySelector('.badge-success');
    expect(activeBadge).toBeInTheDocument();
    expect(activeBadge?.textContent?.trim()).toBe('Active');

    const disabledBadgeContainer = screen.getByTestId('status-badge-open_editor');
    const disabledBadge = disabledBadgeContainer.querySelector('.badge-neutral');
    expect(disabledBadge).toBeInTheDocument();
    expect(disabledBadge?.textContent?.trim()).toBe('Disabled');
  });
});