//! `CAP-11`'s persistence half - `FR-30`, `FR-31`, `FR-32`.
//!
//! `BUG-72`: the domain type and the burner were both written, both tested, and there was no table,
//! no port method and no read. `finding_store.rs` hardcoded `visual_annotations: vec![]` at both
//! read paths, so a `FindingDetail` could not carry one however it was created. Every test in here
//! fails against that code, and the first one fails at `add_annotation` not existing at all.

use snapdown_core::domain::finding::{AnnotationShape, Finding, Note};
use snapdown_core::error::CoreError;
use snapdown_core::ports::FindingStore;
use snapdown_store::sqlite::SqliteFindingStore;

fn store_with_a_finding(fid: &str) -> SqliteFindingStore {
    let store = SqliteFindingStore::open_in_memory().expect("open memory store");
    let finding = Finding {
        id: fid.to_string(),
        image_path: format!("findings/{fid}.png"),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-28T09:00:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,800,600".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: format!("note-{fid}"),
        finding_id: fid.to_string(),
        body: String::new(),
        updated_at: "2026-08-28T09:00:00Z".to_string(),
    };
    store.create_finding(&finding, &note, &[]).unwrap();
    store
}

fn rect(x: f64, y: f64) -> AnnotationShape {
    AnnotationShape::Rect {
        x,
        y,
        width: 0.2,
        height: 0.1,
        stroke_color: Some("#dc2626".to_string()),
        stroke_width: Some(3.0),
    }
}

/// The whole of `BUG-72` in one assertion: an annotation survives being written.
#[test]
fn an_annotation_placed_on_a_finding_comes_back_from_the_store() {
    let store = store_with_a_finding("f-1");

    store
        .add_annotation("f-1", "a-1", &rect(0.1, 0.2), "2026-08-28T09:01:00Z")
        .expect("add_annotation must succeed on an existing Finding");

    let detail = store.get_finding("f-1").unwrap().unwrap();
    assert_eq!(
        detail.visual_annotations.len(),
        1,
        "get_finding must carry the annotation. It used to hardcode an empty vec, which is what \
         made every part of CAP-11 above this line unreachable"
    );

    let stored = &detail.visual_annotations[0];
    assert_eq!(stored.id, "a-1");
    assert_eq!(stored.finding_id, "f-1");
    assert_eq!(stored.created_at, "2026-08-28T09:01:00Z");
    assert_eq!(
        stored.data,
        rect(0.1, 0.2),
        "the shape must round-trip whole - every field, not just the geometry"
    );
}

/// `list_findings` is a separate read path and had a separate `vec![]`.
#[test]
fn the_list_read_path_carries_annotations_too() {
    let store = store_with_a_finding("f-2");
    store
        .add_annotation("f-2", "a-2", &rect(0.3, 0.3), "2026-08-28T09:02:00Z")
        .unwrap();

    let listed = store.list_findings().unwrap();
    let found = listed
        .iter()
        .find(|d| d.finding.id == "f-2")
        .expect("the Finding must be listed");

    assert_eq!(
        found.visual_annotations.len(),
        1,
        "list_findings had its own `visual_annotations: vec![]`. Fixing only get_finding leaves the \
         Editor's own load path - which lists - showing nothing"
    );
}

/// All five variants, because the JSON column is the only thing standing between the enum and disk.
#[test]
fn all_five_shapes_round_trip_through_the_json_column() {
    let store = store_with_a_finding("f-3");

    let shapes = vec![
        rect(0.05, 0.05),
        AnnotationShape::Blur {
            x: 0.1,
            y: 0.1,
            width: 0.3,
            height: 0.05,
            blur_radius: Some(12.0),
        },
        AnnotationShape::Arrow {
            start_x: 0.1,
            start_y: 0.9,
            end_x: 0.6,
            end_y: 0.4,
            color: Some("#dc2626".to_string()),
            stroke_width: Some(4.0),
        },
        AnnotationShape::Text {
            x: 0.2,
            y: 0.7,
            width: 0.25,
            height: 0.06,
            text: "look here".to_string(),
            font_size: Some(18.0),
            font_family: Some("IBM Plex Sans".to_string()),
            text_color: Some("#ffffff".to_string()),
            text_align: None,
        },
        AnnotationShape::Callout {
            x: 0.5,
            y: 0.1,
            width: 0.3,
            height: 0.12,
            tail_x: 0.4,
            tail_y: 0.35,
            text: "the button is disabled".to_string(),
            font_size: Some(14.0),
            font_family: Some("IBM Plex Sans".to_string()),
            bg_color: Some("#111827".to_string()),
            text_color: Some("#ffffff".to_string()),
            text_align: Some("center".to_string()),
        },
    ];

    for (i, shape) in shapes.iter().enumerate() {
        store
            .add_annotation("f-3", &format!("a-{i}"), shape, "2026-08-28T09:03:00Z")
            .unwrap_or_else(|e| panic!("shape {i} must store: {e}"));
    }

    let detail = store.get_finding("f-3").unwrap().unwrap();
    let stored: Vec<AnnotationShape> = detail
        .visual_annotations
        .iter()
        .map(|a| a.data.clone())
        .collect();

    assert_eq!(
        stored, shapes,
        "every variant must survive serialization, in the order it was drawn"
    );
}

/// Z-order is what the Reviewer saw. A later annotation covers an earlier one, and the burner walks
/// this order, so the read has to preserve it rather than fall back on the id.
#[test]
fn annotations_come_back_in_the_order_they_were_drawn() {
    let store = store_with_a_finding("f-4");

    // Ids deliberately in the reverse of the drawing order: an `ORDER BY id` would pass without a
    // `position` column at all, and this is the assertion that refuses it.
    for (id, x) in [("z-first", 0.1), ("m-second", 0.2), ("a-third", 0.3)] {
        store
            .add_annotation("f-4", id, &rect(x, 0.5), "2026-08-28T09:04:00Z")
            .unwrap();
    }

    let detail = store.get_finding("f-4").unwrap().unwrap();
    let ids: Vec<&str> = detail
        .visual_annotations
        .iter()
        .map(|a| a.id.as_str())
        .collect();

    assert_eq!(
        ids,
        vec!["z-first", "m-second", "a-third"],
        "drawing order is z-order, and the store owns it"
    );
}

#[test]
fn an_annotation_can_be_moved_and_the_move_is_what_is_stored() {
    let store = store_with_a_finding("f-5");
    store
        .add_annotation("f-5", "a-5", &rect(0.1, 0.1), "2026-08-28T09:05:00Z")
        .unwrap();

    let moved = rect(0.6, 0.6);
    let returned = store.update_annotation("f-5", "a-5", &moved).unwrap();

    assert_eq!(
        returned.created_at, "2026-08-28T09:05:00Z",
        "an update is not a creation: the drawing time must be read back, not re-stamped"
    );

    let detail = store.get_finding("f-5").unwrap().unwrap();
    assert_eq!(detail.visual_annotations.len(), 1, "a move is not a copy");
    assert_eq!(detail.visual_annotations[0].data, moved);
}

#[test]
fn deleting_an_annotation_leaves_the_others_alone() {
    let store = store_with_a_finding("f-6");
    for id in ["a", "b", "c"] {
        store
            .add_annotation("f-6", id, &rect(0.1, 0.1), "2026-08-28T09:06:00Z")
            .unwrap();
    }

    store.delete_annotation("f-6", "b").unwrap();

    let detail = store.get_finding("f-6").unwrap().unwrap();
    let ids: Vec<&str> = detail
        .visual_annotations
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["a", "c"],
        "and the survivors keep their relative order - z-order is an ordering, not a sequence, so \
         there is no gap to close"
    );
}

/// `finding` cascades to `note` and `marker`; `visual_annotation` has to be in that cascade too or
/// deleting a Finding leaves rows pointing at nothing - the orphan class `UC-12` exists to prevent.
#[test]
fn deleting_a_finding_takes_its_annotations_with_it() {
    let store = store_with_a_finding("f-7");
    store
        .add_annotation("f-7", "a-7", &rect(0.1, 0.1), "2026-08-28T09:07:00Z")
        .unwrap();

    store.delete_finding("f-7").unwrap();

    assert!(store.get_finding("f-7").unwrap().is_none());
    // Re-creating a Finding with the same id must not inherit the dead annotation.
    let store2 = store;
    let finding = Finding {
        id: "f-7".to_string(),
        image_path: "findings/f-7.png".to_string(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-28T09:08:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,800,600".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "note-f-7b".to_string(),
        finding_id: "f-7".to_string(),
        body: String::new(),
        updated_at: "2026-08-28T09:08:00Z".to_string(),
    };
    store2.create_finding(&finding, &note, &[]).unwrap();

    let detail = store2.get_finding("f-7").unwrap().unwrap();
    assert!(
        detail.visual_annotations.is_empty(),
        "the cascade must have taken it. A surviving row would attach a stranger's redaction box to \
         a new capture"
    );
}

#[test]
fn an_annotation_cannot_be_placed_on_a_finding_that_does_not_exist() {
    let store = store_with_a_finding("f-8");
    let err = store
        .add_annotation("nope", "a-8", &rect(0.1, 0.1), "2026-08-28T09:09:00Z")
        .expect_err("a Finding that does not exist must be refused");
    assert!(matches!(err, CoreError::NotFound(_)), "got {err:?}");
}

#[test]
fn updating_or_deleting_an_annotation_that_is_not_there_is_reported() {
    let store = store_with_a_finding("f-9");

    let err = store
        .update_annotation("f-9", "ghost", &rect(0.1, 0.1))
        .expect_err("an update must not silently create");
    assert!(matches!(err, CoreError::NotFound(_)), "got {err:?}");

    let err = store
        .delete_annotation("f-9", "ghost")
        .expect_err("a delete must not silently succeed");
    assert!(matches!(err, CoreError::NotFound(_)), "got {err:?}");
}

/// An annotation belongs to ONE Finding, and the id alone must not reach across.
#[test]
fn an_annotation_id_does_not_reach_across_findings() {
    let store = store_with_a_finding("f-10");
    let finding = Finding {
        id: "f-11".to_string(),
        image_path: "findings/f-11.png".to_string(),
        image_width: 800,
        image_height: 600,
        captured_at: "2026-08-28T09:10:00Z".to_string(),
        source_monitor: "DISPLAY1".to_string(),
        region: "0,0,800,600".to_string(),
        resolved_long_edge: None,
        resolved_encoder_quality: None,
        budget_name: None,
    };
    let note = Note {
        id: "note-f-11".to_string(),
        finding_id: "f-11".to_string(),
        body: String::new(),
        updated_at: "2026-08-28T09:10:00Z".to_string(),
    };
    store.create_finding(&finding, &note, &[]).unwrap();

    store
        .add_annotation("f-10", "shared-id", &rect(0.1, 0.1), "2026-08-28T09:10:00Z")
        .unwrap();

    assert!(
        store.delete_annotation("f-11", "shared-id").is_err(),
        "the other Finding must not be able to delete it"
    );
    assert_eq!(
        store
            .get_finding("f-10")
            .unwrap()
            .unwrap()
            .visual_annotations
            .len(),
        1,
        "and it must still be there"
    );
}

/// The coordinates are normalized to the image, exactly as a Marker's are. Out of range does not
/// fail at burn time - it draws off-canvas and is lost, and for a Blur that means the password stays
/// on the image.
#[test]
fn a_shape_outside_the_image_is_refused() {
    let store = store_with_a_finding("f-12");

    let cases: Vec<(&str, AnnotationShape)> = vec![
        ("x past the right edge", rect(1.4, 0.1)),
        ("negative y", rect(0.1, -0.2)),
        (
            "zero-width box - a mis-click, not a drawing",
            AnnotationShape::Rect {
                x: 0.1,
                y: 0.1,
                width: 0.0,
                height: 0.1,
                stroke_color: None,
                stroke_width: None,
            },
        ),
        (
            "NaN coordinate",
            AnnotationShape::Blur {
                x: f64::NAN,
                y: 0.1,
                width: 0.2,
                height: 0.2,
                blur_radius: None,
            },
        ),
        (
            "arrow ending off-canvas",
            AnnotationShape::Arrow {
                start_x: 0.1,
                start_y: 0.1,
                end_x: 2.0,
                end_y: 0.5,
                color: None,
                stroke_width: None,
            },
        ),
        (
            "callout whose tail points off-canvas",
            AnnotationShape::Callout {
                x: 0.1,
                y: 0.1,
                width: 0.2,
                height: 0.1,
                tail_x: -0.5,
                tail_y: 0.5,
                text: String::new(),
                font_size: None,
                font_family: None,
                bg_color: None,
                text_color: None,
                text_align: None,
            },
        ),
    ];

    for (why, shape) in cases {
        let err = store
            .add_annotation("f-12", "bad", &shape, "2026-08-28T09:11:00Z")
            .expect_err(&format!("{why} must be refused"));
        assert!(
            matches!(err, CoreError::Validation(_)),
            "{why}: expected a validation error, got {err:?}"
        );
    }

    // The same check guards an update, or a valid shape could be dragged off the image afterwards.
    store
        .add_annotation("f-12", "good", &rect(0.1, 0.1), "2026-08-28T09:11:00Z")
        .unwrap();
    assert!(
        store.update_annotation("f-12", "good", &rect(1.5, 0.1)).is_err(),
        "an update must be validated too: otherwise a shape is dragged off the image after the fact"
    );
}

/// Z-order can be rewritten, and it is the whole order or nothing.
#[test]
fn annotations_can_be_reordered() {
    let store = store_with_a_finding("f-13");
    for id in ["a", "b", "c"] {
        store
            .add_annotation("f-13", id, &rect(0.1, 0.1), "2026-08-28T12:00:00Z")
            .unwrap();
    }

    store.reorder_annotations("f-13", &["c", "a", "b"]).unwrap();

    let detail = store.get_finding("f-13").unwrap().unwrap();
    let ids: Vec<&str> = detail
        .visual_annotations
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["c", "a", "b"],
        "the read must come back in the order that was written - z-order IS the read order"
    );
}

/// A partial order would leave the annotations it omitted at positions nobody reasoned about.
#[test]
fn a_partial_or_foreign_reorder_is_refused() {
    let store = store_with_a_finding("f-14");
    for id in ["a", "b", "c"] {
        store
            .add_annotation("f-14", id, &rect(0.1, 0.1), "2026-08-28T12:01:00Z")
            .unwrap();
    }

    assert!(
        store.reorder_annotations("f-14", &["a", "b"]).is_err(),
        "a short list must be refused: the omitted annotation would keep a position the caller never \
         considered, and the result is an order nobody chose"
    );
    assert!(
        store
            .reorder_annotations("f-14", &["a", "b", "stranger"])
            .is_err(),
        "and an id from somewhere else must be refused rather than silently ignored"
    );

    let detail = store.get_finding("f-14").unwrap().unwrap();
    let ids: Vec<&str> = detail
        .visual_annotations
        .iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(
        ids,
        vec!["a", "b", "c"],
        "and a refused reorder must leave the order exactly as it was"
    );
}
