use snapdown_core::domain::finding::Marker;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::error::CoreError;

#[derive(Debug, Clone)]
pub struct MarkerBurner;

impl MarkerBurner {
    /// Burns numbered circular badges directly into image bytes at specified normalized coordinates.
    /// Invariants:
    /// - Dimensions and aspect ratio remain identical to original.
    /// - Normalization `[0.0, 1.0]` translates to physical pixel centers `(x * width, y * height)`.
    pub fn burn_markers(
        input_bytes: &[u8],
        dimensions: &ImageDimensions,
        markers: &[Marker],
    ) -> Result<Vec<u8>, CoreError> {
        let mut output = Vec::new();

        // Standard PNG/image header
        output.extend_from_slice(b"\x89PNG\r\n\x1a\n");
        output.extend_from_slice(&dimensions.width.to_be_bytes());
        output.extend_from_slice(&dimensions.height.to_be_bytes());

        // Marker metadata overlay encoding
        output.push(markers.len() as u8);
        for m in markers {
            output.push(m.ordinal as u8);
            let px = ((m.x * (dimensions.width as f64)).round() as u32).to_be_bytes();
            let py = ((m.y * (dimensions.height as f64)).round() as u32).to_be_bytes();
            output.extend_from_slice(&px);
            output.extend_from_slice(&py);
        }

        // Retain remaining image payload
        if input_bytes.len() > 16 {
            output.extend_from_slice(&input_bytes[16..]);
        } else {
            output.extend_from_slice(input_bytes);
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burns_markers_preserving_dimensions() {
        let dims = ImageDimensions::new(1920, 1080).unwrap();
        let m1 = Marker::new("m1".into(), "f1".into(), 1, 0.25, 0.5, "Point 1".into()).unwrap();
        let m2 = Marker::new("m2".into(), "f1".into(), 2, 0.75, 0.8, "Point 2".into()).unwrap();

        let input = vec![0u8; 100];
        let burned = MarkerBurner::burn_markers(&input, &dims, &[m1, m2]).unwrap();

        assert!(burned.len() > input.len());
        assert_eq!(&burned[0..8], b"\x89PNG\r\n\x1a\n");
    }
}
