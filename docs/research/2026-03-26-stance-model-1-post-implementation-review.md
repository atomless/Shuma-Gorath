# STANCE-MODEL-1 Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`../../src/runtime/non_human_policy.rs`](../../src/runtime/non_human_policy.rs)
- [`../../src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
- [`../../src/observability/benchmark_beneficial_non_human.rs`](../../src/observability/benchmark_beneficial_non_human.rs)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Scope Reviewed

This closeout reviewed the full `STANCE-MODEL-1` tranche:

1. define canonical stance presets and a resolved effective policy contract,
2. remove the independent verified-identity top-level stance authority,
3. align runtime, snapshot, benchmark, and dashboard surfaces to the same policy truth.

# What Landed

1. Shuma now materializes one canonical `effective_non_human_policy_v1` contract in [`src/runtime/non_human_policy.rs`](../../src/runtime/non_human_policy.rs).
2. The seeded default operator-objectives profile remains `human_only_private`, and verified identity is evidence only under that strict profile.
3. The later relaxed preset `humans_plus_verified_only` is machine-readable through the same contract and only changes verified-identity handling to `explicit_overrides_eligible`; it does not introduce a second policy regime.
4. Verified-identity resolution in [`src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs) now falls back to canonical category posture rather than a separate top-level stance enum.
5. The independent config authority `verified_identity.non_human_traffic_stance` and env var `SHUMA_VERIFIED_IDENTITY_NON_HUMAN_TRAFFIC_STANCE` were removed from config, admin write surfaces, export, and dashboard Advanced JSON paths.
6. Runtime verified-identity enforcement now derives its context from operator objectives in [`src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs) and [`src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs).
7. Operator snapshot, benchmark results, and the Game Loop dashboard now project the resolved effective policy and verified-identity override mode instead of the legacy split stance model.

# Review Result

The tranche meets its intended contract:

1. there is now one authoritative non-human stance authority,
2. strict and relaxed presets are explicit and machine-readable through the same policy summary,
3. verified identity no longer acts like a competing top-level authorization system,
4. and operator-facing dashboard truth now matches the runtime policy the request path actually enforces.

The key remaining follow-on is not more stance cleanup. It is proving the now-canonical strict `human_only_private` methodology operationally through the full-power Scrapling lane and repeated judged game-loop cycles.

# Evidence

- `make test-verified-identity-policy`
- `make test-verified-identity-config`
- `make test-operator-objectives-contract`
- `make test-benchmark-results-contract`
- `make test-dashboard-game-loop-accountability`
- `make test-dashboard-verified-identity-pane`
