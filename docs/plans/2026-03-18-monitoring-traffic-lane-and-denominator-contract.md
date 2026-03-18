Date: 2026-03-18
Status: Settled and implemented foundation contract

Related context:

- [`2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](./2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](./2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`../research/2026-03-17-operator-decision-support-telemetry-audit.md`](../research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](./2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_facts.rs`](../../src/runtime/request_facts.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/config/mod.rs`](../../src/config/mod.rs)
- [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define the concrete contract for the first telemetry-foundation prerequisite:

1. the canonical operator-facing traffic-lane model,
2. the denominator boundary for Monitoring,
3. the minimal route and action-family grouping that stays within cost guardrails,
4. and the rule that operators tune the underlying routing and scoring controls, not a second lane-specific scoring system.

This note is intentionally a design contract, not the implementation plan for `MON-TEL-1-1`.

# Objectives

1. Give Monitoring one stable, reusable traffic-lane vocabulary.
2. Keep that vocabulary aligned with Shuma's actual runtime decision model.
3. Make human-friction rates and suspicious-cost summaries share one denominator boundary.
4. Preserve future room for verified bots, signed agents, and declared user-triggered fetchers without forcing fake support before those subsystems exist.
5. Keep the contract cheap enough for bounded hot-read summaries.

# Non-goals

1. Introducing a second independent scoring system just for Monitoring.
2. Treating lane labels as ground-truth statements about personhood or intent.
3. Widening raw event tails or adding a new analytics subsystem.
4. Forcing verified-identity or crawler-classification support to exist before those tranches land.
5. Mixing adversary-sim traffic into the same primary live-traffic denominator.

# Design Principles

## 1. Lanes are interpretation, not a second control plane

The canonical traffic lane is an operator-facing interpretation layer built from the same runtime facts and policy thresholds that already drive routing.

That means:

1. operators tune existing thresholds, weights, and policy controls,
2. Monitoring shows how traffic moves between lanes as a result,
3. and Shuma must not add separate "lane thresholds" that drift away from the real runtime model.

Examples of the real underlying controls that remain operator-tunable:

1. `not_a_bot_risk_threshold`
2. `not_a_bot_pass_score`
3. `not_a_bot_fail_score`
4. `challenge_puzzle_risk_threshold`
5. `botness_maze_threshold`
6. `botness_weights.*`
7. `rate_limit`
8. `geo_*`
9. `defence_modes.*`
10. `js_required_enforced`
11. `cdp_detection_threshold`
12. `edge_integration_mode`
13. later, verified-identity authorization policy

## 2. Traffic origin is a separate dimension from traffic lane

Adversary-sim traffic should not become a first-class lane alongside human and non-human traffic.

Instead:

1. `traffic_lane` describes what kind of client behavior or identity the request represents,
2. `traffic_origin` describes whether the request is live or adversary-sim.

Canonical origin dimension:

1. `live`
2. `adversary_sim`

This keeps simulation visible without muddying live operator denominators.

## 3. Monitoring needs a primary denominator and several secondary scopes

Not every inbound request should count the same way in operator traffic-mix charts.

Monitoring therefore needs:

1. a primary ingress denominator for "what traffic is hitting the site",
2. secondary follow-up scopes for challenge and trap flows,
3. explicit exclusions for control-plane and contributor-only surfaces.

# Canonical Traffic-Lane Contract

## Lane enum

The canonical operator-facing lane enum should be:

1. `likely_human`
2. `unknown_interactive`
3. `suspicious_automation`
4. `declared_crawler`
5. `declared_user_triggered_agent`
6. `verified_bot`
7. `signed_agent`

This is intentionally future-capable. It gives Shuma the vocabulary it needs for the agentic era without forcing all lanes to be populated in phase 1.

## Current-runtime support posture

Phase-1 support should be honest:

1. `likely_human`
   - supported now
2. `unknown_interactive`
   - supported now
3. `suspicious_automation`
   - supported now
4. `declared_crawler`
   - reserved until Shuma has an explicit inbound crawler classification contract
5. `declared_user_triggered_agent`
   - reserved until verified identity or an equivalent request classification contract exists
6. `verified_bot`
   - reserved until verified-bot and Web Bot Auth work lands
7. `signed_agent`
   - reserved until RFC 9421 or provider-signed-agent verification lands

Reserved does not mean "invent a placeholder count." It means the lane name is part of the canonical vocabulary, but summaries must explicitly report it as unavailable until the relevant subsystem exists.

# Lane Meaning And Exactness

The lane label is always an operational classification, not metaphysical truth.

The formal operator-summary contract should express this as two dimensions:

1. `exactness`
   - `exact`
   - `derived`
   - `best_effort`
2. `basis`
   - `observed`
   - `policy`
   - `verified`
   - `residual`
   - `mixed`

This document uses that two-part language directly so implementation does not have to guess later.

## `likely_human`

Meaning:

1. requests with direct evidence of recent human verification or similarly strong local proof,
2. and no stronger contradictory evidence that should demote the request into a suspicious lane.

Current strongest evidence:

1. `not_a_bot_marker_valid`
2. successful human-verification follow-up that results in the clean request carrying valid marker state

Exactness posture:

1. `exact`
2. basis `observed`
3. never a claim of ground-truth humanity

## `unknown_interactive`

Meaning:

1. requests that appear to be public-site interactive traffic,
2. but do not yet have strong positive human proof,
3. and do not yet trip strong suspicious or declared/verified non-human classification.

Examples:

1. clean public requests without human-proof marker
2. traffic challenged only by GEO or JS posture
3. static-asset bypass requests where Shuma intentionally avoids expensive bot checks

Exactness posture:

1. `derived`
2. basis `residual`
3. this is intentionally a residual operational bucket, not a confident identity statement

## `suspicious_automation`

Meaning:

1. requests with strong hostile or automation evidence,
2. or requests whose score and signal composition clearly place them on the non-human side of Shuma's current runtime policy model.

Current evidence examples:

1. honeypot hit
2. existing ban
3. rate-limit exceeded
4. botness score at or above challenge or maze thresholds
5. later, strong CDP and fingerprint enforcement signals
6. later, authoritative edge verdicts

Exactness posture:

1. `exact` with basis `policy` when driven by high-confidence direct signals or explicit enforcement state
2. `derived` with basis `residual` when driven by thresholded score composition rather than direct single-signal certainty

## `declared_crawler`

Meaning:

1. requests that identify as crawler or search automation,
2. where classification is explicit enough to separate them from suspicious undeclared automation,
3. but without the stronger trust of verified identity.

Phase-1 posture:

1. reserved

Future exactness posture:

1. `exact`
2. basis `observed`
3. not based on naive user-agent substring matching alone

## `declared_user_triggered_agent`

Meaning:

1. requests attributable to user-triggered assistant or fetcher behavior,
2. distinct from autonomous crawl or indexing traffic.

Phase-1 posture:

1. reserved

Future exactness posture:

1. `exact`
2. basis `observed` or `verified` depending on verification method

## `verified_bot`

Meaning:

1. requests whose bot identity is authenticated and locally authorized under verified-bot policy.

Phase-1 posture:

1. reserved

Future exactness posture:

1. `exact`
2. basis `verified`

## `signed_agent`

Meaning:

1. requests whose agent identity is cryptographically authenticated, for example via HTTP Message Signatures and Web Bot Auth-compatible verification.

Phase-1 posture:

1. reserved

Future exactness posture:

1. `exact`
2. basis `verified`

# Denominator Contract

## Primary operator denominator: `ingress_primary`

This is the denominator used for:

1. traffic-mix summaries
2. suspicious-forwarded rate
3. likely-human friction rate
4. top-level Monitoring charts and operator budgets

`ingress_primary` should include public traffic requests that represent a real attempt to fetch site content or assets.

Included classes:

1. normal public requests that reach policy evaluation
2. static-asset bypass requests
3. public requests that are ultimately forwarded
4. public requests that are ultimately blocked, challenged, redirected, mazed, tarpitted, or otherwise short-circuited

Excluded classes:

1. admin and control-plane endpoints
2. health and metrics endpoints
3. fingerprint reporting endpoints
4. challenge submit and proof submit follow-up endpoints
5. maze traversal, checkpoint, issue-links, and tarpit progress endpoints
6. explicit sim-public helper endpoints
7. requests rejected before Shuma can reasonably treat them as monitored site traffic, such as environment/bootstrap failure responses

## Secondary scope: `defence_followup`

This scope covers follow-up requests generated by defence workflows rather than original site-ingress attempts.

Included classes:

1. not-a-bot follow-up
2. challenge puzzle follow-up
3. proof-of-work verification
4. maze traversal pages
5. maze checkpoint and link-issuance traffic
6. tarpit progress traffic

This scope is essential for subsystem funnels and operator drill-down, but it must not inflate top-level ingress denominators.

## Secondary scope: `bypass_and_control`

This scope records traffic that is meaningful operationally but should not pollute the primary Monitoring narrative.

Included classes:

1. path allowlist bypass
2. IP allowlist bypass
3. internal control-plane requests
4. admin and operational endpoints

These counters remain useful for Status and supporting diagnostics.

# Minimal Route And Action-Family Grouping

To stay within cardinality guardrails, Monitoring should use only the following coarse route or action families in the new operator summaries:

1. `public_content`
2. `static_asset`
3. `defence_followup`
4. `allowlist_bypass`
5. `control_plane`
6. `sim_public`

This grouping is enough to:

1. prevent challenge and trap follow-ups from polluting ingress denominators
2. preserve static-asset cost visibility
3. keep allowlist traffic visible without muddying public-traffic tuning
4. keep internal and sim helper traffic out of the primary operator story

Important clarification:

1. `sim_public` is a fallback content-surface provenance for development and demo scenarios, not a distinct long-term traffic lane concept.
2. The meaningful monitoring boundary is `traffic_origin` (`live` vs `adversary_sim`), not whether synthetic traffic traversed fallback `/sim/public/*` pages or a real hosted public surface.
3. Until origin-aware operator summaries land, Shuma must not let adversary-sim traffic feed live-only inference signals such as clean-allow or likely-human evidence.

# Current-Runtime Branch Mapping Guardrail

Before `MON-TEL-1-1` is implemented in code, Shuma must lock one deterministic mapping table from today's actual terminal branches to:

1. `measurement_scope`
2. `route_action_family`
3. `traffic_lane`
4. `policy_source`

The current recommended starting matrix is:

| Current branch family | Measurement scope | Route or action family | Lane guidance | Policy source |
| --- | --- | --- | --- | --- |
| `/health`, dashboard redirect, admin or control-plane early routes, maze assets | `excluded` | `control_plane` | lane omitted | `early_route` |
| static bypass | `ingress_primary` only if a cheap telemetry-safe path is approved; otherwise not exact primary ingress yet | `static_asset` | `unknown_interactive` when counted | `static_asset_bypass` |
| path or IP allowlist bypass | `bypass_and_control` | `allowlist_bypass` | lane omitted from primary mix | `allowlist_bypass` |
| first-tranche terminal suspicious decisions such as `HoneypotHit`, `RateLimitHit`, `ExistingBan`, and enforcing IP-range actions other than advisory or emergency allowlist | `ingress_primary` | `public_content` | `suspicious_automation` | `policy_graph_first_tranche` |
| GEO challenge, GEO maze, GEO block, and JS challenge | `ingress_primary` | `public_content` | `unknown_interactive` unless stronger contradictory evidence exists | `policy_graph_second_tranche` |
| botness-driven not-a-bot, challenge, challenge fallback, or maze | `ingress_primary` | `public_content` | `suspicious_automation` | `policy_graph_second_tranche` |
| clean public allow after policy evaluation | `ingress_primary` | `public_content` | `likely_human` when direct human-proof evidence exists, otherwise `unknown_interactive` | `clean_allow` |
| challenge follow-up, proof-of-work, maze traversal, checkpoint, issue-links, and tarpit progress | `defence_followup` | `defence_followup` | lane inherited where the runtime can do so cheaply, otherwise omitted from primary mix | `defence_followup` |
| sim-public helper endpoints | `excluded` from primary live ingress | `sim_public` | lane omitted from primary mix | `sim_public` |

This table is deliberately grouped rather than path-by-path. The implementation must still cover every existing terminal branch, but it must do so by mapping each branch into one of these grouped rows rather than inventing new local categories.

The `sim_public` row must be read narrowly:

1. it exists because Shuma currently has a fallback dummy public surface for contributors and evaluators who do not yet have a real hosted site behind the gateway,
2. it must not be interpreted as meaning that adversary-sim traffic against those fallback pages deserves fundamentally different defence treatment,
3. and future operator summaries should prefer origin-aware separation over path-based special casing once `MON-TEL-1` introduces those summaries.

# Lane Derivation Order

Lane derivation should happen in a stable order so the semantics remain predictable.

## Step 1: Determine origin and scope

Before assigning a lane, determine:

1. `traffic_origin`
2. `measurement_scope`
3. `route_action_family`

If the request is outside `ingress_primary`, it must not be counted in top-level operator denominators even if it still records subsystem telemetry elsewhere.

## Step 2: Apply verified-identity lanes first

Once verified identity exists, it should win first:

1. `signed_agent`
2. `verified_bot`

These are stronger and more explicit than score-based heuristics.

## Step 3: Apply declared-non-human lanes

Once Shuma has a proper declared-crawler and user-triggered-fetcher classifier, those lanes should be considered before residual score-based interpretation.

This keeps "declared but locally restricted" separate from "undeclared and suspicious."

## Step 4: Apply strong human-proof lane

If the request carries direct human-proof evidence such as valid not-a-bot marker state, classify as `likely_human` unless a stronger explicit contradictory signal requires demotion.

## Step 5: Apply suspicious-automation lane

If the request has strong suspicious evidence or crosses runtime thresholds that currently trigger bot-directed routing, classify as `suspicious_automation`.

This includes:

1. high-confidence direct suspicious signals
2. thresholded botness outcomes
3. active enforcement states such as existing ban

## Step 6: Residual interactive lane

If none of the above apply and the request is in `ingress_primary`, classify as `unknown_interactive`.

This is intentionally the residual operational lane for site traffic that is neither confidently human nor strongly suspicious.

# Relationship To Tuning

The operator should be able to tune traffic movement between lanes, but only by adjusting the real underlying runtime controls.

That means:

1. the lane contract does not introduce new lane thresholds
2. the Tuning tab should expose the real scoring and routing thresholds that influence lane movement
3. Monitoring should show when those changes move too much traffic into or out of `likely_human`, `unknown_interactive`, or `suspicious_automation`

Examples of how tuning should interact with lanes:

1. if too much suspicious traffic is still forwarded, the operator may lower `challenge_puzzle_risk_threshold`, lower `not_a_bot_risk_threshold`, raise relevant `botness_weights.*`, or tighten later identity policy
2. if likely-human friction is too high, the operator may raise `not_a_bot_risk_threshold`, adjust pass or fail scores, reduce certain weights, or change defence modes and risk-country routing
3. if a high volume of requests stays in `unknown_interactive`, that can indicate a need for stronger classification inputs or better thresholds, not a need for a separate lane-specific scoring system

# Implementation Guidance For `MON-TEL-1-1`

`MON-TEL-1-1` should treat this note as the contract to implement.

The expected implementation shape is:

1. add a shared runtime-owned traffic-lane domain object
2. add a shared `measurement_scope` or equivalent classification alongside it
3. make telemetry emission depend on those shared classifications rather than ad hoc per-branch rules
4. expose per-lane availability and exactness honestly so future reserved lanes do not masquerade as implemented

It should not:

1. add fake placeholder counts for verified or signed lanes
2. classify declared crawlers from naive UA matching alone
3. use defence-follow-up requests as top-level ingress denominator
4. add a second set of lane-specific thresholds

# Definition Of Done

This prerequisite is complete when:

1. Shuma has one canonical traffic-lane vocabulary,
2. Shuma has one explicit denominator boundary for top-level operator Monitoring,
3. defence follow-up flows are separated from primary ingress denominators,
4. simulation is tracked as origin, not as a primary lane,
5. and the operator tuning story remains "change the real policy knobs, observe the lane movement" rather than "tune a second analytics-only scoring model."
