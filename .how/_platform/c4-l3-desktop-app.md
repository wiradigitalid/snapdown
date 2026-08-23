---
type: c4
level: 3
container: desktop-app
created: "2026-08-22"
updated: "2026-08-22"
---

# C4 L3 — desktop-app

The boxes are Product Components. All five live here, which is why this container has an L3 and the
other three do not.

## Diagram

```mermaid
graph TB
    R(["Reviewer"])
    MB(["mcp-bridge<br/><i>another container</i>"])
    WA(["web-api<br/><i>another container</i>"])
    OS(["Windows 11"])

    subgraph DA["desktop-app"]
        FIND["<b>finding</b><br/>Capture, reduce, hold,<br/>mark, edit, delete a Finding"]
        BUN["<b>bundle</b><br/>Compose, list, copy,<br/>delete a Bundle"]
        SET["<b>settings</b><br/>Vault, hotkeys, startup,<br/>Quality Budget, service address"]
        AA["<b>agent-access</b><br/>Issue and revoke the Access Key,<br/>serve the Local API"]
        SH["<b>sharing</b><br/>Publish and unpublish,<br/>hold Publication state"]
    end

    R --> FIND
    R --> BUN
    R --> SET
    R --> AA
    R --> SH

    BUN -->|"reads Findings, Notes, Markers"| FIND
    FIND -->|"reads the Quality Budget<br/>and the Vault location"| SET
    BUN -->|"reads the Vault location"| SET
    AA -->|"reads Bundles, read-only"| BUN
    SH -->|"reads Bundles, read-only"| BUN
    SH -->|"reads the service address"| SET
    FIND -->|"screen pixels, hotkey registration"| OS
    SET -->|"startup registration, credential store"| OS
    MB -->|"Local API, Access Key per request"| AA
    SH -->|"HTTPS publish and unpublish"| WA
```

Dependency direction is acyclic and one-way: `agent-access` and `sharing` read `bundle`, `bundle`
reads `finding`, and everything reads `settings`. Nothing reads back up. `finding` does not know
Bundles exist, and `bundle` does not know publishing exists — which is what lets CAP-8 be dropped
without touching either.

## Elements

| Element | What it is | Notes |
| --- | --- | --- |
| `finding` | Everything about one observation: the hotkey and overlay, region selection, the note field at capture time, image reduction, the Finding list in the Editor, Note editing, Markers, deletion, orphan reporting | Owns `Finding`, `Note`, `Marker`. `mode: deep` (raised from `guarded` 2026-08-23), `risk_accepted: low`. Carries AD-1, AD-2, AD-3, AD-4 |
| `bundle` | Composition of selected Findings into one stored Markdown document with its own images, the Bundle list, copying the Markdown, and hard deletion | Owns `Bundle`, `BundleItem`. `mode: deep` (raised from inherited `outline` 2026-08-23), `risk_accepted: low`. Carries AD-9 and, with `finding`, AD-2 |
| `settings` | The persisted choices — Vault location, hotkey bindings, Quality Budget, run at Windows startup, whether the Editor opens after a Capture, where the web service is — **and the Editor's window frame**: the two personas of AD-11, the primary navigation, and the surface fit `FR-29` requires | Owns `Setting`. `mode: deep` (raised from `catalog` 2026-08-23, where it had been skipping G4 by design), `risk_accepted: medium`. Carries `CAP-9` and, with every drawing component, AD-10 |
| `agent-access` | Issuing, showing, copying, and revoking the one Access Key, and serving the loopback Local API that the bridge calls | Owns `AccessKey`. `mode: guarded`, `risk_accepted: low`. Carries AD-5 and AD-7 |
| `sharing` | The publish client and the Publication record: publishing a named Bundle, unpublishing it, and showing where it is | Owns `Publication`. `mode: guarded`, `risk_accepted: low`. Carries AD-6, AD-8, and with `web-api` AD-5 and AD-7 |

## Relationships

| From | To | Purpose | Over |
| --- | --- | --- | --- |
| Reviewer | `finding` | Capture, note, mark, edit, delete | Global hotkey, Capture Overlay, Editor window |
| Reviewer | `bundle` | Select, compose, name, list, copy, delete | Editor window |
| Reviewer | `settings` | Set the Vault, the hotkeys, startup, the Quality Budget, the service address | Settings surface |
| Reviewer | `settings` | Tell which persona is on screen, and reach any surface from any other | The Editor shell — `UC-24`, `UC-25`, `LC-028` |
| Reviewer | `agent-access` | Issue, copy, and revoke the Access Key | Editor window |
| Reviewer | `sharing` | Publish, unpublish, copy a Publication URL | Editor window |
| `bundle` | `finding` | Read the Findings, Notes, and Markers a composition needs | In-process call into the domain core |
| `finding` | `settings` | Read the Quality Budget and the Vault location | In-process call |
| `bundle` | `settings` | Read the Vault location | In-process call |
| `agent-access` | `bundle` | Read the Bundle list, one Bundle's Markdown, and its images. Read-only, per AD-5 | In-process call |
| `sharing` | `bundle` | Read the Bundle being published. Read-only | In-process call |
| `sharing` | `settings` | Read the web service address and fetch the publish credential | In-process call, then the OS credential store |
| `finding` | Windows 11 | Grab screen pixels; register and unregister global hotkeys | OS APIs |
| `settings` | Windows 11 | Register run-at-sign-in; store and read the Access Key and publish credential | OS APIs, credential store |
| `mcp-bridge` | `agent-access` | The three reads the Local API exposes | HTTP on `127.0.0.1`, Access Key per request |
| `sharing` | `web-api` | Upload a confirmed publish; delete it on unpublish | HTTPS with a publish credential |

## What is deliberately not shown

- **The hexagonal split inside each box.** Which part of `finding` is core and which is adapter is
  the spine's Design Paradigm and the component's SDD; drawing it here would duplicate both.
- **The React webview as a separate box.** It is the presentation surface of all five components at
  once, not a component. Its screens are in `inventory-screen.md`.
- **SQLite and the Vault folder.** Storage inside this container's own process, per L2, and their
  contents are `inventory-db.md`.
- **The Tauri command layer.** It is the adapter that carries every arrow labelled "in-process call"
  from the webview into the core, and naming it as a box would make it look like a component.
- **The two personas as two boxes.** **Snapdown** (tray, hotkeys, overlay) and **Snapdown Editor**
  (the workspace window) are personas of one process, per AD-11 and `DEC-003`. Drawing them as two
  boxes at L3 would say what L2 explicitly decided against: they are not containers, they do not
  deploy separately, and there is exactly one `Snapdown.exe`. The frame that draws one of them is
  `LC-028` `editor-shell`, inside `settings`.

**Amended 2026-08-23.** Three `mode` values changed in the Elements table above, and `settings` gained
the window frame. The frame was previously drawn by nothing: it is inline JSX at the top of `App.tsx`,
owned by no component, which is how `FR-27` and `FR-28` came to be unmet without any document being
wrong.
