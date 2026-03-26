Date: 2026-03-26
Status: Proposed planning driver

Related context:

- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md`](2026-03-24-game-loop-budget-visualization-and-category-target-achievement-review.md)
- [`2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
- [`../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md`](../plans/2026-03-24-scorecard-protocol-and-held-out-eval-separation-plan.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)

Primary external sources inspected:

- [Google SRE Workbook: Alerting on SLOs](https://sre.google/workbook/alerting-on-slos/)
- [Holzapfel et al. 2024: Event-Triggered Safe Bayesian Optimization on Quadcopters](https://proceedings.mlr.press/v242/holzapfel24a.html)

# Purpose

Explore what an ideal RSI-ready Game Loop scoring system should look like for Shuma if the goal is not merely bounded config recommendation plumbing, but fast, trustworthy adaptation when a red-team adversary suddenly finds a new path through the defenses.

# Assumptions

1. The fast loop remains a bounded config-tuning loop over the controller-safe move ring.
2. Code evolution remains a slower second ring with a separate proof threshold.
3. The judge should remain machine-first and independent.
4. The current `scalarization=forbidden` direction in the Shuma scorecard is correct and should be preserved.

# Executive Summary

Shuma should not evolve toward one bigger blended score.

The ideal RSI-ready judge is a hierarchical score vector with four distinct layers:

1. hard no-harm guardrails,
2. adversary-outcome and exploit-progress objectives,
3. evidence-quality and diagnosis-confidence gates,
4. urgency and change-detection signals.

The current system already has the right instinct in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs): explicit optimization targets, hard guardrails, regression inputs, diagnostics, and `scalarization=forbidden`.

What is missing is not a prettier version of the current metrics.
What is missing is a judge that can tell the difference between:

1. aggregate suppression,
2. truthful attacker-surface exercise,
3. exploit depth,
4. confidence in the evidence,
5. and urgent new bypass behavior that should break homeostasis immediately.

The recommended path is therefore:

1. keep the score vector structure,
2. deepen the attacker-success model,
3. add evidence-quality gates,
4. add burn-rate and novelty urgency scoring,
5. and make the move selector reason over expected effect vectors rather than only family buckets.

# What Is Not Good Enough In The Current System

## 1. Aggregate suspicious-origin suppression is too easy to confuse with true defensive success

Today the strict loop is heavily driven by suspicious forwarded request, byte, and latency leakage.

That is useful, but it is only one layer of truth.

An ideal loop must distinguish:

1. the attacker got nowhere because Shuma fail-closed early,
2. the attacker still truthfully exercised required defenses and failed where it should,
3. the attacker reached deep surfaces but was stopped late,
4. and the attacker now reliably succeeds at a formerly blocked stage.

The current system is too close to treating all of those as variations of "how much suspicious traffic leaked."

## 2. The scorecard is too weak on exploit depth and exploit chain completion

Shuma already has a defense-surface matrix for Scrapling in [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs).

But the main loop still lacks a first-class exploit-progress objective such as:

1. depth reached in the exploit chain,
2. reliability of bypass at each stage,
3. cost imposed on the attacker,
4. and whether a new surface transitioned from "historically blocked" to "now reliably passable."

Without that, the loop is strong at judging coarse suppression and weak at judging where the defensive perimeter actually failed.

## 3. The scorecard is too weak on evidence quality and causal confidence

An RSI loop should not optimize from weak or conflated evidence.

Today the game loop has:

1. category posture metrics,
2. adversary-effectiveness proxies,
3. and Scrapling surface receipts,

but it does not yet make evidence quality itself a judge-visible scored dimension.

It needs explicit answers to:

1. is this metric category-native or projected,
2. is the sample size sufficient,
3. is the evidence fresh,
4. did multiple personas reproduce the result,
5. and is the diagnosis confident enough to justify a config move instead of observe-longer or code-gap escalation.

## 4. Homeostasis is too static for sudden adversary improvements

The current homeostasis concept is valuable, but an ideal defensive RSI loop must break homeostasis aggressively when a new exploit appears.

In practical terms, homeostasis should not only mean:

1. recent cycles are not changing much,
2. and benchmark verdicts are flat.

It should also mean:

1. no new exploit path has appeared,
2. no attack-success burn rate is spiking,
3. and no trusted regression anchor has been crossed.

# Candidate Designs

## Option A: Patch the current family system

This approach would:

1. add Scrapling surface-contract metrics into the existing benchmark families,
2. improve the Game Loop copy,
3. and tighten the diagnoser thresholds.

Advantages:

1. smallest change,
2. low risk,
3. fits current architecture with minimal upheaval.

Disadvantages:

1. still leaves the loop overly family-centric,
2. still weak on exploit depth and urgency,
3. and likely keeps diagnosis too coarse for fine-grained RSI adaptation.

## Option B: Recommended: hierarchical score vector plus urgency and diagnosis confidence

This approach keeps Shuma's current scorecard discipline but upgrades the semantics.

The judge stays non-scalar, but each episode now produces a richer score vector over:

1. hard guardrails,
2. exploit outcome,
3. evidence quality,
4. urgency,
5. and move confidence.

Advantages:

1. aligns with the repo's existing scorecard direction,
2. gives the diagnoser a much better basis for fine-grained config moves,
3. and provides a clean handoff to later code-gap escalation.

Disadvantages:

1. materially more work than a local patch,
2. requires new benchmark and reconcile contracts,
3. and needs clearer judge vs diagnoser vs move-selector boundaries.

## Option C: Collapse everything into one scalar reward

This would try to produce one reward number and optimize directly against it.

Advantages:

1. superficially simple.

Disadvantages:

1. destroys interpretability,
2. hides tradeoffs,
3. makes no-harm regression easier to miss,
4. and directly conflicts with Shuma's existing `scalarization=forbidden` discipline.

Recommendation:

1. reject Option C,
2. use Option A only as a temporary repair slice,
3. and treat Option B as the real north star.

# Recommended Ideal Scoring Model

## 1. Keep lexicographic judge layers, not one blended reward

The current Shuma direction is already close to the right philosophy.

The ideal judge should evaluate in this order:

1. hard safety and truth gates,
2. exploit suppression and attacker-progress objectives,
3. adaptation urgency,
4. move confidence and diagnosis quality.

That means:

1. a guardrail failure should stop autonomous config progression even if adversary suppression improved,
2. exploit-success breakthroughs should outrank cosmetic budget improvements,
3. and low-confidence diagnosis should block config changes even when pressure is real.

In other words, the loop should decide:

1. is the state safe,
2. is the attacker winning anywhere important,
3. how urgent is the change,
4. how confident are we about the fix.

## 2. Replace coarse adversary scoring with exploit-progress scoring

The ideal attacker score is not just "blocked percentage."

It should capture:

1. surface reach,
2. exploit-chain depth,
3. success reliability,
4. attacker cost,
5. and novelty.

For each relevant surface or exploit stage, Shuma should track:

1. expected contract,
2. observed contract,
3. success rate,
4. median and tail latency added to the attacker,
5. attempts required per success,
6. and whether this stage represents a regression from the accepted baseline.

This would let the judge distinguish:

1. shallow nuisance probes,
2. partial bypasses,
3. and real exploit completion.

That matters because an RSI loop should optimize hardest against what most threatens the defended posture, not merely against what moves request counts.

## 3. Make evidence quality a first-class scored input

The ideal judge should never quietly optimize from low-confidence evidence.

Add an explicit evidence-quality layer with dimensions such as:

1. independence of measurements,
2. category-native versus projected attribution,
3. sample-size sufficiency,
4. persona diversity,
5. freshness,
6. and reproducibility.

This should behave like a tuning gate:

1. high-confidence exploit evidence can drive config changes,
2. medium-confidence evidence can trigger more probing or replay promotion,
3. low-confidence evidence should remain diagnostic only.

This is especially important for Shuma because the loop is meant to make fine-grained config changes.
Fine-grained moves require better evidence than "the run looked bad overall."

## 4. Add urgency and burn-rate scoring, not only current-state scoring

Google's SRE guidance on multiwindow burn-rate alerting is useful here as an analogy, not as a literal service-SLO transplant.

The key idea is:

1. current badness is not enough,
2. rate of budget consumption matters,
3. and multiwindow urgency is better than one threshold.

Shuma should treat exploit success the same way.

Instead of only asking "what is the current success rate of the adversary," it should also ask:

1. how quickly is exploit budget being consumed,
2. is the exploit getting worse quickly,
3. is this a short sharp spike or sustained success,
4. and does it justify page-level urgency, ticket-level urgency, or simple observation.

Recommended dimensions:

1. short-window exploit burn rate,
2. medium-window exploit burn rate,
3. short-window human-friction burn rate,
4. medium-window human-friction burn rate,
5. and novel-surface regression rate.

## 5. Add event-triggered change detection for new adversary capability

The ETSO paper is a strong conceptual fit for the "homeostasis until a new exploit appears" problem.

Its most useful lesson for Shuma is:

1. safe optimization should detect when expected performance and observed performance diverge,
2. then reset the optimization state and re-explore from a safe baseline.

Mapped into Shuma:

1. maintain an accepted homeostasis baseline,
2. detect a statistically meaningful exploit regression or novel bypass,
3. break homeostasis immediately,
4. revert to the last safe config baseline if needed,
5. and reopen fine-grained config search from there.

This is much closer to the desired behavior when an adversary suddenly improves:

1. do not just keep averaging old windows,
2. detect the change,
3. and adapt from the known-safe frontier.

## 6. Separate judge, diagnoser, and move selector more sharply

An ideal RSI loop needs three distinct roles:

1. judge: compute the score vector and verdict,
2. diagnoser: infer likely causes of the shortfall,
3. move selector: rank bounded config changes or escalate to code work.

Today those concerns are too compressed.

The improved design should make the diagnoser produce:

1. exploit cluster,
2. likely failing surfaces,
3. likely over-tight or under-tight controls,
4. evidence confidence,
5. and candidate action families.

Then the move selector should rank candidate patches by:

1. expected exploit reduction,
2. expected human-friction risk,
3. expected collateral risk to beneficial traffic,
4. confidence from prior episode history,
5. and patch size.

## 7. Teach the system when config tuning is no longer enough

An RSI loop that only knows "recommend config" or "observe longer" is not ideal.

It also needs an explicit config-insufficiency verdict.

Recommended criteria:

1. the exploit remains after the best safe config moves have been tried,
2. the exploit sits on a surface with no strong controller-tunable lever,
3. the required surface contract is still unsupported,
4. or the evidence suggests a missing detector rather than a mis-tuned threshold.

That verdict should produce a code-evolution referral with proof, not vague hand-waving.

## 8. Model homeostasis as "stable and defended," not merely "flat"

Ideal homeostasis is not just low movement in recent cycle judgments.

It should require:

1. hard guardrails satisfied,
2. exploit burn rates low,
3. no new exploit path or surface regression,
4. diagnosis confidence healthy,
5. and no unresolved code-gap referral.

Then, if a new adversary method appears, homeostasis should break immediately even if the old averages still look good.

# A Better Judge Layout For Shuma

## Hard guardrails

1. likely-human friction and hard-block ceilings,
2. beneficial and verified non-human no-harm,
3. config legality and mutability compliance,
4. evidence-quality minimums for autonomous tuning.

## Optimization targets

1. exploit success rate by adversary persona,
2. exploit-chain depth,
3. per-surface contract satisfaction,
4. suspicious-origin leakage budgets,
5. attacker cost-imposition metrics.

## Regression anchors

1. prior accepted baseline,
2. held-out exploit variants,
3. historical blocked surfaces that must stay blocked,
4. later human-traversal calibration anchors.

## Diagnostics only

1. raw lane volumes,
2. support and exactness details,
3. explanatory counters,
4. UI-friendly summaries.

# What This Would Change In Practice

If Scrapling suddenly develops a new way past Shuma's defenses, the improved loop would:

1. detect the regression quickly through exploit burn-rate and novelty scoring,
2. mark homeostasis as broken immediately,
3. confirm whether the exploit evidence is high confidence,
4. localize the regression to surfaces and likely control families,
5. rank the smallest safe config moves with the highest expected exploit reduction,
6. canary those moves,
7. retain or roll back based on the full score vector,
8. and escalate to code work if the config ring demonstrably cannot close the gap.

That is substantially better than the current "outside budget or inside budget" flow because it gives the system a principled answer to:

1. what failed,
2. how badly it failed,
3. how urgent it is,
4. how confident we are,
5. and whether config is even the right repair ring.

# Recommended Sequence For Shuma

1. First repair evidence truth:
   1. category-native adversary receipts,
   2. Scrapling surface-contract truth in controller-grade scoring.
2. Then add exploit-progress and per-surface benchmark families.
3. Then add evidence-quality and diagnosis-confidence gates.
4. Then add urgency scoring with multiwindow exploit and friction burn rates.
5. Then add event-triggered homeostasis break and safe re-optimization from the last accepted baseline.
6. Only after that should Shuma consider more aggressive automated move ranking or later code-evolution loops.

# Conclusion

The ideal RSI-ready scoring system for Shuma is not "more math on the current dashboard."

It is a more explicit defensive judge:

1. hard guardrails first,
2. attacker exploit progress second,
3. evidence quality third,
4. urgency and novelty fourth,
5. and bounded move selection over that richer state.

That would make the loop much better suited to:

1. fine-grained config tuning,
2. fast reaction to new adversary capability,
3. truthful escalation from config to code work,
4. and stable return to homeostasis once the exploit is closed.
