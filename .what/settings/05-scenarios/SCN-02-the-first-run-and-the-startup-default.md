---
type: scenario
id: SCN-02
component: settings
branches_from: UC-16
created: "2026-08-23"
---

# SCN-02 — The first run, and the startup default that must not come back

Branches from `UC-16`. It exists because `BR-112` is a rule about *time*, and a rule about time cannot
be checked by looking at one moment.

## The three runs that must be distinguished

| Run | Stored value | Windows registration | What must happen |
|---|---|---|---|
| **1 — fresh install** | unset | none | Snapdown registers itself. The toggle shows **On** |
| **2 — Reviewer turns it off** | `off` | removed | The toggle shows **Off** |
| **3 — next sign-in** | `off` | none | Snapdown does **not** re-register. The toggle shows **Off** |

Run 3 is the whole scenario. The naive implementation — *if not registered, register* — passes runs 1
and 2 and fails run 3 silently, by doing exactly what the Reviewer asked it not to. Nothing errors,
nothing logs, and the Reviewer discovers it by noticing Snapdown running again.

## The fourth run, which is the trap

| Run | Stored value | Windows registration | What must happen |
|---|---|---|---|
| **4 — registration removed outside Snapdown** | `on` | none | The toggle shows **Off** |

Something else removed the registration: a cleanup tool, a policy, a Windows reset, another user
profile. The stored value says `on` and the truth says otherwise.

`FR-18` and `BR-114` settle it: the control reflects the **actual** registration, so it shows **Off**.
Snapdown does not silently re-register to make the stored value true, because it cannot tell run 4
from run 3 — and being wrong in run 3's direction means overriding a decision, while being wrong in
run 4's direction means only that a Reviewer clicks a toggle.

The stored value is not thereby useless. `BR-112` reads it to tell *unset* from *off*, which is the
one question the OS cannot answer.

## The fifth run, found by review

| Run | Stored value | Windows registration | What happens |
|---|---|---|---|
| **5 — the store is lost, the registration is not** | *(store gone)* | matches whatever it was | `startup.registered` is absent, so the first-run branch fires |

A `library.db` that is replaced, corrupted and reopened, or moved without its Vault takes
`startup.registered` with it. The OS registration is untouched, because it lives in the registry.

Where the Reviewer had it **on**, this is harmless: the branch registers something already registered
and writes `expressed`.

Where the Reviewer had it **off**, the default re-applies and Snapdown starts registering itself
again. `BR-112` is not violated — it says the default applies to a first run where nothing was
configured, and as far as anything can tell, this *is* one. The record of the decision was lost, and
a rule cannot honour a decision nothing remembers.

This is recorded rather than fixed. The alternative — reading the OS registration to infer intent when
the store is absent — is exactly the conflation this whole scenario exists to forbid, and it would get
run 4 wrong to get run 5 right. Losing the store is rare; losing it is already visible to the
Reviewer, because every Finding goes with it.

## What the Reviewer sees during the read

Every run passes through `Unknown` (`state-machines.md` § 1) before it can show anything. The toggle
renders its not-yet-known state and is inert until Windows answers. It never renders `On` or `Off`
first.

## Tests this scenario names

- `settings::a_first_run_with_nothing_configured_registers_at_startup`
- `settings::a_run_after_the_reviewer_disabled_it_does_not_re_register`
- `settings::a_registration_removed_outside_snapdown_shows_off_and_is_not_restored`
- `settings::the_startup_toggle_renders_unknown_until_the_os_has_answered`
- `settings::the_startup_toggle_never_renders_a_definite_state_before_the_read_resolves`
