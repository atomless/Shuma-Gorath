Date: 2026-03-21
Status: Proposed

Related context:

- [`../research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../research/2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md`](../research/2026-03-21-agent-first-feedback-loop-sequencing-review.md)
- [`../research/2026-03-21-loop-closure-execution-readiness-review.md`](../research/2026-03-21-loop-closure-execution-readiness-review.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](../research/2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md`](2026-03-21-agent-first-loop-structural-decomposition-implementation-plan.md)
- [`2026-03-21-agent-first-loop-truth-completion-implementation-plan.md`](2026-03-21-agent-first-loop-truth-completion-implementation-plan.md)
- [`2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md`](2026-03-21-agent-first-loop-reconcile-and-agent-implementation-plan.md)
- [`2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`2026-03-20-benchmark-suite-v1-design.md`](2026-03-20-benchmark-suite-v1-design.md)
- [`2026-03-20-mature-adversary-sim-evolution-roadmap.md`](2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Close the first real Shuma feedback loop and structurally decompose the control-plane hotspots before additional loop logic lands in already oversized files.

# Core Decisions

1. The highest-priority gap is loop closure, not more feature breadth.
2. Structural decomposition must happen in behavior-preserving slices before more benchmark, operator-snapshot, or oversight logic is added to the current hotspot files.
3. The first controller remains recommend-only. It must not apply or schedule changes until benchmark comparison, objectives, and decision lineage are truthful.
4. Monitoring and Tuning remain projections and control surfaces over machine-first contracts, not separate semantic systems.
5. The first shared-host agent tweaker loop must precede `MON-OVERHAUL-1` so the human surfaces project proven backend semantics rather than invent them ahead of the loop.
6. Replay-promotion lineage must move from sidecar test tooling into backend contracts before later scheduled-agent planning is reopened.
7. The next coding tranche should start from the detailed 2026-03-21 execution-ready implementation plans, not only from this high-level sequencing note.
8. The first truly closed autonomous tuning loop must not use `synthetic_traffic` as tuning evidence; it must depend on protected Scrapling runtime evidence plus replay-promoted or equivalently confirmed frontier or LLM lineage.
9. Monitoring overhaul should follow the first proven closed loop, not merely the first recommend-only loop, so human surfaces reflect the final protected-evidence and rollback semantics.
10. The representativeness contract for Scrapling and frontier or LLM lanes must be judged against Shuma's canonical non-human taxonomy, not lane-local assumptions.
11. Category classification confidence must land before lane representativeness is considered trustworthy enough for autonomous tuning.
12. The taxonomy comes before attackers: Shuma must define the categories it intends to model before it has enough observed adversary traffic to learn them site-locally.
13. The initial taxonomy should stay stable enough for the first closed loop; what should evolve first is the fingerprinting and classification quality within it. Taxonomy expansion is a later contingency only if important non-human traffic persistently falls outside the existing categories.
14. Taxonomy entries must carry stable machine and human-facing metadata because operator objectives and later tuning surfaces will bind posture directly to those categories.
15. The next LLM adversary step should be bounded category-fulfillment modes behind a pluggable containerized backend contract; the full first-class LLM runtime actor remains later.
16. The first genuinely closed loop ends at bounded config tuning and rollback; the later LLM diagnosis harness and later LLM code loop remain downstream phases rather than part of the first closure slice.

# Target Architecture

## 1. Control Contracts

The first real loop should converge on one backend contract chain:

1. persisted `operator_objectives_v1`,
2. materialized `operator_snapshot_v1`,
3. materialized `benchmark_results_v1` with baseline and candidate comparison semantics,
4. durable decision-evidence ledger,
5. bounded `allowed_actions_v1`,
6. replay-promotion lineage contract,
7. recommend-only reconcile engine,
8. first shared-host agent tweaker harness,
9. thin Monitoring and Tuning projections.

## 2. Structural Goal

The control plane should stop concentrating unrelated responsibilities in single files.

The target state is:

1. `src/admin/api.rs` becomes a thin auth/rate-limit/router shell that delegates to domain modules.
2. `src/admin/adversary_sim.rs` becomes focused on shared state and orchestration composition, not every lane/runtime/diagnostic detail.
3. `src/observability/operator_snapshot.rs` becomes a top-level builder over focused objective, runtime-posture, recent-change, and verified-identity summary modules.
4. `src/observability/benchmark_results.rs` becomes a top-level comparator over focused family evaluators and history/comparison helpers.
5. `src/config/controller_action_surface.rs` becomes a thin derived surface over a smaller catalog and policy helper structure.
6. `scripts/tests/adversarial_simulation_runner.py` becomes an orchestrator over focused modules for contract loading, execution, evidence shaping, discovery scoring, and governance/report checks.

# Phase Plan

## Phase 1: Structural Decomposition Prerequisites

These slices are first because the repo should not continue to land control-loop behavior into the current hotspot files.

### `ARCH-API-1`

Split `src/admin/api.rs` into domain-routed modules without changing endpoint contracts.

Acceptance:

1. auth, rate-limit, and top-level routing remain centralized,
2. operator snapshot, benchmark, monitoring, config, adversary-sim, and diagnostics handlers each move behind dedicated modules,
3. endpoint behavior and focused `make` verification remain unchanged.

### `ARCH-OBS-1`

Split operator snapshot and benchmark materialization into focused modules before more loop semantics land there.

Acceptance:

1. objective-profile helpers, recent-change shaping, verified-identity summary shaping, benchmark family evaluators, and history/comparison helpers each have focused homes,
2. the public contract shape remains unchanged until the later semantic tranche,
3. the top-level orchestrator files stop growing as the next features land.

### `ARCH-SIM-1`

Split `src/admin/adversary_sim.rs` into control-state, lane-runtime, diagnostics, and corpus/worker-plan helpers before reconcile integration work.

Acceptance:

1. adversary control-state transitions remain intact,
2. Scrapling and deterministic lane logic stay behavior-identical,
3. bot-red-team placeholder behavior remains explicit rather than hidden.

### `ADV-RUN-ARCH-1`

Execute the existing adversarial runner refactor as part of this phase, not as later platform cleanup.

Acceptance:

1. contract loading, execution, evidence shaping, discovery scoring, and governance/report logic stop cohabiting one 6k+ line file,
2. promotion and frontier lineage semantics remain unchanged,
3. the runner becomes safe to integrate with later backend promotion work.

## Phase 2: Loop Truth Completion

These slices complete the missing truth the controller needs.

### `OPS-BENCH-2`

Materialize real benchmark history and comparator semantics.

Must include:

1. prior-window or explicit baseline persistence,
2. `improvement_status`,
3. representative adversary scenario-family results,
4. beneficial non-human posture metrics,
5. verified-identity-aware capability gating,
6. explicit candidate-vs-current comparison support for later tuning or code-evolution loops.

### `OPS-SNAPSHOT-2`

Replace backend-default and placeholder operator state with typed site-owned contract surfaces.

Must include:

1. persisted writable `operator_objectives_v1`,
2. objective revision/reference in the snapshot,
3. typed verified-identity summary instead of placeholder text,
4. causal decision/watch-window evidence rather than only recent-change summaries,
5. durable evidence references needed for later reconcile and rollback reasoning.

### `ADV-PROMO-1`

Promote emergent finding and deterministic replay lineage into backend contracts.

Must include:

1. typed replay-candidate and promotion-lineage contract,
2. integration point from current promotion tooling into backend-readable state,
3. snapshot or benchmark visibility for promoted or review-pending replay candidates,
4. no uncontrolled mutation of the deterministic corpus.

## Phase 3: Recommend-Only Reconcile Loop

### `OVR-RECON-1`

Land the first backend recommend-only reconciler using the now-truthful contracts.

Must include:

1. pure reconcile engine,
2. patch policy against `allowed_actions_v1`,
3. typed proposal output,
4. decision ledger persistence,
5. adversary-verification requirement for guarded families,
6. explicit fail-closed behavior when evidence is stale, degraded, or contradictory.

This phase must reuse:

1. the existing adversary-sim lease/idempotency pattern,
2. existing config validation seams,
3. and the machine-first snapshot and benchmark contracts.

## Phase 4: First Machine-First Agent Tweaker Loop

### `OVR-AGENT-1`

Land the first shared-host agent tweaker loop over the truthful backend contracts before Monitoring or Tuning projection work.

Must include:

1. one agent invocation path that calls the same internal reconcile contract whether triggered periodically or immediately after a qualifying adversary-sim run,
2. consumption of `operator_snapshot_v1`, `benchmark_results_v1`, replay-promotion lineage, and recent decision evidence,
3. typed recommend-only proposal outputs and durable evidence references rather than prose-only diagnostics,
4. explicit `no_change`, `insufficient_evidence`, `rerun_sim_required`, and equivalent fail-closed outcomes when signal is stale, contradictory, or incomplete,
5. shared-host control-plane execution only, never request-path or edge-gateway execution.

This phase must prove:

1. the backend loop can read sim-cost and benchmark deltas end to end,
2. proposal families and evidence semantics are now real enough for human projection,
3. and later Monitoring/Tuning work can be derived from demonstrated backend behavior instead of speculative UI-first modeling.

## Phase 5: Protected Tuning Evidence And Closed-Loop Safety

Execution-ready plan chain:

1. [`2026-03-22-taxonomy-and-classification-implementation-plan.md`](2026-03-22-taxonomy-and-classification-implementation-plan.md)
2. [`2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md)
3. [`2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md`](2026-03-22-category-aware-objectives-benchmarks-and-apply-loop-implementation-plan.md)

### `TRAFFIC-TAX-1`

Define the canonical non-human traffic taxonomy that later tuning and lane-representativeness work will use.

### `TRAFFIC-TAX-2`

Materialize bounded category-confidence and evidence receipts so Shuma can tell when both simulated and observed traffic categorization are trustworthy enough to use in tuning decisions.

### `SIM-LLM-FIT-1`

Implement the minimum bounded LLM-backed browser or request modes needed for category fulfillment behind a pluggable model-backend contract, with frontier-backed execution as the initial reference path for the highest-capability categories and optional later local-model backends only if evals prove parity.

### `SIM-FULFILL-1`

Implement the category-to-lane fulfillment matrix across Scrapling and frontier or containerized LLM modes before claiming lane representativeness.

### `SIM-PROTECTED-1`

Codify protected tuning evidence eligibility and explicitly exclude `synthetic_traffic` from any future auto-apply evidence basis.

### `SIM-COVER-1`

Define the representativeness matrix and bounded coverage receipts across Scrapling runtime traffic and replay-promoted frontier or LLM lineage for the non-human categories Shuma intends to optimize over, using the canonical taxonomy rather than lane-local labels.

### `OPS-OBJECTIVES-3`

Extend `operator_objectives_v1` with category-aware non-human intent so the controller can distinguish `allowed`, `tolerated`, `cost_reduced`, `restricted`, and `blocked` posture by category.

### `OPS-BENCH-3`

Extend `benchmark_results_v1` with protected-lane eligibility and category-aware comparison semantics suitable for canary apply and rollback.

### `OVR-APPLY-1`

Only after the above gates are real, add the first bounded canary apply, watch-window, compare, and rollback loop.

Status update (2026-03-22): complete and live-proven on shared-host per [`../research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md`](../research/2026-03-22-ovr-apply-1-canary-apply-and-rollback-post-implementation-review.md).

### `ADV-DIAG-1`

Before Monitoring and Tuning are reopened, reconcile adversary-sim status diagnostics with the persisted event telemetry that the closed loop now correctly treats as authoritative. The live `OVR-APPLY-1` proof showed that `sim_run_id` event evidence can be truthful while shared-host generation counters remain zero.

Execution reference: [`2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md`](2026-03-23-adv-diag-1-adversary-sim-status-truth-implementation-plan.md)

Status update (2026-03-23): complete per [`../research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](../research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md).

## Phase 6: Human Operator Projection

### `MON-OVERHAUL-1`

Rebuild Monitoring as the thin human projection over the machine-first contracts after the backend truth, first working closed loop, and `ADV-DIAG-1` diagnostics-truth follow-up are complete.

### `TUNE-SURFACE-1`

Finish the operator control surface once the controller inputs, safe action families, first working agent loop semantics, and adversary-sim diagnostics truth are all aligned, including per-category posture controls over the stable operator-facing taxonomy.

These phases should not be started early just because the UI can be edited sooner.

## Phase 7: Later Scheduled-Agent And Code-Evolution Loops

These later items remain intentionally non-execution-ready until the three Phase 5 implementation plans above are complete and live-proved.

### `OVR-AGENT-2`

Reopen the later LLM-backed diagnosis/config harness only after the first shared-host agent loop, the first closed config loop, Monitoring projection, Tuning surface, replay-promotion contract, and central-intelligence architecture all exist.

### `OVR-CODE-1`

Keep the later benchmark-driven LLM code-evolution or PR-generation path behind the bounded config loop, the later diagnosis harness, and benchmark-comparison proof.

# Scheduling Rules

1. Execute the phases in order.
2. Do not blend structural decomposition and semantic expansion in the same tranche.
3. Keep the first decomposition slices behavior-preserving and test-focused.
4. Keep one hotspot file as the primary target per refactor tranche wherever practical.
5. Do not reopen `MON-OVERHAUL-1`, `TUNE-SURFACE-1`, or `OVR-AGENT-2` until the blockers listed in this plan are satisfied, including protected tuning evidence and category-coverage proof.
6. Treat periodic scheduling and post-sim triggering as adapter paths over one reconcile or agent contract, not as separate controller implementations.

# File-Length And Separation Guardrails

These are review heuristics, not the product goal themselves:

1. `src/admin/api.rs` should trend toward a thin router shell rather than continue as the home of endpoint implementations.
2. `scripts/tests/adversarial_simulation_runner.py` should become a driver/orchestrator rather than a repository of unrelated execution and governance logic.
3. `src/admin/adversary_sim.rs` should stop co-locating control-state machinery with lane-specific runtime details.
4. `src/observability/operator_snapshot.rs` and `src/observability/benchmark_results.rs` should stop being the only homes for every summary/comparator concern.
5. Any new loop feature that would grow one of these hotspot files without first attempting extraction should be treated as a planning failure.

# Exit Criteria

This plan is satisfied when:

1. the hotspot decompositions are complete enough that new loop work no longer lands into monolithic files by default,
2. the benchmark contract can express improvement or regression against a real baseline,
3. the operator snapshot contains persisted objectives, typed verified-identity summary, and causal decision evidence,
4. the recommend-only reconcile engine exists as a backend contract,
5. the first shared-host agent tweaker loop exists and can exercise the backend contracts against sim-cost and benchmark feedback,
6. replay-promotion lineage is part of the backend control plane rather than sidecar-only tooling,
7. Monitoring and Tuning consume those contracts rather than parallel semantics,
8. the first autonomous tuning loop is blocked until protected evidence and category-aware objective gates are delivered,
9. Monitoring and Tuning consume the proven closed-loop semantics rather than the earlier recommend-only subset,
10. and only then the later scheduled-agent and code-evolution planning can resume.
