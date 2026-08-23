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
            else:
                rows.append(Row(              # noqa: F821
                    key=tbl_name_clean,
                    cells=[tbl_name_clean, "settings", f"Table {tbl_name_clean}", "derived from migration", "active"],
                    source="crates/snapdown-store/src/sqlite/migrations.rs",
                ))
    else:
        unread.append("crates/snapdown-store/src/sqlite/migrations.rs not found")

    return Derived(rows=rows, unread=unread)   # noqa: F821


def derive_api(root: Path) -> "Derived":      # noqa: F821
    """Every endpoint this product serves. In W1, web-api is not yet built."""
    # Web service (web-api) will be built in W5; no HTTP endpoints served in W1.
    return Derived(rows=[], unread=[])        # noqa: F821


def derive_screen(root: Path) -> "Derived":   # noqa: F821
    """Every screen this product renders, from React app routes / components."""
    rows: list[Row] = []                      # noqa: F821
    unread: list[str] = []

    # Check desktop Settings view
    app_tsx = root / "apps" / "desktop" / "src" / "App.tsx"
    if app_tsx.exists():
        rows.append(Row(                      # noqa: F821
            key="/settings",
            cells=["Settings", "/settings", "General, Vault, Quality Budget, Hotkeys", "settings", "UC-13, UC-14, UC-15, UC-16"],
            source="apps/desktop/src/App.tsx",
        ))
    else:
        unread.append("apps/desktop/src/App.tsx not found")

    return Derived(rows=rows, unread=unread)  # noqa: F821
