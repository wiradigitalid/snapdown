export interface QualityBudget {
  max_long_edge: number;
  encoder_quality: number;
}

export interface Settings {
  vault_path: string;
  quality_budget: QualityBudget;
  latest_finding_size: number | null;
}
