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
        visual_annotations: vec![],
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
        visual_annotations: vec![],
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
        "",
        &[(&item1, &f1), (&item2, &f2)],
        "bundles/b1/bundle.md",
    );

    let expected = "# Sprint 42 Bug Batch\n\
\n\
## Finding 1\n\
\n\
![Finding 1](./finding_1_burned.png)\n\
\n\
### Notes\n\
\n\
First defect observed\n\
\n\
### Marker Notes\n\
\n\
1. Badge 1 comment\n\
\n\
## Finding 2\n\
\n\
![Finding 2](./finding_2_burned.png)\n\
\n\
### Notes\n\
\n\
Second defect observed\n\
\n";

    assert_eq!(
        doc, expected,
        "MarkdownSerializer output must match full document byte-for-byte across multiple findings"
    );
}

/// The Bundle's own note - what this handoff is about - and the rule that an empty one is absent.
///
/// It carries `## Bundle Notes` rather than `## Notes`. The level alone would tell an outline reader
/// whose note it is, and told nobody else - and `### Notes` CONTAINS `## Notes`, so the obvious
/// substring assertion here found the Finding's heading while claiming to check the Bundle's. Naming
/// the scope removed the trap instead of working around it.
#[test]
fn a_bundle_note_is_rendered_above_the_findings_and_omitted_when_empty() {
    let fid = "fid-intro";
    let detail = FindingDetail {
        finding: Finding {
            id: fid.into(),
            image_path: "findings/one.png".into(),
            image_width: 800,
            image_height: 600,
            captured_at: "2026-08-27T10:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "0,0,800,600".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        },
        note: Note {
            id: "n".into(),
            finding_id: fid.into(),
            body: "the row wraps at 1280".into(),
            updated_at: "2026-08-27T10:00:00Z".into(),
        },
        markers: vec![],
        visual_annotations: vec![],
    };
    let item = BundleItem {
        id: "bi".into(),
        bundle_id: "b".into(),
        finding_id: fid.into(),
        position: 1,
        image_path: "bundles/b/finding_1_burned.png".into(),
    };

    let with_intro = MarkdownSerializer::serialize_bundle(
        "Checkout regressions",
        "  Three layout defects found while testing the checkout at 1280.  ",
        &[(&item, &detail)],
        "bundles/b/bundle.md",
    );
    assert!(
        with_intro.contains(
            "## Bundle Notes\n\nThree layout defects found while testing the checkout at 1280.\n\n"
        ),
        "the Bundle note must be rendered under `## Bundle Notes`, trimmed:\n{with_intro}"
    );
    let intro_at = with_intro.find("## Bundle Notes").expect("checked above");
    let first_finding = with_intro
        .find("## Finding 1")
        .expect("a Finding must follow");
    assert!(
        intro_at < first_finding,
        "the Bundle note says what the handoff is about, so it comes before the Findings that make \
         it up"
    );

    let without = MarkdownSerializer::serialize_bundle(
        "Checkout regressions",
        "   ",
        &[(&item, &detail)],
        "bundles/b/bundle.md",
    );
    assert!(
        !without.contains("## Bundle Notes"),
        "a Bundle note that is empty or whitespace must be absent, not an empty heading:\n{without}"
    );
    // And the Finding's own note is untouched by either case.
    assert!(without.contains("### Notes"));
}
