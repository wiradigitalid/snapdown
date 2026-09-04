# 02: Undo and redo treat a Marker like any other shape

**What to build:** Ctrl+Z reverses what the Reviewer just did to a Marker, and Ctrl+Y puts it back —
the same promise annotations already keep. Delete a Marker, undo, and it comes back where it was,
with the number it had.

Three actions become reversible, which is full parity with what an annotation edit records:

- **Placing** a Marker — undo removes it
- **Dragging** a Marker — undo returns it to where it was
- **Deleting** a Marker — undo brings it back, at its original ordinal

Marker note text is **excluded**, and deliberately: that field writes through on every keystroke, so
recording it would put one undo step per letter on the stack.

Two things make this more than a copy of the annotation case, and they are the substance of the
ticket:

**The history is one stack, not two.** Ctrl+Z has to reverse the most recent thing the Reviewer did,
whichever kind it was — so a Marker edit is a new variant on the existing per-Finding edit history,
not a parallel stack beside it. A parallel stack would let Ctrl+Z skip past a newer annotation edit
to reach an older Marker one, which is not what the keystroke promises. The history stays scoped to
one Finding and stays in memory, unchanged.

**Restoring a Marker restores its number, and that needs two calls.** Adding a Marker always appends
it at the end of the numbering. So re-adding a deleted Marker 2 lands it last, and the Reviewer's
Note would silently renumber under them — the exact damage that made Markers draggable in the first
place. The reorder port method already exists and takes a full ordering; undo captures the ordering
at the moment of deletion and replays it after the re-add. No new store method is needed.

Redo is the same walk in the other direction and must be exercised, not assumed: an undone deletion
sits on the redo stack as a placement, and redoing it must delete the Marker again *and* leave the
numbering as the original deletion left it.

**Blocked by:** 01 — the Delete key path is what makes marker deletion reachable from the keyboard,
and the no-op-click guard from 01 is what stops a bare click recording a spurious move on the stack.

**Status:** ready-for-agent

- [ ] Place a Marker, undo — it is gone; redo — it is back
- [ ] Drag a Marker, undo — it returns to its former position; redo — it is where the drag left it
- [ ] Delete a Marker, undo — it is back **at its original ordinal**, with its note text intact, and
      the Marker Notes list and the Markdown line numbers agree
- [ ] Delete Marker 2 of 4, undo, and Markers 1–4 read 1, 2, 3, 4 in their original order
- [ ] Redo after undoing a deletion removes it again and leaves the same numbering the first deletion
      produced
- [ ] A Marker edit and an annotation edit interleave correctly: undo reverses them in the order they
      happened, newest first, across both kinds
- [ ] A click on a Marker that moves it nowhere records no undo step
- [ ] Editing a Marker's note records no undo step
- [ ] Opening a different Finding clears the history, as it already does for annotations
- [ ] Nothing to undo still reports nothing to undo
- [ ] Tests cover the delete-undo-redo round trip through the store, asserting the restored ordinal,
      not only that the Marker exists again
