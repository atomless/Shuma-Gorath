# MZ-T1 Live Maze Spin Integration Plan

Date: 2026-03-24
Status: Implemented
Parent plan: [`2026-02-25-maze-carry-forward-plan.md`](2026-02-25-maze-carry-forward-plan.md)

## Goal

Land the missing live Spin-path proof for opaque maze traversal across multiple hops with deterministic fallback action and reason assertions.

## Scope

Add a focused local gate that proves:

1. admin preview can discover a real opaque public maze path,
2. the public opaque path serves a tokenized follow-on link,
3. following deeper without a checkpoint produces the expected deterministic fallback,
4. `POST <maze_path_prefix>checkpoint` accepts the live checkpoint token,
5. `POST <maze_path_prefix>issue-links` returns bounded hidden links after checkpointed progress,
6. a replayed token escalates through the expected live fallback path and is evidenced in recent events.

## Implementation shape

- Add a dedicated script at [`scripts/tests/maze_live_traversal.py`](../../scripts/tests/maze_live_traversal.py).
- Add focused helper tests at [`scripts/tests/test_maze_live_traversal.py`](../../scripts/tests/test_maze_live_traversal.py).
- Add canonical `make` targets:
  - `make test-maze-live-traversal-unit`
  - `make test-maze-live-traversal-contract`

## Guardrails

- Restore only the exact config keys the gate mutates.
- Use loopback identity for admin and health reads.
- Use a fresh external-looking forwarded IP bucket for the public traversal path so stale violation state cannot pollute assertions.
- Assert persisted recent-event evidence, not only page copy.
