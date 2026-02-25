# Rate Limiting Carry-Forward Plan

Date: 2026-02-25
Status: Active (carry-forward)
Supersedes: Historical baseline in [`docs/plans/archive/2026-02-13-rate-limiting-excellence-plan.md`](archive/2026-02-13-rate-limiting-excellence-plan.md)

## Scope

This plan carries forward the unfinished slices from the 2026-02-13 rate-limiting plan.

Delivered baseline already includes:
- Atomic external distributed counter adapter (Redis `INCR` + TTL).
- Route-class outage posture and fallback controls.
- Admin-auth and main-traffic limiter split.
- Baseline degradation/drift observability.

## Remaining Work

1. RL-2: Add multi-window/burst policy primitives.
   - Introduce explicit short-window burst control plus longer-window sustained-rate control.
   - Keep semantics consistent across internal and external limiter modes.

2. RL-8: Add low-rate attack simulation regression coverage.
   - Add deterministic low-and-slow abuse scenarios in the adversarial simulation program.

3. RL-9: Expand operator runbook for threshold tuning and rollback.
   - Provide tuning playbooks by traffic profile, plus explicit rollback trigger thresholds.

4. RL-10: Complete distributed-state SLO and alert posture.
   - Define measurable SLOs for limiter correctness and fallback frequency.
   - Tie rate-limiter and ban-sync lag evidence into promotion gates for enterprise authoritative posture.

## Dependency Linkage

This carry-forward plan is intentionally aligned with:
- [`docs/plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- open backlog slices `DEP-ENT-*` and `SIM-*` in [`todos/todo.md`](../../todos/todo.md)

## Definition of Done

- Multi-window burst + sustained control is implemented and verified.
- Low-rate simulation scenarios run in canonical Makefile verification paths.
- Runbook and SLO/alert criteria are explicit, measurable, and used as enterprise promotion gates.
