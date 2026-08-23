use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::QualityBudget;
use snapdown_core::ports::BlobStore;
use snapdown_store::image::ImageReducer;
use snapdown_store::vault::VaultBlobStore;
use tempfile::TempDir;

#[test]
fn quality_budget_downscaling_and_compression() {
    let budget = QualityBudget::new(1600, 75).unwrap();
    let original_dims = ImageDimensions::new(3840, 2160).unwrap();
    let input_bytes = vec![255u8; 4096];

    let result = ImageReducer::reduce_image(&input_bytes, original_dims, &budget, true).unwrap();

    assert_eq!(result.dimensions.width, 1600);
    assert_eq!(result.dimensions.height, 900);
    assert_eq!(result.thumbnail_dimensions.as_ref().unwrap().width, 320);
    assert_eq!(result.thumbnail_dimensions.as_ref().unwrap().height, 180);
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
