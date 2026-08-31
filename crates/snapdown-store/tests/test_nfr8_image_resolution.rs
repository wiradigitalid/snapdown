//! NFR-8: "A Bundle's Markdown renders in a plain CommonMark reader, with every image reference
//! resolving relative to the Markdown file's own folder."
//!
//! `requirements.yaml` names its enforcement as "a rendering test over a composed Bundle", and for
//! five waves that test did not exist. What existed instead were four string-containment assertions
//! that baked the *broken* link form in as the expected output, so the defect was green everywhere
//! (`BUG-86`).
//!
//! This test does the one thing those could not: it lays the composed document and its image out on
//! disk exactly as `write_bundle` does, then resolves every `![...](...)` link **the way a reader
//! does** — against the Markdown file's own folder — and asserts the file is there. It is
//! insensitive to how the serializer computes the path, which is the point: it catches the next
//! drift too, whatever shape it takes.

use snapdown_core::domain::bundle::BundleItem;
use snapdown_core::domain::finding::{Finding, FindingDetail, Note};
use snapdown_core::domain::markdown::MarkdownSerializer;

/// Pulls every image target out of a CommonMark document: the `...` in `![alt](...)`.
fn image_targets(markdown: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = markdown;
    while let Some(start) = rest.find("![") {
        rest = &rest[start..];
        let Some(open) = rest.find("](") else { break };
        let Some(close) = rest[open..].find(')') else {
            break;
        };
        targets.push(rest[open + 2..open + close].to_string());
        rest = &rest[open + close..];
    }
    targets
}

fn finding_detail(id: &str) -> FindingDetail {
    FindingDetail {
        finding: Finding {
            id: id.into(),
            image_path: format!("findings/{id}.png"),
            image_width: 1920,
            image_height: 1080,
            captured_at: "2026-08-31T00:00:00Z".into(),
            source_monitor: "TEST".into(),
            region: "0,0,1920,1080".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        },
        note: Note {
            id: format!("n-{id}"),
            finding_id: id.into(),
            body: "The submit button overlaps the input on narrow viewports.".into(),
            updated_at: "2026-08-31T00:00:00Z".into(),
        },
        markers: vec![],
        visual_annotations: vec![],
    }
}

#[test]
fn every_image_link_in_a_composed_bundle_resolves_from_the_markdown_files_own_folder() {
    let vault = tempfile::tempdir().expect("temp vault");
    let vault_root = vault.path();

    // The layout `write_bundle` actually produces: the document and its burned copies are siblings
    // inside one per-Bundle folder.
    let bundle_id = "b-nfr8";
    let markdown_path = format!("bundles/{bundle_id}/bundle.md");

    let items: Vec<BundleItem> = (1..=3)
        .map(|position| BundleItem {
            id: format!("bi-{position}"),
            bundle_id: bundle_id.into(),
            finding_id: format!("f-{position}"),
            position,
            image_path: format!("bundles/{bundle_id}/finding_{position}_burned.png"),
        })
        .collect();
    let details: Vec<FindingDetail> = (1..=3).map(|n| finding_detail(&format!("f-{n}"))).collect();
    let pairs: Vec<(&BundleItem, &FindingDetail)> = items.iter().zip(details.iter()).collect();

    let markdown = MarkdownSerializer::serialize_bundle(
        "Checkout regressions",
        "Round two.",
        &pairs,
        &markdown_path,
    );

    // Lay the Bundle out on disk exactly as the Vault holds it.
    let doc_abs = vault_root.join(&markdown_path);
    std::fs::create_dir_all(doc_abs.parent().expect("document has a parent folder"))
        .expect("create bundle folder");
    std::fs::write(&doc_abs, markdown.as_bytes()).expect("write bundle.md");
    for item in &items {
        std::fs::write(
            vault_root.join(&item.image_path),
            b"not a real png, presence is the point",
        )
        .expect("write burned image");
    }

    let doc_folder = doc_abs.parent().expect("document has a parent folder");

    let targets = image_targets(&markdown);
    assert_eq!(
        targets.len(),
        items.len(),
        "every Finding must contribute exactly one image reference"
    );

    for (target, item) in targets.iter().zip(items.iter()) {
        assert!(
            !target.starts_with('/') && !target.contains(':'),
            "NFR-8: image reference must be relative, got {target:?}"
        );

        // Resolve it the way a CommonMark reader does: against the document's own folder.
        let resolved = doc_folder.join(target.trim_start_matches("./"));
        assert!(
            resolved.is_file(),
            "NFR-8: {target:?} does not resolve to a file from the Markdown's own folder.\n  \
             document:  {}\n  resolved:  {}\n  the image is actually at: {}",
            doc_abs.display(),
            resolved.display(),
            vault_root.join(&item.image_path).display()
        );
    }
}
