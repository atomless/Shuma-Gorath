Date: 2026-03-21
Status: Architecture review

Related context:

- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md)
- [`../plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../plans/2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Question Reviewed

Where has Shuma accumulated technical debt or architectural drift relative to the intended closed loop between:

1. adversary simulation or real adversary traffic,
2. periodic machine analysis,
3. bounded config tuning,
4. adversary re-exercise,
5. and benchmarked judgment of whether defender cost and human friction improved?

# Decision Summary

1. The repo is not directionless. The machine-first snapshot, bounded benchmark shell, shared-host adversary direction, and conservative action surface are all on the right bearing.
2. The main shortfall is that the loop is not yet closed. Observation and adversary generation matured faster than objective materialization, baseline comparison, causal attribution, and the recommend-only reconciler.
3. The highest-priority debt is not new feature breadth. It is completing the first real closed loop and decomposing oversized control-plane modules before more logic is added to them.
4. The biggest structural debt is concentration of unrelated concerns in a handful of files, especially `src/admin/api.rs`, `src/admin/adversary_sim.rs`, `src/observability/operator_snapshot.rs`, `src/observability/benchmark_results.rs`, `src/config/controller_action_surface.rs`, and `scripts/tests/adversarial_simulation_runner.py`.
5. The correct next move is a phased sequence: behavior-preserving architectural decomposition first, then benchmark/objective/decision-evidence completion, then the recommend-only reconciler, then Monitoring/Tuning projection, then later scheduled agent and code-evolution loops.

# Repo Review

## What Is Already Aligned

Several important parts of the repository already match the intended feedback-loop architecture:

1. `operator_snapshot_v1`, `benchmark_suite_v1`, `benchmark_results_v1`, and `allowed_actions_v1` exist as bounded machine-first contracts rather than dashboard-only semantics.
2. The mature adversary roadmap already treats deterministic traffic as oracle/comparator/memory and emergent lanes as the primary adaptive discovery inputs.
3. The current adversary control plane already has a useful lease/idempotency pattern that later oversight work should reuse rather than replace.
4. The shared-host direction is now explicit, which keeps the future diagnose/tune/apply loop off the edge request path.

## Findings

### 1. Benchmark comparison truth is still incomplete

Shuma still cannot answer the key loop question: "did the last change improve things against a stable baseline?"

Why:

1. `benchmark_results_v1` still reports `baseline_reference.status = not_available`.
2. `benchmark_results_v1` still reports `improvement_status = not_available`.
3. `representative_adversary_effectiveness` remains `not_yet_supported`.
4. `beneficial_non_human_posture` remains `not_yet_supported`.

Consequence:

1. Shuma can describe current budget distance.
2. Shuma cannot yet support a trustworthy tune-confirm-repeat comparator.
3. Verified-identity policy is now ahead of its benchmark home.

### 2. The desired-state side of the loop is still partially synthetic

The loop wants persisted operator objectives and explicit site-owned posture. The current snapshot still relies on backend defaults and placeholders.

Why:

1. `operator_snapshot_v1` still injects `default_operator_objectives()`.
2. adversary-sim expectations are still placeholder text.
3. verified-identity snapshot content is still placeholder text.

Consequence:

1. A future controller does not yet have a persisted local utility function to optimize.
2. Site-specific objective variance remains outside the main backend contract.
3. The verified-identity tranche landed runtime power before corresponding operator-loop truth.

### 3. The recommend-only reconciler is still plan-only

The repo has the right oversight design notes, but not the implementation that closes the loop.

Why:

1. the plan calls for `oversight_reconcile`, `oversight_patch_policy`, `oversight_apply`, and scheduler adapters,
2. but those modules do not yet exist in `src/`.

Consequence:

1. Shuma has read models and adversary control, but not the bounded diagnose-plan-act-watch-rollback engine that ties them together.
2. `OVR-AGENT-2` remains correctly blocked, but the blocker is now more specific than "later controller work."

### 4. Change attribution is not yet strong enough for tune-confirm-repeat

The current snapshot has bounded `recent_changes`, but not the stronger evidence chain the control loop needs.

Why:

1. `recent_changes` tells Shuma that a change happened, when, and in what family.
2. The oversight design expects a decision ledger with expected impact, required verification, rollback window, and durable evidence references.
3. Those fields currently exist only in plan docs, not in `src/`.

Consequence:

1. Operators and future controller logic cannot yet reconstruct "this evidence justified this patch and this watch window confirmed or rejected it."
2. Rollback logic would be forced to infer more than it should.

### 5. Discovery-to-memory promotion is implemented, but off to the side

The replay-promotion path exists as Python governance tooling and JSON artifacts, not yet as a first-class backend contract.

Why:

1. candidate and rejection lineage live in `scripts/tests/adversarial_simulation_runner.py`,
2. promotion lineage and reproducibility gating live in `scripts/tests/adversarial_promote_candidates.py`,
3. but that lineage is not yet projected into `operator_snapshot_v1` or `benchmark_results_v1`.

Consequence:

1. emergent finding to deterministic memory is real,
2. but it remains adjacent to the operator loop rather than integrated into its core contracts.

### 6. Monitoring still bypasses the machine-first contracts

The dashboard still primarily consumes legacy monitoring endpoints rather than the new snapshot and benchmark contracts.

Consequence:

1. the human operator surface and future machine controller surface are still split,
2. which risks semantic drift and duplicated interpretation logic,
3. even though the repo has already decided Monitoring should become a thin projection over machine-first contracts.

### 7. Large-file concentration is now architecture debt, not just style debt

The file sizes are now significant enough to indicate poor separation of concerns:

1. `src/admin/api.rs`: 19,675 lines
2. `scripts/tests/adversarial_simulation_runner.py`: 6,950 lines
3. `src/admin/adversary_sim.rs`: 2,637 lines
4. `src/config/controller_action_surface.rs`: 952 lines
5. `src/observability/operator_snapshot.rs`: 922 lines
6. `src/observability/benchmark_results.rs`: 772 lines

Consequence:

1. The next loop-completion work will otherwise keep landing into already-overloaded modules.
2. This makes review harder, reuse weaker, and future controller work riskier.
3. The repo's own research direction prefers many narrow controllers and bounded contracts rather than control-plane monoliths.

### 8. The mature four-role adversary model is not yet fully represented

The repo now has deterministic and Scrapling roles in code, but the later higher-capability adversary role is still stubbed.

Consequence:

1. this does not block the first closed loop,
2. but it confirms that the future mature loop should continue to prioritize Scrapling and benchmark truth before frontier-agent expansion.

# Architecture Consequences

The right next shape for Shuma is:

1. persisted `operator_objectives_v1`,
2. bounded `operator_snapshot_v1`,
3. materialized `benchmark_results_v1` with real history and comparison semantics,
4. durable decision-evidence ledger,
5. recommend-only reconcile engine over `allowed_actions_v1`,
6. replay-promotion lineage as a backend contract,
7. Monitoring and Tuning as thin projections over those same contracts,
8. and only then the later scheduled agent and code-evolution loops.

That means the repo should not treat this review as a cue to do one giant rewrite.

Instead it should:

1. decompose the oversized modules in behavior-preserving slices first,
2. land the missing loop-truth contracts on top of those cleaner seams,
3. and delay scheduled-agent work until the loop can actually prove improvement or regression.

# Backlog Consequences

This review implies one immediate planning and backlog update:

1. add a dedicated feedback-loop closure and structural-restructuring plan,
2. add active TODO items for benchmark history, writable objectives, decision lineage, recommend-only reconcile, replay-promotion integration, and architectural decomposition,
3. keep `MON-OVERHAUL-1`, `TUNE-SURFACE-1`, and `OVR-AGENT-2` blocked until those prerequisites are explicitly met,
4. and pull the large-file refactors onto the active path instead of leaving them as vague future cleanup.

# Conclusion

The main technical debt is not random sprawl.

It is concentrated in two related problems:

1. the first real feedback loop is still missing its comparison, objective, attribution, and reconcile joints,
2. and the control-plane modules that should host that loop have become too large to keep absorbing unrelated behavior cleanly.

The correct response is therefore:

1. close the loop,
2. decompose the hotspots,
3. and only then proceed to operator-surface and scheduled-agent work.
