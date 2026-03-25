# SEC-GDPR-2 Fingerprint Retention Cleanup Post-Implementation Review

Date: 2026-03-24
Status: Completed

Related work:

- [`../plans/2026-03-24-fingerprint-state-retention-cleanup-plan.md`](../plans/2026-03-24-fingerprint-state-retention-cleanup-plan.md)
- [`2026-03-24-fingerprint-state-retention-cleanup-review.md`](2026-03-24-fingerprint-state-retention-cleanup-review.md)
- [`../../src/signals/fingerprint.rs`](../../src/signals/fingerprint.rs)
- [`../../Makefile`](../../Makefile)

## What landed

`src/signals/fingerprint.rs` now performs bounded deterministic cleanup for persisted fingerprint retention keys:

1. stale `fp:state:*` entries are evicted by embedded timestamp against `fingerprint_state_ttl_seconds`,
2. stale `fp:flow:*` bucket counters are evicted whenever their bucket is no longer the current configured flow window,
3. stale `fp:flow:last_bucket:*` markers are evicted when they point at a no-longer-current bucket,
4. and the cleanup scan is cadence-gated behind `fp:cleanup:v1:last_run_ts` so `get_keys()` is not forced on every request.

The fix stays local to the fingerprint module and does not widen into a second retention worker or global telemetry-retention change.

## Proof

Focused verification now exists in:

- [`../../Makefile`](../../Makefile) as `make test-fingerprint-retention-cleanup`

The target proves:

1. stale sibling fingerprint state is swept even when the same identity never revisits,
2. stale flow and last-bucket keys are swept while current-window keys survive,
3. repeated requests inside the cleanup cadence do not rescan the full fingerprint keyset.

Verification run for this tranche:

1. `make test-fingerprint-retention-cleanup`
2. `git diff --check`

## Operational and privacy impact

This reduces privacy and retention drift by making the configured fingerprint TTL and flow-window controls truthful for stored state, not only for scoring logic.

The cleanup is still intentionally bounded:

1. it scans only fingerprint prefixes,
2. it runs no more often than the derived cleanup cadence,
3. and it leaves broader telemetry retention ownership unchanged.

## Remaining follow-up

`SEC-GDPR-3` and `SEC-GDPR-4` remain open:

1. storage-level event-log IP minimization mode,
2. deployer-ready privacy and cookie disclosure guidance.
