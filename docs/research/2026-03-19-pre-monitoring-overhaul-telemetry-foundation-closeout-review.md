Date: 2026-03-19
Status: Closeout review

Related context:

- [`2026-03-17-operator-decision-support-telemetry-audit.md`](./2026-03-17-operator-decision-support-telemetry-audit.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](./2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](./2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`../plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md`](../plans/2026-03-19-controller-grade-monitoring-telemetry-foundation-follow-on-plan.md)
- [`../plans/2026-03-19-monitoring-origin-aware-followup-telemetry-plan.md`](../plans/2026-03-19-monitoring-origin-aware-followup-telemetry-plan.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)

# Purpose

Confirm whether Shuma has completed the backend monitoring-telemetry foundation work that must land before the Monitoring UI overhaul.

This review is intentionally narrow. It is not a new architecture exploration. It checks the delivered telemetry foundation against the controller-grade requirements already recorded in research, plans, roadmap sequencing, and TODOs.

# Review Conclusion

The backend telemetry foundation required before `MON-OVERHAUL-1` is now complete.

No further telemetry architecture sweep is needed before the Monitoring overhaul discussion and section-ownership planning work begins.

# What Is Now Complete

## 1. Outcome-attributed byte telemetry

Shuma now preserves bounded byte attribution by request-outcome cohort instead of only blended response-byte totals.

That gives the backend summary surface the primitives needed for:

1. suspicious bytes that still reached origin,
2. locally served friction or block bytes,
3. control-response bytes kept separate from the main ingress story.

## 2. Bounded benchmark summary shapes

`MonitoringSummary.request_outcomes` now exposes bounded grouped rows for:

1. `response_kind`,
2. `policy_source`,
3. `route_action_family`.

Those semantics are no longer stranded in raw counters. The admin monitoring read contract and dashboard snapshot path now preserve the richer summary families without widening raw tails or inventing ad hoc UI-only interpretation layers.

## 3. Terminal-path truth boundary

The remaining terminal-path coverage question is now closed at the correct boundary.

### Included

`config` bootstrap failure is under the request-outcome contract through the request-flow-owned bootstrap failure finalization path.

### Intentionally excluded

The remaining pre-store paths remain explicitly excluded for now:

1. HTTPS-required rejection,
2. early routes,
3. static asset bypass,
4. store-open failure.

This is acceptable before the Monitoring overhaul because:

1. the exclusions are explicit and documented,
2. the runtime classification model distinguishes them from live ingress rather than silently folding them into site-traffic summaries,
3. and the project has deliberately chosen not to burden the pre-store fast path with ad hoc telemetry writes.

## 4. Origin integrity for follow-up telemetry

Legacy `challenge`, `not_a_bot`, and `pow` follow-up telemetry is now origin-aware.

That means:

1. live operator summaries no longer silently blend adversary-sim evidence,
2. richer defence-funnel stages can use live-safe backing counters,
3. adversary-sim evidence remains present in the appropriate detail and tuning-oriented paths.

## 5. Admin read contract and boundedness proof

The admin monitoring snapshot now exposes the richer operator summary contract, and the hot-read projection includes an explicit bootstrap-size budget assertion.

So the pre-overhaul backend work now has:

1. a bounded hot-read summary surface,
2. explicit operator-summary exactness and ownership rules,
3. focused backend API proof,
4. focused dashboard snapshot-path proof,
5. and budget-safe serialization proof.

# What This Means For The Roadmap

Stage 1 controller-grade monitoring telemetry foundations are now complete.

The next work is not more telemetry architecture. The next work is:

1. discuss the Monitoring overhaul,
2. write the Monitoring section-ownership and composition plan,
3. then execute `MON-OVERHAUL-1`.

`TUNE-SURFACE-1` should remain behind that Monitoring work, exactly as already planned.

# Remaining Risks

There are still future telemetry and monitoring improvements Shuma may want, but they are not blockers for the Monitoring overhaul:

1. richer later-stage adversary-sim-specific operator views,
2. verified-identity monitoring once Web Bot Auth work lands,
3. later controller benchmark families built on top of the current foundation.

Those are next-stage enhancements, not missing prerequisites.

# Recommendation

Do not perform another broad telemetry research or architecture sweep before the Monitoring overhaul.

Instead:

1. treat the backend telemetry foundation as complete,
2. move `MON-OVERHAUL-1` to the next active discussion and planning focus,
3. keep the overhaul blocked only on the section-ownership plan and user alignment for the Monitoring operator surface.
