import React, { useState } from 'react';
import { Button, TextField } from '@snapdown/ui';
import { HotkeyAction, HotkeySettingsDto } from '../types/settings';

interface HotkeySectionProps {
  hotkeySettings: HotkeySettingsDto;
  onSaveHotkey: (action: HotkeyAction, shortcut: string) => Promise<void>;
  onClearHotkey: (action: HotkeyAction) => Promise<void>;
  disabled?: boolean;
}

const ACTION_LABELS: Record<HotkeyAction, string> = {
  capture: 'Capture Region',
  open_editor: 'Open Editor',
};

export const HotkeySection: React.FC<HotkeySectionProps> = ({
  hotkeySettings,
  onSaveHotkey,
  onClearHotkey,
  disabled = false,
}) => {
  const [localInputs, setLocalInputs] = useState<Record<string, string>>({});
  const [errorMessages, setErrorMessages] = useState<Record<string, string>>({});
  const [savingAction, setSavingAction] = useState<string | null>(null);

  const getInputValue = (action: HotkeyAction, defaultVal: string) => {
    return localInputs[action] !== undefined ? localInputs[action] : defaultVal;
  };

  const handleInputChange = (action: HotkeyAction, value: string) => {
    setLocalInputs((prev) => ({ ...prev, [action]: value }));
    setErrorMessages((prev) => ({ ...prev, [action]: '' }));
  };

  const handleSave = async (action: HotkeyAction) => {
    const item = hotkeySettings.hotkeys.find((h) => h.action === action);
    const shortcutToSave = getInputValue(action, item?.shortcut || '').trim();

    if (!shortcutToSave) {
      setErrorMessages((prev) => ({
        ...prev,
        [action]: 'Shortcut cannot be empty. Use "Clear" to disable this action.',
      }));
      return;
    }

    try {
      setSavingAction(action);
      await onSaveHotkey(action, shortcutToSave);
      setErrorMessages((prev) => ({ ...prev, [action]: '' }));
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessages((prev) => ({ ...prev, [action]: msg }));
    } finally {
      setSavingAction(null);
    }
  };

  const handleClear = async (action: HotkeyAction) => {
    try {
      setSavingAction(action);
      await onClearHotkey(action);
      setLocalInputs((prev) => ({ ...prev, [action]: '' }));
      setErrorMessages((prev) => ({ ...prev, [action]: '' }));
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      setErrorMessages((prev) => ({ ...prev, [action]: msg }));
    } finally {
      setSavingAction(null);
    }
  };

  return (
    <section
      data-testid="hotkey-section"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
        padding: 'var(--space-4)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
        backgroundColor: 'var(--color-bg-card)',
      }}
    >
      <div>
        <h2
          style={{
            margin: 0,
            fontSize: 'var(--text-lg)',
            fontWeight: 600,
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text)',
          }}
        >
          Hotkeys
        </h2>
        <p
          style={{
            margin: 'var(--space-1) 0 0 0',
            fontSize: 'var(--text-sm)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Configure global shortcut keys for capturing screenshots and opening the editor.
        </p>
      </div>

      {hotkeySettings.startup_warnings && hotkeySettings.startup_warnings.length > 0 && (
        <div
          data-testid="startup-warning-banner"
          style={{
            padding: 'var(--space-3)',
            borderRadius: 'var(--radius-md)',
            backgroundColor: 'var(--color-warning-subtle, rgba(234, 179, 8, 0.1))',
            border: '1px solid var(--color-warning-border, rgba(234, 179, 8, 0.3))',
            color: 'var(--color-warning-text, #ca8a04)',
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-1)',
          }}
        >
          <div style={{ fontWeight: 600 }}>Hotkey Registration Warning (BR-26):</div>
          {hotkeySettings.startup_warnings.map((warning, index) => (
            <div key={index}>{warning}</div>
          ))}
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
        {hotkeySettings.hotkeys.map((item) => {
          const action = item.action as HotkeyAction;
          const label = ACTION_LABELS[action] || action;
          const currentVal = getInputValue(action, item.shortcut);
          const isDirty = currentVal !== item.shortcut;
          const isBusy = savingAction === action || disabled;
          const error = errorMessages[action];

          return (
            <div
              key={action}
              data-testid={`hotkey-row-${action}`}
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: 'var(--space-2)',
                padding: 'var(--space-3)',
                borderRadius: 'var(--radius-md)',
                backgroundColor: 'var(--color-bg-subtle, #f9fafb)',
                border: '1px solid var(--color-border-subtle, #e5e7eb)',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <label
                  htmlFor={`hotkey-input-${action}`}
                  style={{
                    fontSize: 'var(--text-sm)',
                    fontWeight: 600,
                    fontFamily: 'var(--font-ui)',
                    color: 'var(--color-text)',
                  }}
                >
                  {label}
                </label>
                <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
                  <span
                    data-testid={`status-badge-${action}`}
                    style={{
                      fontSize: 'var(--text-xs)',
                      fontFamily: 'var(--font-mono)',
                      padding: '2px 8px',
                      borderRadius: 'var(--radius-sm)',
                      backgroundColor: item.is_active
                        ? 'var(--color-success-subtle, #dcfce7)'
                        : 'var(--color-muted-subtle, #f3f4f6)',
                      color: item.is_active
                        ? 'var(--color-success-text, #166534)'
                        : 'var(--color-text-muted, #6b7280)',
                    }}
                  >
                    {item.is_active ? 'Active' : 'Disabled / Inactive'}
                  </span>
                </div>
              </div>

              <div
                style={{
                  display: 'flex',
                  gap: 'var(--space-2)',
                  alignItems: 'flex-start',
                }}
              >
                <div style={{ flex: 1 }}>
                  <TextField
                    id={`hotkey-input-${action}`}
                    name={`hotkey-${action}`}
                    value={currentVal}
                    placeholder="e.g. CommandOrControl+Shift+S"
                    onChange={(e) => handleInputChange(action, e.target.value)}
                    disabled={isBusy}
                    invalid={Boolean(error)}
                    errorMessage={error}
                  />
                </div>
                <Button
                  variant="secondary"
                  onClick={() => handleSave(action)}
                  disabled={isBusy || !isDirty}
                  aria-label={`Save ${label}`}
                >
                  Save
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => handleClear(action)}
                  disabled={isBusy || (!item.shortcut && !currentVal)}
                  aria-label={`Clear ${label}`}
                >
                  Clear
                </Button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
};
