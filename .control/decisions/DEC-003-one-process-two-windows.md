---
type: decision
id: DEC-003
status: applied
touches:
  - .control/product-glossary.md
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
  - .how/_platform/c4-l3-desktop-app.md
  - .how/_platform/inventory-screen.md
  - .how/settings/04-components/LC-028-editor-shell.md
  - .what/_prd/capture-to-markdown/prd.md
  - .what/_product-brief/brief.md
  - .what/business-rules.md
  - .what/settings/04-usecases/EXPERIENCE.md
supersedes: null
superseded_by: null
created: "2026-08-23"
---

# DEC-003 — Snapdown is one process wearing two window personas, not two executables

## Decision

Snapdown ships as a single executable, `Snapdown.exe`, that owns the tray icon, the global hotkeys,
the capture overlay, and one workspace window titled **Snapdown Editor**. The daemon and the editor
are two personas of one process, and there is no second executable for the editor.

## Why

The question this answers is the one the owner asked out loud: *why does the window say something
other than Snapdown, and how many Snapdown applications are there?* The honest answer had to be one
of two shapes, and only one of them survives what this product already is.

Snagit is the obvious model, and it splits: a capture daemon in the tray and a separate editor
binary. That split is a consequence of Snagit's age and its C++/WPF lineage, not a property of the
problem. Tauri v2 already gives one process a hidden main window, a tray, and a transparent
always-on-top overlay per monitor. Every reason Snagit had for a second process is a reason this
product does not have.

Against that, the second executable would buy nothing and cost four things at once. The editor
would cold-start on every open, which is the single most visible latency in the product, because
opening the editor is what the Reviewer does immediately after a capture. The Library — one SQLite
file and one Vault directory — would be reachable from two processes, so `finding-store` and
`bundle-store` would need a lock discipline they do not have today and that no test currently
covers. AD-1's shape, one Rust core with no I/O dependency, would have to be duplicated or split.
And the tray, which is the only always-resident surface, would have to marshal capture results to a
process that may not be running.

The persona split the owner actually wants is a naming problem, and it is solved where naming
problems belong: the tray identifies the product as **Snapdown**, and the workspace window titles
itself **Snapdown Editor**. The Reviewer sees the two things they expected to see. The machine runs
one of them.

The `mcp-bridge` executable stays a separate binary, and that is not an inconsistency. DEC-002
decided it for a reason that still holds — an MCP client launches it, not the Reviewer — and it
holds no window at all.

## Cost

- **The editor window is never truly closed.** Closing it hides it, because destroying it would
  throw away the warm start this decision is buying. That means webview memory stays resident for
  the whole session, and any state the editor holds must survive a hide/show cycle rather than a
  fresh mount. A leak in the editor is now a leak in the daemon.
- **One crash takes both down.** A panic in the editor's Tauri commands kills the tray, the hotkeys,
  and the overlay with it. With two processes the daemon would have survived. This raises the bar on
  every `unwrap` in the command layer, and it makes the hotkey re-registration path load-bearing.
- **Naming now lives in two places that must agree.** `productName` in `tauri.conf.json` and the
  window title set at runtime. They can drift, and the drift is exactly the confusion this decision
  exists to remove — so it needs a test, not a convention.
- **The stale-binary trap is real and already bit.** Renaming the product left `desktop.exe` beside
  `Snapdown.exe` in `target/release/`, and the owner ran the old one. One process means one name,
  and the build has to enforce that rather than leave both.

## Alternatives

Required here: `finding` sits at `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| Two executables, Snagit's shape | Buys process isolation the product cannot spend, and pays cold-start on the most frequent action, a two-writer Library, and a marshalling hop for every capture |
| One executable, one window, no persona naming | What exists today, and it is what produced the owner's question. The Reviewer has no word for the thing they opened |
| One executable, editor window destroyed on close | Saves resident memory and gives back the cold start this decision is buying. Reconsider only if webview residency is measured as a real cost |
| Rename the product to match the window, "Snapdown Desktop" | Solves the confusion by accepting it. The product is called Snapdown; the window is a persona of it |
| Two windows, both persona-named, no tray | The tray is what makes the hotkey plausible when no window is open. Removing it breaks the capture loop's whole premise |

## Reversal trigger

- The editor webview is measured holding enough resident memory that the tray becomes a nuisance on
  the Reviewer's machine. That reopens destroy-on-close, not the process count.
- A crash in editor code is observed taking the hotkeys down in normal use, more than once after the
  command layer is hardened. That reopens the process count itself.
- A second front end appears that genuinely must outlive or precede the desktop app — not the
  `mcp-bridge`, which is already separate for a different reason.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Source material | Owner's question of 2026-08-23; `.how/_platform/c4-l2-containers.md` § `desktop-app`; DEC-002 for why `mcp-bridge` stays separate |
