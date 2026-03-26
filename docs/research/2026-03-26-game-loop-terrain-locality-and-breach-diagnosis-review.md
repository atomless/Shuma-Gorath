Date: 2026-03-26
Status: Proposed planning driver

Related context:

- [`2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
- [`2026-03-26-ideal-rsi-game-loop-scoring-review.md`](2026-03-26-ideal-rsi-game-loop-scoring-review.md)
- [`../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)

# Purpose

Clarify the missing locality requirement in Shuma's Game Loop scoring model:
the host site should be treated as the terrain or board, adversary-sim traffic is the invading enemy, and Shuma's configurable defenses are the bounded pieces the system may reposition or retune.

That framing is only useful if the machine-first judge can say where the invasion advanced, which defense failed there, and what the smallest credible repair is.

# Assumptions

1. The current strict phase is all-out exclusion: adversary-sim lanes are authoritative non-human traffic and should not receive tolerance budgets.
2. The first repair ring is bounded config change, not arbitrary code mutation.
3. Telemetry remains the authoritative map of runtime reality.
4. Human-friction tradeoffs are a separate later ring and must not dilute the sim-only exclusion logic.

# Why The Current Model Is Still Underspecified

## 1. Sitewide aggregates are not enough to steer local repair

Suspicious forwarded leakage and coarse category achievement are useful state summaries, but they do not identify the exact breach locus.

They cannot by themselves answer:

1. which path or defense stage was penetrated,
2. whether the adversary advanced shallowly or deeply through the terrain,
3. what resource it consumed after the breach,
4. or which bounded config move is the smallest repair likely to close that exact gap.

## 2. Fine-grained tuning needs a breach-locus contract

For Shuma's loop to tune defenses precisely, attacker evidence needs to be local enough to name:

1. the relevant adversary lane and persona,
2. the route, surface, or challenge stage reached,
3. the defense layer expected to stop or redirect the adversary,
4. the observed contract outcome,
5. the resource impact consumed beyond the expected boundary,
6. and the confidence that this was a true breach rather than noise.

Without that tuple, config tuning remains vulnerable to broad family nudges that might move many defenses at once without proving they address the real weakness.

## 3. Move selection should prefer the smallest effective local repair

If the board model is real, Shuma should not respond to every exploit signal by sweeping the whole board.

The default response should be:

1. identify the narrowest failing locus,
2. rank the bounded legal moves that act on that locus,
3. estimate expected exploit reduction and collateral risk,
4. and prefer the smallest move that plausibly closes the gap.

Broad, multi-family changes should be treated as an exception that requires explicit evidence of distributed or ambiguous failure.

# Recommended Contract Additions

## 1. Terrain-local exploit evidence

`RSI-SCORE-2A` should require exploit-progress evidence that is localized enough to support repair, not just explanation.

At minimum, the judge-facing exploit evidence should preserve:

1. `surface_id` or equivalent defense-stage identity,
2. path, route, or interaction locus where relevant,
3. expected vs observed contract,
4. adversary lane and persona,
5. resource consumed past the expected stop point where Shuma can measure it,
6. reproducibility and novelty versus the accepted baseline.

## 2. Diagnosis must emit explicit breach loci

`RSI-SCORE-2B` should require the diagnoser to emit ranked breach loci with confidence, not just a family-level pressure label.

A controller-grade diagnosis should say, in machine-readable form:

1. what exact locus appears weak,
2. why the evidence is strong enough to act,
3. which nearby defense controls are the plausible repair surface,
4. and whether the signal is too weak or too distributed for bounded config tuning.

## 3. Move selection must be anti-scattershot by design

`RSI-SCORE-2D` should require a smallest-effective-repair discipline.

The move selector should rank candidate moves by:

1. expected closure of the named breach locus,
2. likely-human and beneficial-traffic risk,
3. blast radius,
4. patch size,
5. and precedent from prior accepted episodes.

When no localized bounded move can plausibly close the gap, the correct answer should be `config_ring_exhausted` or equivalent, not repeated broad retuning.

## 4. Homeostasis break should be tied to new breach loci, not only rolling pressure

`RSI-SCORE-2C` should treat the appearance of a new successful breach locus as a first-class reason to break homeostasis, even before broader aggregates drift materially.

## 5. Operator projection should show board state, not one blended score

`RSI-SCORE-2E` should make the Game Loop show:

1. where the attack advanced,
2. which defenses are implicated,
3. whether evidence quality is high enough to act,
4. and what bounded repair or escalation the system selected.

# Planning Implications

1. `RSI-SCORE-2` should explicitly require terrain-localized exploit-progress scoring and diagnosis output.
2. The older recursive-game move-selection contract should be tightened so "shortfall attribution" means locus-level attribution, not only benchmark-family attribution.
3. The TODO closure gates should reject any implementation that still recommends broad config changes without a named breach locus and causal basis.

# Recommended Acceptance-criteria Direction

The next planning revision should require that:

1. a meaningful attacker regression is machine-visibly localized to one or more breach loci,
2. each controller-grade recommendation cites those loci and the bounded repair surface it intends to move,
3. broad multi-family config changes require explicit distributed-failure evidence,
4. and the Game Loop makes this distinction legible to operators.
