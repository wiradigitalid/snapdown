import React, { useState } from 'react';
import { Badge, Button, HotkeyChip } from '@snapdown/ui';
import { HotkeyAction, HotkeySettingsDto } from '../types/settings';

export interface HotkeySectionProps {
  hotkeySettings: HotkeySettingsDto;
  onSaveHotkey: (action: HotkeyAction, shortcut: string) => Promise<void>;
  onClearHotkey: (action: HotkeyAction) => Promise<void>;
  disabled?: boolean;
}

const ACTION_LABELS: Record<HotkeyAction, string> = {
  capture: 'Capture Region',
  open_editor: 'Open Workspace / Editor',
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

  const handleRecord = (action: HotkeyAction, combo: string) => {
    setLocalInputs((prev) => ({ ...prev, [action]: combo }));
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
      aria-label="Hotkeys"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-3)',
        padding: 'var(--space-4)',
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--color-border)',
        backgroundColor: 'var(--color-surface)',
      }}
    >
      <div>
        <h2
          style={{
            margin: 0,
            fontSize: 'var(--text-base)',
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
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Click a shortcut box and press your preferred key combination.
        </p>
      </div>

      {hotkeySettings.startup_warnings && hotkeySettings.startup_warnings.length > 0 && (
        <div
          data-testid="startup-warning-banner"
          style={{
            padding: 'var(--space-2) var(--space-3)',
            borderRadius: 'var(--radius-sm)',
            backgroundColor: 'var(--color-warning-bg)',
            border: '1px solid var(--color-border)',
            color: 'var(--color-warning-text)',
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-1)',
          }}
        >
          <div style={{ fontWeight: 600 }}>Hotkey Registration Warning:</div>
          {hotkeySettings.startup_warnings.map((warning, index) => (
            <div key={index}>{warning}</div>
          ))}
        </div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-3)' }}>
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
                backgroundColor: 'var(--color-bg)',
                border: '1px solid var(--color-border)',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <span
                  style={{
                    fontSize: 'var(--text-sm)',
                    fontWeight: 600,
                    fontFamily: 'var(--font-ui)',
                    color: 'var(--color-text)',
                  }}
                >
                  {label}
                </span>
                <span data-testid={`status-badge-${action}`}>
                  <Badge variant={item.is_active ? 'success' : 'neutral'}>
                    {item.is_active ? 'Active' : 'Disabled'}
                  </Badge>
                </span>
              </div>

              <div
                style={{
                  display: 'flex',
                  gap: 'var(--space-2)',
                  alignItems: 'center',
                }}
              >
                <div style={{ flex: 1 }}>
                  <HotkeyChip
                    shortcut={currentVal}
                    onRecord={(combo) => handleRecord(action, combo)}
                    disabled={isBusy}
                    aria-label={`Record shortcut for ${label}`}
                    style={{ width: '100%' }}
                  />
                </div>

                <Button
                  variant="primary"
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

              {error && (
                <span
                  style={{
                    fontSize: 'var(--text-xs)',
                    color: 'var(--color-danger)',
                    fontFamily: 'var(--font-ui)',
                  }}
                >
                  {error}
                </span>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
};
