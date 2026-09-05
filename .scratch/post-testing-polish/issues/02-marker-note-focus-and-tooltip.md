# 02: Auto-focus a Marker's Note field, and a hover tooltip previewing it

**What to build:** The moment a Marker is placed, or an existing Marker is clicked open, the Note field
gets real keyboard focus — no extra click needed (`UC-5` step 3: "the Reviewer types the sub-comment
for line 1" right after placing badge 1). Dragging a Marker (`UC-5` step 5) is a different gesture and
must NOT trigger this focus. Separately, hovering any Marker shows a tooltip previewing its Note text;
an empty note shows no tooltip or an explicit empty-state one (builder's choice, but never stale text
left over from a previously-hovered Marker).

**Blocked by:** None (can start immediately). Independent of ticket 01 (zoom) — different area of the
canvas/marker code, no shared files expected beyond `appwindow.slint` itself (large file, low collision
risk on serial merge).

**Status:** ready-for-agent

Realizes `FR-8`, `UC-5`. See `.scratch/post-testing-polish/spec.md` Implementation Decisions §
"Marker note auto-focus and hover tooltip" for the full design.

## Seam

Structural focus-claim test copying `library.slint`'s existing pattern (`library-keys := FocusScope {
init => { self.focus(); } }`) — assert the Note field's `FocusScope`/text-input claims focus in both
the place-Marker and click-Marker handlers, not a literal string match. Tooltip: assert it reads the
hovered Marker's own Note text (decode the actual bound property, not a copy of test input) and clears
when hovering a different Marker or none.

## Acceptance

- [ ] Placing a new Marker focuses its Note field immediately
- [ ] Clicking an existing Marker to open it focuses its Note field immediately
- [ ] Dragging a Marker does NOT trigger focus (unchanged from today)
- [ ] Hovering a Marker shows a tooltip with its Note text; an empty note shows no tooltip or an
      explicit empty state, never stale text from a different Marker
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
      `cargo test --workspace --no-fail-fast` all exit 0
- [ ] **Look at:** place a Marker and type immediately with no click; open an existing Marker and type
      immediately; hover several Markers in sequence and confirm the tooltip always matches the one
      under the pointer
