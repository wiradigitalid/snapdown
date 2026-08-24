use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::Marker;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_store::image::MarkerBurner;

const BADGE_OUTER_RADIUS: i32 = 16;

fn make_gradient_image(w: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = (((x + y) * 128) / (w + h).max(1)) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn encode_png(img: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .expect("Failed to encode test PNG fixture");
    bytes
}

/// Covers:
/// - AD-4 (ARCHITECTURE-SPINE.md:106-109): "A Bundle's image is a copy of the Finding's image
///   with Markers drawn on it, at the same dimensions. No later stage ... may re-encode or re-scale
///   a stored image."
/// - BR-8: Bundle items preserve the source finding's dimensions without re-scaling.
/// - UC-9 alt-1 & SCN-04: Findings with no drawable markers yield an exact byte-identical copy.
/// - When drawable markers are present: copy preserves exact dimensions, leaves off-badge pixels
///   byte-identical to the source, and modifies only badge-footprint pixels.
///
/// Backed by: AD-4, BR-8, UC-9 alt-1, SCN-04.
///
/// Explicitly does NOT cover:
/// - The file on disk at `BundleItem.image_path` (bound by BUG-19; file-level assertions are deferred
///   to W8-S6).
#[test]
fn a_bundle_copies_the_same_bytes_as_the_finding_it_came_from() {
    let width = 400;
    let height = 300;
    let dims = ImageDimensions::new(width, height).unwrap();
    let source_img = make_gradient_image(width, height);
    let mut source_bytes = encode_png(&source_img);
    // Add trailing byte tag to ensure that byte-identity truly preserves exact source payload without re-encoding
    source_bytes.extend_from_slice(b"AD-4-BYTE-IDENTITY-PRESERVATION");

    // Fixture proof obligation: prove fixture decodes before use
    let verified_source_decode = image::load_from_memory(&source_bytes)
        .expect("Source fixture must be a valid decodable PNG");
    assert_eq!(verified_source_decode.width(), width);
    assert_eq!(verified_source_decode.height(), height);

    // ------------------------------------------------------------------------
    // Half 1: No drawable markers -> Exact byte-identity (AD-4, UC-9 alt-1, SCN-04)
    // ------------------------------------------------------------------------
    let zero_marker_copy = MarkerBurner::burn_markers(&source_bytes, &dims, &[])
        .expect("burn_markers with zero markers must succeed");
    assert_eq!(
        zero_marker_copy, source_bytes,
        "Bundle copy of finding with zero markers must be byte-identical to source"
    );

    let empty_marker = Marker::new(
        "m-empty".into(),
        "f1".into(),
        1,
        0.5,
        0.5,
        "   \t\n ".into(),
    )
    .unwrap();
    let scn04_copy = MarkerBurner::burn_markers(&source_bytes, &dims, &[empty_marker])
        .expect("burn_markers with SCN-04 whitespace-only marker must succeed");
    assert_eq!(
        scn04_copy, source_bytes,
        "Bundle copy with whitespace-only comment marker must return source bytes unchanged"
    );

    // ------------------------------------------------------------------------
    // Half 2: With drawable markers -> Dimension & pixel preservation (AD-4, BR-8)
    // ------------------------------------------------------------------------
    let m1 = Marker::new("m1".into(), "f1".into(), 1, 0.25, 0.25, "Defect 1".into()).unwrap();
    let m2 = Marker::new("m2".into(), "f1".into(), 2, 0.75, 0.75, "Defect 2".into()).unwrap();

    let cx1 = (0.25 * width as f64).round() as i32; // 100
    let cy1 = (0.25 * height as f64).round() as i32; // 75
    let cx2 = (0.75 * width as f64).round() as i32; // 300
    let cy2 = (0.75 * height as f64).round() as i32; // 225

    let burned_bytes = MarkerBurner::burn_markers(&source_bytes, &dims, &[m1, m2])
        .expect("burn_markers with active markers must succeed");

    let decoded_burned = image::load_from_memory(&burned_bytes)
        .expect("Burned image must decode cleanly as PNG")
        .to_rgba8();

    // 1. Dimensions equal source exactly
    assert_eq!(decoded_burned.width(), width);
    assert_eq!(decoded_burned.height(), height);

    // 2. Pixels away from every badge are byte-identical to the source's pixels
    let r_sq = BADGE_OUTER_RADIUS * BADGE_OUTER_RADIUS;
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let dist1_sq = (x - cx1) * (x - cx1) + (y - cy1) * (y - cy1);
            let dist2_sq = (x - cx2) * (x - cx2) + (y - cy2) * (y - cy2);

            if dist1_sq > r_sq && dist2_sq > r_sq {
                let src_px = source_img.get_pixel(x as u32, y as u32);
                let burned_px = decoded_burned.get_pixel(x as u32, y as u32);
                assert_eq!(
                    src_px, burned_px,
                    "Off-badge pixel at ({x}, {y}) must be identical to source pixel"
                );
            }
        }
    }

    // 3. Pixels at badge centers differ from the source
    assert_ne!(
        source_img.get_pixel(cx1 as u32, cy1 as u32),
        decoded_burned.get_pixel(cx1 as u32, cy1 as u32),
        "Badge 1 center pixel must differ from source"
    );
    assert_ne!(
        source_img.get_pixel(cx2 as u32, cy2 as u32),
        decoded_burned.get_pixel(cx2 as u32, cy2 as u32),
        "Badge 2 center pixel must differ from source"
    );
}

/// Covers:
/// - AD-4: A Bundle's image copy faithfully tracks its source finding's underlying pixels.
///   Changing one pixel in the source propagates to the copy without being fabricated.
///
/// Backed by: AD-4, BR-8.
///
/// Explicitly does NOT cover:
/// - The file on disk at `BundleItem.image_path` (bound by BUG-19; file-level assertions are deferred
///   to W8-S6).
#[test]
fn changing_one_pixel_of_a_source_image_changes_the_bundle_copy() {
    let width = 400;
    let height = 300;
    let dims = ImageDimensions::new(width, height).unwrap();

    let img_a = make_gradient_image(width, height);

    let m1 = Marker::new("m1".into(), "f1".into(), 1, 0.25, 0.25, "Point 1".into()).unwrap();
    let m2 = Marker::new("m2".into(), "f1".into(), 2, 0.75, 0.75, "Point 2".into()).unwrap();

    let cx1 = (0.25 * width as f64).round() as i32; // 100
    let cy1 = (0.25 * height as f64).round() as i32; // 75
    let cx2 = (0.75 * width as f64).round() as i32; // 300
    let cy2 = (0.75 * height as f64).round() as i32; // 225

    // Choose target pixel (10, 10) and prove it lies outside every badge footprint
    let px: u32 = 10;
    let py: u32 = 10;
    let dist1_sq = (px as i32 - cx1) * (px as i32 - cx1) + (py as i32 - cy1) * (py as i32 - cy1);
    let dist2_sq = (px as i32 - cx2) * (px as i32 - cx2) + (py as i32 - cy2) * (py as i32 - cy2);
    let r_sq = BADGE_OUTER_RADIUS * BADGE_OUTER_RADIUS;

    assert!(
        dist1_sq > r_sq,
        "Target pixel ({px}, {py}) must be strictly outside badge 1"
    );
    assert!(
        dist2_sq > r_sq,
        "Target pixel ({px}, {py}) must be strictly outside badge 2"
    );

    // Build Image B by modifying exactly that one pixel
    let mut img_b = img_a.clone();
    let orig_pixel = *img_a.get_pixel(px, py);
    let mutated_pixel = Rgba([
        orig_pixel[0] ^ 0xFF,
        orig_pixel[1] ^ 0xFF,
        orig_pixel[2] ^ 0xFF,
        255,
    ]);
    img_b.put_pixel(px, py, mutated_pixel);

    let bytes_a = encode_png(&img_a);
    let bytes_b = encode_png(&img_b);

    // Fixture proof obligation: prove both source fixtures decode and differ
    let dec_a = image::load_from_memory(&bytes_a).expect("Fixture A must decode");
    let dec_b = image::load_from_memory(&bytes_b).expect("Fixture B must decode");
    assert_eq!(dec_a.width(), width);
    assert_eq!(dec_a.height(), height);
    assert_eq!(dec_b.width(), width);
    assert_eq!(dec_b.height(), height);
    assert_ne!(
        bytes_a, bytes_b,
        "Fixture A and Fixture B bytes must differ"
    );

    // Burn markers into both sources
    let burned_a = MarkerBurner::burn_markers(&bytes_a, &dims, &[m1.clone(), m2.clone()])
        .expect("burn_markers on source A must succeed");
    let burned_b = MarkerBurner::burn_markers(&bytes_b, &dims, &[m1, m2])
        .expect("burn_markers on source B must succeed");

    assert_ne!(
        burned_a, burned_b,
        "Burned copy A must differ from Burned copy B when one source pixel changed"
    );

    // Decode burned copies and verify the difference is preserved at that specific pixel
    let dec_burned_a = image::load_from_memory(&burned_a)
        .expect("Burned A must decode")
        .to_rgba8();
    let dec_burned_b = image::load_from_memory(&burned_b)
        .expect("Burned B must decode")
        .to_rgba8();

    assert_eq!(
        *dec_burned_a.get_pixel(px, py),
        orig_pixel,
        "Burned A must retain original pixel at ({px}, {py})"
    );
    assert_eq!(
        *dec_burned_b.get_pixel(px, py),
        mutated_pixel,
        "Burned B must retain mutated pixel at ({px}, {py})"
    );
    assert_ne!(
        dec_burned_a.get_pixel(px, py),
        dec_burned_b.get_pixel(px, py),
        "Decoded burned copies must differ at the mutated pixel ({px}, {py})"
    );
}
