---
type: decision
id: DEC-001
status: applied
touches:
  - .how/_platform/ARCHITECTURE-SPINE.md
  - .how/_platform/c4-l2-containers.md
supersedes: null
superseded_by: null
created: "2026-08-22"
---

# DEC-001 — Snapdown is built as a Tauri v2 desktop app in Rust with React front ends, and a Go web service

## Decision

The desktop application is Tauri v2 with a Rust core and a React + Vite + TypeScript webview; the MCP
bridge is a Rust binary in the same Cargo workspace; the web service is Go using `net/http` with
`chi`, serving a React + Vite single-page application; and both stores are embedded SQLite. Next.js and
Express are excluded and MUST NOT be introduced without a decision that supersedes this one.

## Why

The repository's initial commit carried a Rust `.gitignore` — Cargo, rustfmt, cargo-mutants, RustRover
— so Rust was the starting assumption before any of this was written. Rust plus Tauri v2 is also the
only combination that gives screen capture, global hotkey registration, tray presence, and sign-in
startup registration on Windows from one toolchain, at a binary size small enough that a tool the
Reviewer runs all day is not noticed.

React on both front ends and Go for the service are the repo owner's own calls, made during the G2 run
after the alternative had been drafted with Svelte and Axum. React was chosen so that one component
library serves both the desktop webview and the published-Bundle reader — which is what makes the
shared design system in `.how/_platform/design-system.md` possible rather than aspirational. Go was
chosen for the service, and its consequence is worth stating plainly: this product is two languages,
not one, and the seam is the publish request.

The exclusions are the owner's and they are the half of this record that would otherwise be lost.
Nothing in the code will ever explain why Next.js is absent from a project with a React front end, or
why a Go service uses `chi` rather than a framework — which is exactly the test for recording a
decision at all.

## Cost

- **Two languages, two toolchains, two test runners.** A change that crosses the publish seam is
  written twice and cannot be typechecked end to end. The `web-api` contract in
  `inventory-api.md` rows 12–14 is the only thing holding the two sides together, and nothing
  mechanically enforces it.
- **No server-side rendering for `web-ui`.** Excluding Next.js means the published-Bundle reader is a
  client-rendered page. An agent fetching raw Markdown is unaffected — which is why the cost is
  acceptable — but a human on a slow connection sees a blank frame first.
- **`chi` is a router, not a framework.** Middleware, configuration loading, and error mapping are
  written rather than adopted. That is a few hundred lines this project now owns.
- **Tauri's webview is the platform's, not bundled.** Rendering depends on the WebView2 runtime
  installed on the Reviewer's machine, which is a support surface a bundled engine would not have.
- **Rust for the capture path is unforgiving.** Per-monitor DPI handling and overlay window lifetime
  are the two hardest parts of this product, and they are in the language with the least room for a
  quick fix.

## Alternatives

Required here: `finding`, `agent-access`, and `sharing` all sit at `risk_accepted: low`.

| Option | Why not |
| --- | --- |
| Electron with a Node backend | Ships an engine and a runtime for a tool that idles all day; NFR-6's 150 MB working set is not reachable. Screen capture and global hotkeys would still need native code |
| .NET with WPF or WinUI | The strongest Windows-only alternative, and genuinely better at the OS surface. It gives up the Cargo workspace the MCP bridge shares with the core, and the owner's stack is not .NET |
| Svelte for the desktop webview | The original draft. Smaller and faster to write, and it means the desktop and `web-ui` cannot share a component library — which is the whole basis of the shared design system |
| Rust with Axum for the web service | One language for the entire product, one toolchain, one test runner. Rejected by the owner in favour of Go |
| Next.js for `web-ui` | Excluded by the owner. It would also add a Node runtime to a host that otherwise runs one static binary, against NFR-14 |
| Express for `web-api` | Excluded by the owner, and it contradicts the Go choice |
| Postgres or another database server | Contradicts NFR-14 — one executable, one config file, state in one directory. Nothing in this product needs concurrent writers |
| A hosted service instead of `web-api` | Puts images that may contain personal data in an account the Reviewer does not control, and makes unpublish a promise a third party keeps |

## Reversal trigger

Any of these makes revisiting correct:

- WebView2 turns out to be absent or broken on a machine the Reviewer needs Snapdown on, and no
  install path fixes it. That reopens the desktop framework, not the language.
- The publish seam produces two or more defects caused by the two sides disagreeing about the
  contract. That reopens the Go choice in favour of one language.
- Per-monitor DPI capture proves not to be solvable in the Rust ecosystem to NFR-1's 200 ms. That
  reopens .NET for the capture adapter specifically.
- `web-ui` needing to be readable by something that will not run JavaScript. That reopens
  server-side rendering, though the raw-Markdown route (endpoint 10) exists so this should not
  happen.

## Trace

| | |
| --- | --- |
| Meeting note | — |
| Open question | — |
| Note | `.constitution/project/codebase-stack-guide.md` is deliberately NOT in `touches`. It is born empty and its own header forbids filling it before code exists that ratifies it; the first wave's distillation promotes `bmad-spec`'s `stack.md` companion into it |
| Source material | `.control/memlog/prd-capture-to-markdown.md` and `.control/memlog/prd-agent-handoff.md`, both carrying the `change` entry where the owner set the stack mid-run; `.what/_prd/agent-handoff/addendum.md` § Options weighed |
