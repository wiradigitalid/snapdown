use snapdown_core::domain::bundle::BundleItem;
use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::markdown::MarkdownSerializer;

#[test]
fn the_markdown_serializer_renders_two_findings_byte_exactly() {
    let f1 = FindingDetail {
        finding: Finding {
            id: "f1".into(),
            image_path: "findings/f1.png".into(),
            image_width: 1920,
            image_height: 1080,
            captured_at: "2026-08-23T10:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "0,0,1920,1080".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        },
        note: Note {
            id: "n1".into(),
            finding_id: "f1".into(),
            body: "First defect observed".into(),
            updated_at: "2026-08-23T10:00:00Z".into(),
        },
        markers: vec![Marker::new(
            "m1".into(),
            "f1".into(),
            1,
            0.1,
            0.2,
            "Badge 1 comment".into(),
        )
        .unwrap()],
    };

    let f2 = FindingDetail {
        finding: Finding {
            id: "f2".into(),
            image_path: "findings/f2.png".into(),
            image_width: 800,
            image_height: 600,
            captured_at: "2026-08-23T11:00:00Z".into(),
            source_monitor: "DISPLAY2".into(),
            region: "10,10,800,600".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        },
        note: Note {
            id: "n2".into(),
            finding_id: "f2".into(),
            body: "Second defect observed".into(),
            updated_at: "2026-08-23T11:00:00Z".into(),
        },
        markers: vec![],
    };

    let item1 = BundleItem {
        id: "bi1".into(),
        bundle_id: "b1".into(),
        finding_id: "f1".into(),
        position: 1,
        image_path: "bundles/b1/finding_1_burned.png".into(),
    };

    let item2 = BundleItem {
        id: "bi2".into(),
        bundle_id: "b1".into(),
        finding_id: "f2".into(),
        position: 2,
        image_path: "bundles/b1/finding_2_burned.png".into(),
    };

    let doc = MarkdownSerializer::serialize_bundle(
        "Sprint 42 Bug Batch",
        &[(&item1, &f1), (&item2, &f2)],
    );

    let expected = "# Sprint 42 Bug Batch\n\
\n\
## Finding 1\n\
\n\
![Finding 1](./bundles/b1/finding_1_burned.png)\n\
\n\
- **Captured:** 2026-08-23T10:00:00Z\n\
- **Resolution:** 1920 \u{00D7} 1080 px\n\
- **Monitor:** DISPLAY1\n\
\n\
First defect observed\n\
\n\
### Annotations\n\
\n\
1. Badge 1 comment\n\
\n\
## Finding 2\n\
\n\
![Finding 2](./bundles/b1/finding_2_burned.png)\n\
\n\
- **Captured:** 2026-08-23T11:00:00Z\n\
- **Resolution:** 800 \u{00D7} 600 px\n\
- **Monitor:** DISPLAY2\n\
\n\
Second defect observed\n\
\n";

    assert_eq!(
        doc, expected,
        "MarkdownSerializer output must match full document byte-for-byte across multiple findings"
    );
}
