# MZ-T1 Live Maze Spin Integration Review

Date: 2026-03-24
Status: Accepted

## Why this tranche exists

The maze module already had strong native and in-memory traversal proof, but `MZ-T1` in the active carry-forward plan still lacked live Spin-path evidence for:

- opaque public entry,
- tokenized multi-hop traversal,
- checkpoint acceptance,
- progressive `issue-links` issuance,
- and deterministic fallback action and reason assertions.

That left a real gap between runtime semantics and what the canonical local verification surface actually proved.

## Existing truth we should reuse

- [`src/maze/simulation.rs`](../../src/maze/simulation.rs) already proves replay, checkpoint-missing, checkpointed traversal, and binding semantics against the real runtime helpers.
- [`src/maze/runtime.rs`](../../src/maze/runtime.rs) already exposes the exact live contracts we need:
  - `POST <maze_path_prefix>checkpoint`
  - `POST <maze_path_prefix>issue-links`
  - deterministic fallback reason/action mapping
- [`docs/plans/2026-02-25-maze-carry-forward-plan.md`](../plans/2026-02-25-maze-carry-forward-plan.md) already defines the required live sequence for `MZ-T1`.

## Review conclusion

The cleanest slice is a dedicated focused live gate, not more archaeology in the giant integration shell.

That gate should:

1. snapshot and restore only the exact runtime keys it mutates,
2. use admin preview only to discover the opaque public entry path,
3. traverse the real public maze path from an external-looking forwarded IP,
4. assert both direct HTTP fallback shape and persisted recent-event evidence,
5. remain exposed through a truthful focused `make` target.
