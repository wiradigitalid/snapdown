import { invoke } from '@tauri-apps/api/core';
import {
  HotkeyAction,
  HotkeySettingsDto,
  NamedBudget,
  QualityBudget,
  QualityBudgetPresetDto,
  ResolvedPair,
  Settings,
  StartupSettingsDto,
} from '../types/settings';

export const getSettings = async (): Promise<Settings> => {
  return await invoke<Settings>('get_settings');
};

export const getQualityBudgetPresets = async (): Promise<QualityBudgetPresetDto[]> => {
  return await invoke<QualityBudgetPresetDto[]>('get_quality_budget_presets');
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
  budget: NamedBudget,
  advanced?: ResolvedPair | null
): Promise<QualityBudget> => {
  return await invoke<QualityBudget>('set_quality_budget', {
    budget,
    advanced: advanced || null,
  });
};

export const getLatestFindingSize = async (): Promise<number | null> => {
  return await invoke<number | null>('get_latest_finding_size');
};

export const openVaultFolder = async (): Promise<void> => {
  await invoke<void>('open_vault_folder');
};

export const pickVaultFolder = async (): Promise<string | null> => {
  return await invoke<string | null>('pick_vault_folder');
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

export const getStartupStatus = async (): Promise<StartupSettingsDto> => {
  return await invoke<StartupSettingsDto>('get_startup_status');
};

export const setStartupStatus = async (
  enabled: boolean
): Promise<StartupSettingsDto> => {
  return await invoke<StartupSettingsDto>('set_startup_status', {
    enabled,
  });
};