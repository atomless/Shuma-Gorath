Date: 2026-03-27
Status: Proposed planning driver

Related context:

- [`2026-03-26-ideal-rsi-game-loop-scoring-review.md`](2026-03-26-ideal-rsi-game-loop-scoring-review.md)
- [`2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
- [`2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
- [`2026-03-27-game-loop-category-posture-scoring-audit.md`](2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

# Purpose

Freeze the non-negotiable rules that should govern Shuma's Game Loop before further scoring, diagnoser, controller, or later LLM-referral work proceeds.

The central clarification is that the Game Loop is not an abstract benchmark board.
The host site is the board.
The adversary is the invading traffic.
Shuma's configurable defense surfaces are the pieces that may be repositioned or retuned.
The judge must therefore score where the adversary advanced on the host, what cost it imposed there, which defense surface fell short, and whether the next move should be a bounded config change, a code referral, or no move at all.

# Foundational Rules

## 1. The host site is the board

The defended routes, pages, forms, challenge stages, and resource-consuming surfaces are the actual terrain.
The loop should not treat sitewide aggregates as if they were the board itself.

## 2. Shuma's defense surfaces are the movable pieces

The legal move ring consists of bounded changes to Shuma's defense surfaces and their tunable parameters.
The loop is only useful if it can explain which piece is being moved, why, and at which breach locus.

## 3. The primary objective is dual

The loop must reduce the host cost imposed by non-human traffic while also minimizing friction on real humans.
The first strict phase remains all-out exclusion for adversary-sim lanes, but the architecture must still preserve human-friction measurement as a first-class objective rather than a forgotten later afterthought.

## 4. Sim traffic and real traffic must share the same judge path

This is the critical unbreakable rule.

Adversary-sim traffic must be treated exactly like real traffic at classification and scoring time.
The simulator may generate traffic patterns, browser behavior, concurrency, traversal, timing, proxy posture, and other observable attacker behavior.
It must not provide Shuma with privileged category truth, pre-identification, or convenience labels that real external traffic would never volunteer.

Implication:

1. `sim_profile`, `sim_lane`, worker fulfillment modes, or equivalent harness metadata may support harness control, replay, or offline audit,
2. but they must not become category truth or score inputs for the Game Loop,
3. and if Shuma cannot classify a request from its own observable evidence, the result must remain unscored rather than guessed.

## 5. Adversary lanes must be full-spectrum and attacker-faithful

The loop cannot benchmark itself against a tame adversary.
If Scrapling or the later LLM lane leaves attacker-relevant power unused, the loop will produce false confidence.
Any omitted attacker capability must therefore be explicitly justified as adding complexity without adding meaningful power against Shuma's defenses.

## 6. Non-human cost scoring must be breach-local and defense-local

The score is not only "how much suspicious traffic leaked."
It must show:

1. where the adversary advanced on the host,
2. what resource or interaction cost it consumed after that advance,
3. which defense surface was expected to stop, redirect, or raise attacker cost there,
4. and what shortfall was observed.

Without that tuple, config changes remain too close to scattershot tuning.

## 7. The judge output must be interpretable as config vs code

The system must be able to tell the difference between:

1. a local shortfall that bounded config changes may plausibly close,
2. a distributed or ambiguous shortfall that needs more evidence,
3. and a genuine code gap where the bounded config ring is exhausted.

That means breach-local scoring and diagnosis are prerequisites for later code-evolution work rather than optional polish.

## 8. The config loop must remember what it tried

If a config change is applied and no improvement is observed, the loop should roll it back, rerun, and avoid proposing near-equivalent failed moves indefinitely.
The move selector therefore needs trial memory, rollback lineage, and failure memory tied to the same named breach loci.

## 9. Code evolution is a later second ring, not a hidden side effect

Later code-change suggestions belong in a separate, explicitly bounded ring.
That ring should consume:

1. machine-first judge outputs,
2. named breach loci,
3. evidence quality and diagnosis confidence,
4. config-ring exhaustion or explicit code-gap referral,
5. and strict regression anchors including the human-only reference stance.

The frontier LLM belongs in that later code-evolution review and proposal flow as a bounded player under the independent judge, not as a hidden heuristic inside earlier config tuning.

## 10. Human friction must be measured from human evidence

Human friction cannot be inferred from adversary-sim traffic.
It needs its own calibration ring over real human traversal or an explicitly human-operated test path.

At minimum the later human-friction ring should measure:

1. whether a human reached the intended content,
2. challenge or verification completion success,
3. challenge completion latency and repeated-challenge loops,
4. added bytes or latency on the human path where measurable,
5. and abandonment or forced-retry signals where Shuma has truthful evidence.

# Current Gaps Against These Rules

## 1. Shared-path truth is still fragile in practice

The March 27 category-posture audit already proved that the repo can drift toward projected or misleading category truth when exact Shuma-side classification is absent.
That is exactly the class of failure these rules are intended to forbid.

## 2. The Game Loop still mixes different planes too loosely

Origin leakage, exploit progress, surface-contract satisfaction, category posture, and tuning readiness are still too easy to read as one story.
That makes it hard for the operator to see what the board state actually is.

## 3. Cost attribution is still not local enough

The loop can now preserve breach loci better than before, but it still needs a more explicit record of what cost was consumed past the expected stop point and which defense surface is accountable for that shortfall.

## 4. Failed config memory is not yet the center of move selection

Rollback lineage exists, but the doctrine that near-equivalent failed moves must be remembered and actively excluded still needs to be made explicit in the refactoring chain.

## 5. Code-evolution and human-friction rings are under-specified

The repo already has later LLM player decomposition and human-calibration research, but those pieces are not yet re-expressed from the board-state doctrine outward.
That makes the future shape understandable only if the reader mentally stitches several plans together.

# Decisions

1. Treat the nine rules above as doctrine, not optional design taste.
2. Tighten the active Game Loop refactor chain so it explicitly inherits:
   1. board-state locality,
   2. shared-path sim vs real traffic truth,
   3. breach-local host-cost attribution,
   4. anti-scattershot move selection,
   5. explicit config trial memory,
   6. later bounded frontier-LLM code-evolution referral,
   7. and later human-friction calibration over real human evidence.
3. Keep `RSI-SCORE-2F` as the immediate honesty repair because projected category posture already violates the shared-path doctrine if it pretends to know more than Shuma can actually infer.
4. Reframe later code-evolution work so it starts only from machine-first code-gap referral and config-ring exhaustion, never from raw dashboard impression or simulator metadata.
5. Reframe later human-friction work as a separate calibration ring that must not water down the strict adversary exclusion proof.

# Planning Implications

The next refactoring tranche should require:

1. explicit separation of board-state planes in the Game Loop,
2. explicit shared-path truth tests that fail if simulator metadata leaks into category or score truth,
3. breach-local host-cost and shortfall accounting,
4. remembered failed config moves tied to named loci,
5. an explicit handoff contract from code-gap referral to later frontier-LLM code suggestions,
6. and a human-friction measurement contract that stays separate from sim-only proof.
