import { invoke } from '@tauri-apps/api/core';

export interface BundleDto {
  id: string;
  name: string;
  markdown: string;
  markdown_path: string;
  composed_at: string;
}

export interface BundleItemDto {
  id: string;
  bundle_id: string;
  finding_id: string;
  position: number;
  image_path: string;
}

export interface BundleDetailDto {
  bundle: BundleDto;
  items: BundleItemDto[];
}

export interface CreateBundlePayload {
  name: string;
  finding_ids: string[];
}

export const createBundle = async (
  input: CreateBundlePayload
): Promise<BundleDetailDto> => {
  return await invoke<BundleDetailDto>('create_bundle', { input });
};

export const listBundles = async (): Promise<BundleDetailDto[]> => {
  return await invoke<BundleDetailDto[]>('list_bundles');
};

export const getBundleDetail = async (
  id: string
): Promise<BundleDetailDto | null> => {
  return await invoke<BundleDetailDto | null>('get_bundle_detail', { id });
};

export const deleteBundle = async (id: string): Promise<void> => {
  await invoke<void>('delete_bundle', { id });
};
