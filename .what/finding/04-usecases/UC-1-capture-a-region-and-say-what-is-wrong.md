---
type: uc
id: UC-1
component: finding
satisfies: [FR-1, FR-2]
critical: false
created: "2026-08-22"
---

# UC-1 — I press a key, box the thing that is wrong, and say what is wrong with it

## Trigger

The Reviewer presses the Capture hotkey, from inside whatever application they are looking at.

## Precondition

Snapdown is running. The Capture hotkey is registered. A Vault location is in effect — the Reviewer's
choice, or the shipped default, so this holds on a fresh install (BR-28).

## Main Flow

1. The Reviewer presses the Capture hotkey.
2. Snapdown dims every connected monitor, renders full-screen crosshair guides and a pixel loupe magnifier (6x-8x) with live dimensions, and auto-detects windows/sub-panels with un-dimmed cutout highlighting.
3. The Reviewer either clicks once on a highlighted container, clicks the top-center Fullscreen button, or drags a custom rectangle; Snapdown shows pixel dimensions and aspect ratio tags (16:9, 4:3, 1:1, 21:9).
4. The Reviewer releases or clicks to commit the region.
5. Snapdown highlights the selected region, and shows a focused note field anchored to it.
6. The Reviewer types what is wrong with it (or re-selects another region by clicking/dragging elsewhere).
7. The Reviewer saves (Enter key or Save button), from the keyboard alone if they choose.
8. Snapdown stores the Finding, closes the overlay, returns focus to the window that had it, and shows
   a transient toast with the running count of Findings.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 3 | The Reviewer drags on a different monitor than the pointer started on | The crosshair follows. Any monitor may be dragged on (BR-38) |
| 6 | The Reviewer types nothing | The Finding saves with an empty Note. A Finding with no words is still a Finding (BR-4) |
| 6 | The Reviewer types several lines | Every line, blank ones included, is preserved verbatim (BR-34) |
| 8 | The Reviewer has turned on opening the Editor after a Capture | The Editor opens as well. The default is off (BR-37) |
| 8 | The Editor is already open | The new Finding appears at the top of the list without the Editor being reopened (BR-44) |

## Failure Flows

| From step | Failure | What the system does | What the user is left with |
| --- | --- | --- | --- |
| 2 | No overlay can be shown — a secure desktop, a locked session | Abandons the Capture before anything is captured | A toast saying the screen could not be read, with the reason Windows gave. Nothing in the Vault |
| 3 | The dragged region is smaller than 8 × 8 pixels | Refuses the selection and leaves the overlay open, so the Reviewer can drag again (BR-31) | Still in the overlay, nothing lost |
| 3 or 6 | The Reviewer presses Escape | Discards the Capture entirely (BR-32) | Back where they were. Nothing stored, no file, no row |
| 4 | The screen capture returns fewer pixels than the region asked for | Abandons the Capture rather than storing a wrong image | A toast naming the mismatch. Nothing in the Vault |
| 3 | A monitor is attached or removed mid-drag | Closes the overlay and abandons the Capture rather than redrawing mid-selection | Back where they were. Pressing the hotkey again is the whole recovery |
| 7 | The Vault is unreachable — unplugged drive, revoked permission | Abandons the Capture before committing anything | A toast naming the Vault path, with an action that opens Settings. No half-Finding in the list |
| 8 | The image write fails after the overlay has closed | Removes the row it had just committed, so no broken Finding is left | A toast saying the Finding could not be stored. The list is unchanged |
| 8 | `library.db` cannot be written | Refuses the Capture and disables further capture rather than losing Findings quietly | A blocking banner in the Editor and a tray badge |

## Outcome

One Finding exists, holding the captured region as a reduced image in the Vault and the Note that was
typed against it. No Snapdown window is open, focus is back where it was, and the Reviewer can press
the hotkey again immediately (UC-2).

## Business Rules

BR-4, BR-5, BR-8, BR-28, BR-31, BR-32, BR-33, BR-34, BR-35, BR-36, BR-37, BR-38, BR-39, BR-40, BR-41,
BR-42, BR-44.
