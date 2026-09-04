---
title: Capture to Markdown
initiative: capture-to-markdown
created: "2026-08-22"
updated: "2026-09-04"
---

# PRD: Capture to Markdown

> **This is the working PRD.** It cites requirement ids instead of repeating their text, so §3 lists
> `FR`/`NFR` by id, and there is no Glossary, Non-Goals, Open Questions, or Assumptions Index section
> here — each of those facts has its own home.
>
> **To read or hand over one complete, self-contained document, run `/wdi-report render prd`.**
> It writes `.what-rendered/_prd/capture-to-markdown/prd.md` with the capabilities, the requirement
> statements and proofs of done, the glossary terms this PRD uses, the non-goals, and the open
> questions filled in from their own homes. That file is regenerated, never hand-edited.

## Revision History

| Date | What changed | Why | Releases affected |
|---|---|---|---|
| 2026-08-22 | Initial version | The desktop review loop is the product's core; nothing else can be handed off until findings exist | r1 |
| 2026-08-23 | Added § 4.7 (`CAP-9`, `FR-27`–`FR-29`) and `NFR-16`–`NFR-18`; rewrote `FR-5`; amended `FR-18` | r1 and r2 shipped every promised capability, and the first sustained use produced a list of experience defects rather than missing features. `BG-7` now carries that, and this initiative gains the requirements that make it checkable | r3 |
| 2026-08-31 | Closed the last of ticket 03's open items. `FR-43` promises opening a Bundle's folder — an action the Library row already carried with nothing behind it. `FR-12` now states that the Reviewer is told the copied Markdown carries absolute image paths, and its encoding is settled by test rather than argument. `NFR-19` corrected: written the same morning, it forbade the very page-slicing that the tall-image research it rests on prescribes. One wording fix in § 2.3, where a journey called the Markdown an "export" — in this product **export means PDF**, and the Markdown path is **Copy**. | A row action with no requirement cannot be specced at all, and the clipboard promise was silent about the one thing a Reviewer cannot discover for themselves: the pasted text contains absolute disk paths, which carry the operator's user name. `NFR-19` was a self-contradiction introduced hours earlier and would have been carried into the first spec that cited it. | r3 |
| 2026-08-31 | Grew four promises the Bundle Library needs and corrected two that contradicted them. **New:** § 3.10 Export PDF (`CAP-12`, `FR-39`, `NFR-19`), `FR-40` editing a composed Bundle's title and notes, `FR-41` discarding a Bundle's source Findings, `FR-42` the reclaim-space surface. **Removed from § 4.2:** the non-goals "Renaming a Bundle" and "Exporting a Bundle to anything but Markdown". **Corrected:** `FR-12` no longer fixes the clipboard to relative image links, and `FR-14` no longer offers to destroy a Bundle's source Findings in the same confirmation. | A Reviewer can assemble Bundles but has almost no way to live with them afterwards — no way to fix a typo without recomposing, no way to hand one to a person rather than an agent, and no way to reclaim the disk an archived review is holding. The two corrections are older promises that pointed the opposite way from these four, and leaving them would have made this document promise two designs at once. `FR-41` and `FR-42` sit under `CAP-5` rather than `CAP-4` because they destroy Findings, which the `bundle` component has no authority to write. | r3 |
| 2026-09-03 | § 2.3's canvas-annotation journey renumbered `UJ-5` → `UJ-7`, and registered in `requirements-capture-to-markdown.yaml` for the first time | It had carried `UJ-5` since it was written without ever being registered, and that id was already allocated to `agent-handoff`'s own `UJ-5` — a collision found and flagged rather than silently resolved | — |
| 2026-09-03 | `bmad-review` pass: `FR-40`'s gate note updated now that `BR-11` was narrowed by `DEC-012` and FR-40 is legal (§4.1 already listed it as delivered); `FR-38` moved from § 3.9 to § 3.10's sibling § 3.8, matching its own registered capability; three Capability lines' `serves` lists corrected to match the registry (`CAP-12`→`BG-8`, `CAP-11` drops `BG-7`, § 3.9 gains `BG-2`, `NFR-18` drops `BG-7`); two stale Revision History section citations fixed and the table's row order corrected; the two `wdi-upgrade` tooling comments moved to the memlog; `FR-41`'s duplicated paragraph, `FR-39`'s superseded page-break bullet, and `FR-12`'s "still unsettled" paragraph (all three questions it named are answered a few paragraphs earlier in the same addendum) corrected | Compliance pass against `prd-guide.md`; every change above is a correction to something already decided elsewhere in the corpus, not a new promise | — |
| 2026-09-04 | `wdi-upgrade` (0.5.15 → 0.6.1): §0 Document Purpose, §3 Glossary, §8 Open Questions, and §9 Assumptions Index removed — each fact now lives in its own home (`product-glossary.md`, `.control/questions/`). Every `#### FR-N` block's Proof of done and Consequences moved to `requirements-capture-to-markdown.yaml` (`proof:`) and `addendum.md` (already holding the same content, confirmed word for word); each feature now cites `FR`/`NFR` ids only, under **Realizes:**. Sections renumbered 1–7 to the current template's order | Mechanical structural migration; no promise, proof, or consequence text changed | — |

## 1. Why This Initiative

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

- **UJ-7. Visual markup and redaction on canvas.**
  - **Persona + context:** the Reviewer in Editor, holding a screenshot with confidential API keys and a complex UI layout.
  - **Entry state:** Editor open, Finding selected on canvas.
  - **Path:** selects Blur tool and drags a box over the API key line → selects Shape tool and drags a red outline box around a misaligned card → selects Callout tool and types a clarification note, adjusting its tail pointer → selects Arrow tool and drags an arrow from the callout to the broken button.
  - **Climax:** the screenshot now has crystal-clear visual cues and masked credentials, but the Finding's Note lines and the composed Markdown contain strictly the numbered lines.
  - **Resolution:** burning and composing the Bundle includes the blurred and annotated image cleanly without note clutter.

## 3. Features

### 3.1 Capture

**Capability:** CAP-1, CAP-10 — serves BG-2, BG-7.

**Description:** A global hotkey puts a Capture Overlay on every monitor. Precision crosshair guides, a magnifying loupe with live pixel grid/color readout, and intelligent auto-detection of windows and sub-panels (with dynamic cutout highlighting and a top-center Fullscreen shortcut) assist the Reviewer. The Reviewer can 1-click select a detected window/panel or drag a custom region, and re-select prior to saving. On release, a compact Note field appears anchored to the rectangle, pre-focused. Saving stores the Finding and dismisses the overlay, returning focus to whatever had it before. The loop is designed to be run six times in ninety seconds, so nothing in it opens a window, steals focus afterwards, or requires a decision the Reviewer did not come to make. Realizes UJ-1.

The Editor does **not** open after a Capture. A toast confirms the save, shows the running count of Findings, and offers an action to open the Editor.
`[ASSUMPTION: not auto-opening is what the Reviewer wants; a setting exists precisely because this may be wrong.]`

**Realizes:** FR-1, FR-2, FR-3

### 3.2 Image reduction

**Capability:** CAP-2 — serves BG-3.

**Description:** Every Capture is downscaled to fit within the Quality Budget's maximum long edge
and re-encoded lossily before it reaches the Vault. Reduction happens once, on the way in; the
original full-resolution pixels are not retained, because keeping them would mean the Vault grows for
no reader. The Reviewer controls the two numbers and can see what the reduction cost them.
Realizes UJ-1.
`[ASSUMPTION: agent reading cost tracks pixel area, making the long-edge cap the dominant lever.]`

**Realizes:** FR-4, FR-5

### 3.3 The Library and the Editor

**Capability:** CAP-3 — serves BG-1.

**Description:** The Editor is one window listing every Finding, newest first, each showing its image
and its Note. Notes are editable in place. Markers are placed on an image and are bound to numbered
lines in that Finding's Note — one sequence, not two things kept in step. Findings can be selected in
bulk, which is what makes both deletion and composition possible. Realizes UJ-2, UJ-3.

**Realizes:** FR-6, FR-7, FR-8, FR-9

### 3.4 Bundles

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

**FR-12 — Copy a Bundle's Markdown.** **Corrected 2026-08-31.** The second consequence used to read
*"The image references in the copied text are the same relative paths as in the file"*, and the proof
of done used to end *"yields the Bundle's complete Markdown, unchanged"*. Together they fixed the
clipboard to folder-relative links, which resolve for nobody reading the pasted text anywhere but
inside the Bundle's own folder — so the primary handoff path was promising something that does not
arrive. This requirement now **permits** an absolute rendering of those links. Note that `NFR-8` is
untouched throughout: it governs the **stored** file, which keeps its relative links — the two
conventions serve two different readers, and neither replaces the other.

**FR-14 — Delete a Bundle with its images.** **Corrected 2026-08-31.** A fourth consequence used to
read *"The Reviewer can choose, in the same confirmation, to delete the Bundle's source Findings
too."* It is withdrawn. Destroying a Bundle and destroying the captures behind it are now two
deliberate acts — `FR-14` and `FR-41` — because one click is the wrong price for the most destructive
operation in this product, and both outcomes are still reachable in two steps. The withdrawn line
also claimed authority this requirement never had: deleting a Finding is a write the `bundle`
component cannot make, and `FR-14`'s own registry row never listed it.

**FR-43 — Open a Bundle's folder in the file manager.** **This is the thinnest promise in this
document, and that is deliberate.** The Library row has carried this action since the screen was
designed, with nothing promising it — which the product's own rule forbids, because an unbacked
button is how the toolbar ended up showing shortcut badges that did nothing. It gets no use case: the
Reviewer does not open Snapdown in order to look at a folder, and the decision of 2026-08-31 was that
*why* someone reaches for this is deliberately not traced — it is a power user's way out to the
filesystem. The promise exists so the next reader knows the affordance is intentional **and**
intentionally unexplained.

It is also not an export. In this product **export means PDF** (`FR-39`); the Markdown path is
**Copy** (`FR-12`), and this is neither — it hands over a location, not a document.

**FR-40 — Edit a composed Bundle's title and notes.** **Legal as of 2026-08-31.** `BR-11` used to read
*"A Bundle is never edited in place. A change means composing a new Bundle"*, which forbade this
requirement outright. `DEC-012` narrowed it, closing `OQ-12`: `BR-11` now permits a Bundle's stored
document to be changed by the composer writing it again over the Bundle's own copy, provided no
surface edits it directly and no change to a Bundle ever reads or writes a Finding — exactly what
`FR-40` does. `AD-9`, which `BR-11` cites as its source, governs only the way **out** of a Bundle, so
it was not contradicted and was not amended; only `BR-11`'s own wording was.

**Realizes:** FR-10, FR-11, FR-12, FR-14, FR-43, FR-40

### 3.5 Removal

**Capability:** CAP-5 — serves BG-5.

**Description:** A review is meant to be thrown away. Deleting a Finding deletes its image file, and
there is no soft delete, no bin, and no state in which the Library holds a record pointing at a file
that is gone or a file nothing points at. Deletion is confirmed once, because it is irreversible.

Deleting a Bundle is the same promise on the other object, and it belongs with Bundles, in §3.4.
`FR-41` and `FR-42` are here rather than there for the same reason read the other way: both are
reached from a Bundle, but what they destroy is a Finding, and this is the section that promises a
Finding can be destroyed.

**FR-41 — Discard the source Findings behind a Bundle, keeping the Bundle.** This is deliberately
**not** part of composing, and `BR-59` — *"Composing does not remove the Findings it used from the
Library"* — stays true and is not amended. Composing still leaves every capture in place; this is a
separate, later, explicit act, and that separation is the whole point.

**Realizes:** FR-13, FR-15, FR-41, FR-42

### 3.6 Staying out of the way

**Capability:** CAP-6 — serves BG-6.

**Description:** Four settings, set once. Where images go, which keys trigger what, whether Snapdown
starts with Windows, and whether the Editor opens after a Capture. Nothing here is a preference
panel for its own sake — each one exists because leaving it fixed would make the tool the thing being
managed. Realizes UJ-4.

**Realizes:** FR-16, FR-17, FR-18

### 3.7 The surface itself

**Capability:** CAP-9 — serves BG-7.

**Description:** The three requirements here are not features. They are the conditions under which
every other feature in this document is actually reachable, and they are written down because r1 and
r2 shipped every capability in § 3.1–§ 3.6 and the Reviewer still could not name the application they
had opened, find the Editor, or read a label. A promise the Reviewer cannot reach is not kept.

`CAP-9` is administered by `settings`, and that placement needs a word of defence because the
requirements below govern surfaces `finding` and `bundle` own: `settings` is where the app's own
machinery lives, rather than any one screen's — see `addendum.md` for the component-level reasoning.
The alternative, giving each surface its own copy of these requirements, produces three statements
that must be kept identical by hand, which is how a shell drifts.

**Realizes:** FR-27, FR-28, FR-29

### 3.8 Canvas Visual Annotations and Privacy Redaction

**Capability:** CAP-11 — serves BG-1.

**Description:** While numbered Markers serve as the single structured bridge between visual screenshots and numbered lines in Markdown notes, visual walkthroughs and bug reports frequently require drawing attention to specific components, guiding spatial flow, or masking sensitive data (passwords, tokens, customer emails). Snapdown supports five visual overlay elements: transparent outlined Shapes, directional Arrows, Callout bubbles with font/tail control, floating Text, and Blur redaction boxes. These elements are interactive, resizable, orderable front-to-back, and rendered directly onto the burnt image files without producing lines in the Markdown notes. Realizes UJ-7.

**Realizes:** FR-30, FR-31, FR-32, FR-33, FR-38

### 3.9 Getting a capture in and out by hand

**Capability:** CAP-1, CAP-3, CAP-9 — serves BG-1, BG-2, BG-7.

**Description:** Four behaviours the Reviewer asked for on 2026-08-28 that the product had no promise
for. They are recorded here in the order the corpus should have had them, and the record says plainly
that it did not: `OQ-29` opened because seven behaviours were requested with no `FR-` covering any of
them. Three of those seven are not here — `undo`/`redo` was already promised inside FR-33, and `crop`
and destructive `resize` are named non-goals in the product brief's Scope Out, so they are refused
rather than promised.

None of the four changes what a Finding IS. They are ways into and out of one.

**Realizes:** FR-34, FR-35, FR-36, FR-37

### 3.10 Handing a review to a person

**Capability:** CAP-12 — serves BG-8.

**Description:** Every handoff path this product had was built for a machine to read. A Bundle can be
copied as Markdown or, in the `agent-handoff` initiative, served to an agent — but a Reviewer who
needs to send the same review to a colleague, attach it to a ticket, or keep it where Snapdown is not
installed has nothing. This is the one path whose reader is a person.

The agent path is unaffected and stays the primary one: `FR-12` is what an agent gets, and nothing
here re-renders what `FR-12` serves.

**Realizes:** FR-39

## 4. MVP Scope

### 4.1 In Scope

- Region capture from an editable global hotkey, with the Note written at capture time.
- Automatic image reduction under a Quality Budget the Reviewer controls.
- The Editor: the Finding list, Note editing, numbered Markers, multi-select.
- Bundles: compose, list, open, copy Markdown, edit a composed Bundle's title and notes, and
  export one as a PDF for a person to read.
- Hard deletion of Findings and of Bundles, with their files, plus orphan reporting, plus discarding
  the captures behind a finished Bundle and a surface that does it in bulk.
- Vault folder, hotkeys, run-at-startup, and open-Editor-after-capture as settings.

### 4.2 Out of Scope for MVP

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

## 5. Success Metrics

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

## 6. Cross-Cutting NFRs

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
  two Rust guards over `apps/desktop/ui/theme.slint`, the palette that actually ships: a contrast test
  over every token in both themes, and a test refusing a colour literal in the capture overlay. (Read
  *"a test that renders every screen in both themes and by a lint rule that refuses a literal colour
  outside the token file"* until 2026-09-01 — that lint covered only `web/ui/src`, deleted under
  `OQ-27`, and never the Slint surfaces that are the whole shipped UI.)
- **NFR-18** — serves BG-3. The parameters Auto resolved for a Capture are stored with that
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

## 7. Constraints and Guardrails

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
