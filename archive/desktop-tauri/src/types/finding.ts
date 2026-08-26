export interface Finding {
  id: string;
  image_path: string;
  image_width: number;
  image_height: number;
  captured_at: string;
  source_monitor: string;
  region: string;
  resolved_long_edge?: number | null;
  resolved_encoder_quality?: number | null;
  budget_name?: string | null;
}

export interface Note {
  id: string;
  finding_id: string;
  body: string;
  updated_at: string;
}

export interface Marker {
  id: string;
  finding_id: string;
  ordinal: number;
  x: number;
  y: number;
  comment: string;
}

export interface FindingDetail {
  finding: Finding;
  note: Note;
  markers: Marker[];
}