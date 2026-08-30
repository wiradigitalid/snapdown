# 01: Copy the selected region to the clipboard without saving it

**Status:** implemented, awaiting a run in the real app

**What to build:** A second way out of the capture overlay that puts the selected region on the
clipboard and writes **nothing** — no PNG in the Vault, no `Finding` row in `library.db`, no `Note`.
`Enter` keeps its current meaning (save a Finding); `Ctrl+C` and `Ctrl+Enter` mean copy-and-discard.

The point is not tidiness. A screenshot that never touches disk cannot be recovered, backed up, or
carried through a Vault migration — so this is the path for a shot whose contents should not persist,
and for the pasted-into-Slack-once case where a Finding is pure overhead.

## Decisions already settled by the owner

Do not re-open these.

1. **Chords:** `Ctrl+C` primary, `Ctrl+Enter` alias.
   - `Ctrl+C` copies the **text selection** when the note field has one, and the **image** otherwise.
     An empty field is just the no-selection case; do not check it separately.
   - `Ctrl+Enter` **always** copies the image, selection or not. Deliberate: it is the unconditional
     escape hatch that makes `Ctrl+C`'s conditional safe.
   - Copying text leaves the overlay **open** — the Reviewer is still writing the note. Only the
     image path closes it. One chord, two window outcomes; get this wrong and a text copy throws the
     note away.
2. **The Quality Budget applies**, the same as on the `Enter` path. Reversed from an earlier
   decision to skip it, on the owner's *"ikuti juga quality budget, agar aman"*. Two things fall out,
   both wanted: the clipboard image now matches what the same capture would have looked like as a
   Finding, and the payload is bounded by the resolved long edge instead of by the selection.
3. **A typed note is discarded**, silently, on the image path. `Enter` and `Ctrl+Enter` are therefore
   deliberately different behaviours over the same typed state, not two spellings of one action.
4. **Volatility is accepted.** No hidden backup, no temp file, no ring buffer. The next clipboard
   write destroys it and that is the feature.
5. **No annotations, no blur, no redaction on this path.** There is no `Finding`, so there are no
   Markers to burn. A shot needing redaction must go through `Enter`.

## The focus trap — read this before writing any key handling

`appwindow.slint:1013` gives the note field focus the moment the note panel appears
(`init => self.focus-input()`), so in the normal flow **the note field always has keyboard focus by
the time the Reviewer reaches for a chord.** Handling these chords only in the overlay's
`key-handler` FocusScope does not work: `Ctrl+C` is consumed by `TextInput` as a text copy, and
`Ctrl+Enter` meets `commit-on-enter: true`, which would **save a Finding** — the opposite of what was
asked. This is the same shape as the historical "Enter went nowhere" defect documented at
`appwindow.slint:337-353`.

### The hook that solves it

Verified against the installed Slint 1.17.1 (`i-slint-compiler-1.17.1/builtins.slint`), not guessed:

- `TextInput` has `callback key_pressed(event: KeyEvent) -> EventResult`, documented as *"Use this
  callback to handle keys **before** TextInput does. Return `accept` ... or `reject` to let TextInput
  handle it."* Use it. No bubbling tricks, and no need to sacrifice text copying.
- Text-selection state is `cursor-position_byte-offset != anchor-position-byte-offset`. Both are
  marked *"Internal, undocumented property, only exposed for tests"* in Slint's own source. Say so in
  a comment where you read them, so the next reader does not mistake them for public API.
  `Cargo.toml` pins `slint = "1.9"` (a caret range resolving to 1.17.1), so a `cargo update` could
  remove them — but that fails the `.slint` compile loudly rather than killing the feature silently,
  which is why the risk is accepted.
- The note field hands `has_selection` to Rust and obeys the answer. Rust returns whether it consumed
  the press: `true` and the event is accepted, `false` and it is **rejected**, so `TextInput` performs
  its own native Ctrl+C. Nothing calls `copy()` by hand, so a double copy is not possible. (An earlier
  draft had Rust invoke a `public function` to copy the text; that cannot work - the note panel lives
  inside `if is-narrating`, and an element in a conditional subtree is not addressable from the root.)
- `SdTextField` is the product's one text field and the Editor's Observation Summary uses it too, so
  whatever is added there must default to off and leave that field's behaviour unchanged.

### `event.text` for `Ctrl+C` is not `"c"`

It arrives as the ASCII control code ETX, not the letter. This repo already paid for that once and
the finding is recorded at `main.rs:1808`: *"Ctrl+C's own text arriving as the ASCII control code
rather than the letter"*. Matching `"c"` produces a shortcut that never fires, with a cause that
takes a long time to find.

### Put the decision in Rust, not in Slint

Slint reads the two offsets and passes `has_selection: bool` across; **Rust** decides text-vs-image
and answers with a bool the field turns into `accept` or `reject`. Slint contributes no `if` over those
flags — otherwise the rule would exist in two places and a test could only assert a copy of its own
input. This is the repo's own pattern (`main.rs:1837`: *"Rust builds it, not Slint, so ... the only place
that decision is made"*), and it is here for a concrete reason — corrected during implementation, the
first version of this ticket got it wrong. There IS a seam for the overlay's keys:
`test_capture_interaction.rs`. But those guards read the `.slint` SOURCE. They can prove a branch
exists; they cannot prove it decides correctly, so an inverted condition stays green. A Rust decision
function can be watched failing. Both layers are now used: source guards for reachability, Rust tests
for the rule.

## Acceptance criteria

- [ ] `Ctrl+C` with a region selected and **no text selected** in the note field puts the region on
      the clipboard and closes the overlay
- [ ] `Ctrl+C` with text selected in the note field copies **the text**, leaves the image alone, and
      leaves the overlay **open**
- [ ] `Ctrl+Enter` copies the image regardless of any text selection, and closes the overlay
- [ ] All of the above work **while the note field has focus** — the default state
      (`appwindow.slint:1013`), so this is the criterion that actually matters
- [x] The text-vs-image decision is a Rust function with its own unit tests, seen red
- [ ] Neither chord does anything when no region is selected yet (the same guard `Enter` has)
- [x] After an image copy: no new file under `findings/` in the Vault, and no new row in
      `library.db` — asserted, not eyeballed
- [x] The Quality Budget governs the clipboard image exactly as it governs a saved Finding: the same
      capture, copied and saved, yields the same dimensions
- [ ] The Library's selection is unchanged: no `load_findings_into_window` with a new id, no card
      becomes active, no strip rebuild
- [ ] The Editor is **not** raised even when "Open the Editor after a hotkey capture" is ON — there
      is nothing to edit. `REVEAL_EDITOR_AFTER_CAPTURE` must still be reset so a stale `true` cannot
      leak into the next capture
- [ ] A toast on the image path says the image was copied **and not saved**, in those terms — the
      volatility is the one thing the Reviewer must not be allowed to misread. The text path needs
      its own distinguishable feedback, or the Reviewer cannot tell which of the two happened
- [x] The overlay's hint says so before the keystroke. Implemented as two lines rather than one:
      `Enter to save · Esc to cancel` above `Ctrl+C copies only — nothing is saved`. Two lines because
      the promise that matters most is the one about not saving, and a single 440px-wide caption
      sharing its row with two buttons had nowhere to put it.
- [x] A visible button sits beside "Save Finding" for the copy path — a keystroke with no affordance
      is a feature nobody finds
- [x] `Enter`, the "Save Finding" button, and `Esc` all behave exactly as before

## Implementation pointers

Everything needed already exists; this is a recombination, not new machinery.

- The crop: `persist_finding` (`main.rs:2787`) already crops from the overlay's live RGBA canvas via
  `RegionCapturer::crop_rgba_from_slice`, with clamping for a drag released off-screen. The crop is
  what this path wants; the reduce, `write_blob` and `create_finding` steps are what it must skip.
  Prefer extracting the crop over duplicating the clamp arithmetic.
- The clipboard write: `copy_burned_image` (`main.rs:1275`) already encodes BMP and calls
  `clipboard_win::set_clipboard`. Reuse its two hard-won details — BMP because the Windows clipboard
  image format is a DIB, and **RGB8 not RGBA8** because a DIB with alpha pastes fully transparent in
  Explorer and elsewhere.
- Read the canvas from `LIVE_OVERLAYS` the way `on_capture_completed` does (`main.rs:4309`). Do
  **not** clone it into a closure: the comment there explains that a clone still alive at the next
  capture's `make_mut_bytes` becomes a ~92 MB copy-on-write, the exact cost the reused buffer exists
  to avoid.
- Windows-only, like `copy_burned_image`. Keep the same `#[cfg(not(windows))]` arm returning an error
  string rather than failing to compile.
- The overlay must still close and still call `set_capture_exclusion(.., false)` on the image path.

## Why the Quality Budget matters here

A DIB is uncompressed, so the clipboard cost is `w * h * 3` bytes of the FINAL dimensions: ~6 MB for
a 1080p window, ~25 MB for a full 4K screen, ~50 MB for a 7680x2160 dual-4K selection. That is the
ceiling decision 2 removes — the resolved long edge caps it before the BMP encode ever happens.
`copy_burned_image` was already safe for the same reason: it reads an already-reduced blob.

## Verification

From the repo root:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`cargo build` **does** build this application. `AGENTS.md` still says it does not, citing `BUG-11`;
that paragraph is stale — it describes the Tauri app, which now lives in `archive/desktop-tauri`.
`apps/desktop` is a plain cargo bin (`[[bin]] name = "Snapdown"`) built with `slint-build`, with no
Tauri dependency at all.

Before rebuilding, check `Get-Process -Name Snapdown`: a still-running instance locks its own
executable, and `Access is denied (os error 5)` is what that looks like.

Note for whoever writes the tests: the "nothing was written" criteria are the ones with real teeth,
and they are also the ones that pass trivially if the test never exercises the new path. Assert
against a Vault directory listing and a `SELECT` count taken before and after, and confirm the test
goes red when the copy path is pointed back at `persist_finding`.

## What was verified, and how

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
`cargo test --workspace --no-fail-fast` are all green.

Five mutations were introduced and each was watched failing the test that names it, because a test
never seen red is a claim:

| Mutation | Test that caught it |
| --- | --- |
| `copy_chord_target` always answers `Image` | `ctrl_c_copies_the_image_unless_the_note_field_has_text_selected` |
| `force_image` dropped from the rule | `ctrl_enter_copies_the_image_even_with_text_selected` |
| the copy path writes a blob into the Vault | `a_copy_writes_nothing_and_hands_over_a_decodable_image` |
| the note field stops forwarding the chords | `the_copy_chords_are_reachable_from_the_note_field_and_from_the_window` |
| the hint stops promising "nothing is saved" | `the_copy_only_path_is_offered_in_the_overlay_and_says_it_saves_nothing` |

One of those runs found a defect in a test rather than in the code. The pre-existing
`enter_saves_from_the_window_not_only_from_the_note_field` located the FIRST `Key.Return` after the
key handler and looked 200 characters ahead for `root.has-selection` — so the moment the copy chords
added an earlier `Key.Return`, it began guarding the wrong branch, and a mutation that deleted Enter's
own guard passed. The first attempt at fixing it split on `else if` over the whole remaining overlay
source, whose last segment absorbed every other mention of `root.has-selection` in the file and passed
the same mutation. It now bounds the search to the handler body and was watched failing. A source-text
guard is only as good as its window.

## What is NOT executed yet

Nothing below is broken as far as the code can be read; none of it has been observed in a running app,
and the unticked criteria above are exactly this list:

- both chords while the note field has focus, which is the default state;
- the overlay staying open on a text copy (structurally guaranteed - `on_copy_chord` returns before
  `close_overlay()` on that branch - but not seen);
- the toast, its wording, and the fact that it lands on a main window that may be hidden;
- the Editor not being raised with "Open the Editor after a hotkey capture" ON.

The chords, the button, and the hint are guarded at source level, which proves they are wired, not
that they behave. Only a run in the real app closes these.

## Comments

Created from a UI/UX discussion on 2026-08-30. Feasibility was checked against the code first: the
crop and the clipboard write both already exist and are proven, which is why this is one ticket and
not a spec.

Implementation notes, in the order they were found:

- `SdTextField` gained `forward-copy-chords` (default OFF, so the Editor's Observation Summary is
  untouched) and a `copy-chord-pressed(bool, bool) -> bool` callback, intercepting via
  `TextInput::key-pressed`.
- `persist_finding`'s crop-and-reduce half was extracted as `prepare_region` and is now shared with
  the copy path, so the off-screen-drag clamp exists once.
- `encode_region_for_clipboard` was split from `copy_region_to_clipboard` so the bytes can be tested
  without replacing whatever the developer had on their clipboard during `cargo test`.
- The window-level branch's condition is written key-first (`Key.Return` before `modifiers.control`)
  because that reads in the order it happens, not to satisfy a scanner - the scanner was fixed
  instead.

The `Ctrl+C` rule started out as "always copy the image, sacrifice text copying for determinism" and
the owner replaced it with the conditional now recorded in decision 1. Reading Slint 1.17.1's own
`builtins.slint` showed the conditional is the better call: `TextInput::key_pressed` is documented as
the hook for handling keys before the widget does, and the selection offsets are readable, so nothing
has to be sacrificed. The same read turned up the ETX trap and the unbounded-payload consequence
above.
