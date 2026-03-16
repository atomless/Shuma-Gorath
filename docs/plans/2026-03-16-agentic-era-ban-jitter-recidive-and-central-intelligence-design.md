Date: 2026-03-16
Status: Proposed

Related context:

- [`../research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`../project-principles.md`](../project-principles.md)
- [`../module-boundaries.md`](../module-boundaries.md)

# Objectives

1. Make Shuma's ban behavior harder for hostile automation to predict or synchronize against.
2. Escalate repeated local abuse progressively without introducing broad false-positive compounding.
3. Add optional central intelligence in a way that improves defender memory without creating opaque community auto-bans.
4. Keep all request-path decisions deterministic and Rust-owned.
5. Expose all three features to the bounded oversight controller as tunable policy families.

# Non-goals

1. LLMs deciding individual request outcomes.
2. Replacing local evidence with central reputation.
3. Treating all bots as one class.
4. Global community auto-banning of arbitrary single IPs in the first tranche.
5. Adding sticky post-expiry suspicion as part of the initial rollout.
6. Keeping backward-compatibility shims for old ban contracts in this pre-launch stage.

# Architectural Positioning

These three features should be treated as one coordinated subsystem with three horizons.

## 1. Ban Jitter

Horizon:

1. immediate request-plane actuation.

Purpose:

1. break exact-expiry predictability,
2. spread re-entry load,
3. preserve severity meaning.

## 2. Repeat-Offender Ladder

Horizon:

1. short-lived local recidive memory.

Purpose:

1. make repeated local abuse progressively more expensive,
2. without carrying full long-term historical state.

## 3. Central Intelligence

Horizon:

1. medium-lived shared memory and enrichment.

Purpose:

1. improve site-local decision quality with outside reputation or deny feeds,
2. while keeping source trust and blast radius explicit.

## 4. Oversight Controller

Horizon:

1. long-lived budget tuning and policy adaptation.

Purpose:

1. tune jitter bands,
2. ladder windows and caps,
3. central-intelligence modes and confidence thresholds,
4. and roll back regressions.

# Traffic-Lane Interaction Model

All three features must preserve Shuma's three-lane model.

## Lane A: Verified Beneficial Agents

Examples:

1. cryptographically verified agents,
2. signed requests,
3. authenticated or otherwise high-confidence beneficial traffic.

Rules:

1. central intelligence must not override verified identity by default,
2. jitter and repeat-offender escalation should only apply when the agent violates local policy,
3. beneficial agents should remain eligible for cheaper, structured responses.

## Lane B: Declared Crawlers and Search Bots

Rules:

1. robots and AI preference surfaces remain advisory inputs,
2. central intelligence can bias risk or rate posture,
3. repeat-offender logic should be conservative unless the actor also violates declared behavior locally.

## Lane C: Unverified or Suspicious Automation

Rules:

1. jitter, repeat-offender escalation, and central intelligence all matter here,
2. but local evidence should still outrank shared reputation for stronger enforcement.

# Core Design

## A. Banded Ban Jitter

### Design

Each ban family keeps a base duration.

The effective duration is:

1. base family duration,
2. multiplied by any eligible repeat-offender factor,
3. then adjusted by a bounded percentage jitter,
4. then clamped into the non-overlapping family band.

### Why percentage bands

Percentage bands scale with duration families naturally and are easier for the oversight controller to tune consistently than separate per-family min/max values.

### Why deterministic secret-keyed jitter

The duration should be derived from stable issuance inputs such as:

1. site id,
2. principal bucket,
3. ban family,
4. ban reason,
5. issue timestamp bucket or operation id,
6. recidive step,
7. policy epoch.

The derivation should use a Shuma secret so:

1. operators can reproduce the result,
2. attackers cannot predict it.

### Family constraints

Bands must not overlap. Example shape:

1. short-ban family remains strictly below rate-limit family,
2. rate-limit remains strictly below admin/CDP,
3. honeypot remains the strongest normal static family unless explicitly superseded.

### Telemetry

Ban events should expose:

1. `ban_family`
2. `base_duration_seconds`
3. `repeat_offender_multiplier`
4. `jitter_percent`
5. `effective_duration_seconds`
6. `duration_derivation_version`

## B. Repeat-Offender Ladder

### Design

Store a tiny separate recidive record:

1. keyed by site and offender bucket,
2. containing step/count,
3. containing expiry,
4. capped at a maximum step.

This record is consulted only when a new eligible ban is about to be issued.

### Bucket model

Default:

1. IP bucket.

Optional refinement for specific high-confidence families:

1. fingerprint-backed bucket refinement for strong CDP or verified edge fingerprint bans.

The first tranche should default to IP bucket only.

### Eligible families

Default first-tranche candidates:

1. `honeypot`
2. `ip_range_honeypot`
3. `cdp_automation`
4. `edge_fingerprint_automation`
5. `tarpit_persistence`

Not first-tranche candidates:

1. plain `rate_limit`
2. challenge-submit short bans
3. manual admin bans

### Safety guardrails

1. short recidive TTL,
2. capped multiplier ladder,
3. manual unban clears recidive state,
4. family allowlist,
5. optional requirement for multi-signal corroboration before promoting lower-confidence families,
6. operator-visible ladder state in monitoring.

### Why this avoids false-positive compounding

Because the ladder is:

1. not universal,
2. not indefinite,
3. not hot-path suspicion,
4. and not driven by community signals alone.

## C. Central Intelligence

### What central intelligence is not

Central intelligence is not deployment-local active ban synchronization.

Shuma should keep these as separate systems:

1. edge-instance ban sync
   - one site or deployment,
   - exact current active bans,
   - fast convergence,
   - synchronized unban and expiry,
   - operational correctness;
2. central intelligence
   - cross-site or fleet memory,
   - advisory reputation or high-confidence deny feeds,
   - broader retention horizon,
   - governance and false-positive process,
   - reputation or deny-candidate enrichment.

This distinction matters because:

1. active ban sync is an authoritative mirror of what one deployment has already decided,
2. central intelligence is a separate evidence source that may influence a later local decision,
3. and local unban semantics should not be coupled automatically to fleet or community reputation state.

### Design split

Central intelligence should be modeled as two classes, not one.

#### Class 1: Advisory Reputation Feeds

Examples:

1. CrowdSec-style CTI or reputation data,
2. optional community reports,
3. site-shared risk observations.

Default effects:

1. score bias,
2. lane bias,
3. routing bias,
4. challenge bias,
5. deny-candidate generation only.

#### Class 2: High-Confidence Deny Feeds

Examples:

1. Spamhaus DROP-style worst-of-the-worst range feeds,
2. explicitly curated range or ASN deny sets.

Default effects:

1. explicit deny or hard block eligibility,
2. with tighter governance and stronger operator surfacing.

### Interaction with edge-instance ban sync

The clean interaction model is:

1. central intelligence may bias or inform a local decision,
2. once Shuma makes a local ban decision for a site, that active ban is propagated through the deployment-local ban-sync mechanism,
3. but the local active ban record and the central intelligence record remain distinct objects with distinct TTL, governance, and removal workflows.

Examples:

1. a worst-offender feed may make a request more likely to be challenged or denied locally,
2. but it should not be treated as "already banned on this site" until the local site actually issues a ban,
3. and a local manual unban should clear the site's synced active ban without silently deleting a fleet-level worst-offender record.

### Central intelligence operating modes

1. `off`
2. `observe`
3. `score_only`
4. `route_bias`
5. `deny_candidates`
6. `deny_high_confidence_ranges`

The first tranche should stop at `score_only` or `route_bias` for community-style feeds.

### Governance requirements

Every intelligence source needs:

1. source id,
2. signed or authenticated fetch where practical,
3. freshness timestamp,
4. confidence class,
5. scope class:
   - ip,
   - nearby range,
   - CIDR,
   - ASN,
   - declared bot identity,
6. appeal or removal posture,
7. operator-visible provenance.

### Reporting model

Shuma instances may optionally publish back only the highest-confidence local observations, and only from explicitly approved families.

Initial publication candidates:

1. repeated honeypot hits,
2. strong CDP automation bans,
3. tarpit persistence abuse once tightened,
4. optionally declared-bot policy violation evidence.

Initial non-candidates:

1. raw rate-limit bans,
2. one-off challenge failures,
3. anything lacking clear confidence or collateral-risk analysis.

# Oversight Controller Fit

The oversight controller should tune these features, not replace them.

## Auto-tunable families

1. per-family jitter percentages,
2. recidive TTL,
3. recidive multipliers,
4. family allowlist for recidive,
5. central-intelligence mode,
6. central-intelligence confidence thresholds,
7. route-bias and challenge-bias weights.

## Manual-only surfaces

1. source-trust anchors,
2. source credentials and signing roots,
3. hard allowlists,
4. operator identity and auth boundaries,
5. global deny feed enrollment for high-risk production use.

## Controller workflow

1. read one budget snapshot,
2. assess whether ban-wave risk, false-positive risk, or bot-cost budgets are breached,
3. propose a narrow patch,
4. validate,
5. run adversary checks,
6. canary,
7. watch,
8. rollback if necessary.

# Monitoring And Budget Contract

## New metrics and evidence

### Ban jitter

1. expiry re-entry burst histogram,
2. ban issue distribution by family,
3. mean and p95 effective duration by family,
4. jitter band utilization.

### Repeat-offender ladder

1. bans by recidive step,
2. manual unban ratio for laddered bans,
3. ladder-triggered ban count by family,
4. ladder state cardinality and TTL health.

### Central intelligence

1. matches by source and confidence,
2. actions influenced by central intelligence,
3. freshness and fetch failures,
4. source disagreement rates,
5. appeal/removal and false-positive counters where applicable.

## Key oversight budgets

1. human friction budget,
2. suspicious traffic cost budget,
3. beneficial-agent success budget,
4. telemetry truthfulness budget,
5. central-intelligence poisoning or disagreement budget.

# Security And Abuse Considerations

1. Do not let shared reputation automatically escalate signed beneficial agents.
2. Do not let community feeds create hard blocks for arbitrary single IPs in the first tranche.
3. Prefer CIDR/ASN hard enforcement only for explicitly high-confidence feeds.
4. Keep recidive storage short-lived and bounded.
5. Make central-intelligence fetch and publish channels authenticated and auditable.
6. Record source provenance on every influenced action.
7. Treat poisoning, stale feed drift, and over-broad community collateral as first-class risks.

# Recommended Rollout Posture

1. Ship banded jitter first.
2. Add repeat-offender ladder only for high-confidence families.
3. Add central intelligence in observe-only and score-only modes first.
4. Let the oversight controller tune these families only after their monitoring surfaces are truthful.

# Design Call Summary

1. Ban jitter should be percentage-banded, secret-keyed, deterministic, and non-overlapping by severity family.
2. Repeat-offender logic should be a local recidive primitive, not sticky suspicion.
3. Central intelligence should start as advisory enrichment, with hard-block behavior reserved for explicitly higher-confidence feeds.
4. The controller should tune the envelopes, never make per-request decisions.
