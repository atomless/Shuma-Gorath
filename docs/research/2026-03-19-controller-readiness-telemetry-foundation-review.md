Date: 2026-03-19
Status: Active review addendum

Related context:

- [`2026-03-17-operator-decision-support-telemetry-audit.md`](./2026-03-17-operator-decision-support-telemetry-audit.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](./2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`../plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](../plans/2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`../plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](../plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)

# Purpose

Review the newly landed monitoring-telemetry foundation work against Shuma's longer-horizon operating model:

1. realistic hostile traffic from outside agents,
2. inside adversary agents generating tuning evidence,
3. inside oversight agents proposing or applying bounded configuration changes,
4. and the need for telemetry that can support benchmarks rather than only operator charts.

This note does not change the architectural direction already chosen. It records which pieces of the current foundation are genuinely ready for that future and which remain missing but should now be treated as first-order foundation work rather than later polish.

# Current Strengths

The recent telemetry-foundation tranche established the right seam.

Shuma now has:

1. one runtime-owned request-outcome object,
2. one live-versus-adversary-sim origin split,
3. one compact scope and lane summary in `MonitoringSummary`,
4. and focused proof that those summaries survive the hot-read refresh path.

That is the correct backbone for the agentic era because it gives Shuma:

1. one place where final request truth is emitted,
2. one bounded summary surface rather than a tail-of-events approximation,
3. and one architecture that can later feed both human operators and bounded inside controllers.

# Remaining Controller-Readiness Gaps

## 1. Byte attribution is not yet benchmark-grade

The current request-outcome counters record total bytes by scope cohort and lane cohort, but they do not preserve the byte split by final outcome class.

Today Shuma can answer:

1. how many requests were forwarded,
2. how many were short-circuited,
3. and how many bytes each scope or lane served in total.

It cannot yet answer:

1. how many suspicious bytes still reached origin,
2. how many bytes Shuma served locally as friction or block content,
3. or how much likely-human friction cost is attributable to specific local response classes.

That matters because future benchmarks will need more than request counts. A controller or operator trying to tune for:

1. lower suspicious origin cost,
2. lower likely-human friction cost,
3. or better cost-shift asymmetry

needs forwarded-versus-local byte attribution, not a blended byte total.

### Design consequence

The next telemetry foundation work should extend request-outcome counters and summaries so bytes are attributable by:

1. `outcome_class`,
2. and where useful, by bounded `response_kind`.

This should be treated as part of the telemetry foundation, not deferred to the monitoring overhaul.

## 2. The summary contract is still too coarse for benchmarking

The request-outcome counters already collect more semantics than the current summary exposes:

1. `response_kind`,
2. `policy_source`,
3. and `route_action_family`.

But the current `MonitoringSummary` only materializes:

1. `request_outcomes.by_scope`,
2. and `request_outcomes.by_lane`.

That is enough for initial lane mix, but it is not enough for the benchmark layer Shuma will eventually need.

Future operator and controller questions will include:

1. are suspicious requests being challenged, mazed, blocked, or still forwarded,
2. is most friction coming from first-tranche hard signals or second-tranche thresholded routing,
3. are humans mostly affected by challenge, not-a-bot, JS challenge, or fallback content,
4. and is one route family leaking more suspicious traffic than another.

Those questions require bounded summary shapes for:

1. `response_kind`,
2. `policy_source`,
3. and `route_action_family`,

not just hidden raw counters.

### Design consequence

The next summary tranche should not jump straight to UI work. It should first surface bounded summary rows or equivalent aggregates for the semantics already being collected, so the eventual monitoring overhaul and controller loop both stand on the same backend contract.

## 3. Request-outcome coverage is still incomplete at the control or fail edges

The current foundation cleanly covers store-backed handled responses through the runtime finalization hook, but several terminal responses still return before that hook:

1. HTTPS-required rejection,
2. early routes,
3. store-open failure,
4. config-load failure,
5. and static bypass.

Static bypass is an explicit deliberate exclusion for now, which is acceptable so long as it stays clearly documented.

The control-plane and fail-mode paths are more important. Those responses are part of Shuma's operational truth, and future inside agents will need to distinguish:

1. defence behavior,
2. degraded platform behavior,
3. and control-plane bypass or failure responses.

If that distinction is absent, a controller can optimize around incomplete evidence and draw the wrong conclusion about whether a change improved defences or merely coincided with fail-open or fail-closed drift.

### Design consequence

The remaining non-finalized terminal paths should be:

1. brought under the request-outcome contract where feasible,
2. or explicitly documented and summarized as intentional exclusions where not.

This is foundation work because it affects the truth boundary of every later summary.

# Roadmap Implication

This review changes prioritization more than direction.

The roadmap should now treat "controller-grade monitoring telemetry foundations" as a distinct substage inside Stage 1, before:

1. the full Monitoring UI overhaul,
2. tuning-surface completion,
3. and certainly before any recommend-or-apply inside agent work.

The right order is:

1. canonical lane and outcome contracts,
2. origin-aware counters,
3. compact scope and lane summaries,
4. outcome-byte attribution,
5. bounded response-kind or policy-source or route-family summaries,
6. close remaining fail-mode and control-path coverage gaps,
7. then operator Monitoring overhaul,
8. then Tuning completion,
9. then later benchmarked oversight-controller work.

# Benchmark Implication

For Shuma's future inside tuning agent, telemetry is only useful insofar as it can support benchmark families like:

1. suspicious origin cost,
2. likely-human friction rate,
3. likely-human friction byte or latency burden,
4. outcome mix by lane,
5. action-family effectiveness by route family,
6. shadow asserted action versus enforced actual action,
7. and sim-versus-live drift.

The newly landed foundation is necessary for those benchmarks, but not sufficient yet.

The missing pieces above are therefore not "nice to have monitoring detail." They are prerequisites for reliable benchmark-driven defence evolution.

# Recommended Backlog Changes

The active telemetry foundation tranche should explicitly prioritize:

1. forwarded-versus-local byte attribution,
2. bounded summary exposure for `response_kind`, `policy_source`, and `route_action_family`,
3. and closure of the remaining control-path and fail-path outcome gaps.

The Monitoring overhaul should remain blocked on those telemetry foundations rather than trying to invent a UI contract ahead of them.

The scheduled oversight-agent planning should also continue to remain blocked until those benchmark-grade telemetry foundations are in place.

# Outcome

The current direction remains correct.

Shuma has chosen the right telemetry seam for the agentic era.

The main insight from this review is that several missing pieces should now be treated as first-class foundation work:

1. byte attribution by outcome,
2. richer but still bounded backend summary semantics,
3. and fuller terminal-path coverage.

Those are the next telemetry priorities if Shuma wants its later autonomous tuning loop to optimize against truth rather than against a convenient but incomplete summary.
