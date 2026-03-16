Date: 2026-03-16
Status: Proposed

Related context:

- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](../research/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)

# Implementation Goals

1. Add deterministic banded ban jitter without changing the meaning of existing severity families.
2. Add a bounded local repeat-offender ladder for high-confidence ban families only.
3. Add optional central intelligence as an observe-first and advisory-first subsystem.
4. Make all three features visible to monitoring and available to the future oversight reconciler.

# Delivery Strategy

Implement in narrow slices. Each slice should leave the runtime truthful and keep documentation, tests, and operator visibility aligned.

# Phase 0: Contract And Observability Prep

## BI-0.1 Define canonical config surface

Add config placeholders and validation rules for:

1. global ban jitter enable toggle,
2. per-family jitter percentages,
3. recidive enable toggle,
4. recidive TTL and multipliers,
5. recidive family allowlist,
6. central-intelligence mode,
7. intelligence source registry shape,
8. confidence and action thresholds.

Acceptance:

1. `config/defaults.env` is the source of truth,
2. admin config and dashboard schema parity is preserved,
3. invalid overlap or unsafe combinations fail validation clearly.

## BI-0.2 Define telemetry contract

Add typed event and monitoring fields for:

1. effective ban duration derivation,
2. recidive step,
3. central-intelligence influence and provenance.

Acceptance:

1. monitoring truthfully distinguishes local-only from intelligence-influenced decisions,
2. no synthetic placeholder data.

# Phase 1: Banded Ban Jitter

## BI-1.1 Factor ban-duration derivation into one shared runtime helper

Replace scattered fixed-duration issue paths with one canonical duration resolver consumed by:

1. effect-intent bans,
2. challenge-abuse short bans,
3. tarpit persistence short bans,
4. provider-backed ban issue paths.

Acceptance:

1. no duplicate jitter math in separate modules,
2. existing family semantics still map to the same base durations before jitter.

## BI-1.2 Add deterministic percentage-band jitter

Implement a secret-keyed duration derivation helper that:

1. takes base duration and family band,
2. derives bounded jitter from stable issuance inputs,
3. clamps into the family band,
4. returns structured derivation details.

Acceptance:

1. durations are stable for a given issuance context,
2. attackers cannot infer a public formula,
3. bands never overlap across families.

## BI-1.3 Update monitoring and docs

Expose:

1. base vs effective duration,
2. jitter percentage,
3. distribution by family.

Acceptance:

1. operators can explain why a ban lasted as long as it did,
2. docs explain the intent in cost-shaping terms, not vague randomness terms.

# Phase 2: Repeat-Offender Ladder

## BI-2.1 Add bounded local recidive state primitive

Create a small TTL-backed recidive record separate from active bans.

Requirements:

1. keyed by site + offender bucket,
2. short expiry,
3. capped step,
4. lazy cleanup compatible with current storage backends.

Acceptance:

1. no active-ban retention coupling,
2. storage growth remains bounded.

## BI-2.2 Apply ladder only to high-confidence families

Wire the duration resolver to consult recidive state only for an allowlist of families.

Initial candidates:

1. `honeypot`
2. `ip_range_honeypot`
3. `cdp_automation`
4. `edge_fingerprint_automation`
5. `tarpit_persistence`

Acceptance:

1. `rate_limit` and generic challenge-abuse paths remain out unless explicitly promoted later,
2. manual unban clears recidive state.

## BI-2.3 Add safety evidence

Add monitoring counters for:

1. laddered bans by family,
2. recidive step distribution,
3. manual unban ratio for laddered bans.

Acceptance:

1. the oversight controller has enough evidence to decide whether the ladder should expand or contract.

# Phase 3: Central Intelligence Observe-Only

## BI-3.1 Define source and evidence contracts

Create explicit types for:

1. source id,
2. source class:
   - advisory CTI,
   - deny feed,
   - local federation,
3. freshness,
4. confidence,
5. scope,
6. provenance.

Acceptance:

1. no untyped JSON blobs standing in for source semantics,
2. sources are distinguishable in monitoring and logs.

## BI-3.2 Add ingest + cache pipeline

Implement bounded fetch and local caching for optional central sources.

Initial stance:

1. observe-only,
2. no enforcement changes,
3. monitoring and admin visibility only.

Acceptance:

1. freshness and fetch failure states are explicit,
2. source payload size and cardinality are bounded.

## BI-3.3 Add operator truth surfaces

Expose:

1. source freshness,
2. source confidence mix,
3. matched IP/range/ASN evidence counts,
4. disagreement and stale-feed diagnostics.

Acceptance:

1. operators can inspect what the intelligence layer is saying before it influences policy.

# Phase 4: Advisory Intelligence Influence

## BI-4.1 Add score-only and route-bias modes

Allow central intelligence to bias:

1. botness score,
2. lane classification,
3. route or challenge selection.

Do not allow community-style feeds to hard block traffic in this tranche.

Acceptance:

1. every influenced action records source provenance,
2. verified beneficial agents remain protected from coarse downgrade by default.

## BI-4.2 Add optional publish contract

Design and implement optional outbound reporting for approved high-confidence local observations.

Requirements:

1. opt-in only,
2. signed or authenticated where practical,
3. family allowlist,
4. explicit publication criteria.

Acceptance:

1. no rate-limit or low-confidence noise enters the shared channel,
2. publish failures degrade safely.

# Phase 5: High-Confidence Deny Feeds

## BI-5.1 Separate high-confidence deny feeds from advisory CTI

Introduce a distinct path for DROP-style feeds:

1. CIDR/ASN oriented,
2. strongly governed,
3. explicit operator enablement.

Acceptance:

1. advisory sources cannot silently inherit hard-block semantics,
2. high-confidence deny sources are surfaced distinctly in monitoring and docs.

## BI-5.2 Add bounded enforcement modes

Possible modes:

1. `deny_candidates`
2. `deny_high_confidence_ranges`

Acceptance:

1. each stronger mode requires explicit validation and operator documentation,
2. blast radius is measurable before promotion.

# Phase 6: Oversight Controller Integration

## BI-6.1 Add budget snapshot fields

Extend the future oversight snapshot to include:

1. expiry re-entry burst metrics,
2. jitter family distributions,
3. ladder false-positive proxies,
4. central-intelligence source health,
5. intelligence-influenced action rates.

## BI-6.2 Add proposal families

Allow the controller to propose narrow changes to:

1. jitter percentages,
2. recidive TTL,
3. recidive multipliers,
4. recidive family allowlist,
5. intelligence bias thresholds,
6. intelligence operating modes.

## BI-6.3 Add adversary verification paths

Adversary checks should prove:

1. jitter breaks rigid ban-expiry synchronization,
2. repeat-offender escalation does not inflate low-confidence false positives,
3. central intelligence enriches suspicious automation handling without harming verified beneficial agents.

# Testing Plan

## Unit

1. duration derivation is deterministic and band-safe,
2. recidive state expires and caps correctly,
3. intelligence source parsing and freshness logic are strict.

## Integration

1. repeated eligible bans escalate only when family and TTL rules permit,
2. manual unban clears recidive state,
3. advisory intelligence changes score/routing but not hard enforcement in advisory mode.

## Dashboard and admin

1. monitoring truthfully shows jittered durations and recidive steps,
2. central-intelligence source state and provenance render correctly,
3. oversight-related docs and operator surfaces stay in sync.

## Adversary simulation

1. repeated attack cohorts no longer align on a fixed unban rhythm,
2. repeated high-confidence offenders escalate measurably faster than first offenders,
3. central intelligence affects suspicious automation cohorts while leaving verified-agent cohorts healthy.

# Documentation Plan

Update:

1. `docs/configuration.md`
2. `docs/api.md`
3. `docs/dashboard-tabs/monitoring.md`
4. `docs/dashboard-tabs/ip-bans.md`
5. `docs/adversarial-operator-guide.md`
6. `docs/value-proposition.md`
7. `docs/testing.md`

# Security And Operational Notes

1. Shared intelligence introduces poisoning and stale-data risk; provenance and freshness are mandatory.
2. Hard-block community intelligence is a later step, not a first step.
3. Jitter and recidive state must remain bounded in storage and cheap to compute.
4. Verified beneficial-agent handling must remain explicit throughout rollout.

# Suggested Rollout Sequence

1. Phase 0 and Phase 1
2. Phase 2
3. Phase 3
4. Phase 4
5. Phase 6
6. Phase 5 only after evidence justifies stronger shared enforcement
