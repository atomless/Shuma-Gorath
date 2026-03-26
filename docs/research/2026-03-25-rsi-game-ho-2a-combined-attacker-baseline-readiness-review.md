Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md`](2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md)
- [`2026-03-25-sim-llm-browser-1-container-browser-mode-post-implementation-review.md`](2026-03-25-sim-llm-browser-1-container-browser-mode-post-implementation-review.md)
- [`2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-post-implementation-review.md`](2026-03-25-rsi-game-ho-1c-strict-improvement-unlock-post-implementation-review.md)
- [`../plans/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md`](../plans/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/test_support.rs`](../../src/test_support.rs)
- [`../../Makefile`](../../Makefile)

# RSI-GAME-HO-2A Combined-Attacker Baseline Readiness Review

## Purpose

Decide the first atomic mixed-attacker strict-baseline slice now that:

1. the strict `human_only_private` Scrapling-only proof chain is complete, and
2. the live `bot_red_team` actor now executes bounded browser-mode fulfillment truthfully.

## What is already true

The repo can already prove three important things independently:

1. the strict `human_only_private` stance is active and fail-closed in the first working Game Loop proof,
2. repeated strict-baseline cycles can retain and later roll back bounded config changes under Scrapling pressure,
3. and the live LLM attacker now leaves bounded request-mode and browser-mode recent-run receipts with truthful operator visibility.

Those proofs live in:

1. [`../../src/admin/api.rs`](../../src/admin/api.rs)
2. [`../../src/observability/llm_runtime_recent_run.rs`](../../src/observability/llm_runtime_recent_run.rs)
3. [`../../Makefile`](../../Makefile)
4. [`../../docs/testing.md`](../../docs/testing.md)

## What is not yet true

The repo does not yet have one focused proof that the strict Game Loop is operating under combined attacker pressure.

Current gap:

1. the route-level strict-loop proofs still model only Scrapling-driven post-sim iterations,
2. the active strict-loop Make targets only prove Scrapling-only repeated cycles and improvement unlock,
3. and there is no focused mixed-attacker gate showing both Scrapling and `bot_red_team` evidence participating in the same strict-baseline loop story.

So the missing seam is not the canary or watch-window mechanism anymore.

It is the proof that the next strict-baseline phase is really mixed-attacker rather than merely adjacent single-lane proofs.

## Recommended decomposition

`RSI-GAME-HO-2` should follow the same explicit tranche split used successfully for `RSI-GAME-HO-1`.

### `RSI-GAME-HO-2A`

Add the first combined-attacker strict-baseline proof.

Required contract:

1. the route-level proof still runs under `human_only_private`,
2. both Scrapling and `bot_red_team` are present as recent strict-baseline attacker pressure sources,
3. the loop still accepts bounded post-sim progression without inventing a second mixed-attacker harness,
4. and the proof path becomes a focused Make target rather than an implicit future umbrella.

### `RSI-GAME-HO-2B`

Extend the first mixed-attacker proof into repeated retained config-change cycles.

### `RSI-GAME-HO-2C`

Make the mixed-attacker unlock condition explicit before any later relaxed stance opens.

## Why this is the right next slice

Jumping directly to the full `RSI-GAME-HO-2` umbrella would blur three different claims:

1. first mixed-attacker participation,
2. repeated mixed-attacker iteration,
3. and the later unlock threshold.

`RSI-GAME-HO-2A` is the clean next step because it adds the missing combined-attacker seam without overclaiming repeated improvement before that first seam exists.

## Definition of ready

This readiness review is satisfied when:

1. the repo treats `RSI-GAME-HO-2A` as the next active atomic tranche,
2. the plan names the exact proof seam to add,
3. and the backlog no longer treats `RSI-GAME-HO-2` as one unstructured next blob.
