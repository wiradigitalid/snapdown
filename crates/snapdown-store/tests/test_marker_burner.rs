use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::Marker;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_store::image::MarkerBurner;

fn make_test_png(w: u32, h: u32, color: Rgba<u8>) -> Vec<u8> {
    let img = RgbaImage::from_pixel(w, h, color);
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
        .expect("Failed to encode test PNG fixture");
    bytes
}

fn make_gradient_png(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = (x % 256) as u8;
            let g = (y % 256) as u8;
            let b = ((x + y) % 256) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
        .expect("Failed to encode gradient test PNG fixture");
    bytes
}

#[test]
fn a_burned_image_decodes_and_differs_from_its_source_in_pixels() {
    let dims = ImageDimensions::new(400, 300).unwrap();
    let bg_color = Rgba([50, 50, 50, 255]);
    let source_bytes = make_test_png(400, 300, bg_color);

    let marker = Marker::new(
        "m-1".into(),
        "f-1".into(),
        1,
        0.5,
        0.5,
        "Issue with layout".into(),
    )
    .unwrap();

    let burned_bytes = MarkerBurner::burn_markers(&source_bytes, &dims, &[marker]).unwrap();

    let source_img = image::load_from_memory(&source_bytes).unwrap().to_rgba8();
    let burned_img = image::load_from_memory(&burned_bytes).unwrap().to_rgba8();

    assert_eq!(burned_img.width(), source_img.width());
    assert_eq!(burned_img.height(), source_img.height());

    let mut differing_pixels = 0;
    for y in 0..dims.height {
        for x in 0..dims.width {
            if source_img.get_pixel(x, y) != burned_img.get_pixel(x, y) {
                differing_pixels += 1;
            }
        }
    }

    assert!(
        differing_pixels > 0,
        "Burned image must differ from source image in pixel values"
    );
}

#[test]
fn a_burned_marker_changes_pixels_at_its_own_coordinates() {
    let dims = ImageDimensions::new(500, 500).unwrap();
    let bg_color = Rgba([128, 128, 128, 255]);
    let source_bytes = make_test_png(500, 500, bg_color);

    // Marker placed exactly at center (0.5, 0.5) -> (250, 250)
    let marker = Marker::new(
        "m-center".into(),
        "f-1".into(),
        1,
        0.5,
        0.5,
        "Broken CTA".into(),
    )
    .unwrap();

    let burned_bytes = MarkerBurner::burn_markers(&source_bytes, &dims, &[marker]).unwrap();
    let burned_img = image::load_from_memory(&burned_bytes).unwrap().to_rgba8();

    // Center pixel should not be the background color anymore
    let center_pixel = burned_img.get_pixel(250, 250);
    assert_ne!(
        center_pixel, &bg_color,
        "Pixel at marker center coordinates (250, 250) must change"
    );

    // Pixels at far corners (e.g. 10, 10) must remain unchanged
    let corner_pixel = burned_img.get_pixel(10, 10);
    assert_eq!(
        corner_pixel, &bg_color,
        "Pixel far from marker (10, 10) must remain untouched background color"
    );
}

#[test]
fn a_burned_image_keeps_the_dimensions_of_its_source() {
    let dims = ImageDimensions::new(640, 480).unwrap();
    let source_bytes = make_test_png(640, 480, Rgba([200, 200, 200, 255]));

    let marker = Marker::new(
        "m-1".into(),
        "f-1".into(),
        2,
        0.2,
        0.3,
        "Alignment error".into(),
    )
    .unwrap();

    let burned_bytes = MarkerBurner::burn_markers(&source_bytes, &dims, &[marker]).unwrap();
    let burned_img = image::load_from_memory(&burned_bytes).unwrap();

    assert_eq!(burned_img.width(), 640);
    assert_eq!(burned_img.height(), 480);
}

#[test]
fn a_marker_with_no_note_line_is_drawn_as_an_ordinary_badge() {
    // SCN-04, in its own words:
    //
    //   1. "Marker 2 stays on the image, at its position, numbered 2."
    //   4. "The image shows nothing unusual. A badge is a badge."
    //
    // and: "A 'this marker has no line' ANNOTATION on the image would be a permanent artifact of a
    // temporary editing state."
    //
    // So the scenario forbids drawing anything EXTRA for such a Marker. It requires the badge.
    // This test used to be called `..._is_never_drawn_on_the_image` and asserted the opposite - one
    // word away from the scenario's own named test, `a_marker_with_no_line_is_not_annotated_on_the_
    // image`, and the whole behaviour inverted. The owner found it from the other end: Markers
    // placed on the canvas were missing from the Assemble preview, because a Marker is placed
    // before it is described.
    let dims = ImageDimensions::new(300, 300).unwrap();
    let bg_color = Rgba([80, 80, 80, 255]);
    let source_bytes = make_test_png(300, 300, bg_color);

    let empty_marker = Marker::new("m-empty".into(), "f-1".into(), 1, 0.5, 0.5, "".into()).unwrap();
    let ws_marker = Marker::new(
        "m-ws".into(),
        "f-1".into(),
        2,
        0.3,
        0.3,
        "   	
  "
        .into(),
    )
    .unwrap();

    let burned_bytes =
        MarkerBurner::burn_markers(&source_bytes, &dims, &[empty_marker, ws_marker]).unwrap();
    let burned = image::load_from_memory(&burned_bytes).unwrap().to_rgba8();

    // Marker 1 at (0.5, 0.5) -> (150, 150)
    assert_ne!(
        burned.get_pixel(150, 150),
        &bg_color,
        "a Marker with an empty note line must still be drawn: SCN-04 point 1"
    );
    // Marker 2 at (0.3, 0.3) -> (90, 90)
    assert_ne!(
        burned.get_pixel(90, 90),
        &bg_color,
        "a whitespace-only note line is still no line, and the badge is still a badge"
    );

    // And nothing extra anywhere else: the corner is untouched, so no warning glyph or overlay has
    // been added for the ragged sequence. That is the half of SCN-04 point 4 that DOES forbid
    // something.
    assert_eq!(
        burned.get_pixel(4, 4),
        &bg_color,
        "no extra annotation may be drawn for a Marker with no line - that is what SCN-04 forbids"
    );

    // Mixed: a described Marker and an undescribed one, both drawn.
    let active_marker = Marker::new(
        "m-active".into(),
        "f-1".into(),
        3,
        0.8,
        0.8,
        "Valid comment".into(),
    )
    .unwrap();
    let empty_marker2 =
        Marker::new("m-empty2".into(), "f-1".into(), 4, 0.2, 0.2, "".into()).unwrap();

    let mixed_burned =
        MarkerBurner::burn_markers(&source_bytes, &dims, &[empty_marker2, active_marker]).unwrap();
    let mixed_img = image::load_from_memory(&mixed_burned).unwrap().to_rgba8();

    assert_ne!(
        mixed_img.get_pixel(240, 240),
        &bg_color,
        "the described Marker must be drawn"
    );
    assert_ne!(
        mixed_img.get_pixel(60, 60),
        &bg_color,
        "and so must the one that has not been described yet"
    );
}

#[test]
fn burning_no_markers_returns_the_source_bytes_unchanged() {
    let dims = ImageDimensions::new(250, 250).unwrap();
    let mut source_bytes = make_gradient_png(250, 250);
    // Add custom metadata / comment chunk or dummy byte slice to ensure codec re-encode changes bytes
    source_bytes.extend_from_slice(b"AD-9-BYTE-IDENTITY-VERIFICATION");

    let burned_bytes = MarkerBurner::burn_markers(&source_bytes, &dims, &[]).unwrap();

    assert_eq!(
        burned_bytes, source_bytes,
        "Burning zero markers must return the exact source bytes unchanged"
    );
}
