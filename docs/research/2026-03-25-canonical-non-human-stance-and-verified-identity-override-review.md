Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../../src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../docs/configuration.md`](../../docs/configuration.md)

# Purpose

Decide how Shuma should remove the current policy-model mismatch between:

1. the independent verified-identity stance used on the request path,
2. the canonical non-human category posture matrix used by benchmarks and Game Loop,
3. and the stricter development reference stance now intended for later recursive-improvement work.

# Findings

## 1. The current model permits two independent non-human policy sources to disagree

Today Shuma can simultaneously hold:

1. a verified-identity request-path stance in [`../../src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs),
2. and a separate canonical category posture matrix in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs).

That means:

1. runtime enforcement can express one non-human stance,
2. while Game Loop and benchmark output can project another,
3. and operators can see apparently contradictory truths without a clear parent contract.

Conclusion:

1. this is a real policy-model design fault, not only a copy or dashboard problem.

## 2. Verified identity is a partial evidence and authorization surface, not the full non-human policy space

Verified identity currently covers only a subset of the canonical taxonomy after crosswalk:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `http_agent`
4. `agent_on_behalf_of_human`
5. `verified_beneficial_bot`

It does not directly span the full canonical non-human posture space.

Conclusion:

1. verified identity is the wrong place to own the top-level non-human stance,
2. because it is an evidence and exception path over only part of the taxonomy.

## 3. Verified identity should be evidence first, not tolerance by default

The verified-identity research chain already says:

1. authenticated identity is not reputation,
2. authenticated identity is not automatic allow,
3. and operators must be able to deny all non-human traffic even when identity verified successfully.

Conclusion:

1. the correct model is `authenticated identity, then local policy`,
2. not `authenticated identity, therefore tolerated`.

## 4. The strict development reference stance must also deny verified non-human traffic

The reference-stance methodology correctly treats `Human-only / private` as the cleanest first development game.

But that only remains clean if verified non-human traffic is denied or equivalently suppressed under that stance too.

Otherwise:

1. the loop no longer measures a true "all non-human denied" baseline,
2. cost comparisons against later relaxed stances become muddied,
3. and verified identities create a hidden permissive exception during the supposed strict baseline.

Conclusion:

1. `human_only_private` must treat verified non-human traffic the same as other non-human traffic for enforcement purposes,
2. while still surfacing verified identity as telemetry and attribution.

## 5. A later relaxed stance for humans plus verified identities is valuable, but it should come second

After the strict baseline proves useful, a second relaxed preset becomes meaningful:

1. humans allowed,
2. verified non-human traffic eligible for explicitly lower-friction handling,
3. unverified non-human traffic still deny-first.

This is a strong next-step comparison because it asks:

1. what do we gain by trusting only authenticated non-human traffic,
2. and what friction or cost can we remove without reopening general non-human access.

Conclusion:

1. `humans_plus_verified_only` is a good later sweep or product stance candidate,
2. but it should follow the strict baseline rather than precede it.

## 6. Runtime, benchmark, Game Loop, and Tuning need one resolved effective policy contract

Shuma needs one machine-first resolved view that says, per canonical category:

1. what the base stance is,
2. whether verified identity creates an explicit named exception or override,
3. what the effective posture is,
4. and why.

Without that:

1. request path, benchmark scoring, Game Loop, and Tuning are all liable to drift semantically,
2. and later automated tuning risks optimizing against the wrong policy picture.

Conclusion:

1. Shuma should materialize one canonical `effective_non_human_policy` contract and make every consumer read that.

## 7. Pre-launch status makes a clean redesign better than compatibility layering

Because Shuma remains pre-launch:

1. this is the right time to remove the redundant verified-identity stance model,
2. rather than preserve it behind aliases, shims, or silently overlapping semantics.

Conclusion:

1. the cleanest path is to collapse onto one canonical stance model now.

# Decisions

1. Replace the independent verified-identity top-level stance with one canonical non-human stance model over the full taxonomy.
2. Keep verified identity as evidence, named exception policy, and optional service-profile override, not as a separate stance authority.
3. Add explicit canonical stance presets, with at least:
   1. `human_only_private`
   2. `humans_plus_verified_only`
4. Define `human_only_private` so verified non-human traffic remains denied or equivalently suppressed under that strict baseline.
5. Treat `humans_plus_verified_only` as a later relaxed preset or sweep after the strict baseline has proven useful.
6. Expose one machine-first resolved effective policy contract for runtime, benchmark, Game Loop, and Tuning.
7. Treat the current dual-stance model as design debt to remove, not preserve.

# Result

The corrected model should be:

1. one canonical non-human stance source of truth,
2. verified identity as authenticated evidence plus explicit exception handling,
3. one resolved effective policy contract consumed everywhere,
4. a strict `human_only_private` starting position for Game Loop work,
5. and a later `humans_plus_verified_only` relaxation only after the strict baseline is understood.
