//! `BUG-107`: cropping a Finding must remap its existing Markers/annotations into the cropped
//! image's own coordinate space - not merely leave them present.
//!
//! Every test here decodes the actual post-crop coordinates and checks them against the
//! mathematically correct new position, the way `AGENTS.md` requires for this repository ("a
//! reachability test is not proof of correctness" applies to geometry the same way it applies to
//! images: the count staying unchanged, or a row still existing, is exactly the hollow assertion
//! that would pass whether or not the remap ever ran).

use snapdown_core::domain::finding::{AnnotationShape, CropRect, Finding, Note};
use snapdown_core::ports::FindingStore;
use snapdown_store::sqlite::SqliteFindingStore;

/// A Finding whose image is `width` x `height`, with no markers yet.
fn store_with_a_finding(fid: &str, width: u32, height: u32) -> SqliteFindingStore {
    let store = SqliteFindingStore::open_in_memory().expect("open memory store");
    let finding = Finding {
        id: fid.to_string(),
        image_path: format!("findings/{fid}.png"),
        image_width: width,
        image_height: height,
        captured_at: "2026-09-05T09:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: format!("0,0,{width},{height}"),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: format!("note-{fid}"),
        finding_id: fid.to_string(),
        body: String::new(),
        updated_at: "2026-09-05T09:00:00Z".to_string(),
    };
    store.create_finding(&finding, &note, &[]).unwrap();
    store
}

/// The crop this whole file uses: a 1000x1000 image, cropped down to the 400x500 rectangle whose
/// top-left corner sits at (200, 100). The resulting image is therefore 400x500.
const OLD_W: u32 = 1000;
const OLD_H: u32 = 1000;
const CROP: CropRect = CropRect {
    x: 200,
    y: 100,
    width: 400,
    height: 500,
};
const NEW_W: u32 = 400;
const NEW_H: u32 = 500;

/// The whole of `BUG-107` in one assertion: a Marker well inside the crop lands at its
/// mathematically exact new fraction, decoded back out of the store - not merely "still exists".
#[test]
fn a_marker_inside_the_crop_is_remapped_to_the_exact_new_fraction() {
    let store = store_with_a_finding("f-inside", OLD_W, OLD_H);

    // Old pixel (500, 500) -> new pixel (300, 400) -> new fraction (0.75, 0.8).
    store
        .add_marker("f-inside", "m-1", 0.5, 0.5, "the bug is here")
        .unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-inside", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let detail = store.get_finding("f-inside").unwrap().unwrap();
    assert_eq!(
        detail.markers.len(),
        1,
        "the marker must survive a crop that still contains its pixel"
    );
    let marker = &detail.markers[0];
    assert!(
        (marker.x - 0.75).abs() < 1e-9,
        "expected x=0.75 (the OLD image's pixel (500,500) minus the crop's own origin (200,100), \
         re-normalized against the NEW 400-wide image), got x={}",
        marker.x
    );
    assert!(
        (marker.y - 0.8).abs() < 1e-9,
        "expected y=0.8, got y={}",
        marker.y
    );
    assert_eq!(
        marker.comment, "the bug is here",
        "the comment must survive untouched"
    );
}

/// A Marker anchored to a pixel the crop removed must be DROPPED - not left at a stale fraction,
/// and not clamped onto the new image's edge, which would silently misrepresent where the
/// Reviewer actually pointed. This is the domain decision `BUG-107`'s own defect row flagged as
/// not yet made; see the accompanying report for the reasoning.
#[test]
fn a_marker_outside_the_crop_is_dropped_and_the_remaining_ones_are_renumbered() {
    let store = store_with_a_finding("f-outside", OLD_W, OLD_H);

    // m-survivor: old pixel (500, 500) -> stays inside the crop.
    store
        .add_marker("f-outside", "m-survivor", 0.5, 0.5, "kept")
        .unwrap();
    // m-doomed: old pixel (100, 100) -> above and to the left of the crop rectangle entirely.
    store
        .add_marker("f-outside", "m-doomed", 0.1, 0.1, "cropped out")
        .unwrap();
    // m-second-survivor: old pixel (550, 550) -> also stays inside the crop, ordinal 3 before the
    // remap - this is what proves the renumber ran, not just the delete.
    store
        .add_marker("f-outside", "m-second-survivor", 0.55, 0.55, "also kept")
        .unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-outside", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let detail = store.get_finding("f-outside").unwrap().unwrap();
    let ids: Vec<&str> = detail.markers.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["m-survivor", "m-second-survivor"],
        "the marker whose pixel was cropped away must be gone, and the survivors must keep their \
         relative order"
    );
    let ordinals: Vec<u32> = detail.markers.iter().map(|m| m.ordinal).collect();
    assert_eq!(
        ordinals,
        vec![1, 2],
        "a drop must not leave a gap in the ordinal sequence - the same invariant delete_marker \
         already enforces for a single deletion"
    );
}

/// A box-shaped annotation that only partially overlaps the crop must be CLIPPED to what survives,
/// the same way the pixels underneath it were clipped by the crop - not dropped, and not left at
/// its old (now wrong) box.
#[test]
fn a_rect_annotation_partially_inside_the_crop_is_clipped_to_the_exact_surviving_box() {
    let store = store_with_a_finding("f-rect", OLD_W, OLD_H);

    // Old pixel box (50, 50)-(250, 250): straddles the crop's top-left corner (200, 100). After
    // the crop, only the piece from (0, 0) to (50, 150) in NEW pixels remains.
    let shape = AnnotationShape::Rect {
        x: 0.05,
        y: 0.05,
        width: 0.2,
        height: 0.2,
        stroke_color: Some("#dc2626".to_string()),
        stroke_width: Some(3.0),
    };
    store
        .add_annotation("f-rect", "a-1", &shape, "2026-09-05T09:01:00Z")
        .unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-rect", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let detail = store.get_finding("f-rect").unwrap().unwrap();
    assert_eq!(
        detail.visual_annotations.len(),
        1,
        "the rect partially survives, so it must remain"
    );
    match &detail.visual_annotations[0].data {
        AnnotationShape::Rect {
            x,
            y,
            width,
            height,
            stroke_color,
            stroke_width,
        } => {
            assert!((x - 0.0).abs() < 1e-9, "expected x=0.0, got x={x}");
            assert!((y - 0.0).abs() < 1e-9, "expected y=0.0, got y={y}");
            assert!(
                (width - 0.125).abs() < 1e-9,
                "expected width=50/400=0.125 (clipped to what survives), got width={width}"
            );
            assert!(
                (height - 0.3).abs() < 1e-9,
                "expected height=150/500=0.3, got height={height}"
            );
            assert_eq!(
                stroke_color.as_deref(),
                Some("#dc2626"),
                "styling must survive untouched"
            );
            assert_eq!(*stroke_width, Some(3.0));
        }
        other => panic!("expected a Rect back, got {other:?}"),
    }
}

/// A box-shaped annotation with no surviving area at all must be dropped entirely.
#[test]
fn a_rect_annotation_entirely_outside_the_crop_is_deleted() {
    let store = store_with_a_finding("f-rect-gone", OLD_W, OLD_H);

    // Old pixel box (0, 0)-(100, 100): entirely above and to the left of the crop rectangle.
    let shape = AnnotationShape::Rect {
        x: 0.0,
        y: 0.0,
        width: 0.1,
        height: 0.1,
        stroke_color: None,
        stroke_width: None,
    };
    store
        .add_annotation("f-rect-gone", "a-1", &shape, "2026-09-05T09:01:00Z")
        .unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-rect-gone", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let detail = store.get_finding("f-rect-gone").unwrap().unwrap();
    assert_eq!(
        detail.visual_annotations.len(),
        0,
        "an annotation with zero surviving area must be deleted, not kept at a fabricated position"
    );
}

/// An Arrow straddling the crop boundary keeps both endpoints, clamped onto the new image's own
/// edge rather than dropped whole - a line, unlike a Marker's single point, still means something
/// with one end pinned to the edge.
#[test]
fn an_arrow_straddling_the_crop_edge_has_its_endpoints_clamped() {
    let store = store_with_a_finding("f-arrow", OLD_W, OLD_H);

    // Old pixel start (150, 150): just outside the crop, to the left. Old pixel end (350, 350):
    // well inside it.
    let shape = AnnotationShape::Arrow {
        start_x: 0.15,
        start_y: 0.15,
        end_x: 0.35,
        end_y: 0.35,
        color: Some("#dc2626".to_string()),
        stroke_width: Some(4.0),
    };
    store
        .add_annotation("f-arrow", "a-1", &shape, "2026-09-05T09:01:00Z")
        .unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-arrow", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let detail = store.get_finding("f-arrow").unwrap().unwrap();
    assert_eq!(detail.visual_annotations.len(), 1);
    match &detail.visual_annotations[0].data {
        AnnotationShape::Arrow {
            start_x,
            start_y,
            end_x,
            end_y,
            ..
        } => {
            // start: new pixel (-50, 50) clamps to (0, 50) -> fraction (0.0, 0.1).
            assert!((start_x - 0.0).abs() < 1e-9, "got start_x={start_x}");
            assert!((start_y - 0.1).abs() < 1e-9, "got start_y={start_y}");
            // end: new pixel (150, 250), already inside -> fraction (0.375, 0.5).
            assert!((end_x - 0.375).abs() < 1e-9, "got end_x={end_x}");
            assert!((end_y - 0.5).abs() < 1e-9, "got end_y={end_y}");
        }
        other => panic!("expected an Arrow back, got {other:?}"),
    }
}

/// The list read path (`list_findings`) is a separate query from `get_finding` in this store, and
/// `BUG-72` was exactly a fix landing on one and not the other - so the remap's persistence is
/// checked through both.
#[test]
fn the_list_read_path_sees_the_remapped_marker_too() {
    let store = store_with_a_finding("f-list", OLD_W, OLD_H);
    store.add_marker("f-list", "m-1", 0.5, 0.5, "note").unwrap();

    store
        .remap_markers_and_annotations_for_crop("f-list", OLD_W, OLD_H, CROP, NEW_W, NEW_H)
        .expect("remap must succeed");

    let listed = store.list_findings().unwrap();
    let found = listed.iter().find(|d| d.finding.id == "f-list").unwrap();
    assert_eq!(found.markers.len(), 1);
    assert!(
        (found.markers[0].x - 0.75).abs() < 1e-9,
        "got x={}",
        found.markers[0].x
    );
    assert!(
        (found.markers[0].y - 0.8).abs() < 1e-9,
        "got y={}",
        found.markers[0].y
    );
}
