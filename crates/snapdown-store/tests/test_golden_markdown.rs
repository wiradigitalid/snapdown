use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::bundle::BundleItem;
use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::markdown::MarkdownSerializer;
use snapdown_core::domain::setting::ResolvedPair;
use snapdown_store::image::ImageReducer;

fn make_gradient_png(w: u32, h: u32) -> Vec<u8> {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = (((x + y) * 128) / (w + h).max(1)) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
        .expect("Failed to encode test gradient PNG");
    bytes
}

#[test]
fn the_golden_bundle_markdown_is_regenerated_from_real_image_output() {
    // 1. Synthesise deterministic non-uniform source image in memory (no screen capture)
    let src_w = 3840;
    let src_h = 2160;
    let source_bytes = make_gradient_png(src_w, src_h);
    let orig_dims = ImageDimensions::new(src_w, src_h).unwrap();

    // 2. Reduce image with pinned ResolvedPair through real ImageReducer pipeline
    let pinned_pair = ResolvedPair::new(1920, 85).unwrap();
    let reduced = ImageReducer::reduce_image(&source_bytes, orig_dims, &pinned_pair, false)
        .expect("ImageReducer::reduce_image should succeed");

    // 3. Decode reduced bytes and extract dimensions directly from decoded image
    let decoded = image::load_from_memory(&reduced.bytes)
        .expect("Reduced image bytes must decode cleanly as PNG");
    let decoded_w = decoded.width();
    let decoded_h = decoded.height();

    // Assert decoded dimensions match the pinned expectation
    assert_eq!(
        decoded_w, 1920,
        "Decoded width must match pinned pair width"
    );
    assert_eq!(
        decoded_h, 1080,
        "Decoded height must match pinned pair height"
    );
    assert_eq!(reduced.dimensions.width, 1920);
    assert_eq!(reduced.dimensions.height, 1080);

    // 4. Construct FindingDetail using decoded dimensions (provenance from decode, not literals)
    let fid = "018f2345-6789-7abc-8def-012345678901";
    let f1 = FindingDetail {
        finding: Finding {
            id: fid.into(),
            image_path: "findings/capture_login.png".into(),
            image_width: decoded_w,
            image_height: decoded_h,
            captured_at: "2026-08-23T10:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "100,100,1920,1080".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        },
        note: Note {
            id: "n-1".into(),
            finding_id: fid.into(),
            body: "The submit button has incorrect margin on narrow viewports.".into(),
            updated_at: "2026-08-23T10:00:00Z".into(),
        },
        markers: vec![
            Marker::new(
                "m-1".into(),
                fid.into(),
                1,
                0.2,
                0.3,
                "Button overlap with input field".into(),
            )
            .unwrap(),
            Marker::new(
                "m-2".into(),
                fid.into(),
                2,
                0.8,
                0.85,
                "Footer text clipped".into(),
            )
            .unwrap(),
        ],
        visual_annotations: vec![],
    };

    let item1 = BundleItem {
        id: "bi-golden-1".into(),
        bundle_id: "b-golden".into(),
        finding_id: fid.into(),
        position: 1,
        image_path: "bundles/b-golden/finding_1_burned.png".into(),
    };

    // 5. Serialize bundle and compare byte-for-byte with inline golden reference (AD-4, AD-9, BUG-21)
    let doc = MarkdownSerializer::serialize_bundle(
        "Release Quality Gate Assessment",
        "",
        &[(&item1, &f1)],
    );

    let expected_golden = "# Release Quality Gate Assessment\n\
\n\
## Finding 1\n\
\n\
![Finding 1](./bundles/b-golden/finding_1_burned.png)\n\
\n\
### Notes\n\
\n\
The submit button has incorrect margin on narrow viewports.\n\
\n\
### Marker Notes\n\
\n\
1. Button overlap with input field\n\
2. Footer text clipped\n\
\n";

    assert_eq!(
        doc, expected_golden,
        "MarkdownSerializer output must match golden reference byte-for-byte (AD-9, INV-EXPORT-001)"
    );
}
