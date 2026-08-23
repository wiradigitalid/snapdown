use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::markdown::MarkdownSerializer;

#[test]
fn golden_file_bundle_markdown_exact_snapshot() {
    let fid = "018f2345-6789-7abc-8def-012345678901";
    let f1 = FindingDetail {
        finding: Finding {
            id: fid.into(),
            image_path: "findings/capture_login.png".into(),
            image_width: 1920,
            image_height: 1080,
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
    };

    let doc = MarkdownSerializer::serialize_bundle("Release Quality Gate Assessment", &[f1]);

    let expected_golden = r#"# Release Quality Gate Assessment

## Finding 1

![Finding 1](./findings/capture_login.png)

- **Captured:** 2026-08-23T10:00:00Z
- **Resolution:** 1920 × 1080 px
- **Monitor:** DISPLAY1

The submit button has incorrect margin on narrow viewports.

### Annotations

1. Button overlap with input field
2. Footer text clipped

"#;

    assert_eq!(
        doc, expected_golden,
        "MarkdownSerializer output must match golden reference byte-for-byte (AD-9, INV-EXPORT-001)"
    );
}
