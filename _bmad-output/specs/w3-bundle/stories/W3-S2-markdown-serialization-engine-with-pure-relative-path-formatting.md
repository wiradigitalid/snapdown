---
id: W3-S2
title: Markdown serialization engine with pure relative path formatting
wave: W3
status: done
created: 2026-08-23
dependencies:
  - W3-S1
files:
  - crates/snapdown-core/src/domain/markdown.rs
  - crates/snapdown-core/src/domain/mod.rs
  - crates/snapdown-core/src/lib.rs
  - crates/snapdown-core/tests/test_markdown_serializer.rs
---

# W3-S2: Markdown serialization engine with pure relative path formatting

## User Story
As a user sharing test findings across teams, I want a pure Markdown serialization engine that generates human-readable and standard-compliant Markdown with embedded relative image links and numbered marker callouts, so that reports can be rendered anywhere without proprietary dependencies.

## Acceptance Criteria
- [ ] Implement `MarkdownSerializer` in `crates/snapdown-core/src/domain/markdown.rs`.
- [ ] Format bundle title, captured finding headers, note descriptions, and marker annotations with numbered lists.
- [ ] Format image paths relative to bundle markdown root (`./images/...` or `./findings/...`).
- [ ] No I/O dependencies inside `snapdown-core` (`test_no_io.rs` remains green).
- [ ] Automated test suite covering single/multi-finding bundles and special characters escaping.
