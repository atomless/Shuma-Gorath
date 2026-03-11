# Shared-Host Telemetry Storage and Query Evidence

Date: 2026-03-11

## Scope

This note records the first live shared-host telemetry evidence pass after the `TEL-STORE-1` storage/query-efficiency tranche landed.

Environment:
- Active remote: `dummy-static-site-fresh`
- Public base URL: `https://shuma.jamestindall.org`
- Remote host: `172.239.98.102`
- Evidence receipt: [/.spin/telemetry_shared_host_evidence.json](/Users/jamestindall/Projects/Shuma-Gorath/.spin/telemetry_shared_host_evidence.json)

Collection path:
- `make telemetry-shared-host-evidence`

## Key Findings

### Keyspace shape

- Default store total keys: `3032`
- Eventlog keys: `1115`
- Monitoring keys: `1081`
- Monitoring rollup keys: `0`
- Retention bucket indexes:
  - `eventlog`: `43`
  - `monitoring`: `43`
  - `monitoring_rollup`: `0`

Interpretation:
- hot-path telemetry is no longer read by whole-store enumeration
- the live host has enough retained hourly data to prove bucket-catalog addressing on both monitoring and eventlog domains
- daily monitoring rollups are implemented but there were no completed prior-day buckets yet on this host, so the rollup tier correctly remains empty

### Live query cost

- `/admin/monitoring`
  - `84.10 ms`
  - `37691 B`
- `/admin/monitoring` with gzip
  - `66.02 ms`
  - `5597 B`
  - `85.15%` compression reduction
- `/admin/monitoring/delta`
  - `69.60 ms`
  - `22328 B`
- `/admin/monitoring/stream`
  - `70.93 ms`
  - `22498 B`

Budget surface from the live response:
- `cost_class=heavy`
- `cost_units=1497`
- `query_budget_status=within_budget`
- `bucket_density=2.357142857142857`
- `density_penalty_units=0`
- `residual_scan_keys=0`

Read-surface breakdown:
- `monitoring_buckets=7`
- `monitoring_keys=9`
- `eventlog_buckets=7`
- `eventlog_keys=24`
- `rollup_buckets=0`
- `rollup_keys=0`
- `detail_catalog_keys=0`
- `residual_scan_keys=0`

Interpretation:
- normal operator reads are now scaling with requested retained buckets, not total keyspace size
- transport gzip remains highly effective and cheap enough to keep for monitoring payloads
- the live read path shows zero residual whole-keyspace scans in the normal monitoring surface

### Retention health

- `state=healthy`
- `retention_hours=168`
- `redacted_summary_retention_hours=168`
- `high_risk_retention_hours=72`
- `high_risk_retention_max_hours=72`
- `purge_lag_hours=0.0`
- `pending_expired_buckets=0`

Interpretation:
- the revised retention split is live and healthy
- raw high-risk operator evidence is capped to `72h`
- monitoring counters and summary views retain `168h`
- the purge worker is keeping up on the live shared host

## Assessment

`TEL-STORE-1` achieved the intended architectural shift:
- monitoring summary, delta, stream, and event-history reads now use bucket catalogs rather than whole-keyspace scans
- telemetry-adjacent monitoring details no longer depend on residual raw-key enumeration in normal reads
- retention now distinguishes raw event evidence from longer-lived monitoring state and rollup retention
- the query budget is storage-aware and reports the actual read surface and density instead of using only `hours * limit`

The remaining caveat is not a blocker:
- the live host had no completed prior-day monitoring rollups yet, so the rollup tier is proven by code/tests and by the empty live key family rather than by non-zero live rollup counts

## Compression Decision

Hot-path KV compression is rejected for now.

Reason:
- the live cost issue was read amplification, not value size
- live monitoring gzip already cuts snapshot payload size by `85.15%`
- compressing many small KV counter values would add CPU/complexity while making hot retrieval and inspection harder

Decision:
- keep transport gzip for monitoring payloads
- keep hot telemetry in directly readable KV form
- consider compression only for a future cold archival tier if later measurement shows clear benefit

## Verification

- `make test-telemetry-storage`
- `make test-deploy-linode`
- `make remote-update`
- `make telemetry-shared-host-evidence`
