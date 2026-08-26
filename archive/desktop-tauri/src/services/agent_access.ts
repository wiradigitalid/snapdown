import { invoke } from '@tauri-apps/api/core';

export interface AccessKeyStatusDto {
  has_active_key: boolean;
  key_id: string | null;
  issued_at: string | null;
}

export interface GeneratedAccessKeyDto {
  key_id: string;
  secret: string;
  issued_at: string;
}

export const getAccessKeyStatus = async (): Promise<AccessKeyStatusDto> => {
  return await invoke<AccessKeyStatusDto>('get_access_key_status');
};

export const generateAccessKey = async (): Promise<GeneratedAccessKeyDto> => {
  return await invoke<GeneratedAccessKeyDto>('generate_access_key');
};

export const revokeAccessKey = async (): Promise<void> => {
  await invoke<void>('revoke_access_key');
};
