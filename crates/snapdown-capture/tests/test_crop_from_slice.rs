//! `crop_rgba_from_slice` is how a Finding is cut out of a canvas the app no longer owns as an
//! `RgbaImage` - see `BUG-28`. It replaces an `image::imageops::crop_imm` call, so what matters is
//! that it agrees with it exactly.

use image::{Rgba, RgbaImage};
use snapdown_capture::{CaptureError, RegionCapturer};
use snapdown_core::domain::finding::Region;

/// Every pixel distinct in both axes, so a crop that is off by a row or a column cannot pass.
fn canvas(width: u32, height: u32) -> RgbaImage {
    let mut image = RgbaImage::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = Rgba([(x % 251) as u8, (y % 241) as u8, ((x ^ y) % 239) as u8, 255]);
    }
    image
}

/// The equivalence that matters: same bytes as the `crop_imm` this replaces.
#[test]
fn identical_to_the_crop_imm_it_replaces() {
    let source = canvas(200, 120);

    for region in [
        Region::new(0, 0, 200, 120),
        Region::new(0, 0, 1, 1),
        Region::new(37, 11, 64, 48),
        Region::new(199, 119, 1, 1),
        Region::new(100, 60, 100, 60),
    ] {
        let reference = image::imageops::crop_imm(
            &source,
            region.x as u32,
            region.y as u32,
            region.width,
            region.height,
        )
        .to_image();

        let cropped = RegionCapturer::crop_rgba_from_slice(
            source.as_raw(),
            source.width(),
            source.height(),
            &region,
        )
        .expect("a region inside the canvas must crop");

        assert_eq!(
            cropped.dimensions(),
            (region.width, region.height),
            "{region:?} produced the wrong dimensions"
        );
        assert_eq!(
            cropped.as_raw(),
            reference.as_raw(),
            "{region:?} did not match imageops::crop_imm"
        );
    }
}

/// A region that would read past the canvas is refused rather than clamped or panicked on. The
/// caller clamps first; this is the backstop, and in this process a panic takes the tray, the
/// hotkeys and the Editor with it (`AD-11`).
#[test]
fn a_region_outside_the_canvas_is_an_error_not_a_panic() {
    let source = canvas(64, 48);

    for region in [
        Region::new(-1, 0, 8, 8),
        Region::new(0, -1, 8, 8),
        Region::new(60, 0, 8, 8),
        Region::new(0, 44, 8, 8),
        Region::new(0, 0, 65, 48),
        Region::new(0, 0, 8, 0),
        Region::new(0, 0, 0, 8),
    ] {
        let result = RegionCapturer::crop_rgba_from_slice(
            source.as_raw(),
            source.width(),
            source.height(),
            &region,
        );
        assert!(
            matches!(result, Err(CaptureError::RegionExceedsMonitorBounds { .. })),
            "{region:?} should have been refused, got {:?}",
            result.map(|i| i.dimensions())
        );
    }
}

/// A buffer whose length disagrees with the dimensions it claims is refused. Getting this wrong
/// would read a Finding out of the wrong pixels, silently.
#[test]
fn a_buffer_that_does_not_match_its_dimensions_is_an_error() {
    let source = canvas(64, 48);
    let region = Region::new(0, 0, 8, 8);

    assert!(matches!(
        RegionCapturer::crop_rgba_from_slice(source.as_raw(), 64, 47, &region),
        Err(CaptureError::InvalidRegion(_))
    ));
    assert!(matches!(
        RegionCapturer::crop_rgba_from_slice(&source.as_raw()[..100], 64, 48, &region),
        Err(CaptureError::InvalidRegion(_))
    ));
}

/// The output must survive a real PNG round trip, pixel for pixel. A signature and a dimension are
/// what a fabricated 17-byte header passes; decoding is what it does not.
#[test]
fn the_cropped_region_survives_a_png_round_trip() {
    let source = canvas(128, 96);
    let region = Region::new(17, 23, 40, 30);

    let cropped = RegionCapturer::crop_rgba_from_slice(
        source.as_raw(),
        source.width(),
        source.height(),
        &region,
    )
    .expect("crop must succeed");

    let encoded = RegionCapturer::crop_and_encode_image(&cropped, &Region::new(0, 0, 40, 30))
        .expect("encoding must succeed");

    let decoded = image::load_from_memory(&encoded)
        .expect("the encoded bytes must decode as a real image")
        .to_rgba8();

    assert_eq!(decoded.dimensions(), (40, 30));
    assert_eq!(
        decoded.as_raw(),
        cropped.as_raw(),
        "the decoded pixels must be the cropped pixels, not merely the right size"
    );
    // And they must be the canvas's own pixels at that offset, not some other region's.
    assert_eq!(decoded.get_pixel(0, 0), source.get_pixel(17, 23));
    assert_eq!(decoded.get_pixel(39, 29), source.get_pixel(56, 52));
}
