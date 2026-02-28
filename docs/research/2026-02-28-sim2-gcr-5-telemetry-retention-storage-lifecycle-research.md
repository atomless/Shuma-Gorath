# SIM2-GCR-5 Research: Telemetry Retention and Storage Lifecycle Best Practices

Date: 2026-02-28  
Status: Recommended lifecycle model selected

## Objective

Define a Rust-appropriate telemetry retention/storage lifecycle for high-volume monitoring/event data with deterministic purge semantics, bounded query cost, and operator-visible retention health.

## Repository Baseline (Current State)

1. Event and monitoring telemetry retention is driven by `SHUMA_EVENT_LOG_RETENTION_HOURS`.
2. Cleanup currently runs opportunistically in read/summary paths (`load_recent_event_records`, `maybe_cleanup_monitoring`) and iterates all keys via `store.get_keys()` before deleting expired keys.
3. Cleanup cadence is effectively once per hour per process, with no explicit purge watermark/ledger exposed to operators.
4. Current model is functional for small volumes but risks read-path amplification and retention drift under scale.

## Primary-Source Findings

1. Log lifecycle should include explicit retention and disposal policy decisions as part of operational logging governance.
   Source: [NIST SP 800-92](https://www.nist.gov/publications/guide-computer-security-log-management)
2. Secure logging guidance requires verified disposal and policy-defined retention windows.
   Source: [OWASP Logging Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html)
3. Redis key expiry is supported (`EXPIRE`) with active/passive expiration mechanisms; expiry timing is not an exact deterministic deletion clock.
   Sources:
   - [Redis EXPIRE](https://redis.io/docs/latest/commands/expire/)
   - [Redis expiration algorithm](https://redis.io/docs/latest/operate/oss_and_stack/management/optimization/latency/#redis-evict-expire-cycle)
4. Keyspace-wide scans are expensive; `KEYS` is O(N) and not intended for regular production control paths.
   Source: [Redis KEYS](https://redis.io/docs/latest/commands/keys/)
5. Partitioned storage enables low-cost retention purges by dropping old partitions/chunks instead of row-by-row deletion.
   Sources:
   - [PostgreSQL Partitioning](https://www.postgresql.org/docs/current/ddl-partitioning.html)
   - [Timescale drop_chunks](https://docs.timescale.com/api/latest/hypertable/drop_chunks/)
6. Periodic background cadence should be explicit and bounded rather than piggybacked on user reads.
   Source: [Tokio `time::interval`](https://docs.rs/tokio/latest/tokio/time/fn.interval.html)

## Inferences for Shuma (Derived from Sources)

1. Read-path cleanup with full key scans is the wrong place for retention enforcement at scale.
2. Retention should be driven by time-bucketed storage layout + deterministic purge worker + watermark ledger.
3. Backend TTL can remain as safety net, but deterministic purge success should not depend only on passive expiry behavior.
4. Operators need retention health metrics (`oldest_data_age`, `purge_lag`, `keys_pending_purge`, `last_successful_purge_at`) to detect drift early.

## Architecture Options

### Option A: Keep Opportunistic Read-Path Cleanup (Current Shape)

Continue hourly, request-triggered cleanup that scans keyspace and deletes stale entries during read paths.

### Option B: Backend TTL-Only Strategy

Assign TTL at write time and rely on backend expiration behavior, with minimal explicit purge orchestration.

### Option C: Time-Bucketed Storage + Deterministic Purge Worker + Health Ledger (Recommended)

Store telemetry in time buckets, maintain a purge watermark/index, run bounded background purge batches on explicit cadence, and expose retention health status to admin/monitoring surfaces.

### Option D: Offload all telemetry retention to external observability warehouse

Treat local store as short-lived cache and rely on external data platform lifecycle policies.

## Decision Matrix

| Option | Benefits | Risks | Resource Cost | Security Impact | Rollback Complexity |
|---|---|---|---|---|---|
| A. Opportunistic cleanup | Minimal immediate change | Read-path amplification, nondeterministic purge outcomes, retention drift visibility gaps | Low initial, rising runtime cost | Moderate (data over-retention risk) | Low |
| B. TTL-only | Simpler write path, low code | Expiry timing not strictly deterministic; weak operator purge observability | Low-medium | Moderate | Low |
| C. Bucketed + deterministic purge + ledger (recommended) | Predictable purge semantics, bounded cleanup work, strong operator visibility | Requires bucket/index schema and purge worker orchestration | Medium | Strong (retention policy enforceable/auditable) | Medium |
| D. External warehouse only | Potentially strongest long-horizon analytics | Operational complexity and dependency expansion | High | Strong but externalized trust boundary | High |

## Recommendation

Adopt **Option C**.

Required lifecycle contract:

1. **Storage layout**
   1. Time-bucketed telemetry keys/partitions (`hour` or `day`) with explicit bucket index records.
   2. Separation of hot window (query path) from cold/expired buckets.
2. **Deterministic purge semantics**
   1. Purge worker runs on explicit cadence outside read paths.
   2. Purge by bucket (drop/delete whole bucket set), not key-by-key scans where avoidable.
   3. Persist purge watermark (`last_purged_bucket`) and outcome stats.
3. **Safety net and drift control**
   1. Optional backend TTL remains as secondary guardrail.
   2. Reconciliation detects stale buckets past retention and raises health alerts.
4. **Operator-visible retention health**
   1. Expose `retention_hours`, `oldest_retained_ts`, `purge_lag_hours`, `last_purge_success_ts`, `last_purge_error`, and `pending_expired_buckets`.

## Quantitative Targets (for TODO enforcement)

1. Retention purge lag: `<= 1 hour` beyond configured retention window under normal envelope.
2. Purge worker runtime budget: `<= 500ms` per cadence tick in hot path process budget.
3. Read-path retention overhead: zero full-keyspace scan triggered by admin monitoring refresh paths.
4. Drift tolerance: `pending_expired_buckets == 0` in healthy state; any non-zero state must surface degraded retention status.

## Plan and TODO Impact

1. New plan doc: `docs/plans/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-plan.md`.
2. Add dedicated telemetry-retention lifecycle slice (`SIM2-GC-15`) covering bucket schema, purge worker, health metrics, and deterministic tests.
3. Add verification tasks under `SIM2-GC-11` for retention determinism, purge lag thresholds, and read-path no-scan regressions.
