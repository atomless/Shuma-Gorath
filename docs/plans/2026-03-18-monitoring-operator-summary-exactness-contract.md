Date: 2026-03-18
Status: Settled and implemented foundation contract

Related context:

- [`2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](./2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](./2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](./2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`../research/2026-03-17-operator-decision-support-telemetry-audit.md`](../research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../../src/observability/hot_read_contract.rs`](../../src/observability/hot_read_contract.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define the concrete exactness contract for the new operator summaries that `MON-TEL-1` will add.

This note exists because Shuma's current hot-read contract distinguishes only:

1. `exact`
2. `best_effort`

That is no longer expressive enough for the Monitoring-overhaul telemetry foundation. The next tranche needs to distinguish:

1. what is exact,
2. what is derived but still useful and truthful,
3. what is best-effort because the canonical source is mutable or lossy,
4. and what kind of basis the summary rests on.

# Objectives

1. Give operator summaries explicit exactness semantics.
2. Separate exactness from evidentiary basis so Monitoring can stay honest without becoming vague.
3. Make future controller and tuning consumers treat derived summaries as informative, not as ground-truth facts.
4. Keep the contract small enough to fit naturally into the existing hot-read metadata model.

# Non-goals

1. Creating a giant ontology for every metric family.
2. Reclassifying every existing hot-read component immediately.
3. Pretending that every operator summary can or should become exact.
4. Turning summary exactness into a runtime request-path concern.

# Core Design

## Exactness and basis are separate dimensions

Operator summaries must declare two different properties:

1. `exactness`
2. `basis`

Those are not interchangeable.

`exactness` answers:

1. how strong the summary is as a measurement claim
2. whether the number is exact, derived, or best-effort

`basis` answers:

1. what kind of evidence or reasoning produced the classification
2. whether the result came from observation, policy, verification, residual inference, or a mixture

This separation keeps Shuma from collapsing all nuance into one overloaded enum.

## Canonical exactness enum

The future hot-read and operator-summary contract should use:

1. `exact`
2. `derived`
3. `best_effort`

### `exact`

Meaning:

1. the summary is a truthful exact count or state projection from its canonical source
2. Shuma is not inferring the core classification from leftovers or heuristics at read time

Examples:

1. recent event rows rebuilt from immutable event log
2. verified-identity counts once verified-identity lanes exist
3. exact current runtime posture derived from direct runtime/config state

### `derived`

Meaning:

1. the summary is deterministic and useful,
2. but the classification involves deliberate inference or interpretation rather than direct exact observation of the claimed semantic bucket

Examples:

1. `unknown_interactive`
2. some `suspicious_automation` counts driven by thresholded score composition rather than a single decisive signal
3. residual operator groupings that are intentionally policy-shaped interpretations of runtime truth

This is the missing middle state Shuma now needs.

### `best_effort`

Meaning:

1. the summary is bounded and useful,
2. but the canonical source or projection path is not strong enough to support exact or deterministic-derived claims across all deployment targets

Examples:

1. summaries sourced from shared mutable counters on non-atomic edge KV
2. retention health summaries assembled from mutable worker catalogs
3. any supporting summary that must tolerate bounded lossiness by design

## Canonical basis enum

The future operator-summary contract should also use:

1. `observed`
2. `policy`
3. `verified`
4. `residual`
5. `mixed`

### `observed`

Meaning:

1. classification rests on direct runtime-observed evidence
2. for example valid human marker or explicit declared-bot signal once that exists

### `policy`

Meaning:

1. classification rests on explicit local policy state or routing outcome
2. for example existing ban, honeypot enforcement, or locally authorized verified-bot treatment

### `verified`

Meaning:

1. classification rests on cryptographic or equivalent strong identity verification
2. this is reserved for later verified-bot and signed-agent work

### `residual`

Meaning:

1. classification is intentionally the remaining operational bucket after stronger identities and policies are accounted for
2. this is the right basis for `unknown_interactive`

### `mixed`

Meaning:

1. the summary merges groups whose basis differs
2. this should be used sparingly and documented clearly

# Contract Shape

## Extend the hot-read component contract rather than inventing a parallel metadata scheme

The existing `HotReadComponentContract` should evolve rather than being bypassed.

Recommended future shape:

1. `key`
2. `exactness`
3. `basis`
4. `canonical_source`
5. `projection_model`
6. `note`

That keeps summary truth metadata in one canonical place.

## Summary-level exactness is about the claimed summary, not every raw constituent

The exactness and basis declared for a summary apply to the semantic claim of the summary itself.

This matters because:

1. a summary can be exact even if it is composed from many rows,
2. and a summary can be derived even when all of its raw ingredients are exact.

Example:

1. an `unknown_interactive` lane count can be deterministically computed from exact request outcomes
2. yet the lane classification is still `derived` because the lane itself is an intentional operational inference

# Required Usage Rules

## Rule 1: every new operator summary must declare exactness and basis explicitly

`MON-TEL-1-5` and later Monitoring-overhaul work must not add a new operator summary without:

1. exactness
2. basis
3. canonical source
4. projection model
5. a short note explaining the claim boundary

## Rule 2: derived is not a downgrade label

`derived` must not be treated as an error or a shame bucket.

It means:

1. the summary is intentional and deterministic
2. but the semantic claim is interpretive rather than exact observation

That is often the correct posture for operator decision support.

## Rule 3: controllers and future auto-tuning consumers must not treat `derived` as `exact`

The scheduled oversight/controller work may consume derived summaries, but must do so as bounded signals rather than hard truth.

Practical implication:

1. exact verified-identity counts can gate strong policy choices
2. derived suspicious-lane proportions can guide tuning or alerting
3. best-effort summaries must not be treated as sole hard gates for autonomous action

## Rule 4: mixed-basis summaries must stay secondary unless the composition is obvious

If a summary combines different evidentiary bases, it should normally be:

1. a supporting summary,
2. or accompanied by a note that the headline mixes verified, policy-driven, and residual interpretations

Top-level Monitoring should prefer cleanly interpretable summaries where possible.

# Initial Classification Guidance

The following table should guide `MON-TEL-1` and later work.

| Summary family | Exactness | Basis | Why |
| --- | --- | --- | --- |
| runtime posture summary | `exact` | `policy` | Direct current config/runtime state, not accumulated telemetry |
| recent events tail | `exact` | `observed` | Immutable event log rebuild |
| recent sim runs summary | `exact` | `observed` | Immutable event-log-derived compact run history |
| `likely_human` lane count | `exact` when backed by direct human-proof evidence only; otherwise do not promote weaker evidence into this lane | `observed` | Marker-backed or equivalent direct local proof |
| `unknown_interactive` lane count | `derived` | `residual` | Intentional residual operator bucket |
| direct-policy suspicious counts such as existing ban or honeypot-driven terminal outcomes | `exact` | `policy` | Explicit local policy or enforcement state |
| score-threshold suspicious counts that group residual score outcomes | `derived` | `residual` or `mixed` | Thresholded operational interpretation rather than direct single-signal truth |
| future declared crawler counts | `exact` | `observed` | Explicit classification once Shuma has a trustworthy crawler contract |
| future verified-bot or signed-agent counts | `exact` | `verified` | Cryptographic or equivalent strong identity verification |
| mutable shared-counter summaries such as current security/privacy posture | `best_effort` | `mixed` | Useful, but current source path is still mutable and non-atomic |
| retention health summary | `best_effort` | `mixed` | Worker-maintained mutable catalogs and state |

# Relationship To The Lane Contract

The traffic-lane contract already uses the language of:

1. `exact`
2. `derived`
3. `best_effort`
4. plus basis values such as `observed`, `policy`, `verified`, and `residual`

This note turns that language into the formal contract that implementation should follow, rather than leaving it as prose local to the lane document.

# Relationship To Bootstrap And Supporting Summaries

This exactness contract does not itself decide whether a summary belongs in bootstrap.

It does, however, constrain bootstrap composition:

1. bootstrap may include derived summaries
2. but bootstrap headlines must not silently present derived or best-effort summaries as exact facts

That ownership question is handled separately in the bootstrap/supporting-summary contract.

# Impact On `MON-TEL-1`

## `MON-TEL-1-1`

Must classify lane families using this exactness-plus-basis vocabulary rather than local adjectives.

## `MON-TEL-1-5`

Must attach exactness and basis metadata to each new operator summary.

## `MON-TEL-1-6`

Must expose that metadata through the admin monitoring contract so the dashboard can present truthful operator wording.

## `MON-OVERHAUL-1`

Must use the metadata to shape labels and explanations rather than implying that every headline chart is equally exact.

# Definition Of Done

This prerequisite is complete when:

1. the operator-summary exactness taxonomy is explicit,
2. exactness and basis are separate dimensions,
3. the future hot-read contract shape is clear,
4. lane and operator summaries can be classified without inventing local wording,
5. and later implementation can expose Monitoring headlines without overstating truthfulness.
