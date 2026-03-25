Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-rsi-game-ho-1a-human-only-private-loop-readiness-review.md`](2026-03-25-rsi-game-ho-1a-human-only-private-loop-readiness-review.md)
- [`../plans/2026-03-25-rsi-game-ho-1a-human-only-private-loop-plan.md`](../plans/2026-03-25-rsi-game-ho-1a-human-only-private-loop-plan.md)

# What landed

`RSI-GAME-HO-1A` is now complete.

The existing machine-first first-working-loop proof now runs against the strict `human_only_private` baseline instead of silently inheriting the older balanced default. The focused Rust route proof and the shared-host feedback-loop verifier both now require:

1. `non_human_stance_presets.active_preset_id == "human_only_private"`,
2. `effective_non_human_policy.active_preset_id == "human_only_private"`,
3. `effective_non_human_policy.verified_identity_mode == "verified_identities_denied"`.

# Why this was the right closeout

Before this slice, the repo could truthfully say the first working loop existed, but not that it was operating under the intended strict baseline. That left a real interpretability risk:

1. the loop could appear healthy while still benchmarking a looser stance than the one the project had chosen as its first reference position,
2. live shared-host verification could keep passing while only checking operator-snapshot schema presence rather than the actual resolved stance.

This slice closes that gap without widening the loop mechanism or relaxing any boundary. It simply makes the baseline explicit and fail-closed in the proof paths that already mattered most.

# Remaining gap

`RSI-GAME-HO-1A` is only the first strict-baseline operational slice.

The next active work is:

1. `RSI-GAME-HO-1B` to repeat the Scrapling-driven loop through multiple real config-change and watch-window cycles,
2. then `RSI-GAME-HO-1C` to define and satisfy the unlock condition for demonstrated retained improvement toward the strict target.
