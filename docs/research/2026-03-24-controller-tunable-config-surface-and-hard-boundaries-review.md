Date: 2026-03-24
Status: Proposed

Related context:

- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/config/controller_action_catalog.rs`](../../src/config/controller_action_catalog.rs)
- [`../../src/config/controller_action_surface.rs`](../../src/config/controller_action_surface.rs)
- [`../../src/config/controller_action_guardrails.rs`](../../src/config/controller_action_guardrails.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/admin/operator_objectives_api.rs`](../../src/admin/operator_objectives_api.rs)
- [`../../dashboard/src/lib/domain/config-schema.js`](../../dashboard/src/lib/domain/config-schema.js)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)

# Objective

Define the correct hard boundary between:

1. operator-owned policy targets,
2. admin-writable but controller-forbidden configuration,
3. and the genuinely controller-tunable config action space.

# Findings

## 1. The repo currently has four related but non-identical surfaces

The current implementation distinguishes, but does not yet fully unify:

1. the full admin-writable config surface in `POST /admin/config`,
2. the operator-target surface in `operator_objectives_v1`,
3. the machine-first controller envelope in `allowed_actions_v1`,
4. and the narrower patch set the current reconcile or apply path can actually emit.

That means the repo already has bounded controller intent, but it does not yet have one canonical mutability contract that answers, with no ambiguity, which fields are:

1. permanently out of bounds,
2. manual-only,
3. or controller-tunable.

## 2. Operator objectives must remain permanently outside the controller action space

`operator_objectives_v1` is the rule set for the game, not part of the move set.

The loop must never mutate:

1. `profile_id`,
2. `window_hours`,
3. `compliance_semantics`,
4. all `category_postures.*`,
5. all `budgets.*`,
6. all `adversary_sim_expectations.*`,
7. all `rollout_guardrails.*`.

Those values define the target state the loop is trying to satisfy. Letting the loop change them would let it redefine success.

## 3. The hard-never config surface is broader than the current `forbidden` family list

The existing `allowed_actions_v1` catalog already keeps some families out of the first controller surface, but the repo needs a stronger permanent no-touch set.

The following config areas should be treated as hard-never for the feedback loop:

### A. Runtime mode and measurement harness

1. `shadow_mode`
2. `adversary_sim_duration_seconds`
3. any dedicated adversary-sim enable or disable control state handled outside `POST /admin/config`

These change how Shuma measures or exercises the system rather than how it tunes defences.

### B. Provider topology and deployment architecture

1. `provider_backends.rate_limiter`
2. `provider_backends.ban_store`
3. `provider_backends.challenge_engine`
4. `provider_backends.maze_tarpit`
5. `provider_backends.fingerprint_signal`
6. `edge_integration_mode`

These define architecture and trust-boundary topology, not bounded tuning moves.

### C. Explicit trust, authorization, and tolerated-bot policy

1. all `verified_identity.*`
2. `robots_enabled`
3. `ai_policy_block_training`
4. `ai_policy_block_search`
5. `ai_policy_allow_search_engines`
6. `robots_crawl_delay`

These are operator authorization choices about beneficial or tolerated traffic and must not become emergent controller behavior.

### D. Explicit trust exceptions and site-local allowlists

1. `bypass_allowlists_enabled`
2. `allowlist`
3. `path_allowlist_enabled`
4. `path_allowlist`
5. `browser_policy_enabled`
6. `browser_block`
7. `browser_allowlist`
8. `geo_risk`
9. `geo_allow`
10. `geo_challenge`
11. `geo_maze`
12. `geo_block`
13. `geo_edge_headers_enabled`
14. `ip_range_policy_mode`
15. `ip_range_emergency_allowlist`
16. `ip_range_custom_rules`
17. `honeypot_enabled`
18. `honeypots`

These are trust-boundary and site-specific exception policy, with high collateral-risk if a loop mutates them.

### E. Privacy posture

1. `fingerprint_pseudonymize`

Privacy and observability policy must not be rewritten by an optimization loop.

### F. Punishment horizon and sanction policy

1. `ban_duration`
2. all `ban_durations.*`

These change punishment horizon, not bounded detection sensitivity.

### G. Defender resource ceilings and safety budgets

1. all `tarpit_*`
2. all `maze_*` except:
   1. `maze_enabled`
   2. `maze_auto_ban`
   3. `maze_rollout_phase`

These fields define Shuma's own cost, concurrency, content, seed, and fallback guardrails and should not be controller-tunable.

### H. Implementation-composition selectors

1. `cdp_probe_family`
2. all `defence_modes.*`
3. `maze_seed_provider`

These are implementation, rollout, or asset-selection concerns rather than safe tuning knobs.

## 4. The candidate controller-tunable set should stay narrow and sensitivity-oriented

The first canonical in-bounds surface should be limited to bounded sensitivity and rollout controls:

1. `js_required_enforced`
2. `pow_enabled`
3. `pow_difficulty`
4. `pow_ttl_seconds`
5. `challenge_puzzle_enabled`
6. `challenge_puzzle_transform_count`
7. `challenge_puzzle_seed_ttl_seconds`
8. `challenge_puzzle_attempt_limit_per_window`
9. `challenge_puzzle_attempt_window_seconds`
10. `challenge_puzzle_risk_threshold`
11. `not_a_bot_enabled`
12. `not_a_bot_risk_threshold`
13. `not_a_bot_pass_score`
14. `not_a_bot_fail_score`
15. `not_a_bot_nonce_ttl_seconds`
16. `not_a_bot_marker_ttl_seconds`
17. `not_a_bot_attempt_limit_per_window`
18. `not_a_bot_attempt_window_seconds`
19. `botness_maze_threshold`
20. all `botness_weights.*`
21. `maze_enabled`
22. `maze_auto_ban`
23. `maze_rollout_phase`
24. `cdp_detection_enabled`
25. `cdp_auto_ban`
26. `cdp_detection_threshold`
27. `cdp_probe_rollout_percent`
28. `fingerprint_signal_enabled`
29. `fingerprint_state_ttl_seconds`
30. `fingerprint_flow_window_seconds`
31. `fingerprint_flow_violation_threshold`
32. `fingerprint_entropy_budget`
33. `fingerprint_family_cap_header_runtime`
34. `fingerprint_family_cap_transport`
35. `fingerprint_family_cap_temporal`
36. `fingerprint_family_cap_persistence`
37. `fingerprint_family_cap_behavior`

This is still broader than what the current proposer mutates, but it is the right candidate surface to ratify or narrow intentionally.

## 5. There is still catalog/proposer drift that must be fixed before the surface is trustworthy

Two specific problems stand out:

1. the controller catalog is broader than the patch proposer, so "controller-tunable" and "actually proposable today" are not yet the same thing,
2. `challenge_puzzle_risk_threshold` is currently patched under the `challenge` family even though the catalog classifies that path under `botness.thresholds`.

That means the repo is close to a clear mutability model, but not yet fully there.

# Recommended Direction

Codify one explicit mutability policy with three rings:

1. `never`
2. `manual_only`
3. `controller_tunable`

Then make the following surfaces derive from that single truth:

1. `allowed_actions_v1` or its successor,
2. `oversight_patch_policy`,
3. benchmark escalation family mapping,
4. docs and Advanced JSON classification,
5. later Monitoring or Tuning controller-explanation surfaces.

The controller must remain a bounded config optimizer over policy-defined targets. It must not be allowed to change the game rules, the security topology, the trust exceptions, or Shuma's own safety-critical operating envelope.
