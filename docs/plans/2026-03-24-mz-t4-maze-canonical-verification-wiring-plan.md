# MZ-T4 Maze Canonical Verification Wiring Plan

Date: 2026-03-24
Status: Implemented
Parent plan: [`2026-02-25-maze-carry-forward-plan.md`](2026-02-25-maze-carry-forward-plan.md)

## Goal

Make the new maze live traversal, live browser, and concurrency proof part of Shuma's canonical local and CI verification path so maze regressions fail fast before merge.

## Scope

Land one canonical maze verification gate that covers:

1. deterministic benchmark protection,
2. live Spin traversal proof,
3. live browser/session proof,
4. native burst/concurrency proof.

Then wire that gate into the canonical local and CI paths that are supposed to represent pre-merge truth.

## Implementation shape

- Add a focused wiring proof in `scripts/tests/` that asserts:
  - the canonical maze aggregate target exists,
  - it includes the four expected maze proof targets,
  - `make test` routes through that aggregate target,
  - the relevant CI workflow path still executes the canonical umbrella or equivalent maze gate.
- Add the aggregate `make` target in [`../../Makefile`](../../Makefile).
- Replace the old maze-only benchmark step inside `make test` with the new aggregate gate.
- Keep release-oriented CI honest by reusing the same aggregate gate rather than a divergent ad hoc maze command list.
- Update testing/docs/TODO history.

## Guardrails

- Do not invent a second canonical maze story; there should be one aggregate gate reused by the umbrella and CI wiring.
- Keep naming truthful. This target is a verification gate, not a pure unit contract and not a remote operational proof.
- Reuse existing browser/bootstrap behavior; do not add a second Chromium install path just for maze.
