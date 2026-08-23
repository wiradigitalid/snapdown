import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { App } from '../App';
import * as settingsService from '../services/settings';

vi.mock('../services/settings', () => ({
  getSettings: vi.fn(),
  setVaultPath: vi.fn(),
  setQualityBudget: vi.fn(),
  getLatestFindingSize: vi.fn(),
  openVaultFolder: vi.fn(),
}));

describe('Desktop Settings Screen (Screen 12)', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('app_renders_shell and displays settings sections', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });

    render(<App />);

    expect(screen.getByTestId('app-shell')).toBeInTheDocument();
    expect(screen.getByText('Snapdown Settings')).toBeInTheDocument();
    expect(screen.getByText('Vault Folder')).toBeInTheDocument();
    expect(screen.getByText('Quality Budget')).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
      expect(screen.getByDisplayValue('1600')).toBeInTheDocument();
      expect(screen.getByDisplayValue('75')).toBeInTheDocument();
    });
  });

  it('settings_shows_the_stored_size_of_the_latest_finding with no captures', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });

    render(<App />);

    await waitFor(() => {
      const sizeIndicator = screen.getByTestId('latest-finding-size');
      expect(sizeIndicator).toHaveTextContent('No captures yet');
    });
  });

  it('settings_shows_the_stored_size_of_the_latest_finding when capture exists', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: 245760, // 240 KB
    });

    render(<App />);

    await waitFor(() => {
      const sizeIndicator = screen.getByTestId('latest-finding-size');
      expect(sizeIndicator).toHaveTextContent('240.0 KB');
    });
  });

  it('validates quality budget inputs and refuses out-of-range values', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });

    render(<App />);

    // Wait until settings are loaded (e.g. vault path is populated from getSettings)
    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });

    const edgeInput = screen.getByLabelText('Max Long Edge (px)');
    const saveBtn = screen.getByRole('button', { name: 'Save Quality Budget' });

    // Enter out of range edge (100 px < 320 px)
    fireEvent.change(edgeInput, { target: { value: '100' } });
    expect(saveBtn).not.toBeDisabled();
    fireEvent.click(saveBtn);

    expect(screen.getByText('Max long edge must be between 320 and 7680 px')).toBeInTheDocument();
    expect(settingsService.setQualityBudget).not.toHaveBeenCalled();

    // Enter out of range quality (105 > 100)
    fireEvent.change(edgeInput, { target: { value: '1920' } });
    const qualityInput = screen.getByLabelText('Encoder Quality (10-100)');
    fireEvent.change(qualityInput, { target: { value: '105' } });
    fireEvent.click(saveBtn);

    expect(screen.getByText('Quality must be between 10% and 100%')).toBeInTheDocument();
    expect(settingsService.setQualityBudget).not.toHaveBeenCalled();

    // Enter valid values
    fireEvent.change(qualityInput, { target: { value: '85' } });
    vi.mocked(settingsService.setQualityBudget).mockResolvedValue({
      max_long_edge: 1920,
      encoder_quality: 85,
    });

    fireEvent.click(saveBtn);
    await waitFor(() => {
      expect(settingsService.setQualityBudget).toHaveBeenCalledWith(1920, 85);
      expect(screen.getByText('Quality Budget saved successfully')).toBeInTheDocument();
    });
  });

  it('opens vault folder in explorer on button click', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });
    vi.mocked(settingsService.openVaultFolder).mockResolvedValue();

    render(<App />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });

    const openExplorerBtn = screen.getByRole('button', { name: 'Open in Explorer' });
    fireEvent.click(openExplorerBtn);

    expect(settingsService.openVaultFolder).toHaveBeenCalledTimes(1);
  });

  it('triggers confirmation dialog when changing vault path', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });
    vi.mocked(settingsService.setVaultPath).mockResolvedValue('D:/NewVault');

    render(<App />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });

    const pathInput = screen.getByLabelText('Vault Path');
    fireEvent.change(pathInput, { target: { value: 'D:/NewVault' } });

    const applyBtn = screen.getByRole('button', { name: 'Apply Change' });
    fireEvent.click(applyBtn);

    // Modal dialog appears
    expect(screen.getByText('Move Existing Files?')).toBeInTheDocument();

    const moveFilesBtn = screen.getByRole('button', { name: 'Move Files' });
    fireEvent.click(moveFilesBtn);

    await waitFor(() => {
      expect(settingsService.setVaultPath).toHaveBeenCalledWith('D:/NewVault', true);
      expect(screen.getByText('Vault folder location updated successfully')).toBeInTheDocument();
    });
  });

  it('supports Leave Files option on vault path change confirmation', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });
    vi.mocked(settingsService.setVaultPath).mockResolvedValue('D:/NewVault');

    render(<App />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });

    const pathInput = screen.getByLabelText('Vault Path');
    fireEvent.change(pathInput, { target: { value: 'D:/NewVault' } });

    const applyBtn = screen.getByRole('button', { name: 'Apply Change' });
    fireEvent.click(applyBtn);

    const leaveFilesBtn = screen.getByRole('button', { name: 'Leave Files' });
    fireEvent.click(leaveFilesBtn);

    await waitFor(() => {
      expect(settingsService.setVaultPath).toHaveBeenCalledWith('D:/NewVault', false);
      expect(screen.getByText('Vault folder location updated successfully')).toBeInTheDocument();
    });
  });

  it('handles backend refusal when changing vault path to unwritable directory', async () => {
    vi.mocked(settingsService.getSettings).mockResolvedValue({
      vault_path: 'C:/Users/test/Vault',
      quality_budget: {
        max_long_edge: 1600,
        encoder_quality: 75,
      },
      latest_finding_size: null,
    });
    vi.mocked(settingsService.setVaultPath).mockRejectedValue(
      new Error('Directory is not writable: access denied')
    );

    render(<App />);

    await waitFor(() => {
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });

    const pathInput = screen.getByLabelText('Vault Path');
    fireEvent.change(pathInput, { target: { value: 'E:/ReadOnlyVault' } });

    const applyBtn = screen.getByRole('button', { name: 'Apply Change' });
    fireEvent.click(applyBtn);

    const moveFilesBtn = screen.getByRole('button', { name: 'Move Files' });
    fireEvent.click(moveFilesBtn);

    await waitFor(() => {
      expect(
        screen.getByText('Directory is not writable: access denied')
      ).toBeInTheDocument();
      // Input resets to current valid vault path
      expect(screen.getByDisplayValue('C:/Users/test/Vault')).toBeInTheDocument();
    });
  });
});
