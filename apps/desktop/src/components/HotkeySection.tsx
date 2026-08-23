import React, { useState } from 'react';
import { Button } from '@snapdown/ui';
import { HotkeyAction, HotkeySettingsDto } from '../types/settings';

interface HotkeySectionProps {
  hotkeySettings: HotkeySettingsDto;
  onSaveHotkey: (action: HotkeyAction, shortcut: string) => Promise<void>;
  onClearHotkey: (action: HotkeyAction) => Promise<void>;
  disabled?: boolean;
}

const ACTION_LABELS: Record<HotkeyAction, string> = {
  capture: 'Capture Region',
  open_editor: 'Open Workspace / Editor',
};

const formatKeyCombination = (e: React.KeyboardEvent): string | null => {
  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    return null; // Ignore pure modifier press
  }

  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) {
    parts.push('CommandOrControl');
  }
  if (e.altKey) {
    parts.push('Alt');
  }
  if (e.shiftKey) {
    parts.push('Shift');
  }

  let key = e.key.toUpperCase();
  if (key === ' ') key = 'SPACE';
  if (key === 'ESCAPE') return null;

  parts.push(key);
  return parts.join('+');
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
  const [recordingAction, setRecordingAction] = useState<string | null>(null);

  const getInputValue = (action: HotkeyAction, defaultVal: string) => {
    return localInputs[action] !== undefined ? localInputs[action] : defaultVal;
  };

  const handleKeyDown = (action: HotkeyAction, e: React.KeyboardEvent) => {
    if (recordingAction !== action) return;

    if (e.key === 'Escape') {
      setRecordingAction(null);
      return;
    }

    e.preventDefault();
    e.stopPropagation();

    const combination = formatKeyCombination(e);
    if (combination) {
      setLocalInputs((prev) => ({ ...prev, [action]: combination }));
      setErrorMessages((prev) => ({ ...prev, [action]: '' }));
      setRecordingAction(null);
    }
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
          Click a shortcut box and press your preferred key combination on the keyboard.
        </p>
      </div>

      {hotkeySettings.startup_warnings && hotkeySettings.startup_warnings.length > 0 && (
        <div
          data-testid="startup-warning-banner"
          style={{
            padding: 'var(--space-2) var(--space-3)',
            borderRadius: 'var(--radius-sm)',
            backgroundColor: '#fef3c7',
            border: '1px solid #fde047',
            color: '#854d0e',
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
          const isRecording = recordingAction === action;
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
                <span
                  data-testid={`status-badge-${action}`}
                  style={{
                    fontSize: 'var(--text-xs)',
                    fontFamily: 'var(--font-mono)',
                    padding: '2px 8px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: item.is_active ? '#dcfce7' : '#f1f5f9',
                    color: item.is_active ? '#166534' : '#64748b',
                    fontWeight: 500,
                  }}
                >
                  {item.is_active ? 'Active' : 'Disabled'}
                </span>
              </div>

              <div
                style={{
                  display: 'flex',
                  gap: 'var(--space-2)',
                  alignItems: 'center',
                }}
              >
                <div
                  tabIndex={0}
                  role="button"
                  aria-label={`Record shortcut for ${label}`}
                  onClick={() => setRecordingAction(action)}
                  onKeyDown={(e) => handleKeyDown(action, e)}
                  onBlur={() => {
                    if (recordingAction === action) setRecordingAction(null);
                  }}
                  style={{
                    flex: 1,
                    padding: '8px 12px',
                    borderRadius: 'var(--radius-sm)',
                    border: isRecording ? '2px solid var(--color-accent)' : '1px solid var(--color-border)',
                    backgroundColor: isRecording ? '#eff6ff' : 'var(--color-surface)',
                    color: isRecording ? 'var(--color-accent)' : 'var(--color-text)',
                    fontFamily: 'var(--font-mono)',
                    fontSize: 'var(--text-xs)',
                    fontWeight: 600,
                    cursor: 'pointer',
                    outline: 'none',
                    textAlign: 'center',
                  }}
                >
                  {isRecording ? 'Press shortcut keys on keyboard (ESC to cancel)...' : currentVal || 'Click to record shortcut'}
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
