# Snapdown — Project Handover (All 5 Waves Complete)

## Status: ✅ COMPLETE
All five waves (W1–W5) of the Snapdown product have been delivered, tested, and merged.

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
- Web reader SPA (`PublishedBundleReader.tsx` / Screen 14; `PublicationNotFound.tsx` / Screen 15)
- Publish dialog (Screen 11) with BR-86 confirmation + publication badges + copy URL

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