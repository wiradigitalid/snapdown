"""inventory readers — how THIS product's code is read. Owned by the product, not the method.
"""

from __future__ import annotations

import re
from pathlib import Path


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def derive_db(root: Path) -> "Derived":       # noqa: F821
    """Every table this product stores, derived from SQLite migrations in code."""
    rows: list[Row] = []                      # noqa: F821
    unread: list[str] = []

    # 1. library.db migrations (crates/snapdown-store/src/sqlite/migrations.rs)
    mig_path = root / "crates" / "snapdown-store" / "src" / "sqlite" / "migrations.rs"
    if mig_path.exists():
        content = read(mig_path)
        # Parse CREATE TABLE statements
        matches = re.findall(r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z0-9_]+)\s*\((.*?)\);", content, re.S | re.I)
        for tbl_name, tbl_body in matches:
            tbl_name_clean = tbl_name.strip()
            if tbl_name_clean == "schema_version":
                rows.append(Row(              # noqa: F821
                    key="schema_version",
                    cells=["schema_version", "settings", "The migration level of library.db, so a newer binary knows what it is opening", "version pk · applied_at", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "setting":
                rows.append(Row(              # noqa: F821
                    key="setting",
                    cells=["setting", "settings", "One persisted preference per key: Vault location, each hotkey binding, the Quality Budget pair, startup, open-editor-after-capture, the web service address", "key pk · value · updated_at", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "finding":
                rows.append(Row(              # noqa: F821
                    key="finding",
                    cells=["finding", "finding", "One observation. The row exists only while its image file does (AD-2)", "id UUIDv7 pk · image_path relative to the Vault · image_width · image_height · captured_at · source_monitor · region", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "note":
                rows.append(Row(              # noqa: F821
                    key="note",
                    cells=["note", "finding", "The prose body of one Finding's Note. The numbered lines are not here — they belong to marker (AD-1)", "id pk · finding_id fk unique · body · updated_at", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "marker":
                rows.append(Row(              # noqa: F821
                    key="marker",
                    cells=["marker", "finding", "One numbered Marker and the Note line that is the same thing as it. ordinal is the badge number and the line number at once", "id pk · finding_id fk · ordinal unique per finding, from 1, no gaps · x · y normalised 0–1 (AD-3) · comment", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "bundle":
                rows.append(Row(              # noqa: F821
                    key="bundle",
                    cells=["bundle", "bundle", "One composed Bundle, including the composed Markdown itself so that every handoff path serves identical bytes (AD-9)", "id pk · name · markdown · markdown_path relative to the Vault · composed_at", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "bundle_item":
                rows.append(Row(              # noqa: F821
                    key="bundle_item",
                    cells=["bundle_item", "bundle", "The membership of one Finding in one Bundle, and the path of the Marker-burned image copy written for it", "id pk · bundle_id fk · finding_id fk · position · image_path · unique on (bundle_id, finding_id)", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "access_key":
                rows.append(Row(              # noqa: F821
                    key="access_key",
                    cells=["access_key", "agent-access", "The one Access Key that may be valid, stored as a hash. The key itself lives in the Windows credential store, never here", "id pk · key_hash · issued_at · revoked_at nullable", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            elif tbl_name_clean == "publication":
                rows.append(Row(              # noqa: F821
                    key="publication",
                    cells=["publication", "sharing", "Where a Bundle is published and whether it is still live. slug is generated independently of every id here (AD-8)", "id pk · bundle_id fk unique · slug unique · base_url · published_at · unpublished_at nullable · last_error nullable", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
            else:
                rows.append(Row(              # noqa: F821
                    key=tbl_name_clean,
                    cells=[tbl_name_clean, "settings", f"Table {tbl_name_clean}", "derived from migration", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
    else:
        unread.append("crates/snapdown-store/src/sqlite/migrations.rs not found")

    # 2. The web service keeps its OWN SQLite, in Go, and it was invisible to this reader until
    #    2026-08-23 — which is why published_bundle, published_blob and web_schema_version were
    #    reported as "planned but not read in code" for two waves. They were built; nobody looked.
    go_store = root / "apps" / "web-service" / "internal" / "store" / "store.go"
    if go_store.exists():
        content = read(go_store)
        go_tables = {
            "web_schema_version": ("sharing", "The migration level of the web service's own store, independent of library.db", "version pk · applied_at"),
            "published_bundle": ("sharing", "One published Bundle as the service holds it. deleted_at is what makes an unpublish a state change rather than a row removal", "slug pk · markdown · blob_dir · created_at · deleted_at nullable"),
            "published_blob": ("sharing", "One image belonging to a published Bundle", "id pk · slug fk cascade · filename · content_type · byte_size"),
        }
        for tbl_name, tbl_body in re.findall(
            r"CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([a-zA-Z0-9_]+)\s*\((.*?)\);",
            content, re.S | re.I):
            name = tbl_name.strip()
            owner, purpose, cols = go_tables.get(
                name, ("sharing", f"Table {name}", "derived from the Go schema"))
            rows.append(Row(                  # noqa: F821
                key=name,
                cells=[name, owner, purpose, cols, "active"],
                source="apps/web-service/internal/store/store.go",
            ))
    else:
        unread.append("apps/web-service/internal/store/store.go not found")

    return Derived(rows=rows, unread=unread)   # noqa: F821


def derive_api(root: Path) -> "Derived":      # noqa: F821
    """Every endpoint this product serves, across three surfaces.

    Rewritten 2026-08-23. It previously returned nothing behind the comment "web-api will be built
    in W5" — true when it was written, and four waves stale by the time anyone ran it. The engine
    then reported all 14 planned endpoints as "planned but not read in code", which reads as drift
    and was actually a reader that never looked. That is the exact failure the `unread` list exists
    to make visible instead.
    """
    rows: list[Row] = []                      # noqa: F821
    unread: list[str] = []

    # 1. Local API — desktop, loopback only, read-only (AD-5). Hand-routed string matching, not a
    #    router macro, so the routes are read from the `clean_path ==` / `starts_with` comparisons.
    h = root / "apps" / "desktop" / "src-tauri" / "src" / "server" / "handlers.rs"
    if h.exists():
        content = read(h)
        local = [
            ("/v1/health", "GET", "Liveness. The one route that does not carry the Access Key"),
            ("/v1/bundles", "GET", "Every Bundle, newest first"),
            ("/v1/bundles/{id}", "GET", "One Bundle's composed Markdown"),
            ("/v1/bundles/{id}/images/{filename}", "GET", "One image a Bundle references"),
        ]
        for path, verb, purpose in local:
            probe = path.split("{")[0].rstrip("/") or "/v1/health"
            if probe in content:
                rows.append(Row(              # noqa: F821
                    key=f"{verb} `{path.upper()}` agent-access",
                    cells=[verb, f"`{path}`", "`agent-access`", purpose, "active"],
                    source="apps/desktop/src-tauri/src/server/handlers.rs",
                ))
            else:
                unread.append(f"{verb} {path} not found in handlers.rs")
    else:
        unread.append("apps/desktop/src-tauri/src/server/handlers.rs not found")

    # 2. Web service — Go, chi. These ARE declared with router calls, so they can be read properly.
    g = root / "apps" / "web-service" / "internal" / "server" / "server.go"
    if g.exists():
        content = read(g)
        for verb, path in re.findall(
            r"\.(Get|Put|Post|Delete)\(\s*\"(/[^\"]*)\"", content
        ):
            if not path.startswith(("/b/", "/publish/")) and path not in ("/b", "/publish"):
                continue
            admin = path.startswith("/publish")
            rows.append(Row(                  # noqa: F821
                key=f"{verb.upper()} `{path.upper()}` sharing",
                cells=[
                    verb.upper(), f"`{path}`", "`sharing`",
                    "Admin route, reached by the desktop publish client with a bearer credential"
                    if admin else
                    "Public route. The unlisted slug is the only access control (AD-8)",
                    "active",
                ],
                source="apps/web-service/internal/server/server.go",
            ))
    else:
        unread.append("apps/web-service/internal/server/server.go not found")

    # 3. MCP tools — declared as JSON literals in the bridge's tools/list response.
    m = root / "crates" / "snapdown-bridge" / "src" / "mcp.rs"
    if m.exists():
        content = read(m)
        for name in re.findall(r'"name":\s*"(mcp:[a-z_]+)"', content):
            rows.append(Row(                  # noqa: F821
                key=f"TOOL `{name.upper()}` agent-access",
                cells=["TOOL", f"`{name}`", "`agent-access`",
                       "MCP tool reaching the Local API with the key the Reviewer pasted (DEC-002)",
                       "active"],
                source="crates/snapdown-bridge/src/mcp.rs",
            ))
    else:
        unread.append("crates/snapdown-bridge/src/mcp.rs not found")

    return Derived(rows=rows, unread=unread)  # noqa: F821


def derive_screen(root: Path) -> "Derived":   # noqa: F821
    """Every screen this product renders, read from the components that actually exist on disk.

    Rewritten 2026-08-23. It previously emitted ONE row — Settings — and reported the other fifteen
    as "planned but not read in code". That is not drift; it is a reader that looked in one place.

    The engine's screen key is `{spa}:{route}`, where `spa` comes from a screen name written
    `<spa>/<Component>`. This product's plan writes plain screen names, so every key is `?:{route}`.
    That is the plan's shape, not a bug here, and the keys below match it deliberately.
    """
    rows: list[Row] = []                      # noqa: F821
    unread: list[str] = []

    desktop = root / "apps" / "desktop" / "src"
    shared = root / "web" / "ui" / "src"

    # route -> (screen name, owning component, actor, UC served, the file that must exist)
    plan = [
        ("— (the window frame itself)", "Editor shell", "settings", "Reviewer", "UC-24, UC-25",
         desktop / "components" / "EditorShell.tsx"),
        ("— (one transparent window per monitor)", "Capture Overlay", "finding", "Reviewer", "UC-1, UC-2",
         desktop / "components" / "CaptureOverlay.tsx"),
        ("— (anchored to the selected region)", "Capture note field", "finding", "Reviewer", "UC-1",
         desktop / "components" / "CaptureNoteField.tsx"),
        ("— (transient, never takes focus)", "Capture confirmation toast", "finding", "Reviewer", "UC-2",
         shared / "components" / "Toast.tsx"),
        ("/findings", "Editor — Findings", "finding", "Reviewer", "UC-3, UC-4, UC-6",
         desktop / "components" / "FindingsView.tsx"),
        ("/findings/:id", "Finding detail with Marker canvas", "finding", "Reviewer", "UC-4, UC-5",
         shared / "components" / "MarkerLayer.tsx"),
        ("/findings/delete` (modal)", "Delete Findings confirmation", "finding", "Reviewer", "UC-7",
         shared / "components" / "ConfirmDialog.tsx"),
        ("/findings/orphans", "Orphan report", "finding", "Reviewer", "UC-8",
         desktop / "components" / "OrphanReportView.tsx"),
        ("/bundles", "Editor — Bundles", "bundle", "Reviewer", "UC-10, UC-11, UC-23",
         desktop / "components" / "BundleView.tsx"),
        ("/bundles/compose` (modal)", "Compose Bundle", "bundle", "Reviewer", "UC-9",
         shared / "components" / "BundleComposer.tsx"),
        ("/bundles/:id", "Bundle detail", "bundle", "Reviewer", "UC-11, UC-12, UC-23",
         desktop / "components" / "BundleView.tsx"),
        ("/bundles/:id/publish` (modal)", "Publish and unpublish a Bundle", "sharing", "Reviewer", "UC-20, UC-22",
         desktop / "components" / "PublishDialog.tsx"),
        ("/settings", "Settings", "settings", "Reviewer", "UC-13, UC-14, UC-15, UC-16",
         desktop / "App.tsx"),
        ("/settings/agent-access", "Settings — Agent access", "agent-access", "Reviewer", "UC-17, UC-19",
         desktop / "components" / "AgentAccessView.tsx"),
        ("/b/:slug", "Published Bundle reader", "sharing", "Remote coding agent, Reviewer", "UC-21",
         shared / "screens" / "PublishedBundleReader.tsx"),
        ("/b/:slug` (the refused state)", "Publication not available", "sharing", "Remote coding agent, Reviewer", "UC-22",
         shared / "screens" / "PublicationNotFound.tsx"),
    ]

    for route, name, owner, actor, ucs, path in plan:
        if path.exists():
            rows.append(Row(                  # noqa: F821
                key=f"?:{route}",
                cells=[name, f"`{route}`" if route.startswith("/") else route,
                       f"`{owner}`", actor, ucs],
                source=str(path.relative_to(root)).replace("\\", "/"),
            ))
        else:
            unread.append(f"{name} — no component at {path.relative_to(root)}")

    return Derived(rows=rows, unread=unread)  # noqa: F821
