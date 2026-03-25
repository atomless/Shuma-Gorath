# Maze Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/2026-02-13-maze-excellence-plan.md`](2026-02-13-maze-excellence-plan.md)

## Scope

Core maze architecture from the 2026-02-13 plan is delivered. This carry-forward plan focuses on remaining test-closure work.

## Remaining Work

1. MZ-T1: Add Spin integration coverage for live opaque maze traversal across multiple hops.
   - Include entry -> tokenized follow -> checkpoint -> issue-links -> fallback/escalation assertions.

2. MZ-T3: Add concurrency/soak coverage for maze state/budget primitives.
   - Validate replay/checkpoint/budget key behavior under burst traversal and confirm bounded host-write behavior.

3. MZ-T4: Wire the new integration/E2E/soak coverage into canonical Makefile + CI gates.

## Definition of Done

- Maze live-path behavior is covered by integration, browser E2E, and soak tests.
- CI fails fast on maze regression in traversal correctness, replay protection, or budget behavior.
