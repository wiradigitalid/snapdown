---
type: component
lc: LC-028
name: editor-shell
component: settings
container: desktop-app
created: "2026-08-23"
---

# LC-028 — editor-shell

The window frame of the **Snapdown Editor** persona. New at this gate; the thing it replaces is inline
JSX at the top of `apps/desktop/src/App.tsx`, owned by no component.

## Responsibility

Three things, and nothing else:

1. Declare which persona is on screen — the window title reads `Snapdown Editor` and does not change
   as the Reviewer moves between surfaces (`FR-27`, `BR-121`).
2. Present navigation to every primary surface, on every primary surface (`FR-28`, `BR-120`).
3. Give the Capture action a home that is reachable from anywhere.

## What it must not do

- **Read state.** It depends on nothing (`SDD-settings.md` § Structure). A frame that reads state is a
  frame that can fail to draw, and `FR-28` requires navigation to survive any surface's failure.
- **Own routing logic beyond which surface is active.** A surface's own loading and error states are
  the surface's.
- **Hide a surface whose component is frozen.** `BR-120` is explicit: `DEC-005` freezes `sharing` and
  `agent-access`, and their surfaces stay listed.

## Boundaries

| Direction | With | Contract |
|---|---|---|
| out | The active surface | Which surface is active. One value |
| out | `finding` | Capture requested |
| in | Nothing | It has no inbound dependency, deliberately |

## The delta from what runs today

| Today (`App.tsx`) | Required |
|---|---|
| A top tab row, ~64 px of every surface's height | A left rail, `200px`, so `FR-29` has the vertical room it needs |
| Active tab distinguished by fill colour alone | Fill **and** a left edge bar — `NFR-16` forbids state carried by colour alone |
| The Capture action has nowhere to live | Pinned to the rail's foot, separated by a rule |
| Window title `Snapdown`, matching the tray | `Snapdown Editor`, distinct from the tray, per `DEC-003` |
| Inline in `App.tsx`, owned by nothing, no test | Its own module with its own tests |

`[MISSING]` for every row above. All are planned work, not bugs: the requirements (`FR-27`–`FR-29`)
were written on 2026-08-23, after the code.
