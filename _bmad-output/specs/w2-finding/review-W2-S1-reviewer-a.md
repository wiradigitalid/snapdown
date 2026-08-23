---
reviewer: reviewer-a
model: 9router/combo
date: 2026-08-23
story: W2-S1
verdict: ACCEPTED
---

# Code Review: W2-S1 — SQLite Schema for Finding, Note, and Marker tables with FindingStore

## Scope & Implementation Review
- **Domain Entities & Types**: Implemented `Finding`, `Note`, `Marker`, `Region`, `FindingKind`, and `FindingStatus` in `crates/snapdown-core/src/domain/finding.rs` adhering to value object constraints and validation invariants (e.g. 0.0..=1.0 relative coordinates, positive marker ordinals).
- **Store Port Trait**: `FindingStore` interface cleanly declared in `snapdown-core/src/ports/finding_store.rs` without any platform/IO dependencies.
- **Migration & SQLite Implementation**: Added migration v2 in `snapdown-store/src/sqlite/migrations.rs` creating `finding`, `note`, and `marker` tables with foreign keys and index constraints. Implemented `SqliteFindingStore` supporting transactional find, save, delete, list, and contiguous marker renumbering.
- **Verification**: Unit and integration tests cover migration idempotency, CRUD lifecycle, single-sequence contiguous renumbering on delete, and invalid relative coordinate rejection.

## Invariant Adherence
- `INV-SCHEMA-001` (Migration Idempotence): Verified via test `migrations_v2_apply_cleanly_and_idempotently`.
- `INV-STORE-001` (Transactional Atomicity): Verified with rollback guarantees on finding + note + marker operations.
- `INV-DOMAIN-001` (Normalized Coordinates): Coordinates clamped and validated to `[0.0, 1.0]`.

## Verdict
ACCEPTED.
