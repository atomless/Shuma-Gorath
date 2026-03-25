Date: 2026-03-25
Status: Completed

Related context:

- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`2026-03-25-stance-model-1a-canonical-preset-and-resolved-policy-post-implementation-review.md`](2026-03-25-stance-model-1a-canonical-preset-and-resolved-policy-post-implementation-review.md)
- [`../../src/bot_identity/policy.rs`](../../src/bot_identity/policy.rs)
- [`../../src/config/mod.rs`](../../src/config/mod.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/observability/operator_snapshot_verified_identity.rs`](../../src/observability/operator_snapshot_verified_identity.rs)
- [`../../src/observability/operator_snapshot_effective_non_human_policy.rs`](../../src/observability/operator_snapshot_effective_non_human_policy.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)

# Scope delivered

`STANCE-MODEL-1B` is now landed as the runtime and config-side removal of verified identity as an independent stance authority.

Delivered artifacts:

1. removal of `verified_identity.non_human_traffic_stance` from the live config contract,
2. runtime verified-identity resolution based on named policies, category defaults, and explicit default deny rather than a second top-level stance,
3. snapshot and benchmark terminology rebased from competing-stance language to explicit override-mode language,
4. admin/config/dashboard parity updates removing the obsolete writable path and surfacing `override_mode` instead.

# What now works

## 1. Verified identity is no longer a second stance regime

The runtime verified-identity path now resolves in one order:

1. named policy match,
2. category-default override,
3. explicit default deny.

That means authenticated identity still matters, but it no longer carries an independent top-level policy stance that can silently conflict with the canonical non-human posture model.

## 2. Config and admin surfaces no longer publish the obsolete stance field

`SHUMA_VERIFIED_IDENTITY_NON_HUMAN_TRAFFIC_STANCE` and the matching admin/dashboard JSON path are gone from:

1. seeded defaults,
2. config parsing,
3. admin config mutation,
4. dashboard advanced JSON paths,
5. typed operator-snapshot normalization.

This removes one major source of policy-model drift and contributor confusion.

## 3. Machine-first projection now talks about explicit override mode

Verified-identity summary and beneficial non-human benchmarking now project:

1. `verified_identities_denied`,
2. `verified_identities_only`,
3. `disabled`,

instead of implying a competing non-human stance.

This is a cleaner machine-first bridge into the later Game Loop and benchmark alignment work.

# What remains intentionally open

## 1. The category benchmark and Game Loop semantics still need full resolved-policy rebasing

This tranche removes the second stance authority and cleans the projection language, but it does **not** yet fully rebase every benchmark family and Game Loop interpretation onto the resolved effective policy contract.

That remains the core scope of `STANCE-MODEL-1C`.

## 2. Human-only execution proof still comes later

This tranche does not yet run the repeated strict `human_only_private` Scrapling game loop.

That remains sequenced behind:

1. `STANCE-MODEL-1C`,
2. `SIM-SCR-FULL-1`,
3. and then `RSI-GAME-HO-1`.

# Verification

- `make test-verified-identity-policy`
- `make test-verified-identity-config`
- `make test-operator-objectives-contract`
- `make test-verified-identity-calibration-readiness`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`
