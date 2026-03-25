Date: 2026-03-25
Status: Completed

Related context:

- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`2026-03-25-stance-model-1a-canonical-preset-and-resolved-policy-post-implementation-review.md`](2026-03-25-stance-model-1a-canonical-preset-and-resolved-policy-post-implementation-review.md)
- [`2026-03-25-stance-model-1b-explicit-verified-identity-override-post-implementation-review.md`](2026-03-25-stance-model-1b-explicit-verified-identity-override-post-implementation-review.md)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/operator_snapshot_effective_non_human_policy.rs`](../../src/observability/operator_snapshot_effective_non_human_policy.rs)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Scope delivered

`STANCE-MODEL-1C` is now landed as the benchmark and Game Loop consumer rebase onto the resolved effective non-human policy contract.

Delivered artifacts:

1. effective policy rows now publish explicit `effective_posture` and `effective_posture_source`,
2. the canonical `non_human_category_posture` benchmark family now scores against that resolved target posture instead of raw base posture rows,
3. the dashboard operator snapshot adapter now preserves resolved-policy context,
4. the Game Loop category section now shows effective posture, base posture when it differs, and the verified-override resolution path that produced the judged target.

# What now works

## 1. Benchmark and snapshot agree on what target is being judged

The category benchmark no longer asks operators to mentally reconcile:

1. raw persisted category posture,
2. verified-identity override mode,
3. and the actual runtime target.

Instead, it consumes the same resolved policy row published in `operator_snapshot_v1.effective_non_human_policy`.

## 2. Game Loop rows are now interpretable

The dashboard now shows:

1. the active preset and verified-identity mode,
2. the effective posture being scored,
3. the base posture when it differs,
4. and whether the target was resolved via verified override or base posture.

That closes the operator confusion where `Category Target Achievement` could previously imply that Shuma was judging against a target different from the one the runtime was actually enforcing.

## 3. `STANCE-MODEL-1` is effectively complete

With `1A`, `1B`, and `1C` landed, Shuma now has:

1. one canonical stance preset vocabulary,
2. verified identity as explicit override rather than competing stance,
3. and one resolved effective policy contract shared by runtime-facing and Game Loop-facing consumers.

# What remains intentionally open

## 1. Strict human-only execution proof is still a later tranche

This tranche makes the model interpretable and internally aligned, but it does **not** yet prove:

1. repeated strict `human_only_private` Scrapling loop cycles,
2. retained config-change improvement under that stance,
3. or the later mixed Scrapling plus LLM attacker proof.

Those remain sequenced behind `SIM-SCR-FULL-1`, `RSI-GAME-HO-1`, and later `RSI-GAME-HO-2`.

## 2. Tuning posture editing remains intentionally separate

The current Tuning surface still edits botness-scoring controls only, so this tranche did not need a separate UI rework there. The important boundary is that future stance editing must continue to consume the same resolved policy model rather than reintroducing a second stance system.

# Verification

- `make test-operator-objectives-contract`
- `make test-verified-identity-calibration-readiness`
- `make test-dashboard-game-loop-accountability`
- `git diff --check`
