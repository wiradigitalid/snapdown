# Snapdown — Project Handover

## Status: W1–W5 delivered · W6 open

All five waves W1–W5 were delivered, tested, and merged. Every capability the PRDs promised exists
and works.

**That was not the same as being done, and 2026-08-23 is where that became clear.** The owner's first
sustained use of the shipped product produced a list of **experience** defects rather than missing
features: they could not tell which application they had opened, could not find the Editor, could not
read the labels on Findings and Bundles against their own background, and were asked to set two
numbers the PRD itself admits have never been measured.

The root cause was a single absence — **no `wdi-ux` output had ever been written for this product.**
No document anywhere said what a screen owes.

G1 through G4 were re-run at greater depth on 2026-08-23, the UX gate ran for the first time, and
**wave W6 is open**: ten stories across four epics, targeting release `r3`. Two of those ten are
defects found by reading the code against the newly-deepened documents — see below.

| Wave | Status |
|---|---|
| W1–W5 | Closed. r1 and r2 |
| **W6** | **Open.** The desktop experience rework. r3 |

---

## Wave Summary

| Wave | Title | Component | Status | Release |
|------|-------|-----------|--------|---------|
| **W1** | Settings complete — workspace, stores, CI | `settings` | ✅ Closed | r1 |
| **W2** | Finding complete — capture, editor, markers, deletion | `finding` | ✅ Closed | r1 |
| **W3** | Bundle complete — compose, list, copy, delete | `bundle` | ✅ Closed | r1 |
| **W4** | Agent access complete — key, Local API, MCP bridge | `agent-access` | ✅ Closed | r2 |
| **W5** | Sharing complete — publish, web service, reader | `sharing` | ✅ Closed | r2 |

---

## Key Deliverables

### W1 — Settings (r1)
- Cargo workspace, Tauri v2 shell, React webview, system tray
- SQLite migrations v1–v2 (`setting`, `schema_version`, `finding`, `note`, `marker`)
- `VaultBlobStore` with path traversal guards
- Settings UI: Vault folder picker, Quality Budget, Hotkeys, Windows startup

### W2 — Finding (r1)
- Multi-monitor capture overlay with DPI-aware region selection
- Image reduction pipeline under Quality Budget
- Findings Editor (list + detail with inline Note editing)
- Marker canvas (click-to-place, drag, badge annotations, contiguous renumbering)
- Synchronous file deletion on finding delete + `OrphanSweeper` tool

### W3 — Bundle (r1)
- SQLite migration v3 (`bundle`, `bundle_item`) + `SqliteBundleStore`
- Pure Markdown serializer with relative paths (AD-9)
- Marker burner pipeline (burned badges on exported screenshots)
- BundleComposer UI + Tauri commands + `BundleView`
- Clipboard export + golden-file markdown tests
- Atomic bundle deletion with vault file sync

### W4 — Agent Access (r2)
- SQLite migration v4 (`access_key`) + `AccessKey` domain model + `SqliteAccessKeyStore` (constant-time verification, auto-revocation)
- Local API server (`127.0.0.1`) with routes: `/v1/health`, `/v1/bundles`, `/v1/bundles/:id`, `/v1/bundles/:id/images/:filename`
- Standard JSON error envelopes (`cross-cutting.md`); refusal vs empty result distinction (RISK-6)
- `snapdown-bridge` CLI (stdio MCP): `set_access_key`, `list_bundles`, `read_bundle`, `read_bundle_image`
- Settings Agent Access screen: Generate/Re-copy/Revoke key

### W5 — Sharing (r2)
- SQLite migration v5 (`publication`) + `Publication` model with 160-bit CSPRNG slug (AD-8)
- Go web service (`apps/web-service`): staged all-or-nothing publish, unpublish, reconcile, public read routes with NFR-15 identical 404
- Desktop publish client (`LC-020`) with sticky `last_error` + unpublish cascade on bundle delete (BR-23)
- ~~Web reader SPA (`PublishedBundleReader.tsx` / Screen 14; `PublicationNotFound.tsx` / Screen 15)~~
- ~~Publish dialog (Screen 11) with BR-86 confirmation + publication badges + copy URL~~

  **Corrected 2026-08-23 — these three were never written.** No such files exist anywhere in the
  repository. `GET /b/{slug}` returns the stored Markdown inside a bare `<pre>` with no stylesheet
  and no rendered images. The machine-facing path works; the human reader does not exist.
  Registered as `BUG-2`. It went unnoticed for two waves because all three inventories sat at
  `derived_from: plan` and the screen reader read exactly one component.

---

## Repo State

```
D:\Developer\wiradigital.id\snapdown (main)
├── crates/snapdown-core     # Pure domain, ports, no I/O
├── crates/snapdown-store    # SQLite + Vault adapters
├── crates/snapdown-bridge   # MCP stdio bridge binary
├── apps/desktop             # Tauri v2 desktop app
├── apps/web-service         # Go web-api (web-api container)
├── web/ui                   # @snapdown/ui shared React library
├── .control/                # Registry, reports, decisions
│   ├── registry/waves.yaml  # All waves closed
│   └── reports/RTR-W1..W5.md
├── _bmad-output/specs/      # SPECs, stories, review records
└── AGENTS.md / CLAUDE.md    # Agent rules
```

---

## Commands for Next Agent

```bash
# Verify full workspace
cd D:\Developer\wiradigital.id\snapdown
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
npm --prefix web/ui run typecheck && npm --prefix web/ui run lint && npm --prefix web/ui run test
npm --prefix apps/desktop run typecheck && npm --prefix apps/desktop run lint && npm --prefix apps/desktop run test
cd apps/web-service && go test ./...
uv run .constitution/method/scripts/validate.py --check
```

---

## Open Questions (Carried Forward)

From `.control/questions/`:
- **OQ-13**: Host for `web-api` (go-live)
- **OQ-14**: Domain for Publication URLs (go-live)
- **OQ-1, OQ-7, OQ-8**: Agent image fetch + unlisted slug access control assumptions

These are go-live items; product is feature-complete.

---

## Next Session

No further work required unless:
1. Go-live configuration (host, domain, TLS)
2. New feature requests
3. Bug reports

The product satisfies all FR/NFR/AD/UC/CR from both PRDs (`capture-to-markdown`, `agent-handoff`).
---

## 2026-08-23 — what changed, and what it cost

### Three decisions

| | |
|---|---|
| **DEC-003** | Snapdown is **one process wearing two window personas**, not two executables. The tray is **Snapdown**; the workspace window titles itself **Snapdown Editor**. Snagit splits into two processes; that is a consequence of its C++/WPF lineage, not a property of the problem, and Tauri v2 removes every constraint that forced it. |
| **DEC-004** | The Quality Budget is chosen as a **named intent** — `Auto` (default), `Sharp`, `Balanced`, `Small` — with the raw numbers behind **Advanced** and a fifth state, `Custom`. `Auto` derives the long edge and encoder quality **per capture**: a 312×118 tooltip and a 3840×2160 screen cannot be served by one constant. |
| **DEC-005** | The desktop experience is finished **before** `sharing` and `agent-access` are touched again. Neither is cancelled; both keep their shipped code, and their surfaces stay reachable. |

### Depth raised

`finding` guarded → deep · `bundle` outline → deep · `settings` **catalog → deep**.

The `settings` change is the finding of the pass. At `mode: catalog` **G4 is skipped by design**, so
that component had no flow, no state machine, no failure behaviour, and no screen specification —
for five waves. Every one of the owner's Settings complaints was a question the corpus had no slot to
answer, because the gate that would have answered it was configured off.

### The two that matter most: the product's defining interactions have no working UI

**`BUG-4` — the capture path does not work.** `capture.rs:106` opens the overlay window at
`index.html?overlay=true`. Nothing in the frontend reads `window.location.search`, there is one html
entry point, and `vite.config.ts` declares no second input. The overlay window mounts the Editor
shell — opening on Settings — instead of `CaptureOverlay`. No dim, no crosshair, no region drag, no
note field, no Finding. `FR-1`, `FR-2`, `UC-1`, `UC-2` unmet.

**`BUG-5` — the Editor never renders a Finding's image.** `MarkerLayer` is exported from
`web/ui/src/index.ts` and mounted nowhere. `FindingsEditor.tsx` shows metadata, a Note field, and
`{f.markers.length} markers` as *text*. Markers cannot be placed. `AD-1` — Markers and Note lines are
one sequence, the invariant this product is built on — has **no user interface**. `BG-1` says a note
is unambiguously attached to the image it describes; the attachment lives in the database and is
invisible.

**`BUG-6`** — the orphan report is built, tested, and unreachable. No route, tab, or mount point.

#### The finding underneath all three

A sweep for `<ComponentName` across the tree found **four components built, unit-tested, and mounted
nowhere**: `CaptureOverlay`, `MarkerLayer`, `OrphanReportView`, `EmptyState`. Every one correct in
isolation. None assembled into anything a Reviewer can reach.

**This repository has no composition test of any kind.** Every suite proves a part; nothing asks
whether the parts were wired together. Five waves closed green on that basis, and three requirements
are unmet in a build whose tests all pass. `V12` cannot catch it — it checks that an `LC` is
*registered*, not that it is *reached*. Filed as `OQ-23`; the grep that catches it is now the first
pitfall in `AGENTS.md` § Code.

`W6-S2` carries `BUG-4`. `W6-S7` carries `BUG-5` and `BUG-6`.

### Two more, found by reading code against the deepened documents

**`BUG-1` — deleting a Finding silently guts every Bundle that holds it.**
`bundle_item.finding_id` carries `ON DELETE CASCADE`, and `foreign_keys` is `ON`. `FR-13`'s third
consequence has said the opposite since G2: *a Finding that belongs to a Bundle can still be deleted;
the Bundle keeps its own copy and stays readable.* The Bundle's stored Markdown survives, so the
document still reads correctly and still copies the same bytes — only the **item list** loses a row.
The delivered document and the record of it disagree, and nothing reports it.

It went undetected for five waves because a test that a cascade does **not** fire is one nobody writes
unless a document says the cascade must not exist. That document did not exist until G4 ran at `deep`.

**The Vault move reports success while leaving an unreported duplicate.**
`vault_migration.rs` swallows both `fs::remove_file` results. The move itself is *stronger* than the
documents assumed — copy everything, verify everything, delete sources last, so no file ever exists in
neither place — but a source file that will not delete leaves a second copy of an image that may hold
personal data, and the Reviewer is told the move succeeded.

### One repository finding

The brief's constraint — *this repository is public; forbids committing a captured screenshot* — has
been `active` since G1 and **nothing enforces it**. A UX audit staged five screenshots of the owner's
machine for commit and was caught by hand. `.gitignore` now covers them; CI still has no guard.

### The false alarm worth remembering

The owner reported the application was called "Desktop", that they could only see Settings, that the
Vault folder had no Browse button, and that hotkeys could not be set manually.

All four traced to **one stale binary**: `target/release/desktop.exe`, older than the commit that
renamed the product and added tab navigation, the Browse button, and the hotkey recorder. They had
been running it. `FR-27` now makes a second desktop executable in the output directory a **build
failure**, not clutter.

The white-on-white was real, and on a different screen than reported: `FindingsView` and `BundleView`
paint light-theme panels unconditionally inside a shell whose tokens follow `prefers-color-scheme`.
23 hex literals live outside the token file. `AD-10` now makes a colour literal in a component a
defect, enforced by a lint rule.
