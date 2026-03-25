Date: 2026-03-24
Status: Proposed

Related context:

- [`../plans/2026-03-24-fingerprint-state-retention-cleanup-plan.md`](../plans/2026-03-24-fingerprint-state-retention-cleanup-plan.md)
- [`../../src/signals/fingerprint.rs`](../../src/signals/fingerprint.rs)
- [`../../docs/plans/2026-02-13-fingerprint-excellence-plan.md`](../../docs/plans/2026-02-13-fingerprint-excellence-plan.md)
- [`../../docs/research/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md`](../../docs/research/2026-02-28-sim2-gcr-7-telemetry-artifact-security-privacy-controls-research.md)

# Fingerprint State Retention Cleanup Review

## Question

What is still missing to make fingerprint-state retention truthful and privacy-bounded for:

1. `fp:state:*`
2. `fp:flow:*`
3. `fp:flow:last_bucket:*`

## Current code-grounded state

`src/signals/fingerprint.rs` already deletes stale `fp:state:*` entries opportunistically when the same identity is revisited through `load_state(...)`.

It also keeps `fp:flow:*` bounded only in a narrow per-identity way:

1. the current request updates one `fp:flow:<identity>:<bucket>` key,
2. the previous bucket is deleted only when that same identity returns in a later bucket,
3. and `fp:flow:last_bucket:<identity>` is updated but never independently expired.

That means stale fingerprint keys can remain indefinitely when the owning identity does not revisit.

## Why the current behavior is insufficient

This falls short of the repo's already-stated privacy and retention direction:

1. fingerprinting work must ship with TTL and retention controls,
2. pseudonymization is not enough if stale keyed state can still linger forever,
3. and the flow-window controls should mean something operationally for stored keys, not only for scoring logic.

The gap is especially clear for `fp:flow:last_bucket:*`: it has no lifecycle beyond overwrite.

## Constraints

The KV abstraction does support `get_keys()`, so Shuma can enumerate keys when needed.

But a full keyspace scan on every request would be a poor fit for the request path. Any cleanup fix needs to stay bounded and cadence-gated.

## Recommended fix

Add one bounded fingerprint cleanup path inside `src/signals/fingerprint.rs` that:

1. scans only the fingerprint prefixes,
2. runs no more often than a small configured cadence derived from the existing fingerprint TTL/window controls,
3. deletes stale `fp:state:*` by embedded timestamp,
4. deletes stale `fp:flow:*` bucket counters by comparing stored bucket suffixes to the current flow window,
5. deletes stale `fp:flow:last_bucket:*` markers when their referenced bucket is no longer current.

This is not a new retention worker or a new global subsystem. It is the smallest truthful lifecycle fix for the fingerprint state Shuma already persists.

## Recommended proof

The implementation should prove:

1. stale `fp:state:*` keys are evicted according to `fingerprint_state_ttl_seconds`,
2. stale `fp:flow:*` and `fp:flow:last_bucket:*` keys are evicted according to `fingerprint_flow_window_seconds`,
3. active current-window keys are preserved,
4. the cleanup path is cadence-gated so repeated requests do not force a full key scan every time.

## Result

`SEC-GDPR-2` should land as a focused fingerprint-retention slice:

1. bounded cleanup logic in `src/signals/fingerprint.rs`,
2. focused unit coverage for stale state and flow-key eviction,
3. a narrow Make target for the new proof,
4. and updated testing or privacy docs so the lifecycle contract is explicit.
