import React from 'react';
import { Button, Toggle } from '@snapdown/ui';
import { StartupState } from '../types/settings';

export interface GeneralSectionProps {
  startupStatus: StartupState;
  onToggleStartup: (enabled: boolean) => Promise<void>;
  onRetryStartup?: () => Promise<void> | void;
  disabled?: boolean;
}

export const GeneralSection: React.FC<GeneralSectionProps> = ({
  startupStatus,
  onToggleStartup,
  onRetryStartup,
  disabled = false,
}) => {
  const isIndeterminate = startupStatus === 'unknown';
  const isChecked = startupStatus === 'on';
  const isToggleDisabled = disabled || isIndeterminate;

  return (
    <section
      data-testid="general-section"
      aria-label="Startup"
      style={{
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-3)',
        padding: 'var(--space-4)',
        border: '1px solid var(--color-border)',
        borderRadius: 'var(--radius-md)',
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
          Startup
        </h2>
        <p
          style={{
            margin: 'var(--space-1) 0 0 0',
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Configure application launch and background startup behavior.
        </p>
      </div>

      <div
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--space-2)',
          paddingTop: 'var(--space-1)',
        }}
      >
        {startupStatus === 'unreadable' ? (
          <div
            data-testid="startup-unreadable-message"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: 'var(--space-2) var(--space-3)',
              backgroundColor: 'var(--color-warning-bg)',
              color: 'var(--color-warning-text)',
              borderRadius: 'var(--radius-sm)',
              fontSize: 'var(--text-xs)',
              fontFamily: 'var(--font-ui)',
            }}
          >
            <span>Could not read Windows startup status</span>
            {onRetryStartup && (
              <Button
                variant="secondary"
                onClick={() => void onRetryStartup()}
                style={{ padding: 'var(--space-1) var(--space-2)', fontSize: 'var(--text-xs)' }}
              >
                Retry
              </Button>
            )}
          </div>
        ) : (
          <div
            style={{
              minHeight: 'var(--settings-row-height)',
              display: 'flex',
              alignItems: 'center',
              gap: 'var(--space-3)',
            }}
          >
            <Toggle
              id="startup-toggle-switch"
              data-testid="startup-toggle"
              aria-label="Run at Windows startup"
              indeterminate={isIndeterminate}
              checked={isChecked}
              disabled={isToggleDisabled}
              onChange={(newChecked) => {
                void onToggleStartup(newChecked);
              }}
            />
            <label
              htmlFor="startup-toggle-switch"
              style={{
                fontSize: 'var(--text-sm)',
                fontFamily: 'var(--font-ui)',
                color: 'var(--color-text)',
                cursor: isToggleDisabled ? 'default' : 'pointer',
                userSelect: 'none',
              }}
            >
              Run at Windows startup
            </label>
          </div>
        )}
        <span
          style={{
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
          }}
        >
          Starts Snapdown silently in the system tray when you sign in to Windows.
        </span>
      </div>
    </section>
  );
};
