Date: 2026-03-18
Status: Active research

Related context:

- [`2026-03-17-operator-decision-support-telemetry-audit.md`](./2026-03-17-operator-decision-support-telemetry-audit.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`../plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](../plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`../plans/2026-03-13-compact-event-telemetry-implementation-plan.md`](../plans/2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`2026-03-14-compact-event-telemetry-live-evidence.md`](./2026-03-14-compact-event-telemetry-live-evidence.md)
- [`../observability.md`](../observability.md)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Compare Shuma's current telemetry collection with the operator-grade, agentic-era telemetry model described in the recent research, while preserving the hard-won efficiency, hot-read, and retention discipline already established in the telemetry architecture.

This note is intentionally a gap analysis and prioritization document. It is not yet the implementation plan for `MON-OVERHAUL-1`.

# Executive Summary

Shuma is in a better place than it may feel at first glance.

The project already has:

1. a bounded hot-read architecture,
2. compact event rows,
3. explicit freshness and retention health,
4. subsystem summaries for several important defence families,
5. and a clean separation between external traffic telemetry and operator activity.

The main shortfall is not lack of telemetry volume. It is mismatch between:

1. what Shuma currently summarizes,
2. what operators actually need to decide,
3. and what future controller or scheduled-agent loops will need to reconcile against budgets.

More concretely:

1. Shuma is strong on triggered defence evidence.
2. Shuma is weak on total traffic denominators and traffic-lane summaries.
3. Shuma is strong on telemetry trustworthiness and boundedness.
4. Shuma is weak on operator-grade cost and effectiveness summaries.
5. Shuma is already efficient enough that the next tranche should add carefully chosen summary counters, not a new analytics subsystem.

The highest-value next move is therefore:

1. keep the current hot-read and retention architecture,
2. add a small number of coarse lane and denominator counters,
3. materialize one operator decision-support summary document,
4. and avoid widening raw event tails or introducing high-cardinality per-request analytics.

# Non-Negotiable Constraints From Existing Telemetry Excellence Work

The recent telemetry work established several constraints that should now be treated as architecture, not optional taste.

## 1. Bounded hot-read documents, not request-time reconstruction

The hot-read architecture explicitly moved Shuma toward bounded materialized documents because the expensive path was request-time assembly, especially on Fermyon:

1. monitoring bootstrap should read compact hot-read documents,
2. drill-down should remain secondary and bounded,
3. no new heavy read path should be introduced for operator summaries.

Implication:

1. the Monitoring overhaul should add or extend bounded summary documents,
2. not teach the dashboard to infer operator meaning from raw tails or Prometheus families at render time.

## 2. One shared telemetry architecture across targets

The repo has already rejected:

1. a Fermyon-only telemetry path,
2. a SQLite split,
3. and external database dependence as the default next move.

Implication:

1. any telemetry additions must fit the existing KV-backed, bounded summary model,
2. with deterministic rebuild or commutative update semantics that remain safe across targets.

## 3. Retained footprint is now dominated by hot-read documents and metadata, not only raw rows

The compact-row live evidence showed that raw rows got materially smaller, but the retained-byte footprint on the measured shared-host sample was still dominated by:

1. hot-read documents,
2. retention metadata,
3. and supporting indexes.

Implication:

1. future telemetry changes must be judged at the document and retained-tier level,
2. not only at the per-event-row level.

## 4. Monitoring must remain external-traffic-only

This is already corrected in the backend and should remain a hard rule.

Implication:

1. operator actions, contributor diagnostics, and future audit telemetry must not creep back into Monitoring's main evidence surfaces.

## 5. Exactness must stay explicit

The hot-read contract work already introduced exactness language and supporting component metadata.

Implication:

1. new operator summaries should either be exact,
2. or clearly labeled as derived/best-effort,
3. especially if they become inputs to future automated oversight.

# Current Telemetry Compared To The Desired Model

The desired operator-grade model from the recent research is:

1. lane-aware,
2. identity-aware,
3. action-aware,
4. friction-aware,
5. cost-aware,
6. explicit about policy source and exactness,
7. and cheap enough to serve repeatedly.

Below is where Shuma stands today.

## 1. Traffic lane summary

Desired:

1. likely human,
2. unknown interactive client,
3. suspicious automation,
4. declared crawler/search bot,
5. user-triggered assistant/fetcher,
6. verified bot,
7. signed agent,
8. adversary-sim traffic.

Current state:

1. partial only.
2. event rows carry `botness_score`, reasons, taxonomy, and sim metadata.
3. monitoring summaries are subsystem-oriented rather than lane-oriented.
4. there is no canonical coarse lane summary with total request denominators.

Assessment:

1. this is the single biggest gap.
2. without lane totals, operators cannot interpret friction rates, suspicious cost, or verified-agent impact.

Cost-aware recommendation:

1. add a coarse lane counter family and a bounded lane summary document,
2. do not attempt fine-grained identity-class analytics at per-request cardinality across every dimension yet.

## 2. Identity and verification telemetry

Desired:

1. claimed identity,
2. verification method,
3. verification result,
4. policy decision,
5. policy source.

Current state:

1. essentially absent in runtime telemetry today, because verified identity is not implemented yet.
2. some groundwork exists in design and research only.

Assessment:

1. this is a real future gap, but not a reason to distort current monitoring before the identity lane exists.

Cost-aware recommendation:

1. design the summary schema so identity slots can be added later,
2. but do not create fake placeholder summaries that imply runtime support before the verified-identity tranche lands.

## 3. Human-friction telemetry

Desired:

1. friction issuance rate for likely-humans,
2. pass/fail/escalate/abandon,
3. route/action family segmentation,
4. solve latency,
5. challenge escalation.

Current state:

1. partially strong.
2. `not_a_bot` and `pow` already carry the richest useful human-friction evidence.
3. challenge failure reasons are present.
4. likely-human denominators and route-family segmentation are still weak.

Assessment:

1. Shuma already has a good base here.
2. the main missing piece is denominator quality, not another friction subsystem.

Cost-aware recommendation:

1. add coarse total request and likely-human estimate counters by action family or route family,
2. reuse existing friction summaries rather than duplicating them in a new form.

## 4. Suspicious-traffic cost telemetry

Desired:

1. suspicious requests received,
2. suspicious requests forwarded,
3. suspicious bytes served,
4. approximate origin work,
5. expensive versus low-cost content profile usage,
6. cost-shifted tarpit or maze delivery.

Current state:

1. weak on operator summary, better in building blocks.
2. Prometheus already has forwarding metrics and some defence-specific byte or duration buckets.
3. Monitoring does not currently summarize host cost in a lane-aware way.

Assessment:

1. this is one of the two most important missing operator-facing families.
2. it matters directly to Shuma's project principles.

Cost-aware recommendation:

1. add a small number of coarse counters for forwarded requests, forwarded bytes, maze bytes, tarpit bytes, and low-cost content bytes by lane,
2. materialize them into one suspicious-cost summary,
3. avoid per-path or per-IP cost accounting in the first tranche.

## 5. Defence-effectiveness funnels

Desired:

Per defence family:

1. candidate traffic,
2. triggered traffic,
3. friction issued,
4. passes,
5. failures,
6. escalations,
7. denials,
8. post-defence suspicious traffic still forwarded,
9. likely-human traffic affected.

Current state:

1. partial and fragmented.
2. many of these facts exist by subsystem,
3. but they are not normalized into one comparable funnel contract.

Assessment:

1. this is a high-value gap, but it can be closed with summary shaping more than with brand-new event logging.

Cost-aware recommendation:

1. define one coarse funnel schema reused across defence families,
2. populate it from existing counters plus a small number of new denominator and forwarded-outcome counters.

## 6. Dimensioned drill-down

Desired:

1. country,
2. ASN,
3. route family,
4. hostname,
5. user-agent family,
6. identity class,
7. detection family,
8. verification source.

Current state:

1. partial.
2. top offenders, top paths, top countries, and some attack-specific breakdowns already exist.
3. ASN, hostname, coarse UA family, lane, and detection-family drill-down are not yet first-class.

Assessment:

1. this gap matters, but it is also where cost and cardinality risk rises fastest.

Cost-aware recommendation:

1. only add coarse bounded dimensions with explicit caps,
2. prefer normalized route family and UA family over raw path and raw user-agent string,
3. defer identity and verification dimensions until the verified-identity lane is real.

## 7. Misclassification and feedback telemetry

Desired:

1. suspected false positives,
2. suspected false negatives,
3. operator feedback,
4. later controller confidence inputs.

Current state:

1. mostly absent.
2. the closest existing signals are challenge success/abandon patterns and some likely-human evidence for IP-range policy.

Assessment:

1. this is valuable, but not first-wave telemetry.
2. it depends on the rest of the monitoring model becoming stable first.

Cost-aware recommendation:

1. reserve the schema and roadmap space,
2. do not prioritize this ahead of lane summaries, denominators, and suspicious-cost summaries.

## 8. Shadow-mode and enforced-mode truth

Desired:

1. explicit separation,
2. truthful would-have-enforced visibility during `shadow_mode`,
3. no misleading mixed-mode interpretation.

Current state:

1. groundwork is already good.
2. the event contract has `execution_mode`, `intended_action`, and `enforcement_applied`.
3. the missing work is summary shaping and UI ownership, not raw collection.

Assessment:

1. this is more a materialization and monitoring-surface problem than a telemetry-emission problem.

Cost-aware recommendation:

1. build shadow-mode summaries from the existing fields,
2. do not duplicate the event stream or attempt live paired counterfactual analytics.

# What Shuma Already Has That Should Be Preserved

Several existing telemetry qualities are worth defending explicitly.

## 1. Compact event rows

These are now good enough for drill-down and forensic context without being the primary operator surface.

Preserve:

1. compact sparse event rows,
2. no regression to verbose display-heavy canonical rows.

## 2. Bounded recent tails

Recent tails are good for drill-down and diagnostics.

Preserve:

1. tight caps,
2. explicit overflow signaling,
3. no temptation to widen them just to prop up Monitoring.

## 3. Freshness and retention health

This is now part of operator trust and future controller exactness.

Preserve:

1. explicit freshness state,
2. explicit retention health,
3. explicit boundedness diagnostics in Status or collapsed diagnostics.

## 4. Separate hot-read supporting documents

The split between:

1. monitoring summary,
2. retention summary,
3. security/privacy summary,
4. recent event tail,
5. recent sim runs,
6. and bootstrap envelope

is a good design foundation.

Preserve:

1. bounded documents with narrow responsibilities,
2. deterministic rebuild semantics.

# What Shuma Should Avoid Collecting In The Next Tranche

The new research could easily tempt the project into over-collecting. It should resist that.

## 1. Do not add raw full-request analytics for every request

Avoid:

1. logging every header family,
2. logging every candidate lane and all intermediate reasoning in raw event rows,
3. logging raw user-agent strings or high-cardinality path details beyond current normalized patterns.

## 2. Do not widen hot-read documents with everything operators might someday want

Avoid:

1. turning bootstrap into a second analytics warehouse,
2. stuffing every dimension and detail into the default hot-read document.

## 3. Do not use raw event tails as lane-denominator substitutes

Avoid:

1. estimating site-wide lane mix from a recent tail,
2. deriving suspicious-cost ratios from paged event windows.

## 4. Do not make Prometheus the primary source for dashboard operator meaning

Prometheus remains valuable, but the dashboard should still read bounded admin/hot-read summaries rather than assemble meaning from metric families at render time.

# Recommended Next Telemetry Tranche

The most efficient next tranche is a deliberately small one.

## 1. Add coarse denominator counters

Add bounded counters for:

1. total requests seen,
2. total forwarded requests,
3. total short-circuited requests,
4. total bytes forwarded or served,
5. coarse route or action family where justified.

These should be keyed by:

1. traffic lane,
2. execution mode,
3. and a very small action or route family taxonomy.

## 2. Add one lane summary hot-read document

This should answer:

1. what mix of traffic Shuma is seeing,
2. how much of it is suspicious,
3. how much likely-human traffic saw friction,
4. how much suspicious traffic still consumed host cost.

## 3. Add one suspicious-cost summary

This should aggregate:

1. forwarded suspicious requests,
2. forwarded suspicious bytes,
3. low-cost content versus full-content delivery by lane,
4. tarpit and maze cost-shift bytes.

## 4. Add one normalized defence-funnel summary

This should compare defence families in one common shape:

1. candidates,
2. triggers,
3. pass,
4. fail,
5. escalate,
6. deny,
7. suspicious forwarded after trigger,
8. likely-human affected.

## 5. Keep drill-down additions narrow

If a first dimension tranche is added, prefer:

1. route family,
2. country,
3. ASN,
4. UA family,
5. detection family.

Only if bounded and capped cleanly.

# Recommended Sequencing For `MON-OVERHAUL-1`

To stay aligned with the research and the efficiency work, the Monitoring overhaul should likely proceed in this order:

1. define the coarse lane taxonomy and denominator contract,
2. add the minimal new counters needed for lane, friction-rate, and suspicious-cost summaries,
3. materialize one bounded operator summary document,
4. redesign the Monitoring surface around that summary,
5. keep contributor diagnostics, retention, freshness, and raw feed details in their secondary surfaces,
6. only then consider a second tranche of bounded drill-down dimensions.

# Conclusion

Shuma does not need a telemetry rewrite.

It needs a disciplined translation from:

1. subsystem counters and compact event rows,
2. into lane-aware operator summaries with real denominators,
3. while preserving the bounded, cheap, hot-read-first architecture already established.

The cost-aware target is therefore:

1. add a little new collection,
2. add a lot of better summary meaning,
3. avoid high-cardinality temptation,
4. and keep retention/read costs visible as first-class constraints while Monitoring becomes much more operator-useful.
