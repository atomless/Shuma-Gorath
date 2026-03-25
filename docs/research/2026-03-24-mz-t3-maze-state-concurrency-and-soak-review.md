# MZ-T3 Maze State Concurrency And Soak Review

Date: 2026-03-24
Status: Accepted

## Why this tranche exists

`MZ-T1` and `MZ-T2` now prove the live maze path and the live browser/session path. They still do not prove that the maze state primitives remain correct under burst concurrency:

- shared maze budget acquisition and release,
- replay protection over the same traversal token,
- checkpoint visibility and bounded fallback behavior under overlapping requests.

The current primitives are also a real correctness seam, not only a missing test seam. The active logic is mostly read-then-write over `MazeStateStore`:

- [`src/deception/primitives.rs`](../../src/deception/primitives.rs) `try_acquire_shared_budget(...)`
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs) `replay_seen(...)` + `mark_replay_seen(...)`
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs) checkpoint load/write logic

Under overlapping same-process requests, those sequences can admit races unless the tranche adds a small correctness guard in the same place the new burst proof will target.

## Existing truth we should reuse

- [`src/deception/primitives.rs`](../../src/deception/primitives.rs) already centralizes the shared budget primitive used by both maze and tarpit.
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs) already centralizes token replay, issue replay, checkpoint state, and high-confidence escalation logic.
- [`src/maze/simulation.rs`](../../src/maze/simulation.rs) already proves deterministic crawler semantics for replay and checkpoint behavior in single-threaded form.
- [`docs/plans/2026-02-25-maze-carry-forward-plan.md`](../plans/2026-02-25-maze-carry-forward-plan.md) already defines `MZ-T3` as concurrency/soak closure over state and budget primitives.

## Review conclusion

The cleanest `MZ-T3` slice is:

1. add focused burst/concurrency tests directly around the real maze state primitives,
2. add the smallest shared-host correctness hardening needed for those tests to be truthful,
3. expose the result through a focused native `make` target, not a large live-server soak harness.

Given the repo’s current shared-host-first mainline, a process-local critical-section guard is the right minimal hardening for this tranche:

- enough to prevent same-process oversubscription and duplicate replay admission under burst load,
- small and local to the actual state-transition seams,
- without pretending to solve a later multi-instance distributed coordination problem that the repo is not currently executing.

## Guardrails

- Keep this tranche focused on maze state correctness, not a generalized distributed locking subsystem.
- Reuse the real maze/tarpit shared-budget primitive rather than adding maze-only duplicate counter logic.
- Make the new `make` target truthful about scope: native burst/concurrency proof over state primitives, not a live full-path browser/integration soak.
