# Tuning Surface Defunct Planning Chain Review

Date: 2026-03-29
Status: Completed retirement review

Related context:

- [`../../docs/dashboard-tabs/tuning.md`](../../docs/dashboard-tabs/tuning.md)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/config/controller_mutability_policy.rs`](../../src/config/controller_mutability_policy.rs)
- [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](../plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)
- [`../plans/2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md`](../plans/2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Decide whether the March 23-24 Tuning expansion chain still represents executable product direction, or whether it should be retired as defunct so the backlog and plan indexes stop presenting it as live work.

# Findings

## 1. The shipped Tuning contract is intentionally narrow

The live dashboard contract in [`../../docs/dashboard-tabs/tuning.md`](../../docs/dashboard-tabs/tuning.md) is still a bounded botness-threshold and signal-weight editor.

That matches the current implementation reality better than the broader March 23-24 planning chain.

## 2. The current config loop does not support the old Tuning expansion shape

The live bounded patch shaper in [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs) only auto-changes a narrow set of config families such as:

1. fingerprint signal enablement,
2. CDP detection enablement,
3. proof-of-work,
4. challenge,
5. not-a-bot,
6. maze core,
7. and core JS-required policy.

That is materially narrower than the old Tuning expansion chain, which assumed near-term ownership of category-posture editing, broader botness or fingerprint consolidation, objective-budget editing, and an `Identification`-tab ownership split.

## 3. Operator objectives and category posture are not a current Tuning execution path

The canonical controller mutability policy in [`../../src/config/controller_mutability_policy.rs`](../../src/config/controller_mutability_policy.rs) keeps operator objectives outside the controller-tunable ring.

That means the old plan chain's center of gravity, namely making `Tuning` the primary editor for operator objectives and category posture, is no longer aligned with the current live control-plane direction.

## 4. The old March 23-24 Tuning plans are now historical planning artifacts, not current sequencing

The old chain was useful when the operator-surface architecture was still open, but it now overstates what the product is about to do next.

Leaving those docs and blocked TODOs in the active chain creates false roadmap truth.

## 5. Not all Tuning-adjacent work is dead

This retirement applies to the March 23-24 expansion chain:

1. `TUNE-SURFACE-1`
2. `TUNE-SURFACE-1A`
3. `TUNE-SURFACE-1B`
4. `TUNE-SURFACE-1C`

It does not automatically kill every future Tuning change. Later Tuning work, such as the separate `INSPECT-1` inspection-controls track, must be judged on its own merits and reopened through fresh research and planning if needed.

# Decision

Retire the March 23-24 Tuning expansion chain as defunct.

That means:

1. remove the `TUNE-SURFACE-1` blocked backlog items,
2. mark the associated March 23-24 Tuning and Identification research and plan docs as defunct,
3. move those docs out of active index sections into historical or defunct sections,
4. clean active roadmap and dependency notes so they no longer point at `TUNE-SURFACE-1` as live future work,
5. and leave the current Tuning tab documented only for its narrow shipped contract.

# Reopen Rule

If broader Tuning work is reopened later, it must start from fresh research against the then-current control-plane and product direction.

Do not resume the retired March 23-24 chain as if it were still current.
