use snapdown_core::domain::finding::Marker;
use snapdown_core::domain::image::ImageDimensions;
use snapdown_store::image::MarkerBurner;

#[test]
fn marker_burner_encodes_badges_and_maintains_image_integrity() {
    let dims = ImageDimensions::new(800, 600).unwrap();
    let markers = vec![
        Marker::new("m1".into(), "f1".into(), 1, 0.1, 0.2, "Badge 1".into()).unwrap(),
        Marker::new("m2".into(), "f1".into(), 2, 0.9, 0.8, "Badge 2".into()).unwrap(),
    ];

    let raw_bytes = vec![255u8; 512];
    let burned = MarkerBurner::burn_markers(&raw_bytes, &dims, &markers).unwrap();

    assert!(burned.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(burned.len() > raw_bytes.len());
}
