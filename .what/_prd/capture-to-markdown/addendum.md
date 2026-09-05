---
type: addendum
parent: prd
initiative: capture-to-markdown
status: draft
created: "2026-08-22"
updated: "2026-09-03"
---

# Addendum — PRD: Capture to Markdown

Depth that earned a place beside the PRD but would derail it. Nothing here is a promise, and nothing
here may be cited as a design.

## Technical how — testable consequences per FR

### FR-1 — Capture a screen region from a global hotkey with precision guides and auto-detection

- The overlay covers every connected monitor, including monitors with different scale factors.
- Full-screen crosshair guide lines follow the pointer across the entire monitor canvas during targeting and dragging.
- A circular pixel loupe magnifier displays a magnified view (6x-8x) with pixel grid and live dimensions readout near the pointer.
- Moving the mouse over top-level windows and sub-panels highlights the detected container with an un-dimmed cutout preview.
- Clicking once on a highlighted container selects that exact rectangle without requiring manual dragging.
- Clicking the top-center Fullscreen button selects the entire active monitor area.
- Clicking or dragging elsewhere before saving allows re-selecting the region seamlessly.
- The selected region's dimensions are shown while dragging.
- Pressing Escape at any point before saving dismisses the overlay and writes nothing.
- A region smaller than 8 × 8 pixels is refused rather than saved.
- The hotkey works while another application has focus and without administrator rights.

**Out of Scope:**
- Video, GIF, and scrolling capture.
- Capturing a specific monitor by index without dragging or clicking.

### FR-2 — Write the Note at capture time

- The Note field is focused on appearance; no click is needed before typing.
- Saving is possible from the keyboard alone.
- An empty Note saves successfully — a Finding with no words is still a Finding.
- Escape from the Note field discards both the Note and the Capture.
- Multi-line text is accepted and preserved verbatim, including blank lines.

### FR-3 — Capture repeatedly without leaving the loop

- After a save, keyboard focus is back with the window that held it before the hotkey.
- The confirmation is a transient toast that never takes focus and never needs dismissing.
- The toast states the running count of Findings and offers an action to open the Editor.
- Whether the Editor opens automatically after a Capture is a setting, off by default.

### FR-4 — Reduce every captured image automatically

- An image already within the long-edge limit is not upscaled.
- Aspect ratio is preserved; the image is never stretched or cropped to fit.
- The budget carries **two** reduction levers, and they are different tools. A **maximum long edge**
  bounds the worst case and leaves a capture already under it alone. A **resize percentage** applies
  to every capture regardless of its size. The percentage is applied first and the maximum to the
  result — the other order would take the share off the maximum rather than off the capture, and two
  captures of different sizes would come out identical.
- Reduction never blocks the overlay from closing.
- Exactly one image file per Finding exists in the Vault after reduction.

### FR-5 — Choose a Quality Budget by naming the intent

- The shipped default is `Auto`, and a Reviewer who never opens Settings never sees a raw number.
- Auto resolves a *different* long edge and encoder quality for a small region than for a full-screen
  capture. A test that captures both and finds identical parameters is a failing test.
- Settings show the stored size of the most recent Finding **and the budget that produced it**, so the
  effect of a change is visible and attributable.
- Editing an Advanced value moves the named state to `Custom` visibly, in the same interaction. The
  Reviewer never leaves Auto without seeing that they left it.
- A change applies only to Captures taken after it; existing Findings are never re-encoded.
- Values outside a sane range are refused at the point of entry, not on next capture.
- Each of the four named budgets has a stated intent, and no two resolve to the same parameters for
  the same input.

This replaces the promise of two raw numbered fields. That promise was not wrong — it already said
"defaults the Reviewer never has to change" — but it was unkeepable as presented: `1600` and `75` are
values a Reviewer can accept and cannot judge, and § 8 records that the team cannot defend `1600`
either. `OQ-3` is not closed by this. It is restated: the question stops being *what is the right
constant* and becomes *is the derivation legible at its smallest output*.

### FR-6 — See every Finding with its Note

- Findings are ordered newest first.
- A Finding captured while the Editor is open appears without the Editor being reopened.
- A Finding whose image file is missing from the Vault is shown as broken rather than omitted.
- The list stays usable at several hundred Findings.

### FR-7 — Edit a Note

- An edit is saved without an explicit save action.
- Clearing a Note entirely is allowed.
- The numbered lines belonging to Markers cannot be renumbered by hand — that ordering belongs to the
  Markers.

### FR-8 — Place numbered Markers bound to numbered Note lines

- Markers are numbered from 1 upward with no gaps.
- Deleting a Marker removes its line and renumbers every Marker and line after it.
- A Marker can be repositioned without changing its number.
- Marker positions survive closing and reopening the Editor.
- The Marker's sub-comment may be empty; the numbered line still exists.

### FR-9 — Select several Findings at once

- Range selection and individual toggling both work.
- The count of selected Findings is visible.
- Selecting all and clearing the selection are each one action.
- The selection survives scrolling and is cleared when the action completes.

### FR-10 — Compose selected Findings into a named Bundle

- The Bundle's Markdown references its images by relative path, and every referenced file exists.
- A Finding's Markers appear in the Bundle's image, and its numbered lines appear in its Note.
- The Bundle's name appears as the document's heading.
- Composing does not remove the Findings from the Library.
- Composition refuses, naming the Finding, if a selected Finding's image file is missing.
- One Finding may appear in more than one Bundle.

### FR-11 — List and reopen Bundles

- Bundles are ordered newest first.
- Opening a Bundle shows the composed content, not a live view of the current Findings.
- A Bundle whose Markdown file is missing is shown as broken rather than omitted.

### FR-12 — Copy a Bundle's Markdown

- The copied text is the Bundle's document, with no added wrapper.
- The image links in the copied text are absolute, so they resolve regardless of which folder the
  reader of the pasted text is working in.
- The Reviewer is told the copy succeeded **and** that what was copied carries absolute image paths.
  A person reading the pasted text cannot discover that for themselves, and it is the only place they
  learn that the text contains disk locations.

**Encoding, settled 2026-08-31 by test rather than by argument.** The links are rendered as forward
slashes wrapped in angle brackets. Six candidate forms were run through a CommonMark reference
implementation over three realistic Vault paths — one containing a space, one a space and parentheses,
one a space and an apostrophe. Raw Windows backslashes and bare forward slashes produce **no image at
all**: the space ends the destination. Percent-encoding works but hands the reader `Snapdown%20Vault`
to decode. Both `file:///` forms fail — and not on syntax, which is valid, but on the link-security
blocklist that real Markdown readers apply to the `file:` scheme and that a reader cannot switch off.
Angle brackets win because a conforming parser strips them and yields the path with its space intact.

**What the Reviewer is told, and what they are deliberately not told.** The toast names the paths, not
the images. Images could never travel on a text clipboard — no chat client loads a local file — so
warning about them would warn about an impossibility. What is worth saying is that the text carries
absolute disk locations, because those contain the operator's user name and go wherever the text is
pasted. The toast does **not** point the Reviewer at `Open file location`: that action sits one place
away on the same row, already visible.

### FR-13 — Delete Findings, and their image files with them

- Deletion asks for confirmation once, naming how many Findings will go.
- Deletion is refused, and nothing is removed, if the confirmation is declined.
- A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image
  and stays readable.
- A file that cannot be deleted is reported rather than silently skipped, and the Finding stays.

### FR-14 — Delete a Bundle with its images

- Deletion asks for confirmation once and names the Bundle.
- The Findings the Bundle was composed from are not deleted, and become assemblable again.
- Deleting a published Bundle also ends its Publication.

### FR-15 — Report orphans

- The check runs on start and can be run on demand.
- An unreferenced file in the Vault is listed, and deleting it is one action.
- The check never deletes anything on its own.
- A clean Vault reports itself as clean rather than saying nothing.

### FR-16 — Choose the Vault folder

- A default location is used until the Reviewer picks one, so capture works before any setup.
- A folder that cannot be written to is refused at the point of choosing.
- Changing the folder offers to move existing files, and either moves all of them or none.
- The current folder can be opened in Explorer in one action.

### FR-17 — Edit the hotkeys

- Every registered hotkey is listed with its current combination.
- A combination already held by another process is refused with a message naming the conflict.
- A hotkey can be cleared, disabling that action rather than leaving a broken binding.
- Two Snapdown actions cannot be bound to the same combination.
- A combination that fails to register at startup is reported, not swallowed.

### FR-18 — Run at Windows startup, on by default after first run

- **It is on after first run, without the Reviewer enabling it.** A capture tool the Reviewer has to
  remember to launch is a capture tool that is not there when the observation happens, which is the
  one moment it exists for.
- Enabling it requires no administrator rights.
- Starting this way opens no window — the tray icon only.
- The setting reflects the actual registration state, not a remembered intention. In particular, the
  control never shows an intended state before the real one has been read.
- Disabling it removes the registration rather than leaving it and ignoring it, and a later run does
  not re-enable it. The default applies to a first run, never to a Reviewer's decision.

### FR-27 — Name the surface the Reviewer is looking at

- The installed executable, the tray tooltip, and the window title never disagree about the product's
  name. A test asserts the three against one source.
- The build produces exactly one desktop executable. A second one left in the output directory is a
  build failure, not clutter — a stale `desktop.exe` beside the correct `Snapdown.exe` is precisely
  what caused the Reviewer to run the wrong binary and conclude the product had no navigation.
- The workspace window is titled for its persona, not for the section currently open. Its title does
  not change as the Reviewer moves between sections.

### FR-28 — Reach every surface from every surface

- Navigation to every primary surface is present and visible on every primary surface.
- The surface the Reviewer is on is distinguishable from the ones they are not, by more than colour
  alone.
- Opening the Editor from the tray, from a hotkey, and after a Capture all arrive somewhere the
  Reviewer can navigate out of. None of them is a dead end.

### FR-29 — A primary surface fits the window it opens in

- Settings presents its four groups — startup, Vault folder, Quality Budget, hotkeys — within the
  window at its minimum supported size. Agent access is **not** one of them: it is a primary surface
  of its own (`FR-28`, `inventory-screen.md` row 13), and counting it here would have it appear twice
  in one product.
- Scrolling to read *more* of a list is allowed and expected. Scrolling to discover that a control
  *exists* is not, and the distinction is what this requirement turns on.
- No layout gives a group vertical space it does not use in order to match a neighbour's height. The
  shipped Settings screen paired a one-checkbox group with a four-control group in equal columns and
  left roughly a third of the screen empty to do it.

### FR-30 — Draw transparent outlined shapes and directional arrows on canvas

- Shape box has transparent fill and solid stroke border.
- Arrow has a start point, shaft line, and triangular arrow head at the end point.
- Active elements show transformation handles.
- Deleting via Delete/Backspace keyboard keys or context menu removes the element cleanly.

### FR-31 — Draw blur redaction boxes over sensitive image regions

- Underlying screenshot pixels are redacted in both visual display and final burnt PNG.
- Blur rectangle supports standard 8-point resizing and dragging handles.
- Multiple blur areas can overlap or exist independently.

### FR-32 — Place floating text and callout bubbles with font and tail controls

- Callout consists of a text bubble container plus an adjustable pointer tail pointing to a target
  coordinate.
- Floating text has a transparent background with legible text fill.
- Font size and font family can be adjusted per text/callout element.
- Text contents are preserved across editor sessions.

### FR-33 — Transform and manipulate canvas annotation elements with interactive handles

- Active state displays distinct resize/transform handles.
- Element can be moved by dragging within its bounds.
- Pressing Escape deselects the active element.
- Redo/Undo history is supported for canvas additions, moves, edits, and deletions.

### FR-34 — Zoom the canvas to inspect a capture at more or less than natural size

- Zoom is a view state, never a stored one. Reopening a Finding shows it at natural size.
- Marker and annotation coordinates stay normalised to the image, so nothing drifts with the zoom.
- Placing a Marker while zoomed puts it where the pointer is, not where the pointer would have been
  at 100%.

### FR-35 — Paste an image from the clipboard as a new Finding

- The pasted image goes through the same Quality Budget a Capture does. A pasted 4K screenshot is
  reduced exactly as a captured one would be, or BG-3 has a second door with no lock on it.
- A clipboard holding no image says so and does nothing, rather than creating an empty Finding.
- Its `source_monitor` says it was pasted. A Finding that claims a monitor it never came from is a
  small lie the Reviewer would have no way to catch.

### FR-36 — Copy a Finding's burned image to the clipboard

- The bytes are what a Bundle would carry — the same burn, not a second rendering path that could
  disagree with it.
- A redaction that is in the file is in the clipboard copy. This requirement is the one place the
  Reviewer can hand over an image without a Bundle, and a blur that only existed in the Bundle
  would make that path leak.

### FR-37 — Reach a canvas or filmstrip action from a right-click context menu

- No entry exists that has no other route. A context menu is a shortcut to what is already
  reachable, never the only way to reach something — NFR-16 forbids a control that has to be
  discovered.
- The menu names the thing under the pointer. A right-click on empty canvas and a right-click on a
  Callout do not offer the same list.

### FR-38 — Change the front-to-back order of canvas annotations

- Four movements: to the front, forward one, backward one, to the back.
- The captured image is always underneath every annotation. It is not in the order and cannot be
  moved within it — it is the thing being annotated, not an annotation.
- Order is stored, so it survives closing the Editor.
- A movement that changes nothing writes nothing.

### FR-39 — Export a Bundle as a PDF

- One column, A4 only. The Bundle's title and its composition date open the document as a title
  block, not as a cover page of its own.
- Every page is numbered.
- No image is ever split across a page break except by the deliberate slicing a very tall image
  requires.
- The text is a real text layer: selecting, searching and copying all work, and a machine reading the
  file is not obstructed by it.
- Exporting changes nothing about the Bundle, and the same Bundle exported twice produces the same
  document.

### FR-40 — Edit a composed Bundle's title and notes

- Editable: the Bundle's title, its Bundle notes, its Finding notes, and its Marker notes. Nothing
  else, and the set of images is frozen — none can be added, removed, reordered, or replaced.
- Saving rewrites the document's heading to the new title, so the title and the heading cannot
  disagree.
- Nothing is written until the Reviewer saves, and abandoning the edit leaves the Bundle as it was.
- No edit made here changes any Finding, its Note, or its Markers.
- A Bundle whose source Findings are already gone can still be edited this way.

### FR-41 — Discard the source Findings behind a Bundle, keeping the Bundle

- Confirmation is asked once, naming the Bundle and how many captures will go.
- The Bundle's own images and document are untouched by the act.
- After it, the Bundle offers deletion only — it can no longer return its Findings to the Library.
- A capture that another Bundle also holds is still destroyed; that other Bundle keeps its own copy.
- A file that cannot be deleted is reported rather than silently skipped.

Why this is not part of composing is stated once, in prd.md §3.5.

### FR-42 — See which Bundles still hold original captures, and reclaim their disk in bulk

- The surface is reachable from the Library and from the Vault section of Settings.
- A Bundle whose originals are already gone is not listed.
- Confirmation is asked once for the whole selected set, naming the total that will be reclaimed.
- Nothing is destroyed for a Bundle the Reviewer did not select.

### FR-43 — Open a Bundle's folder in the file manager

- It opens the Bundle's own folder, not the Vault root.
- It changes nothing: no file is written, moved, or renamed.
- A Bundle whose folder is missing reports that rather than opening the wrong folder.

### FR-44 — Delete several Bundles and their images together, in bulk

- Reached from the same reclaim-space surface as FR-42, over the same selected set — a select-all
  checkbox ticks every row currently listed.
- One confirmation names the whole selected set and what is destroyed (both the Bundles themselves
  and their originals), stating it cannot be undone — the same discipline FR-14's own single-Bundle
  "Delete both" confirmation already follows.
- Triggers FR-42's own bulk Finding-discard act over the same set; it does not re-implement it.
- A Bundle outside the selection sharing a Finding with one inside it (`BR-122`) is named in the
  confirmation, read live at confirmation time — the same mechanism FR-42's own bulk confirmation
  already uses.
- A Finding shared by two selected Bundles is destroyed and reported exactly once, never once per
  Bundle that names it.

## Rejected alternatives

| Option | Why it lost |
| --- | --- |
| Write the Note in the Editor rather than at capture time | The Note is cheapest to write in the second the Reviewer noticed the thing. Deferring it means either the Editor opens on every Capture — which breaks the loop — or a queue of un-noted images the Reviewer has to go back and interpret, which is exactly the failure the product exists to remove. |
| Open the Editor after every Capture, as a general capture tool does | The loop has to survive six runs in ninety seconds. A window taking focus after each one turns six Captures into six dismissals. Kept as a setting, off by default. |
| One overlay window spanning the whole virtual desktop | Simpler until the monitors have different DPI scaling, at which point the selection rectangle and the pixels it maps to stop agreeing. Per-monitor overlays cost more code and are correct. |
| Keep the original full-resolution capture beside the reduced one | Sounds free and is not: the Vault doubles, deletion has two files to keep consistent, and nothing ever reads the original. If the reduction was too aggressive the answer is to change the Quality Budget and re-capture, which takes seconds. |
| Reduce images at compose time rather than capture time | Puts the expensive work in the path the Reviewer is waiting on, and leaves the Vault full of unreduced files in the meantime. Reduce once, on the way in. |
| Marker numbers the Reviewer assigns by hand | Two sequences that have to be kept in step is the drift this feature exists to prevent. One sequence, owned by the Markers, renumbered on delete. |
| Freeform annotation layers (arrows, boxes, highlights) stored as vectors | Every one of them is invisible to the reader that matters — the agent reading Markdown, who sees only Marker lines. This rejection held until `CAP-11` (2026-08-31) added exactly these shapes for a *different* reader: the person looking at the burnt image or an exported PDF, who never touches the Markdown. A numbered badge is still the only annotation the Markdown-reading agent gets. |
| Soft delete with a recycle bin | Makes the Vault and the Library disagree by design, and the whole point of BG-5 is that a review leaves completely. Confirmation once is the safety, not a bin. |
| Store Notes inside the image file's metadata | Survives file moves and needs no database — but multi-select, Bundles, Marker ordering, and Publication state are all queries, and answering them by re-parsing a folder is a database with worse ergonomics. |
| One rolling Markdown file per day instead of named Bundles | Cheap to write, useless to hand over. A Handoff has a subject, and the grouping is what makes it readable. |
| Make a Bundle a live view over its Findings | Then a Bundle already handed to an agent changes underneath the conversation about it. A Bundle is a snapshot; that is what makes it citable. |
| Let a Bundle's Markdown be edited in place | The Bundle would drift from the Library that produced it, with no way to tell which was right. Recompose instead. |

## Options weighed

### Where the Note is written

Criteria fixed before scoring: keystrokes per Finding; whether focus is stolen; whether the Reviewer
can still see what they are describing; whether an un-noted backlog can accumulate.

| Placement | Keystrokes | Focus kept | Subject visible | No backlog |
|---|---|---|---|---|
| Inline field at the selected region | fewest | yes | yes | yes |
| A dialog window after release | more | no | partly | yes |
| In the Editor, later | most | no | no | no |
| Not at all — image only | fewest | yes | yes | no note exists |

The inline field wins on every criterion that was set, which is unusual enough to be worth recording:
it means the decision is not a trade-off and should not be revisited as if it were one.

### Marker rendering

Criteria: the Marker must be visible to a machine reading the image; it must not obscure the thing it
points at; it must survive image reduction; the Reviewer must be able to reposition it.

Two shapes were considered — burning the badge into the stored image at capture time, or storing
Marker coordinates and burning them in only when a Bundle is composed. The second is what the PRD
promises, and the reason is FR-8's requirement that a Marker be repositionable and renumberable: a
badge already burned into the file cannot be moved. It does mean the Bundle's image and the Finding's
image are not the same bytes, which is why FR-14 has to delete both.

### Why CAP-9 sits under `settings`

`settings` already holds the container-level Logical Components — the startup registrar, the hotkey
registrar, the settings store — which are the app's own machinery rather than any one screen's. The
window shell CAP-9 governs is machinery of the same kind, so it sits beside them rather than under
`finding` or `bundle`, whose screens it also reaches into.

### Ordering inside a Bundle

Selection order is the ordering, and no second mechanism exists in r1. The alternative — a drag-to-
reorder step during composition — was left out because it introduces a second source of truth for
"what order are these in" while the first one is free. If reordering is wanted later it belongs as a
property of the selection, not of the composed Bundle.

## Mechanism and transport

Not a design. The SDD owns that, and a builder MUST NOT follow this section.

- Per-monitor transparent overlay windows, created on hotkey and destroyed on save or cancel. Their
  lifetime being that short is what keeps them from interfering with anything.
- The reduction step wants to be off the path that dismisses the overlay, so that NFR-2's 500 ms does
  not include encoding time. That implies the save is recorded first and the file is finished
  immediately after, and it implies a Finding can briefly exist with its image still being written —
  which the Editor has to tolerate.
- Marker coordinates want to be stored normalised to the image, not in pixels, so that they survive
  any later change to the Quality Budget.
- The Note's numbered lines and the Marker list are one structure. Anything that stores them as two
  lists joined by a number will drift on the first delete.
- Deletion of a file and deletion of its row want to be one unit of work that either completes or
  does not, because NFR-5 is stated as an invariant rather than as a best effort.

## Sizing

Nothing sized. Wave sizing happens at G4 and G5 against the story list.

One figure recorded because it drives the FR-4 and NFR-3 defaults: a full-screen capture on a
3840 × 2160 monitor is about 8.3 megapixels; the same view at a 1600 px long edge is about
1.4 megapixels, roughly a sixth. Source: the primary user's own monitor resolution, not a benchmark,
and it is the reason OQ-3 is open rather than answered.

### Two path conventions for one Bundle — `FR-12` and `NFR-8`

`NFR-8` governs the **stored** `bundle.md` and keeps its image links relative to that file's own
folder. `FR-12` governs the **clipboard**, and as of 2026-08-31 it permits those same links to be
rendered as absolute paths. The two are not in conflict: they serve two different readers, and the
stored file is not what the clipboard hands over.

The mechanism belongs here rather than in the PRD. One composer takes a base path and serves both
renderings; a second serializer alongside the first is what must **not** be built, because two
serializers drift and the golden-file test only pins one of them.

**Settled 2026-08-31, all three.** Ticket 03 on the Bundle Library map opened three questions here —
which encoding the absolute form uses, whether the Reviewer is told that images do not travel on a
text clipboard, and whether `Open file location` survives the answer — and all three are answered
above, under FR-12's own "Encoding" and "What the Reviewer is told" entries: forward slashes wrapped
in angle brackets, proven against a Vault path containing a space; the Reviewer is told the copy
carries absolute disk locations, not that images cannot travel on a clipboard, which would warn about
an impossibility; and `Open file location` is untouched, sitting one place away on the same row.

### Rendering a Bundle as a PDF — `FR-39`

`FR-39` states what the Reviewer gets and deliberately names no engine, because `prd.md` carries no
solution shape. The mechanism is nonetheless settled work rather than an open question, and this is
the pointer to where it is recorded so nobody redoes it:

**`.scratch/bundle-library/issues/07-research-the-pdf-render-engine.md`** holds the whole
investigation — the engine chosen and why, its licence, the disk and idle-memory cost measured rather
than estimated, how a screenshot too tall for a page is fitted, and why the escape set for inserted
text is closed by definition. It also records the rejected candidate and the specific way it failed:
it was a third of the size and silently dropped every image across three input forms, which is the
kind of failure only decoding the output reveals.

Two things there are deliberately **not** settled, and both belong to the Export PDF effort rather
than to this PRD:

- **Packaging** — whether the exporter runs in-process or as a separate crate. The research reversed
  itself twice on it, which marks it as an architectural judgement rather than a measurable fact, so
  it is owed a `DEC-` at the point the work is planned.
- **The two tall-image thresholds** — the clamp and the slice points are solved and measured from a
  PDF's own placement matrices, but they still need calibrating against real screenshots rather than
  synthetic ones.

## Personas and research detail

Two facts about the primary user shaped this initiative more than any persona detail, and both come
from their own account rather than from research:

- They review in bursts. Four or five Findings arrive minutes apart, so every requirement about focus,
  toasts, and not opening windows traces back to this one fact.
- They have abandoned this workflow before, in a general capture tool plus a folder plus manual
  pasting. The reason was not any single missing feature; it was that the tool became the thing being
  managed. CAP-6 exists because of that, not because settings are a feature.

No external research was run for this PRD and `_bmad-output/` holds no run folder for it.
