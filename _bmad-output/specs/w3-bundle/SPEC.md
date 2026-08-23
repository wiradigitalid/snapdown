---
id: SPEC-W3
wave: W3
title: Bundle complete — compose, list, copy, delete
status: draft
created: 2026-08-23
---

# Wave W3 Specification Contract: Bundle

## Stories Breakdown
1. **W3-S1**: SQLite Schema Migration v3 (`bundle`, `bundle_item`), `Bundle` & `BundleItem` domain entities, and `BundleStore` implementation.
2. **W3-S2**: Markdown serialization engine with embedded images and relative path formatting (`Pure Markdown Writer`).
3. **W3-S3**: Marker burning pipeline onto exported screenshots (`Marker Burner`).
4. **W3-S4**: Bundle Composer UI in `@snapdown/ui` and `apps/desktop` for selecting findings, rearranging ordinals, and bundling.
5. **W3-S5**: Clipboard export bridge (`copy to clipboard`) and disk bundle export with golden-file regression test suite.
6. **W3-S6**: Bundle deletion with transactional cascade to referenced bundle items and file cleanup.
