export type NamedBudget = 'auto' | 'sharp' | 'balanced' | 'small' | 'custom';

export interface ResolvedPair {
  max_long_edge: number;
  encoder_quality: number;
}

export interface QualityBudgetPresetDto {
  name: string;
  label: string;
  prose: string;
  fixed_pair: ResolvedPair | null;
}

export interface LatestFindingAttributionDto {
  size_bytes: number;
  width: number;
  height: number;
  budget_name: string;
}

export interface QualityBudget {
  named: NamedBudget;
  prose?: string;
  custom_pair?: ResolvedPair | null;
  max_long_edge?: number;
  encoder_quality?: number;
}

export interface Settings {
  vault_path: string;
  quality_budget: QualityBudget;
  latest_finding_size: number | null;
  latest_finding?: LatestFindingAttributionDto | null;
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