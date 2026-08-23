import React from 'react';
import { Checkbox } from '@snapdown/ui';

export interface GeneralSectionProps {
  runAtStartup: boolean;
  onToggleStartup: (enabled: boolean) => Promise<void>;
  disabled?: boolean;
}

export const GeneralSection: React.FC<GeneralSectionProps> = ({
  runAtStartup,
  onToggleStartup,
  disabled = false,
}) => {
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
        <div style={{ minHeight: 'var(--settings-row-height)', display: 'flex', alignItems: 'center' }}>
          <Checkbox
            data-testid="startup-toggle"
            label="Run at Windows startup"
            checked={runAtStartup}
            disabled={disabled}
            onChange={(e) => {
              void onToggleStartup(e.target.checked);
            }}
          />
        </div>
        <span
          style={{
            fontSize: 'var(--text-xs)',
            fontFamily: 'var(--font-ui)',
            color: 'var(--color-text-muted)',
            marginLeft: 'var(--space-5)',
          }}
        >
          Starts Snapdown silently in the system tray when you sign in to Windows.
        </span>
      </div>
    </section>
  );
};
