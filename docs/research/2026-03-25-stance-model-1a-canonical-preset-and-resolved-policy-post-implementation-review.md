Date: 2026-03-25
Status: Completed

Related context:

- [`../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](../plans/2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-review.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/operator_snapshot_effective_non_human_policy.rs`](../../src/observability/operator_snapshot_effective_non_human_policy.rs)
- [`../../src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs)
- [`../../src/observability/hot_read_contract.rs`](../../src/observability/hot_read_contract.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)

# Scope delivered

`STANCE-MODEL-1A` is now landed as a machine-first contract slice.

Delivered artifacts:

1. canonical non-human stance preset catalog in `operator_snapshot_v1.non_human_stance_presets`,
2. strict `human_only_private` and later `humans_plus_verified_only` preset definitions,
3. a resolved `effective_non_human_policy_v1` snapshot section that makes the current verified-identity override conflict explicit category by category,
4. focused proof updates across operator-objectives, hot-read contract, and admin operator-snapshot JSON.

# What now works

## 1. The preset language is explicit and machine-readable

The snapshot now publishes:

1. `balanced_default`,
2. `human_only_private`,
3. `humans_plus_verified_only`,

with the current matched preset id derived from persisted `category_postures` plus the current verified-identity mode.

This gives later Game Loop and Tuning work one canonical preset vocabulary instead of leaving those labels only in planning docs.

## 2. The dual-stance conflict is no longer hidden

The new `effective_non_human_policy` section shows:

1. base posture from `operator_objectives_v1.category_postures`,
2. verified-identity categories that can map into each canonical category,
3. the current override status from the still-legacy verified-identity stance input,
4. the resulting effective verified-identity action or posture when deterministic,
5. and the source-of-authority lineage.

That means Shuma can now show, for example, that the default matrix still says `verified_beneficial_bot=allowed` while the current verified-identity runtime input still denies verified non-human traffic.

## 3. The focused proof path is stronger

`make test-operator-objectives-contract` now proves:

1. preset catalog materialization,
2. resolved policy projection,
3. hot-read contract registration,
4. and the admin `GET /admin/operator-snapshot` JSON surface carrying the new sections.

# What remains intentionally open

## 1. Runtime authorization is not yet rebased

This tranche does **not** remove `verified_identity.non_human_traffic_stance` yet.

Runtime request-path authorization still resolves from that legacy input.

That is intentional and belongs to `STANCE-MODEL-1B`.

## 2. Benchmark, Game Loop, and Tuning do not consume the new resolved policy yet

The new section is available, but the benchmark and dashboard consumers still need to switch from the old split interpretation to the resolved contract in `STANCE-MODEL-1C`.

# Verification

- `make test-operator-objectives-contract`
- `git diff --check`
