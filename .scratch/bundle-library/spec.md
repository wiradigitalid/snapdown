# Spec: Bundle Library

**Status:** ready-for-agent
**Source:** `map.md` and its nine resolved tickets (`issues/01`–`09`). Every decision below was taken
there; this document assembles them into one buildable statement and adds only the implementation
decisions a builder needs. Written 2026-09-02, the day the map's exit condition was met.
**Corpus ids this rests on:** `FR-12` `FR-14` `FR-28` `FR-40` `FR-41` `FR-42` `FR-43` · `UC-29`
`UC-30` `UC-31` · `AD-2` `AD-9` `AD-11` · `DEC-005` `DEC-012` · `BR-5` `BR-10` `BR-11` `BR-12`
`BR-59` `BR-122` · `NFR-8` `NFR-16` `NFR-17`. The corpus is an input, never a gate: where a document
and the code disagree, the code wins and the document is corrected afterwards.

---

## Problem Statement

A Reviewer assembles Findings into a Bundle — one Markdown document plus its own copies of the images
— and then **cannot find it again inside Snapdown**. There is no screen that lists Bundles. The Editor
has a Library icon whose click does nothing. Once composed, a Bundle can only be reached by knowing
where the Vault is on disk and opening the folder by hand.

That makes four promised things impossible from inside the product:

- **Handing a Bundle to an agent.** The Markdown exists on disk, but the one gesture that gets it into
  a chat or an agent's context — copying it — has no button anywhere (`FR-12` has never been
  implemented).
- **Correcting a Bundle.** A typo in a review that has already been put together can only be fixed by
  composing the whole thing again (`FR-40`, `UC-29`).
- **Getting rid of a Bundle**, or of the screenshots behind one you are finished with (`FR-14`,
  `FR-41`, `UC-30`), and **seeing which Bundles are still holding disk** (`FR-42`, `UC-31`).
- **Knowing which surface you are on.** `FR-28` promises every primary surface reachable from every
  other; the Bundles surface is one of the four and it does not exist. This is one of the two
  remaining items keeping the G2 experience bar unmet, which is what keeps cloud Publish frozen
  (`DEC-005`).

Two smaller things the missing screen has been hiding: the Editor's ribbon carries a **Share** button
with nothing behind it and an **Assemble** button that acts on the filmstrip rather than on the canvas
it sits beside; and the ribbon's **Copy** copies an image but does not say so.

## Solution

**The Library**: a full-window overlay in the Editor, opened from the Library icon, listing every
Bundle newest-first with a thumbnail, its name and a one-line meta readout. Hovering a row reveals
its two everyday actions — **Copy Markdown** and **Open file location**; everything else lives in a
menu the row's overflow button and a right-click both open. Clicking the row opens the Bundle in
**Review & Update**, the same window used to assemble, but opened **locked** as a faithful view of
the Bundle as composed; **Edit** unlocks its title and notes, **Save** writes them back through the
same composer that wrote the document the first time.

A Bundle has **two states and the menu says which**. While its source Findings still exist:
**Disassemble…** (the Bundle goes, the Findings come back to the filmstrip) and **Discard
originals…** (the Findings go, the Bundle stays and is sealed). Once the originals are gone, only
**Delete…**. There is deliberately **no single button that destroys both**; that outcome exists, with
its own confirmation, but is reached through a second step. A **Reclaim space** screen, reached from
the Library's header and from Settings' Vault area, does Discard originals in bulk with the disk each
Bundle holds and a running total.

Copy Markdown puts the **whole stored document** on the clipboard with image links rewritten to
**absolute paths** so a local agent can open them, and the toast says the copied text carries disk
locations. An **edited Bundle says so**: it gains a last-edited time, shown in its row only when it
differs from the time it was composed; the list order does not change.

The ribbon loses Share and Assemble and its Copy becomes **Copy Image**.

**Export PDF and Publish are not built by this spec.** Export PDF is its own effort (the engine is
chosen, the packaging judgement and one owner question — installer size — are deliberately deferred to
it); Publish is frozen by `DEC-005`. Neither may appear in the menu as a control that does nothing —
see *Implementation Decisions*.

## User Stories

**Finding my Bundles**

1. As a Reviewer, I want a Library screen that lists every Bundle I have composed, so that a review I
   finished last week is one click away instead of a folder hunt.
2. As a Reviewer, I want the Library to open from the Library icon I already see in the Editor, so
   that I do not learn a new place to look.
3. As a Reviewer, I want the Library to lie over the Editor rather than replace it, so that closing it
   returns me to exactly the canvas, selection and scroll I left.
4. As a Reviewer, I want each row to show a thumbnail of the Bundle's first image, its name and how
   many Findings it holds, so that I recognise a review by sight before I read its name.
5. As a Reviewer, I want each row's meta line to say when the Bundle was composed in plain relative
   words ("yesterday", "last week"), so that I orient by memory rather than by parsing a timestamp.
6. As a Reviewer, I want Bundles ordered newest-composed first, so that the review I just made is at
   the top.
7. As a Reviewer with no Bundles yet, I want the empty Library to tell me the next action — tick
   Findings in the strip and press Assemble — so that emptiness reads as a starting point rather than
   a dead end.
8. As a Reviewer, I want the Library to show skeleton rows in the list's own shape while it loads, so
   that nothing jumps when the real rows arrive.
9. As a Reviewer, I want the Library to tell me plainly when it could not be read and what refused,
   with **Try again** and **Open file location** beside the message, so that a locked database is a
   thing I can act on rather than a blank screen.
10. As a Reviewer, I want the Library's header to state how many Bundles I have, so that "6 Bundles"
    confirms nothing is missing.

**Handing a Bundle over**

11. As a Reviewer, I want **Copy Markdown** on a row's hover, so that getting a review into an agent
    or a chat is one gesture from the list.
12. As a Reviewer, I want the copied Markdown to be the whole document exactly as composed — same
    words, same order — so that what the agent reads is what I reviewed.
13. As a Reviewer, I want the image links in the copied text to be absolute paths a local agent can
    open, so that the agent sees the screenshots and not broken references.
14. As a Reviewer, I want the toast after copying to tell me the text carries the images' locations
    on my disk, so that I know what travels with it if I paste it somewhere shared.
15. As a Reviewer, I want **Open file location** on a row's hover, so that I can reach the Bundle's
    folder — its Markdown and images together — when I want the files themselves.
16. As a Reviewer, I want the stored Markdown file to keep its folder-relative image links even
    though the clipboard gets absolute ones, so that moving my Vault does not break my Bundles.

**Correcting a Bundle**

17. As a Reviewer, I want clicking a row to open the Bundle in Review & Update, so that reading and
    correcting a review start from the same place.
18. As a Reviewer, I want Review & Update to open **locked**, showing the Bundle exactly as composed
    with no editing affordances, so that opening a Bundle to read it cannot change it by accident.
19. As a Reviewer, I want the window's header to state how many Findings the Bundle holds and when it
    was composed, with a badge that reads **As composed** or **Editing**, so that I always know which
    mode I am in.
20. As a Reviewer, I want an **Edit** button that unlocks the Bundle's title, its notes, each
    Finding's note and each Marker's note, so that a typo in any of them is a direct fix.
21. As a Reviewer, I want the images in edit mode to carry a small **Fixed at compose** chip, so that
    I understand why I cannot add, remove, reorder or replace them here.
22. As a Reviewer, I want **Save** to be always clickable once I am editing, so that I never wonder
    whether the button knows about my change.
23. As a Reviewer, I want the toast after Save to distinguish "Saved" from "Saved. Nothing had
    changed.", so that a click with no edits is not a mystery.
24. As a Reviewer, I want **Cancel** to return the window to locked immediately when I typed nothing,
    and to ask me first only when I did, so that leaving is cheap and losing work is not.
25. As a Reviewer, I want editing a Bundle's note to leave the original Finding's note untouched,
    so that a correction to one review does not silently rewrite the Finding every other Bundle
    shares (`BR-10`, `BR-11`).
26. As a Reviewer, I want to correct a Bundle whose original Findings I have already discarded, so
    that sealing a Bundle does not freeze its text.
27. As a Reviewer, I want a Bundle's row to show **edited <when>** beside its composed time only when
    the two differ, so that I can see which reviews I have touched since composing without every row
    growing a second date.
28. As a Reviewer, I want a Save that changed nothing to leave the edited time alone, so that the row
    does not claim an edit that did not happen.
29. As a Reviewer, I want the Library's order to stay newest-composed first even after I edit an old
    Bundle, so that fixing a typo does not throw a month-old review to the top of the list.

**Getting rid of things**

30. As a Reviewer, I want the row menu of a Bundle whose Findings still exist to offer
    **Disassemble…** and **Discard originals…**, so that the two things I might mean by "delete" are
    named apart.
31. As a Reviewer, I want **Disassemble** to remove the Bundle and its image copies and return its
    Findings to the filmstrip with notes and markers intact, so that a review I want to redo costs me
    nothing I captured.
32. As a Reviewer, I want **Discard originals** to remove the source Findings and their files while
    the Bundle keeps its own copies and stays readable, so that I can reclaim the disk behind a review
    I am finished with.
33. As a Reviewer, I want a Bundle whose originals are gone to show only **Delete…**, so that the menu
    never offers a Disassemble that cannot give anything back.
34. As a Reviewer, I want the outcome that destroys both the Bundle and its originals to exist but to
    need a second, explicit step, so that the most destructive act in the product is never one click
    away.
35. As a Reviewer, I want every one of these confirmations to name the Bundle, count what goes, say
    what comes back (or that nothing does), and state that it cannot be undone, so that I confirm a
    specific act and not a generic "Are you sure?".
36. As a Reviewer, I want the confirmation for Discard originals to tell me when another Bundle
    shares those Findings and will be sealed too, so that I am not surprised later by a Disassemble
    that has quietly disappeared elsewhere (`BR-12`, `BR-122`).
37. As a Reviewer, I want a failed deletion to leave me with something I can see and retry — never a
    Bundle still listed whose images are already gone — so that a crash mid-delete is recoverable
    (`AD-2`).
38. As a Reviewer, I want a **Reclaim space** screen listing every Bundle still holding original
    captures with the disk each one holds and a running total, so that I can see where my disk went.
39. As a Reviewer, I want to tick several Bundles in Reclaim space and discard their originals in one
    confirmed act, with the footer telling me how many are selected and how much will be freed.
40. As a Reviewer, I want Reclaim space reachable from both the Library header and Settings' Vault
    area, so that I find it whether I am thinking about reviews or about disk.
41. As a Reviewer, I want Reclaim space to say "Nothing to reclaim" and why when no Bundle holds
    originals, so that an empty list is an answer and not a bug.

**The Editor around it**

42. As a Reviewer, I want the ribbon's **Copy** renamed **Copy Image**, so that the button says what it
    copies — the burned image of the Finding on the canvas, never Markdown.
43. As a Reviewer, I want the ribbon's **Share** button gone, so that the Editor stops offering an
    action that does nothing.
44. As a Reviewer, I want the ribbon's **Assemble** button gone, because Assemble acts on the
    filmstrip selection and the filmstrip footer already carries it in the right place; the ribbon
    should act on the canvas.
45. As a Reviewer, I want the one button left in the ribbon's action group sized and placed so it
    looks deliberate rather than like a gap where two buttons used to be.
46. As a Reviewer, I want every new surface — the Library, Review & Update's locked mode, the
    confirmations, Reclaim space — to read correctly in both the light and the dark theme, with every
    text element meeting AA contrast (`NFR-16`, `NFR-17`).
47. As a Reviewer who has never used Snapdown, I want to reach the Library from the Editor and the
    Editor from the Library without being told how (`FR-28`).

## Implementation Decisions

**Scope of this spec, and what is deliberately left out of it.** The Library, Review & Update in
update mode, the two-state deletion lifecycle and its four confirmations, Reclaim space, Copy
Markdown, Open file location, the last-edited time, and the ribbon rework. **Export PDF is a separate
effort** and gets its own spec: its engine is chosen (ticket 07), but the research itself deferred the
packaging judgement to that effort and one owner question — whether ~60 MiB of installer is
acceptable — is still open. **Publish is frozen** by `DEC-005`.

**No control that does nothing.** The map's own rule: an unbacked button repeats the mistake of the
toolbar's fake 1–6 shortcut badges, removed the same week. Therefore the row menu **does not render
Export PDF until the exporter exists, and does not render Publish until `DEC-005` lifts** — neither a
greyed row nor a "soon" marker. Ticket 01 drew a greyed "Publish — soon" at the owner's request and
then flagged, in its own text, that it should not ship that way. The menu is built so that both rows
can be added without touching the row layout, which is the reason ticket 01 chose a menu. *The owner
may overrule this and ask for the greyed marker; the spec records the recommendation and the reason.*

**The Library is a full-window overlay over the Editor**, the same pattern as Review & Assemble, so
the Editor's state survives underneath. Closed from the header's X and by Escape. Header: title,
Bundle count, a Reclaim space entry. The header's "All Bundles" and "Newest first" from the artboards
are **static readouts of what the list is, not controls** — there is no search, filter or sort in
this release, and a control that cannot be operated would break the rule above.

**A row is one Bundle**: thumbnail (always the BundleItem at position 1), name, meta line. The meta
line reads `N Findings · composed <relative time>` and appends ` · edited <relative time>` only when
the last-edited time differs from the composed time. Two hover actions — Copy Markdown, Open file
location — and an overflow button; right-click on the row opens the same menu, matching the gesture
the filmstrip already teaches. Clicking the row opens Review & Update.

**Menu order and the two states.** **Corrected 2026-09-02, found by `/code-review` against tickets
10-19's landed code.** This paragraph used to list the menu as *"Edit · Copy Markdown · Open file
location · [Export PDF] — [Publish] — then the destructive group"*. Tickets 11/12/16/17's own
acceptance checklists never asked for an Edit row, and none of the four implementer agents that built
the menu added one: Edit is reached by clicking the row into Review & Update (ticket 13) and pressing
its footer's Edit button (ticket 14), not from the row menu. The built order is: Copy Markdown · Open
file location · [Export PDF] — [Publish] — then the destructive group, which depends on state.
**Whether a Bundle still holds its
Findings is read live from whether those Findings exist, never from a stored flag** (`BR-122`): a
Bundle is *unsealed* when every one of its BundleItems' Findings still exists and *sealed* otherwise.
Unsealed: **Disassemble…**, **Discard originals…**. Sealed: **Delete…** only. The **Delete both**
outcome (Bundle and originals, "nothing comes back to the filmstrip") is offered as a second-step
choice from within the Disassemble confirmation, never as a menu row.

**The four confirmations** carry the copy the artboards settled: each names the Bundle in quotes,
counts what goes, states what comes back or that nothing does, and ends "This cannot be undone." The
cancel verb keeps what the act would destroy ("Keep it", "Keep them"); the confirm verb is the act
("Disassemble", "Discard", "Delete both", "Delete"). The Discard originals confirmation additionally
names any other Bundle that shares one of those Findings and will therefore be sealed by the act —
that consequence is real today (`BR-12` + `BR-122`) and the confirmation is the only place to say it.
The `FR-41` wording in the corpus will be brought into line afterwards; the safer behaviour is built
first.

**Write ordering on delete follows `AD-2`: the record first, then its files.** Disassemble and Delete
remove the Bundle's row (its BundleItems cascade) and only then the Bundle's folder. A crash between
the two leaves image files nothing points at — the recoverable state, which the Vault's orphan sweeper
already owns — and never a listed Bundle whose images are gone. Ticket 02 had chosen the opposite
order and was corrected by review; if files-first is ever wanted again, that narrows `AD-2` and a
`DEC-` is mandatory. **Disassemble writes no Finding**: the Findings reappear because the filmstrip
filters out whatever a Bundle holds, and the Bundle no longer holds them. **Discard originals** deletes
each source Finding through the existing whole-Finding deletion path (record, then files, per
Finding) and touches nothing on the Bundle — sealing is a consequence of the Findings being gone,
not a write.

**Review & Update opens locked, and it never reads a Finding.** This is the one architectural
decision this spec adds, and it falls straight out of two facts: `BR-11` — *no change to a Bundle
ever reads or writes a Finding* — and the sealed state, in which the Findings **do not exist**. So
the window cannot be fed from Findings the way Review & Assemble is. **The composer gains the inverse
of what it already does**: it can read its own document back into the block structure it emits
(title · bundle notes · per Finding: image, note, marker notes). Review & Update is built from that
parse of the stored document alone; edits stay in a buffer; Save hands the edited blocks back to the
composer, which writes the document again. Round-tripping is a hard requirement: parsing a document
the composer wrote and serialising it again must reproduce it exactly, and the existing golden
document is the first fixture for that. This keeps `AD-9` and `DEC-012` intact — the composer, and
only the composer, produces a Bundle's document — and it is why a sealed Bundle is as editable as an
unsealed one.

**Editable fields are exactly four**: Bundle title, Bundle notes, each Finding's note, each Marker's
note. Images are frozen — no add, remove, reorder or replace — and carry a **Fixed at compose** chip
in edit mode only. Locked mode shows no affordance of any kind on any element. The header carries the
static provenance line and the **As composed / Editing** badge. There is no Edit/Preview pair: locked
*is* the preview. Footer: locked → primary **Edit**, secondary Close; editing → primary **Save**
(always enabled), secondary **Cancel**, which returns to locked at once when the buffer equals the
stored document and confirms first when it does not.

**Save writes both the Bundle's stored document and its file, and its title, under `BR-5`.** The store
gains an update operation that writes name and document together; the existing document-only update
that nothing calls is subsumed by it. Ordering: the file is written first, atomically (a temporary
file beside it, then a rename), then the row. If the row write fails, the previous file content —
held in memory for exactly this — is written back, the Reviewer is told which part refused, and the
edited buffer survives in the window so Save can be tried again. A Save whose edited blocks serialise
to a document identical to the stored one **and** whose title is unchanged writes nothing and toasts
"Saved. Nothing had changed."

**A Bundle gains a last-edited time (ticket 09, option B).** One new column, backfilled for every
existing Bundle with its composed time, so an untouched Bundle reads as never edited and that is
true. It moves **only when Save actually changed the stored document or the title**; the no-op Save
above leaves it alone, which is what keeps ticket 05's always-clickable Save free of visible side
effects. It is shown in the Library row only when it differs from the composed time and in Review &
Update's header the same way. **The list order is unchanged** — newest-composed first.

**Copy Markdown is the composer rebasing its own document.** `AD-9` as narrowed by `DEC-012`: every
handoff path serves the same authored document, a path may substitute the base of its image links so
they resolve for its own reader, and that substitution is made by the composer — no surface rewrites
a document the composer produced. So the clipboard text is produced by asking the composer for the
stored document with its image links rebased to **absolute paths, forward slashes, wrapped in `<>`**
(ticket 03, settled against a CommonMark reference implementation over Vault paths containing
spaces, parentheses and an apostrophe; `file:///` forms are rejected by readers' default `file:`
scheme blocklist and must not be used). Nothing else differs from the stored document. The stored
file keeps folder-relative links (`NFR-8`). The toast follows the house pattern of saying what did
and did not travel: it states that the copied text carries the images' locations on this disk. The
OS clipboard call is the thinnest possible layer around a string the composer produced.

**Open file location opens the Bundle's own folder** in the file manager — the folder holding its
Markdown and its image copies — using the existing open-a-folder path, not the select-a-file path,
because the object is the folder (`FR-43`). Why a Reviewer reaches for it is deliberately not traced.

**Reclaim space** is a second full-window overlay, reached from the Library header and from
Settings' Vault area (a new entry beside the existing Vault path controls). It lists every unsealed
Bundle — name, count of original captures, relative composed time, and the disk its **original**
Findings' files occupy — with a header total and a checkbox per row. The footer reads `N of M
selected · X MB will be freed`, Cancel, and **Discard originals**, which runs the per-Bundle Discard
originals act for each ticked Bundle behind one confirmation that counts Bundles and captures. Its
empty state states that no Bundle is holding original captures and why. Sizes are measured from the
files on disk, not estimated.

**Ribbon rework.** Share is removed, with its excuse row in the callback-reachability ratchet (the
test fails if the excuse outlives the button). Ribbon Assemble is removed; the filmstrip footer's
Assemble and the context-menu entry remain, so Assemble keeps two doors. Copy is renamed Copy Image
and keeps its behaviour. The surviving button is sized and placed so the group reads as intentional;
that is the rewritten ribbon-sizing ticket, which this spec unblocks.

**The Library icon's click opens the Library**, and the stub that printed a line leaves the
known-stubs ratchet (the test fails if a stub that started doing real work stays listed). The Bundles
Drawer's stub is **not** touched — it is out of scope and a different, undescribed pattern.

**Every surface is built from the shared components and the design tokens** — the modal header,
action button, context menu, checkbox, text field and the theme's tokens — with the exact values the
artboards in the design folder record and its README traces to the running app. Thumbnails sit on
the canvas ground and are theme-invariant, like the capture overlay. No colour literal appears in any
new component (the existing contrast gate refuses one), and every new text/background pairing is
asserted by that gate in both themes.

**Reachability is a shipped test, not a review item.** The repository's signature failure is a
component built, unit-tested and mounted nowhere. Each new component ships with a test asserting
that something instantiates it and that each of its callbacks is bound in Rust — the shape of the
existing annotation-wiring test.

**Corpus follow-ups this work will owe, none of which gates it**: two screen-inventory rows (the
Library, Reclaim space) and a use-case list update; `FR-41`'s confirmation wording; the last-edited
time as an amendment to the Bundle's domain model and the store's design; the callback-reachability
and known-stubs lists shrink. Documents follow the code.

## Testing Decisions

**A good test here asserts what a Reviewer or an agent can observe — the document produced, the rows
listed, the files present, the toast text — never how it was produced.** The highest seam in this
feature is the **composer's document**: most of the behaviour above (edit, no-op detection, rebasing
links, the sealed case) is a pure transformation over that document and is tested there, with no UI
and no database.

**Seam 1 — the composer (core, pure).**
- Round-trip: parse(serialize(doc)) reproduces the document byte-for-byte, over the existing golden
  document and over generated documents with every field populated, empty, and containing Markdown
  metacharacters.
- Rebasing: the clipboard rendering differs from the stored document **only** in image link
  destinations, which are absolute, forward-slashed, `<>`-wrapped, and correct for Vault paths with a
  space, with parentheses, and with an apostrophe. Asserted by diffing the two renderings, not by
  inspecting the code.
- No-op detection: an edit buffer that equals the stored blocks serialises to an identical document.
- Prior art: the golden Markdown test and the `NFR-8` image-resolution test.

**Seam 2 — the store (SQLite + Vault, real files in a temp dir).**
- Migration adds the last-edited column and backfills it with the composed time; opening a database
  written by the previous version is the fixture.
- Update writes name and document together; the last-edited time moves only when either changed.
- Delete removes the row before the files; a failure injected between the two leaves the files and
  no row, never the reverse.
- Discard originals deletes the Findings and leaves the Bundle's row, items, document and image copies
  byte-identical.
- Prior art: the bundle-store tests, the bundle-deletion test, the finding-deletion-leaves-Bundle
  tests already present.

**Seam 3 — the desktop (source-reading tests, the house style).**
- Callback reachability: every callback the Library, Review & Update and Reclaim space declare is
  bound in Rust; `library-clicked` has left the known-stubs list; `share-bundle-clicked` has left the
  excused list. The existing ratchet test does this and fails in both directions.
- Component wiring: each new component is instantiated somewhere and its callbacks bound — the
  annotation-wiring test's shape, one file per component.
- Design system: the new surfaces use the shared header, button, menu and checkbox components and no
  colour literal; every new text/background pairing passes AA in both themes through the existing
  contrast gate.
- Prior art: the callback-reachability test, the annotation-wiring test, the design-system test, the
  theme-contrast test.

**Seen red before trusted.** The core of this feature includes several guards — round-trip, no-op,
write ordering, the two ratchets releasing. Each is broken deliberately, watched fail, and restored;
a verification run uses `--no-fail-fast` so a first failure cannot hide a second.

**What no test here can vouch for, and the owner is told to look at**: the Library's look in both
themes; the four confirmations reading correctly; Explorer actually opening the Bundle's folder; the
clipboard contents pasted into a real Markdown reader resolving to real images; the Fixed-at-compose
chip and the As composed / Editing badge. Each implementation ticket names what to look at.

## Out of Scope

- **Export PDF** — separate effort with its own spec; the engine choice, escaping strategy, tall-image
  handling and measurements in ticket 07 are its inputs. Until it exists, no Export PDF row renders.
- **Publish / cloud** — frozen by `DEC-005` until the G2 experience bar is met and verified; two of
  its four failing items were fixed on 2026-09-02, this Library is a third, the fourth is the owner's
  call on Settings tabs. Until it lifts, no Publish row renders.
- **Search, filter, sort, pagination, virtualisation** in the Library — r2 per the corpus.
- **Editing images in a Bundle** — add, remove, reorder, replace, re-annotate. Images are fixed at
  compose.
- **Changing the compose-time flow** — Review & Assemble's write-through of note and marker edits to
  the live Finding is deliberately left alone.
- **The Bundles Drawer** — a separate stubbed toggle; nothing asked for it.
- **The Local API / MCP bridge** (`BUG-59`) and any `SNAPDOWN_VAULT_PATH` mechanism — different
  effort.
- **A Markdown export to a second local folder** — the Vault copy is already portable; Copy Markdown
  and Open file location are the hand-off.
- **Raw Markdown editing** in Review & Update — the four fields are the only edit surface, which is
  what makes the composer's round-trip a closed problem.

## Further Notes

- **The sealed state is legal on purpose.** Migration v6 dropped the BundleItem → Finding foreign
  key so that a Bundle can outlive its Findings. Everything above that reads "does this Finding still
  exist" depends on that.
- **`FR-12` has never had an implementation**, so every decision about Copy Markdown was made on paper.
  `OQ-1` — whether an agent handed the Markdown can actually open the images — is the first thing to
  observe once the button exists; the absolute-link decision acts on that risk without having tested
  it.
- **Assembling copies, it never moves.** A Finding's image survives assembly untouched; the Bundle
  gets a burned copy. That is why Disassemble gives Findings back for free and why Discard originals
  leaves the Bundle readable.
- **Two prerequisites the map named**: the broken relative image links are fixed (`BUG-86`); the
  filmstrip's "Copy image" targeting the wrong Finding is a separate open defect and is not fixed
  here.
- **The design source** is the artboards in the design folder beside this spec; their README records
  which running-app component each value was lifted from, and warns that the light artboard is
  derived from the dark one by token substitution and must be regenerated rather than edited.
- **Corpus contradictions found on the way and left for the document track**: none new. Ticket 08's
  list of owed corpus work stands and none of it gates this spec.
