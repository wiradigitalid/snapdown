use std::fs;
use std::path::Path;

/// BUG-25: Escape did nothing while the capture overlay was open - there was no key handler at
/// all, even though the note popup's own hint text promised "Enter to save · Esc to cancel".
#[test]
fn capture_overlay_window_binds_escape_to_cancel() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let slint_path = Path::new(manifest_dir).join("ui/appwindow.slint");
    let source = fs::read_to_string(&slint_path)
        .unwrap_or_else(|e| panic!("Failed to read {slint_path:?}: {e}"));

    let start = source
        .find("component CaptureOverlayWindow")
        .expect("CaptureOverlayWindow component not found in appwindow.slint");
    let end = source[start..]
        .find("\nexport component AppWindow")
        .map(|rel| start + rel)
        .unwrap_or(source.len());
    let overlay_block = &source[start..end];

    assert!(
        overlay_block.contains("FocusScope"),
        "CaptureOverlayWindow must contain a FocusScope to receive key events at all (BUG-25)"
    );
    assert!(
        overlay_block.contains("Key.Escape") && overlay_block.contains("root.overlay-cancelled()"),
        "CaptureOverlayWindow must map Key.Escape to overlay-cancelled() (BUG-25)"
    );
    assert!(
        overlay_block.contains("self.focus()"),
        "the overlay's FocusScope must claim keyboard focus itself - a top-level window does not \
         get it for free, so Escape would still never arrive without this (BUG-25)"
    );
}
