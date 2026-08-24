use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{NamedBudget, QualityBudget, ResolvedPair};
use snapdown_core::error::CoreError;
use snapdown_core::ports::BlobStore;
use snapdown_store::image::ImageReducer;
use snapdown_store::vault::VaultBlobStore;
use tempfile::TempDir;

/// Programmatically generates a valid PNG byte vector with a deterministic gradient pattern.
/// No screenshot captures or real screen data are used (repository policy).
fn create_test_png(width: u32, height: u32) -> Vec<u8> {
    create_test_png_with_pattern(width, height, |x, y| {
        let r = ((x * 255) / width.max(1)) as u8;
        let g = ((y * 255) / height.max(1)) as u8;
        let b = (((x + y) * 128) / (width + height).max(1)) as u8;
        [r, g, b, 255]
    })
}

fn create_test_png_with_pattern<F>(width: u32, height: u32, pattern: F) -> Vec<u8>
where
    F: Fn(u32, u32) -> [u8; 4],
{
    let mut img = RgbaImage::new(width, height);
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let [r, g, b, a] = pattern(x, y);
        *pixel = image::Rgba([r, g, b, a]);
    }
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(img.as_raw(), width, height, ExtendedColorType::Rgba8)
        .expect("Failed to encode test PNG");
    bytes
}

#[test]
fn a_reduced_image_decodes_and_its_pixels_are_the_scaled_source() {
    let src_w = 800;
    let src_h = 600;
    let input_bytes = create_test_png(src_w, src_h);
    let orig_dims = ImageDimensions::new(src_w, src_h).unwrap();

    let resolved = ResolvedPair::new(400, 80).unwrap();
    let result = ImageReducer::reduce_image(&input_bytes, orig_dims, &resolved, false).unwrap();

    // Must decode cleanly as valid PNG
    let decoded = image::load_from_memory(&result.bytes).expect("Reduced bytes must be valid PNG");
    let decoded_rgba = decoded.to_rgba8();

    assert_eq!(decoded_rgba.width(), 400);
    assert_eq!(decoded_rgba.height(), 300);
    assert_eq!(result.dimensions.width, 400);
    assert_eq!(result.dimensions.height, 300);

    // Verify scaled source pixels match expected Lanczos3 resized pixels
    let src_image = image::load_from_memory(&input_bytes).unwrap().to_rgba8();
    let expected_resized =
        image::imageops::resize(&src_image, 400, 300, image::imageops::FilterType::Lanczos3);

    // Check pixel equality across multiple points
    for y in [0, 50, 150, 250, 299] {
        for x in [0, 50, 200, 350, 399] {
            let actual_px = decoded_rgba.get_pixel(x, y);
            let expected_px = expected_resized.get_pixel(x, y);
            assert_eq!(
                actual_px, expected_px,
                "Pixel mismatch at ({x}, {y}): actual={:?}, expected={:?}",
                actual_px, expected_px
            );
        }
    }
}

#[test]
fn a_reduced_image_honours_the_resolved_long_edge() {
    // Landscape 4K test: 3840x2160 -> max long edge 1600 -> 1600x900
    let src_w = 3840;
    let src_h = 2160;
    let input_bytes = create_test_png(src_w, src_h);
    let orig_dims = ImageDimensions::new(src_w, src_h).unwrap();

    let resolved = ResolvedPair::new(1600, 75).unwrap();
    let result = ImageReducer::reduce_image(&input_bytes, orig_dims, &resolved, false).unwrap();

    let decoded = image::load_from_memory(&result.bytes).unwrap();
    assert_eq!(decoded.width().max(decoded.height()), 1600);
    assert_eq!(decoded.width(), 1600);
    assert_eq!(decoded.height(), 900);
    assert_eq!(result.dimensions.width, 1600);
    assert_eq!(result.dimensions.height, 900);

    // Portrait test: 1200x2400 -> max long edge 1200 -> 600x1200
    let port_w = 1200;
    let port_h = 2400;
    let port_bytes = create_test_png(port_w, port_h);
    let port_dims = ImageDimensions::new(port_w, port_h).unwrap();
    let port_resolved = ResolvedPair::new(1200, 75).unwrap();
    let port_result =
        ImageReducer::reduce_image(&port_bytes, port_dims, &port_resolved, false).unwrap();

    let port_decoded = image::load_from_memory(&port_result.bytes).unwrap();
    assert_eq!(port_decoded.width().max(port_decoded.height()), 1200);
    assert_eq!(port_decoded.width(), 600);
    assert_eq!(port_decoded.height(), 1200);
}

#[test]
fn an_image_already_under_the_long_edge_is_not_upscaled() {
    // Image 400x300 with max_long_edge 1600: must remain 400x300 (BR-40)
    let src_w = 400;
    let src_h = 300;
    let input_bytes = create_test_png(src_w, src_h);
    let orig_dims = ImageDimensions::new(src_w, src_h).unwrap();

    let resolved = ResolvedPair::new(1600, 75).unwrap();
    let result = ImageReducer::reduce_image(&input_bytes, orig_dims, &resolved, false).unwrap();

    let decoded = image::load_from_memory(&result.bytes).unwrap();
    assert_eq!(decoded.width(), 400);
    assert_eq!(decoded.height(), 300);
    assert_eq!(result.dimensions.width, 400);
    assert_eq!(result.dimensions.height, 300);
}

#[test]
fn a_thumbnail_decodes_and_is_smaller_than_its_source() {
    let src_w = 1600;
    let src_h = 1200;
    let input_bytes = create_test_png(src_w, src_h);
    let orig_dims = ImageDimensions::new(src_w, src_h).unwrap();

    let resolved = ResolvedPair::new(1600, 75).unwrap();
    let result = ImageReducer::reduce_image(&input_bytes, orig_dims, &resolved, true).unwrap();

    assert!(result.thumbnail_bytes.is_some());
    assert!(result.thumbnail_dimensions.is_some());

    let thumb_bytes = result.thumbnail_bytes.unwrap();
    let thumb_dims = result.thumbnail_dimensions.unwrap();

    let decoded_thumb = image::load_from_memory(&thumb_bytes).unwrap();
    assert_eq!(decoded_thumb.width(), thumb_dims.width);
    assert_eq!(decoded_thumb.height(), thumb_dims.height);

    // Strictly smaller than original and target dimensions
    assert!(decoded_thumb.width() < src_w);
    assert!(decoded_thumb.height() < src_h);
    assert!(decoded_thumb.width().max(decoded_thumb.height()) <= 320);
    assert_eq!(decoded_thumb.width(), 320);
    assert_eq!(decoded_thumb.height(), 240);
}

#[test]
fn the_resolved_pair_arithmetic_is_unchanged_by_this_story() {
    // 1. Presets pinned constants
    let sharp = QualityBudget::new(NamedBudget::Sharp, None);
    let balanced = QualityBudget::new(NamedBudget::Balanced, None);
    let small = QualityBudget::new(NamedBudget::Small, None);

    assert_eq!(sharp.resolve(3840), ResolvedPair::new(2560, 90).unwrap());
    assert_eq!(balanced.resolve(3840), ResolvedPair::new(1600, 75).unwrap());
    assert_eq!(small.resolve(3840), ResolvedPair::new(1280, 50).unwrap());

    // 2. Auto resolution intervals
    let auto = QualityBudget::new(NamedBudget::Auto, None);
    assert_eq!(auto.resolve(400), ResolvedPair::new(1280, 92).unwrap()); // <= 800
    assert_eq!(auto.resolve(1000), ResolvedPair::new(1600, 82).unwrap()); // 801..=1920
    assert_eq!(auto.resolve(2560), ResolvedPair::new(1600, 70).unwrap()); // > 1920
    assert_eq!(auto.resolve(3840), ResolvedPair::new(1600, 70).unwrap()); // > 1920

    // 3. compute_reduced_dimensions_for_pair / compute_reduced_dimensions_with_edge
    let dims_4k = ImageDimensions::new(3840, 2160).unwrap();
    let reduced_1600 = dims_4k.compute_reduced_dimensions_with_edge(1600);
    assert_eq!(reduced_1600.width, 1600);
    assert_eq!(reduced_1600.height, 900);

    let dims_small = ImageDimensions::new(300, 200).unwrap();
    let reduced_no_upscale = dims_small.compute_reduced_dimensions_with_edge(1600);
    assert_eq!(reduced_no_upscale.width, 300);
    assert_eq!(reduced_no_upscale.height, 200);

    let dims_portrait = ImageDimensions::new(1080, 1920).unwrap();
    let reduced_portrait = dims_portrait.compute_reduced_dimensions_with_edge(960);
    assert_eq!(reduced_portrait.width, 540);
    assert_eq!(reduced_portrait.height, 960);

    // 4. compute_thumbnail_dimensions
    let thumb_dims = dims_4k.compute_thumbnail_dimensions(320);
    assert_eq!(thumb_dims.width, 320);
    assert_eq!(thumb_dims.height, 180);
}

#[test]
fn auto_derivation_varies_reduction_between_small_tooltip_and_4k_screen() {
    let auto_budget = QualityBudget::new(NamedBudget::Auto, None);

    // Capture A: Tooltip (312 x 118)
    let dims_a = ImageDimensions::new(312, 118).unwrap();
    let bytes_a = create_test_png(312, 118);
    let resolved_a = auto_budget.resolve(dims_a.long_edge());
    let result_a =
        ImageReducer::reduce_image(&bytes_a, dims_a.clone(), &resolved_a, false).unwrap();

    // Capture B: 4K dashboard (3840 x 2160)
    let dims_b = ImageDimensions::new(3840, 2160).unwrap();
    let bytes_b = create_test_png(3840, 2160);
    let resolved_b = auto_budget.resolve(dims_b.long_edge());
    let result_b =
        ImageReducer::reduce_image(&bytes_b, dims_b.clone(), &resolved_b, false).unwrap();

    // SCN-03 core assertion: resolved(A) != resolved(B)
    assert_ne!(resolved_a, resolved_b);
    assert_eq!(result_a.dimensions.width, 312); // No downscaling for small region
    assert_eq!(result_a.dimensions.height, 118);
    assert_eq!(result_b.dimensions.width, 1600); // Downscaled for 4K region
    assert_eq!(result_b.dimensions.height, 900);

    let decoded_a = image::load_from_memory(&result_a.bytes).unwrap();
    assert_eq!(decoded_a.width(), 312);
    assert_eq!(decoded_a.height(), 118);

    let decoded_b = image::load_from_memory(&result_b.bytes).unwrap();
    assert_eq!(decoded_b.width(), 1600);
    assert_eq!(decoded_b.height(), 900);
}

#[test]
fn fixed_presets_downscale_to_pinned_constants() {
    let orig_dims = ImageDimensions::new(3840, 2160).unwrap();
    let input_bytes = create_test_png(3840, 2160);

    // Sharp: 2560 px long edge, 90 quality
    let sharp_budget = QualityBudget::new(NamedBudget::Sharp, None);
    let sharp_res = ImageReducer::reduce_image_with_budget(
        &input_bytes,
        orig_dims.clone(),
        &sharp_budget,
        true,
    )
    .unwrap();
    assert_eq!(sharp_res.dimensions.width, 2560);
    assert_eq!(sharp_res.dimensions.height, 1440);

    // Balanced: 1600 px long edge, 75 quality
    let balanced_budget = QualityBudget::new(NamedBudget::Balanced, None);
    let balanced_res = ImageReducer::reduce_image_with_budget(
        &input_bytes,
        orig_dims.clone(),
        &balanced_budget,
        true,
    )
    .unwrap();
    assert_eq!(balanced_res.dimensions.width, 1600);
    assert_eq!(balanced_res.dimensions.height, 900);

    // Small: 1280 px long edge, 50 quality
    let small_budget = QualityBudget::new(NamedBudget::Small, None);
    let small_res =
        ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims, &small_budget, true)
            .unwrap();
    assert_eq!(small_res.dimensions.width, 1280);
    assert_eq!(small_res.dimensions.height, 720);
}

#[test]
fn custom_pair_reduction_honors_explicit_limits() {
    let custom_pair = ResolvedPair::new(1920, 85).unwrap();
    let custom_budget = QualityBudget::new(NamedBudget::Custom, Some(custom_pair));
    let orig_dims = ImageDimensions::new(3840, 2160).unwrap();
    let input_bytes = create_test_png(3840, 2160);

    let res =
        ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims, &custom_budget, false)
            .unwrap();
    assert_eq!(res.dimensions.width, 1920);
    assert_eq!(res.dimensions.height, 1080);

    let decoded = image::load_from_memory(&res.bytes).unwrap();
    assert_eq!(decoded.width(), 1920);
    assert_eq!(decoded.height(), 1080);
}

#[test]
fn corrupt_or_invalid_image_returns_validation_error() {
    let garbage_bytes = vec![0u8; 100];
    let dims = ImageDimensions::new(800, 600).unwrap();
    let resolved = ResolvedPair::new(1280, 80).unwrap();

    let err = ImageReducer::reduce_image(&garbage_bytes, dims, &resolved, false).unwrap_err();
    match err {
        CoreError::Validation(msg) => {
            assert!(msg.contains("Failed to decode image"));
        }
        other => panic!("Expected CoreError::Validation, got {:?}", other),
    }
}

#[test]
fn zero_byte_reservation_and_async_write_completion() {
    let tmp = TempDir::new().unwrap();
    let store = VaultBlobStore::new(tmp.path()).unwrap();

    let relative_path = "findings/test_reservation.png";
    let payload = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // Verify reservation and write
    ImageReducer::reserve_and_write(&store, relative_path, &payload).unwrap();

    assert!(store.blob_exists(relative_path).unwrap());
    let read_back = store.read_blob(relative_path).unwrap();
    assert_eq!(read_back, payload);
}
