# Benchmark Escalation Boundary Post-Implementation Review

Date: 2026-03-20
Status: Complete

Related context:

- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-20-benchmark-suite-v1-implementation-plan.md`](../plans/2026-03-20-benchmark-suite-v1-implementation-plan.md)
- [`2026-03-20-benchmark-results-contract-post-implementation-review.md`](./2026-03-20-benchmark-results-contract-post-implementation-review.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)

## Review Scope

Review the first explicit benchmark escalation boundary against:

1. the benchmark-suite design,
2. the machine-first feedback-loop direction,
3. the requirement that Shuma separate config-tuning candidates from true code-evolution candidates,
4. and the requirement that the boundary remain bounded, review-aware, and driven by existing backend contracts rather than by dashboard-only heuristics.

## Delivered In This Slice

1. Extended `benchmark_results_v1.escalation_hint` with explicit machine-facing fields for:
   - `availability`
   - `decision`
   - `review_status`
   - `trigger_family_ids`
   - `candidate_action_families`
   - `blockers`
   - `note`
2. Derived `candidate_action_families` from `allowed_actions_v1` family metadata rather than from UI-local logic.
3. Materialized the first explicit decision boundary between:
   - `config_tuning_candidate`
   - `observe_longer`
   - `code_evolution_candidate`
4. Added focused proof that:
   - healthy or empty current-instance state remains `observe_longer`,
   - a supported outside-budget miss becomes `config_tuning_candidate`,
   - and an outside-budget miss with no matching config surface becomes `code_evolution_candidate`.

## Comparison Against Intent

### What matches the plan well

1. The boundary is now machine-readable instead of being implicit in prose.
2. The result remains review-aware because `manual_review_required` stays explicit.
3. The logic is derived from existing backend contracts:
   - benchmark family results,
   - current action-surface families,
   - and current watch-window evidence.
4. The first slice stays conservative by marking the escalation contract as `partial_support` while baseline history is not yet materialized.

### What remains intentionally deferred

1. Escalation still uses current-window evidence only; it does not yet compare repeated windows or stored baselines.
2. The adversary-sim and beneficial non-human benchmark families remain capability-gated and therefore are not yet rich escalation inputs.
3. Monitoring still does not project `benchmark_results_v1` or the escalation boundary.
4. Fleet or central-intelligence enrichment remains out of scope for this slice.

## Architecture Review

The slice is on the right bearing.

Why:

1. It avoids inventing a second decision model by reusing `allowed_actions_v1` as the config-surface source of truth.
2. It keeps code-evolution escalation narrow and explicit instead of treating every bad metric as justification for changing the repository.
3. It preserves the later project-evolution loop's need for explicit benchmark evidence while still giving the current system a useful boundary now.

## Shortfalls Found

No new architectural blocker was found in this slice.

The next remaining benchmark step is the expected one:

1. project `benchmark_suite_v1` and `benchmark_results_v1` into `operator_snapshot_v1` and later Monitoring without creating a second semantic model.

## Conclusion

This slice meets the intended purpose:

1. Shuma now has an explicit benchmark-driven boundary between config tuning, continued observation, and later code-evolution review,
2. the active backlog can stop describing the escalation boundary as unfinished work,
3. and the next optimal step is benchmark projection into snapshot and Monitoring, not a return to benchmark decision semantics.
