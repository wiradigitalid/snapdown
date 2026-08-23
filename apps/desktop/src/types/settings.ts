export interface QualityBudget {
  max_long_edge: number;
  encoder_quality: number;
}

export interface Settings {
  vault_path: string;
  quality_budget: QualityBudget;
  latest_finding_size: number | null;
}

export type HotkeyAction = 'capture' | 'open_editor';

export interface HotkeyItem {
  action: HotkeyAction;
  shortcut: string;
  is_registered: boolean;
  is_active: boolean;
}

export interface HotkeySettingsDto {
  hotkeys: HotkeyItem[];
  startup_warnings: string[];
}

export interface StartupSettingsDto {
  enabled: boolean;
}
