---
title: Capture to Markdown
initiative: capture-to-markdown
created: "2026-08-22"
updated: "2026-08-31"
---

# PRD: Capture to Markdown

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-08-22 | Initial version | The desktop review loop is the product's core; nothing else can be handed off until findings exist | r1 |
| 2026-08-31 | Closed the last of ticket 03's open items. `FR-43` promises opening a Bundle's folder — an action the Library row already carried with nothing behind it. `FR-12` now states that the Reviewer is told the copied Markdown carries absolute image paths, and its encoding is settled by test rather than argument. `NFR-19` corrected: written the same morning, it forbade the very page-slicing that the tall-image research it rests on prescribes. One wording fix in § 2.3, where a journey called the Markdown an "export" — in this product **export means PDF**, and the Markdown path is **Copy**. | A row action with no requirement cannot be specced at all, and the clipboard promise was silent about the one thing a Reviewer cannot discover for themselves: the pasted text contains absolute disk paths, which carry the operator's user name. `NFR-19` was a self-contradiction introduced hours earlier and would have been carried into the first spec that cited it. | r3 |
| 2026-08-31 | Grew four promises the Bundle Library needs and corrected two that contradicted them. **New:** § 4.10 Export PDF (`CAP-12`, `FR-39`, `NFR-19`), `FR-40` editing a composed Bundle's title and notes, `FR-41` discarding a Bundle's source Findings, `FR-42` the reclaim-space surface. **Removed from § 6.2:** the non-goals "Renaming a Bundle" and "Exporting a Bundle to anything but Markdown". **Corrected:** `FR-12` no longer fixes the clipboard to relative image links, and `FR-14` no longer offers to destroy a Bundle's source Findings in the same confirmation. | A Reviewer can assemble Bundles but has almost no way to live with them afterwards — no way to fix a typo without recomposing, no way to hand one to a person rather than an agent, and no way to reclaim the disk an archived review is holding. The two corrections are older promises that pointed the opposite way from these four, and leaving them would have made this document promise two designs at once. `FR-41` and `FR-42` sit under `CAP-5` rather than `CAP-4` because they destroy Findings, which the `bundle` component has no authority to write. | r3 |
| 2026-08-23 | Added § 4.7 (`CAP-9`, `FR-27`–`FR-29`) and `NFR-16`–`NFR-18`; rewrote `FR-5`; amended `FR-18` | r1 and r2 shipped every promised capability, and the first sustained use produced a list of experience defects rather than missing features. `BG-7` now carries that, and this initiative gains the requirements that make it checkable | r3 |

## 0. Document Purpose

This PRD is for the Product Owner and for the downstream blueprint and component work. It covers the
desktop loop: capturing a screen region, writing a Note about it, marking it up, and composing
selected Findings into a Bundle — plus the settings that keep the tool out of the way. It stops at
the moment a Bundle exists. How a Bundle reaches an agent is a different functional area and lives in
the `agent-handoff` PRD; a reader looking for MCP or web publishing will not find it here.

Vocabulary is anchored in `.control/product-glossary.md` and used verbatim. Features are grouped, with
`FR` nested underneath. Where something was inferred rather than confirmed it carries an inline
`[ASSUMPTION]` tag and appears again in §9. The problem, the primary user, and the product boundary
come from `.what/_product-brief/brief.md` and are not restated.

## 1. Vision

A Reviewer looking at running software finds something wrong with it, presses one key, drags a box
around the evidence, types what is wrong, and is back where they were — in a few seconds, with
nothing opened and nothing to file away. They do that as many times as the review needs.

When the review is done, they open the Editor, see every Finding they took with its Note beside it,
tidy the wording, put numbered Markers on the two images that need pointing at, select the five that
belong to one concern, and name a Bundle. Snapdown writes one Markdown document where every Note
sits under the image it describes.

What makes that document worth anything is the binding: Note four is under image four because the
product never let them apart, not because the Reviewer remembered to keep them in order. Everything
in this initiative exists to keep that binding intact from the hotkey to the file — and to keep the
images cheap enough that a machine can afford to read them.

## 2. Target User

### 2.1 Jobs To Be Done

- Report several visual defects at once without any of them losing the image it refers to.
- Get a finding out of my head and onto disk before I lose the thread of what I was doing.
- Point at one specific thing inside a screenshot without describing where it is in words.
- Hand over a review as an artifact I can re-send, revise, or give to a second reader.
- Stop paying for a full-resolution 4K screenshot every time I show a machine a button.
- Throw a review away completely — notes and image files together — when it is done.
- Set the hotkey, the folder, and the startup behaviour once, and never think about the tool again.

### 2.2 Non-Users (v1)

- A second person on the same Library. There is one Reviewer, no accounts, and no sharing of the
  Library itself.
- Anyone who wants a picture for a human audience. Arrows, callouts, and effects are not here and
  are not coming.
- An agent. Agents read what this initiative produces; nothing here is driven by one.

### 2.3 Key User Journeys

- **UJ-1. The Reviewer files five findings during one pass over a running app.**
  - **Persona + context:** the primary user from the brief — an agent-assisted developer, mid-review,
    with the app under test in front of them and an agent conversation waiting in another window.
  - **Entry state:** Snapdown is running in the tray. No window is open. The app under test has
    focus.
  - **Path:** presses the Capture hotkey → the screen dims and a crosshair appears → drags a box
    around a misaligned button → releases → a small Note field appears at the box → types "this
    button should be right-aligned with the card, it is 8px off" → presses save → the overlay closes
    and the app under test has focus again. Repeats four more times over the next ninety seconds.
  - **Climax:** the fifth save shows a toast reading "5 findings" with an *Open editor* action. Every
    Finding is already on disk with its Note attached; nothing had to be organised.
  - **Resolution:** the Reviewer opens the Editor and moves on to composing. Nothing was interrupted
    and no file was named by hand.
  - **Edge case:** a capture where they meant to press Escape. Pressing Escape on the overlay, or on
    the Note field, discards the Capture and writes nothing to the Vault.

- **UJ-2. The Reviewer points at three specific spots inside one screenshot.**
  - **Persona + context:** same Reviewer, now in the Editor, holding a Finding showing a form with
    three separate problems.
  - **Entry state:** Editor open, the Finding selected, its Note showing the sentence typed at
    capture time.
  - **Path:** clicks *Add marker* and clicks the first field → badge `1` appears on the image and
    line `1.` appears in the Note → types "label is truncated" → repeats for `2` and `3` → drags
    badge `2` slightly to sit beside the control rather than on it.
  - **Climax:** the Note now reads as a numbered list whose numbers are visible in the image. Nothing
    had to be described positionally.
  - **Resolution:** the Finding is ready to compose. Deleting badge `2` renumbers the third badge and
    its line to `2`, with no gap left behind.

- **UJ-3. The Reviewer composes and hands over one Bundle.**
  - **Persona + context:** same Reviewer, review finished, eleven Findings in the Library covering two
    unrelated concerns.
  - **Entry state:** Editor open, all eleven Findings listed newest first.
  - **Path:** ticks the five Findings about the checkout screen → clicks *Compose bundle* → names it
    "checkout alignment pass" → Snapdown writes the Markdown and its images → the Bundle appears in
    the Bundle list → clicks *Copy Markdown*.
  - **Climax:** pasting into the agent conversation puts five images and five notes in, each note
    under its own image, in the order the Reviewer chose.
  - **Resolution:** the six unbundled Findings are still in the Library, untouched. When the checkout
    work is done the Reviewer deletes the Bundle and its images go with it.
  - **Edge case:** composing with a Finding whose image file has gone missing from the Vault stops
    and names the Finding, rather than writing a Bundle with a broken image reference.

- **UJ-4. First run.**
  - **Persona + context:** the Reviewer, minutes after installing.
  - **Entry state:** freshly installed, nothing configured.
  - **Path:** the app opens Settings on first run only → they pick a Vault folder → accept the
    default Capture hotkey or change it because it collides with something → switch on run at
    Windows startup → close Settings.
  - **Climax:** the first hotkey press dims the screen. Nothing else was required, and no account
    was asked for.
  - **Resolution:** the tray icon is the only trace of the app until the hotkey is pressed again.
  - **Edge case:** the chosen hotkey is already taken by another process. Snapdown says so at the
    moment of choosing and refuses to save it, rather than failing silently later.

- **UJ-5. Visual markup and redaction on canvas.**
  - **Persona + context:** the Reviewer in Editor, holding a screenshot with confidential API keys and a complex UI layout.
  - **Entry state:** Editor open, Finding selected on canvas.
  - **Path:** selects Blur tool and drags a box over the API key line → selects Shape tool and drags a red outline box around a misaligned card → selects Arrow tool and drags an arrow from the callout text to the broken button → selects Callout tool and types a clarification note, adjusting its tail pointer.
  - **Climax:** the screenshot now has crystal-clear visual cues and masked credentials, but the Finding's Note lines and the composed Markdown contain strictly the numbered findings.
  - **Resolution:** burning and composing the Bundle includes the blurred and annotated image cleanly without note clutter.

## 3. Glossary

Every domain noun this document uses is defined once in `.control/product-glossary.md` and used
verbatim here: **Bundle**, **Capture**, **Capture Overlay**, **Editor**, **Finding**, **Handoff**,
**Library**, **Marker**, **Note**, **Publication**, **Quality Budget**, **Reviewer**, **Vault**.
**Access Key**, **Local API**, and **MCP Bridge** were glossary entries this document used too, until
`DEC-016` withdrew what they named on 2026-09-04.

No synonym for any of them appears anywhere in this PRD. A new noun introduced here is added to that
file in the same pass.

## 4. Features

### 4.1 Capture

**Capability:** CAP-1, CAP-10 — serves BG-2, BG-7.

**Description:** A global hotkey puts a Capture Overlay on every monitor. Precision crosshair guides, a magnifying loupe with live pixel grid/color readout, and intelligent auto-detection of windows and sub-panels (with dynamic cutout highlighting and a top-center Fullscreen shortcut) assist the Reviewer. The Reviewer can 1-click select a detected window/panel or drag a custom region, and re-select prior to saving. On release, a compact Note field appears anchored to the rectangle, pre-focused. Saving stores the Finding and dismisses the overlay, returning focus to whatever had it before. The loop is designed to be run six times in ninety seconds, so nothing in it opens a window, steals focus afterwards, or requires a decision the Reviewer did not come to make. Realizes UJ-1.

The Editor does **not** open after a Capture. A toast confirms the save, shows the running count of Findings, and offers an action to open the Editor.
`[ASSUMPTION: not auto-opening is what the Reviewer wants; a setting exists precisely because this may be wrong.]`

**Functional Requirements:**

#### FR-1: Capture a screen region from a global hotkey

The Reviewer can press a system-wide hotkey, from inside any application, and get a Capture Overlay on every connected monitor on which they can select or drag out a rectangular region. Realizes UJ-1.

**Proof of done:** From a maximised third-party window, pressing the hotkey dims all screens, and selecting a detected window/panel or dragging a box then releasing produces an image of exactly that region.

**Consequences (testable):**
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

#### FR-2: Write the Note at capture time

The Reviewer can type a Note into a field that appears at the selected region on release, and save
the Finding with that Note in one keystroke. Realizes UJ-1.

**Proof of done:** A Finding saved this way shows, in the Editor, exactly the text that was typed at
capture time against exactly the region that was dragged.

**Consequences (testable):**
- The Note field is focused on appearance; no click is needed before typing.
- Saving is possible from the keyboard alone.
- An empty Note saves successfully — a Finding with no words is still a Finding.
- Escape from the Note field discards both the Note and the Capture.
- Multi-line text is accepted and preserved verbatim, including blank lines.

#### FR-3: Capture repeatedly without leaving the loop

The Reviewer can perform a Capture immediately after the previous one, with focus returned to the
application they were reviewing and no Snapdown window in the way. Realizes UJ-1.

**Proof of done:** Six Captures in succession, each started by the hotkey from the application under
review, produce six Findings, and at no point does a Snapdown window need to be closed or dismissed.

**Consequences (testable):**
- After a save, keyboard focus is back with the window that held it before the hotkey.
- The confirmation is a transient toast that never takes focus and never needs dismissing.
- The toast states the running count of Findings and offers an action to open the Editor.
- Whether the Editor opens automatically after a Capture is a setting, off by default.

### 4.2 Image reduction

**Capability:** CAP-2 — serves BG-3.

**Description:** Every Capture is downscaled to fit within the Quality Budget's maximum long edge
and re-encoded lossily before it reaches the Vault. Reduction happens once, on the way in; the
original full-resolution pixels are not retained, because keeping them would mean the Vault grows for
no reader. The Reviewer controls the two numbers and can see what the reduction cost them.
Realizes UJ-1.
`[ASSUMPTION: agent reading cost tracks pixel area, making the long-edge cap the dominant lever.]`

**Functional Requirements:**

#### FR-4: Reduce every captured image automatically

The system reduces each captured image to the Quality Budget before storing it, with no action from
the Reviewer.

**Proof of done:** A capture of a full 4K screen is stored as a file whose long edge matches the
configured maximum and whose size is a small fraction of the unreduced capture, and it is still
legible enough to read the UI text in it.

**Consequences (testable):**
- An image already within the long-edge limit is not upscaled.
- Aspect ratio is preserved; the image is never stretched or cropped to fit.
- The budget carries **two** reduction levers, and they are different tools. A **maximum long edge**
  bounds the worst case and leaves a capture already under it alone. A **resize percentage** applies
  to every capture regardless of its size. The percentage is applied first and the maximum to the
  result — the other order would take the share off the maximum rather than off the capture, and two
  captures of different sizes would come out identical.
- Reduction never blocks the overlay from closing.
- Exactly one image file per Finding exists in the Vault after reduction.

#### FR-5: Choose a Quality Budget by naming the intent

The Reviewer chooses a Quality Budget by naming what they want of it — **Auto**, **Sharp**,
**Balanced**, or **Small** — and Auto, the shipped default, derives the long edge and the encoder
quality from each captured region rather than from a stored constant. The underlying numbers remain
settable behind an **Advanced** disclosure, and setting either moves the budget to a fifth named
state, **Custom**. Realizes UJ-4. Recorded as `DEC-004`.

**Proof of done:** A Reviewer who has never opened Advanced gets a legible, small stored image from
both a 300×120 tooltip capture and a full 4K screen capture, and Settings can state in one word which
budget produced each.

**Consequences (testable):**
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

### 4.3 The Library and the Editor

**Capability:** CAP-3 — serves BG-1.

**Description:** The Editor is one window listing every Finding, newest first, each showing its image
and its Note. Notes are editable in place. Markers are placed on an image and are bound to numbered
lines in that Finding's Note — one sequence, not two things kept in step. Findings can be selected in
bulk, which is what makes both deletion and composition possible. Realizes UJ-2, UJ-3.

**Functional Requirements:**

#### FR-6: See every Finding with its Note

The Reviewer can open the Editor and see every Finding in the Library, each with its image, its Note,
and when it was captured. Realizes UJ-3.

**Proof of done:** After six Captures, the Editor lists exactly those six Findings with the six Notes
that were typed, each against the right image.

**Consequences (testable):**
- Findings are ordered newest first.
- A Finding captured while the Editor is open appears without the Editor being reopened.
- A Finding whose image file is missing from the Vault is shown as broken rather than omitted.
- The list stays usable at several hundred Findings.

#### FR-7: Edit a Note

The Reviewer can change the text of any Finding's Note after capture, and the change persists.

**Proof of done:** Editing a Note, closing the Editor, and reopening it shows the edited text.

**Consequences (testable):**
- An edit is saved without an explicit save action.
- Clearing a Note entirely is allowed.
- The numbered lines belonging to Markers cannot be renumbered by hand — that ordering belongs to the
  Markers.

#### FR-8: Place numbered Markers bound to numbered Note lines

The Reviewer can place a numbered Marker anywhere on a Finding's image; doing so creates the matching
numbered line in that Finding's Note, ready for a sub-comment. Realizes UJ-2.

**Proof of done:** Placing three Markers on one image produces lines `1.`, `2.`, and `3.` in that
Finding's Note, and the badge numbers on the image match the line numbers exactly.

**Consequences (testable):**
- Markers are numbered from 1 upward with no gaps.
- Deleting a Marker removes its line and renumbers every Marker and line after it.
- A Marker can be repositioned without changing its number.
- Marker positions survive closing and reopening the Editor.
- The Marker's sub-comment may be empty; the numbered line still exists.

#### FR-9: Select several Findings at once

The Reviewer can select any number of Findings, with the keyboard or the mouse, and act on the
selection as a whole. Realizes UJ-3.

**Proof of done:** Selecting four of eleven Findings and choosing an action applies it to exactly
those four.

**Consequences (testable):**
- Range selection and individual toggling both work.
- The count of selected Findings is visible.
- Selecting all and clearing the selection are each one action.
- The selection survives scrolling and is cleared when the action completes.

### 4.4 Bundles

**Capability:** CAP-4 — serves BG-1.

**Description:** A Bundle is a named group of Findings composed once into one Markdown document, with
its own copy of the images. Composition is the point at which the review becomes an artifact. A
Bundle is deleted the same way a Finding is — hard, with its files. Realizes UJ-3.

**Corrected 2026-08-31.** This section used to read *"A Bundle is not edited afterwards — it is
recomposed — because a Bundle that drifts from the Findings that produced it is worse than no
Bundle"*, and it carried `[ASSUMPTION: recomposing is acceptable in place of editing a Bundle's
written Markdown.]` (filed as `OQ-12`). Both are withdrawn by `FR-40`. The reasoning behind them was
sound and survives: a Bundle must not drift from the Findings that produced it. What was wrong was
the conclusion that forbidding all editing is the only way to prevent that drift. `FR-40` edits the
Bundle's **own stored copy** and never reads or writes a Finding, so the snapshot promise is kept by
construction rather than by prohibition — which is why `BR-10` and `BR-65` both stay true under it
and are not amended.

**Functional Requirements:**

#### FR-10: Compose selected Findings into a named Bundle

The Reviewer can turn a selection of Findings into a Bundle with a name they choose, producing one
Markdown document in which each Note appears under the image it describes. Realizes UJ-3.

**Proof of done:** Composing five selected Findings under a chosen name produces one Markdown file
where the five Notes appear under their own five images, in the order the Reviewer selected.

**Consequences (testable):**
- The Bundle's Markdown references its images by relative path, and every referenced file exists.
- A Finding's Markers appear in the Bundle's image, and its numbered lines appear in its Note.
- The Bundle's name appears as the document's heading.
- Composing does not remove the Findings from the Library.
- Composition refuses, naming the Finding, if a selected Finding's image file is missing.
- One Finding may appear in more than one Bundle.

#### FR-11: List and reopen Bundles

The Reviewer can see every Bundle in the Library with its name, its Finding count, and when it was
composed, and can open one to read what it contains.

**Proof of done:** After composing two Bundles, both appear in the Bundle list with the right names
and counts, and opening either shows the Findings it holds.

**Consequences (testable):**
- Bundles are ordered newest first.
- Opening a Bundle shows the composed content, not a live view of the current Findings.
- A Bundle whose Markdown file is missing is shown as broken rather than omitted.

#### FR-12: Copy a Bundle's Markdown

The Reviewer can put a Bundle's Markdown on the clipboard in one action, ready to paste into an agent
conversation. Realizes UJ-3.

**Proof of done:** Copying a Bundle and pasting into a plain text editor yields the Bundle's complete
document, with every image link naming a location the reader of the pasted text can open.

**Consequences (testable):**
- The copied text is the Bundle's document, with no added wrapper.
- The image links in the copied text are absolute, so they resolve regardless of which folder the
  reader of the pasted text is working in.
- The Reviewer is told the copy succeeded **and** that what was copied carries absolute image paths.
  A person reading the pasted text cannot discover that for themselves, and it is the only place they
  learn that the text contains disk locations.

**Corrected 2026-08-31.** The second consequence used to read *"The image references in the copied
text are the same relative paths as in the file"*, and the proof of done used to end *"yields the
Bundle's complete Markdown, unchanged"*. Together they fixed the clipboard to folder-relative links,
which resolve for nobody reading the pasted text anywhere but inside the Bundle's own folder — so the
primary handoff path was promising something that does not arrive. This requirement now **permits**
an absolute rendering of those links. All three of those are now settled, and this is what each turned out to be. Note that `NFR-8` is
untouched throughout: it governs the **stored** file, which keeps its relative links — the two
conventions serve two different readers, and neither replaces the other.

**Settled 2026-08-31, and by test rather than by argument.** The links are rendered as forward slashes
wrapped in angle brackets. Six candidate forms were run through a CommonMark reference implementation
over three realistic Vault paths — one containing a space, one a space and parentheses, one a space and
an apostrophe. Raw Windows backslashes and bare forward slashes produce **no image at all**: the space
ends the destination. Percent-encoding works but hands the reader `Snapdown%20Vault` to decode. Both
`file:///` forms fail — and not on syntax, which is valid, but on the link-security blocklist that
real Markdown readers apply to the `file:` scheme and that a reader cannot switch off. Angle brackets
win because a conforming parser strips them and yields the path with its space intact.

**What the Reviewer is told, and what they are deliberately not told.** The toast names the paths, not
the images. Images could never travel on a text clipboard — no chat client loads a local file — so
warning about them would warn about an impossibility. What is worth saying is that the text carries
absolute disk locations, because those contain the operator's user name and go wherever the text is
pasted. The toast does **not** point the Reviewer at `Open file location`: that action sits one place
away on the same row, already visible.

#### FR-43: Open a Bundle's folder in the file manager

The Reviewer can open the folder holding a Bundle's Markdown and its images, in the operating system's
own file manager.

**Proof of done:** Choosing it on a Bundle's row opens that Bundle's own folder in the file manager,
with the Bundle's Markdown and its images visible in it.

**Consequences (testable):**
- It opens the Bundle's own folder, not the Vault root.
- It changes nothing: no file is written, moved, or renamed.
- A Bundle whose folder is missing reports that rather than opening the wrong folder.

**This is the thinnest promise in this document, and that is deliberate.** The Library row has carried
this action since the screen was designed, with nothing promising it — which the product's own rule
forbids, because an unbacked button is how the toolbar ended up showing shortcut badges that did
nothing. It gets no use case: the Reviewer does not open Snapdown in order to look at a folder, and
the decision of 2026-08-31 was that *why* someone reaches for this is deliberately not traced — it is
a power user's way out to the filesystem. The promise exists so the next reader knows the affordance
is intentional **and** intentionally unexplained.

It is also not an export. In this product **export means PDF** (`FR-39`); the Markdown path is
**Copy** (`FR-12`), and this is neither — it hands over a location, not a document.

#### FR-14: Delete a Bundle with its images

The Reviewer can delete a Bundle, and its Markdown file and its copied images leave the Vault with it.

**Proof of done:** Deleting a Bundle removes it from the Bundle list and leaves neither its Markdown
nor its images in the Vault folder.

**Consequences (testable):**
- Deletion asks for confirmation once and names the Bundle.
- The Findings the Bundle was composed from are not deleted, and become assemblable again.
- Deleting a published Bundle also ends its Publication.

**Corrected 2026-08-31.** A fourth consequence used to read *"The Reviewer can choose, in the same
confirmation, to delete the Bundle's source Findings too."* It is withdrawn. Destroying a Bundle and
destroying the captures behind it are now two deliberate acts — `FR-14` and `FR-41` — because one
click is the wrong price for the most destructive operation in this product, and both outcomes are
still reachable in two steps. The withdrawn line also claimed authority this requirement never had:
deleting a Finding is a write the `bundle` component cannot make, and `FR-14`'s own registry row
never listed it.

#### FR-40: Edit a composed Bundle's title and notes

The Reviewer can reopen a Bundle they have already composed and correct its title, its Bundle-level
notes, and the note text on the Findings inside it, saving the result as one act.

**Proof of done:** Reopening a composed Bundle, changing its title and one of its notes, and saving
produces a document whose heading and that note both read the new text, while its images and the
Findings it was composed from are untouched.

**Consequences (testable):**
- Editable: the Bundle's title, its Bundle notes, its Finding notes, and its Marker notes. Nothing
  else, and the set of images is frozen — none can be added, removed, reordered, or replaced.
- Saving rewrites the document's heading to the new title, so the title and the heading cannot
  disagree.
- Nothing is written until the Reviewer saves, and abandoning the edit leaves the Bundle as it was.
- No edit made here changes any Finding, its Note, or its Markers.
- A Bundle whose source Findings are already gone can still be edited this way.

**Not yet legal, and stated rather than assumed.** `BR-11` — *"A Bundle is never edited in place. A
change means composing a new Bundle"*, status active — still forbids this requirement at the time of
writing. `BR-11` is a cross-component rule, so narrowing it to the handoff path is not this
document's act. **This gate MUST NOT open on `FR-40` until that amendment lands.** The reasoning is
recorded with the amendment request: `AD-9`, which `BR-11` cites as its source, governs only the way
**out** of a Bundle, and its own closing clause directs a surface needing a different shape to change
the composer — which is what saving here does. `AD-9` is therefore not contradicted and is not
amended.

### 4.5 Removal

**Capability:** CAP-5 — serves BG-5.

**Description:** A review is meant to be thrown away. Deleting a Finding deletes its image file, and
there is no soft delete, no bin, and no state in which the Library holds a record pointing at a file
that is gone or a file nothing points at. Deletion is confirmed once, because it is irreversible.

Deleting a Bundle is the same promise on the other object, and it belongs with Bundles, in §4.4.
`FR-41` and `FR-42` are here rather than there for the same reason read the other way: both are
reached from a Bundle, but what they destroy is a Finding, and this is the section that promises a
Finding can be destroyed.

**Functional Requirements:**

#### FR-13: Delete Findings, and their image files with them

The Reviewer can delete one Finding or a selection of them, and each deleted Finding's image file
leaves the Vault at the same time.

**Proof of done:** Deleting three Findings removes them from the Editor and leaves no trace of their
three image files in the Vault folder.

**Consequences (testable):**
- Deletion asks for confirmation once, naming how many Findings will go.
- Deletion is refused, and nothing is removed, if the confirmation is declined.
- A Finding that belongs to a Bundle can still be deleted; the Bundle keeps its own copy of the image
  and stays readable.
- A file that cannot be deleted is reported rather than silently skipped, and the Finding stays.

#### FR-15: Report orphans

The system tells the Reviewer about files in the Vault that no Finding or Bundle points at, and about
Findings or Bundles whose files are missing, and lets them resolve each.

**Proof of done:** After a file is deleted from the Vault outside Snapdown, the Reviewer is shown that
the Finding is now broken and offered to delete the Finding.

**Consequences (testable):**
- The check runs on start and can be run on demand.
- An unreferenced file in the Vault is listed, and deleting it is one action.
- The check never deletes anything on its own.
- A clean Vault reports itself as clean rather than saying nothing.

#### FR-41: Discard the source Findings behind a Bundle, keeping the Bundle

The Reviewer can destroy the captures behind a Bundle they consider final, reclaiming the disk those
originals hold, while the Bundle itself stays readable on its own copies.

**Proof of done:** Discarding the originals behind a Bundle leaves that Bundle readable with its own
images, while its source Findings and their image files are gone from the Library and from the Vault,
and the Bundle can no longer be taken apart.

**Consequences (testable):**
- Confirmation is asked once, naming the Bundle and how many captures will go.
- The Bundle's own images and document are untouched by the act.
- After it, the Bundle offers deletion only — it can no longer return its Findings to the Library.
- A capture that another Bundle also holds is still destroyed; that other Bundle keeps its own copy.
- A file that cannot be deleted is reported rather than silently skipped.

This is deliberately **not** part of composing, and `BR-59` — *"Composing does not remove the
Findings it used from the Library"* — stays true and is not amended. Composing still leaves every
capture in place; this is a separate, later, explicit act, and that separation is the whole point.

#### FR-42: See which Bundles still hold original captures, and reclaim their disk in bulk

The Reviewer can see, in one place, every Bundle still holding its source captures and how much disk
each one is holding, and discard the originals behind several of them in one pass.

**Proof of done:** The reclaim surface lists every Bundle still holding source Findings with the disk
each one holds and a running total, and discarding a chosen set reduces the reported total by the sum
of what was discarded.

**Consequences (testable):**
- The surface is reachable from the Library and from the Vault section of Settings.
- A Bundle whose originals are already gone is not listed.
- Confirmation is asked once for the whole selected set, naming the total that will be reclaimed.
- Nothing is destroyed for a Bundle the Reviewer did not select.

### 4.6 Staying out of the way

**Capability:** CAP-6 — serves BG-6.

**Description:** Four settings, set once. Where images go, which keys trigger what, whether Snapdown
starts with Windows, and whether the Editor opens after a Capture. Nothing here is a preference
panel for its own sake — each one exists because leaving it fixed would make the tool the thing being
managed. Realizes UJ-4.

**Functional Requirements:**

#### FR-16: Choose the Vault folder

The Reviewer can choose which folder holds Finding and Bundle files, and change it later. Realizes
UJ-4.

**Proof of done:** Choosing a folder and taking a Capture puts the image file in that folder;
choosing a different folder and taking another Capture puts the new image in the new folder.

**Consequences (testable):**
- A default location is used until the Reviewer picks one, so capture works before any setup.
- A folder that cannot be written to is refused at the point of choosing.
- Changing the folder offers to move existing files, and either moves all of them or none.
- The current folder can be opened in Explorer in one action.

#### FR-17: Edit the hotkeys

The Reviewer can see and change every global hotkey Snapdown registers, and is told immediately when
a chosen combination is unavailable. Realizes UJ-4.

**Proof of done:** Changing the Capture hotkey makes the new combination work and the old one stop
working, without restarting Snapdown.

**Consequences (testable):**
- Every registered hotkey is listed with its current combination.
- A combination already held by another process is refused with a message naming the conflict.
- A hotkey can be cleared, disabling that action rather than leaving a broken binding.
- Two Snapdown actions cannot be bound to the same combination.
- A combination that fails to register at startup is reported, not swallowed.

#### FR-18: Run at Windows startup

The Reviewer can have Snapdown start when they sign in to Windows, and turn that off again. Realizes
UJ-4.

**Proof of done:** With the setting on, signing in to Windows leaves Snapdown running in the tray
with its hotkeys registered; with it off, it does not start.

**Consequences (testable):**
- **It is on after first run, without the Reviewer enabling it.** A capture tool the Reviewer has to
  remember to launch is a capture tool that is not there when the observation happens, which is the
  one moment it exists for.
- Enabling it requires no administrator rights.
- Starting this way opens no window — the tray icon only.
- The setting reflects the actual registration state, not a remembered intention. In particular, the
  control never shows an intended state before the real one has been read.
- Disabling it removes the registration rather than leaving it and ignoring it, and a later run does
  not re-enable it. The default applies to a first run, never to a Reviewer's decision.

### 4.7 The surface itself

**Capability:** CAP-9 — serves BG-7.

**Description:** The three requirements here are not features. They are the conditions under which
every other feature in this document is actually reachable, and they are written down because r1 and
r2 shipped every capability in § 4.1–§ 4.6 and the Reviewer still could not name the application they
had opened, find the Editor, or read a label. A promise the Reviewer cannot reach is not kept.

`CAP-9` is administered by `settings`, and that placement needs a word of defence because the
requirements below govern surfaces `finding` and `bundle` own. `settings` already holds the
container-level Logical Components — the startup registrar, the hotkey registrar, the settings store
— which are the app's own machinery rather than any one screen's. The window shell is machinery of
the same kind. The alternative, giving each surface its own copy of these requirements, produces three
statements that must be kept identical by hand, which is how a shell drifts.

**Functional Requirements:**

#### FR-27: Name the surface the Reviewer is looking at

Snapdown always identifies itself, and identifies which of its two personas is on screen: the tray and
the installed executable are **Snapdown**, and the workspace window titles itself **Snapdown Editor**.
Realizes UJ-4. Recorded as `DEC-003`.

**Proof of done:** A Reviewer with the workspace window open and no prior knowledge can say what the
application is called and which part of it they are in, from the window alone.

**Consequences (testable):**
- The installed executable, the tray tooltip, and the window title never disagree about the product's
  name. A test asserts the three against one source.
- The build produces exactly one desktop executable. A second one left in the output directory is a
  build failure, not clutter — a stale `desktop.exe` beside the correct `Snapdown.exe` is precisely
  what caused the Reviewer to run the wrong binary and conclude the product had no navigation.
- The workspace window is titled for its persona, not for the section currently open. Its title does
  not change as the Reviewer moves between sections.

#### FR-28: Reach every surface from every surface

Every primary surface of the Editor — Findings, Bundles, Settings, and Agent access — is reachable
from every other one, without the Reviewer knowing it exists beforehand. Realizes UJ-4.

**Proof of done:** From a cold open on any one surface, a Reviewer who has never used Snapdown reaches
each of the other three without being told how.

**Consequences (testable):**
- Navigation to every primary surface is present and visible on every primary surface.
- The surface the Reviewer is on is distinguishable from the ones they are not, by more than colour
  alone.
- Opening the Editor from the tray, from a hotkey, and after a Capture all arrive somewhere the
  Reviewer can navigate out of. None of them is a dead end.

#### FR-29: A primary surface fits the window it opens in

Nothing on a primary surface has to be scrolled to before the Reviewer knows it is there. Realizes
UJ-4.

**Proof of done:** At the window's minimum supported size, every control on every primary surface is
either visible or visibly indicated; nothing is discovered only by scrolling.

**Consequences (testable):**
- Settings presents its four groups — startup, Vault folder, Quality Budget, hotkeys — within the
  window at its minimum supported size. Agent access is **not** one of them: it is a primary surface
  of its own (`FR-28`, `inventory-screen.md` row 13), and counting it here would have it appear twice
  in one product.
- Scrolling to read *more* of a list is allowed and expected. Scrolling to discover that a control
  *exists* is not, and the distinction is what this requirement turns on.
- No layout gives a group vertical space it does not use in order to match a neighbour's height. The
  shipped Settings screen paired a one-checkbox group with a four-control group in equal columns and
  left roughly a third of the screen empty to do it.


### 4.8 Canvas Visual Annotations and Privacy Redaction

**Capability:** CAP-11 — serves BG-1, BG-7.

**Description:** While numbered Markers serve as the single structured bridge between visual screenshots and numbered lines in Markdown notes, visual walkthroughs and bug reports frequently require drawing attention to specific components, guiding spatial flow, or masking sensitive data (passwords, tokens, customer emails). Snapdown supports five visual overlay elements: transparent outlined Shapes, directional Arrows, Callout bubbles with font/tail control, floating Text, and Blur redaction boxes. These elements are interactive, resizable, orderable front-to-back, and rendered directly onto the burnt image files without producing lines in the Markdown notes. Realizes UJ-5.

**Functional Requirements:**

#### FR-30: Draw transparent outlined shapes and directional arrows on canvas
The Reviewer can select a Shape tool (transparent fill with theme-accent outline) or an Arrow tool from the canvas toolbar, click-drag on the screenshot canvas to create it, and reposition or resize it.
- **Proof of done:** Dragging a box produces a transparent rectangle with an outline; dragging an arrow produces a directional arrow from origin to release point. Both elements can be selected, moved, and deleted, and neither adds lines to the Markdown note.
- **Consequences (testable):**
  - Shape box has transparent fill and solid stroke border.
  - Arrow has a start point, shaft line, and triangular arrow head at the end point.
  - Active elements show transformation handles.
  - Deleting via Delete/Backspace keyboard keys or context menu removes the element cleanly.

#### FR-31: Draw blur redaction boxes over sensitive image regions
The Reviewer can drag blur redaction boxes over confidential screen contents (credentials, PII, tokens).
- **Proof of done:** Drawing a blur box immediately renders a blurred/pixelated preview over that rectangle on canvas, and burning the image permanently modifies those pixel values in the exported/saved image.
- **Consequences (testable):**
  - Underlying screenshot pixels are redacted in both visual display and final burnt PNG.
  - Blur rectangle supports standard 8-point resizing and dragging handles.
  - Multiple blur areas can overlap or exist independently.

#### FR-32: Place floating text and callout bubbles with font and tail controls
The Reviewer can add floating text labels or callout bubbles onto the canvas, edit the text content inline, and change font family and size.
- **Proof of done:** Double-clicking text/callout enters inline editing; font size and family can be customized; callout tail anchor can be dragged to point to any spot; Markdown notes remain unchanged.
- **Consequences (testable):**
  - Callout consists of a text bubble container plus an adjustable pointer tail pointing to a target coordinate.
  - Floating text has a transparent background with legible text fill.
  - Font size and font family can be adjusted per text/callout element.
  - Text contents are preserved across editor sessions.

#### FR-33: Transform and manipulate canvas annotation elements with interactive handles
The Reviewer can manipulate any placed canvas annotation element using visual control points.
- **Proof of done:** Clicking an element highlights it with dedicated control handles (8 bounding box handles for Shape/Blur/Text/Callout body, 2 endpoints for Arrow, 1 tail point for Callout); dragging handles transforms the element.
- **Consequences (testable):**
  - Active state displays distinct resize/transform handles.
  - Element can be moved by dragging within its bounds.
  - Pressing Escape deselects the active element.
  - Redo/Undo history is supported for canvas additions, moves, edits, and deletions.

### 4.9 Getting a capture in and out by hand

**Capability:** CAP-1, CAP-3, CAP-9 — serves BG-1, BG-7.

**Description:** Four behaviours the Reviewer asked for on 2026-08-28 that the product had no promise
for. They are recorded here in the order the corpus should have had them, and the record says plainly
that it did not: `OQ-29` opened because seven behaviours were requested with no `FR-` covering any of
them. Three of those seven are not here — `undo`/`redo` was already promised inside FR-33, and `crop`
and destructive `resize` are named non-goals in §5 and in the Product Brief, so they are refused
rather than promised.

None of the four changes what a Finding IS. They are ways into and out of one.

**Functional Requirements:**

#### FR-34: Zoom the canvas to inspect a capture at more or less than natural size
The Reviewer can zoom the canvas in and out and return it to natural size.
- **Proof of done:** The image grows and shrinks on screen; a Marker placed at 200% is on the same
  pixel when the canvas returns to 100%; the file on disk is unchanged by any of it.
- **Consequences (testable):**
  - Zoom is a view state, never a stored one. Reopening a Finding shows it at natural size.
  - Marker and annotation coordinates stay normalised to the image, so nothing drifts with the zoom.
  - Placing a Marker while zoomed puts it where the pointer is, not where the pointer would have been
    at 100%.

#### FR-35: Paste an image from the clipboard as a new Finding
The Reviewer can paste an image held on the Windows clipboard and get a Finding from it.
- **Proof of done:** With an image on the clipboard, Paste produces a Finding carrying those pixels;
  it appears in the filmstrip and can be noted, marked and bundled like any capture.
- **Consequences (testable):**
  - The pasted image goes through the same Quality Budget a Capture does. A pasted 4K screenshot is
    reduced exactly as a captured one would be, or BG-3 has a second door with no lock on it.
  - A clipboard holding no image says so and does nothing, rather than creating an empty Finding.
  - Its `source_monitor` says it was pasted. A Finding that claims a monitor it never came from is a
    small lie the Reviewer would have no way to catch.

#### FR-36: Copy a Finding's burned image to the clipboard
The Reviewer can copy the image the way an agent would receive it.
- **Proof of done:** Copy puts the image on the clipboard with its Markers and annotations burned in;
  pasting it into a chat shows the numbered badges and the redaction boxes.
- **Consequences (testable):**
  - The bytes are what a Bundle would carry — the same burn, not a second rendering path that could
    disagree with it.
  - A redaction that is in the file is in the clipboard copy. This requirement is the one place the
    Reviewer can hand over an image without a Bundle, and a blur that only existed in the Bundle
    would make that path leak.

#### FR-37: Reach a canvas or filmstrip action from a right-click context menu
The Reviewer can right-click the canvas or a filmstrip card and reach the actions that apply to it.
- **Proof of done:** Right-clicking a filmstrip card offers that Finding's own actions; right-clicking
  an annotation offers Delete; every entry does the same thing as the control of the same name.
- **Consequences (testable):**
  - No entry exists that has no other route. A context menu is a shortcut to what is already
    reachable, never the only way to reach something — NFR-16 forbids a control that has to be
    discovered.
  - The menu names the thing under the pointer. A right-click on empty canvas and a right-click on a
    Callout do not offer the same list.

#### FR-38: Change the front-to-back order of canvas annotations
The Reviewer can move any placed annotation forward or backward through the others.
- **Proof of done:** With two overlapping annotations, sending the upper one to the back puts the
  other in front of it — on the canvas and in the burned image, which must agree.
- **Consequences (testable):**
  - Four movements: to the front, forward one, backward one, to the back.
  - The captured image is always underneath every annotation. It is not in the order and cannot be
    moved within it — it is the thing being annotated, not an annotation.
  - Order is stored, so it survives closing the Editor.
  - A movement that changes nothing writes nothing.

### 4.10 Handing a review to a person

**Capability:** CAP-12 — serves BG-2.

**Description:** Every handoff path this product had was built for a machine to read. A Bundle can be
copied as Markdown or, in the `agent-handoff` initiative, served to an agent — but a Reviewer who
needs to send the same review to a colleague, attach it to a ticket, or keep it where Snapdown is not
installed has nothing. This is the one path whose reader is a person.

The agent path is unaffected and stays the primary one: `FR-12` is what an agent gets, and nothing
here re-renders what `FR-12` serves.

**Functional Requirements:**

#### FR-39: Export a Bundle as a PDF

The Reviewer can turn a Bundle into a single PDF file that reads as a document — one section per
Finding, its image above the note about it — which anyone can open without Snapdown.

**Proof of done:** Exporting a Bundle produces one A4 PDF a person can read without Snapdown, in
which each Finding appears as its own section with its image whole on a single page, the pages are
numbered, and the text can be selected and searched rather than being a picture of text.

**Consequences (testable):**
- One column, A4 only. The Bundle's title and its composition date open the document as a title
  block, not as a cover page of its own.
- Every page is numbered.
- No image is ever split across a page break.
- The text is a real text layer: selecting, searching and copying all work, and a machine reading the
  file is not obstructed by it.
- Exporting changes nothing about the Bundle, and the same Bundle exported twice produces the same
  document.

**Deliberately not stated here.** How the PDF is produced — the engine, its licence, its cost on disk
and in memory, how a very tall screenshot is fitted to a page, and how text is escaped on the way in
— is settled work but it is solution shape, and this document does not carry solution shape.

## 5. Non-Goals (Explicit)

- Visual annotation elements (Shapes, Arrows, Callouts, Text, Blur) MUST NOT generate or alter structured lines in Markdown notes. Numbered Markers remain the sole structured finding bindings.
- Snapdown is not a full-blown graphics illustration app (no gradient mesh, freehand pencil drawings, Bézier path editors).
- Snapdown does not read what is in a screenshot. No OCR, no classification, no auto-captioning. The
  Reviewer's judgement is the payload.
- **Editing the captured pixels after the fact — crop, rotate, or destructive resize.** Asked for on
  2026-08-28 and refused, because it was already refused: the Product Brief and `SRS-finding` both
  name it. It is load-bearing rather than merely tidy — `AD-9` promises a Bundle's image copy is
  byte-identical to the Finding when nothing is drawn on it, and the Vault keeps no second copy, so a
  destructive edit is the one operation in this product that cannot be undone by any means. The
  Quality Budget is where an image's size is decided, once, on the way in. Reversing this is a `DEC-`,
  not a story.
- Snapdown is not a video or GIF recorder.
- Snapdown is not multi-user. No accounts, no permissions, no shared Library, no second Reviewer.
- Snapdown is not a sync client. Nothing leaves the machine as a background activity.
- The Library is not agent-writable. This initiative produces nothing an agent can change.

## 6. MVP Scope

### 6.1 In Scope

- Region capture from an editable global hotkey, with the Note written at capture time.
- Automatic image reduction under a Quality Budget the Reviewer controls.
- The Editor: the Finding list, Note editing, numbered Markers, multi-select.
- Bundles: compose, list, open, copy Markdown, correct a composed Bundle's title and notes, and
  export one as a PDF for a person to read.
- Hard deletion of Findings and of Bundles, with their files, plus orphan reporting, plus discarding
  the captures behind a finished Bundle and a surface that does it in bulk.
- Vault folder, hotkeys, run-at-startup, and open-Editor-after-capture as settings.

### 6.2 Out of Scope for MVP

- Reordering Findings inside a Bundle after composition. Recompose instead — the ordering is the
  selection order, and a second ordering mechanism is a second source of truth.
- Searching or filtering the Library. `[NOTE FOR PM]` This is the first thing that will hurt once
  the Library passes a few hundred Findings; revisit for r2.
- Tags or folders over Findings. Bundles are the only grouping in r1.
- A second Vault, or switching between Vaults. One at a time.
- Any Handoff path beyond copying the Markdown and exporting a PDF. MCP and web publishing are the
  `agent-handoff` initiative.

**Two entries left this list on 2026-08-31**, and what they used to say is recorded rather than
quietly dropped:

- *"Renaming a Bundle. Same reason; and a rename that does not rewrite the document's heading is a
  lie."* Grown into `FR-40`. The scope boundary applied and has been lifted; **the objection never
  applied at all**, and that is worth stating because it reads as a reason. `FR-40` rewrites the
  document's heading when the title changes, so the rename it promises is not the lie this line was
  guarding against. The "same reason" it borrowed — a second source of truth — does not survive
  either: the title and the heading are one field written once, not two that can disagree.
- *"Exporting a Bundle to anything but Markdown."* Grown into `CAP-12` and `FR-39`. It was an MVP
  boundary rather than a judgement about value, and it held for as long as every reader of a Bundle
  was a machine.

## 7. Success Metrics

**Primary**

- **SM-1**: Time from first hotkey press to a five-Finding Bundle on the clipboard, median over the
  Reviewer's real reviews — target under 120 seconds. Validates FR-1, FR-2, FR-3, FR-10, FR-12.
- **SM-2**: Notes attached to the wrong image, per review — target zero, always. Validates FR-2,
  FR-6, FR-8, FR-10.

**Secondary**

- **SM-3**: Stored bytes per Finding against the unreduced capture — target a small fraction, at a
  quality the Reviewer does not override. Validates FR-4, FR-5.
- **SM-4**: Files left in the Vault that nothing points at, after a month of use — target zero.
  Validates FR-13, FR-14, FR-15.
- **SM-5**: Findings per Handoff — expected to rise above one, since batching is the behaviour the
  product is trying to enable. Validates FR-9, FR-10.

**Counter-metrics (do not optimize)**

- **SM-C1**: Bytes per Finding. Driving this down further past the point where UI text stops being
  legible destroys the only reason the image is there. Counterbalances SM-3.
- **SM-C2**: Time in the Editor. Low time here would mean Notes are not being improved and Markers
  are not being placed, which is the work that makes a Bundle worth reading. Counterbalances SM-1.

## 8. Open Questions

1. Should a Finding be removable from the Library while staying inside a Bundle that already holds
   its own copy of the image? FR-13 currently says yes; the alternative is refusing the deletion.
2. ~~What is the right default long edge?~~ **Restated by `DEC-004`.** There is no longer a default
   long edge to be right or wrong; Auto derives one per capture. The open question becomes: is Auto's
   output legible at its smallest? Still unmeasured, still filed as OQ-3.
5. Are four named Quality Budgets distinguishable enough that a Reviewer picks between them rather
   than leaving Auto forever? If not, Advanced and Custom are cost with no buyer, and `DEC-004`'s own
   reversal trigger fires. Filed as OQ-18.
6. Are Snagit and Cobalt Capture the right experience benchmark for `BG-7`, given that both are built
   for a human reader and Snapdown's reader is a machine? This sits underneath `CAP-9` itself. Filed
   as OQ-20.
3. Does the Reviewer want the Editor to open after the first Capture of a session, as a middle
   position between always and never? Deferred until the default has been lived with. Filed as OQ-9.
4. ~~Should composing a Bundle offer to delete the Findings it consumed, the way deleting a Bundle
   offers to delete its Findings?~~ **Answered 2026-08-31, and its premise is gone.** Deleting a
   Bundle no longer offers to delete its Findings — `FR-14` was corrected — so there is no longer a
   behaviour for composing to be consistent with. The want underneath the question is met by `FR-41`
   as a separate act, and coupling it to composing is refused for the same reason: destroying
   captures is not a side effect of anything.
7. Is `BG-2` a close enough goal for `CAP-12` that a sixth business goal is not needed? `BG-2`
   measures handoff *time* and promises no file management; a PDF is a file to manage, read by a
   person rather than an agent. Filed as OQ-31, and it may cost `BG-2` a measure amendment at G1.

## 9. Assumptions Index

- §4.1 — not auto-opening the Editor after a Capture is what the Reviewer wants. Filed as OQ-9.
- §4.2 — agent reading cost tracks image pixel area, making the long-edge cap the dominant lever.
  Filed as OQ-2.
- §4.4 — ~~recomposing a Bundle is acceptable in place of editing its written Markdown (OQ-12)~~.
  **Withdrawn 2026-08-31 by `FR-40`**, which is this assumption turning out false. `OQ-12` is closed
  in place rather than deleted; the row and its reasoning stay where they were filed.
- §4.10 — `BG-2` is a close enough goal for `CAP-12` that a sixth business goal is not needed. Filed
  as OQ-31.
- Carried from the brief and still load-bearing here: a coding agent can open relative image paths
  (OQ-1); a 1600 px long edge stays legible (OQ-3); numbered Markers are sufficient annotation
  (OQ-4); Windows hotkeys need no administrator rights (OQ-5); one Vault at a time is enough (OQ-11).

## Cross-Cutting NFRs

- **NFR-1** — serves BG-2. The Capture Overlay is visible within 200 ms of the hotkey press, on a
  machine with three monitors. Enforced by a timed test in the capture component.
- **NFR-2** — serves BG-2. Saving a Finding dismisses the overlay and returns focus within 500 ms,
  with image reduction completing without blocking that. Enforced by a timed test in the capture
  component.
- **NFR-3** — serves BG-3. Every stored Finding image fits within the Quality Budget's long edge, and
  the shipped default keeps a full-screen 4K capture under 250 KB. Enforced by an assertion over the
  stored file in the image-reduction tests.
- **NFR-4** — serves BG-2. No part of the capture path performs a network call, and capture works with
  networking disabled. Enforced by a test that runs the capture path with no network available.
- **NFR-5** — serves BG-5. After any deletion, the Vault holds no file that the Library does not point
  at, and the Library points at no file that is absent. Enforced by an invariant check in the
  deletion tests and by the orphan report of FR-15.
- **NFR-6** — serves BG-6. Snapdown reaches a usable tray icon with its hotkeys registered within 3
  seconds of a cold start, and idles under 150 MB of working set with the Editor closed. Enforced by
  a startup timing test and a measured idle check.
- **NFR-7** — serves BG-6. Every hotkey registration and every startup registration succeeds without
  administrator rights. Enforced by running the integration suite as a standard user.
- **NFR-8** — serves BG-1. A Bundle's Markdown renders in a plain CommonMark reader, with every image
  reference resolving relative to the Markdown file's own folder. Enforced by a rendering test over
  a composed Bundle.
- **NFR-16** — serves BG-7. Every text element on every surface meets WCAG AA contrast against its
  own background, in both the Windows light and the Windows dark theme. Enforced by an automated
  contrast assertion run over both themes, not by inspection. This is the requirement the shipped
  build fails: colour values were hard-coded for a light theme inside components whose tokens follow
  `prefers-color-scheme`, so the two disagree wherever they meet.
- **NFR-17** — serves BG-7. No colour is defined only for one theme. Every surface renders correctly
  under either Windows theme setting and under a change of that setting while running. Enforced by
  a test that renders every screen in both themes and by a lint rule that refuses a literal colour
  outside the token file.
- **NFR-18** — serves BG-3 and BG-7. The parameters Auto resolved for a Capture are stored with that
  Finding, so a Finding can always say what produced it and a change to the derivation cannot
  silently rewrite the past. Enforced by an assertion that every stored Finding carries its resolved
  budget. This exists because `FR-5` forbids re-encoding an existing Finding: without the record,
  two Findings taken a month apart on "the same" setting differ with nothing to explain why.
- **NFR-19** — serves BG-2. An exported PDF carries a real text layer, and no image in it is broken
  across a page boundary except by the deliberate slicing a very tall image requires. **Corrected
  2026-08-31**, hours after it was written: it first said "no image in it is split across a page
  break", which forbade exactly what the tall-image research behind `FR-39` prescribes — above a
  certain aspect ratio a screenshot is drawn at full text width and cut per page, embedded once. What
  the requirement protects is the *accidental* split, an image landing half on one page because it
  fell near the boundary. That is still forbidden. Enforced by a test that extracts the text back out
  of the produced PDF and
  asserts the notes it should contain, plus a placement check read from the PDF's own image
  matrices. Asserting a `%PDF-` signature and a page count does **not** satisfy this: a fabricated
  header passes that, and this product has already spent five waves reporting a requirement met on
  the strength of a correct number inside a fabrication.

## Constraints and Guardrails

### Safety

Nothing beyond the brief. There is no destructive action here other than deletion, which is
confirmed once and named in FR-13, FR-14, FR-41 and FR-42. `FR-41` and `FR-42` are the most
destructive acts the product offers, because what they remove is the original capture rather than a
copy of it — which is why no single action destroys both a Bundle and its originals, and why both are
reachable only in two deliberate steps.

### Privacy

- A Capture may contain personal data, so nothing in this initiative may transmit, upload, or
  telemeter a Finding, a Note, or a Bundle. There is no crash reporter that carries content and no
  usage analytics.
- The Vault is a plain folder of files the Reviewer chose. It is not encrypted, and the product must
  not imply that it is.

### Cost

Beyond the brief: none. This initiative has no runtime cost, no account, and no service behind it.

### Beyond the brief

Everything else that binds here is already a product-wide constraint in
`.what/_product-brief/brief.md` — Windows-only capture, a public repository that never carries a
captured screenshot, no account or network as a precondition for capturing, and hard deletion. None
of them is restated.
