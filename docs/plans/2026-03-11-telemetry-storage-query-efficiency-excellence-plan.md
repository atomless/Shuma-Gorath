# Telemetry Storage and Query Efficiency Excellence Plan

Date: 2026-03-11  
Status: Proposed

Reference context:

- [`docs/plans/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-plan.md`](../plans/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-plan.md)
- [`docs/research/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md`](../research/2026-02-28-sim2-gcr-5-telemetry-retention-storage-lifecycle-research.md)
- [`docs/research/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md`](../research/2026-02-28-sim2-gcr-6-monitoring-cost-efficiency-patterns-research.md)
- [`todos/security-review.md`](../../todos/security-review.md)

## Objective

Reassess and improve KV-backed operational telemetry now that shared-host deployment evidence exists, with emphasis on reducing storage growth, retention lag, and monitoring query cost without weakening operator visibility or security evidence.

## Repository Baseline (Current State)

1. Hot monitoring writes are already buffered and flushed in batches, which reduces write amplification for per-request counters.
2. Monitoring and event retention now use hourly bucket catalogs and a deterministic purge worker, and retention health is exposed to admin/monitoring surfaces.
3. Monitoring summary and recent-event reads still enumerate `store.get_keys()` and filter in process, which means query cost grows with whole-keyspace size rather than the requested telemetry window.
4. Monitoring response gzip is already implemented, so transport compression is not the primary missing capability.
5. Event logs are stored as immutable per-event KV records, which preserves evidence fidelity but can become expensive for cursor, snapshot, and recent-history reads as volume grows.

## Problem Statement

The storage lifecycle tranche solved deterministic purge and retention visibility, but the query path remains too dependent on whole-keyspace scans. This means telemetry cost and latency can still drift as the shared-host deployment accumulates:

1. Storage cost rises with every new hourly counter key and raw event record.
2. Query cost rises because summary, delta, and stream handlers still walk the global keyspace before narrowing to the requested window.
3. Retention policy is smarter than before, but not yet tiered enough for long-lived operational summaries versus short-lived forensic raw data.
4. Compression is already present on transport, but the repository has not yet proven whether at-rest compression would produce meaningful savings relative to its retrieval and complexity costs.

## Principles

1. Optimize data shape before adding compression.
2. Prefer bucket-addressable reads over whole-keyspace enumeration.
3. Keep hot operational summaries cheap, bounded, and fast to query.
4. Preserve raw security evidence where it matters, but retain it for the shortest justified window.
5. Make cost and retention health first-class operator-visible signals.
6. Avoid introducing backend-specific complexity unless measurement proves the current KV posture is insufficient.

## Non-goals

1. Replacing the local KV store outright in this tranche.
2. Moving all telemetry to an external observability platform.
3. Compressing all stored telemetry values by default.
4. Reworking the dashboard monitoring product surface beyond what is required to support cheaper and more truthful reads.

## Recommended Direction

### 1. Replace keyspace scans with bucket-addressable reads

Current write paths already register monitoring and event keys into hourly retention bucket indexes. The next step is to consume those indexes on read paths so that:

1. Monitoring summary reads only load bucketed monitoring keys in the requested hours.
2. Event history, delta, and stream reads only load event buckets in the requested hours.
3. Query cost scales with requested window and bucket density, not global key count.

This is the highest-leverage change and should come before any storage compression work.

### 2. Separate raw-event retention from operational-summary retention

Current retention still effectively shares one main configured horizon, with high-risk event logs capped to a shorter maximum. The next improvement should introduce tiered lifecycle thinking:

1. Raw event records: shortest justified retention, especially for high-risk/raw artifacts.
2. Monitoring counters: medium retention for operational review and dashboard trends.
3. Derived rollups: longer retention for coarse operational history at lower storage cost.

This keeps forensic detail available when needed without forcing raw granularity to carry all long-horizon monitoring history.

### 3. Precompute cheap rollups for common monitoring views

Dashboard summaries repeatedly reconstruct aggregates from per-hour counters and raw events. Precomputed rollups should be introduced for the dominant read paths:

1. short rolling windows (`1h`, `24h`)
2. coarse longer windows (`7d`, `30d`)
3. top-N and trend payloads that currently require repeated aggregation work

Raw detail remains available for investigation, but the normal dashboard path should not rebuild everything from base counters on every read.

### 4. Treat compression as a cold-tier or transport concern, not a hot-path default

Transport gzip is already present and should remain the default compression mechanism for monitoring payloads. At-rest compression should be evaluated only for colder artifacts where:

1. data is large enough to compress meaningfully,
2. reads are infrequent,
3. searchability and point-read simplicity are not primary needs.

The likely candidates are archived raw event bundles or future cold-history exports, not the hot KV counters that power the live dashboard.

## Architecture Phases

### Phase 1: Measurement and Shared-Host Baseline

1. Capture real shared-host telemetry evidence:
   - total monitoring key count
   - total eventlog key count
   - keys/hour by domain
   - payload sizes for `/admin/monitoring`, `/admin/monitoring/delta`, and `/admin/monitoring/stream`
   - latency/cost of those endpoints under the live shared-host profile
2. Record acceptable operating targets for:
   - query latency
   - payload size
   - retention lag
   - keyspace growth per hour/day

Acceptance criteria:

1. Shared-host telemetry growth and query-cost baselines are archived in a dated evidence note.
2. Query/storage targets are explicit enough to drive regression checks.

### Phase 2: Bucket-Indexed Read Paths

1. Replace whole-keyspace scans in monitoring summary aggregation with bucket-catalog-driven reads.
2. Replace whole-keyspace scans in recent event loading, delta, and stream endpoints with bucket-catalog-driven reads.
3. Preserve current cursor semantics and forensic-mode behavior.

Acceptance criteria:

1. Monitoring summary no longer calls `store.get_keys()` across the whole keyspace for normal reads.
2. Event delta/stream no longer call `store.get_keys()` across the whole keyspace for normal reads.
3. Regression tests prove requested-window cost no longer scales with unrelated historical key accumulation.

### Phase 3: Tiered Retention and Rollups

1. Define explicit retention tiers for:
   - raw event records
   - operational monitoring counters
   - derived rollups
2. Add rollup generation and lifecycle management aligned to the read patterns that matter most.
3. Expose tier health and lag in existing retention/cost governance payloads.

Acceptance criteria:

1. Operators can distinguish raw retention from summary/rollup retention.
2. Dashboard summary reads prefer rollups where available.
3. Storage growth per retained day is lower for long-horizon monitoring views.

### Phase 4: Cost Governance Hardening

1. Replace simplistic `hours * limit` query-budget heuristics with estimates that account for bucket density and response shaping.
2. Add regression thresholds for:
   - payload size
   - query latency
   - bucket read count
   - retention lag
3. Surface degraded-state reasons when telemetry cost envelopes are exceeded.

Acceptance criteria:

1. Monitoring cost governance reflects real storage/query behavior rather than only request parameters.
2. CI/verification fails when keyspace-scan regressions or payload regressions are reintroduced.

### Phase 5: Cold-Tier Compression Decision

1. Measure realistic compression ratios for candidate cold artifacts.
2. Compare storage savings against:
   - decompression overhead
   - query/search complexity
   - operator UX cost
3. Only add at-rest compression if measurements show a clear win for cold data.

Acceptance criteria:

1. Compression is either explicitly rejected for hot telemetry with written reasoning, or introduced only for cold/history artifacts with tests and docs.
2. No hot monitoring/dashboard read path depends on decompressing large stored blobs.

## Verification Strategy

1. Add focused Make targets for telemetry storage/query cost measurement and regression checks.
2. Keep `make test` as the umbrella gate after those focused targets exist.
3. Add explicit verification for:
   - no whole-keyspace scan in normal monitoring summary/delta/stream paths
   - retention-lag health behavior
   - payload-size and query-budget degradation states
   - rollup correctness against raw-source truth

## Operational and Security Notes

1. Security-critical event classes must remain unsampled and queryable within their declared raw retention window.
2. Shorter raw retention must not silently reduce operator forensic capability; any reduction must be explicit in docs and monitoring payloads.
3. Derived rollups must not become the only source of security-significant truth.

## Definition of Done

1. Normal monitoring and event-history reads are bucket-addressable and no longer depend on whole-keyspace scans.
2. Retention is tiered enough to distinguish raw evidence from longer-lived operational summaries.
3. Monitoring cost governance reflects real storage/query behavior and exposes degraded states honestly.
4. Shared-host evidence proves the revised approach reduces storage/query cost without degrading operator UX.
5. Compression decisions are evidence-based and limited to tiers where they clearly help.
