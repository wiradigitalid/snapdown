use snapdown_core::domain::finding::{Finding, FindingDetail, Marker, Note};
use snapdown_core::domain::markdown::MarkdownSerializer;

#[test]
fn markdown_serializer_multi_finding_golden_flow() {
    let f1 = FindingDetail {
        finding: Finding {
            id: "f1".into(),
            image_path: "findings/f1.webp".into(),
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
            image_path: "findings/f2.webp".into(),
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

    let doc = MarkdownSerializer::serialize_bundle("Sprint 42 Bug Batch", &[f1, f2]);

    assert!(doc.starts_with("# Sprint 42 Bug Batch\n\n"));
    assert!(doc.contains("## Finding 1"));
    assert!(doc.contains("![Finding 1](./findings/f1.webp)"));
    assert!(doc.contains("1. Badge 1 comment"));
    assert!(doc.contains("## Finding 2"));
    assert!(doc.contains("![Finding 2](./findings/f2.webp)"));
}
