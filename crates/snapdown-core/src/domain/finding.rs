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
