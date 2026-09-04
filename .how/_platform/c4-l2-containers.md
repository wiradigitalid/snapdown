---
type: c4
level: 2
created: "2026-08-22"
updated: "2026-09-04"
---

# C4 L2 — Containers

This file owns the container list. `components.yaml` → `containers:` is its registry form, and
`product_components[].containers` is the SSOT for the matrix at the bottom; V25 fails when the two
disagree.

## Diagram

```mermaid
graph TB
    R(["Reviewer"])
    RA(["Remote coding agent"])

    subgraph MACHINE["The Reviewer's Windows machine"]
        DA["<b>desktop-app</b><br/>Rust, native Slint UI<br/>(DEC-007)<br/><i>capture, Editor, Library,<br/>publish client</i>"]
        VAULT[("Vault folder<br/>+ library.db<br/><i>files on disk</i>")]
    end

    subgraph HOST["Host the Reviewer controls"]
        WA["<b>web-api</b><br/>Go — net/http + chi,<br/>embedded SQLite, blob dir<br/><i>serves Publications</i>"]
        WSTORE[("Publication store<br/>SQLite + blob dir")]
    end

    R -->|"hotkey, window, tray"| DA
    DA <-->|"read and write"| VAULT
    DA -->|"HTTPS, publish and unpublish,<br/>with a publish credential"| WA
    WA <-->|"read and write"| WSTORE
    RA -->|"HTTPS GET, raw Markdown"| WA
```

A **Local coding agent** node and a **`mcp-bridge`** container stood here until 2026-09-04, joined by
an MCP-over-stdio edge into the bridge and an HTTP-with-Access-Key edge from the bridge into
`desktop-app`'s Local API. `DEC-016` withdrew all of it: there is no running channel left for a local
agent to reach. (The diagram also carried two dangling edges into an undrawn `WU` node, left over from
`web-ui`'s removal under `DEC-015` — those are removed in the same pass, since nothing here still
argues for keeping them.)

## Elements

| Element | What it is | Notes |
| --- | --- | --- |
| `desktop-app` | A single Rust process (`snapdown-desktop`) owning the domain core, the SQLite and Vault adapters, the capture adapter, and the publish client, with a native Slint UI (`apps/desktop/ui`) for the Editor and Settings — `DEC-007` moved this off the original Tauri v2 + React + Vite + TypeScript webview, archived for reference at `archive/desktop-tauri` | `built: true`. The only writer in the whole system. Holds four of the four remaining Product Components, so it is the one container with an L3. **One process, two personas** — the tray (**Snapdown**) and the workspace window (**Snapdown Editor**) — and exactly one executable, `Snapdown.exe`, per AD-11 and `DEC-003` |
| `web-api` | Go service, `net/http` with `chi`, embedded SQLite and a blob directory, deployed as one binary with one configuration file | `built: true`. Serves Publications and nothing else. Never reads the Library |
| Vault folder + `library.db` | The Reviewer's chosen folder of image and Markdown files, plus the SQLite file holding metadata | **Not a container.** Embedded SQLite and a filesystem are storage inside `desktop-app`'s own process, with no runtime anyone deploys. Their invariants are AD-2 and AD-4 |
| Publication store | `web-api`'s SQLite file and blob directory | **Not a container**, for the same reason. NFR-14 keeps the whole of it inside one directory |

Windows 11 is the platform, not a container, and appears at L1.

## Relationships

| From | To | Purpose | Over |
| --- | --- | --- | --- |
| Reviewer | `desktop-app` | Every write in the system: capture, edit, mark, compose, delete, publish, configure | Global hotkey, desktop window, system tray |
| `desktop-app` | Vault + `library.db` | Read and write Findings, Notes, Markers, Bundles, Publications, Settings, and their files | Filesystem and embedded SQLite, in the same process |
| `desktop-app` | `web-api` | Publish a confirmed Bundle, and remove it on unpublish | HTTPS with a publish credential |
| Remote coding agent | `web-api` | Fetch a published Bundle as raw Markdown, and the images it references | HTTPS GET |
| `web-api` | Publication store | Read and write published Markdown, images, and slug records | Filesystem and embedded SQLite, in the same process |

## Product Components per container

Product Components per container — see `.control/registry/components.yaml`, each component's
`containers:`.

`desktop-app` holds more than one Product Component, so `c4-l3-desktop-app.md` exists. The other two
hold one each.

**A fourth container, `web-ui`, was drawn on this diagram, listed in the element table, given an HTTPS
arrow to `web-api`, and mapped to `sharing` — until 2026-09-01, when `DEC-015` withdrew it.** It never
existed: no such application was written, and `web-api` serves no static assets to deliver one with.
Its element row argued explicitly that it *is* a container rather than static assets, because it runs
in the reader's browser as its own process. **That argument was sound and would be correct again the
day such a client exists** — it is recorded here because deleting the row deletes the reasoning with
it. The published Bundle page is rendered by `web-api` itself, in Go, and `LC-027 bundle-reader` moved
there rather than being retired.

## What is deliberately not shown

- **Deployment topology for `web-api`** — the host, the reverse proxy, the certificate, the process
  supervisor. It belongs to the devops repository and is referenced from here, never drawn here.
- **The build pipeline.** How the Slint-bundled desktop binary and the two web front ends are
  produced is a `structure-codebase.md` and CI concern.
- **The Windows credential store.** It holds the publish credential and is named in the spine's
  Consistency Conventions; drawing it as a box would suggest it is a container.
- **Inside any container.** Product Components inside `desktop-app` are L3.

**Amended 2026-08-23.** `DEC-003` was weighed and rejected the alternative that would have changed
this diagram: a second executable for the Editor, Snagit's shape. It would have made `desktop-app`
two containers with a second writer to one SQLite file and one Vault directory — which AD-11 now
forbids. The container count is unchanged and that is the outcome of a decision, not the absence of
one. What did change is that this container now has two named personas, and the L3 carries the
`editor-shell` that draws one of them.
