---
id: W4-S4
title: Settings Agent Access UI screen and Tauri IPC commands
wave: W4
status: planned
created: 2026-08-23
dependencies: [W4-S1, W4-S2]
files:
  - web/ui/src/screens/AgentAccessView.tsx
  - web/ui/src/index.ts
  - apps/desktop/src/components/AgentAccessView.tsx
  - apps/desktop/src/services/agent_access.ts
  - apps/desktop/src-tauri/src/commands/agent_access.rs
  - apps/desktop/src-tauri/src/lib.rs
  - apps/desktop/src/test/agent_access_view.test.tsx
---

# W4-S4: Settings Agent Access UI screen and Tauri IPC commands

## User Story
As a reviewer, I want a Settings screen at `/settings/agent-access` to issue a new Access Key, view active key status/timestamp, copy/re-copy the secret key to my clipboard, and revoke active access at any time.

## Acceptance Criteria
- [ ] Create `AgentAccessView.tsx` component in `@snapdown/ui` and `apps/desktop` adhering to project design system tokens.
- [ ] Render active key status: Issued timestamp, active indicator, or "No active key" state.
- [ ] Implement UI actions: "Generate Access Key", "Copy Key", "Revoke Key".
- [ ] Implement Tauri backend IPC commands in `apps/desktop/src-tauri/src/commands/agent_access.rs`:
  - `get_access_key_status`
  - `generate_access_key`
  - `copy_access_key`
  - `revoke_access_key`
- [ ] Secure memory handling: Access key cleartext is never logged or written to disk.
- [ ] Vitest component unit tests with mocked Tauri invoke bridge.
