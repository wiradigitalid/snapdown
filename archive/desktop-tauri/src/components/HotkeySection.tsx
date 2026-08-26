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

  const handleCancel = (action: HotkeyAction) => {
    setLocalInputs((prev) => {
      const next = { ...prev };
      delete next[action];
      return next;
    });
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
      setLocalInputs((prev) => {
        const next = { ...prev };
        delete next[action];
        return next;
      });
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

      <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-3)' }}>
        {hotkeySettings.hotkeys.map((item) => {
          const action = item.action as HotkeyAction;
          const label = ACTION_LABELS[action] || action;
          const currentVal = getInputValue(action, item.shortcut);
          const isDirty = currentVal !== item.shortcut;
          const isBusy = savingAction === action || disabled;
          const error = errorMessages[action];

          const startupWarning =
            item.startup_error ||
            hotkeySettings.startup_warnings?.find(
              (w) =>
                w.toLowerCase().includes(`'${action}'`) ||
                w.toLowerCase().includes(`action '${action}'`) ||
                w.toLowerCase().includes(action)
            );
          const hasStartupFailure = (!item.is_registered && !!item.shortcut) || !!startupWarning;

          const chipState = error
            ? 'conflicted'
            : hasStartupFailure && !isDirty
            ? 'conflicted'
            : !currentVal
            ? 'unbound'
            : 'bound';

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
                <div
                  data-testid={`status-badge-${action}`}
                  style={{ display: 'flex', gap: 'var(--space-1)', alignItems: 'center' }}
                >
                  {hasStartupFailure && (
                    <Badge variant="warning" data-testid={`startup-warning-badge-${action}`}>
                      Conflict
                    </Badge>
                  )}
                  <Badge variant={item.is_active ? 'success' : 'neutral'}>
                    {item.is_active ? 'Active' : 'Disabled'}
                  </Badge>
                </div>
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
                    state={chipState}
                    onRecord={(combo) => handleRecord(action, combo)}
                    onCancel={() => handleCancel(action)}
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

              {hasStartupFailure && !isDirty && (
                <span
                  data-testid={`startup-error-${action}`}
                  style={{
                    fontSize: 'var(--text-xs)',
                    color: 'var(--color-warning-text)',
                    fontFamily: 'var(--font-ui)',
                  }}
                >
                  {startupWarning ||
                    `Failed to register shortcut for action '${action}' at startup: combination is already held by Windows or another application`}
                </span>
              )}

              {error && (
                <span
                  data-testid={`error-message-${action}`}
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