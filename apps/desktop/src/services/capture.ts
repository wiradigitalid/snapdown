import { invoke } from '@tauri-apps/api/core';

export interface CaptureRegionPayload {
  x: number;
  y: number;
  width: number;
  height: number;
  note: string;
  source_monitor?: string;
}

export interface CaptureResultDto {
  image_path: string;
  image_width: number;
  image_height: number;
  source_monitor: string;
  region: string;
}

export const captureScreenRegion = async (
  region: CaptureRegionPayload
): Promise<CaptureResultDto> => {
  return await invoke<CaptureResultDto>('capture_screen_region', { region });
};

export const triggerOverlay = async (): Promise<void> => {
  await invoke<void>('trigger_overlay');
};

export const dismissOverlay = async (): Promise<void> => {
  await invoke<void>('dismiss_overlay');
};

export const getMonitorSnapshot = async (sourceMonitor?: string): Promise<string> => {
  return await invoke<string>('get_monitor_snapshot', { sourceMonitor });
};

export interface DetectedRegionDto {
  x: number;
  y: number;
  width: number;
  height: number;
}

export const detectWindowAtPoint = async (
  x: number,
  y: number
): Promise<DetectedRegionDto | null> => {
  return await invoke<DetectedRegionDto | null>('detect_window_at_point', { x, y });
};