# TAH-19 Tarpit Persistence Collateral-Risk Post-Implementation Review

Date: 2026-03-24

## Scope

Close `TAH-19` by tightening tarpit persistence escalation so coarse bucket state no longer drives punitive escalation.

## Delivered

1. Tarpit now keeps two persistence truths:
   - coarse bucket counts for operator visibility,
   - bounded exact-principal counts for punitive escalation.
2. The punitive escalation path now uses exact-principal persistence counts, so a fresh IP no longer inherits short-ban pressure from other actors in the same `/24` or `/64`.
3. Exact-principal tracking is bounded behind a capped catalog and fails open for unseen principals when that bounded tracker is full.
4. The tarpit docs now explicitly say offender buckets remain a compact telemetry view and are not the punitive escalation basis.
5. The tarpit escalation defaults were re-evaluated against the corrected evidence basis and intentionally left unchanged for now:
   - short ban at `>= 5`
   - block at `>= 10`
   - `ban_durations.tarpit_persistence` unchanged

## Verification

- `make test-tarpit-collateral-risk-contract`
- `git diff --check`

## Follow-on

1. `TAH-12` remains for dashboard/operator visibility and safe-tuning guidance around the expanded tarpit telemetry.
2. If later live evidence still shows tarpit persistence is too aggressive, revisit the thresholds or duration defaults against this corrected exact-principal basis rather than reintroducing coarser escalation evidence.
