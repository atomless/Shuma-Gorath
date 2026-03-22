Date: 2026-03-22
Status: Proposed

Related context:

- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`../plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](../plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Goal

Make the active closed-loop sequence implementation-ready end to end instead of leaving it split across roadmap prose, design guardrails, and partially specified bridge notes.

# Findings

## 1. The sequence is correct, but most tasks are still specified at contract level

The active path is now the right one:

1. `TRAFFIC-TAX-1`
2. `TRAFFIC-TAX-2`
3. `SIM-LLM-FIT-1`
4. `SIM-FULFILL-1`
5. `SIM-COVER-1`
6. `SIM-PROTECTED-1`
7. `OPS-OBJECTIVES-3`
8. `OPS-BENCH-3`
9. `OVR-APPLY-1`
10. later `OVR-AGENT-2`
11. later `OVR-CODE-1`

But the current March 22 plans still mostly describe what the system must mean, not exactly which modules and focused verification targets must be changed for each tranche.

## 2. The most under-specified execution gaps are `SIM-LLM-FIT-1` and `OVR-APPLY-1`

Those two items still lack the crispest implementation map:

1. `SIM-LLM-FIT-1` still needs an exact actor boundary, capability/tool contract, backend abstraction, and receipt model.
2. `OVR-APPLY-1` still needs the explicit bounded controller state machine, canary lineage, watch-window comparison contract, and rollback rules.

## 3. The repo already has the core homes needed for execution

The existing codebase now has the right dominant seams to extend rather than replace:

1. taxonomy/classification can extend `src/runtime/traffic_classification.rs`, `src/observability/operator_snapshot.rs`, and `src/observability/benchmark_results.rs`,
2. lane fulfillment and coverage can extend `src/admin/adversary_sim_*`, `scripts/tests/adversarial_runner/*`, and `src/observability/replay_promotion.rs`,
3. category-aware objectives and first auto-apply can extend `src/observability/operator_snapshot_objectives.rs`, `src/admin/operator_objectives_api.rs`, `src/admin/oversight_api.rs`, `src/admin/oversight_reconcile.rs`, and a new `src/admin/oversight_apply.rs`.

## 4. Later loops should remain later

`OVR-AGENT-2` and `OVR-CODE-1` should remain blocked even after the new implementation-ready planning pass. They depend on the first closed config loop being real, live-proved, and already projected through truthful operator contracts.

# Decision

Split the execution-ready work into three implementation plans:

1. taxonomy and classification,
2. bounded LLM fulfillment, coverage, and protected evidence,
3. category-aware objectives, benchmark gating, and first canary apply and rollback.

Keep the existing bridge plan as the high-level path, but make these three new plan documents the coding source of truth for the next execution sequence.

# Exit Criteria

This readiness review is complete when:

1. every active tranche through `OVR-APPLY-1` points to an execution-ready plan with exact files and verification,
2. `OVR-AGENT-2` and `OVR-CODE-1` remain explicitly later-blocked,
3. and the roadmap and backlog reference the new plan chain rather than only the earlier high-level bridge prose.
