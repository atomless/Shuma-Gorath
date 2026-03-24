# Reference Stance And Run-To-Homeostasis Review

Date: 2026-03-24
Status: Proposed planning driver

Related context:

- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md`](2026-03-23-tuning-tab-taxonomy-posture-matrix-and-policy-archetypes-review.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](../plans/2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

External references:

- [AlphaZero: A general reinforcement learning algorithm that masters chess, shogi, and Go through self-play](https://pubmed.ncbi.nlm.nih.gov/30523106/)
- [OpenAI competitive self-play](https://openai.com/index/competitive-self-play/)
- [Curriculum Learning for Reinforcement Learning Domains: A Framework and Survey](https://www.jmlr.org/papers/v21/20-212.html)

# Purpose

Decide how Shuma should structure the next-stage recursive self-improvement regimen:

1. what stance should act as the first development reference game,
2. when a recursive-improvement episode should stop,
3. how later relaxed operator stances should be introduced,
4. and how strict defensive capability should remain a regression anchor when later code evolution begins.

# Findings

## 1. Product stance and development reference stance are different concerns

The earlier tuning-surface work correctly treats per-category posture as a site-owned product decision.

That remains true.

An operator-facing stance such as:

1. `Human-only / private`,
2. `Search-visible, AI-restricted`,
3. `Agent-friendly, scraper-hostile`,
4. or `Open access / low friction`

is part of the product contract.

But the user is right that recursive-improvement development does not need to begin by optimizing across all of those rule sets at once.

Conclusion:

1. Shuma should distinguish the **product stance space** from the **development reference stance** used to mature the recursive-improvement loop itself.

## 2. `Human-only / private` is the strongest first reference game

For development purposes, `Human-only / private` has several advantages over a more permissive mixed stance:

1. every non-human category is adversarial from the optimizer's perspective,
2. the objective surface is narrower and less internally conflicted,
3. the loop must still learn how to distinguish likely humans from the full non-human taxonomy,
4. and the feedback signal is cleaner while the recursive loop itself is still being stabilized.

This does **not** imply that `Human-only / private` should become Shuma's universal product default.

It means:

1. it is the cleanest first training or development environment for the recursive loop,
2. much like a simpler fixed game is often the right initial learning arena in self-play systems.

The external literature supports this shape:

1. AlphaZero improves inside a fixed rule set and repeatedly learns from self-play outcomes.
2. OpenAI's competitive self-play note emphasizes that self-play keeps the difficulty at a useful level for continued improvement.
3. Curriculum-learning literature supports beginning with a narrower or more learnable task regime before broadening task distributions.

Conclusion:

1. `Human-only / private` should be the first development reference stance for Shuma's later recursive-improvement program.

## 3. Karpathy's "never stop" insight should become run-to-homeostasis

Karpathy's loop keeps going because useful learning signal still exists.

That idea is valuable, but Shuma needs a bounded operational form.

The right adaptation is not literal infinite operation.
It is **run-to-homeostasis**:

1. keep iterating while the chosen target stance is still not satisfied,
2. and while recent completed cycles still show meaningful improvement,
3. and while no hard guardrail or budget stop has fired.

For Shuma, a cycle means a fully completed evaluate -> propose -> apply or preview -> watch-window -> compare outcome.

Conclusion:

1. later recursive-improvement episodes should continue until target-not-met and progress-not-flat are both true,
2. rather than stopping after one change or running without any plateau logic.

## 4. Plateau detection must be confidence-shaped, not eyeballed

The user's "last ten cycles have not entirely flattened out" intuition is strong.

But this must be formalized.

Shuma's later loop should not stop on a subjective reading of trend lines.
It should stop based on a bounded plateau detector over recent completed cycles.

The right direction is:

1. examine the last 10 completed watch-window judgments,
2. measure whether improvement remains above the loop's observed noise floor,
3. and treat near-zero confidence improvement as homeostasis.

This is consistent with the `pi-autoresearch` lesson already captured in the earlier autoresearch review: noisy domains need confidence-aware selection, not raw delta worship.

Conclusion:

1. later stopping logic should use a rolling, confidence-shaped homeostasis detector over recent completed cycles.

## 5. Later stance sweeps should be deliberate relaxations from the strict reference stance

Once the recursive loop works well against the strict reference stance, Shuma should broaden its development regimen.

But the right next move is not to optimize all posture permutations equally from day one.

The cleaner sequence is:

1. stabilize on the strict reference stance,
2. then introduce relaxed preset stances one by one,
3. and learn what additional config or code changes are required to support those permissive rule sets without losing the strict baseline capability.

This also matches the product structure we already want:

1. stance presets are useful product entry points,
2. but they are different rule sets,
3. and they should be explored after the base loop has proven itself on the narrowest general defensive game.

Conclusion:

1. relaxed preset sweeps should come after strict-stance stabilization, not before it.

## 6. The strict reference stance should remain a regression anchor for later code evolution

The user's refinement here is especially important.

Even after Shuma begins optimizing more permissive stances, any later code-evolution loop should continue to refer back to the strict reference stance.

Why:

1. code changes that help a permissive stance may accidentally weaken core defensive capability,
2. and Shuma needs a standing benchmark that says it can still robustly defend the "deny or heavily suppress all non-human traffic" regime with bounded human friction.

Conclusion:

1. once code evolution begins, the strict reference stance should remain a mandatory regression anchor, not just a historical training phase.

## 7. This proposal does not change the current seeded product default

Shuma's current seeded objective profile in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) is already a mixed public-web defensive stance.

That is fine.

This review is not recommending a product-default change.

It is recommending a **development and recursive-improvement methodology**:

1. use the strict stance first to improve the loop,
2. then broaden the stance set later.

# Decisions

1. Distinguish **development reference stance** from **operator product stances**.
2. Use `Human-only / private` as the first development reference stance for later recursive-improvement work.
3. Define the later stopping rule as **run-to-homeostasis**, not one-shot and not literal forever.
4. Formalize homeostasis over the last 10 completed watch-window cycles with confidence-aware plateau detection.
5. After strict-stance stabilization, add deliberate preset sweeps over common relaxed stances.
6. When code evolution begins, keep the strict reference stance as a mandatory regression anchor.
7. Do not change the seeded product default merely because the development reference stance is stricter.

# Result

The later recursive-improvement program should look like this:

1. choose a strict reference stance,
2. run the loop until target-not-met and progress-not-flat are both true,
3. stop when the loop reaches homeostasis or a hard guardrail,
4. then broaden to relaxed preset sweeps,
5. and require later code evolution to continue passing the strict reference stance as a regression anchor.

This gives Shuma:

1. a simpler first game,
2. a clearer learning signal,
3. a durable base-defense benchmark,
4. and a cleaner path from strict defense capability toward more permissive product stances.
