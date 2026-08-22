import React from 'react';
import { TextField } from '@snapdown/ui';

export const App: React.FC = () => {
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
        </div>
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
    </main>
  );
};
