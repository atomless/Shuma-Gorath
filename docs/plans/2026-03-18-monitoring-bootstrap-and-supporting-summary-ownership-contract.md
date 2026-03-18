Date: 2026-03-18
Status: Proposed design

Related context:

- [`2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](./2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`2026-03-18-monitoring-operator-summary-exactness-contract.md`](./2026-03-18-monitoring-operator-summary-exactness-contract.md)
- [`2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](./2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](./2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`2026-03-13-compact-event-telemetry-implementation-plan.md`](./2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`../research/2026-03-14-compact-event-telemetry-live-evidence.md`](../research/2026-03-14-compact-event-telemetry-live-evidence.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define which Monitoring summaries belong in bootstrap, which belong in supporting hot-read documents, and which should remain drill-down or diagnostics-only surfaces.

This note exists because the next telemetry tranche wants to add more operator-grade summaries, but Shuma has already proven that hot-read document growth has real storage and read-path cost. The right answer is not "show less," but "put each summary in the correct ownership tier."

# Objectives

1. Keep Monitoring bootstrap fast and bounded.
2. Give operators immediate access to the handful of summaries they genuinely need on first paint.
3. Prevent raw event tails and diagnostics surfaces from becoming accidental substitutes for missing operator summaries.
4. Keep room for richer supporting summaries without re-bloating the default read path.

# Non-goals

1. Moving every existing bootstrap component immediately.
2. Replacing recent-event or recent-run supporting data with nothing.
3. Forcing one API call per widget.
4. Treating bootstrap ownership as a UI-only concern instead of a telemetry architecture concern.

# Core Ownership Principle

Shuma should treat operator Monitoring data as three tiers:

1. `bootstrap_critical`
2. `supporting_summary`
3. `diagnostic_or_drilldown`

That tiering should be explicit in implementation planning and admin read-contract design.

## `bootstrap_critical`

Meaning:

1. the summary must be present on initial Monitoring load
2. the operator cannot sensibly read the top-level Monitoring narrative without it
3. the summary must stay compact enough to fit inside current hot-read discipline

## `supporting_summary`

Meaning:

1. the summary is important and should be cheap to read,
2. but it is not required for the very first Monitoring paint
3. it may live in a separate bounded hot-read document or follow-up read path

## `diagnostic_or_drilldown`

Meaning:

1. the data is useful for contributors, deep operator investigation, or secondary workflows
2. it must not dominate first-paint Monitoring semantics
3. raw feeds and transport diagnostics belong here

# Bootstrap Budget Rules

The bootstrap payload should contain only:

1. compact summaries that shape the top-level operator story,
2. their freshness and exactness metadata,
3. and no contributor-diagnostic detail unless that detail is required to interpret the headline truth safely.

Bootstrap must not absorb:

1. raw or long recent-event detail just because it already exists,
2. deep dimensioned drilldowns,
3. large per-family funnel tables,
4. transport debugging detail,
5. or contributor-only diagnostics that belong in collapsed surfaces.

# Ownership Contract For The Monitoring Tranche

## Tier 1: Bootstrap-critical operator summaries

`MON-TEL-1` should treat the following as bootstrap-critical:

1. compact runtime posture or execution-mode summary that tells the operator whether the site is currently enforcing or running in `shadow_mode`
2. compact lane-mix headline summary for primary ingress traffic
3. compact human-friction headline summary
4. compact suspicious-cost headline summary
5. explicit freshness and exactness metadata for those summaries

These are the minimum operator questions that should be answerable immediately:

1. what kind of traffic is reaching the site
2. how much likely-human traffic is seeing friction
3. how much suspicious traffic is still consuming host cost
4. are these numbers enforced, shadow, exact, derived, or best-effort

## Tier 2: Supporting summaries

`MON-TEL-1` and `MON-OVERHAUL-1` should treat the following as supporting summaries:

1. normalized defence-effectiveness funnels
2. dimensioned breakdowns by path family, country, ASN, lane, or signal family
3. compact recent sim runs
4. recent event tails
5. retention health and security/privacy summaries when they are useful for operator confidence but not essential to the top Monitoring narrative

These should be available cheaply, but they do not all belong in the first response if that would bloat bootstrap.

## Tier 3: Diagnostics and raw feeds

The following should remain explicitly diagnostic:

1. raw telemetry feed
2. transport-path details
3. overflow and read-path contributor diagnostics
4. internal debugging counters or transient contributor instrumentation

These are valuable, but they should stay in collapsed or secondary surfaces and must not become the hidden foundation for primary operator interpretation.

# Composition Rules

## Rule 1: bootstrap summaries must be compact first, rich second

If a summary can be:

1. made compact enough for bootstrap,
2. or expressed only as a large table,

Shuma should place the compact headline in bootstrap and push the detail into a supporting summary.

## Rule 2: supporting summaries must still be bounded and materialized

Supporting summaries are not permission to:

1. scan raw event history on every read,
2. widen recent tails,
3. or reconstruct operator meaning in the dashboard from low-level data

They should still be:

1. bounded
2. materialized
3. explicit about freshness and exactness

## Rule 3: raw tails must not become silent fallback analytics

If a desired operator summary is missing, the answer is:

1. add the correct bounded summary

not:

1. infer the same meaning in the dashboard from recent events,
2. or promote contributor diagnostics into a quasi-operator surface

This is especially important for:

1. suspicious-cost posture
2. human-friction rates
3. defence-effectiveness funnels

## Rule 4: bootstrap should prefer one narrative summary per operator question

Avoid pushing many adjacent summaries into bootstrap that answer the same question in slightly different ways.

Example:

1. a lane-mix headline and a suspicious-cost headline may both belong in bootstrap
2. three different versions of suspicious-cost by path, response kind, and signal family do not

# Initial Ownership Guidance Table

| Summary or data family | Ownership tier | Why |
| --- | --- | --- |
| runtime posture summary | `bootstrap_critical` | First-paint Monitoring must know whether it is interpreting enforced or shadow telemetry |
| compact lane mix | `bootstrap_critical` | Core top-level operator story |
| compact human-friction posture | `bootstrap_critical` | Core human-impact story |
| compact suspicious-cost posture | `bootstrap_critical` | Core attacker-cost and leakage story |
| exactness and freshness metadata for those headlines | `bootstrap_critical` | Headline truthfulness depends on it |
| normalized defence funnel summary | `supporting_summary` | Important, but larger and less essential for first paint |
| detailed lane or route-family drilldowns | `supporting_summary` | Useful next step, not first-paint requirement |
| recent event rows | `supporting_summary` | Useful investigative context, not primary narrative |
| recent sim runs | `supporting_summary` | Important for Red Team and supporting context, not first-paint Monitoring truth |
| retention health summary | `supporting_summary` unless needed to explain stale Monitoring state | Important, but generally secondary |
| security/privacy summary | `supporting_summary` | Useful posture context, not primary Monitoring story |
| raw telemetry feed | `diagnostic_or_drilldown` | Contributor/operator deep inspection surface |
| transport and overflow diagnostics | `diagnostic_or_drilldown` | Diagnostics, not narrative monitoring |

# Relationship To Current Bootstrap

This note does not require Shuma to move every currently bootstrapped component immediately.

It does require:

1. all new Monitoring-overhaul summaries to obey this ownership contract,
2. and any future cleanup of current bootstrap to move toward this model rather than away from it

So the rule for `MON-TEL-1` is:

1. do not make bootstrap fatter just because a new summary is useful
2. choose the correct ownership tier first

# Relationship To Exactness

Ownership tier and exactness are separate.

Examples:

1. a bootstrap-critical summary can be `derived`
2. a supporting summary can be `exact`
3. a diagnostic feed can still be exact raw data

The bootstrap decision is about operator first-paint importance and budget discipline, not about prestige or truth value.

# Impact On `MON-TEL-1`

## `MON-TEL-1-5`

Must materialize new operator summaries into ownership tiers deliberately rather than defaulting everything into bootstrap.

## `MON-TEL-1-6`

Must expose the new summaries through the admin monitoring contract in a way that preserves the ownership split instead of flattening it back into one oversized payload.

## `MON-OVERHAUL-1`

Must build the UI around this ownership model:

1. primary operator headlines first
2. supporting summaries second
3. diagnostics collapsed or clearly secondary

# Definition Of Done

This prerequisite is complete when:

1. new operator summaries have an explicit ownership tier,
2. bootstrap-critical Monitoring data is clearly scoped,
3. supporting summaries are explicitly bounded and materialized,
4. raw feeds and diagnostics are prevented from becoming accidental primary analytics,
5. and `MON-TEL-1` can proceed without silently growing the default hot-read footprint.
