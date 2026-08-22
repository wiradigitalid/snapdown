---
topic: W1 — workspace, the two stores, and Settings complete
artifact: _bmad-output/specs/w1-settings/SPEC.md
updated: 2026-08-22T23:22
---

- (event) wave W1 opened. Release r1, PRD capture-to-markdown, FR-5/16/17/18, size L, component settings
- (decision) five waves, one component each. Not a method rule — a wave MAY cross components — but it keeps V3 green after every wave instead of only at the end. The cost is that W2 is a wave L
- (decision) settings goes first: four of five components read a Setting before they can do anything, so building it first means no later wave stubs it. It is also the cheapest place to get the workspace shape wrong
- (change) LC-009 hotkey-registrar moved from finding to settings during wave planning. settings owns the binding it registers; finding subscribes to a capture-requested event. Propagated into SDD-finding and SDD-settings
- (decision) SPEC carries the verification commands because codebase-stack-guide.md is born empty and its own header forbids filling it before code ratifies it. They are promoted into that guide at wave close
- (decision) web/ui tokens and base elements are created in W1 even though the web service is W5 — the desktop webview imports them, and design-system.md already names that path as the source of truth
- (note) W1-S1 and W1-S2 satisfy no use case, deliberately. They are substrate, and their success criterion is the named tests passing
- (note) the capture-requested event is wired and left unconsumed. A placeholder capture would be a lie about what the wave delivered
- (gap) hotkey conflict detection at binding time is assumed possible on Windows 11 (BR-26). If it is not, FR-17's promise weakens and that is the owner's call, not a code workaround
- (gap) no CI existed before this wave. korpus.yml is born here, and it is expected red on V24 and V25 until code lands
