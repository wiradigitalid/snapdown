import { invoke } from '@tauri-apps/api/core';

export interface CaptureRegionPayload {
  x: number;
  y: number;
  width: number;
  height: number;
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
