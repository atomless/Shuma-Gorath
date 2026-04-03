Date: 2026-04-03
Status: In Progress

Related context:

- [`../research/2026-04-03-adversary-sim-telemetry-unification-architecture-review.md`](../research/2026-04-03-adversary-sim-telemetry-unification-architecture-review.md)
- [`../research/2026-04-02-agentic-recent-run-coverage-gap-review.md`](../research/2026-04-02-agentic-recent-run-coverage-gap-review.md)
- [`2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](./2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)

# Adversary Sim Telemetry Unification Plan

## Goal

Remove the current split where recent-run and Red Team surfaces mix:

1. shared observed monitoring truth,
2. Scrapling-specific surface receipts,
3. Agentic runtime receipts,
4. and simulator-only realism sidecars

inside the same operator-facing traffic columns.

The end state must be:

1. one shared Shuma telemetry spine for traffic events, surfaces reached, defence interactions, and outcomes across real and sim traffic,
2. origin-aware filtering at read time rather than lane-specific collection stacks,
3. and simulator receipts retained only for privileged attacker-internal metadata.

## Architecture Contract

### Shared observed telemetry owns

1. request/outcome truth,
2. defence-contact truth,
3. observed surface contact and progression,
4. monitoring event counts,
5. defence delta counts,
6. and later real-vs-sim comparable penetration/outcome summaries.

### Simulator sidecars own

1. fulfillment modes,
2. category targets and fulfilled categories,
3. frontier provider/model lineage,
4. generation source and fallback reason,
5. realism-envelope provenance,
6. and worker-internal targeting strategy or planning detail.

### Presentation rule

`Recent Red Team Runs`, operator snapshot, and downstream Game Loop surfaces may render both evidence classes, but they must not present simulator sidecars as if they were shared observed traffic truth.

## Important Scope Constraint

Shared collection does not imply shared denominators.

This plan does not require:

1. live human-friction or clean-traffic summaries to include sim-origin requests,
2. or every monitoring surface to blend real and sim traffic together.

It does require:

1. one shared collection path,
2. sim attribution on that path,
3. and origin-aware filtering when summaries need real-only or sim-only views.

## Implementation Tranches

### Tranche 1: Freeze the shared observed-surface contract

Purpose:

1. define the canonical observed surface/outcome evidence Shuma should collect from any inbound traffic,
2. map the existing sim-only receipt fields into either shared-observable truth or privileged sidecars,
3. and make exactness explicit before any runtime edits.

Expected code/doc targets:

- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../docs/observability.md`](../../docs/observability.md)

Acceptance criteria:

1. one canonical list exists for which recent-run columns must be derived from shared observed telemetry,
2. one canonical list exists for which columns remain simulator-sidecar truth,
3. and any temporary best-effort fallbacks are explicitly named as such.

Proof surface:

1. research/plan docs,
2. hot-read contract definitions,
3. and focused unit contracts for new summary types or exactness flags.

Verification path:

- `make test-monitoring-telemetry-foundation-unit`

### Tranche 2: Route adversary-sim traffic through the same observed request telemetry path

Purpose:

1. ensure sim-origin requests emit the same shared request/outcome and defence-contact evidence as real traffic,
2. while preserving origin tags so later summaries can filter them cleanly.

Likely code targets:

- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)

Acceptance criteria:

1. sim allow/challenge/block/tarpit/maze/follow-up interactions materialize through the same observed request telemetry path as real traffic,
2. any intentionally excluded summaries are filtered after collection rather than dropped by separate sim-only collection logic,
3. and sim-origin tags remain available for read-time scoping.

Proof surface:

1. bounded runtime unit/integration tests,
2. recent event materialization with sim tags,
3. and admin read payload truth.

Verification path:

- `make test-adversary-sim-shared-observed-telemetry-contract`

### Tranche 3: Rebuild recent-run surface and outcome summaries on shared observed evidence

Purpose:

1. make `Coverage`, `Monitoring Deltas`, defence penetrations, and related recent-run summaries derive from one evidence class,
2. stop using receipt-backed coverage as a substitute for shared observed traffic evidence,
3. and preserve simulator sidecars only as enrichment.

Likely code targets:

- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte`](../../dashboard/src/lib/components/dashboard/monitoring/AdversaryRunPanel.svelte)

Acceptance criteria:

1. `Coverage` for both Scrapling and Agentic rows is derived from shared observed traffic evidence rather than worker receipts,
2. `Monitoring Deltas` and surface/outcome summaries no longer mix receipt-backed claims with zero shared observed deltas,
3. sim-only receipt data remains available for modes/categories/execution lineage,
4. and any remaining temporary fallback is explicitly exactness-labeled rather than silently blended.

Proof surface:

1. backend recent-run aggregation tests,
2. admin monitoring payload shape,
3. dashboard row shaping,
4. and rendered Red Team DOM proof.

Verification path:

- `make test-adversary-sim-recent-run-observed-coverage`
- `make test-dashboard-red-team-pane`
- `make test-code-quality`

### Tranche 4: Reduce simulator receipts to privileged sidecars

Purpose:

1. cleanly separate shared observed traffic truth from simulator-internal facts,
2. and prevent later Game Loop or operator work from reintroducing lane-specific traffic surrogates.

Likely code targets:

- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../docs/dashboard-tabs/red-team.md`](../../docs/dashboard-tabs/red-team.md)

Acceptance criteria:

1. recent-run and operator-snapshot contracts clearly distinguish shared observed evidence from simulator sidecars,
2. simulator sidecars remain available for representativeness and classifier-calibration work,
3. and the repo no longer depends on lane-specific receipt coverage summaries to claim observed penetration truth.

Proof surface:

1. operator snapshot payloads,
2. dashboard docs/contracts,
3. and focused backend/dashboard tests around sidecar separation.

Verification path:

- `make test-adversary-sim-sidecar-separation-contract`
- `make test-dashboard-red-team-pane`

### Tranche 5: Fix frontier provider env parity for host-side Agentic workers

Purpose:

1. make local frontier availability truthful across the Spin runtime and the host-side supervisor workers,
2. so Agentic execution lineage and recent-run execution truth stop degrading to `no_configured_frontier_provider` on correctly configured contributor machines.

Likely code targets:

- [`../../scripts/run_with_adversary_sim_supervisor.sh`](../../scripts/run_with_adversary_sim_supervisor.sh)
- [`../../scripts/adversary_sim_supervisor_launch.sh`](../../scripts/adversary_sim_supervisor_launch.sh)
- [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs)
- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
- [`../../Makefile`](../../Makefile)

Acceptance criteria:

1. when local env or Make config exposes frontier provider keys, the host-side LLM runtime worker sees the same configured providers as the Spin runtime,
2. local Agentic rows no longer report `no_configured_frontier_provider` under a configured frontier setup,
3. and verification proves both the env propagation seam and the rendered operator truth.

Proof surface:

1. focused supervisor/worker env tests,
2. live lane dispatch contract,
3. and rendered Red Team execution copy.

Verification path:

- `make test-adversarial-llm-runtime-frontier-env-parity`
- `make test-dashboard-red-team-pane`
- `make test-code-quality`

## Sequencing

Execute the tranches in this order:

1. freeze the shared observed-vs-sidecar contract,
2. restore shared collection parity for sim-origin requests,
3. rebuild recent-run summaries on that shared evidence,
4. reduce receipts to privileged sidecars,
5. fix frontier env parity for host-side Agentic workers.

Reason:

1. without the contract freeze, the repo risks more local table patches,
2. without shared collection parity, recent-run unification still lacks one consistent evidence source,
3. and without frontier env parity, Agentic execution truth remains degraded even after the telemetry model is corrected.

## Explicit Supersession Note

This plan supersedes the narrow direction in [`2026-04-02-agentic-recent-run-coverage-plan.md`](./2026-04-02-agentic-recent-run-coverage-plan.md) where Agentic `Coverage` was improved by projecting more receipt-backed runtime evidence into the shared table while leaving `Monitoring Deltas` external-event-only.

That earlier slice solved a local presentation gap.

This plan addresses the larger architectural requirement:

1. observed traffic truth must come from one shared Shuma telemetry spine,
2. and simulator receipts must remain explicit sidecars rather than becoming a second penetration-truth system.
