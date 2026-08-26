import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { AgentAccessView } from '../components/AgentAccessView';
import * as agentAccessService from '../services/agent_access';

vi.mock('../services/agent_access', () => ({
  getAccessKeyStatus: vi.fn(),
  generateAccessKey: vi.fn(),
  revokeAccessKey: vi.fn(),
}));

describe('AgentAccessView Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders no active key state initially', async () => {
    vi.mocked(agentAccessService.getAccessKeyStatus).mockResolvedValue({
      has_active_key: false,
      key_id: null,
      issued_at: null,
    });

    render(<AgentAccessView />);

    await waitFor(() => {
      expect(screen.getByTestId('desktop-agent-access-view')).toBeInTheDocument();
      expect(screen.getByText('No Active Key')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Generate Access Key' })).toBeInTheDocument();
    });
  });

  it('generates access key and displays secret token banner', async () => {
    vi.mocked(agentAccessService.getAccessKeyStatus)
      .mockResolvedValueOnce({
        has_active_key: false,
        key_id: null,
        issued_at: null,
      })
      .mockResolvedValueOnce({
        has_active_key: true,
        key_id: 'k-12345',
        issued_at: '2026-08-23T10:00:00Z',
      });

    vi.mocked(agentAccessService.generateAccessKey).mockResolvedValue({
      key_id: 'k-12345',
      secret: 'sd_key_secret_generated_123',
      issued_at: '2026-08-23T10:00:00Z',
    });

    render(<AgentAccessView />);

    const generateBtn = await screen.findByRole('button', { name: 'Generate Access Key' });
    fireEvent.click(generateBtn);

    await waitFor(() => {
      expect(screen.getByTestId('generated-key-banner')).toBeInTheDocument();
      expect(screen.getByText('sd_key_secret_generated_123')).toBeInTheDocument();
      expect(screen.getByText('Active Key Configured')).toBeInTheDocument();
    });
  });

  it('revokes access key', async () => {
    vi.mocked(agentAccessService.getAccessKeyStatus)
      .mockResolvedValueOnce({
        has_active_key: true,
        key_id: 'k-12345',
        issued_at: '2026-08-23T10:00:00Z',
      })
      .mockResolvedValueOnce({
        has_active_key: false,
        key_id: null,
        issued_at: null,
      });

    vi.mocked(agentAccessService.revokeAccessKey).mockResolvedValue();

    render(<AgentAccessView />);

    const revokeBtn = await screen.findByRole('button', { name: 'Revoke Key' });
    fireEvent.click(revokeBtn);

    await waitFor(() => {
      expect(agentAccessService.revokeAccessKey).toHaveBeenCalledTimes(1);
      expect(screen.getByText('No Active Key')).toBeInTheDocument();
    });
  });
});
