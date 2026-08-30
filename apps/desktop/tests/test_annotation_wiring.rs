//! `CAP-11` is REACHABLE, not merely built. `BUG-72`.
//!
//! The capability had a domain type, a burner and five passing burner tests, and no table, no port
//! method, no read and no canvas. Every one of those tests was green. `AGENTS.md` names this the
//! repository's signature failure and gives the check in plain words: *"before closing any story
//! that adds a component, grep for `<ComponentName` across `apps/desktop/src` and `web/ui/src`"*.
//!
//! This file is that grep, made permanent, plus the joins that a compile cannot check: that the
//! canvas mounts the component, that the burn is handed the annotations, and that the Properties
//! panel is gated to the two kinds that have words.

use std::fs;
use std::path::Path;

fn read(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {path:?}: {e}"))
}

/// The same source with every run of whitespace collapsed to one space.
///
/// Needed because `rustfmt` decides where a method chain breaks, and it broke
/// `ctx.finding_store.update_annotation(` across two lines the moment the receiver got longer. A
/// guard that a reformat can turn red is a guard nobody keeps.
fn flat(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The source with `//` comment lines removed.
///
/// A guard that asserts a string is ABSENT will otherwise match the comment explaining why it was
/// removed - which makes the comment unwritable, and an unexplained removal is how the next reader
/// puts it back.
fn code_only(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join(
            "
",
        )
}

/// The component exists AND something mounts it.
#[test]
fn the_annotation_component_is_mounted_on_the_canvas() {
    let window = read("ui/appwindow.slint");

    assert!(
        window.contains(
            r#"import { AnnotationItem, AnnotationData } from "components/annotation.slint";"#
        ),
        "the canvas must import the annotation component"
    );
    assert!(
        window.contains("for a in root.annotations : AnnotationItem {"),
        "`AnnotationItem` must be REPEATED over the model. A component that compiles and is mounted \
         nowhere is what `BUG-4`, `BUG-5`, `BUG-6` and `BUG-72` all were"
    );
    assert!(
        window.contains("in-out property <[AnnotationData]> annotations: []"),
        "and the model it repeats over must be a window property Rust can fill"
    );
}

/// A Finding's annotations reach the burn. This is the join `BUG-72` said "only needs real data" -
/// and it was passing an empty slice.
#[test]
fn the_bundle_burn_is_handed_the_annotations() {
    let main = read("src/main.rs");

    assert!(
        main.contains("MarkerBurner::burn_all("),
        "the Bundle's burn must be `burn_all`. `burn_markers` passes an empty annotation slice, so \
         every box, arrow, callout and - worst - every blur redaction is dropped from the copy the \
         agent actually receives"
    );
    assert!(
        !main.contains("MarkerBurner::burn_markers("),
        "and nothing in the app may still call `burn_markers`: a second call site is a second place \
         for a redaction to be lost"
    );

    let at = main
        .find("MarkerBurner::burn_all(")
        .expect("the burn call must exist");
    let call = &main[at..(at + 220).min(main.len())];
    assert!(
        call.contains("detail.visual_annotations"),
        "and it must be handed the Finding's OWN annotations, not an empty slice"
    );
}

/// Every port method the canvas needs is called from somewhere. A store method with no caller is
/// `BUG-61`'s whole shape.
#[test]
fn every_annotation_port_method_has_a_caller_in_the_app() {
    let main = flat(&read("src/main.rs"));
    for method in [
        "add_annotation(",
        "update_annotation(",
        "delete_annotation(",
    ] {
        assert!(
            main.contains(&format!(".finding_store .{method}"))
                || main.contains(&format!(".finding_store.{method}")),
            "`{method}` has no caller in the desktop app - it would be a port written for nobody"
        );
    }
}

/// `FR-33`: eight handles on a box, two on an Arrow, one on a Callout's tail, and Escape.
#[test]
fn the_transform_handles_and_their_keys_exist() {
    let annotation = read("ui/components/annotation.slint");
    let window = read("ui/appwindow.slint");

    assert!(
        annotation.contains("for i in [0, 1, 2, 3, 4, 5, 6, 7]"),
        "`FR-33` asks for eight bounding-box handles and the loop must produce eight of them"
    );
    assert!(
        annotation.contains("for end in [0, 1]"),
        "an Arrow gets two endpoint handles, not a bounding box - a box around a diagonal offers \
         corners that are not on the arrow at all"
    );
    assert!(
        annotation.contains(r#"if root.selected && root.ann.kind == "callout""#),
        "and a Callout gets its tail handle"
    );

    assert!(
        window.contains("canvas-keys := FocusScope"),
        "the canvas keys must live in their own FocusScope. A window-wide handler would take \
         Backspace away from every text field in the Editor"
    );
    let scope = window
        .find("canvas-keys := FocusScope")
        .expect("the scope must exist");
    let body = flat(&window[scope..(scope + 2200).min(window.len())]);
    assert!(
        body.contains("Key.Escape") && body.contains(r#"root.annotation-selected("")"#),
        "`FR-33`: pressing Escape deselects the active element"
    );
    assert!(
        body.contains("Key.Delete"),
        "`FR-30`: Delete removes the active element"
    );
    assert!(
        window.contains("canvas-keys.focus();"),
        "and something must GIVE it focus - a FocusScope nothing focuses receives no key at all"
    );
}

/// Backspace is a text key, and a Callout's words are typed on the canvas.
///
/// `FR-30` names Delete AND Backspace, and this deliberately implements only the first. Once the
/// Callout became editable in place, Backspace inside one is a person deleting a letter - and a
/// single keystroke landing outside the caret would take the whole annotation instead. The owner
/// asked for it directly: *"Tombol backspace jangan mendelete element"*.
#[test]
fn backspace_does_not_delete_an_annotation() {
    let window = read("ui/appwindow.slint");
    let scope = window
        .find("canvas-keys := FocusScope")
        .expect("the scope must exist");
    let body = flat(&window[scope..(scope + 2200).min(window.len())]);

    assert!(
        !body.contains("Key.Backspace"),
        "Backspace must not be bound to deletion. It is the key that removes a LETTER from the \
         Callout the Reviewer is typing into"
    );
}

/// The owner scoped the Properties panel twice: to two kinds, and to one place.
///
/// It has now moved twice too - off the canvas, where it covered the capture it described, and then
/// into the tab already called Properties. That tab held a placeholder: a font field bound to
/// nothing and a font size that was a `Text`. This asserts the real one replaced it.
#[test]
fn the_properties_panel_lives_in_the_properties_tab_and_only_for_text_and_callout() {
    let window = read("ui/appwindow.slint");

    let at = window
        .find("// THE PROPERTIES TAB")
        .expect("the Properties tab must hold the real controls - `FR-32` asks for font controls");
    let panel = flat(&window[at..(at + 20000).min(window.len())]);

    assert!(
        panel.contains("if root.selected-has-words : VerticalLayout"),
        "the controls must be gated to the two kinds that carry words. A Shape, an Arrow and a Blur \
         would get three controls that change nothing"
    );

    // Size, family, alignment - in that order, size first.
    let size_at = panel.find(r#"text: "Size""#).expect("a Size control");
    let style_at = panel.find(r#"text: "Style""#).expect("a Style control");
    let align_at = panel.find(r#"text: "Align""#).expect("an Align control");
    assert!(
        size_at < style_at && style_at < align_at,
        "size, then family, then alignment"
    );

    // The placeholder is gone, both halves of it.
    assert!(
        !window.contains(r#"text: "IBM Plex Sans, sans-serif";"#),
        "the pre-filled font field bound to nothing must be gone - it looked like a working control"
    );
    assert!(
        !window.contains(r#"text: "Font Size: 14px""#),
        "and so must the font size that was a label rather than a control"
    );

    // The WORDS are not here. They are typed on the shape.
    assert!(
        !panel.contains("root.annotation-text-edited("),
        "the text is typed on the canvas now, so a field here would be a second place to type one \
         string"
    );

    // And picking one up opens the tab that can change it.
    assert!(
        flat(&window).contains("root.active-tab-index = 1;"),
        "selecting a Text or Callout must open the Properties tab, or the controls sit behind a tab \
         nobody has a reason to click"
    );
}

/// The canvas is a preview of the burn, so the two must agree about the colour they share.
#[test]
fn the_canvas_stroke_is_the_same_red_the_burner_uses() {
    let theme = read("ui/theme.slint");
    let burner = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/snapdown-store/src/image/burner.rs"),
    )
    .expect("the burner must be readable from here");

    assert!(
        theme.contains("out property <color> annotation-stroke: #dc2626;"),
        "the canvas stroke token must be #dc2626"
    );
    assert!(
        burner.contains("const COLOR_MARKER_FILL: Rgba<u8> = Rgba([220, 38, 38, 255]);"),
        "and the burner must draw in the same value - 220,38,38 IS #dc2626. Two reds would mean the \
         canvas is not a preview of the file"
    );

    // ONE colour, for every shape and for the words in them. The Callout's dark plate and white
    // text are gone from both sides, and neither may come back on one side alone.
    for gone in ["annotation-callout-bg", "annotation-callout-text"] {
        assert!(
            !theme.contains(gone),
            "`{gone}` belonged to the filled Callout plate and must be gone with it"
        );
    }
    assert!(
        !burner.contains("COLOR_CALLOUT_BG"),
        "and the burner must not still carry the plate colour"
    );
}

/// A click is not a drawing, and there is no default size.
#[test]
fn a_gesture_that_is_not_a_drag_draws_nothing() {
    let window = read("ui/appwindow.slint");
    let flat_window = flat(&window);

    assert!(
        flat_window.contains("private property <bool> drag-is-real:"),
        "a real drag must be a named condition, not an inline number in one branch"
    );
    assert!(
        flat_window
            .contains("if (root.active-tool-kind == \"\" || !root.drag-is-real) { return; }"),
        "a gesture that never became a drag must draw NOTHING. It used to invent a default-sized \
         shape, and the owner rejected it: an annotation names a region, and a tap names none"
    );
}

/// A Callout's tail names a place on the screenshot. Moving the bubble must not re-aim it.
#[test]
fn moving_a_callout_leaves_its_tail_where_it_was_pointing() {
    let main = read("src/main.rs");
    let at = main
        .find("fn with_geometry(")
        .expect("the geometry rewrite must exist");
    let body = flat(&main[at..(at + 3000).min(main.len())]);

    assert!(
        body.contains("tail_x: *tail_x, tail_y: *tail_y,"),
        "the tail must be carried through UNCHANGED. It used to be offset by the bubble's own \
         movement, which re-pointed it at whatever had ended up underneath"
    );
}

/// A Marker can be moved after it is placed, and moving it is not a delete-and-replace.
#[test]
fn a_marker_can_be_dragged_after_it_is_placed() {
    let window = read("ui/appwindow.slint");
    let main = read("src/main.rs");

    assert!(
        window.contains("root.marker-moved(m.id,"),
        "the reticle must report a drag"
    );
    assert!(
        flat(&main).contains("main_window.on_marker_moved("),
        "and Rust must handle it"
    );

    let at = main
        .find("on_marker_moved(")
        .expect("the handler must exist");
    let body = flat(&main[at..(at + 1400).min(main.len())]);
    assert!(
        body.contains(".update_marker("),
        "a move is an UPDATE. Delete-and-re-place would renumber every Marker after it, so \
         correcting one Marker's position would rewrite its line number in the Note"
    );
}

/// The blur the Reviewer approves is the blur that lands in the file.
#[test]
fn the_canvas_previews_a_blur_with_the_burn_own_code() {
    let main = read("src/main.rs");
    let annotation = read("ui/components/annotation.slint");

    assert!(
        flat(&main).contains("MarkerBurner::blur_rect(&mut blurred,"),
        "the canvas preview must be produced by the BURN's own blur, not by a second approximation \
         of it. A redaction preview that differs from the output is the one preview that can get \
         somebody hurt, because the Reviewer approves what they see"
    );
    assert!(
        annotation.contains("source: root.blurred-source;"),
        "and the canvas must draw those pixels rather than a stand-in. It used to paint an opaque \
         grey box, which told the Reviewer nothing about what the file would hold"
    );
    assert!(
        annotation.contains("clip: true;"),
        "a Blur is a clipped WINDOW onto the pre-blurred capture, at 1:1. Stretching a crop is what \
         made a drag show the previous region's pixels while a new one was being sized"
    );
}

/// The canvas's blur default and the burn's must be the same number.
///
/// They are written twice on purpose - the store owns its default, and the canvas must not become a
/// caller of it - so this is the check that the copy has not drifted. A drift here does not fail
/// anything at runtime; it just makes the preview quietly wrong, which is the failure mode this
/// whole pairing exists to prevent.
#[test]
fn the_canvas_blur_matches_the_burn_default() {
    let main = flat(&read("src/main.rs"));
    let burner = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/snapdown-store/src/image/burner.rs"),
    )
    .expect("the burner must be readable from here");
    let burner = flat(&burner);

    assert!(
        burner.contains("(short / 14.0).clamp(3.0, 16.0)"),
        "the burn's default radius must still be a fourteenth of the shorter side, 3 to 16"
    );
    assert!(
        main.contains("(short / 14.0).clamp(3.0, 16.0) as i32"),
        "and the canvas must use the same one, or the preview softens by a different amount than \
         the file does"
    );
}

/// An Arrow is grabbed on the arrow, not on a rectangle around it.
#[test]
fn an_arrow_is_grabbed_along_its_own_line() {
    let annotation = read("ui/components/annotation.slint");

    assert!(
        annotation.contains("for f in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]"),
        "the arrow's grab areas must be stepped along the segment by FRACTION - spaced by pixels \
         they would leave gaps on a long arrow and pile up on a short one"
    );
    assert!(
        !annotation.contains("root.is-box ? root.gx :"),
        "and the bounding-box hit area must be gone. Over a diagonal it was mostly empty canvas, \
         which the Reviewer could not click through to draw underneath"
    );

    // The head is the burn's head, not a second one.
    assert!(
        annotation.contains("153deg"),
        "the canvas head must leave the tip at the same +/-153 degrees `draw_arrow` uses. A \
         different head is a canvas that disagrees with the file about which way an arrow points"
    );
}

/// Single click moves; double-click types. Both, and in that order.
#[test]
fn a_text_bearing_shape_is_moved_by_a_click_and_typed_by_a_double_click() {
    let annotation = read("ui/components/annotation.slint");

    assert!(
        annotation.contains("double-clicked => {") && annotation.contains("root.editing = true;"),
        "double-click must open the words: *\"double klik mentrigger mengubah teks\"*"
    );
    assert!(
        annotation.contains("if root.has-words && root.editing : TextInput"),
        "and the field must exist ONLY while editing. A field permanently over the shape takes the \
         click that should have moved it, which is why the Callout could not be dragged"
    );
    assert!(
        annotation.contains("init => { self.focus(); }"),
        "it must take focus as it appears - it does not exist before the double-click, so nothing \
         can focus it beforehand"
    );
    assert!(
        annotation.contains("changed selected => {"),
        "and deselecting must leave editing, or a shape stays in text mode after the Reviewer has \
         moved on"
    );
}

/// A click that moved nothing writes nothing - and that is what makes double-click reachable.
///
/// Every press began a drag and every release committed it, so a plain click wrote the same geometry
/// back and Rust reloaded the Finding. The reload replaced the model, the repeater rebuilt, and the
/// `AnnotationItem` that was about to receive the SECOND click of a double-click had already been
/// destroyed. Typing into a Callout was unreachable for that reason alone.
#[test]
fn a_click_that_moved_nothing_does_not_write() {
    let annotation = flat(&read("ui/components/annotation.slint"));

    assert!(
        annotation.contains("if (root.d-x1 != root.ann.x1 || root.d-y1 != root.ann.y1"),
        "commit-drag must compare against the committed geometry before firing. Without it every \
         click on the canvas costs a SQLite write and a full model rebuild - and destroys the \
         element mid-gesture"
    );
    assert!(
        annotation.contains("if (root.d-tx != root.ann.tail-x || root.d-ty != root.ann.tail-y)"),
        "and the tail handle needs the same guard for the same reason"
    );
}

/// `FR-37`, and the rule it sets itself: no entry with no other route.
#[test]
fn every_context_menu_entry_has_another_route() {
    let window = read("ui/appwindow.slint");
    let flat_window = flat(&window);

    assert!(
        window.contains("if root.menu-subject != \"\" : SdContextMenu {"),
        "the menu must be mounted"
    );
    assert!(
        window.contains("root.open-menu(\"annotation\"")
            || flat_window.contains("root.open-menu(\"annotation\""),
        "the canvas must raise it"
    );
    assert!(
        flat_window.contains("root.open-menu(\"finding\", thumb.id,"),
        "and so must a filmstrip card - the owner asked for both surfaces"
    );

    // The four ordering actions are also buttons in the Properties tab, which is what stops the
    // menu being the only way to reach them.
    for action in ["front", "forward", "backward", "back"] {
        let in_menu = flat_window.contains(&format!("action: \"{action}\", label:"));
        let in_panel = flat_window.contains(&format!("{{ action: \"{action}\", label:"));
        assert!(
            in_menu && in_panel,
            "`{action}` must appear both on the menu and in the Properties tab. `FR-37`: a context \
             menu is a shortcut to what is already reachable, never the only way to reach it"
        );
    }

    // Every other entry, and the second route it has - or the reason it has none yet.
    //
    // `FR-37`'s own consequence is that no entry exists with no other route, and two of these do not
    // satisfy it. They are listed rather than quietly excused, in the shape `DELIBERATELY_UNHANDLED`
    // uses in `test_ui_callbacks_reach_rust.rs`: a gap that is written down is a gap somebody can
    // close, and one that is asserted away is a gap that stops existing on paper only.
    let routes: &[(&str, &str)] = &[
        ("assemble", "the Assemble tile beside the filmstrip"),
        ("copy", "the Copy button on the toolbar"),
        ("delete", "the Delete key, and the Properties tab's button"),
        ("delete-marker", "the Marker Notes list's own delete button"),
        ("undo", "the Undo button on the toolbar"),
        ("redo", "the Redo button on the toolbar"),
    ];
    for (action, _route) in routes {
        assert!(
            flat_window.contains(&format!("action: \"{action}\", label:")),
            "`{action}` must be on a menu"
        );
    }

    // MENU-ONLY, and recorded as such. Both arrived with the menu and have no other home yet:
    // `reveal` shows the file in Explorer, `delete-finding` is `FR-13`/`UC-7`'s deletion. `FR-13` is
    // a promise, so a deletion reachable only by right-click is a real gap - `BUG-76`.
    for menu_only in ["reveal", "delete-finding"] {
        assert!(
            flat_window.contains(&format!("action: \"{menu_only}\", label:")),
            "`{menu_only}` must be on the filmstrip menu"
        );
    }
}

/// The screenshot is the bottom element and is not in the order.
#[test]
fn the_ordering_actions_move_only_annotations() {
    let main = read("src/main.rs");
    let at = main
        .find("fn reordered(")
        .expect("the reorder arithmetic must exist");
    let body = flat(&main[at..(at + 1600).min(main.len())]);

    assert!(
        body.contains(r#""front" => last"#) && body.contains(r#""back" => 0"#),
        "front is the end of the list and back is the start - later is drawn later"
    );
    assert!(
        body.contains("if to == from { return None; }"),
        "a movement that changes nothing must write nothing, or the first element sent backward \
         costs a transaction and a reload to store the order it already had"
    );
}

/// Size is a slider and a number, and the panel has one heading.
#[test]
fn the_size_control_is_a_slider_and_a_field_under_one_heading() {
    let window = read("ui/appwindow.slint");
    let flat_window = flat(&window);

    assert!(
        flat_window.contains("SdSlider { horizontal-stretch: 1;"),
        "size must be a slider - a five-step ladder could not answer \"a bit bigger than that\""
    );
    assert!(
        flat_window.contains("size-field := SdTextField"),
        "and a field, for \"18, the same as the last one\""
    );
    assert!(
        window.contains("self.text.to-float()"),
        "the field must actually parse what is typed"
    );

    // One heading, and it does not rename itself.
    let code = code_only(&window);
    for gone in ["CALLOUT PROPERTIES", "TEXT PROPERTIES"] {
        assert!(
            !code.contains(gone),
            "`{gone}` must be gone: the panel is named for what it is, not for what is in it, and \
             there were two copies of the label"
        );
    }
    assert_eq!(
        window.matches(r#"text: "ELEMENT PROPERTIES";"#).count(),
        1,
        "exactly one heading"
    );
}

/// A Text has room for the line it is meant to hold.
#[test]
fn a_text_annotation_is_at_least_one_line_tall() {
    let window = flat(&read("ui/appwindow.slint"));

    assert!(
        window.contains("private property <float> text-min-height:"),
        "the minimum must be derived from the font size and the image height - a 6px-tall drag \
         cannot render an 18px line, and the words would be clipped to nothing"
    );
    assert!(
        window.contains(r#"if (root.active-tool-kind == "text") {"#),
        "and the Text tool must apply it when the drag commits"
    );
}

/// A Callout points where the Reviewer pressed, whichever way they dragged.
#[test]
fn the_callout_bubble_is_inset_from_the_press_point() {
    let main = read("src/main.rs");
    let at = main
        .find("fn shape_from_drag(")
        .expect("the constructor must exist");
    let body = flat(&main[at..(at + 3000).min(main.len())]);

    assert!(
        body.contains("x: (x1 + (x2 - x1) * 0.3).min(x2),"),
        "the inset must carry the drag's SIGN. `min + 0.3 * (max - min)` measures from the smaller \
         coordinate, so dragging upward put the bubble's far edge exactly on the tail and the arrow \
         came out flush with the bubble's own bottom line"
    );
}

/// Deleting from the filmstrip takes the selection, the way every file manager does.
#[test]
fn deleting_a_selected_finding_takes_the_whole_selection() {
    let main = read("src/main.rs");
    let window = flat(&read("ui/appwindow.slint"));

    let at = main
        .find("fn findings_to_delete(")
        .expect("the target set must be worked out in one named place");
    let body = flat(&main[at..(at + 900).min(main.len())]);
    assert!(
        body.contains("if selected.iter().any(|id| id == target) { selected }"),
        "a right-click ON the selection must take all of it"
    );
    assert!(
        body.contains("vec![target.to_string()]"),
        "and a right-click OUTSIDE it must take only that one - otherwise a right-click on an          unselected card quietly deletes eight others"
    );

    // The dialog has to say how many, or the confirmation is confirming something invisible.
    assert!(
        window.contains("root.pending-delete-count"),
        "the confirmation must count what it is about to delete"
    );
}

/// Explorer's `/select` needs the argument passed through untouched.
#[test]
fn open_file_location_passes_explorer_a_switch_it_can_parse() {
    let main = read("src/main.rs");

    assert!(
        main.contains(".raw_arg(format!(\"/select,\\\"{native}\\\"\"))"),
        "`.arg()` wraps the WHOLE argument in quotes when it contains a comma, which Explorer          cannot parse - and its documented response is to open the default folder instead. That is          why the owner got their Desktop"
    );
    assert!(
        main.contains(r#"replace('/', "\\")"#),
        "and the separators must be native: `image_path` is a Vault-relative key with forward          slashes, so joining it yields a mixed path Explorer also rejects"
    );
}

/// The Settings screen is a WIRE to what already existed, not a second copy of it.
///
/// `settings-clicked` was a `println!` for the whole life of the Slint Editor (`BUG-57`), while the
/// startup registrar, the hotkey registrar, the Quality Budget domain type and the vault path
/// setting all sat built and unreachable. This asserts the screen reads from them rather than from
/// state of its own - a preferences screen with its own copy of a preference is `BUG-45`.
#[test]
fn the_settings_screen_reads_from_the_registrars_and_the_store() {
    let main = flat(&read("src/main.rs"));

    assert!(
        main.contains("fn load_settings_into_window("),
        "one loader, called on open and after every change, so nothing on the screen is a copy that \
         can drift from what the product is using"
    );
    for source in [
        "startup.is_enabled()",
        "open_editor_after_capture(ctx)",
        "configured_vault_path(ctx).display()",
        "current_budget(ctx)",
        "hotkeys.get_bindings()",
        "hotkeys.get_startup_failures()",
    ] {
        assert!(
            main.contains(source),
            "`{source}` must feed the screen - that value already existed and had no surface"
        );
    }

    // Every control writes through to the thing that owns it.
    for (callback, target) in [
        ("on_startup_toggled", "startup.enable()"),
        (
            "on_open_editor_after_capture_toggled",
            "SettingKey::OpenEditorAfterCapture",
        ),
        ("on_budget_chosen", "store_budget("),
        ("on_hotkey_key_pressed", "validate_and_rebind("),
        ("on_hotkey_cleared", "hotkeys.clear(target)"),
    ] {
        assert!(
            main.contains(callback) && main.contains(target),
            "`{callback}` must reach `{target}`"
        );
    }

    // A hand-set pair is Custom, because `Auto` resolves a different pair per capture and the two
    // cannot both be true.
    assert!(
        main.contains("QualityBudget::new(NamedBudget::Custom, Some(pair))"),
        "setting a number by hand must make the budget Custom, or the screen shows a preset name \
         over numbers that preset does not use"
    );

    // A bare modifier must not bind.
    let at = main
        .find("fn shortcut_from_key(")
        .expect("the shortcut composer must exist");
    let body = &main[at..(at + 1600).min(main.len())];
    assert!(
        body.contains("return Err(ShortcutRefusal::NoModifier);"),
        "a shortcut with no modifier would swallow a plain letter globally, in every application -          and it must be REFUSED WITH A REASON rather than silently dropped, which is the whole point          of the taxonomy borrowed from `wira-desk`"
    );
    assert!(
        body.contains("Ok(None)"),
        "and a bare modifier must be reported as mid-gesture, not as an error. The Reviewer holding          Ctrl on the way to Ctrl+Shift+S is not doing anything wrong"
    );
    assert!(
        body.contains("CommandOrControl"),
        "the composed string must match `DEFAULT_HOTKEY_CAPTURE`'s own format, or a rebind and a \
         default read differently in the database"
    );
}

/// The Slint attribution is finally in the product. `NTL-1`.
#[test]
fn the_about_tab_carries_the_slint_attribution() {
    let settings = read("ui/components/settings.slint");

    assert!(
        settings.contains("https://slint.dev"),
        "Slint's Royalty-free licence requires this acknowledgement IN the product. It was recorded \
         as owed in `NTL-1` and never paid"
    );
    assert!(
        settings.contains("Royalty-free"),
        "and it has to name the licence, not just link the site"
    );
    assert!(
        settings.contains("SIL Open Font License"),
        "IBM Plex ships in the binary too"
    );
}

/// `BUG-63` is closed: the encoder quality lever does something.
///
/// It was stored and read by nothing for the life of the product, because the obvious reading of it -
/// a JPEG-style quality dial - does not exist in PNG. What it does instead is round colour and then
/// store the capture as a PALETTE when it fits in 256, which on the measured fixture is 26% of the
/// lossless size for a per-channel error of one.
#[test]
fn the_encoder_quality_lever_actually_encodes() {
    let pipeline = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/snapdown-store/src/image/pipeline.rs"),
    )
    .expect("the pipeline must be readable from here");
    let flat_pipeline = flat(&pipeline);

    assert!(
        flat_pipeline.contains(
            "pub(crate) fn encode_png( image: &RgbaImage, width: u32, height: u32, quality: u8,"
        ),
        "`encode_png` must TAKE the quality. It did not, which is the whole of `BUG-63`"
    );
    assert!(
        pipeline.contains("fn bits_for_quality(quality: u8) -> u32"),
        "the quality must map to something concrete - a bit depth - rather than being stored and          ignored"
    );
    assert!(
        pipeline.contains("fn encode_png_indexed("),
        "and the palette path is where the size actually goes: one byte per pixel instead of three"
    );
    assert!(
        flat_pipeline.contains("if quality >= 100 { return encode_png_lossless("),
        "100 must stay lossless. Nothing may be thrown away that the Reviewer did not ask to have          thrown away"
    );

    // The Finding's own quality reaches the burn, or the handoff is larger than the original.
    let main = flat(&read("src/main.rs"));
    assert!(
        main.contains("f.resolved_encoder_quality .unwrap_or(snapdown_store::image::LOSSLESS)")
            || main
                .contains("f.resolved_encoder_quality.unwrap_or(snapdown_store::image::LOSSLESS)"),
        "the Bundle's burn must use the FINDING's quality, not a fresh choice"
    );

    // And the screen no longer says it is dead.
    let settings = read("ui/components/settings.slint");
    assert!(
        !settings.contains("Stored and not applied."),
        "the disclaimer must be gone with the defect"
    );
    assert!(
        !settings.contains("encoder-quality-live"),
        "and so must the flag that marked it inert"
    );
}

/// The resize ratio is a second tool beside the cap, not a replacement for it.
#[test]
fn the_resize_ratio_applies_before_the_cap() {
    let image = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/snapdown-core/src/domain/image.rs"),
    )
    .expect("the image domain must be readable from here");
    let flat_image = flat(&image);

    assert!(
        flat_image.contains("scaled.compute_reduced_dimensions_with_edge(pair.max_long_edge)"),
        "the ratio must be applied FIRST and the cap to the result. The other order takes a fifth          off the CAP rather than off the capture, so a 4K and a 2K screen come out the same size and          the ratio stops being a ratio"
    );
    assert!(
        flat_image.contains("if pair.resize_percent >= 100 { self.clone() }"),
        "100 must change nothing, so every Finding already reduced under the cap alone is unaffected"
    );

    let settings = read("ui/components/settings.slint");
    assert!(
        settings.contains("text: \"Resize every capture to\";"),
        "and it needs a control of its own - the owner asked for a percentage, not a cap"
    );
}
