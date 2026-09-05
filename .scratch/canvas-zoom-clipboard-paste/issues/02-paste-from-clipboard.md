# 02: Paste an image from the Windows clipboard as a new Finding

**What to build:** Pressing the toolbar's Paste button — today a dead stub whose `paste-clicked()`
callback does nothing — reads whatever image is on the Windows clipboard, reduces it under the same
Quality Budget a Capture or an Import would use (by calling into the very same `persist_finding`
function those already call, never a second reduction path), stores it as a new Finding with an empty
Note, and shows it selected at the top of the filmstrip. Pressing Paste with no image on the clipboard
tells the Reviewer clearly, rather than doing nothing.

**Blocked by:** None (can start immediately). Shares no code path with ticket 01 — the two may proceed
in either order or in parallel.

**Status:** done — already implemented and merged to `main` at `d97b82e` (2026-09-04, "feat(finding): paste
an image from the Windows clipboard as a new Finding (FR-35)"), before this `DEC-023` run began. Verified
in this worktree on 2026-09-05: every acceptance criterion below holds against the code exactly as
written, `paste-clicked` is not in `DELIBERATELY_UNHANDLED`, and `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace --no-fail-fast` are
all green. No code changed for this ticket; only this status line.

Realizes `FR-35`. A second entry point into `UC-1` (`no_uc` — the flow from pixel-arrival onward is
UC-1's own, unchanged). See `.scratch/canvas-zoom-clipboard-paste/spec.md` for the full design
(Implementation Decisions § "Paste").

## Seam

`decode_clipboard_image_bytes(bmp: &[u8]) -> Result<(Vec<u8>, u32, u32), String>` — a new,
`#[cfg(any(windows, test))]` testable function decoding already-fetched clipboard bytes to RGBA8,
mirroring the existing `encode_region_for_clipboard` split (`main.rs:4003-4032`). A new
`#[cfg(windows)]` `paste_clipboard_image(ctx: &AppContext) -> Result<String, String>` does the real
clipboard read (`clipboard_win::raw::is_format_avail(formats::CF_BITMAP)` guard, then
`clipboard_win::get_clipboard::<Vec<u8>, _>(formats::Bitmap)`) and calls the existing `persist_finding`
exactly as `on_open_file_clicked` (Import, `main.rs:6566`) already does.

## Acceptance criteria

- [ ] `decode_clipboard_image_bytes` exists, is unit-tested with fabricated BMP bytes (never the real
      OS clipboard) for both a valid image (asserts decoded dimensions/pixels) and garbage bytes
      (asserts a clean `Err`, no panic) — prior art: the existing `encode_region_for_clipboard` test
      near `main.rs:7217`.
- [ ] `paste_clipboard_image` checks the clipboard actually holds `CF_BITMAP` first and returns a
      specific, clear `Err` if it does not (no silent no-op) — covered by a test that stages "no image
      available" at the seam just below the real OS clipboard call and asserts both the error message
      and that no Finding is created.
- [ ] On a successful paste, `persist_finding` is called with the whole decoded image as the region
      (`(0,0,w,h)`) and a Paste-specific label (parallel to Import's `"Imported: {label}"`, e.g.
      `"Pasted"`) — never a second, independently-written reduction/encode path. An integration test
      (using the same `AppContext`/store test harness every other persistence test in `main.rs` already
      uses) injects fabricated clipboard bytes at the `decode_clipboard_image_bytes` seam and asserts:
      exactly one new Finding exists afterward, carrying those pixels; its stored image's long edge
      fits the active Quality Budget (same assertion shape as the existing `NFR-3`/`NFR-18` reduction
      tests — decode the actual output image, never assert a signature-plus-size); its Note is empty.
- [ ] `main_window.on_paste_clicked(...)` is registered in `main.rs`, calling `paste_clipboard_image`
      and, on `Ok`, `load_findings_into_window(&win, &ctx, Some(&finding_id))` (the same filmstrip
      refresh-and-select every other Finding-creating path already uses) plus a success `toast`; on
      `Err`, an error `toast` naming what went wrong. The handler body is real work, not a `println!`
      stub.
- [ ] `paste-clicked`'s entry is deleted from `DELIBERATELY_UNHANDLED` in
      `apps/desktop/tests/test_ui_callbacks_reach_rust.rs` (currently lines ~64-67) in the same commit
      that wires the handler, and that file's existing generic reachability test now passes for it with
      no other change to that test file.
- [ ] A `#[cfg(not(windows))]` fallback for `paste_clipboard_image` returns a fixed
      "implemented on Windows only" error, mirroring the existing `copy_burned_image`/
      `copy_region_to_clipboard` non-Windows stubs.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` are all green, including every new test above actually
      seen to fail first for the reason it's meant to catch, then pass.
