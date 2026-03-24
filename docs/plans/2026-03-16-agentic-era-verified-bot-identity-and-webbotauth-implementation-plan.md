Date: 2026-03-16
Status: Proposed

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](../research/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`../research/2026-03-21-verified-identity-execution-readiness-refresh.md`](../research/2026-03-21-verified-identity-execution-readiness-refresh.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-implementation-plan.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-implementation-plan.md)

# Implementation Goals

1. Add a first-class verified bot and agent identity subsystem to Shuma.
2. Support authenticated restrict, deny, and explicitly granted allow policy for named bots and agents.
3. Keep identity verification deterministic and request-path safe.
4. Normalize native Web Bot Auth and provider-verified signals into one internal contract.
5. Make verified identity visible to monitoring, dashboard policy surfaces, and the future oversight controller.
6. Ensure authenticated identities can still be denied or tightly constrained cleanly and obviously under a top-level operator non-human traffic stance.

# Delivery Strategy

Implement this in narrow phases. Identity, then exact restriction-and-exception policy, and only then optional low-cost content profiles should land in that order. Trust-boundary controls should stay manual-only until the system is proven.

# Roadmap Fit

This work is intended to sit in the locked pre-launch sequence:

1. after the delivered controller-grade telemetry foundation and the delivered machine-first operator snapshot and benchmark foundations,
2. before mature adversary-sim expansion,
3. before the later Monitoring projection and Tuning surface completion,
4. before central-intelligence architecture,
5. and before the scheduled agent analyzer and reconfigurer.

Reason:

1. Shuma should formalize authentication and local authorization before testing or tuning against realistic agentic traffic.
2. Mature adversary-sim should be able to model both beneficial verified agents and spoofed, replayed, or policy-violating signed-agent traffic against a real identity lane.
3. Monitoring and Tuning should consume the identity lane once it exists; they should not block the first observe-only identity foundation slices.
4. Central intelligence must remain separate from identity and therefore should follow this work rather than precede it.
5. The future scheduled agent loop must not mutate or reason about trust-boundary controls before those controls are explicitly modeled and surfaced to operators.

Primary product stance:

1. verified identity exists first to make non-human restriction more exact and auditable,
2. not to grant blanket preferential treatment,
3. and any looser treatment must remain an explicit local exception.

# Phase 0: Contracts, ADR Alignment, And Config Prep

## WB-0.1 Define the canonical identity domain

Create a dedicated bot-identity domain with types for:

1. verified identity evidence,
2. verification result,
3. local identity policy,
4. content/service profile,
5. policy decision outcome.

Likely files:

1. `src/bot_identity/contracts.rs`
2. `src/bot_identity/verification.rs`
3. `src/bot_identity/policy.rs`
4. `src/bot_identity/telemetry.rs`

Acceptance:

1. the identity contract is independent of any one provider,
2. all later phases normalize into that contract,
3. docs and comments make identity vs policy vs reputation explicit,
4. and the domain types naturally support restrictive defaults plus explicit exception paths.

## WB-0.2 Add config placeholders and validation

Add canonical config for:

1. identity subsystem enablement,
2. native Web Bot Auth enablement,
3. provider-normalized verified-bot enablement,
4. top-level non-human traffic stance,
5. replay and clock-skew windows,
6. directory cache TTLs and freshness requirements,
7. named identity policy entries,
8. category default actions,
9. service-level profiles.

Acceptance:

1. `config/defaults.env` is the source of truth,
2. admin config and dashboard Advanced JSON parity is maintained,
3. unsafe or contradictory configurations fail validation clearly,
4. and the top-level stance makes restrictive non-human defaults easy to express.

# Phase 1: Observe-Only Identity Normalization

## WB-1.1 Add provider seam for verified bot identity

Extend `src/providers/contracts.rs` and the provider registry so provider-managed verified-bot or signed-agent signals can be normalized into the new identity contract.

Acceptance:

1. provider assertions do not bypass the shared contract,
2. internal and provider-backed runtimes surface identity through the same internal types.

## WB-1.2 Add observe-only telemetry

Emit telemetry for:

1. verification attempts,
2. successful verified identities,
3. failed verification outcomes,
4. replay/freshness failures,
5. provider-vs-native source provenance.

Acceptance:

1. observe-only mode does not change routing yet,
2. monitoring can answer "what identities are showing up and how are they verifying?",
3. and operators can later distinguish "recognized and still restricted" from "recognized and explicitly allowed."

## WB-1.3 Add request-path annotations without behavior change

Thread the normalized identity through policy evaluation and monitoring context, but keep enforcement unchanged in this tranche.

Acceptance:

1. no silent allow or deny behavior changes,
2. identity information is available for monitoring and later policy phases.

# Phase 2: Native Web Bot Auth Verification

## WB-2.1 Implement HTTP Message Signature verification path

Add a native verifier that can:

1. parse and validate the relevant signature inputs,
2. validate `Signature-Agent` style identity binding,
3. enforce replay-window and clock-skew rules,
4. and produce a `VerifiedBotIdentity` result or explicit failure.

Acceptance:

1. verification is deterministic,
2. failure reasons are typed and observable,
3. unsigned `Signature-Agent` style claims never count as verified identity.

## WB-2.2 Add directory and key discovery/cache layer

Implement bounded retrieval and caching for:

1. well-known signature directories,
2. key metadata,
3. freshness status.

Acceptance:

1. directory fetch cost and cache size are bounded,
2. stale or failed discovery is surfaced explicitly,
3. trust is still governed by local operator policy.

## WB-2.3 Preserve proxy and edge trust semantics

Document and enforce the expected handling for signature-relevant headers across gateway and edge deployments.

Acceptance:

1. proxy/header mutation risks are explicit,
2. tests cover trusted forwarding and header-preservation expectations.

# Phase 3: Local Identity Authorization Policy

## WB-3.1 Add named identity policy registry

Implement local policy entries that match on:

1. scheme,
2. stable identity,
3. operator,
4. category,
5. optional path scope.

Actions should include:

1. deny,
2. restrict,
3. observe,
4. allow,
5. low-cost profile.

Acceptance:

1. operators can deny or constrain specific authenticated bots or agents precisely, and then allow named exceptions where desired,
2. policy precedence is explicit and testable,
3. named deny does not depend on user-agent-only matching,
4. a top-level "deny all non-human traffic" stance remains easy to express without forcing operators to enumerate every identity first.

## WB-3.2 Add downgrade and violation handling

Verified identities that violate local policy should be able to:

1. lose low-friction treatment,
2. fall back to stricter routing,
3. or be denied explicitly.

Acceptance:

1. verified identity is not treated as unconditional allow,
2. monitoring makes downgrade causes explicit.

# Phase 4: Service-Level And Low-Cost Agent Profiles

## WB-4.1 Add service/content profile selection

Implement policy-driven selection of service levels such as:

1. browser-like,
2. structured-agent,
3. metadata-only,
4. denied.

Acceptance:

1. the content profile decision is explicit in telemetry,
2. beneficial authenticated agents can be given cheaper access than full browser delivery.

## WB-4.2 Keep profile selection separate from authentication

Do not infer low-cost profile solely from successful verification. It must come from local policy.

Acceptance:

1. authenticated training or scraping identities can still be denied or restricted,
2. authenticated user-triggered agents can receive different service levels than search bots.

# Phase 5: Dashboard, Admin API, And Monitoring Surfaces

## WB-5.1 Add monitoring surfaces

Add monitoring summaries for:

1. verification outcomes by scheme,
2. verified identities seen,
3. policy actions by identity and category,
4. replay and freshness failures,
5. verified-agent rate and byte budgets.

Acceptance:

1. monitoring clearly distinguishes identity telemetry from crawler-policy or intelligence telemetry,
2. operators can understand where verified agents are helping or harming budgets.

## WB-5.2 Add operator control surface

Add dashboard/admin support for:

1. enabling identity verification modes,
2. managing named identity policies,
3. selecting the top-level non-human traffic stance,
4. viewing directory/source freshness,
5. assigning low-cost profiles.

Acceptance:

1. this surface is clearly separate from `robots.txt` controls,
2. trust-boundary settings are labeled as manual-only,
3. the UI makes it obvious that "verified" does not mean "allowed".

## WB-5.3 Update documentation

Update:

1. `docs/configuration.md`
2. `docs/api.md`
3. `docs/dashboard-tabs/game-loop.md`
4. `docs/dashboard-tabs/robots.md` or the successor policy doc
5. `docs/observability.md`
6. `docs/bot-defence.md`

# Phase 6: Oversight Integration

## WB-6.1 Extend oversight budgets

Add fields for:

1. verified-agent success rate,
2. verified-agent bytes served,
3. verification failure spikes,
4. replay rejection rates,
5. downgrade rates for authenticated identities.

Acceptance:

1. the oversight controller can see whether verified identity is helping or hurting human-friction and cost budgets.

## WB-6.2 Restrict what the controller may tune

Controller-autotunable in later phases:

1. verified-agent rate budgets,
2. content-profile thresholds,
3. alerting thresholds,
4. observe-vs-restrict posture for already trusted categories.

Manual-only:

1. trust roots,
2. accepted verification schemes,
3. named allow or deny identities,
4. directory source trust.

Acceptance:

1. trust-boundary mutation remains out of autonomous control.

# Phase 7: Central Intelligence Interaction

## WB-7.1 Keep identity and intelligence separate in code and telemetry

Integrate with the March 16 central-intelligence work so that:

1. intelligence can bias treatment of an already identified actor,
2. but it never mints verified identity,
3. and it never silently overrides manual identity policy.

Acceptance:

1. monitoring preserves provenance,
2. policy reasoning remains legible.

# Testing Plan

## Unit

1. signature verification succeeds only when identity binding and signature inputs are correct,
2. replay and freshness windows behave deterministically,
3. local policy matching and precedence are stable,
4. provider adapters normalize to the same internal identity contract.

## Integration

1. verified identity flows through gateway and direct-host paths correctly,
2. provider-verified and native-verified identities both produce equivalent runtime evidence,
3. named deny or restrict rules apply to authenticated identities correctly,
4. authenticated identities downgrade correctly on policy violation.

## Dashboard and admin

1. operator controls preserve config/API parity,
2. monitoring renders identity outcomes and policy actions truthfully,
3. identity policy surfaces remain distinct from robots controls.

## Adversary simulation and oversight

1. verified beneficial-agent cohorts stay out of hostile-bot traps when policy allows them,
2. misbehaving verified cohorts downgrade appropriately,
3. oversight budgets around verified-agent success and cost remain observable.

# Recommended Initial Ordering

1. Phase 0 and Phase 1
2. Phase 2
3. Phase 3
4. Phase 5 monitoring/admin surfaces
5. Phase 4 low-cost content profiles
6. Phase 6 and Phase 7 integration work
