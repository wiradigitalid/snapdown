use serde::{Deserialize, Serialize};

use crate::domain::setting::QualityBudget;
use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    pub fn new(width: u32, height: u32) -> Result<Self, CoreError> {
        if width == 0 || height == 0 {
            return Err(CoreError::Validation(
                "Image dimensions must be non-zero".to_string(),
            ));
        }
        Ok(Self { width, height })
    }

    pub fn long_edge(&self) -> u32 {
        self.width.max(self.height)
    }

    /// Computes downscaled dimensions according to a QualityBudget.
    /// BR-40: An image already within the Quality Budget's long edge is not upscaled.
    /// BR-41: Reduction preserves aspect ratio. Never stretched, never cropped.
    pub fn compute_reduced_dimensions(&self, budget: &QualityBudget) -> Self {
        let current_long_edge = self.long_edge();
        if current_long_edge <= budget.max_long_edge {
            return self.clone();
        }

        let scale = (budget.max_long_edge as f64) / (current_long_edge as f64);
        let new_w = ((self.width as f64) * scale).round().max(1.0) as u32;
        let new_h = ((self.height as f64) * scale).round().max(1.0) as u32;

        Self {
            width: new_w,
            height: new_h,
        }
    }

    /// Computes thumbnail dimensions fitting within `max_thumb_edge` while preserving aspect ratio.
    pub fn compute_thumbnail_dimensions(&self, max_thumb_edge: u32) -> Self {
        let current_long_edge = self.long_edge();
        if current_long_edge <= max_thumb_edge {
            return self.clone();
        }

        let scale = (max_thumb_edge as f64) / (current_long_edge as f64);
        let new_w = ((self.width as f64) * scale).round().max(1.0) as u32;
        let new_h = ((self.height as f64) * scale).round().max(1.0) as u32;

        Self {
            width: new_w,
            height: new_h,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_validation_and_long_edge() {
        assert!(ImageDimensions::new(0, 100).is_err());
        assert!(ImageDimensions::new(100, 0).is_err());

        let dim = ImageDimensions::new(1920, 1080).unwrap();
        assert_eq!(dim.long_edge(), 1920);

        let dim_portrait = ImageDimensions::new(1080, 1920).unwrap();
        assert_eq!(dim_portrait.long_edge(), 1920);
    }

    #[test]
    fn quality_budget_downscaling_and_aspect_ratio_preservation() {
        let budget = QualityBudget::new(1600, 75).unwrap();

        // Already smaller -> no upscale (BR-40)
        let small = ImageDimensions::new(800, 600).unwrap();
        let reduced_small = small.compute_reduced_dimensions(&budget);
        assert_eq!(reduced_small.width, 800);
        assert_eq!(reduced_small.height, 600);

        // Larger 4K -> downscaled to 1600 long edge (BR-41)
        let large_4k = ImageDimensions::new(3840, 2160).unwrap();
        let reduced_4k = large_4k.compute_reduced_dimensions(&budget);
        assert_eq!(reduced_4k.width, 1600);
        assert_eq!(reduced_4k.height, 900); // 1600 * (2160 / 3840) = 900
    }

    #[test]
    fn thumbnail_generation_dimensions_preserve_aspect_ratio() {
        let dim = ImageDimensions::new(1920, 1080).unwrap();
        let thumb = dim.compute_thumbnail_dimensions(320);
        assert_eq!(thumb.width, 320);
        assert_eq!(thumb.height, 180);
    }
}
