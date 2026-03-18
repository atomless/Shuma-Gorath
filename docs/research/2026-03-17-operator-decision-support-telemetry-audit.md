Date: 2026-03-17
Status: Active audit

Related context:

- [`../observability.md`](../observability.md)
- [`../dashboard-tabs/monitoring.md`](../dashboard-tabs/monitoring.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/metrics.rs`](../../src/observability/metrics.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../dashboard/src/lib/components/dashboard/StatusTab.svelte`](../../dashboard/src/lib/components/dashboard/StatusTab.svelte)

# Purpose

Audit the telemetry Shuma currently collects, identify which parts are genuinely useful for operator decision support, identify which parts are diagnostic or contributor-facing rather than operator-facing, and capture the most valuable telemetry that Shuma does not yet collect but should.

This audit is intended to feed `MON-OVERHAUL-1` and the later oversight-controller work. It is not itself an implementation plan.

# Method

This audit traces telemetry across four layers:

1. emission and storage,
2. hot-read and bounded summary materialization,
3. admin API read surfaces,
4. dashboard/operator consumption.

The goal is not merely to list counters, but to answer a harder question:

1. what evidence can a human operator or future scheduled controller actually use to make better policy decisions?

# Executive Summary

Shuma already collects a meaningful amount of telemetry, but it is arranged primarily around defence subsystems and storage/query economics rather than around operator questions.

Today the telemetry is strongest at answering:

1. which defence subsystems fired,
2. which challenge and friction paths succeeded or failed,
3. where specific attack signals such as honeypot, rate, GEO, CDP, maze, tarpit, and not-a-bot are appearing,
4. whether the monitoring substrate itself is fresh, bounded, and retained safely.

Today the telemetry is weak at answering:

1. how much suspicious traffic is still getting through,
2. how much friction likely-human traffic is experiencing,
3. how much host bandwidth/CPU/origin work is still being spent on suspicious traffic,
4. how effective each defence is relative to that cost,
5. how close the site is to explicit operator budgets,
6. how verified beneficial agents should be treated once verified identity lands.

The single biggest structural gap is the absence of strong denominators and lane-oriented summaries. Shuma records many triggered events, but far less of the total traffic context needed to interpret those triggers. As a result, the dashboard can say "there were 200 challenge failures" more easily than it can say "2.1% of likely-human traffic saw challenge friction" or "suspicious traffic still consumed 38% of forwarded bytes."

The right next move is not to keep piling more subsystem widgets into Monitoring. It is to add one operator-grade decision-support summary, materialized as a bounded hot-read document, that is organized around:

1. traffic mix and lane mix,
2. human friction,
3. suspicious traffic cost,
4. defence effectiveness,
5. shadow-vs-enforced comparison,
6. verified-agent outcomes once identity exists.

# What Shuma Collects Today

## 1. Event-log telemetry

The event log remains the richest per-event source. Event records currently carry:

1. event kind (`Ban`, `Unban`, `Challenge`, `Block`, `AdminAction`),
2. basic event fields (`ip`, `reason`, `outcome`),
3. taxonomy and `outcome_code`,
4. `botness_score`,
5. adversary-sim metadata (`sim_run_id`, `sim_profile`, `sim_lane`, `is_simulation`),
6. execution metadata (`execution_mode`, `intended_action`, `enforcement_applied`).

Important current truth:

1. monitoring-facing reads now exclude `AdminAction` and any row with an `admin` actor, so Monitoring is external-traffic-only.
2. shadow execution is already represented in a durable event contract, which is good groundwork for the eventual shadow-vs-enforced operator model.

Limitations:

1. the event log does not model total ingress cleanly enough to act as a denominator for traffic-mix or human-friction rate calculations,
2. many high-level meanings are encoded in `reason` and `outcome` strings rather than in a tighter domain model,
3. event rows remain useful for drill-down and forensics, but they are not themselves the right operator summary surface.

## 2. Counter-based monitoring summary telemetry

The bounded `MonitoringSummary` materialized by [`src/observability/monitoring.rs`](../../src/observability/monitoring.rs) currently covers:

1. `shadow`,
2. `honeypot`,
3. `challenge`,
4. `not_a_bot`,
5. `pow`,
6. `rate`,
7. `geo`.

This is a solid early module-level summary. The most useful operator-grade pieces already present here are:

1. challenge failure reasons and offender counts,
2. proof-of-work success ratio and attempt totals,
3. not-a-bot served/pass/escalate/fail/replay totals,
4. not-a-bot solve latency buckets,
5. not-a-bot abandonment estimate,
6. rate-limit outcomes,
7. GEO action mix,
8. shadow action counts and pass-through totals.

Limitations:

1. the summary is module-centric rather than question-centric,
2. it omits maze, tarpit, CDP, IP-range policy, and active bans from the summary contract,
3. it still lacks explicit cost, denominator, and budget semantics.

## 3. Details-only monitoring telemetry

The broader `/admin/monitoring` `details` payload currently adds:

1. recent event rows,
2. recent sim runs,
3. `event_counts`, `top_ips`, and `unique_ips`,
4. active bans,
5. maze stats,
6. tarpit stats,
7. CDP detections and CDP event rows,
8. IP-range-policy summaries,
9. retention health,
10. security/privacy telemetry,
11. query/payload/compression cost-governance telemetry.

This is useful, but it mixes very different classes of information:

1. operator decisions,
2. contributor diagnostics,
3. storage and query governance,
4. forensic/security hygiene.

That mixing is one reason the Monitoring tab still feels more contributor-diagnostic than operator-decisional.

## 4. Prometheus-only or Prometheus-first telemetry

Prometheus metrics add some valuable signals that are not yet turned into good operator summaries.

Examples include:

1. forward attempts, successes, failures, and latency buckets,
2. botness signal visibility,
3. policy match and policy signal counters,
4. tarpit duration and bytes buckets,
5. defence mode effective state,
6. edge integration mode.

These are useful building blocks, but at present they mostly live at the "metric family" layer rather than at an operator decision-support layer.

## 5. Health, retention, and security/privacy telemetry

Shuma also collects:

1. freshness and lag state,
2. retention worker health,
3. security/privacy classification and scrubbing telemetry,
4. unsampleable drop counts,
5. bounded-read/query-budget diagnostics.

This is important telemetry, but it belongs primarily in `Status` and collapsed diagnostics, not in the main operator outcome narrative.

# What Is Already Valuable For Operator Decision Support

## A. Human-friction evidence that already exists

The strongest existing human-friction telemetry is in the not-a-bot and proof-of-work paths:

1. not-a-bot served/pass/escalate/fail/replay,
2. solve latency buckets,
3. abandonment estimate,
4. proof-of-work attempts, failures, successes, and success ratio.

This is genuinely useful because it begins to answer:

1. are people being asked to do work,
2. are they succeeding,
3. are they giving up,
4. is the proof mechanism becoming too onerous.

## B. Attack hotspot evidence that already exists

The current telemetry is already reasonably good at highlighting where abuse shows up:

1. honeypot top crawlers and top paths,
2. challenge and rate top offenders,
3. GEO top countries,
4. maze top crawlers,
5. CDP detection counts,
6. IP-range-policy match breakdowns.

That helps operators answer:

1. which paths are being probed,
2. which sources are repeatedly hitting specific defences,
3. whether a specific subsystem is carrying too much of the defensive burden.

## C. Shadow-vs-enforced metadata groundwork

The system already records enough per-event truth to keep shadow-mode and enforced-mode semantics separate:

1. `execution_mode`,
2. `intended_action`,
3. `enforcement_applied`.

That is a strong foundation. The problem now is not absence of telemetry, but that the monitoring surface still does not summarize those truths in the operator-friendly way it needs to.

## D. Telemetry trustworthiness and boundedness evidence

The retention/freshness/cost-governance instrumentation is already valuable for one narrow operator question:

1. can I trust that what I am seeing is fresh, bounded, and not silently overflowing?

That belongs in `Status` and diagnostics, and it is especially important for later autonomous oversight.

# What Is Currently Diagnostic Or Contributor-Oriented Rather Than Operator-Oriented

The following telemetry is important, but should not dominate Monitoring's default reading experience.

## 1. Raw transport/read-path details

Examples:

1. `transport`,
2. `slow_consumer_lag_state`,
3. `overflow`,
4. raw feed lines,
5. cursor contracts and stream contracts.

These are useful for debugging monitoring delivery and bounded-read behavior, but they are not the primary evidence an operator needs to tune bot defences.

## 2. Query-budget and payload-budget internals

Examples:

1. exact query cost units,
2. bucket density penalties,
3. residual scan keys,
4. read-surface key counts,
5. compression internals.

These are important for telemetry excellence and performance engineering, but they should mostly live in collapsed diagnostics or a contributor surface unless they breach a budget badly enough to affect operator trust.

## 3. Security/privacy telemetry in the main monitoring narrative

Examples:

1. secret scrub counters,
2. secret canary detection,
3. retention override audit hooks,
4. classification state.

These are operationally important, but they are a different concern from "how are defences performing against traffic?" They belong in `Status`, security review surfaces, or diagnostics.

## 4. Coarse event totals without lane context

Examples:

1. total event count,
2. top IPs by enforced events,
3. raw event-type doughnuts.

These are not useless, but they are much less valuable than they first appear because they lack:

1. traffic denominators,
2. lane classification,
3. distinction between suspicious and likely-human traffic,
4. a clear interpretation of success or failure.

# The Most Important Things Shuma Does Not Yet Collect

## 1. Total traffic denominators by lane

This is the single highest-value missing telemetry class.

Shuma needs bounded summary counts for:

1. total ingress requests,
2. forwarded requests,
3. frictioned requests,
4. denied requests,
5. broken down by at least:
   1. likely-human,
   2. verified beneficial agent,
   3. declared crawler/search bot,
   4. unverified/suspicious automation,
   5. unknown/mixed.

Without this, operators cannot answer percentage questions cleanly.

## 2. Human-friction telemetry across all defence families

Shuma currently has partial friction evidence, mostly through not-a-bot and proof-of-work. It does not yet have a unified operator summary for:

1. challenge rate seen by likely-human traffic,
2. maze rate seen by likely-human traffic,
3. block/deny rate that likely hit real humans,
4. solve/success rate after each friction type,
5. added latency per friction family,
6. pass-through after friction.

This is essential if the project's guiding principle is minimum human burden.

## 3. Suspicious-traffic cost telemetry

Shuma needs operator-facing summaries for:

1. suspicious bytes served,
2. suspicious forwarded request count,
3. suspicious approximate origin work,
4. suspicious latency/CPU proxy cost,
5. cost imposed by tarpit/maze versus cost avoided at the origin.

Some low-level ingredients already exist, especially around tarpit bytes and forward latency, but they are not currently assembled into an operator summary.

## 4. Defence-effectiveness funnel telemetry

Operators need to see how requests move through the system, not just which counters incremented.

For each major defence family, Shuma should eventually summarize something like:

1. candidates seen,
2. friction issued,
3. solved/passed,
4. failed,
5. escalated,
6. denied,
7. banned,
8. recurred later.

This matters more than raw trigger counts because it reveals whether a defence is merely noisy or genuinely effective.

## 5. Probable escape and probable false-positive telemetry

Shuma should gather bounded operator summaries for:

1. suspicious high-botness requests that were still allowed,
2. repeated suspicious actors that kept being allowed after friction,
3. likely-human requests that hit friction or denial but later showed successful human proofs or low-risk follow-on behavior.

This should be carefully labeled as estimated or derived when it is not exact.

## 6. Ban quality and recurrence telemetry

For future banded jitter and repeat-offender work, Shuma needs telemetry that says:

1. which ban families are firing,
2. which ban families recur after expiry,
3. which durations appear too short,
4. which durations appear overly harsh,
5. how often the same offender bucket comes back inside a recidive window.

This is currently not assembled as an operator-visible summary.

## 7. Verified-agent and verified-bot telemetry

This is intentionally absent today because the feature is not yet implemented, but it must be planned now.

Shuma will need to collect:

1. verification attempts,
2. verification successes and failures,
3. replay rejects,
4. route outcomes for verified identities,
5. low-cost profile usage,
6. local policy allow/restrict/deny outcomes per named verified identity or category.

Without that, the later verified-identity lane cannot become an operator-tunable control plane.

## 8. Explicit budget-distance telemetry

The future oversight controller should not have to infer budget alignment from raw counters.

Shuma should expose explicit summaries such as:

1. human challenge rate versus budget,
2. human p95 added latency versus budget,
3. suspicious bytes served versus budget,
4. suspicious origin cost versus budget,
5. verified-agent success versus budget,
6. monitoring freshness versus budget,
7. unsampleable drop count versus budget.

This is more valuable than leaving the controller to derive these ad hoc.

# Recommended Operator Telemetry Model

## Principle

Shuma should keep the existing subsystem telemetry, but Monitoring should stop treating it as the primary product model.

The main operator surface should instead be driven by one bounded, materialized decision-support summary that answers operator questions directly.

## Proposed top-level operator summary groups

### 1. Traffic Mix

What proportion of traffic appears to be:

1. likely-human,
2. verified beneficial agent,
3. declared crawler/search bot,
4. suspicious automation,
5. unknown.

### 2. Human Experience

How much likely-human traffic is seeing:

1. friction,
2. denial,
3. added latency,
4. abandonment after friction,
5. successful resolution after friction.

### 3. Suspicious Traffic Cost

How much suspicious traffic is still:

1. being forwarded,
2. consuming bytes,
3. consuming origin work,
4. getting trapped or shifted into costlier paths.

### 4. Defence Effectiveness

Per defence family:

1. candidates,
2. issued actions,
3. passes,
4. fails,
5. escalations,
6. bans,
7. repeat offenders.

### 5. Shadow Comparison

For the same period:

1. what was enforced,
2. what shadow mode says would have happened,
3. the difference between the two,
4. where shadow suggests stricter or looser tuning may be appropriate.

### 6. Red-Team Evidence

This should remain linked, but separate:

1. recent sim runs,
2. lane/profile mix,
3. defense deltas,
4. ban outcomes,
5. later sim-vs-real comparisons.

## Exactness discipline

To preserve telemetry excellence, every operator summary should carry explicit exactness semantics:

1. exact,
2. derived but deterministic,
3. estimated,
4. sample-based.

This already fits the hot-read architecture and should become more prominent as Monitoring evolves.

# Section Ownership Recommendation

## Monitoring

Should answer:

1. what external traffic is happening,
2. how effective the defences are,
3. what friction humans and beneficial bots are seeing,
4. what shadow predicts versus what enforcement actually did.

Should not be the default home for:

1. raw transport/debugging internals,
2. security/privacy counters,
3. low-level storage/query-budget internals.

## Status

Should answer:

1. can I trust this telemetry,
2. is the dashboard connected and fresh,
3. is retention healthy,
4. are the monitoring feeds delayed or degraded.

## Red Team

Should answer:

1. what the adversary-sim recently did,
2. which lanes and scenarios were exercised,
3. what defensive deltas it triggered,
4. how that evidence should feed tuning.

# Highest-Priority Additions

If Shuma adds only a small number of new telemetry families in the next tranche, the most valuable order is:

1. total traffic denominators by lane,
2. human-friction rates and added latency across all friction families,
3. suspicious bytes/origin-cost summaries,
4. defence-effectiveness funnel summaries,
5. explicit budget-distance summaries.

These five additions would do the most to turn Monitoring into an operator decision surface rather than a subsystem dashboard.

# Conclusions

Shuma's telemetry foundation is already good enough to support a strong Monitoring overhaul, but only if the project changes the organizing model.

Today the system knows a lot about:

1. which defences fired,
2. how bounded the read path is,
3. whether the telemetry substrate is fresh and retained safely.

What it does not yet know well enough, at the operator surface, is:

1. how much friction humans are absorbing,
2. how much suspicious traffic still costs the host,
3. how effective each defence is relative to that cost,
4. how far the deployment is from the desired budgets.

That is the central telemetry gap Shuma should close next.
