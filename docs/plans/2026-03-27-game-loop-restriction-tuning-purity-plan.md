Date: 2026-03-27
Status: Implemented

Related context:

- [`../research/2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md`](../research/2026-03-27-game-loop-restriction-tuning-purity-live-gap-review.md)
- [`../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../research/2026-03-27-game-loop-architecture-alignment-gap-review.md`](../research/2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Repair the last major purity break in the Scrapling Game Loop so restriction tuning can act on real board-state pressure without being blocked by recognition-side category gaps or by simulator-derived persona metadata.

# Core Decisions

1. Restriction tuning must not depend on simulator-known persona or category labels.
2. Recognition-evaluation gaps must remain visible but must not block restriction tuning when surface-native recent-window breach evidence is already strong enough.
3. Restriction-grade exploit evidence quality must be based on board-state evidence only:
   1. named breach loci,
   2. sample sufficiency,
   3. locality,
   4. reproducibility across recent runs,
   5. and any confidence or abuse-pressure signals that come from Shuma-side observation.
4. Latest-run-only simulator-mode diversity is not a valid restriction-tuning gate.
5. Legacy-surface retirement (`RSI-GAME-ARCH-1E`) must wait until this purity repair lands.

# Execution Tranche

## `RSI-GAME-ARCH-1F`

### Purge recognition leakage from restriction tuning and recenter exploit evidence on recent-window board truth

Required contract:

1. `tuning_eligibility` must stop treating recognition-evaluation gaps as hard blockers for restriction tuning when board-state exploit evidence is already restriction-grade,
2. `scrapling_exploit_evidence_quality` must stop using simulator-derived persona diversity in restriction-grade confidence,
3. the replacement confidence gate must instead use recent-window support derived from recent breach loci and shared-path evidence,
4. simulator labels must remain available only to recognition evaluation and harness audit surfaces,
5. the live controller contract must be able to progress from `observe_longer` to an actionable bounded move when the restriction-grade evidence is now strong enough.

Acceptance criteria:

1. no restriction-tuning path consumes `observed_fulfillment_modes`, `sim_profile`, `sim_lane`, or equivalent simulator-known persona metadata as a confidence prerequisite,
2. recognition-side blockers such as `degraded_category_receipts_present` and `insufficient_category_evidence` no longer block restriction tuning once:
   1. exploit progress is outside budget,
   2. evidence is localized,
   3. samples are sufficient,
   4. and recent-window support is reproduced from board-state evidence,
3. the benchmark evidence-quality contract exposes a restriction-grade recent-window support field instead of simulator-persona diversity as the decisive confidence input,
4. the controller contract and dashboard projection remain explicit about recognition-evaluation gaps without letting them masquerade as restriction blockers,
5. focused proof passes through:
   1. `make test-benchmark-results-contract`,
   2. `make test-rsi-score-evidence-quality`,
   3. `make test-rsi-score-move-selection`,
   4. `make test-dashboard-game-loop-accountability`,
6. live verification after the fix can show that the fresh local benchmark payload no longer blocks purely because of recognition-side category gaps when the board-state evidence is otherwise strong enough.

# Sequencing

1. Land `RSI-GAME-ARCH-1F` before `RSI-GAME-ARCH-1E`.
2. Keep `SIM-LLM-1C3` blocked behind a trustworthy Scrapling restriction loop.
3. Continue later retirement cleanup only after this purity repair is proven and the current controller no longer depends on recognition leakage.

# Definition Of Done

This tranche is complete when:

1. the benchmark/controller path is restriction-pure with respect to simulator labels,
2. recognition evaluation remains visible but no longer blocks bounded restriction tuning in the strong Scrapling board-state case,
3. recent-window breach support replaces simulator persona diversity as the decisive exploit-confidence gate,
4. the Game Loop and machine-first API still tell the truth about the distinction between restriction and recognition,
5. and focused verification plus live payload inspection prove the controller moved materially closer to a real bounded RSI loop.
