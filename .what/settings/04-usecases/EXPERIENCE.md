---
type: ux
component: settings
document: experience
created: "2026-08-23"
updated: "2026-08-23"
---

# EXPERIENCE — Settings

## Information architecture

Snapdown has two window personas and one of them holds everything below (`DEC-003`, `FR-27`).

| Persona | What it is | How it is reached |
|---|---|---|
| **Snapdown** | The tray icon. Owns the hotkeys and the capture overlay. No window | Runs from sign-in (`FR-18`) |
| **Snapdown Editor** | One window, four primary surfaces | Tray menu · the Editor hotkey · clicking a capture toast |

The Editor's four primary surfaces are **Findings**, **Bundles**, **Settings**, and **Agent access**.
`FR-28` requires each to be reachable from each, so navigation to all four is present on all four —
there is no surface the Reviewer can arrive at and be stuck on.

Settings itself holds four groups and **is not a second level of navigation**. All four are on one
surface, and all four are visible at the window's minimum supported size without scrolling (`FR-29`):

1. **Startup** — whether Snapdown runs at sign-in (`FR-18`)
2. **Vault folder** — where files go (`FR-16`)
3. **Quality Budget** — how much image quality is bought (`FR-5`)
4. **Hotkeys** — which keys do what (`FR-17`)

**Agent access was not a fifth group.** It stood as a primary surface of its own, listed in the rail
beside Findings, Bundles and Settings, with its own row in `inventory-screen.md` (row 13, `status:
removed` since 2026-09-04). An earlier draft of this document had it both ways — a primary surface
*and* a group inside Settings — which would have put one thing in two places in one product. `DEC-005`
froze it before `DEC-016` withdrew it outright; there is no longer a surface for `FR-28` and `BR-120`
to keep listed.

## Voice and tone

Plain, second person, and never apologetic. A setting explains what it does to the Reviewer's work, not
what it does to the program.

- "Starts Snapdown in the tray when you sign in to Windows." — not "Enables autostart registration."
- "Ctrl+Shift+S is already used by another program." — not "Hotkey registration failed."
- "Sharp keeps small text crisp. Files are larger." — not "quality=92, maxLongEdge=2400."

A number appears in Settings only where the Reviewer can judge it: a file size they can compare, a
pixel dimension of something they just captured. A number they can only accept does not appear
(`FR-5`).

## Component patterns

| Pattern | Behaviour |
|---|---|
| **Group** | A titled block. Takes the height its content needs and no more. Never stretched to match a neighbour |
| **Segmented control** | Quality Budget's four named states. Selecting one applies immediately; there is no Save |
| **Disclosure** | "Advanced" under Quality Budget. Collapsed by default, and its state persists per Reviewer |
| **Hotkey chip** | Click or Enter to listen, then the next combination is captured. Esc cancels and restores what was there |
| **Toggle with three states** | On, off, and *not yet known* — see State patterns |
| **Path field with Browse** | Read-only text plus a native folder picker. The path is never typed |

## State patterns

**The startup toggle has three states, and the third is the point.** `FR-18` requires the control to
reflect the real Windows registration and never a remembered intention. Reading that registration is
asynchronous, so between opening Settings and the answer arriving the control is **indeterminate** — it
shows that it does not yet know, and it is not interactive. It MUST NOT render an assumed value first.
The shipped build assumes `true`, then repaints to `false`, and the Reviewer watches the product change
its mind about its own state.

| Surface state | What the Reviewer sees |
|---|---|
| **Loading** | Every group visible with its title; each control indeterminate and inert. The layout does not move when values arrive |
| **Populated** | Normal |
| **Partial failure** | One group failed to load; that group alone shows its error and a Retry. The other three work |
| **Error** | The settings store could not be opened. One message naming the file and one action: open the folder. Snapdown does not silently recreate the store |

There is no **empty** state for Settings — every setting always has a value, shipped or chosen. Saying
so is the answer to check 4, not an omission.

## Interaction primitives

- **Immediate apply.** Startup, Quality Budget, and hotkeys apply when changed. There is no Save.
- **One exception, and it is deliberate.** Changing the Vault folder has an **Apply**, because
  `FR-16` may move every existing file and that MUST NOT happen on a stray click.
- **Confirmation only where something moves.** Changing the Vault folder asks once whether to move
  existing files, and moves all of them or none.
- **Failure is reported where it happened.** A hotkey that will not register says so under that
  hotkey, naming the conflict (`FR-17`). It does not raise a toast and leave the row looking fine.
- **Esc.** Cancels a listening hotkey chip. It does not close the window.

## Accessibility floor

Meets **WCAG 2.2 AA**, and this is a requirement (`NFR-16`), not an intention.

- Every text element meets AA contrast against its own background, in the Windows light theme and the
  Windows dark theme. Checked by an automated assertion over both themes, not by inspection.
- Every control is reachable and operable from the keyboard alone, in visual order.
- `focus-visible` is never suppressed. The hotkey chip is the one control that swallows key events,
  and it does so **only while listening**; Tab still leaves it, and Esc always stops it.
- State is never carried by colour alone: the active/disabled hotkey badge carries a word, and the
  selected segment carries a checked state, not just a fill.
- Target size at least 24×24 px.
- No motion beyond a 150 ms state transition, and all of it honours `prefers-reduced-motion`.

## Key flows

**Wira signs in on Monday morning.** He has not opened Snapdown since Friday. It is already in the tray,
because `FR-18` put it there without him asking — a capture tool he has to remember to launch is a
capture tool that is not there when he notices something. He presses the capture hotkey and it works.
He never opens Settings. **This is the flow Settings is designed to make unnecessary**, and its climax
beat is that nothing happens.

**Wira changes where files go, mid-project.** He opens the Editor from the tray, clicks Settings, and
sees all four groups at once. Vault folder is second. He clicks Browse, picks the new project folder in
the native picker, and clicks Apply. Snapdown asks once whether to move the 31 existing files. He says
yes. They all move, or none do. The path field now shows the new folder, and Open in Explorer proves it.

**Wira's capture hotkey stops working after he installs another tool.** He opens Settings. The Capture
Region row already shows a warning badge and a line naming the program that took the combination — he
did not have to discover the problem, because `FR-17` requires a failed registration to be reported
rather than swallowed. He clicks the chip, presses Ctrl+Alt+4, and it binds. The old combination stops
working, the new one works, and nothing restarted.

## Edge cases

| Moment | What the Reviewer does next |
|---|---|
| The chosen Vault folder is not writable | Refused at the point of choosing, naming the folder. The old path is still in effect and still shown |
| The Vault move fails halfway | Nothing moved. One message says so plainly. The Reviewer's files are where they were |
| A hotkey is held by a program that is not running right now | It binds, and Snapdown reports at next startup if registration then fails. An honest failure later beats a wrong refusal now |
| Two Snapdown actions are given the same combination | Refused, naming the other action. This is Snapdown's own conflict and it is reported differently from an outside one |
| A hotkey is cleared | That action is disabled, and the row says **Disabled** rather than showing an empty box that looks broken |
| The Reviewer edits an Advanced value | The Quality Budget moves to **Custom** in the same interaction, visibly. They never leave Auto without seeing it |
| Windows theme changes while Settings is open | Every surface repaints correctly. No screen is correct in only one theme (`NFR-17`) |
| Startup registration is removed outside Snapdown | The toggle shows off, because it reads the real state. It does not re-enable itself — the default applies to a first run, never to a decision |
