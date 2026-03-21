Date: 2026-03-21
Status: Proposed

Related context:

- [`../research/2026-03-21-feedback-loop-and-architecture-debt-review.md`](../research/2026-03-21-feedback-loop-and-architecture-debt-review.md)
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
5. Replay-promotion lineage must move from sidecar test tooling into backend contracts before later scheduled-agent planning is reopened.

# Target Architecture

## 1. Control Contracts

The first real loop should converge on one backend contract chain:

1. persisted `operator_objectives_v1`,
2. materialized `operator_snapshot_v1`,
3. materialized `benchmark_results_v1` with baseline and candidate comparison semantics,
4. durable decision-evidence ledger,
5. bounded `allowed_actions_v1`,
6. recommend-only reconcile engine,
7. replay-promotion lineage contract,
8. thin Monitoring and Tuning projections.

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

## Phase 4: Human Operator Projection

### `MON-OVERHAUL-1`

Rebuild Monitoring as the thin human projection over the machine-first contracts after the backend truth is complete.

### `TUNE-SURFACE-1`

Finish the operator control surface once the controller inputs and safe action families are truthful.

These phases should not be started early just because the UI can be edited sooner.

## Phase 5: Later Scheduled-Agent And Code-Evolution Loops

### `OVR-AGENT-2`

Reopen scheduled analyzer/recommender planning only after the recommend-only reconciler, Monitoring projection, Tuning surface, and replay-promotion contract all exist.

### `OVR-CODE-1`

Keep code-evolution or PR-generation planning behind the bounded config loop and benchmark-comparison proof.

# Scheduling Rules

1. Execute the phases in order.
2. Do not blend structural decomposition and semantic expansion in the same tranche.
3. Keep the first decomposition slices behavior-preserving and test-focused.
4. Keep one hotspot file as the primary target per refactor tranche wherever practical.
5. Do not reopen `MON-OVERHAUL-1`, `TUNE-SURFACE-1`, or `OVR-AGENT-2` until the blockers listed in this plan are satisfied.

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
5. replay-promotion lineage is part of the backend control plane rather than sidecar-only tooling,
6. Monitoring and Tuning consume those contracts rather than parallel semantics,
7. and only then the later scheduled-agent and code-evolution planning can resume.
