---
type: uc
id: UC-15
component: settings
satisfies: [FR-17]
critical: false
created: "2026-08-23"
---

# UC-15 — I change the keys that set Snapdown off, because one of them clashes

## Trigger

A hotkey stopped working, or the Reviewer wants a combination that suits their hands better.

## Precondition

Snapdown is running. Each action has a combination or is explicitly disabled (`BR-113`).

## Main Flow

1. The Reviewer opens Settings. Every registered hotkey is listed with its combination and whether it
   is Active or Disabled, in words as well as colour.
2. The Reviewer clicks the chip for the action they want to change; it begins listening.
3. The Reviewer presses the combination.
4. Snapdown captures it and stops listening. A bare modifier press is ignored rather than captured.
5. Snapdown checks the combination against its own other actions (`BR-27`) and against the operating
   system.
6. Snapdown registers it, unregisters the old one, and stores the change.
7. The new combination works and the old one does not, with nothing restarted.

## Alternate Flows

| From step | Condition | What happens |
| --- | --- | --- |
| 2 | The Reviewer presses Esc | Listening stops and the previous combination is restored. Esc does not close the window |
| 2 | Focus leaves the chip | Listening stops the same way. A chip that keeps listening after it loses focus swallows the Reviewer's keystrokes everywhere |
| 4 | The Reviewer wants the action off entirely | They clear it. The action is disabled and the row says **Disabled** rather than showing an empty box that reads as broken (`BR-113`) |
| 5 | The combination is held by another Snapdown action | Refused, naming the other action. Snapdown's own conflict is reported differently from an outside one, because the Reviewer can resolve it and the wording should say so |
| 5 | The combination is held by a program that is not running right now | It binds. Registration is attempted again at each startup, and reported if it then fails (`BR-26`). An honest failure later beats a wrong refusal now |

## Failure Flows

| Condition | What happens |
|---|---|
| The combination is held by another running program | Refused at binding time, naming the program where Windows tells us and saying "another program" where it does not. The old combination stays in effect |
| Registration fails at startup | The row carries a warning badge and a line naming the conflict, before the Reviewer has noticed anything is wrong. It is never swallowed (`BR-26`, `NFR-7`) |
| Registration fails and the Setting was already stored | The Setting keeps the Reviewer's choice. The failure is a fact about the OS right now, not a value (`BR-114`) |
| The settings store cannot be written | The change is refused and the previous binding stays registered. The chip does not show a combination that is not in effect |

## Postcondition

Each action has exactly one combination or is explicitly disabled. No two actions share one
(`BR-27`). What the screen shows is what is registered, or the screen says why it is not.

## The distinction this use case turns on

There are three different things that all look like "the hotkey doesn't work", and the product treats
them as three:

- **Refused at binding** — Snapdown knows now, and the old combination is still working.
- **Disabled** — the Reviewer chose this, and nothing is wrong.
- **Failed at startup** — the Setting is right and the world changed underneath it.

Collapsing them into one error message is how a Reviewer ends up rebinding a hotkey that was never the
problem.
