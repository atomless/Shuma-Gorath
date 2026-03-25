# MZ-T3 Maze State Concurrency And Soak Post-Implementation Review

Date: 2026-03-24
Status: Complete

## Delivered

`MZ-T3` is now covered by a focused native burst/concurrency gate in:

- [`src/deception/primitives.rs`](../../src/deception/primitives.rs)
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs)
- [`Makefile`](../../Makefile) as:
  - `make test-maze-state-concurrency-contract`

The new proof now covers:

- shared budget acquisition under burst contention,
- replay-claim races over the same maze traversal token,
- concurrent checkpoint writes reusing one checkpoint key,
- bounded counter/key footprint for the burst scenarios.

## Important implementation notes

- The tranche intentionally paired proof with minimal correctness hardening. The current shared-host maze/tarpit state primitives were read-then-write store transitions, so same-process burst proof would otherwise be testing a known race rather than a truthful contract.
- The hardening stays local and small:
  - a shared-host critical section in [`src/deception/primitives.rs`](../../src/deception/primitives.rs) for budget acquire/release,
  - a maze progression critical section in [`src/maze/runtime.rs`](../../src/maze/runtime.rs) for replay/checkpoint/issue transitions.
- This is intentionally not a claim of distributed multi-instance coordination. It is the correct minimal guard for the repo’s current shared-host-first execution model.

## Follow-on

- `MZ-T4` is still required to wire the new maze traversal, browser, and state-concurrency gates into broader canonical CI/full-suite paths.
