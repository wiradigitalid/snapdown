import React, { useState } from 'react';
import { Button } from '../components/Button';

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
        padding: '24px',
        backgroundColor: '#ffffff',
        border: '1px solid #e2e8f0',
        borderRadius: '8px',
        display: 'flex',
        flexDirection: 'column',
        gap: '20px',
      }}
    >
      <div>
        <h2 style={{ margin: 0, fontSize: '18px', fontWeight: 600 }}>Local Agent Access</h2>
        <p style={{ margin: '4px 0 0 0', fontSize: '13px', color: '#64748b' }}>
          Grant local AI coding agents read-only access to review bundles via stdio bridge or local HTTP API.
        </p>
      </div>

      {/* Key Status Block */}
      <div
        data-testid="key-status-block"
        style={{
          padding: '16px',
          backgroundColor: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: '6px',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: '13px', fontWeight: 600 }}>Status:</span>
            <span
              data-testid="key-active-badge"
              style={{
                fontSize: '12px',
                fontWeight: 600,
                padding: '2px 8px',
                borderRadius: '4px',
                backgroundColor: hasActiveKey ? '#dcfce7' : '#f1f5f9',
                color: hasActiveKey ? '#15803d' : '#64748b',
              }}
            >
              {hasActiveKey ? 'Active Key Configured' : 'No Active Key'}
            </span>
          </div>

          {hasActiveKey && issuedAt && (
            <div style={{ marginTop: '6px', fontSize: '12px', color: '#64748b' }}>
              Issued at: {issuedAt} (Key ID: {keyId})
            </div>
          )}
        </div>

        <div style={{ display: 'flex', gap: '8px' }}>
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
            padding: '16px',
            backgroundColor: '#eff6ff',
            border: '1px solid #bfdbfe',
            borderRadius: '6px',
            display: 'flex',
            flexDirection: 'column',
            gap: '10px',
          }}
        >
          <div style={{ fontWeight: 600, fontSize: '13px', color: '#1e40af' }}>
            🔑 Save your Access Key now (it will not be displayed again):
          </div>
          <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
            <code
              data-testid="secret-token-display"
              style={{
                flex: 1,
                padding: '8px 12px',
                backgroundColor: '#ffffff',
                border: '1px solid #93c5fd',
                borderRadius: '4px',
                fontFamily: 'monospace',
                fontSize: '13px',
                color: '#1e3a8a',
              }}
            >
              {newSecret}
            </code>
            <Button variant="primary" onClick={handleCopy}>
              Copy Key
            </Button>
            {copyFeedback && (
              <span data-testid="copy-feedback" style={{ fontSize: '12px', color: '#16a34a' }}>
                Copied!
              </span>
            )}
          </div>
          <p style={{ margin: 0, fontSize: '12px', color: '#3b82f6' }}>
            Configure your AI assistant using: <code>mcp:set_access_key</code> or <code>Authorization: Bearer &lt;key&gt;</code>
          </p>
        </div>
      )}
    </div>
  );
};
