Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-live-protected-scrapling-evidence-gap-review.md`](../research/2026-03-27-game-loop-live-protected-scrapling-evidence-gap-review.md)
- [`../research/2026-03-27-rsi-game-arch-1f-restriction-tuning-purity-post-implementation-review.md`](../research/2026-03-27-rsi-game-arch-1f-restriction-tuning-purity-post-implementation-review.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../research/2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md`](../research/2026-03-22-sim-protected-1-protected-tuning-evidence-post-implementation-review.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Make the strong live Scrapling runtime path eligible as protected tuning evidence so the controller can finally test bounded config moves from real board-state pressure, without weakening the existing safety boundary around synthetic and advisory evidence.

# Core decisions

1. `synthetic_traffic` must remain tuning-ineligible.
2. raw frontier or LLM discoveries must remain advisory until replay-promoted or equivalently confirmed.
3. simulator-known category or persona labels must remain absent from runtime defenses and restriction tuning.
4. live Scrapling runtime evidence may become protected only when its proof is runtime-native and strong enough:
   1. shared-path attribution,
   2. localized breach loci,
   3. sufficient samples,
   4. reproduced recent-window support,
   5. and any other no-harm or controller safety gates already required by the bounded loop.

# Execution tranche

## `RSI-GAME-ARCH-1G`

### Add a runtime-native protected basis for strong live Scrapling evidence

Required contract:

1. the protected-evidence summary must distinguish replay-promoted lineage from strong live Scrapling runtime evidence,
2. the controller may treat the live Scrapling basis as protected only when the runtime evidence is board-state-native and meets explicit confidence thresholds,
3. synthetic and advisory lanes must stay ineligible,
4. and the live controller must be able to progress past `protected_tuning_evidence_not_ready` when the Scrapling runtime basis is satisfied.

Acceptance criteria:

1. `ReplayPromotionSummary` or its replacement protected-evidence contract exposes a protected basis for strong live Scrapling runtime evidence instead of only replay-promoted lineage,
2. the benchmark/controller eligibility path can become `eligible` on strong live Scrapling runtime evidence without requiring simulator metadata or replay-promotion materialization,
3. `synthetic_traffic` remains explicitly ineligible,
4. raw frontier or LLM evidence remains advisory until replay-promoted or equivalently confirmed,
5. focused proof exists through:
   1. `make test-protected-tuning-evidence`
   2. `make test-benchmark-results-contract`
   3. `make test-rsi-score-move-selection`
   4. `make test-dashboard-game-loop-accountability`
6. live payload inspection after the fix shows the controller no longer blocks on `protected_lineage_missing` and `protected_tuning_evidence_not_ready` when the Scrapling runtime board evidence is already strong enough.

# Sequencing

1. Land `RSI-GAME-ARCH-1G` before `RSI-GAME-ARCH-1E`.
2. Do not reopen broader combined-attacker or later code-evolution claims until this live protected-evidence blocker is solved.

# Definition of done

This tranche is complete when:

1. strong live Scrapling runtime evidence has an explicit protected basis,
2. the controller can act on that basis without weakening synthetic/advisory safety rules,
3. simulator labels remain absent from runtime and restriction tuning,
4. and both focused tests and live payload evidence show the Game Loop moved materially closer to a real bounded Scrapling RSI loop.
