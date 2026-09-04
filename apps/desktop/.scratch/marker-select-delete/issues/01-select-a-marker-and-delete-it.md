# 01: Select a Marker on the canvas and delete it with the Delete key

**What to build:** the Reviewer can click a numbered Marker on the canvas, see that it is
selected, and press Delete to remove it.

Today a Marker has no selected state at all. It can be placed, dragged, and removed through its
right-click menu, but the canvas's idea of "the thing you have picked up" covers annotations only —
so `Delete` does nothing when a Marker is what the Reviewer is looking at, and there is no visual
answer to *which one would go*.

The gesture, end to end:

- **(a)** Click the Marker to delete. It shows as selected, the same selection language the canvas
  already speaks for boxes and arrows: a ring on the reticle in the selection-ring token. The
  inspector switches to the Marker Notes tab and that Marker's row is highlighted, the way picking up
  a Callout opens the tab that can change it.
- **(b)** Press Delete.
- **(c)** The Marker is gone, the survivors renumber, and the Marker Notes list follows — identical
  to what the right-click *Delete Marker* entry already does, because it goes through the same
  deletion path. Nothing is selected afterwards.

**Backspace deliberately does not delete.** The canvas is surrounded by note fields and a Callout's
words are typed on the canvas itself, so Backspace already means "delete a letter" there. The
annotation path made this choice explicitly and this one matches it.

**Prefactor, first, in this ticket.** A plain click on a Marker currently commits a move: the drag
begins on every pointer-down and commits on every pointer-up, with no guard on whether anything
actually moved. So every click writes to the store and reloads the Finding, which rebuilds the Marker
list and destroys the element mid-gesture. Annotations hit exactly this and fixed it by committing
only on an actual change; the same guard is needed here before a click can reliably mean "select".
Guard it first, then build the selection on top.

Selection is exclusive: picking up a Marker clears any selected annotation and vice versa, only one
thing is ever selected. Escape deselects, and so does clicking bare canvas. Clicking a Marker is what
hands the keyboard to the canvas, so Delete lands there and not in whatever note field had focus.

Undo is **out of scope here** and is ticket 02. Deletion through this path behaves exactly as the
right-click menu does today, which is: not undoable yet.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Clicking a Marker without moving it writes nothing to the store and triggers no reload
- [ ] Dragging a Marker still moves it and still commits on release, unchanged
- [ ] Clicking a Marker selects it and draws a selection ring on the reticle, using the theme's
      selection-ring token — no colour literal
- [ ] Selecting a Marker opens the Marker Notes tab and highlights that Marker's row
- [ ] Selecting a Marker clears any selected annotation; selecting an annotation clears any selected
      Marker; clicking bare canvas clears both
- [ ] Escape deselects a selected Marker
- [ ] Delete removes the selected Marker; survivors renumber and the Marker Notes list agrees
- [ ] Backspace does not remove a Marker
- [ ] Delete with no Marker and no annotation selected does nothing
- [ ] After a Marker is deleted, nothing is selected and a second Delete does nothing
- [ ] A reachability test asserts the selection and delete callbacks are both instantiated in the
      Slint tree and bound in Rust — not merely that the handlers behave. Follow the shape of the
      existing annotation-wiring test
