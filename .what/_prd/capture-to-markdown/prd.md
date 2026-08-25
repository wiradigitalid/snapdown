---
title: Capture to Markdown
initiative: capture-to-markdown
created: "2026-08-22"
updated: "2026-08-22"
---

# PRD: Capture to Markdown

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-08-22 | Initial version | The desktop review loop is the product's core; nothing else can be handed off until findings exist | r1 |
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

## 3. Glossary

Every domain noun this document uses is defined once in `.control/product-glossary.md` and used
verbatim here: **Access Key**, **Bundle**, **Capture**, **Capture Overlay**, **Editor**, **Finding**,
**Handoff**, **Library**, **Local API**, **Marker**, **MCP Bridge**, **Note**, **Publication**,
**Quality Budget**, **Reviewer**, **Vault**.

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
Bundle is not edited afterwards — it is recomposed — because a Bundle that drifts from the Findings
that produced it is worse than no Bundle. A Bundle is deleted the same way a Finding is — hard, with
its files. Realizes UJ-3.
`[ASSUMPTION: recomposing is acceptable in place of editing a Bundle's written Markdown.]`

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
Markdown, unchanged.

**Consequences (testable):**
- The copied text is the Bundle's Markdown exactly, with no added wrapper.
- The image references in the copied text are the same relative paths as in the file.
- The Reviewer is told the copy succeeded.

#### FR-14: Delete a Bundle with its images

The Reviewer can delete a Bundle, and its Markdown file and its copied images leave the Vault with it.

**Proof of done:** Deleting a Bundle removes it from the Bundle list and leaves neither its Markdown
nor its images in the Vault folder.

**Consequences (testable):**
- Deletion asks for confirmation once and names the Bundle.
- The Findings the Bundle was composed from are not deleted.
- Deleting a published Bundle also ends its Publication.
- The Reviewer can choose, in the same confirmation, to delete the Bundle's source Findings too.

### 4.5 Removal

**Capability:** CAP-5 — serves BG-5.

**Description:** A review is meant to be thrown away. Deleting a Finding deletes its image file, and
there is no soft delete, no bin, and no state in which the Library holds a record pointing at a file
that is gone or a file nothing points at. Deletion is confirmed once, because it is irreversible.
Deleting a Bundle is the same promise on the other object and belongs with Bundles, in §4.4.

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


## 5. Non-Goals (Explicit)

- Snapdown is not a capture tool for a human audience. No arrows, no callouts, no blur, no
  redaction, no freehand drawing, no effects. Numbered Markers are the only annotation, in this
  release and after it.
- Snapdown is not an image editor. Captured pixels are never cropped, rotated, or resized after
  capture.
- Snapdown does not read what is in a screenshot. No OCR, no classification, no auto-captioning. The
  Reviewer's judgement is the payload.
- Snapdown is not a video or GIF recorder.
- Snapdown is not multi-user. No accounts, no permissions, no shared Library, no second Reviewer.
- Snapdown is not a sync client. Nothing leaves the machine as a background activity.
- The Library is not agent-writable. This initiative produces nothing an agent can change.

## 6. MVP Scope

### 6.1 In Scope

- Region capture from an editable global hotkey, with the Note written at capture time.
- Automatic image reduction under a Quality Budget the Reviewer controls.
- The Editor: the Finding list, Note editing, numbered Markers, multi-select.
- Bundles: compose, list, open, copy Markdown.
- Hard deletion of Findings and of Bundles, with their files, plus orphan reporting.
- Vault folder, hotkeys, run-at-startup, and open-Editor-after-capture as settings.

### 6.2 Out of Scope for MVP

- Reordering Findings inside a Bundle after composition. Recompose instead — the ordering is the
  selection order, and a second ordering mechanism is a second source of truth.
- Renaming a Bundle. Same reason; and a rename that does not rewrite the document's heading is a lie.
- Searching or filtering the Library. `[NOTE FOR PM]` This is the first thing that will hurt once
  the Library passes a few hundred Findings; revisit for r2.
- Tags or folders over Findings. Bundles are the only grouping in r1.
- A second Vault, or switching between Vaults. One at a time.
- Exporting a Bundle to anything but Markdown.
- Any Handoff path beyond copying the Markdown. MCP and web publishing are the `agent-handoff`
  initiative.

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
4. Should composing a Bundle offer to delete the Findings it consumed, the way deleting a Bundle
   offers to delete its Findings? Left out of r1 deliberately.

## 9. Assumptions Index

- §4.1 — not auto-opening the Editor after a Capture is what the Reviewer wants. Filed as OQ-9.
- §4.2 — agent reading cost tracks image pixel area, making the long-edge cap the dominant lever.
  Filed as OQ-2.
- §4.4 — recomposing a Bundle is acceptable in place of editing its written Markdown. Filed as OQ-12.
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

## Constraints and Guardrails

### Safety

Nothing beyond the brief. There is no destructive action here other than deletion, which is
confirmed once and named in FR-13 and FR-14.

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
