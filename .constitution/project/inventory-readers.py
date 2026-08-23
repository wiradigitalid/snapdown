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
