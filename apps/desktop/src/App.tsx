import React, { useState } from 'react';
import { Button, TextField, Toast } from '@snapdown/ui';

export const App: React.FC = () => {
  const [activeRoute] = useState<string>('/settings');
  const [showToast, setShowToast] = useState<boolean>(false);

  return (
    <main
      data-testid="app-shell"
      style={{
        padding: 'var(--space-6)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
        maxWidth: '48rem',
        margin: '0 auto',
      }}
    >
      <header
        style={{
          borderBottom: '1px solid var(--color-border)',
          paddingBottom: 'var(--space-4)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div>
          <h1
            style={{
              margin: 0,
              fontSize: 'var(--text-xl)',
              fontWeight: 700,
              fontFamily: 'var(--font-ui)',
            }}
          >
            Snapdown Settings
          </h1>
          <p
            style={{
              margin: 'var(--space-1) 0 0',
              fontSize: 'var(--text-sm)',
              color: 'var(--color-text-muted)',
            }}
          >
            Active Route: {activeRoute}
          </p>
        </div>
        <Button variant="primary" onClick={() => setShowToast(true)}>
          Save Configuration
        </Button>
      </header>

      <section
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--space-4)',
          backgroundColor: 'var(--color-surface)',
          padding: 'var(--space-5)',
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--color-border)',
        }}
      >
        <h2
          style={{
            margin: 0,
            fontSize: 'var(--text-lg)',
            fontFamily: 'var(--font-ui)',
          }}
        >
          General
        </h2>
        <TextField
          label="Vault Path"
          placeholder="e.g. D:/SnapdownVault"
          defaultValue=""
        />
      </section>

      {showToast && (
        <Toast
          message="Settings updated successfully"
          onDismiss={() => setShowToast(false)}
        />
      )}
    </main>
  );
};
