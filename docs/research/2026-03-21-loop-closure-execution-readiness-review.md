Date: 2026-03-21
Status: Execution-readiness review

Related context:

- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md)
- [`../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](../plans/2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md)
- [`../plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../plans/2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-21-agent-first-feedback-loop-sequencing-review.md`](./2026-03-21-agent-first-feedback-loop-sequencing-review.md)
- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](./2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Question Reviewed

What still needs to be in place before Shuma can responsibly start the active execution queue:

1. `ARCH-API-1`
2. `ARCH-OBS-1`
3. `ARCH-SIM-1`
4. `ADV-RUN-ARCH-1`
5. `OPS-BENCH-2`
6. `OPS-SNAPSHOT-2`
7. `ADV-PROMO-1`
8. `OVR-RECON-1`
9. `OVR-AGENT-1`

# Decision Summary

1. No new product capability is required before this queue starts.
2. One short planning-and-verification readiness tranche is required before implementation starts.
3. The queue is architecturally correct but not yet execution-ready enough for hotspot refactors and controller work without avoidable churn.
4. The missing prerequisite is not more design direction. It is detailed tranche plans plus truthful focused verification paths.
5. After that readiness tranche, `ARCH-API-1` becomes the correct first implementation slice.

# Findings

## 1. The queue is sequenced, but still too coarse for safe execution

The active order in [`../../todos/todo.md`](../../todos/todo.md) and the high-level closure plan in [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md) are now correct.

But each item is still one roadmap-sized statement rather than an implementation-ready tranche plan with:

1. exact extraction seams,
2. exact future file homes,
3. smallest truthful `make` verification paths,
4. and explicit behavior-preserving versus semantic-expansion boundaries.

That is tolerable for a roadmap. It is not yet good enough for large-file decomposition work in the current hotspot files.

## 2. The repo already has useful focused verification near these areas

The current `Makefile` already provides nearby verification anchors:

1. `test-runtime-preflight-unit`
2. `test-telemetry-hot-read-contract`
3. `test-telemetry-hot-read-projection`
4. `test-operator-snapshot-foundation`
5. `test-benchmark-results-contract`
6. `test-adversarial-python-unit`
7. `test-adversarial-lane-contract`
8. `test-adversary-sim-lifecycle`
9. `test-adversary-sim-runtime-surface`

That is enough to avoid a blank page.

It is not enough to prove the structural refactors cleanly, because some hotspot files still lack tranche-specific gates that are named after the exact contract being preserved.

## 3. The codebase already exposes the reuse seams the next work should honor

The readiness scan found important existing seams that the implementation must reuse:

1. `src/admin/adversary_sim_control.rs` already models lease, idempotency, and audit semantics that `OVR-RECON-1` and `OVR-AGENT-1` should reuse.
2. `src/observability/hot_read_projection.rs` already materializes `operator_snapshot_v1` and nested `benchmark_results_v1`; later work should extend that projection chain rather than invent a parallel read model.
3. `src/admin/api.rs` already owns the bounded `recent_changes` ledger helpers and tests; `OPS-SNAPSHOT-2` should build on that seam, then extract it, rather than replace it.
4. `src/config/controller_action_surface.rs` already exposes `allowed_actions_v1`; reconcile and agent work should consume that exact surface.
5. `scripts/tests/adversarial_simulation_runner.py` and `scripts/tests/adversarial_promote_candidates.py` already hold replay-candidate and promotion-lineage semantics; `ADV-PROMO-1` should promote those semantics into backend contracts rather than inventing a second governance model.

## 4. The real prerequisite set is therefore small and concrete

Before implementation starts, the repo should have:

1. an exact structural decomposition plan for `ARCH-API-1`, `ARCH-OBS-1`, `ARCH-SIM-1`, and `ADV-RUN-ARCH-1`,
2. an exact contract-completion plan for `OPS-BENCH-2`, `OPS-SNAPSHOT-2`, and `ADV-PROMO-1`,
3. an exact reconcile-and-agent plan for `OVR-RECON-1` and `OVR-AGENT-1`,
4. and a named focused-`make` strategy for each phase before code moves.

Nothing else should block the queue.

# Recommended Prerequisite Tranche

The minimal prerequisite tranche should be docs-only and should deliver:

1. one readiness review capturing the true prerequisites,
2. one detailed implementation plan for structural decomposition,
3. one detailed implementation plan for loop-truth completion,
4. one detailed implementation plan for reconcile and first shared-host agent work,
5. and backlog or roadmap links to those plans so the next coding tranche starts from the settled execution chain.

# Conclusion

Shuma does not need another round of broad architectural ideation before this queue starts.

It needs the execution-ready planning layer that sits between roadmap and code:

1. exact file moves,
2. exact contract additions,
3. exact reuse seams,
4. and truthful focused verification.

Once that readiness tranche exists, `ARCH-API-1` should start immediately.
