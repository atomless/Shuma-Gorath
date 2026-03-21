Date: 2026-03-16
Status: Proposed

Related context:

- [`../research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](../research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../project-principles.md`](../project-principles.md)
- [`../module-boundaries.md`](../module-boundaries.md)

# Objectives

1. Give Shuma a first-class verified bot and agent identity lane that fits the emerging Web Bot Auth ecosystem.
2. Let operators restrict or deny non-human traffic with much higher precision than user-agent strings or CIDR lists allow, and then loosen those restrictions only for specifically identified bots or agents when they choose to.
3. Preserve low friction for authenticated agents only as an explicit operator-granted exception, while still allowing Shuma to react when a verified identity violates local policy.
4. Keep request-path identity verification deterministic and Rust-owned.
5. Keep identity, authorization policy, crawler preference signaling, and central intelligence as separate concerns.

# Non-goals

1. Treating `robots.txt` or AI preference signaling as verified identity.
2. Treating authenticated identity as automatic allow.
3. Assuming verified identities should receive lower friction unless an operator opts out.
4. Letting the future oversight controller auto-mutate trust roots, named allow/deny rules, or directory sources in early phases.
5. Building a hosted global reputation system as part of the first verified-identity tranche.
6. Relying on LLMs or agents inside the request path.
7. Adding backward-compatibility aliases or migration clutter in this pre-launch stage.

# Architectural Positioning

Verified bot identity should be modeled as a separate subsystem that feeds Shuma's existing three-lane traffic model.

## Lane A: Verified Identified Agents

Definition:

1. requests with high-confidence cryptographic identity,
2. or provider-verified bot signals normalized to equivalent assurance,
3. where local operator policy can now constrain, deny, or explicitly allow the identity with precision.

Expected treatment:

1. explicit rate and scope policy first,
2. easy expression of named deny or restrictive defaults,
3. eligibility for lower-friction or lower-cost treatment only when policy grants it,
4. downgrade path if behavior violates local policy.

## Lane B: Declared Crawlers and Search Bots

Definition:

1. requests that identify as bots via UA and robots semantics,
2. without strong cryptographic identity.

Expected treatment:

1. advisory crawler policy,
2. rate and scope constraints,
3. escalation when declared posture diverges from observed behavior.

## Lane C: Unverified or Suspicious Automation

Definition:

1. undeclared automation,
2. unverifiable or failed-identity requests,
3. or automation that violates local policy severely enough to lose verified-lane benefits.

Expected treatment:

1. low-cost checks first,
2. challenge, maze, tarpit, and ban escalation,
3. aggressive origin-cost reduction.

# Core Design

## A. Canonical Internal Identity Contract

Shuma should normalize all verified-bot signals into one internal runtime object, tentatively:

1. `BotIdentityEvidence`
2. `VerifiedBotIdentity`
3. `BotIdentityVerificationResult`

Minimum fields:

1. `scheme`
   - `http_message_signatures`
   - `provider_verified_bot`
   - `provider_signed_agent`
   - reserved for future `mtls`
2. `stable_identity`
   - canonical identity string, such as a verified agent name
3. `operator`
   - `openai`, `google`, `anthropic`, `cloudflare`, `vercel`, or explicit custom operator
4. `category`
   - `training`
   - `search`
   - `user_triggered_agent`
   - `preview`
   - `service_agent`
   - `other`
5. `verification_strength`
   - `cryptographic`
   - `provider_asserted`
6. `end_user_controlled`
7. `directory_source`
8. `verification_outcome`
9. `failure_reason` when verification fails
10. `local_policy_match`

This contract must be stable across:

1. direct local verification,
2. Cloudflare-style signed-agent or verified-bot inputs,
3. Vercel-style verified-bot inputs,
4. future provider adapters.

## B. Local Authorization Policy

Authentication is not authorization. Shuma therefore needs a local identity-policy registry that is distinct from:

1. crawler policy,
2. central intelligence,
3. and generic fingerprint scoring.

Each policy entry should match on a combination of:

1. scheme,
2. stable identity,
3. operator,
4. category,
5. optional path scope,
6. optional host/site scope.

Each policy entry should express actions such as:

1. `allow`
2. `allow_low_cost_profile`
3. `observe`
4. `restrict_rate`
5. `restrict_scope`
6. `challenge_on_violation`
7. `deny`

This is the core operator feature the user asked for: allowing or blocking specific bots or agents based on authenticated identity.

The main product value of this subsystem is not automatic bot friendliness. It is that authenticated identity gives Shuma a deterministic way to say:

1. all non-human traffic is restricted by default,
2. this named identity is still denied,
3. this named identity is allowed only on a bounded scope or rate,
4. and only these specific named identities receive looser treatment.

At the top level, the policy model should also expose a small number of obvious operator stances, for example:

1. `deny_all_non_human`
2. `allow_only_explicit_verified_identities`
3. `allow_verified_by_category`
4. `allow_verified_with_low_cost_profiles_only`

These are operator authorization choices, not telemetry classifications.

## C. Service-Level Profiles

Verified identity becomes more valuable if it can select a cheaper service profile.

Initial design profile classes:

1. `browser_like`
   - full normal human-oriented site behavior
2. `structured_agent`
   - lower-cost agent-optimized representation
3. `metadata_only`
   - constrained low-cost access for preview or discovery use cases
4. `denied`

This keeps Shuma aligned with the agentic-era dual strategy:

1. cost-shift suspicious automation,
2. reduce site cost for only those authenticated agents the operator explicitly chooses to treat more cheaply.

It must remain explicit that these service profiles are granted by local policy. A successfully authenticated identity may still map to `denied`.

## D. Verification Model

The request path should support three verifier inputs that all normalize into the same contract.

### 1. Native Web Bot Auth verifier

Use HTTP Message Signatures plus relevant discovery material such as:

1. `Signature-Agent`,
2. key discovery,
3. directory metadata,
4. anti-replay state.

### 2. Provider-verified bot adapters

When an upstream provider already verified the identity, Shuma should consume that through `src/providers/contracts.rs` and normalize it rather than inventing a parallel decision path.

### 3. Provider-signed-agent adapters

Signed-agent semantics from Cloudflare or similar providers should also normalize into the same verified identity object.

The runtime should not trust a raw `Signature-Agent` or equivalent header by itself.

## E. Trust Model

Trust must stay explicit.

### Manual-only trust surfaces

These should be manual-only in the first phases:

1. trusted directory sources,
2. trust-root or key-source rules,
3. named allow or deny identity policy,
4. category default actions,
5. any setting that changes whether an identity is treated as verified at all.

### Why manual-only

These controls define authorization and trust boundaries, not just tuning.

The future oversight controller can watch verified-agent budgets, but it should not:

1. auto-approve a new trusted agent,
2. auto-enable a new directory,
3. or auto-deny a specific authenticated identity.

## F. Failure and Downgrade Behavior

A verified identity does not get permanent immunity.

Downgrade triggers should include:

1. verification failure,
2. replay rejection,
3. stale or missing directory material when policy requires freshness,
4. severe local policy violations,
5. explicit local deny policy.

Downgrade outcomes can include:

1. fallback to declared-crawler handling,
2. fallback to suspicious-automation handling,
3. direct deny where the local policy says so.

This preserves safety without discarding the value of verified identity.

# Config and Control Surfaces

## Manual operator controls

Shuma should add a dedicated identity-policy surface for:

1. enabling the identity subsystem,
2. enabling native Web Bot Auth verification,
3. enabling provider-normalized verified-bot inputs,
4. top-level non-human traffic stance,
5. freshness and replay windows,
6. trusted directory sources,
7. per-identity policy rules,
8. default actions by category,
9. service-level profile mapping.

## Autotunable surfaces

The oversight controller may eventually tune:

1. verified-agent rate budgets,
2. low-cost content profile selection thresholds,
3. monitoring alert thresholds around verification failures,
4. replay-window alerting,
5. observe-vs-restrict budgets for already trusted categories.

It must not autotune:

1. trust roots,
2. named allow or deny identity entries,
3. acceptance of a new verification scheme,
4. or directory source trust.

# Monitoring and Telemetry

Shuma should add truthful monitoring for:

1. verification attempts by scheme,
2. verification successes and failures,
3. replay rejects,
4. stale-directory or stale-key outcomes,
5. identities seen in recent windows,
6. policy actions by identity and category,
7. verified-agent rate and byte budgets,
8. verified-agent success and downgrade rates.

Important truth rule:

1. identity verification telemetry,
2. reputation/intelligence telemetry,
3. and crawler-policy telemetry

must remain distinguishable in monitoring so operators understand what actually drove a decision.

# Dashboard and Operator UX Implications

The current crawler-policy surface is not enough for this feature.

Recommended operator-facing split:

1. keep `robots`/AI policy focused on advisory crawler communication,
2. add verified-identity controls as a dedicated policy surface,
3. add monitoring summaries for identity verification and policy outcomes,
4. show per-identity allow/restrict/deny state clearly,
5. surface structured-content or low-cost profile assignments explicitly.

This can be implemented either as a new dedicated tab or as a clearly separated section under the bot/agent policy area, but it must not be buried as a small extension of `robots.txt`.

# Security and Abuse Considerations

## Required protections

1. anti-replay storage or equivalent replay defense,
2. bounded clock skew tolerance,
3. strict freshness handling for directories and keys,
4. signature-relevant header preservation rules through proxies,
5. bounded directory fetch cost and cache size,
6. clear downgrade semantics when verification cannot be trusted.

## Out-of-scope for v1

1. full multi-provider federated trust exchange,
2. automatic acceptance of arbitrary public registries,
3. identity-driven autonomous code changes,
4. global deny based solely on identity without local policy.

# Interaction With Other Agentic-Era Features

## Central intelligence

Central intelligence may bias treatment of an actor, but it must never create verified identity. The March 16 central-intelligence design remains a separate system.

## Ban jitter and repeat-offender ladder

Verified beneficial agents should normally stay out of coarse recidive or jitter-driven punishment unless they violate local policy. When they do violate policy, the resulting action should record that the actor was verified so operators can distinguish "known agent misbehaved" from "unknown bot."

## Oversight controller

The March 15 oversight design already includes verified-agent success budgets. This identity design is the missing concrete contract that allows those budgets to become real.

# Recommended Rollout

1. observe-only identity normalization,
2. manual identity policy registry,
3. allow/restrict/deny decisions on verified identities,
4. low-cost content profile selection,
5. oversight-controller budget integration.
