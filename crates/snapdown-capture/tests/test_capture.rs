use image::{ImageFormat, Rgba, RgbaImage};
use snapdown_capture::{CaptureError, RegionCapturer};
use snapdown_core::domain::finding::Region;
use std::collections::HashSet;

fn create_synthetic_test_frame(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let r = ((x * 255) / width.max(1)) as u8;
            let g = ((y * 255) / height.max(1)) as u8;
            let b = (((x + y) * 128) / (width + height).max(1)) as u8;
            let a = 255u8;
            img.put_pixel(x, y, Rgba([r, g, b, a]));
        }
    }
    img
}

#[test]
fn a_captured_region_decodes_as_a_real_image() {
    let source_frame = create_synthetic_test_frame(1920, 1080);
    let region = Region::new(100, 150, 400, 300);

    let encoded_bytes = RegionCapturer::crop_and_encode_image(&source_frame, &region)
        .expect("crop_and_encode_image should succeed on valid region");

    // Must be at least standard PNG header + chunks (much larger than 17-byte fake header)
    assert!(
        encoded_bytes.len() > 17,
        "Encoded PNG bytes must be a real PNG image, not a 17-byte placeholder"
    );

    // Verify format is PNG
    let format = image::guess_format(&encoded_bytes).expect("Should identify image format");
    assert_eq!(format, ImageFormat::Png);

    // Decode image from memory
    let decoded = image::load_from_memory(&encoded_bytes)
        .expect("Encoded bytes must decode into a valid DynamicImage");

    assert_eq!(decoded.width(), 400);
    assert_eq!(decoded.height(), 300);
    assert_eq!(decoded.color(), image::ColorType::Rgba8);
}

#[test]
fn a_captured_region_has_the_dimensions_that_were_requested() {
    let source_frame = create_synthetic_test_frame(1920, 1080);
    let requested_w = 320;
    let requested_h = 240;
    let region = Region::new(50, 80, requested_w, requested_h);

    let encoded_bytes = RegionCapturer::crop_and_encode_image(&source_frame, &region)
        .expect("Cropping within source frame should succeed");

    let decoded = image::load_from_memory(&encoded_bytes).expect("Should decode as valid image");

    assert_eq!(decoded.width(), requested_w);
    assert_eq!(decoded.height(), requested_h);

    // Also test another distinct dimension pair
    let region_b = Region::new(200, 300, 128, 64);
    let encoded_b = RegionCapturer::crop_and_encode_image(&source_frame, &region_b)
        .expect("Cropping second region should succeed");
    let decoded_b = image::load_from_memory(&encoded_b).expect("Should decode second image");
    assert_eq!(decoded_b.width(), 128);
    assert_eq!(decoded_b.height(), 64);
}

#[test]
fn a_captured_image_is_not_uniformly_one_colour() {
    // Generate a patterned synthetic test frame with distinct gradients and colors
    let source_frame = create_synthetic_test_frame(800, 600);
    let region = Region::new(20, 30, 200, 150);

    let encoded_bytes = RegionCapturer::crop_and_encode_image(&source_frame, &region)
        .expect("Cropping should succeed");

    let decoded = image::load_from_memory(&encoded_bytes).expect("Must decode image");
    let rgba = decoded.to_rgba8();

    let mut distinct_pixels: HashSet<[u8; 4]> = HashSet::new();
    for pixel in rgba.pixels() {
        distinct_pixels.insert(pixel.0);
    }

    // A real multi-color region must contain many distinct pixel colors, not a uniform block
    assert!(
        distinct_pixels.len() > 10,
        "Captured image must not be uniformly one colour; found only {} distinct pixel values",
        distinct_pixels.len()
    );
}

#[test]
fn a_region_larger_than_the_monitor_is_refused_not_clamped_silently() {
    let source_frame = create_synthetic_test_frame(1920, 1080);

    // Region width exceeds source width
    let oversized_w = Region::new(0, 0, 2560, 1080);
    let err_w = RegionCapturer::crop_and_encode_image(&source_frame, &oversized_w)
        .expect_err("Should refuse region wider than monitor");
    match err_w {
        CaptureError::RegionExceedsMonitorBounds { requested, monitor } => {
            assert_eq!(requested, "0,0,2560,1080");
            assert_eq!(monitor, "1920x1080");
        }
        other => panic!("Expected RegionExceedsMonitorBounds, got {other:?}"),
    }

    // Region offset + dimensions exceed bounds
    let overflowing_rect = Region::new(1800, 1000, 300, 200);
    let err_overflow = RegionCapturer::crop_and_encode_image(&source_frame, &overflowing_rect)
        .expect_err("Should refuse overflowing region");
    match err_overflow {
        CaptureError::RegionExceedsMonitorBounds { requested, monitor } => {
            assert_eq!(requested, "1800,1000,300,200");
            assert_eq!(monitor, "1920x1080");
        }
        other => panic!("Expected RegionExceedsMonitorBounds, got {other:?}"),
    }

    // Negative coordinates
    let negative_rect = Region::new(-10, 0, 100, 100);
    let err_neg = RegionCapturer::crop_and_encode_image(&source_frame, &negative_rect)
        .expect_err("Should refuse negative coordinate");
    assert!(matches!(
        err_neg,
        CaptureError::RegionExceedsMonitorBounds { .. }
    ));
}

#[test]
fn a_region_smaller_than_eight_pixels_is_refused() {
    let source_frame = create_synthetic_test_frame(800, 600);

    let too_narrow = Region::new(10, 10, 4, 100);
    let err_narrow = RegionCapturer::crop_and_encode_image(&source_frame, &too_narrow)
        .expect_err("Should refuse width < 8");
    assert!(matches!(err_narrow, CaptureError::InvalidRegion(_)));

    let too_short = Region::new(10, 10, 100, 7);
    let err_short = RegionCapturer::crop_and_encode_image(&source_frame, &too_short)
        .expect_err("Should refuse height < 8");
    assert!(matches!(err_short, CaptureError::InvalidRegion(_)));
}

#[test]
fn capture_region_on_system_handles_headless_gracefully_without_panicking() {
    let region = Region::new(0, 0, 100, 100);
    let result = RegionCapturer::capture_region(&region, None);

    // On headless CI / VMs without displays, result is Err(NoDisplayFound).
    // On systems with displays, it returns Ok(bytes) or Err(NoDisplayFound/CaptureFailed).
    // In ALL cases, it must NOT panic and if successful, must return valid PNG bytes.
    match result {
        Ok(bytes) => {
            assert!(bytes.len() > 17);
            let decoded = image::load_from_memory(&bytes)
                .expect("Captured bytes from live monitor must decode as PNG");
            assert_eq!(decoded.width(), 100);
            assert_eq!(decoded.height(), 100);
        }
        Err(CaptureError::NoDisplayFound) => {
            // Expected in headless environment
        }
        Err(CaptureError::CaptureFailed(msg)) => {
            // Display/DXGI capture unavailable or permission denied in VM
            assert!(!msg.is_empty());
        }
        Err(other) => {
            panic!("Unexpected error variant: {other:?}");
        }
    }
}
