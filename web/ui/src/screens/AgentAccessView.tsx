import React, { useState } from 'react';
import { Button } from '../components/Button';
import { Badge } from '../components/Badge';

export interface AgentAccessViewProps {
  hasActiveKey: boolean;
  keyId?: string | null;
  issuedAt?: string | null;
  onGenerateKey: () => Promise<string>;
  onRevokeKey: () => Promise<void>;
}

export const AgentAccessView: React.FC<AgentAccessViewProps> = ({
  hasActiveKey,
  keyId,
  issuedAt,
  onGenerateKey,
  onRevokeKey,
}) => {
  const [newSecret, setNewSecret] = useState<string | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isRevoking, setIsRevoking] = useState(false);
  const [copyFeedback, setCopyFeedback] = useState(false);

  const handleGenerate = async () => {
    setIsGenerating(true);
    try {
      const secret = await onGenerateKey();
      setNewSecret(secret);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleRevoke = async () => {
    setIsRevoking(true);
    try {
      await onRevokeKey();
      setNewSecret(null);
    } finally {
      setIsRevoking(false);
    }
  };

  const handleCopy = async () => {
    if (!newSecret) return;
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(newSecret);
    }
    setCopyFeedback(true);
    setTimeout(() => setCopyFeedback(false), 2000);
  };

  return (
    <div
      data-testid="agent-access-panel"
      style={{
        padding: 'var(--space-5)',
        backgroundColor: 'var(--color-surface)',
        border: '1px solid var(--color-border)',
        borderRadius: 'var(--radius-md)',
        display: 'flex',
        flexDirection: 'column',
        gap: 'var(--space-4)',
      }}
    >
      <div>
        <h2 style={{ margin: 0, fontSize: 'var(--text-lg)', fontWeight: 600, color: 'var(--color-text)' }}>Local Agent Access</h2>
        <p style={{ margin: 'var(--space-1) 0 0 0', fontSize: 'var(--text-sm)', color: 'var(--color-text-muted)' }}>
          Grant local AI coding agents read-only access to review bundles via stdio bridge or local HTTP API.
        </p>
      </div>

      {/* Key Status Block */}
      <div
        data-testid="key-status-block"
        style={{
          padding: 'var(--space-4)',
          backgroundColor: 'var(--color-bg)',
          border: '1px solid var(--color-border)',
          borderRadius: 'var(--radius-sm)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--space-2)' }}>
            <span style={{ fontSize: 'var(--text-sm)', fontWeight: 600, color: 'var(--color-text)' }}>Status:</span>
            <span data-testid="key-active-badge">
              <Badge variant={hasActiveKey ? 'success' : 'neutral'}>
                {hasActiveKey ? 'Active Key Configured' : 'No Active Key'}
              </Badge>
            </span>
          </div>

          {hasActiveKey && issuedAt && (
            <div style={{ marginTop: 'var(--space-1)', fontSize: 'var(--text-xs)', color: 'var(--color-text-muted)' }}>
              Issued at: {issuedAt} (Key ID: {keyId})
            </div>
          )}
        </div>

        <div style={{ display: 'flex', gap: 'var(--space-2)' }}>
          <Button variant="primary" onClick={handleGenerate} disabled={isGenerating}>
            {isGenerating ? 'Generating...' : hasActiveKey ? 'Rotate / New Key' : 'Generate Access Key'}
          </Button>
          {hasActiveKey && (
            <Button variant="secondary" onClick={handleRevoke} disabled={isRevoking}>
              {isRevoking ? 'Revoking...' : 'Revoke Key'}
            </Button>
          )}
        </div>
      </div>

      {/* Newly Generated Secret Banner */}
      {newSecret && (
        <div
          data-testid="generated-key-banner"
          style={{
            padding: 'var(--space-4)',
            backgroundColor: 'var(--color-info-bg)',
            border: '1px solid var(--color-border)',
            borderRadius: 'var(--radius-sm)',
            display: 'flex',
            flexDirection: 'column',
            gap: 'var(--space-2)',
          }}
        >
          <div style={{ fontWeight: 600, fontSize: 'var(--text-sm)', color: 'var(--color-info-text)' }}>
            Save your Access Key now (it will not be displayed again):
          </div>
          <div style={{ display: 'flex', gap: 'var(--space-2)', alignItems: 'center' }}>
            <code
              data-testid="secret-token-display"
              style={{
                flex: 1,
                padding: 'var(--space-2) var(--space-3)',
                backgroundColor: 'var(--color-surface)',
                border: '1px solid var(--color-border)',
                borderRadius: 'var(--radius-sm)',
                fontFamily: 'var(--font-mono)',
                fontSize: 'var(--text-sm)',
                color: 'var(--color-text)',
              }}
            >
              {newSecret}
            </code>
            <Button variant="primary" onClick={handleCopy}>
              Copy Key
            </Button>
            {copyFeedback && (
              <span data-testid="copy-feedback" style={{ fontSize: 'var(--text-xs)', color: 'var(--color-success-text)', fontWeight: 600 }}>
                Copied!
              </span>
            )}
          </div>
          <p style={{ margin: 0, fontSize: 'var(--text-xs)', color: 'var(--color-info-text)' }}>
            Configure your AI assistant using: <code>mcp:set_access_key</code> or <code>Authorization: Bearer &lt;key&gt;</code>
          </p>
        </div>
      )}
    </div>
  );
};
