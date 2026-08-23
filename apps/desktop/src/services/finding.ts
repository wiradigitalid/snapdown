import { invoke } from '@tauri-apps/api/core';

export interface FindingDto {
  id: string;
  image_path: string;
  image_width: number;
  image_height: number;
  captured_at: string;
  source_monitor: string;
  region: string;
}

export interface NoteDto {
  id: string;
  finding_id: string;
  body: string;
  updated_at: string;
}

export interface MarkerDto {
  id: string;
  finding_id: string;
  ordinal: number;
  x: number;
  y: number;
  comment: string;
}

export interface FindingDetailDto {
  finding: FindingDto;
  note: NoteDto;
  markers: MarkerDto[];
}

export const listFindings = async (): Promise<FindingDetailDto[]> => {
  return await invoke<FindingDetailDto[]>('list_findings');
};

export const getFindingDetail = async (
  id: string
): Promise<FindingDetailDto | null> => {
  return await invoke<FindingDetailDto | null>('get_finding_detail', { id });
};

export const saveNote = async (
  findingId: string,
  body: string
): Promise<void> => {
  await invoke<void>('save_note', {
    findingId,
    body,
  });
};

export const deleteFinding = async (id: string): Promise<void> => {
  await invoke<void>('delete_finding', { id });
};

export const addMarker = async (
  findingId: string,
  markerId: string,
  x: number,
  y: number,
  comment: string
): Promise<MarkerDto> => {
  return await invoke<MarkerDto>('add_marker', {
    findingId,
    markerId,
    x,
    y,
    comment,
  });
};

export const updateMarker = async (
  findingId: string,
  markerId: string,
  x: number,
  y: number,
  comment: string
): Promise<MarkerDto> => {
  return await invoke<MarkerDto>('update_marker', {
    findingId,
    markerId,
    x,
    y,
    comment,
  });
};

export const deleteMarker = async (
  findingId: string,
  markerId: string
): Promise<void> => {
  await invoke<void>('delete_marker', {
    findingId,
    markerId,
  });
};
