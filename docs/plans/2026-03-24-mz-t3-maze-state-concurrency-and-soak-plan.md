# MZ-T3 Maze State Concurrency And Soak Plan

Date: 2026-03-24
Status: Implemented
Parent plan: [`2026-02-25-maze-carry-forward-plan.md`](2026-02-25-maze-carry-forward-plan.md)

## Goal

Add burst/concurrency coverage for maze replay, checkpoint, and shared-budget primitives, and harden the real shared-host state transition seams where that proof would otherwise reveal same-process races.

## Scope

Land focused native proof for:

1. shared maze budget acquisition does not oversubscribe under parallel attempts,
2. releasing acquired budget returns counters to a bounded steady state,
3. concurrent reuse of the same maze traversal token admits one winner and deterministic replay fallback for the loser(s),
4. repeated concurrent checkpoint-missing style pressure does not create unbounded counter/key drift.

## Implementation shape

- Add focused burst/concurrency tests in the owning native modules:
  - [`src/deception/primitives.rs`](../../src/deception/primitives.rs)
  - [`src/maze/runtime.rs`](../../src/maze/runtime.rs)
- Add the smallest shared-host correctness hardening needed for those tests to pass truthfully.
  - Prefer local process-critical sections around the existing state transitions over a broader architectural lock system.
- Add a focused `make` target:
  - `make test-maze-state-concurrency-contract`

## Coverage contract

The new native proof should cover:

1. **Budget primitive burst**
   - parallel acquisitions against a cap of `1`,
   - no more than one successful lease at a time,
   - counters return to `0` after release.

2. **Replay primitive burst**
   - concurrent requests using the same signed traversal token,
   - exactly one request can advance through the maze,
   - the competing request(s) deterministically fall back as replay.

3. **Bounded state footprint**
   - burst replay/checkpoint pressure does not create more marker/counter state than the bounded primitive requires for that flow/bucket.

## Guardrails

- Do not widen this tranche into a live browser or full HTTP soak harness; keep it native and deterministic.
- Keep the correctness hardening truthful to the current single-host shared-process operating model.
- Do not create maze-specific duplicate budget code; if the shared budget primitive needs hardening, harden the shared primitive.
