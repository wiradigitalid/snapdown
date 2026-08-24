import React, { useCallback, useEffect, useState } from 'react';
import { AgentAccessView as SharedAgentAccessView } from '@snapdown/ui';
import {
  AccessKeyStatusDto,
  generateAccessKey,
  getAccessKeyStatus,
  revokeAccessKey,
} from '../services/agent_access';

export const AgentAccessView: React.FC = () => {
  const [status, setStatus] = useState<AccessKeyStatusDto>({
    has_active_key: false,
    key_id: null,
    issued_at: null,
  });

  const fetchStatus = useCallback(async () => {
    try {
      const res = await getAccessKeyStatus();
      setStatus(res);
    } catch (err) {
      console.error('Failed to get access key status:', err);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  const handleGenerate = async (): Promise<string> => {
    const res = await generateAccessKey();
    await fetchStatus();
    return res.secret;
  };

  const handleRevoke = async (): Promise<void> => {
    await revokeAccessKey();
    await fetchStatus();
  };

  return (
    <div data-testid="desktop-agent-access-view" style={{ padding: 0, width: '100%', boxSizing: 'border-box' }}>
      <SharedAgentAccessView
        hasActiveKey={status.has_active_key}
        keyId={status.key_id}
        issuedAt={status.issued_at}
        onGenerateKey={handleGenerate}
        onRevokeKey={handleRevoke}
      />
    </div>
  );
};
