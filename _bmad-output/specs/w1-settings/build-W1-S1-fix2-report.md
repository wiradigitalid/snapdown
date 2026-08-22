# Build Report — W1-S1 Fix Round 2 of 2

## Changes Implemented

1. **MF2R-1 (`crates/snapdown-core/src/util/id.rs`, `ports/mod.rs`, `lib.rs`)**:
   - Replaced `id_from_timestamp(seconds, nanos)` with `id_from_parts(unix_millis: u64, rand_b: [u8; 10]) -> String`.
   - Entropy is supplied explicitly from external ports rather than derived/assumed internally.
   - Formats a 16-byte UUIDv7 representation into a canonical lowercase hyphenated UUID string conforming to RFC 9562 §5.7 without depending on the `uuid` crate.
   - Extended `Clock` port with `now_unix_millis(&self) -> u64` and added `EntropySource` port trait (`random_bytes_10(&self) -> [u8; 10]`).
   - Rewrote unit tests to assert format, lowercase hex structure, version (7), variant (0b10xxxxxx), and differentiation on distinct random parts.

2. **MF2R-2 (`crates/snapdown-core/Cargo.toml`, `crates/snapdown-core/tests/test_no_io.rs`)**:
   - Dropped `uuid` dependency completely from `snapdown-core`.
   - Updated dependency graph traversal test in `test_no_io.rs` to invoke `cargo metadata` with `--filter-platform <triple>` and removed `target.is_none()` restrictions so target-gated normal dependencies are fully tracked.
   - Verified that `getrandom` and OS entropy dependencies are completely eliminated from `snapdown-core`.

3. **MF2R-3 (`web/ui/src/components/Modal.tsx`, `web/ui/src/styles/components.css`, `web/ui/src/test/components.test.tsx`)**:
   - Reproduced the issue before fixing: tests failed against the un-cleared `isClosing` state.
   - Fixed `Modal.tsx` so `isClosing` is reset to `false` when `isOpen` becomes `false`, allowing the dialog to unmount properly when closed via Escape or scrim click.
   - Added styles for `modal-overlay` and `modal-content` with `[data-state="closing"]` and `[data-state="open"]` in `components.css`.
   - Added comprehensive tests for unmounting on close and keyboard Tab focus trapping.

4. **F-5 (`web/ui/src/components/Toast.tsx`, `web/ui/src/test/components.test.tsx`)**:
   - Removed `tabIndex={-1}` from the action button in `Toast.tsx` so that action buttons are keyboard accessible.
   - Added unit test asserting action button is tabbable and clickable.

## Extra Proofs

### MF2R-2 Proof: `cargo tree --workspace -e normal -i getrandom@0.4.3`
```
getrandom v0.4.3
├── uuid v1.25.0
│   ├── cfb v0.7.3
│   │   └── infer v0.19.0
│   │       ├── tauri-utils v2.9.3
│   │       │   ├── tauri-codegen v2.6.3
│   │       │   │   └── tauri-macros v2.6.3 (proc-macro)
│   │       │   │       └── tauri v2.11.5
│   │       │   │           ├── desktop v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\apps\desktop\src-tauri)
│   │       │   │           └── tauri-plugin-single-instance v2.4.3
│   │       │   │               └── desktop v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\apps\desktop\src-tauri)
│   │       │   └── tauri-macros v2.6.3 (proc-macro) (*)
│   │       └── tauri-utils v2.9.3
│   │           ├── tauri v2.11.5 (*)
│   │           ├── tauri-runtime v2.11.3
│   │           │   ├── tauri v2.11.5 (*)
│   │           │   └── tauri-runtime-wry v2.11.4
│   │           │       └── tauri v2.11.5 (*)
│   │           └── tauri-runtime-wry v2.11.4 (*)
│   ├── schemars v0.8.22
│   │   └── tauri-utils v2.9.3 (*)
│   ├── tauri-codegen v2.6.3 (*)
│   └── tauri-utils v2.9.3 (*)
└── uuid v1.25.0
    ├── snapdown-store v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\crates\snapdown-store)
    │   └── desktop v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\apps\desktop\src-tauri)
    └── tauri-utils v2.9.3 (*)
```
`snapdown-core` does not appear anywhere in the `getrandom` dependency tree.

### MF2R-3 Proof: Modal test failure before fix vs pass after fix

**Test failure before fix:**
```
FAIL src/test/components.test.tsx > web/ui components suite > Modal > unmounts when closed via Escape or scrim click from stateful parent
Error: expect(element).not.toBeInTheDocument()

expected document not to contain element, found <div
  aria-modal="true"
  data-state="closing"
  role="dialog"
  style="background-color: var(--color-surface); color: var(--color-text); border-radius: var(--radius-md); box-shadow: var(--shadow-raised); border: 1px solid var(--color-border); width: 100%; max-width: 32rem; padding: var(--space-5); display: flex; flex-direction: column; gap: var(--space-4); outline: none;"
  tabindex="-1"
>
  <h2
    style="margin: 0px; font-family: var(--font-ui); font-size: var(--text-lg); font-weight: 600;"
  >
    Test Modal
  </h2>
  <div>
    <button>
      Inside Button
    </button>
  </div>
</div> instead
```

**Test pass after fix:**
```
✓ src/test/components.test.tsx (16 tests) 133ms
Test Files  1 passed (1)
Tests  16 passed (16)
```

## Verification Command Outputs

1. `cargo fmt --all -- --check`
   Output: clean, 0 diffs.

2. `cargo clippy --workspace --all-targets -- -D warnings`
   Output:
   ```
   Checking snapdown-core v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\crates\snapdown-core)
   Checking snapdown-store v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\crates\snapdown-store)
   Checking desktop v0.1.0 (D:\Developer\orca-workspaces\snapdown\w1-settings\apps\desktop\src-tauri)
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.60s
   ```

3. `cargo test --workspace`
   Output:
   ```
   running 3 tests
   test domain::setting::tests::quality_budget_defaults_and_validation ... ok
   test domain::setting::tests::setting_key_roundtrip ... ok
   test util::id::tests::generates_valid_lowercase_hyphenated_uuidv7_from_parts ... ok
   test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   running 1 test
   test snapdown_core_has_no_io_dependency ... ok
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

   running 1 test
   test tests::store_crate_initializes ... ok
   test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
   ```

4. `uv run .constitution/method/scripts/validate.py --check`
   Output:
   ```
   RED — 8 findings across 3 validators

     V18  W1-S2: has no story file in _bmad-output/specs/w1-settings/stories/
     V18  W1-S3: has no story file in _bmad-output/specs/w1-settings/stories/
     V18  W1-S4: has no story file in _bmad-output/specs/w1-settings/stories/
     V18  W1-S5: has no story file in _bmad-output/specs/w1-settings/stories/
     V24  .agent/skills/bmad-project-context/references/template.md: cites `src/lib/money.ts` which does not exist
     V24  .agent/skills/bmad-project-context/references/template.md: cites `src/routes/webhooks.ts` which does not exist
     V25  mcp-bridge: `built: true` MUST have a heading in the code map
     V25  web-api: `built: true` MUST have a heading in the code map
   ```
   Finding count: exactly 8 findings, matching `.github/validate-baseline.txt` with zero difference.

5. `apps/desktop` verification:
   - `npm run typecheck` -> passed (`tsc --noEmit`)
   - `npm run lint` -> passed (`eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0`)
   - `npm run test` -> passed (`1 vitest passed`)
   - `npm run build` -> passed (`vite v7.3.6 building client environment for production... built in 704ms`)

6. `web/ui` verification:
   - `npm run typecheck` -> passed (`tsc --noEmit`)
   - `npm run lint` -> passed (`eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0`)
   - `npm run test` -> passed (`16 vitest passed`)
