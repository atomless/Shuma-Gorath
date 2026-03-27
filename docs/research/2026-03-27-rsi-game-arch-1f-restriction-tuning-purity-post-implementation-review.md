Date: 2026-03-27
Status: Implemented

Related context:

- [`2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md`](2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../plans/2026-03-27-game-loop-restriction-tuning-purity-plan.md`](../plans/2026-03-27-game-loop-restriction-tuning-purity-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)

# Objective

Close `RSI-GAME-ARCH-1F` by purging recognition-side blocker leakage and simulator-persona gating from restriction tuning, while keeping recognition evaluation visible as a separate rail.

# What landed

1. `benchmark_results_v1` no longer lets recognition-evaluation gaps block restriction tuning once surface-native Scrapling exploit evidence is already strong enough.
2. restriction-grade evidence quality no longer depends on simulator-derived persona diversity or replayed fulfillment-mode labels.
3. the decisive exploit-confidence input is now `recent_window_support_status`, derived from recent-window board-state evidence instead of harness metadata.
4. the dashboard adapter and Game Loop fixtures now preserve that newer recent-window support contract end to end.

# Why this mattered

Before this slice, the live board could already say:

1. Scrapling made localized incursions,
2. the evidence was strong enough to name exact breach loci,
3. and bounded repair families were available,

yet the controller still refused to move because:

1. recognition-side category gaps were being imported as restriction blockers,
2. and exploit-confidence still expected simulator-known persona diversity.

That meant the loop was still partly judging the wrong game.

# Acceptance review

## 1. Restriction tuning no longer depends on simulator-known persona labels

Accepted.

- restriction-grade evidence quality no longer uses `observed_fulfillment_modes`, `sim_profile`, `sim_lane`, or equivalent simulator metadata as a prerequisite.
- the replacement confidence gate is recent-window board support, not harness-lane variety.

## 2. Recognition-side gaps no longer masquerade as restriction blockers in the strong Scrapling board-state case

Accepted.

- the tuning-eligibility path now keeps recognition evaluation visible without letting degraded category receipts or partial category readiness block a strong surface-native Scrapling exploit case.

## 3. Live payload truth materially improved

Accepted.

Fresh local `/admin/benchmark-results` now shows:

1. `overall_status = outside_budget`,
2. `problem_class = scrapling_exploit_progress_gap`,
3. `evidence_quality.status = high_confidence`,
4. `evidence_quality.attribution_status = surface_native_shared_path`,
5. `evidence_quality.recent_window_support_status = reproduced_recently`,
6. and `tuning_eligibility.blockers` reduced to:
   1. `protected_lineage_missing`
   2. `protected_tuning_evidence_not_ready`

That is the correct outcome for this tranche: the remaining blocker is now a controller safety gate, not recognition leakage.

# Verification

1. `make test-rsi-score-evidence-quality`
2. `make test-rsi-score-move-selection`
3. `make dashboard-build`
4. `make test-dashboard-game-loop-accountability`
5. `git diff --check`

# Remaining follow-on

1. `RSI-GAME-ARCH-1G`
   - make strong live Scrapling runtime evidence eligible as protected tuning evidence without reopening simulator-label leakage or weakening the synthetic/advisory safety gates.
2. `RSI-GAME-ARCH-1E`
   - retire remaining replaced category-first surfaces only after the live protected-evidence path is proven.
