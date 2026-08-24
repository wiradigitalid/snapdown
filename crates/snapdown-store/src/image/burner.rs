use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::Marker;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::error::CoreError;

const COLOR_MARKER_FILL: Rgba<u8> = Rgba([245, 158, 11, 255]); // #f59e0b
const COLOR_MARKER_RING: Rgba<u8> = Rgba([255, 255, 255, 255]); // #ffffff
const COLOR_MARKER_TEXT: Rgba<u8> = Rgba([0, 0, 0, 255]); // #000000

const BADGE_RADIUS_INNER: i32 = 14;
const BADGE_RADIUS_OUTER: i32 = 16;

const DIGIT_3X5: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111], // 0
    [0b010, 0b110, 0b010, 0b010, 0b111], // 1
    [0b111, 0b001, 0b111, 0b100, 0b111], // 2
    [0b111, 0b001, 0b111, 0b001, 0b111], // 3
    [0b101, 0b101, 0b111, 0b001, 0b001], // 4
    [0b111, 0b100, 0b111, 0b001, 0b111], // 5
    [0b111, 0b100, 0b111, 0b101, 0b111], // 6
    [0b111, 0b001, 0b010, 0b010, 0b010], // 7
    [0b111, 0b101, 0b111, 0b101, 0b111], // 8
    [0b111, 0b101, 0b111, 0b001, 0b111], // 9
];

#[derive(Debug, Clone)]
pub struct MarkerBurner;

impl MarkerBurner {
    /// Burns numbered circular badges directly into image bytes at specified normalized coordinates.
    ///
    /// Invariants:
    /// - AD-4: Operates on already-reduced bytes and preserves dimensions without re-scaling.
    /// - AD-9: Returns original bytes unchanged if no eligible markers are present.
    /// - AD-10: Marker colors are theme-invariant (#f59e0b fill, #ffffff ring, #000000 text).
    /// - SCN-04: Markers with empty/whitespace-only comments are not drawn.
    pub fn burn_markers(
        input_bytes: &[u8],
        dimensions: &ImageDimensions,
        markers: &[Marker],
    ) -> Result<Vec<u8>, CoreError> {
        let active_markers: Vec<&Marker> = markers
            .iter()
            .filter(|m| !m.comment.trim().is_empty())
            .collect();

        if active_markers.is_empty() {
            return Ok(input_bytes.to_vec());
        }

        let decoded = image::load_from_memory(input_bytes).map_err(|e| {
            CoreError::Validation(format!("Failed to decode image for burning: {e}"))
        })?;

        if decoded.width() != dimensions.width || decoded.height() != dimensions.height {
            return Err(CoreError::Validation(format!(
                "Image dimensions mismatch: header says {}x{}, decoded {}x{}",
                dimensions.width,
                dimensions.height,
                decoded.width(),
                decoded.height()
            )));
        }

        let mut image_rgba: RgbaImage = decoded.to_rgba8();

        for marker in active_markers {
            let cx = (marker.x * (dimensions.width as f64)).round() as i32;
            let cy = (marker.y * (dimensions.height as f64)).round() as i32;

            Self::draw_badge(&mut image_rgba, cx, cy, marker.ordinal);
        }

        let mut output_bytes = Vec::new();
        let encoder = PngEncoder::new(&mut output_bytes);
        encoder
            .write_image(
                image_rgba.as_raw(),
                dimensions.width,
                dimensions.height,
                ExtendedColorType::Rgba8,
            )
            .map_err(|e| CoreError::Validation(format!("Failed to encode burned PNG: {e}")))?;

        Ok(output_bytes)
    }

    fn draw_badge(img: &mut RgbaImage, cx: i32, cy: i32, ordinal: u32) {
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let r_outer_sq = BADGE_RADIUS_OUTER * BADGE_RADIUS_OUTER;
        let r_inner_sq = BADGE_RADIUS_INNER * BADGE_RADIUS_INNER;

        // Draw circular ring and fill
        for dy in -BADGE_RADIUS_OUTER..=BADGE_RADIUS_OUTER {
            let py = cy + dy;
            if py < 0 || py >= img_h {
                continue;
            }
            for dx in -BADGE_RADIUS_OUTER..=BADGE_RADIUS_OUTER {
                let px = cx + dx;
                if px < 0 || px >= img_w {
                    continue;
                }

                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_inner_sq {
                    img.put_pixel(px as u32, py as u32, COLOR_MARKER_FILL);
                } else if dist_sq <= r_outer_sq {
                    img.put_pixel(px as u32, py as u32, COLOR_MARKER_RING);
                }
            }
        }

        // Draw centered digit glyph(s)
        let display_num = ordinal.clamp(1, 99);
        Self::draw_number(img, cx, cy, display_num);
    }

    fn draw_number(img: &mut RgbaImage, cx: i32, cy: i32, number: u32) {
        let scale = 2;
        if number < 10 {
            let digit = number as usize;
            let start_x = cx - (3 * scale) / 2;
            let start_y = cy - (5 * scale) / 2;
            Self::draw_digit(img, start_x, start_y, digit, scale);
        } else {
            let d1 = (number / 10) as usize;
            let d2 = (number % 10) as usize;
            let spacing = scale; // 1 pixel in font unit
            let total_width = 3 * scale + spacing + 3 * scale;
            let start_x = cx - total_width / 2;
            let start_y = cy - (5 * scale) / 2;
            Self::draw_digit(img, start_x, start_y, d1, scale);
            Self::draw_digit(img, start_x + 3 * scale + spacing, start_y, d2, scale);
        }
    }

    fn draw_digit(img: &mut RgbaImage, start_x: i32, start_y: i32, digit: usize, scale: i32) {
        if digit > 9 {
            return;
        }
        let glyph = DIGIT_3X5[digit];
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        for (row_idx, &row_bits) in glyph.iter().enumerate() {
            for col_idx in 0..3 {
                let bit_set = (row_bits & (1 << (2 - col_idx))) != 0;
                if bit_set {
                    for sy in 0..scale {
                        let py = start_y + (row_idx as i32) * scale + sy;
                        if py < 0 || py >= img_h {
                            continue;
                        }
                        for sx in 0..scale {
                            let px = start_x + col_idx * scale + sx;
                            if px < 0 || px >= img_w {
                                continue;
                            }
                            img.put_pixel(px as u32, py as u32, COLOR_MARKER_TEXT);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_png(w: u32, h: u32, color: Rgba<u8>) -> Vec<u8> {
        let img = RgbaImage::from_pixel(w, h, color);
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
            .unwrap();
        bytes
    }

    #[test]
    fn burns_markers_preserving_dimensions() {
        let dims = ImageDimensions::new(800, 600).unwrap();
        let m1 = Marker::new("m1".into(), "f1".into(), 1, 0.25, 0.5, "Point 1".into()).unwrap();
        let m2 = Marker::new("m2".into(), "f1".into(), 2, 0.75, 0.8, "Point 2".into()).unwrap();

        let input = make_test_png(800, 600, Rgba([100, 100, 100, 255]));
        let burned = MarkerBurner::burn_markers(&input, &dims, &[m1, m2]).unwrap();

        let decoded = image::load_from_memory(&burned).unwrap();
        assert_eq!(decoded.width(), 800);
        assert_eq!(decoded.height(), 600);
    }

    #[test]
    fn edge_and_boundary_coordinates_clamp_safely_without_panic() {
        let dims = ImageDimensions::new(100, 100).unwrap();
        let m_corner1 =
            Marker::new("c1".into(), "f1".into(), 1, 0.0, 0.0, "Top Left".into()).unwrap();
        let m_corner2 = Marker::new(
            "c2".into(),
            "f1".into(),
            99,
            1.0,
            1.0,
            "Bottom Right".into(),
        )
        .unwrap();

        let input = make_test_png(100, 100, Rgba([200, 200, 200, 255]));
        let burned = MarkerBurner::burn_markers(&input, &dims, &[m_corner1, m_corner2]).unwrap();

        let decoded = image::load_from_memory(&burned).unwrap();
        assert_eq!(decoded.width(), 100);
        assert_eq!(decoded.height(), 100);
    }

    #[test]
    fn invalid_input_bytes_returns_validation_error() {
        let dims = ImageDimensions::new(100, 100).unwrap();
        let marker = Marker::new("m1".into(), "f1".into(), 1, 0.5, 0.5, "Valid".into()).unwrap();
        let corrupt_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x00, 0x00];

        let result = MarkerBurner::burn_markers(&corrupt_bytes, &dims, &[marker]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CoreError::Validation(_)));
    }
}
