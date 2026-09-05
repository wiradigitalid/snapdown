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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AnnotationShape {
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        stroke_color: Option<String>,
        stroke_width: Option<f64>,
    },
    Arrow {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
        color: Option<String>,
        stroke_width: Option<f64>,
    },
    Callout {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        tail_x: f64,
        tail_y: f64,
        text: String,
        font_size: Option<f64>,
        font_family: Option<String>,
        bg_color: Option<String>,
        text_color: Option<String>,
        /// `start`, `center` or `end`. The Properties panel's third control, next to family and
        /// size - the owner asked for it by name ("justify page") and `FR-32` does not mention it.
        ///
        /// `#[serde(default)]` because this variant was already serializable before the field
        /// existed: without it, a row written by an earlier build fails to parse and
        /// `read_annotations` reports the whole Finding as corrupt.
        #[serde(default)]
        text_align: Option<String>,
    },
    Blur {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        blur_radius: Option<f64>,
    },
    Text {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        text: String,
        font_size: Option<f64>,
        font_family: Option<String>,
        text_color: Option<String>,
        #[serde(default)]
        text_align: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VisualAnnotation {
    pub id: String,
    pub finding_id: String,
    pub data: AnnotationShape,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindingDetail {
    pub finding: Finding,
    pub note: Note,
    pub markers: Vec<Marker>,
    #[serde(default)]
    pub visual_annotations: Vec<VisualAnnotation>,
}

/// The pixel rectangle a crop selected out of a Finding's OLD image - the origin and size that
/// disappeared for anything now remapped into the NEW, cropped image's own coordinate space.
/// `BUG-107`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Remaps a Finding's Markers and VisualAnnotations from its OLD image's coordinate space into a
/// NEW one produced by cropping it - `BUG-107`.
///
/// Every coordinate a `Marker`/`AnnotationShape` carries is a fraction of its OWN image
/// (`Marker::validate_coordinates`, and the twin check in `snapdown-store`'s `validate_shape`), so
/// cropping the image out from under them leaves every existing fraction answering the wrong
/// question - "where on the OLD image", when the only image left is the NEW one.
///
/// The remap for a single point is: fraction -> OLD pixel space -> subtract the crop's own origin
/// -> NEW pixel space -> fraction of the NEW image. A `Marker` that lands entirely outside the new
/// bounds is reported as gone (`remap_marker` returns `None`) - it is a comment anchored to one
/// specific pixel, and the crop removed that pixel; there is no position left on the new image
/// that means the same thing, and clamping it to the nearest edge would silently misrepresent
/// where the Reviewer actually pointed. A box-shaped annotation (`Rect`, `Blur`, `Text`, and
/// `Callout`'s own box) is instead CLIPPED to whatever of it survives the crop, the same way the
/// image content underneath it was clipped - visible, not silently relocated - and dropped
/// (`None`) only when none of its area survives at all. An `Arrow` is dropped only when its own
/// bounding box - the rectangle spanning both endpoints - has no overlap with the new image
/// whatsoever; otherwise both endpoints are clamped onto the new image's edges so the line stays
/// on-canvas.
pub struct CropRemap {
    old_width: f64,
    old_height: f64,
    crop: CropRect,
    new_width: f64,
    new_height: f64,
}

impl CropRemap {
    pub fn new(
        old_width: u32,
        old_height: u32,
        crop: CropRect,
        new_width: u32,
        new_height: u32,
    ) -> Self {
        Self {
            old_width: f64::from(old_width.max(1)),
            old_height: f64::from(old_height.max(1)),
            crop,
            new_width: f64::from(new_width.max(1)),
            new_height: f64::from(new_height.max(1)),
        }
    }

    /// OLD-image fraction -> NEW-image pixel offset. Not yet bounds-checked or normalized.
    fn to_new_pixels(&self, x: f64, y: f64) -> (f64, f64) {
        let px = x * self.old_width - f64::from(self.crop.x);
        let py = y * self.old_height - f64::from(self.crop.y);
        (px, py)
    }

    fn to_new_fraction(&self, px: f64, py: f64) -> (f64, f64) {
        (
            (px / self.new_width).clamp(0.0, 1.0),
            (py / self.new_height).clamp(0.0, 1.0),
        )
    }

    /// A single anchor point (a Marker, an Arrow endpoint, a Callout's tail): `None` when it falls
    /// outside the new image's closed bounds - the same `[0.0, 1.0]` inclusive range
    /// `Marker::validate_coordinates` accepts, mirrored in pixel space.
    fn remap_point(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let (px, py) = self.to_new_pixels(x, y);
        if px < 0.0 || py < 0.0 || px > self.new_width || py > self.new_height {
            return None;
        }
        Some(self.to_new_fraction(px, py))
    }

    /// The same anchor point, but pinned onto the nearest edge instead of dropped - for an Arrow
    /// endpoint or a Callout tail, where a point pinned to the edge still means something ("the
    /// line/tail continues past the crop"), unlike a Marker's single point which means nothing once
    /// its own pixel is gone.
    fn clamp_point(&self, x: f64, y: f64) -> (f64, f64) {
        let (px, py) = self.to_new_pixels(x, y);
        let cx = px.clamp(0.0, self.new_width);
        let cy = py.clamp(0.0, self.new_height);
        self.to_new_fraction(cx, cy)
    }

    /// A box's own rectangle, remapped and clipped to the new image. `None` when the clipped
    /// rectangle has zero or negative area - nothing of it survives the crop.
    fn remap_box_clip(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Option<(f64, f64, f64, f64)> {
        let (px0, py0) = self.to_new_pixels(x, y);
        let px1 = px0 + width * self.old_width;
        let py1 = py0 + height * self.old_height;

        let cx0 = px0.max(0.0);
        let cy0 = py0.max(0.0);
        let cx1 = px1.min(self.new_width);
        let cy1 = py1.min(self.new_height);

        if cx1 <= cx0 || cy1 <= cy0 {
            return None;
        }

        let (fx0, fy0) = self.to_new_fraction(cx0, cy0);
        let fw = ((cx1 - cx0) / self.new_width).clamp(0.0, 1.0);
        let fh = ((cy1 - cy0) / self.new_height).clamp(0.0, 1.0);
        Some((fx0, fy0, fw, fh))
    }

    /// Whether a rectangle overlaps the new image's closed bounds at all - tolerant of zero width
    /// or zero height, which an axis-aligned Arrow's own bounding box always has. Used only to
    /// decide whether an Arrow survives; `remap_box_clip` above is what a real box shape clips to.
    fn bbox_overlaps_new_bounds(&self, x: f64, y: f64, width: f64, height: f64) -> bool {
        let (px0, py0) = self.to_new_pixels(x, y);
        let px1 = px0 + width * self.old_width;
        let py1 = py0 + height * self.old_height;

        let cx0 = px0.max(0.0);
        let cy0 = py0.max(0.0);
        let cx1 = px1.min(self.new_width);
        let cy1 = py1.min(self.new_height);

        cx1 >= cx0 && cy1 >= cy0
    }

    /// `None` when the Marker's own pixel is no longer inside the new image - it is dropped, not
    /// clamped (see the type-level doc comment for why).
    pub fn remap_marker(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        self.remap_point(x, y)
    }

    /// `None` when nothing of the shape survives the crop.
    pub fn remap_annotation(&self, shape: &AnnotationShape) -> Option<AnnotationShape> {
        match shape {
            AnnotationShape::Rect {
                x,
                y,
                width,
                height,
                stroke_color,
                stroke_width,
            } => {
                let (x, y, width, height) = self.remap_box_clip(*x, *y, *width, *height)?;
                Some(AnnotationShape::Rect {
                    x,
                    y,
                    width,
                    height,
                    stroke_color: stroke_color.clone(),
                    stroke_width: *stroke_width,
                })
            }
            AnnotationShape::Blur {
                x,
                y,
                width,
                height,
                blur_radius,
            } => {
                let (x, y, width, height) = self.remap_box_clip(*x, *y, *width, *height)?;
                Some(AnnotationShape::Blur {
                    x,
                    y,
                    width,
                    height,
                    blur_radius: *blur_radius,
                })
            }
            AnnotationShape::Text {
                x,
                y,
                width,
                height,
                text,
                font_size,
                font_family,
                text_color,
                text_align,
            } => {
                let (x, y, width, height) = self.remap_box_clip(*x, *y, *width, *height)?;
                Some(AnnotationShape::Text {
                    x,
                    y,
                    width,
                    height,
                    text: text.clone(),
                    font_size: *font_size,
                    font_family: font_family.clone(),
                    text_color: text_color.clone(),
                    text_align: text_align.clone(),
                })
            }
            AnnotationShape::Callout {
                x,
                y,
                width,
                height,
                tail_x,
                tail_y,
                text,
                font_size,
                font_family,
                bg_color,
                text_color,
                text_align,
            } => {
                let (x, y, width, height) = self.remap_box_clip(*x, *y, *width, *height)?;
                let (tail_x, tail_y) = self.clamp_point(*tail_x, *tail_y);
                Some(AnnotationShape::Callout {
                    x,
                    y,
                    width,
                    height,
                    tail_x,
                    tail_y,
                    text: text.clone(),
                    font_size: *font_size,
                    font_family: font_family.clone(),
                    bg_color: bg_color.clone(),
                    text_color: text_color.clone(),
                    text_align: text_align.clone(),
                })
            }
            AnnotationShape::Arrow {
                start_x,
                start_y,
                end_x,
                end_y,
                color,
                stroke_width,
            } => {
                let min_x = start_x.min(*end_x);
                let min_y = start_y.min(*end_y);
                let bbox_w = start_x.max(*end_x) - min_x;
                let bbox_h = start_y.max(*end_y) - min_y;
                if !self.bbox_overlaps_new_bounds(min_x, min_y, bbox_w, bbox_h) {
                    return None;
                }

                let (start_x, start_y) = self.clamp_point(*start_x, *start_y);
                let (end_x, end_y) = self.clamp_point(*end_x, *end_y);
                Some(AnnotationShape::Arrow {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    color: color.clone(),
                    stroke_width: *stroke_width,
                })
            }
        }
    }
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

    /// `BUG-107`: a worked example, not just "still exists". A 1000x1000 image cropped to the
    /// rectangle (200, 100, 400, 500) - so the new image is 400x500 - and a Marker that survives
    /// the crop must land at the mathematically exact new fraction, not merely a nearby one.
    fn crop_1000_square_to_400x500_at_200_100() -> CropRemap {
        CropRemap::new(
            1000,
            1000,
            CropRect {
                x: 200,
                y: 100,
                width: 400,
                height: 500,
            },
            400,
            500,
        )
    }

    #[test]
    fn a_marker_inside_the_crop_lands_at_the_exact_new_fraction() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel (500, 500) -> new pixel (300, 400) -> new fraction (0.75, 0.8).
        let (nx, ny) = remap
            .remap_marker(0.5, 0.5)
            .expect("a marker well inside the crop must survive");
        assert!((nx - 0.75).abs() < 1e-9, "got x={nx}");
        assert!((ny - 0.8).abs() < 1e-9, "got y={ny}");
    }

    #[test]
    fn a_marker_on_the_crop_s_far_edge_is_kept_at_fraction_one() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel (600, 600) is exactly the crop's bottom-right corner (200+400, 100+500).
        let (nx, ny) = remap
            .remap_marker(0.6, 0.6)
            .expect("a marker exactly on the crop's far edge must be kept, not dropped");
        assert!((nx - 1.0).abs() < 1e-9, "got x={nx}");
        assert!((ny - 1.0).abs() < 1e-9, "got y={ny}");
    }

    #[test]
    fn a_marker_outside_the_crop_is_dropped_not_clamped() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel (100, 100) is above and to the left of the crop rectangle entirely.
        assert_eq!(
            remap.remap_marker(0.1, 0.1),
            None,
            "a marker anchored to a pixel the crop removed must be dropped, not silently \
             relocated to the crop's edge"
        );

        // Just one pixel past the crop's far edge.
        assert_eq!(
            remap.remap_marker(0.601, 0.6),
            None,
            "a marker one pixel past the crop's far edge must also be dropped"
        );
    }

    #[test]
    fn a_rect_annotation_partially_inside_the_crop_is_clipped_not_dropped() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel box (50, 50) to (250, 250) - straddles the crop's top-left corner (200, 100).
        let shape = AnnotationShape::Rect {
            x: 0.05,
            y: 0.05,
            width: 0.2,
            height: 0.2,
            stroke_color: None,
            stroke_width: None,
        };

        let remapped = remap
            .remap_annotation(&shape)
            .expect("a rect that partially overlaps the new image must survive, clipped");

        match remapped {
            AnnotationShape::Rect {
                x,
                y,
                width,
                height,
                ..
            } => {
                // New pixel box clips to (0, 0)-(50, 150): fraction (0, 0, 50/400, 150/500).
                assert!((x - 0.0).abs() < 1e-9, "got x={x}");
                assert!((y - 0.0).abs() < 1e-9, "got y={y}");
                assert!((width - 0.125).abs() < 1e-9, "got width={width}");
                assert!((height - 0.3).abs() < 1e-9, "got height={height}");
            }
            other => panic!("expected a Rect back, got {other:?}"),
        }
    }

    #[test]
    fn a_rect_annotation_entirely_outside_the_crop_is_dropped() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel box (0, 0)-(100, 100): entirely above and to the left of the crop rectangle.
        let shape = AnnotationShape::Rect {
            x: 0.0,
            y: 0.0,
            width: 0.1,
            height: 0.1,
            stroke_color: None,
            stroke_width: None,
        };

        assert_eq!(
            remap.remap_annotation(&shape),
            None,
            "an annotation with no surviving area must be dropped"
        );
    }

    #[test]
    fn an_arrow_straddling_the_crop_edge_is_clamped_not_dropped() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        // Old pixel start (150, 150) is just outside the crop on the left; old pixel end
        // (350, 350) is well inside it.
        let shape = AnnotationShape::Arrow {
            start_x: 0.15,
            start_y: 0.15,
            end_x: 0.35,
            end_y: 0.35,
            color: None,
            stroke_width: None,
        };

        let remapped = remap
            .remap_annotation(&shape)
            .expect("an arrow whose bounding box overlaps the new image must survive");

        match remapped {
            AnnotationShape::Arrow {
                start_x,
                start_y,
                end_x,
                end_y,
                ..
            } => {
                // start clamps to the new image's left edge: new pixel (-50, 50) -> (0, 50) -> (0.0, 0.1).
                assert!((start_x - 0.0).abs() < 1e-9, "got start_x={start_x}");
                assert!((start_y - 0.1).abs() < 1e-9, "got start_y={start_y}");
                // end is already inside: new pixel (150, 250) -> fraction (0.375, 0.5).
                assert!((end_x - 0.375).abs() < 1e-9, "got end_x={end_x}");
                assert!((end_y - 0.5).abs() < 1e-9, "got end_y={end_y}");
            }
            other => panic!("expected an Arrow back, got {other:?}"),
        }
    }

    #[test]
    fn an_arrow_entirely_outside_the_crop_is_dropped() {
        let remap = crop_1000_square_to_400x500_at_200_100();

        let shape = AnnotationShape::Arrow {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.05,
            end_y: 0.05,
            color: None,
            stroke_width: None,
        };

        assert_eq!(
            remap.remap_annotation(&shape),
            None,
            "an arrow whose bounding box never touches the new image must be dropped"
        );
    }
}
