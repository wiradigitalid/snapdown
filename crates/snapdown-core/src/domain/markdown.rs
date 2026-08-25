use crate::domain::bundle::BundleItem;
use crate::domain::finding::FindingDetail;

#[derive(Debug, Clone)]
pub struct MarkdownSerializer;

impl MarkdownSerializer {
    /// Composes a single CommonMark document from bundle title and list of bundle items paired with their finding details.
    /// Invariants:
    /// - AD-1: Marker ordinal matches line number.
    /// - AD-4: Composed markdown references the bundle's burned copy (BundleItem.image_path).
    /// - AD-9: Byte-identical composition across all platforms.
    /// - Relative image references without absolute paths.
    pub fn serialize_bundle(bundle_name: &str, items: &[(&BundleItem, &FindingDetail)]) -> String {
        let mut out = String::new();

        // Title
        out.push_str("# ");
        out.push_str(bundle_name.trim());
        out.push_str("\n\n");

        if items.is_empty() {
            out.push_str("_No findings included in this bundle._\n");
            return out;
        }

        for (item, detail) in items {
            let position = item.position;
            out.push_str(&format!("## Finding {}\n\n", position));

            // Image markdown reference pointing to bundle's burned copy
            let img_rel = format!("./{}", item.image_path.trim_start_matches('/'));
            out.push_str(&format!("![Finding {}]({})\n\n", position, img_rel));

            // Metadata summary
            out.push_str(&format!(
                "- **Captured:** {}\n- **Resolution:** {} × {} px\n- **Monitor:** {}\n\n",
                detail.finding.captured_at,
                detail.finding.image_width,
                detail.finding.image_height,
                detail.finding.source_monitor
            ));

            // Note body
            if !detail.note.body.trim().is_empty() {
                out.push_str(detail.note.body.trim());
                out.push_str("\n\n");
            }

            // Marker annotations list
            if !detail.markers.is_empty() {
                out.push_str("### Annotations\n\n");
                for marker in &detail.markers {
                    let comment = if marker.comment.trim().is_empty() {
                        "*(No annotation text)*"
                    } else {
                        marker.comment.trim()
                    };
                    out.push_str(&format!("{}. {}\n", marker.ordinal, comment));
                }
                out.push('\n');
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::finding::{Finding, Marker, Note};

    #[test]
    fn serializes_empty_bundle() {
        let md = MarkdownSerializer::serialize_bundle("Empty Review", &[]);
        assert!(md.contains("# Empty Review"));
        assert!(md.contains("No findings included"));
    }

    #[test]
    fn serializes_bundle_with_findings_notes_and_markers() {
        let fid = "fid-1";
        let finding = Finding {
            id: fid.into(),
            image_path: "findings/img1.webp".into(),
            image_width: 1920,
            image_height: 1080,
            captured_at: "2026-08-23T10:00:00Z".into(),
            source_monitor: "DISPLAY1".into(),
            region: "0,0,1920,1080".into(),
            resolved_long_edge: None,
            resolved_encoder_quality: None,
            budget_name: None,
        };
        let note = Note {
            id: "note-1".into(),
            finding_id: fid.into(),
            body: "Found layout misalignment on login card.".into(),
            updated_at: "2026-08-23T10:00:00Z".into(),
        };
        let m1 = Marker::new(
            "m1".into(),
            fid.into(),
            1,
            0.5,
            0.5,
            "Button overlapping text field".into(),
        )
        .unwrap();
        let m2 = Marker::new("m2".into(), fid.into(), 2, 0.8, 0.9, "".into()).unwrap();

        let detail = FindingDetail {
            finding,
            note,
            markers: vec![m1, m2],
            visual_annotations: vec![],
        };

        let item = BundleItem {
            id: "bi-1".into(),
            bundle_id: "b-1".into(),
            finding_id: fid.into(),
            position: 1,
            image_path: "bundles/b-1/finding_1_burned.png".into(),
        };

        let md = MarkdownSerializer::serialize_bundle("Login Review", &[(&item, &detail)]);

        assert!(md.contains("# Login Review"));
        assert!(md.contains("## Finding 1"));
        assert!(md.contains("![Finding 1](./bundles/b-1/finding_1_burned.png)"));
        assert!(md.contains("Found layout misalignment on login card."));
        assert!(md.contains("### Annotations"));
        assert!(md.contains("1. Button overlapping text field"));
        assert!(md.contains("2. *(No annotation text)*"));
    }
}
