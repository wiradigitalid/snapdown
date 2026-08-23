use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn long_edge(&self) -> u32 {
        self.width.max(self.height)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub image_path: String,
    pub image_width: u32,
    pub image_height: u32,
    pub captured_at: String,
    pub source_monitor: String,
    pub region: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_long_edge: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_encoder_quality: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub finding_id: String,
    pub body: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub id: String,
    pub finding_id: String,
    pub ordinal: u32,
    pub x: f64,
    pub y: f64,
    pub comment: String,
}

impl Marker {
    pub fn validate_coordinates(x: f64, y: f64) -> Result<(), CoreError> {
        if !(0.0..=1.0).contains(&x) || x.is_nan() {
            return Err(CoreError::Validation(format!(
                "Marker x coordinate must be in [0.0, 1.0], got {x}"
            )));
        }
        if !(0.0..=1.0).contains(&y) || y.is_nan() {
            return Err(CoreError::Validation(format!(
                "Marker y coordinate must be in [0.0, 1.0], got {y}"
            )));
        }
        Ok(())
    }

    pub fn new(
        id: String,
        finding_id: String,
        ordinal: u32,
        x: f64,
        y: f64,
        comment: String,
    ) -> Result<Self, CoreError> {
        Self::validate_coordinates(x, y)?;
        if ordinal == 0 {
            return Err(CoreError::Validation(
                "Marker ordinal must be >= 1".to_string(),
            ));
        }
        Ok(Self {
            id,
            finding_id,
            ordinal,
            x,
            y,
            comment,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingDetail {
    pub finding: Finding,
    pub note: Note,
    pub markers: Vec<Marker>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::setting::{NamedBudget, QualityBudget};

    #[test]
    fn auto_resolves_a_different_pair_for_a_small_region_than_for_a_full_screen() {
        let budget = QualityBudget::new(NamedBudget::Auto, None);

        // Tooltip region: 312 x 118
        let tooltip_region = Region::new(100, 100, 312, 118);
        let resolved_a = budget.resolve(tooltip_region.long_edge());

        // 4K Dashboard: 3840 x 2160
        let screen_region = Region::new(0, 0, 3840, 2160);
        let resolved_b = budget.resolve(screen_region.long_edge());

        // SCN-03 key assertion: resolved(A) != resolved(B)
        assert_ne!(
            resolved_a, resolved_b,
            "Auto budget must resolve different parameter pairs for small tooltip vs 4K dashboard"
        );
        assert_ne!(resolved_a.encoder_quality, resolved_b.encoder_quality);
    }

    #[test]
    fn auto_resolves_a_higher_encoder_quality_when_no_downscale_applies() {
        let budget = QualityBudget::new(NamedBudget::Auto, None);

        // Small tooltip (no downscale cap needed)
        let tooltip_region = Region::new(0, 0, 312, 118);
        let resolved_small = budget.resolve(tooltip_region.long_edge());

        // Large 4K screen (downscaled)
        let screen_region = Region::new(0, 0, 3840, 2160);
        let resolved_large = budget.resolve(screen_region.long_edge());

        assert!(
            resolved_small.encoder_quality > resolved_large.encoder_quality,
            "Small un-downscaled region must receive higher encoder quality (got small: {}, large: {})",
            resolved_small.encoder_quality,
            resolved_large.encoder_quality
        );
    }

    #[test]
    fn a_finding_can_state_which_named_budget_produced_it() {
        let finding = Finding {
            id: "f-1".to_string(),
            image_path: "findings/f1.png".to_string(),
            image_width: 312,
            image_height: 118,
            captured_at: "2026-08-24T00:00:00Z".to_string(),
            source_monitor: "DISPLAY1".to_string(),
            region: "0,0,312,118".to_string(),
            resolved_long_edge: Some(1280),
            resolved_encoder_quality: Some(92),
            budget_name: Some("Auto".to_string()),
        };

        assert_eq!(finding.budget_name.as_deref(), Some("Auto"));
        assert_eq!(finding.resolved_long_edge, Some(1280));
        assert_eq!(finding.resolved_encoder_quality, Some(92));
    }

    #[test]
    fn marker_coordinate_validation_bounds() {
        assert!(Marker::validate_coordinates(0.0, 0.0).is_ok());
        assert!(Marker::validate_coordinates(1.0, 1.0).is_ok());
        assert!(Marker::validate_coordinates(0.5, 0.5).is_ok());

        assert!(Marker::validate_coordinates(-0.01, 0.5).is_err());
        assert!(Marker::validate_coordinates(1.01, 0.5).is_err());
        assert!(Marker::validate_coordinates(0.5, -0.01).is_err());
        assert!(Marker::validate_coordinates(0.5, 1.01).is_err());
        assert!(Marker::validate_coordinates(f64::NAN, 0.5).is_err());
    }

    #[test]
    fn marker_ordinal_must_be_positive() {
        let m = Marker::new("id".into(), "fid".into(), 0, 0.5, 0.5, "comment".into());
        assert!(m.is_err());

        let m_valid = Marker::new("id".into(), "fid".into(), 1, 0.5, 0.5, "comment".into());
        assert!(m_valid.is_ok());
    }
}