use crate::domain::bundle::BundleItem;
use crate::domain::finding::FindingDetail;
use crate::error::CoreError;

/// One Marker's annotation as it sits inside a Finding's "### Marker Notes" list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarker {
    pub ordinal: u32,
    /// Empty when the document carried `*(No annotation text)*` - that string is the
    /// document's spelling of "no comment", not a comment in its own right.
    pub comment: String,
}

/// One "## Finding N" block, as it sits inside a Bundle document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFinding {
    pub position: u32,
    /// The image reference exactly as it sits in the document's `![Finding N](...)` link -
    /// `./finding_1_burned.png` for a stored document, an absolute `<...>`-wrapped path once
    /// `rebase_image_links` has run over it. Never resolved against a filesystem here.
    pub image_link: String,
    /// Empty when the document carried no "### Notes" heading for this Finding.
    pub note: String,
    pub markers: Vec<ParsedMarker>,
}

/// A Bundle document read back into the blocks `serialize_bundle` composed it from - the
/// composer's own document, not a Finding lookup. A sealed Bundle (`BR-11`) has no Findings left
/// to rebuild this from, so Review & Update (ticket 13/14) and Copy Markdown (ticket 12) both
/// stand on this instead.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedBundleDocument {
    pub title: String,
    /// Empty when the document carried no "## Bundle Notes" heading.
    pub notes: String,
    /// Empty for the early-return empty-bundle document (`_No findings included in this
    /// bundle._`), which has no per-Finding structure at all.
    pub findings: Vec<ParsedFinding>,
}

const EMPTY_BUNDLE_MARKER: &str = "_No findings included in this bundle._\n";
const BUNDLE_NOTES_HEADING: &str = "## Bundle Notes\n\n";
const FINDING_NOTES_HEADING: &str = "### Notes\n\n";
const MARKER_NOTES_HEADING: &str = "### Marker Notes\n\n";
const NO_ANNOTATION_TEXT: &str = "*(No annotation text)*";

#[derive(Debug, Clone)]
pub struct MarkdownSerializer;

impl MarkdownSerializer {
    /// Composes a single CommonMark document from bundle title and list of bundle items paired with their finding details.
    /// Invariants:
    /// - AD-1: Marker ordinal matches line number.
    /// - AD-4: Composed markdown references the bundle's burned copy (BundleItem.image_path).
    /// - AD-9: Byte-identical composition across all platforms.
    /// - NFR-8: every image reference resolves relative to the Markdown file's OWN folder, which is
    ///   why `markdown_path` is a parameter rather than something this function assumes. It was
    ///   assumed once, the document moved into a per-Bundle folder without the serializer's base
    ///   moving with it, and every link in every Bundle silently resolved to nothing (`BUG-86`).
    ///
    /// `intro` is the Bundle's own note - what this handoff is about, as against what any one Finding
    /// is about. Optional, and omitted entirely when empty, which is the rule a Finding's note
    /// already follows.
    ///
    /// It carries `## Bundle Notes` while a Finding's carries `### Notes`. The level alone would say
    /// whose note it is to anything that reads an outline, and it read as ambiguous to everyone else
    /// - so the heading names its scope as well.
    ///
    /// `parse_bundle_document` is this function's inverse: reading a document this composes back
    /// into a `ParsedBundleDocument`, which `render` turns back into text over exactly the same
    /// grammar written out below. Kept as its own body, not routed through `render`, so that the
    /// literal shape a Reviewer or an agent actually reads stays right here where the invariants
    /// above are documented, and so an existing wiring test that pins this exact source text
    /// (`a_bundle_carries_a_note_of_its_own`) keeps meaning what it says.
    pub fn serialize_bundle(
        bundle_name: &str,
        intro: &str,
        items: &[(&BundleItem, &FindingDetail)],
        markdown_path: &str,
    ) -> String {
        let mut out = String::new();

        // Title
        out.push_str("# ");
        out.push_str(bundle_name.trim());
        out.push_str("\n\n");

        if !intro.trim().is_empty() {
            out.push_str("## Bundle Notes\n\n");
            out.push_str(intro.trim());
            out.push_str("\n\n");
        }

        if items.is_empty() {
            out.push_str("_No findings included in this bundle._\n");
            return out;
        }

        for (item, detail) in items {
            let position = item.position;
            out.push_str(&format!("## Finding {position}\n\n"));

            // Image markdown reference pointing to bundle's burned copy, expressed the way the
            // reader of this document will resolve it.
            let img_rel = Self::image_reference(markdown_path, &item.image_path);
            out.push_str(&format!("![Finding {position}]({img_rel})\n\n"));

            // NO capture metadata.
            //
            // It used to emit `Captured`, `Resolution` and `Monitor` for every Finding. The reader of
            // this document is a coding agent, and none of the three tells it anything about what is
            // wrong in the image: a timestamp, a pixel size and a display device name are facts about
            // the CAPTURE, not about the finding. They cost tokens on every item, and `BG-3` is about
            // how few tokens a handoff takes. Removed at the owner's direction.
            //
            // The Vault still holds all three on the Finding row, so nothing is lost - they are
            // simply not part of the handoff.

            // The note, under a heading that says what it is.
            //
            // It used to be a bare paragraph after the image, indistinguishable from any other prose
            // in the document. An agent reading this needs to know that this text is the Reviewer's
            // observation about the whole image, as against a Marker's remark about one point in it,
            // so both now carry a heading and the two headings are different.
            //
            // `###` rather than `##`: it sits under `## Finding N`, and a document whose outline
            // skips a level reads as two documents to anything that parses headings.
            if !detail.note.body.trim().is_empty() {
                out.push_str("### Notes\n\n");
                out.push_str(detail.note.body.trim());
                out.push_str("\n\n");
            }

            // Marker annotations list
            if !detail.markers.is_empty() {
                out.push_str("### Marker Notes\n\n");
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

    /// Reads a document this composer produced back into its blocks - the inverse of
    /// `serialize_bundle`/`render` over exactly the grammar those two write down. A sealed Bundle
    /// (`BR-11`) has no Findings left to reconstruct this from, so the stored document has to stand
    /// on its own; this is what makes that possible.
    ///
    /// Rejects, rather than silently coerces, any document whose shape does not match: every error
    /// names what was expected and what was found instead.
    ///
    /// The bundle name is read from the document's own `# {name}` heading rather than taken as a
    /// separate parameter - the document already carries it, so nothing else needs to.
    ///
    /// This is a literal-anchor parser, not a general Markdown parser: it looks for the exact
    /// section markers `serialize_bundle` writes (`## Bundle Notes`, `## Finding N`, `### Notes`,
    /// `### Marker Notes`, the empty-bundle sentence) at the positions the grammar puts them, and
    /// treats everything else as opaque field text. A field may contain Markdown metacharacters,
    /// including a line that merely *looks* like a heading, without confusing it - what it must not
    /// contain is the literal next section marker verbatim, which is why raw Markdown editing is out
    /// of scope (see the spec's Out of Scope) and the four plain-text fields are the only edit
    /// surface.
    pub fn parse_bundle_document(document: &str) -> Result<ParsedBundleDocument, CoreError> {
        let mut rest = document.strip_prefix("# ").ok_or_else(|| {
            CoreError::Validation(
                "expected the document to open with '# ' (the Bundle title heading)".into(),
            )
        })?;

        let title_end = rest.find("\n\n").ok_or_else(|| {
            CoreError::Validation("expected a blank line after the title heading".into())
        })?;
        let title = rest[..title_end].to_string();
        rest = &rest[title_end + 2..];

        let mut notes = String::new();
        if let Some(after_heading) = rest.strip_prefix(BUNDLE_NOTES_HEADING) {
            let boundary = ["## Finding 1", EMPTY_BUNDLE_MARKER]
                .into_iter()
                .filter_map(|anchor| after_heading.find(anchor))
                .min()
                .ok_or_else(|| {
                    CoreError::Validation(
                        "'## Bundle Notes' is not followed by a Finding section or the \
                         empty-bundle marker"
                            .into(),
                    )
                })?;
            let body = after_heading[..boundary]
                .strip_suffix("\n\n")
                .ok_or_else(|| {
                    CoreError::Validation(
                        "'## Bundle Notes' section has no blank line after it".into(),
                    )
                })?;
            notes = body.to_string();
            rest = &after_heading[boundary..];
        }

        if let Some(after_marker) = rest.strip_prefix(EMPTY_BUNDLE_MARKER) {
            if !after_marker.is_empty() {
                return Err(CoreError::Validation(format!(
                    "unexpected content after the empty-bundle marker: {:?}",
                    Self::preview(after_marker)
                )));
            }
            return Ok(ParsedBundleDocument {
                title,
                notes,
                findings: Vec::new(),
            });
        }

        let mut findings = Vec::new();
        let mut position: u32 = 1;
        while !rest.is_empty() {
            let finding_heading = format!("## Finding {position}\n\n");
            rest = rest.strip_prefix(finding_heading.as_str()).ok_or_else(|| {
                CoreError::Validation(format!(
                    "expected '## Finding {position}' next, found: {:?}",
                    Self::preview(rest)
                ))
            })?;

            let image_prefix = format!("![Finding {position}](");
            rest = rest.strip_prefix(image_prefix.as_str()).ok_or_else(|| {
                CoreError::Validation(format!(
                    "expected the image reference for Finding {position} next, found: {:?}",
                    Self::preview(rest)
                ))
            })?;
            let close = rest.find(")\n\n").ok_or_else(|| {
                CoreError::Validation(format!(
                    "Finding {position}'s image reference is never closed with ')' followed by a \
                     blank line"
                ))
            })?;
            let image_link = rest[..close].to_string();
            rest = &rest[close + 3..];

            // LEGACY (`BUG-95`): a Bundle composed before the per-Finding capture metadata was
            // removed (`BG-3`, "a timestamp, a pixel size and a display device name are facts
            // about the CAPTURE, not about the Finding") still carries
            // "- **Captured:** ...\n- **Resolution:** ...\n- **Monitor:** ...\n\n" immediately
            // after the image line, in every document composed that long ago - and its note (if
            // any) follows directly with no "### Notes" heading of its own, since that heading did
            // not exist at the time either. `serialize_bundle`/`render` never emit this shape for
            // a NEW document, so tolerating it here does not weaken the round-trip guarantee over
            // current documents at all: it only lets an OLD one be read back rather than refusing
            // outright, which is what Copy Markdown and Review & Update both did until this was
            // found - a validation error instead of the Bundle's own content.
            let legacy_metadata_end = rest
                .strip_prefix("- **Captured:**")
                .and_then(|after| after.find("\n\n"))
                .map(|end| "- **Captured:**".len() + end + 2);
            let had_legacy_metadata = legacy_metadata_end.is_some();
            if let Some(end) = legacy_metadata_end {
                rest = &rest[end..];
            }

            let mut note = String::new();
            if let Some(after_heading) = rest.strip_prefix(FINDING_NOTES_HEADING) {
                let next_finding_heading = format!("## Finding {}", position + 1);
                let boundary = [MARKER_NOTES_HEADING, next_finding_heading.as_str()]
                    .into_iter()
                    .filter_map(|anchor| after_heading.find(anchor))
                    .min();

                let (body, remainder) = match boundary {
                    Some(idx) => {
                        let body = after_heading[..idx].strip_suffix("\n\n").ok_or_else(|| {
                            CoreError::Validation(format!(
                                "Finding {position}'s note has no blank line after it"
                            ))
                        })?;
                        (body.to_string(), &after_heading[idx..])
                    }
                    None => {
                        // The last Finding, with no Marker Notes list: the note runs to the end
                        // of the document.
                        let body = after_heading.strip_suffix("\n\n").ok_or_else(|| {
                            CoreError::Validation(format!(
                                "Finding {position}'s note does not end the document with a \
                                 trailing blank line"
                            ))
                        })?;
                        (body.to_string(), "")
                    }
                };
                note = body;
                rest = remainder;
            } else if had_legacy_metadata {
                // No "### Notes" heading existed yet in this era - whatever text runs up to the
                // next boundary (a Marker Notes list, the next Finding, or the end of the
                // document) IS the note, headerless. Only taken when the legacy metadata block
                // was actually seen above, so a genuinely note-less CURRENT-format Finding (no
                // metadata, no heading) still correctly parses as having no note rather than
                // swallowing whatever stray text a malformed document might carry there.
                let next_finding_heading = format!("## Finding {}", position + 1);
                let boundary = [MARKER_NOTES_HEADING, next_finding_heading.as_str()]
                    .into_iter()
                    .filter_map(|anchor| rest.find(anchor))
                    .min();
                let (body, remainder) = match boundary {
                    Some(idx) => (&rest[..idx], &rest[idx..]),
                    None => (rest, ""),
                };
                let body = body.strip_suffix("\n\n").unwrap_or(body).trim();
                if !body.is_empty() {
                    note = body.to_string();
                }
                rest = remainder;
            }

            let mut markers = Vec::new();
            if let Some(after_heading) = rest.strip_prefix(MARKER_NOTES_HEADING) {
                rest = after_heading;
                while let Some((marker, remainder)) = Self::take_marker_line(rest) {
                    markers.push(marker);
                    rest = remainder;
                }
                rest = rest.strip_prefix('\n').ok_or_else(|| {
                    CoreError::Validation(format!(
                        "Finding {position}'s Marker Notes list has no blank line after it"
                    ))
                })?;
            }

            findings.push(ParsedFinding {
                position,
                image_link,
                note,
                markers,
            });
            position += 1;
        }

        Ok(ParsedBundleDocument {
            title,
            notes,
            findings,
        })
    }

    /// Rebases every image link in a stored Bundle document from its stored, folder-relative form
    /// (`./...`) to an absolute, forward-slashed, `<>`-wrapped path - the form Copy Markdown puts on
    /// the clipboard (ticket 03, `AD-9` as narrowed by `DEC-012`). Nothing else in the document
    /// changes: this parses the document, then renders it again with only the link text swapped.
    ///
    /// `vault_root` and `markdown_path` are plain strings, not `std::path::Path`, on purpose:
    /// `AD-9` requires byte-identical composition on every platform, and joining with `Path` pulls
    /// in the native separator - exactly the class of bug `BUG-86` was. Correct for a Vault path
    /// containing a space, parentheses, or an apostrophe: none of those need escaping once the
    /// whole destination sits inside `<>`.
    pub fn rebase_image_links(
        document: &str,
        vault_root: &str,
        markdown_path: &str,
    ) -> Result<String, CoreError> {
        let parsed = Self::parse_bundle_document(document)?;

        let vault_root = vault_root.trim_end_matches(['/', '\\']).replace('\\', "/");
        let markdown_path = markdown_path.trim_start_matches('/');
        let folder = markdown_path.rsplit_once('/').map(|(folder, _)| folder);

        Ok(Self::render(&parsed, |link| {
            let relative = link.trim_start_matches("./");
            let absolute = match folder {
                Some(folder) => format!("{vault_root}/{folder}/{relative}"),
                None => format!("{vault_root}/{relative}"),
            };
            format!("<{absolute}>")
        }))
    }

    /// Renders a parsed document back to plain Markdown, with every image link left exactly as
    /// parsed. The counterpart to `rebase_image_links`, for a caller (ticket 14's Save) that needs
    /// the stored form of an edited set of blocks rather than a rebased one.
    pub fn serialize_parsed(document: &ParsedBundleDocument) -> String {
        Self::render(document, |link| link.to_string())
    }

    /// Tells whether serialising `edited` would reproduce `stored_document` byte for byte - "would
    /// this Save actually change anything" (`BR-5`'s no-op Save, which writes nothing and toasts
    /// "Saved. Nothing had changed.").
    pub fn document_unchanged(edited: &ParsedBundleDocument, stored_document: &str) -> bool {
        Self::serialize_parsed(edited) == stored_document
    }

    /// The link a CommonMark reader must follow: `image_path` expressed relative to the folder the
    /// document itself sits in.
    ///
    /// Both arguments are Vault-relative. The Vault lays a Bundle out as one folder holding
    /// `bundle.md` beside its burned copies, so the document's folder is a prefix of every image
    /// path and this is a prefix strip. If it ever is not - a layout this function was not told
    /// about - the Vault-relative path is returned unchanged rather than guessed at, and
    /// `test_nfr8_image_resolution` is what fails.
    fn image_reference(markdown_path: &str, image_path: &str) -> String {
        let image = image_path.trim_start_matches('/');
        let Some((folder, _file)) = markdown_path.trim_start_matches('/').rsplit_once('/') else {
            // The document sits at the Vault root, so a Vault-relative path already is relative
            // to its folder.
            return format!("./{image}");
        };
        match image
            .strip_prefix(folder)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            Some(rel) => format!("./{rel}"),
            None => format!("./{image}"),
        }
    }

    /// The one place the document's grammar is written down. `link_fn` gets each Finding's stored
    /// image link (e.g. `./finding_1_burned.png`) and returns whatever text should sit inside the
    /// `(...)` of its image reference - identity for a normal render, an absolute `<>`-wrapped path
    /// for `rebase_image_links`.
    fn render(doc: &ParsedBundleDocument, link_fn: impl Fn(&str) -> String) -> String {
        let mut out = String::new();

        // Title
        out.push_str("# ");
        out.push_str(doc.title.trim());
        out.push_str("\n\n");

        if !doc.notes.trim().is_empty() {
            out.push_str(BUNDLE_NOTES_HEADING);
            out.push_str(doc.notes.trim());
            out.push_str("\n\n");
        }

        if doc.findings.is_empty() {
            out.push_str(EMPTY_BUNDLE_MARKER);
            return out;
        }

        for finding in &doc.findings {
            let position = finding.position;
            out.push_str(&format!("## Finding {position}\n\n"));

            let img_dest = link_fn(&finding.image_link);
            out.push_str(&format!("![Finding {position}]({img_dest})\n\n"));

            // NO capture metadata.
            //
            // It used to emit `Captured`, `Resolution` and `Monitor` for every Finding. The reader of
            // this document is a coding agent, and none of the three tells it anything about what is
            // wrong in the image: a timestamp, a pixel size and a display device name are facts about
            // the CAPTURE, not about the finding. They cost tokens on every item, and `BG-3` is about
            // how few tokens a handoff takes. Removed at the owner's direction.
            //
            // The Vault still holds all three on the Finding row, so nothing is lost - they are
            // simply not part of the handoff.

            // The note, under a heading that says what it is.
            //
            // It used to be a bare paragraph after the image, indistinguishable from any other prose
            // in the document. An agent reading this needs to know that this text is the Reviewer's
            // observation about the whole image, as against a Marker's remark about one point in it,
            // so both now carry a heading and the two headings are different.
            //
            // `###` rather than `##`: it sits under `## Finding N`, and a document whose outline
            // skips a level reads as two documents to anything that parses headings.
            if !finding.note.trim().is_empty() {
                out.push_str(FINDING_NOTES_HEADING);
                out.push_str(finding.note.trim());
                out.push_str("\n\n");
            }

            // Marker annotations list
            if !finding.markers.is_empty() {
                out.push_str(MARKER_NOTES_HEADING);
                for marker in &finding.markers {
                    let comment = if marker.comment.trim().is_empty() {
                        NO_ANNOTATION_TEXT
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

    /// Parses one `{ordinal}. {comment}` line off the front of `s`, returning `None` the moment the
    /// line no longer starts with digits - which is exactly the blank line `render` puts after the
    /// last Marker in the list, so the caller's loop stops there without being told separately.
    fn take_marker_line(s: &str) -> Option<(ParsedMarker, &str)> {
        let digits_end = s.find(|c: char| !c.is_ascii_digit())?;
        if digits_end == 0 {
            return None;
        }
        let ordinal: u32 = s[..digits_end].parse().ok()?;
        let after_dot = s[digits_end..].strip_prefix(". ")?;
        let line_end = after_dot.find('\n')?;
        let text = &after_dot[..line_end];
        let comment = if text == NO_ANNOTATION_TEXT {
            String::new()
        } else {
            text.to_string()
        };
        Some((
            ParsedMarker { ordinal, comment },
            &after_dot[line_end + 1..],
        ))
    }

    /// A short, safe-to-print slice of unparsed input for an error message.
    fn preview(s: &str) -> &str {
        let end = s
            .char_indices()
            .nth(40)
            .map(|(idx, _)| idx)
            .unwrap_or(s.len());
        &s[..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::finding::{Finding, Marker, Note};

    #[test]
    fn serializes_empty_bundle() {
        let md =
            MarkdownSerializer::serialize_bundle("Empty Review", "", &[], "bundles/b-1/bundle.md");
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

        let md = MarkdownSerializer::serialize_bundle(
            "Login Review",
            "",
            &[(&item, &detail)],
            "bundles/b-1/bundle.md",
        );

        assert!(md.contains("# Login Review"));
        assert!(md.contains("## Finding 1"));
        assert!(md.contains("![Finding 1](./finding_1_burned.png)"));
        assert!(md.contains("Found layout misalignment on login card."));
        assert!(md.contains("### Marker Notes"));
        assert!(md.contains("1. Button overlapping text field"));
        assert!(md.contains("2. *(No annotation text)*"));

        // The Reviewer's own note carries a heading that says what it is, so an agent can tell the
        // general observation about the image from a Marker's remark about one point in it.
        assert!(md.contains("### Notes"));
        let notes_at = md.find("### Notes").expect("checked above");
        let markers_at = md.find("### Marker Notes").expect("checked above");
        assert!(
            notes_at < markers_at,
            "the general note must come before the per-Marker list"
        );

        // And NONE of the capture metadata. It describes the capture, not the finding, and it cost
        // tokens on every item - BG-3 is about how few of those a handoff takes.
        for absent in ["Captured:", "Resolution:", "Monitor:", "DISPLAY1", "1920"] {
            assert!(
                !md.contains(absent),
                "the handoff document must not carry `{absent}`: it says nothing about what is wrong in the image"
            );
        }
    }

    // --- Ticket 10: parse / rebase / no-op detection --------------------------------------------

    fn finding_detail(id: &str, note: &str, markers: Vec<Marker>) -> FindingDetail {
        FindingDetail {
            finding: Finding {
                id: id.into(),
                image_path: format!("findings/{id}.png"),
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
                id: format!("n-{id}"),
                finding_id: id.into(),
                body: note.into(),
                updated_at: "2026-08-23T10:00:00Z".into(),
            },
            markers,
            visual_annotations: vec![],
        }
    }

    fn bundle_item(bundle_id: &str, finding_id: &str, position: u32) -> BundleItem {
        BundleItem {
            id: format!("bi-{position}"),
            bundle_id: bundle_id.into(),
            finding_id: finding_id.into(),
            position,
            image_path: format!("bundles/{bundle_id}/finding_{position}_burned.png"),
        }
    }

    /// Same fixture shape as `test_golden_markdown.rs`'s golden document, kept independent of it
    /// (a different crate) but proving the same round-trip over the exact bytes that test asserts
    /// against.
    #[test]
    fn round_trips_the_golden_bundle_document() {
        let fid = "018f2345-6789-7abc-8def-012345678901";
        let detail = finding_detail(
            fid,
            "The submit button has incorrect margin on narrow viewports.",
            vec![
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
        );
        let item = bundle_item("b-golden", fid, 1);

        let golden = "# Release Quality Gate Assessment\n\
\n\
## Finding 1\n\
\n\
![Finding 1](./finding_1_burned.png)\n\
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

        let composed = MarkdownSerializer::serialize_bundle(
            "Release Quality Gate Assessment",
            "",
            &[(&item, &detail)],
            "bundles/b-golden/bundle.md",
        );
        assert_eq!(
            composed, golden,
            "sanity: this test's fixture must actually produce the golden bytes"
        );

        let parsed =
            MarkdownSerializer::parse_bundle_document(golden).expect("golden document must parse");
        let reserialized = MarkdownSerializer::serialize_parsed(&parsed);
        assert_eq!(
            reserialized, golden,
            "parsing the golden document then serialising it back must reproduce it byte for byte"
        );
    }

    #[test]
    fn round_trips_a_document_with_every_field_populated() {
        let fid1 = "f-1";
        let fid2 = "f-2";
        let item1 = bundle_item("b-1", fid1, 1);
        let item2 = bundle_item("b-1", fid2, 2);
        let detail1 = finding_detail(
            fid1,
            "First finding note.",
            vec![
                Marker::new("m-1".into(), fid1.into(), 1, 0.1, 0.1, "Marker one.".into()).unwrap(),
            ],
        );
        let detail2 = finding_detail(
            fid2,
            "Second finding note, spanning\ntwo lines.",
            vec![
                Marker::new("m-2".into(), fid2.into(), 1, 0.2, 0.2, "Marker two.".into()).unwrap(),
                Marker::new(
                    "m-3".into(),
                    fid2.into(),
                    2,
                    0.3,
                    0.3,
                    "Marker three.".into(),
                )
                .unwrap(),
            ],
        );

        let doc = MarkdownSerializer::serialize_bundle(
            "Full Review",
            "This handoff covers the checkout flow end to end.",
            &[(&item1, &detail1), (&item2, &detail2)],
            "bundles/b-1/bundle.md",
        );

        let parsed = MarkdownSerializer::parse_bundle_document(&doc).expect("must parse");
        assert_eq!(parsed.title, "Full Review");
        assert_eq!(
            parsed.notes,
            "This handoff covers the checkout flow end to end."
        );
        assert_eq!(parsed.findings.len(), 2);
        assert_eq!(MarkdownSerializer::serialize_parsed(&parsed), doc);
    }

    #[test]
    fn round_trips_a_document_with_every_optional_field_empty() {
        let fid = "f-1";
        let item = bundle_item("b-1", fid, 1);
        // No Bundle notes, no Finding note, one Marker with an empty comment.
        let detail = finding_detail(
            fid,
            "",
            vec![Marker::new("m-1".into(), fid.into(), 1, 0.1, 0.1, "".into()).unwrap()],
        );

        let doc = MarkdownSerializer::serialize_bundle(
            "Minimal Review",
            "",
            &[(&item, &detail)],
            "bundles/b-1/bundle.md",
        );

        assert!(!doc.contains("## Bundle Notes"));
        assert!(!doc.contains("### Notes"));
        assert!(doc.contains(NO_ANNOTATION_TEXT));

        let parsed = MarkdownSerializer::parse_bundle_document(&doc).expect("must parse");
        assert_eq!(parsed.notes, "");
        assert_eq!(parsed.findings[0].note, "");
        assert_eq!(parsed.findings[0].markers[0].comment, "");
        assert_eq!(MarkdownSerializer::serialize_parsed(&parsed), doc);
    }

    #[test]
    fn round_trips_a_document_with_no_findings_at_all() {
        let doc = MarkdownSerializer::serialize_bundle(
            "Empty Review",
            "Nothing made it in.",
            &[],
            "bundles/b-1/bundle.md",
        );

        let parsed = MarkdownSerializer::parse_bundle_document(&doc).expect("must parse");
        assert_eq!(parsed.title, "Empty Review");
        assert_eq!(parsed.notes, "Nothing made it in.");
        assert!(parsed.findings.is_empty());
        assert_eq!(MarkdownSerializer::serialize_parsed(&parsed), doc);
    }

    /// `BUG-95`: a real Bundle document, composed before the per-Finding capture metadata was
    /// removed and before the "### Notes" heading existed - copied byte for byte (line endings
    /// aside) from a Vault fixture that reproduced the exact real-world failure: Copy Markdown
    /// refused every Bundle composed that long ago with "expected '## Finding 2' next, found:
    /// \"- **Captured:**...\"". This is a READ test only: parsing an old document must succeed and
    /// recover its actual note text, never a round-trip test, since re-serializing normalizes the
    /// legacy shape away on purpose (dropping the metadata, adding the "### Notes" heading) rather
    /// than reproducing it byte for byte.
    #[test]
    fn parses_a_legacy_document_with_capture_metadata_and_a_headerless_note() {
        let doc = "# Bundle 2026-08-27 20:28\n\n\
                    ## Finding 1\n\n\
                    ![Finding 1](./bundles/b-legacy/finding_1_burned.png)\n\n\
                    - **Captured:** 2026-08-27T11:04:22Z\n\
                    - **Resolution:** 1234 \u{d7} 883 px\n\
                    - **Monitor:** \\\\.\\DISPLAY2\n\n\
                    makan yuk\n\n\
                    ## Finding 2\n\n\
                    ![Finding 2](./bundles/b-legacy/finding_2_burned.png)\n\n\
                    - **Captured:** 2026-08-27T11:03:56Z\n\
                    - **Resolution:** 916 \u{d7} 681 px\n\
                    - **Monitor:** \\\\.\\DISPLAY2\n\n";

        let parsed = MarkdownSerializer::parse_bundle_document(doc)
            .expect("a legacy document must parse, not refuse outright");
        assert_eq!(parsed.findings.len(), 2);
        assert_eq!(
            parsed.findings[0].image_link,
            "./bundles/b-legacy/finding_1_burned.png"
        );
        assert_eq!(
            parsed.findings[0].note, "makan yuk",
            "the headerless note text must be recovered, not dropped or mistaken for a heading"
        );
        assert_eq!(
            parsed.findings[1].note, "",
            "the last Finding has no note text after its metadata - must parse as empty, not error"
        );

        // Re-serializing normalizes the document to the current shape - no metadata, and a real
        // "### Notes" heading now wraps the recovered note - which is the intended migration, not
        // a round-trip violation (round-trip guarantees are over CURRENT-format documents only).
        let migrated = MarkdownSerializer::serialize_parsed(&parsed);
        assert!(!migrated.contains("**Captured:**"));
        assert!(migrated.contains("### Notes\n\nmakan yuk\n\n"));
    }

    #[test]
    fn round_trips_text_with_markdown_metacharacters() {
        let fid = "f-1";
        let item = bundle_item("b-1", fid, 1);
        let detail = finding_detail(
            fid,
            "A note with *emphasis*, _underscores_, <angle brackets>, a backslash \\ and a hash # \
             sign, plus a line that looks like a heading:\n# Not actually a title, just prose.",
            vec![Marker::new(
                "m-1".into(),
                fid.into(),
                1,
                0.1,
                0.1,
                "Marker with `code`, <tags>, and a trailing backslash \\.".into(),
            )
            .unwrap()],
        );

        let doc = MarkdownSerializer::serialize_bundle(
            "Title with *stars* and _underscores_ and a # sign",
            "Bundle notes with <brackets> and a backslash \\.",
            &[(&item, &detail)],
            "bundles/b-1/bundle.md",
        );

        let parsed = MarkdownSerializer::parse_bundle_document(&doc).expect("must parse");
        assert_eq!(
            parsed.title,
            "Title with *stars* and _underscores_ and a # sign"
        );
        assert_eq!(
            parsed.notes,
            "Bundle notes with <brackets> and a backslash \\."
        );
        assert!(parsed.findings[0].note.contains("# Not actually a title"));
        assert_eq!(MarkdownSerializer::serialize_parsed(&parsed), doc);
    }

    #[test]
    fn rejects_a_document_that_does_not_open_with_a_title_heading() {
        let err = MarkdownSerializer::parse_bundle_document("Not a bundle document at all")
            .expect_err("must be rejected");
        assert!(
            err.to_string().contains("title heading"),
            "error should say what was unexpected, got: {err}"
        );
    }

    #[test]
    fn rejects_a_document_with_a_finding_numbered_out_of_sequence() {
        let broken = "# Review\n\n## Finding 2\n\n![Finding 2](./x.png)\n\n";
        let err = MarkdownSerializer::parse_bundle_document(broken).expect_err("must be rejected");
        assert!(
            err.to_string().contains("Finding 1"),
            "error should name the Finding number it expected, got: {err}"
        );
    }

    #[test]
    fn rejects_a_document_whose_empty_bundle_marker_has_trailing_content() {
        let broken = "# Review\n\n_No findings included in this bundle._\nsomething extra";
        let err = MarkdownSerializer::parse_bundle_document(broken).expect_err("must be rejected");
        assert!(
            err.to_string().contains("empty-bundle marker"),
            "error should name the empty-bundle marker, got: {err}"
        );
    }

    fn two_finding_document(vault_folder_style: &str) -> (String, BundleItem, BundleItem, String) {
        let item1 = bundle_item("b-1", "f-1", 1);
        let item2 = bundle_item("b-1", "f-2", 2);
        let detail1 = finding_detail("f-1", "First note.", vec![]);
        let detail2 = finding_detail("f-2", "Second note.", vec![]);
        let markdown_path = "bundles/b-1/bundle.md";
        let doc = MarkdownSerializer::serialize_bundle(
            "Rebase Review",
            "",
            &[(&item1, &detail1), (&item2, &detail2)],
            markdown_path,
        );
        (doc, item1, item2, vault_folder_style.to_string())
    }

    /// Independently builds the document rebase should produce, by string-replacing only the two
    /// image link destinations in the stored document - a diff against the code under test, not an
    /// inspection of it.
    fn expected_rebase(stored: &str, vault_root_forward_slash: &str) -> String {
        stored
            .replace(
                "(./finding_1_burned.png)",
                &format!("(<{vault_root_forward_slash}/bundles/b-1/finding_1_burned.png>)"),
            )
            .replace(
                "(./finding_2_burned.png)",
                &format!("(<{vault_root_forward_slash}/bundles/b-1/finding_2_burned.png>)"),
            )
    }

    #[test]
    fn rebase_changes_only_image_link_destinations_for_a_vault_path_with_a_space() {
        let (stored, ..) = two_finding_document("unused");
        let vault_root = r"C:\Users\test\Snapdown Vault";
        let rebased =
            MarkdownSerializer::rebase_image_links(&stored, vault_root, "bundles/b-1/bundle.md")
                .expect("must rebase");

        let expected = expected_rebase(&stored, "C:/Users/test/Snapdown Vault");
        assert_eq!(
            rebased, expected,
            "rebasing must change only the image link destinations"
        );
        assert!(
            rebased.contains("<C:/Users/test/Snapdown Vault/bundles/b-1/finding_1_burned.png>")
        );
    }

    #[test]
    fn rebase_changes_only_image_link_destinations_for_a_vault_path_with_parentheses() {
        let (stored, ..) = two_finding_document("unused");
        let vault_root = r"C:\Users\test\Snapdown Vault (2024)";
        let rebased =
            MarkdownSerializer::rebase_image_links(&stored, vault_root, "bundles/b-1/bundle.md")
                .expect("must rebase");

        let expected = expected_rebase(&stored, "C:/Users/test/Snapdown Vault (2024)");
        assert_eq!(
            rebased, expected,
            "rebasing must change only the image link destinations"
        );
        assert!(rebased
            .contains("<C:/Users/test/Snapdown Vault (2024)/bundles/b-1/finding_1_burned.png>"));
    }

    #[test]
    fn rebase_changes_only_image_link_destinations_for_a_vault_path_with_an_apostrophe() {
        let (stored, ..) = two_finding_document("unused");
        let vault_root = r"C:\Users\test\Wira's Vault";
        let rebased =
            MarkdownSerializer::rebase_image_links(&stored, vault_root, "bundles/b-1/bundle.md")
                .expect("must rebase");

        let expected = expected_rebase(&stored, "C:/Users/test/Wira's Vault");
        assert_eq!(
            rebased, expected,
            "rebasing must change only the image link destinations"
        );
        assert!(rebased.contains("<C:/Users/test/Wira's Vault/bundles/b-1/finding_1_burned.png>"));
    }

    #[test]
    fn noop_detection_reports_unchanged_for_blocks_resubmitted_untouched() {
        let (stored, ..) = two_finding_document("unused");
        let parsed = MarkdownSerializer::parse_bundle_document(&stored).expect("must parse");
        assert!(MarkdownSerializer::document_unchanged(&parsed, &stored));
    }

    #[test]
    fn noop_detection_reports_changed_for_a_one_character_title_edit() {
        let (stored, ..) = two_finding_document("unused");
        let mut parsed = MarkdownSerializer::parse_bundle_document(&stored).expect("must parse");
        parsed.title.push('!');
        assert!(!MarkdownSerializer::document_unchanged(&parsed, &stored));
    }

    #[test]
    fn noop_detection_reports_changed_for_a_one_character_bundle_notes_edit() {
        let item = bundle_item("b-1", "f-1", 1);
        let detail = finding_detail("f-1", "Note.", vec![]);
        let stored = MarkdownSerializer::serialize_bundle(
            "Review",
            "Original notes.",
            &[(&item, &detail)],
            "bundles/b-1/bundle.md",
        );
        let mut parsed = MarkdownSerializer::parse_bundle_document(&stored).expect("must parse");
        assert!(MarkdownSerializer::document_unchanged(&parsed, &stored));
        parsed.notes.push('!');
        assert!(!MarkdownSerializer::document_unchanged(&parsed, &stored));
    }

    #[test]
    fn noop_detection_reports_changed_for_a_one_character_finding_note_edit() {
        let (stored, ..) = two_finding_document("unused");
        let mut parsed = MarkdownSerializer::parse_bundle_document(&stored).expect("must parse");
        assert!(MarkdownSerializer::document_unchanged(&parsed, &stored));
        parsed.findings[0].note.push('!');
        assert!(!MarkdownSerializer::document_unchanged(&parsed, &stored));
    }

    #[test]
    fn noop_detection_reports_changed_for_a_one_character_marker_note_edit() {
        let fid = "f-1";
        let item = bundle_item("b-1", fid, 1);
        let detail = finding_detail(
            fid,
            "Note.",
            vec![
                Marker::new("m-1".into(), fid.into(), 1, 0.1, 0.1, "Marker text.".into()).unwrap(),
            ],
        );
        let stored = MarkdownSerializer::serialize_bundle(
            "Review",
            "",
            &[(&item, &detail)],
            "bundles/b-1/bundle.md",
        );
        let mut parsed = MarkdownSerializer::parse_bundle_document(&stored).expect("must parse");
        assert!(MarkdownSerializer::document_unchanged(&parsed, &stored));
        parsed.findings[0].markers[0].comment.push('!');
        assert!(!MarkdownSerializer::document_unchanged(&parsed, &stored));
    }
}
