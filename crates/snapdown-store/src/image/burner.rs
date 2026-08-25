use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::{AnnotationShape, Marker, VisualAnnotation};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::error::CoreError;

const COLOR_MARKER_FILL: Rgba<u8> = Rgba([220, 38, 38, 255]); // #dc2626 solid red
const COLOR_MARKER_TEXT: Rgba<u8> = Rgba([255, 255, 255, 255]); // #ffffff solid white

const BADGE_RADIUS: i32 = 14;

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
    /// Burns numbered circular badges and visual annotations directly into image bytes at specified normalized coordinates.
    pub fn burn_markers(
        input_bytes: &[u8],
        dimensions: &ImageDimensions,
        markers: &[Marker],
    ) -> Result<Vec<u8>, CoreError> {
        Self::burn_all(input_bytes, dimensions, markers, &[])
    }

    /// Burns both numbered markers and rich visual annotations (Shapes, Blur, Arrow, Callout, Text) into image bytes.
    pub fn burn_all(
        input_bytes: &[u8],
        dimensions: &ImageDimensions,
        markers: &[Marker],
        annotations: &[VisualAnnotation],
    ) -> Result<Vec<u8>, CoreError> {
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

        let active_markers: Vec<&Marker> = markers
            .iter()
            .filter(|m| !m.comment.trim().is_empty())
            .collect();

        if active_markers.is_empty() && annotations.is_empty() {
            return Ok(input_bytes.to_vec());
        }

        let mut image_rgba: RgbaImage = decoded.to_rgba8();

        // 1. Burn Blur Redaction Layers first
        for ann in annotations {
            if let AnnotationShape::Blur {
                x,
                y,
                width,
                height,
                blur_radius,
            } = &ann.data
            {
                let bx = (x * dimensions.width as f64).round() as i32;
                let by = (y * dimensions.height as f64).round() as i32;
                let bw = (width * dimensions.width as f64).round() as i32;
                let bh = (height * dimensions.height as f64).round() as i32;
                let radius = blur_radius.unwrap_or(8.0) as i32;
                Self::apply_box_blur(&mut image_rgba, bx, by, bw, bh, radius);
            }
        }

        // 2. Burn Vector Shapes & Arrows
        for ann in annotations {
            match &ann.data {
                AnnotationShape::Rect {
                    x,
                    y,
                    width,
                    height,
                    stroke_width,
                    ..
                } => {
                    let rx = (x * dimensions.width as f64).round() as i32;
                    let ry = (y * dimensions.height as f64).round() as i32;
                    let rw = (width * dimensions.width as f64).round() as i32;
                    let rh = (height * dimensions.height as f64).round() as i32;
                    let sw = stroke_width.unwrap_or(3.0) as i32;
                    Self::draw_rect_outline(&mut image_rgba, rx, ry, rw, rh, sw, COLOR_MARKER_FILL);
                }
                AnnotationShape::Arrow {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    stroke_width,
                    ..
                } => {
                    let x0 = (start_x * dimensions.width as f64).round() as i32;
                    let y0 = (start_y * dimensions.height as f64).round() as i32;
                    let x1 = (end_x * dimensions.width as f64).round() as i32;
                    let y1 = (end_y * dimensions.height as f64).round() as i32;
                    let sw = stroke_width.unwrap_or(4.0) as i32;
                    Self::draw_arrow(&mut image_rgba, x0, y0, x1, y1, sw, COLOR_MARKER_FILL);
                }
                AnnotationShape::Callout {
                    x,
                    y,
                    width,
                    height,
                    tail_x,
                    tail_y,
                    ..
                } => {
                    let cx = (x * dimensions.width as f64).round() as i32;
                    let cy = (y * dimensions.height as f64).round() as i32;
                    let cw = (width * dimensions.width as f64).round() as i32;
                    let ch = (height * dimensions.height as f64).round() as i32;
                    let tx = (tail_x * dimensions.width as f64).round() as i32;
                    let ty = (tail_y * dimensions.height as f64).round() as i32;
                    let rect_box = [cx, cy, cw, ch];
                    let tail = [tx, ty];
                    Self::draw_callout_box(&mut image_rgba, rect_box, tail, COLOR_MARKER_FILL);
                }
                _ => {}
            }
        }

        // 3. Burn Numbered Markers on top
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

    fn apply_box_blur(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, radius: i32) {
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let x0 = x.clamp(0, img_w);
        let y0 = y.clamp(0, img_h);
        let x1 = (x + w).clamp(0, img_w);
        let y1 = (y + h).clamp(0, img_h);

        if x1 <= x0 || y1 <= y0 {
            return;
        }

        let block_size = radius.max(4);

        for by in (y0..y1).step_by(block_size as usize) {
            for bx in (x0..x1).step_by(block_size as usize) {
                let bw_actual = (x1 - bx).min(block_size);
                let bh_actual = (y1 - by).min(block_size);

                let mut r_sum: u32 = 0;
                let mut g_sum: u32 = 0;
                let mut b_sum: u32 = 0;
                let mut a_sum: u32 = 0;
                let mut count: u32 = 0;

                for py in by..(by + bh_actual) {
                    for px in bx..(bx + bw_actual) {
                        let p = img.get_pixel(px as u32, py as u32);
                        r_sum += p[0] as u32;
                        g_sum += p[1] as u32;
                        b_sum += p[2] as u32;
                        a_sum += p[3] as u32;
                        count += 1;
                    }
                }

                if let (Some(r), Some(g), Some(b), Some(a)) = (
                    r_sum.checked_div(count),
                    g_sum.checked_div(count),
                    b_sum.checked_div(count),
                    a_sum.checked_div(count),
                ) {
                    let avg = Rgba([r as u8, g as u8, b as u8, a as u8]);
                    for py in by..(by + bh_actual) {
                        for px in bx..(bx + bw_actual) {
                            img.put_pixel(px as u32, py as u32, avg);
                        }
                    }
                }
            }
        }
    }

    fn draw_rect_outline(
        img: &mut RgbaImage,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        stroke: i32,
        color: Rgba<u8>,
    ) {
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let x0 = x.clamp(0, img_w);
        let y0 = y.clamp(0, img_h);
        let x1 = (x + w).clamp(0, img_w);
        let y1 = (y + h).clamp(0, img_h);

        for px in x0..x1 {
            for s in 0..stroke {
                if y0 + s < img_h {
                    img.put_pixel(px as u32, (y0 + s) as u32, color);
                }
                if y1 - 1 - s >= 0 {
                    img.put_pixel(px as u32, (y1 - 1 - s) as u32, color);
                }
            }
        }
        for py in y0..y1 {
            for s in 0..stroke {
                if x0 + s < img_w {
                    img.put_pixel((x0 + s) as u32, py as u32, color);
                }
                if x1 - 1 - s >= 0 {
                    img.put_pixel((x1 - 1 - s) as u32, py as u32, color);
                }
            }
        }
    }

    fn draw_arrow(
        img: &mut RgbaImage,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        stroke: i32,
        color: Rgba<u8>,
    ) {
        // Draw line using Bresenham
        Self::draw_thick_line(img, x0, y0, x1, y1, stroke, color);

        // Draw arrowhead at (x1, y1)
        let angle = ((y1 - y0) as f64).atan2((x1 - x0) as f64);
        let head_len = 16.0;
        let angle1 = angle + std::f64::consts::PI * 0.85;
        let angle2 = angle - std::f64::consts::PI * 0.85;

        let hx1 = x1 + (head_len * angle1.cos()).round() as i32;
        let hy1 = y1 + (head_len * angle1.sin()).round() as i32;
        let hx2 = x1 + (head_len * angle2.cos()).round() as i32;
        let hy2 = y1 + (head_len * angle2.sin()).round() as i32;

        Self::draw_thick_line(img, x1, y1, hx1, hy1, stroke, color);
        Self::draw_thick_line(img, x1, y1, hx2, hy2, stroke, color);
    }

    fn draw_thick_line(
        img: &mut RgbaImage,
        mut x0: i32,
        mut y0: i32,
        x1: i32,
        y1: i32,
        stroke: i32,
        color: Rgba<u8>,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let half_s = stroke / 2;
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        loop {
            for oy in -half_s..=half_s {
                for ox in -half_s..=half_s {
                    let px = x0 + ox;
                    let py = y0 + oy;
                    if px >= 0 && px < img_w && py >= 0 && py < img_h {
                        img.put_pixel(px as u32, py as u32, color);
                    }
                }
            }

            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_callout_box(img: &mut RgbaImage, rect: [i32; 4], tail: [i32; 2], color: Rgba<u8>) {
        let [x, y, w, h] = rect;
        let [tx, ty] = tail;
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let x0 = x.clamp(0, img_w);
        let y0 = y.clamp(0, img_h);
        let x1 = (x + w).clamp(0, img_w);
        let y1 = (y + h).clamp(0, img_h);

        // Draw solid filled callout bubble
        for py in y0..y1 {
            for px in x0..x1 {
                img.put_pixel(px as u32, py as u32, color);
            }
        }

        // Draw tail line connecting from center of bubble to target point (tx, ty)
        let cx = x0 + (x1 - x0) / 2;
        let cy = y0 + (y1 - y0) / 2;
        Self::draw_thick_line(img, cx, cy, tx, ty, 3, color);
    }

    fn draw_badge(img: &mut RgbaImage, cx: i32, cy: i32, ordinal: u32) {
        let img_w = img.width() as i32;
        let img_h = img.height() as i32;

        let r_sq = BADGE_RADIUS * BADGE_RADIUS;

        // Draw solid red circular fill without ring
        for dy in -BADGE_RADIUS..=BADGE_RADIUS {
            let py = cy + dy;
            if py < 0 || py >= img_h {
                continue;
            }
            for dx in -BADGE_RADIUS..=BADGE_RADIUS {
                let px = cx + dx;
                if px < 0 || px >= img_w {
                    continue;
                }

                let dist_sq = dx * dx + dy * dy;
                if dist_sq <= r_sq {
                    img.put_pixel(px as u32, py as u32, COLOR_MARKER_FILL);
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
    fn burns_visual_annotations_and_blur() {
        let dims = ImageDimensions::new(400, 300).unwrap();
        let input = make_test_png(400, 300, Rgba([50, 50, 50, 255]));

        let shape = VisualAnnotation {
            id: "a1".into(),
            finding_id: "f1".into(),
            data: AnnotationShape::Rect {
                x: 0.1,
                y: 0.1,
                width: 0.3,
                height: 0.3,
                stroke_color: None,
                stroke_width: Some(2.0),
            },
            created_at: "2026-08-25T00:00:00Z".into(),
        };

        let blur = VisualAnnotation {
            id: "a2".into(),
            finding_id: "f1".into(),
            data: AnnotationShape::Blur {
                x: 0.5,
                y: 0.5,
                width: 0.2,
                height: 0.2,
                blur_radius: Some(8.0),
            },
            created_at: "2026-08-25T00:00:00Z".into(),
        };

        let burned = MarkerBurner::burn_all(&input, &dims, &[], &[shape, blur]).unwrap();
        let decoded = image::load_from_memory(&burned).unwrap();
        assert_eq!(decoded.width(), 400);
        assert_eq!(decoded.height(), 300);
    }

    #[test]
    fn invalid_input_bytes_returns_validation_error() {
        let dims = ImageDimensions::new(800, 600).unwrap();
        let err = MarkerBurner::burn_markers(b"garbage", &dims, &[]).unwrap_err();
        assert!(matches!(err, CoreError::Validation(_)));
    }

    #[test]
    fn edge_and_boundary_coordinates_clamp_safely_without_panic() {
        let dims = ImageDimensions::new(100, 100).unwrap();
        let m1 = Marker::new("m1".into(), "f1".into(), 1, 0.0, 0.0, "Top Left".into()).unwrap();
        let m2 = Marker::new("m2".into(), "f1".into(), 2, 1.0, 1.0, "Bottom Right".into()).unwrap();

        let input = make_test_png(100, 100, Rgba([200, 200, 200, 255]));
        let res = MarkerBurner::burn_markers(&input, &dims, &[m1, m2]);
        assert!(res.is_ok());
    }
}
