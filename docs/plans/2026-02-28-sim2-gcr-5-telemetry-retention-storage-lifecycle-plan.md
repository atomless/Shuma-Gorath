# SIM2-GCR-5 Plan: Telemetry Retention and Storage Lifecycle

Date: 2026-02-28  
Status: Proposed

Reference research:

- [`docs/research/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md`](../research/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md)

## Objective

Replace opportunistic read-path retention cleanup with deterministic, bucketed lifecycle management and operator-visible retention health.

## Non-goals

1. Replacing all storage backends in this tranche.
2. Moving telemetry entirely to external observability platform.
3. Expanding retention duration beyond current policy defaults.

## Architecture Decisions

1. Introduce bucketed telemetry storage/index contract for retention operations.
2. Move purge execution to explicit background cadence, not admin read paths.
3. Persist purge watermark/health state and expose it in monitoring/admin surfaces.
4. Keep backend TTL as secondary safety control, not primary retention guarantee.

## Delivery Phases

### Phase 1: Bucket and Index Contract

1. Define canonical bucket key schema and metadata index keys.
2. Ensure all monitoring/event writes include bucket/index updates.
3. Add migration compatibility path for existing keys during pre-launch transition.

Acceptance criteria:

1. New telemetry writes are bucket-addressable without full key scans.
2. Bucket index allows deterministic identification of expired buckets.
3. Legacy keys remain readable during migration window.

### Phase 2: Deterministic Purge Worker

1. Implement purge worker with explicit cadence and bounded batch budget.
2. Delete/drop expired buckets based on retention cutoff and persisted watermark.
3. Persist purge outcomes (`success`, `failed`, `partial`) and reason taxonomy.

Acceptance criteria:

1. Purge execution no longer occurs in monitoring read handlers.
2. Purge lag remains within target (`<=1 hour`) under normal envelope.
3. Worker failures are explicit and retry-safe.

### Phase 3: Retention Health Visibility

1. Add retention health payload to admin monitoring data.
2. Include fields: `retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `pending_expired_buckets`, `last_purge_success_ts`, `last_purge_error`.
3. Add dashboard status indicators for retention health (`healthy`, `degraded`, `stalled`).

Acceptance criteria:

1. Operators can detect retention drift without inspecting raw keys.
2. Degraded/stalled retention states are explicit and actionable.
3. Retention health aligns with purge-worker ground truth.

### Phase 4: Verification and Regression Gates

1. Add deterministic tests for bucket cutoff behavior and watermark progression.
2. Add failure-injection tests for purge worker partial failure/retry recovery.
3. Add guard tests proving monitoring refresh paths do not trigger full keyspace retention scans.

Acceptance criteria:

1. Retention determinism regressions fail with cutoff/watermark diagnostics.
2. Read-path scan regressions fail fast.
3. Retention health metrics are test-covered and CI-visible.

## Verification Strategy

1. `make test-unit`
2. `make test-integration` (with `make dev` running)
3. `make test`

## Rollback Plan

1. If purge worker introduces instability, temporarily disable new worker and keep legacy cleanup while retaining bucket index writes for forward compatibility.
2. Preserve retention health diagnostics even in fallback mode.
3. Restore deterministic purge once failure mode is resolved and test matrix is green.

## Definition of Done

1. Retention enforcement is deterministic and decoupled from read paths.
2. Purge lag and retention drift are observable with explicit health states.
3. Full-keyspace scan dependence is removed from normal monitoring refresh operations.
4. CI catches retention lifecycle regressions deterministically.
