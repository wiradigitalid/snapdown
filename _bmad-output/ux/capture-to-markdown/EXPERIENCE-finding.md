---
type: ux
component: finding
document: experience
created: "2026-08-23"
updated: "2026-08-23"
---

# EXPERIENCE — Finding

## Information architecture

A Finding is created outside the Editor and lives inside it. Two surfaces, and they are deliberately
unalike because they serve opposite moments.

| Surface | Moment | Persona |
|---|---|---|
| **Capture Overlay** | The Reviewer has just noticed something. Every millisecond and every decision is a tax | **Snapdown** (tray). No Editor window need exist |
| **Findings** | The Reviewer is working through what they noticed | **Snapdown Editor** |

The Overlay is not a screen of the Editor and must never require it. `NFR-1` gives it 200 ms to appear
across three monitors, and `NFR-4` forbids any network call in that path.

Findings is one of the Editor's four primary surfaces (`FR-28`) and holds three regions: the **capture
rail** (every Finding, newest first), the **canvas** (one Finding's image with its Markers), and the
**note pane** (that Finding's Note).

## Voice and tone

At capture time, almost nothing. A crosshair, a live dimension readout, and one field that says
`What is wrong here?`. No chrome, no toolbar, no title.

In the Editor, the Reviewer's own words are the content and the product's words stay out of the way.
Every label is a noun the Reviewer already uses: Finding, Note, Marker, Bundle, Vault — all of them
already in `.control/product-glossary.md`. **No new user-facing noun is introduced by this design.**

## Component patterns

| Pattern | Behaviour |
|---|---|
| **Capture rail** | Thumbnails, newest first, with a selection checkbox that appears on hover or focus. Multi-select for `FR-9` |
| **Canvas** | The image at fit-to-pane. Clicking places the next Marker; dragging an existing one moves it |
| **Note pane** | A plain multi-line text field beside the image. Saves as the Reviewer types, debounced |
| **Marker badge** | A numbered amber disc. Its number is its identity and matches a numbered line in the Note |

**There is no tool palette, and its absence is the design.** The PRD makes arrows, callouts, blur and
effects a Non-Goal, and the reasoning is not minimalism for its own sake: the reader is a machine, and
a machine reads `1.` in the text and `①` on the image, not an arrow's implied direction. Cobalt
Capture, building for the same audience, independently arrived at the same place — a crop taken at
capture time and an editable paragraph beside the screenshot, no markup tools. Snagit has the palette
because Snagit's reader is a person. Copying it here would be copying a solution to someone else's
problem.

## State patterns

### Capture Overlay

| State | What the Reviewer sees |
|---|---|
| **Armed** | The screen dims. A crosshair, and a live `W × H` readout following the pointer |
| **Dragging** | The selected region is undimmed and sharp; everything else stays dim. The readout tracks the region |
| **Narrating** | The region stays lit. One field anchored beneath it: `What is wrong here?`. Enter saves; Shift+Enter is a new line |
| **Saving** | The overlay is already gone. Reduction happens behind it — `NFR-2` gives 500 ms to dismiss and return focus, and reduction MUST NOT block that |
| **Error** | The capture could not be written to the Vault. A toast names the folder and offers Settings. The image is not silently lost |

There is no empty state and no loading state for the Overlay. It is either armed or it is not on screen.

### Findings

| State | What the Reviewer sees |
|---|---|
| **Empty** | One line — "No findings yet" — and one action, which is the hotkey itself, shown as the actual bound combination rather than a button. The action that ends the emptiness is a keypress, so the empty state teaches it |
| **Loading** | The rail renders its frame; the canvas and note pane hold their shape. Nothing jumps when data lands |
| **Populated** | Normal |
| **Nothing selected** | The rail is populated, the canvas invites a selection. This is a distinct state from empty and the shipped build conflates their look |
| **Image missing** | The record exists, the file does not. The canvas says so and offers the orphan report (`FR-15`). It does not render a broken image |
| **Error** | The Library could not be read. One message, one Retry. Snapdown does not recreate the store |

## Interaction primitives

- **Enter saves the Note at capture time.** The single most repeated keystroke in the product, and it
  must never require the mouse.
- **Esc cancels the capture.** Nothing is written.
- **Click places a Marker; drag moves it.** No mode to enter and no tool to select first.
- **Deleting a Marker renumbers the rest contiguously.** The numbers are positions, not names.
- **The Note saves as it is typed.** There is no Save button on a Finding.
- **Deletion is confirmed once and is real.** `FR-13` deletes the image file with the record.
- **Multi-select composes.** Selecting several Findings offers one action: compose a Bundle (`FR-9`).

## Accessibility floor

**WCAG 2.2 AA**, per `NFR-16`, with two things this surface owes specifically.

- **The canvas is not the only way to work with Markers.** Each Marker has a corresponding row in the
  note pane, focusable and movable from the keyboard. An image-only interaction is unreachable by
  keyboard, and Markers are load-bearing for `BG-1`.
- **A Marker's number is its accessible name**, and it matches the numbered line it binds to. That
  binding is the product's whole reason for existing; it cannot live in pixels alone.
- Contrast holds in both Windows themes. The **Marker's own colours are theme-invariant** and correct
  by construction: amber, black text, white ring, chosen against the image beneath, not the app.
- The Overlay announces itself and its region dimensions to a screen reader.

## Key flows

**Wira files five findings in one pass — `UJ-1`.** He is walking through a client's staging site. He
sees a misaligned button. `Ctrl+Shift+S`. The screen dims within 200 ms. He drags a box round the
button, types "this drops below the fold at 1280", presses Enter. The dim is gone and his cursor is
back in the browser before he has finished the thought. He does it four more times. **He never opened
the Editor.** The climax beat is that the loop has no Editor in it.

**Wira points at three spots in one screenshot — `UJ-2`.** He opens the Editor from the tray. The
newest Finding is already selected. He clicks three places on the image; three amber discs appear,
numbered 1, 2, 3. In the note pane he writes three numbered lines. He deletes the second disc; the
third becomes 2 and his second line is now unbound — and the pane shows that it is, rather than
silently pointing his numbering at the wrong place.

## Edge cases

| Moment | What the Reviewer does next |
|---|---|
| The region drag is a single click, zero-area | Nothing is captured, the overlay stays armed. A zero-pixel Finding is never created |
| The region spans two monitors with different DPI | The captured image is correct at each monitor's own scale. This is `NFR-1`'s named condition, not a bonus |
| The Reviewer presses the capture hotkey while the overlay is already up | Ignored. Two overlays never stack |
| The Note is left empty | The Finding is saved anyway. An image with no note is still an observation, and forcing text at capture time is the tax the product exists to remove |
| A Marker is placed and the Note has no matching line | The Marker exists and the pane shows it unbound. Neither is deleted to tidy the other |
| The Vault fills up or goes read-only mid-session | The capture fails loudly with the folder named. `FR-16`'s "refused at the point of choosing" cannot cover a folder that changed underneath |
| A Finding is deleted while it is inside a Bundle | Allowed — the Bundle holds its own copy (`FR-13`). The confirmation says so, because the Reviewer will otherwise assume the Bundle broke. **This does not currently hold in the shipped product: `BUG-1`.** `bundle_item.finding_id` cascades, so the deletion silently removes the Bundle's record of that item while its Markdown and image copy survive |
| Windows theme changes while the canvas is open | The app repaints; the image and its burned Markers do not, and must not (`NFR-17`) |
