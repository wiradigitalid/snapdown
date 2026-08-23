use crate::domain::finding::FindingDetail;

#[derive(Debug, Clone)]
pub struct MarkdownSerializer;

impl MarkdownSerializer {
    /// Composes a single CommonMark document from bundle title and list of findings with notes and markers.
    /// Invariants:
    /// - AD-1: Marker ordinal matches line number.
    /// - AD-9: Byte-identical composition across all platforms.
    /// - Relative image references without absolute paths.
    pub fn serialize_bundle(bundle_name: &str, findings: &[FindingDetail]) -> String {
        let mut out = String::new();

        // Title
        out.push_str("# ");
        out.push_str(bundle_name.trim());
        out.push_str("\n\n");

        if findings.is_empty() {
            out.push_str("_No findings included in this bundle._\n");
            return out;
        }

        for (idx, item) in findings.iter().enumerate() {
            let position = idx + 1;
            out.push_str(&format!("## Finding {}\n\n", position));

            // Image markdown reference
            let img_rel = format!("./{}", item.finding.image_path.trim_start_matches('/'));
            out.push_str(&format!("![Finding {}]({})\n\n", position, img_rel));

            // Metadata summary
            out.push_str(&format!(
                "- **Captured:** {}\n- **Resolution:** {} × {} px\n- **Monitor:** {}\n\n",
                item.finding.captured_at,
                item.finding.image_width,
                item.finding.image_height,
                item.finding.source_monitor
            ));

            // Note body
            if !item.note.body.trim().is_empty() {
                out.push_str(item.note.body.trim());
                out.push_str("\n\n");
            }

            // Marker annotations list
            if !item.markers.is_empty() {
                out.push_str("### Annotations\n\n");
                for marker in &item.markers {
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
        };

        let md = MarkdownSerializer::serialize_bundle("Login Review", &[detail]);

        assert!(md.contains("# Login Review"));
        assert!(md.contains("## Finding 1"));
        assert!(md.contains("![Finding 1](./findings/img1.webp)"));
        assert!(md.contains("Found layout misalignment on login card."));
        assert!(md.contains("### Annotations"));
        assert!(md.contains("1. Button overlapping text field"));
        assert!(md.contains("2. *(No annotation text)*"));
    }
}
