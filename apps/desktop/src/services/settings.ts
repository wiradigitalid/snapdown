import { invoke } from '@tauri-apps/api/core';
import { HotkeyAction, HotkeySettingsDto, QualityBudget, Settings } from '../types/settings';

export const getSettings = async (): Promise<Settings> => {
  return await invoke<Settings>('get_settings');
};

export const setVaultPath = async (
  newPath: string,
  migrateFiles: boolean
): Promise<string> => {
  return await invoke<string>('set_vault_path', {
    newPath,
    migrateFiles,
  });
};

export const setQualityBudget = async (
  maxLongEdge: number,
  encoderQuality: number
): Promise<QualityBudget> => {
  return await invoke<QualityBudget>('set_quality_budget', {
    maxLongEdge,
    encoderQuality,
  });
};

export const getLatestFindingSize = async (): Promise<number | null> => {
  return await invoke<number | null>('get_latest_finding_size');
};

export const openVaultFolder = async (): Promise<void> => {
  await invoke<void>('open_vault_folder');
};

export const getHotkeys = async (): Promise<HotkeySettingsDto> => {
  return await invoke<HotkeySettingsDto>('get_hotkeys');
};

export const setHotkey = async (
  action: HotkeyAction,
  shortcut: string
): Promise<void> => {
  await invoke<void>('set_hotkey', {
    action,
    shortcut,
  });
};

export const clearHotkey = async (action: HotkeyAction): Promise<void> => {
  await invoke<void>('clear_hotkey', {
    action,
  });
};
