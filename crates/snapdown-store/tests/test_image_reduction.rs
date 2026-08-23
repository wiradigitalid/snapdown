use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{NamedBudget, QualityBudget, ResolvedPair};
use snapdown_core::ports::BlobStore;
use snapdown_store::image::ImageReducer;
use snapdown_store::vault::VaultBlobStore;
use tempfile::TempDir;

#[test]
fn auto_derivation_varies_reduction_between_small_tooltip_and_4k_screen() {
    let auto_budget = QualityBudget::new(NamedBudget::Auto, None);

    // Capture A: Tooltip (312 x 118)
    let dims_a = ImageDimensions::new(312, 118).unwrap();
    let bytes_a = vec![128u8; 2048];
    let resolved_a = auto_budget.resolve(dims_a.long_edge());
    let result_a = ImageReducer::reduce_image(&bytes_a, dims_a.clone(), &resolved_a, false).unwrap();

    // Capture B: 4K dashboard (3840 x 2160)
    let dims_b = ImageDimensions::new(3840, 2160).unwrap();
    let bytes_b = vec![64u8; 8192];
    let resolved_b = auto_budget.resolve(dims_b.long_edge());
    let result_b = ImageReducer::reduce_image(&bytes_b, dims_b.clone(), &resolved_b, false).unwrap();

    // SCN-03 core assertion: resolved(A) != resolved(B)
    assert_ne!(resolved_a, resolved_b);
    assert_eq!(result_a.dimensions.width, 312); // No downscaling for small region
    assert_eq!(result_a.dimensions.height, 118);
    assert_eq!(result_b.dimensions.width, 1600); // Downscaled for 4K region
    assert_eq!(result_b.dimensions.height, 900);
}

#[test]
fn fixed_presets_downscale_to_pinned_constants() {
    let orig_dims = ImageDimensions::new(3840, 2160).unwrap();
    let input_bytes = vec![255u8; 4096];

    // Sharp: 2560 px long edge, 90 quality
    let sharp_budget = QualityBudget::new(NamedBudget::Sharp, None);
    let sharp_res = ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims.clone(), &sharp_budget, true).unwrap();
    assert_eq!(sharp_res.dimensions.width, 2560);
    assert_eq!(sharp_res.dimensions.height, 1440);

    // Balanced: 1600 px long edge, 75 quality
    let balanced_budget = QualityBudget::new(NamedBudget::Balanced, None);
    let balanced_res = ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims.clone(), &balanced_budget, true).unwrap();
    assert_eq!(balanced_res.dimensions.width, 1600);
    assert_eq!(balanced_res.dimensions.height, 900);

    // Small: 1280 px long edge, 50 quality
    let small_budget = QualityBudget::new(NamedBudget::Small, None);
    let small_res = ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims, &small_budget, true).unwrap();
    assert_eq!(small_res.dimensions.width, 1280);
    assert_eq!(small_res.dimensions.height, 720);
}

#[test]
fn custom_pair_reduction_honors_explicit_limits() {
    let custom_pair = ResolvedPair::new(1920, 85).unwrap();
    let custom_budget = QualityBudget::new(NamedBudget::Custom, Some(custom_pair));
    let orig_dims = ImageDimensions::new(3840, 2160).unwrap();
    let input_bytes = vec![200u8; 4096];

    let res = ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims, &custom_budget, false).unwrap();
    assert_eq!(res.dimensions.width, 1920);
    assert_eq!(res.dimensions.height, 1080);
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