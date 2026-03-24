# Recursive Self-Improvement Game Loop Definition And Move-Selection Review

Date: 2026-03-24
Status: Proposed planning driver

Related context:

- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md`](2026-03-24-controller-tunable-config-surface-and-hard-boundaries-review.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Primary external sources inspected:

- [Hyperagents paper (arXiv 2603.19461)](https://arxiv.org/abs/2603.19461)
- [facebookresearch/HyperAgents](https://github.com/facebookresearch/HyperAgents)

# Purpose

Inspect Shuma's planned recursive self-improvement "game loop" and determine whether the game is already well defined enough to support later real recursive self-improvement.

This review focuses on four questions:

1. are the game rules explicit and immutable,
2. is the scorecard clear enough to measure progress honestly,
3. do current shortfalls point cleanly enough to appropriate changes,
4. and what methodology from Meta's `HyperAgents` is actually worth importing into Shuma's later recursive-improvement path.

# Executive Summary

What makes Shuma's bot-defence loop a real game is:

1. fixed rules,
2. fixed payoffs,
3. legal moves,
4. an independent judge,
5. and memory of prior episodes.

Shuma already has many of the right pieces for a recursive-improvement game:

1. an operator-owned target contract in `operator_objectives_v1`,
2. a machine-first evaluator in `operator_snapshot_v1` and `benchmark_results_v1`,
3. a bounded config action surface in `allowed_actions_v1`,
4. a recommend/apply/watch/rollback loop,
5. and a later reference-stance methodology.

But it does **not** yet have a single explicit game contract tying those pieces together.

In particular:

1. the rules are only partly canonical until the controller-mutability work lands,
2. the evaluator is strong but not yet formalized as the sacred episode scorecard for later recursive-improvement phases,
3. current shortfall attribution is still too coarse,
4. and the loop does not yet have an explicit stepping-stone archive or move-selection policy beyond bounded heuristics.

The Meta paper is useful here in a very specific way.
It reinforces that a self-improving system needs:

1. fixed evaluation,
2. a growing archive of prior attempts,
3. durable performance tracking and memory,
4. and clear separation between the objective and the search policy.

But the released `HyperAgents` code is also a warning:

1. the meta agent is allowed to "modify any part of the codebase",
2. the released parent selector currently chooses randomly from valid candidates,
3. and even the paper's own appendix shows that modifiable parent selection still does not outperform the carefully handcrafted selector.

So the right lesson for Shuma is **not** "let the loop freely rewrite itself".
It is:

1. define the game explicitly,
2. keep the evaluator sacred,
3. keep the move set narrow,
4. make move selection more deliberate than the current heuristic pressure buckets,
5. model later automation as attacker, defender, and judge rather than as two unconstrained agents,
6. and only then reopen later LLM-backed diagnosis or code-evolution phases.

# Findings

## 1. Shuma already has game pieces, but not yet one formal game contract

Today the pieces are spread across several machine-first contracts:

1. the operator-owned rule set lives in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs),
2. benchmark-family evaluation and escalation hints live in [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs),
3. bounded recommendation logic lives in [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs),
4. and bounded patch shaping lives in [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs).

That is good architectural progress.

But the current system still lacks one explicit answer to:

1. what exactly the immutable rules are,
2. what counts as the authoritative scorecard for a loop episode,
3. what the legal move set is,
4. how shortfalls become candidate moves,
5. how accepted and rejected moves are remembered,
6. and what stop rule ends a later recursive-improvement episode.

Conclusion:

1. Shuma's bounded loop is real,
2. but the later recursive-improvement "game" still needs its own canonical contract.

## 2. The game rules are only fully clear once the controller-mutability policy lands

The current rule set is conceptually correct:

1. `operator_objectives_v1` defines what success means,
2. and the user is right that the loop must never be allowed to mutate those targets.

But this is not yet fully expressed as one canonical enforced ring across the whole writable surface.

That is exactly what the new controller-mutability work is for:

1. `operator_objectives_v1` must remain the rule set,
2. hard-never config must stay outside the loop,
3. and only a bounded controller-tunable ring may become legal moves.

Until [`../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md`](../plans/2026-03-24-controller-mutability-policy-and-allowed-action-surface-implementation-plan.md) lands, the game rules are directionally clear but not yet final enough for later autonomous recursive-improvement phases.

Conclusion:

1. the game's immutable rules are conceptually known,
2. but the repo still needs the mutability policy to make them authoritative.

## 3. The evaluator is promising, but it is not yet expressed as the later sacred game scorecard

Shuma is already much better positioned than most recursive-improvement systems because it has a real machine-first evaluator:

1. likely-human friction budgets,
2. suspicious forwarded request, byte, and latency budgets,
3. representative adversary-effectiveness results,
4. verified-identity posture and no-harm checks,
5. category-posture evaluation,
6. and prior-window comparison.

That is exactly the kind of fixed evidence boundary Shuma should preserve.

This aligns strongly with the `HyperAgents` paper's insistence on a fixed `Evaluate(·, T)` procedure and held-out final tests rather than letting the mutator redefine success ([paper](https://arxiv.org/abs/2603.19461)).
The paper also explicitly keeps test tasks held out from self-modification feedback and parent selection.

But Shuma has not yet elevated its current evaluator into one later explicit episode scorecard contract that says:

1. these are the metrics that define progress,
2. these are the regression anchors,
3. these are the protected-evidence or no-harm gates,
4. and these are the bounded recent-loop comparisons that count for homeostasis.

Conclusion:

1. the evaluator substrate is strong,
2. but later recursive-improvement work still needs a formal scorecard contract over it.

## 4. Current shortfall attribution is still too coarse to count as a well-defined game policy

This is the most important current gap.

Today:

1. [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) maps missed benchmark families to broad candidate action families,
2. [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs) collapses primary pressure into only:
   1. `ReduceLikelyHumanFriction`, or
   2. `ReduceSuspiciousOriginCost`,
3. and [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs) then chooses from a static family-priority order.

That works for the current bounded loop, but it is not yet precise enough for a mature recursive-improvement game.

Why:

1. missed `suspicious_forwarded_latency_share` and missed `suspicious_forwarded_request_rate` do not necessarily imply the same best next move,
2. per-category posture shortfalls are not the same as global suspicious-origin-cost misses,
3. verified-traffic harm and likely-human friction are safety constraints, not just one more "pressure" bucket,
4. and the benchmark family to action-family mapping currently still includes drift that the mutability work intends to close.

The clearest example is that `non_human_category_posture` currently maps to `robots_policy` and `verified_identity` candidate action families in [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs), even though the new mutability boundary work explicitly treats those surfaces as operator-owned or hard-never.

Conclusion:

1. current benchmarks do **not** yet point to exact config tweaks,
2. and Shuma needs an explicit intermediate move-selection policy rather than assuming the current heuristic bridge is already the whole game.

## 5. `HyperAgents` reinforces evaluator sacredness and archive-based search, but it does not justify broad self-editing

The paper's most useful lessons for Shuma are:

1. keep the evaluator fixed,
2. keep an archive of stepping stones,
3. let performance tracking and durable memory improve search quality,
4. and recognize that open-ended search depends on more than hill-climbing the latest variant.

Those are directly relevant to Shuma's later need for:

1. accepted and rejected move history,
2. episode baselines,
3. benchmark-family trend memory,
4. and eventually a stepping-stone archive over bounded controller actions and later code-change candidates.

But the paper is careful in ways that matter.
In the main results, parent selection remains handcrafted and outside mutation.
The appendix reports that allowing the system to modify parent selection improved over pure random selection but still did not beat the handcrafted selector.

That matters for Shuma because it argues **against** starting with self-modifying move-selection logic.

Conclusion:

1. Shuma should import the archive and evaluator lessons,
2. but keep the move-selection policy explicit and manually ratified for longer than `HyperAgents` enthusiasts might first assume.

## 6. The released `HyperAgents` repo is a warning about broad mutation and under-specified selection

The released code is looser than the paper reads.

In [`meta_agent.py`](https://github.com/facebookresearch/HyperAgents/blob/main/meta_agent.py), the instruction is simply to "Modify any part of the codebase".
And in [`select_next_parent.py`](https://github.com/facebookresearch/HyperAgents/blob/main/select_next_parent.py), the selector currently returns a random valid parent even though candidate scores and child counts are computed first.

For Shuma, that is valuable mostly as a caution:

1. broad mutation surfaces make attribution and safety much harder,
2. vague or weak parent or move selection pushes too much burden onto luck and archive breadth,
3. and released code can embody a much weaker discipline than the associated paper implies.

Conclusion:

1. Shuma should stay substantially stricter than the released `HyperAgents` code on both mutation scope and move-selection clarity.

## 7. Sensible shortfall-to-change selection in Shuma needs an explicit intermediate policy layer

The user's question here is exactly the right one:
how do we know which changes befit which shortfalls?

The answer should **not** be "one benchmark miss directly names one config variable".
That is too brittle for a real defense system.

The better structure is an explicit intermediate policy layer:

1. benchmark shortfalls identify one or more **problem classes**,
2. each problem class maps to a bounded eligible action-family set,
3. each action family declares:
   1. expected impact direction,
   2. human-friction risk,
   3. tolerated-traffic risk,
   4. required prerequisites,
   5. and the exact bounded patch paths it may use,
4. move selection then chooses the smallest-risk or highest-confidence candidate inside that eligible set,
5. and later accepted canary outcomes become evidence for future move ranking.

That is much closer to a real game policy:

1. rules stay fixed,
2. the evaluator stays fixed,
3. the legal move set stays bounded,
4. but the search over legal moves can still improve over time.

Conclusion:

1. Shuma needs a canonical shortfall-attribution and move-selection contract,
2. not just a family-name bridge and a static priority list.

## 8. Missed target -> exact config or code is tractable in tiers, not as one global problem

The user's "do missed targets point to the exact config to tweak or code that may need changing?" question has a real answer:
this is **partly tractable**, but only if Shuma treats it as a tiered problem.

There are three useful tractability bands:

1. **high tractability**
   - bounded monotonic knobs with small blast radius,
   - where a missed target can justify an exact next move such as stepping one threshold up or down.
2. **medium tractability**
   - the miss points to a problem class and a bounded family set,
   - but not one exact path,
   - so the system needs an intermediate action policy to choose among safe candidate families.
3. **low tractability**
   - the miss is real, but config-space moves are exhausted, ambiguous, or repeatedly ineffective,
   - so the issue belongs in code evolution, simulator quality, or identification quality rather than in another config tweak.

That means later recursive-improvement planning should not chase a fantasy where every benchmark miss deterministically names one exact variable.
It should instead:

1. be exact when the move really is exact,
2. be family-level when the causal surface is broader,
3. and escalate to code or capability gaps when config-space evidence is exhausted.

Conclusion:

1. exact config selection is tractable for some bounded local knobs,
2. family-level move selection is the main tractable middle layer,
3. and repeated unresolved misses should be treated as code, capability, or simulator problems rather than disguised config problems.

## 9. Two agents alone do not make the right game; Shuma needs attacker, defender, and judge

The user is directionally right that putting frontier-model agents on both sides would make Shuma much closer to a real arms race.

But two agents by themselves are not enough.
What makes the loop a proper game is:

1. fixed rules,
2. bounded legal moves,
3. payoffs or scorecard,
4. and an independent judge.

For Shuma, the right later structure is:

1. **attacker agent**
   - evolves adversary behavior inside the sim harness,
2. **defender agent**
   - proposes bounded config moves and later reviewed code moves,
3. **judge**
   - remains the machine-first evaluator over budgets, adversary effectiveness, no-harm guardrails, replay evidence, and regression anchors.

This is the right analogue of the manual real-world loop:

1. attackers evolve,
2. operators observe outcomes,
3. defenders adapt,
4. and reality judges the result.

Automation accelerates that loop, but only if the judge stays outside both players.

Conclusion:

1. later frontier-model agents on both sides would make Shuma's loop more realistic and much faster,
2. but the crucial architecture is triadic, not merely dual-agent,
3. because the judge must remain independent of both players.

## 10. Real recursive self-improvement in Shuma needs an episode archive, not just latest-run state

The earlier `run-to-homeostasis` work already identified the need to reason over recent completed cycles.
This review sharpens that into a structural requirement:

1. later recursive-improvement episodes need an archive or ledger of candidate moves and outcomes,
2. not just the latest recommendation and current config.

That archive should eventually include:

1. target stance,
2. baseline scorecard,
3. candidate move,
4. accepted or refused status,
5. canary/watch-window outcome,
6. benchmark-family deltas,
7. rollback or retain status,
8. and any hard guardrail trigger.

This is the Shuma analogue of the `HyperAgents` stepping-stone archive, but grounded in operational defense outcomes rather than arbitrary code variants.

Conclusion:

1. later recursive self-improvement in Shuma should be archive-backed and episode-shaped,
2. not merely a perpetual stream of local one-step decisions.

## 11. The strict reference stance should become both a curriculum and a regression anchor

The earlier reference-stance review remains right:

1. `Human-only / private` is the best first development reference game,
2. later relaxed stances should be deliberate secondary sweeps,
3. and code evolution should keep the strict reference stance as a regression anchor.

This review adds one more implication:

1. the game contract itself should name which scorecards are:
   1. primary optimization targets,
   2. safety constraints,
   3. and regression anchors.

Otherwise the later loop risks blending:

1. product stance,
2. development reference stance,
3. and held-out regression contexts

into one ambiguous objective surface.

Conclusion:

1. later recursive-improvement planning should explicitly distinguish optimization target, safety gates, and regression anchors.

# Decisions

1. Define one explicit Shuma recursive-improvement game contract before reopening later autonomous recursive-improvement phases.
2. Treat `operator_objectives_v1` plus the canonical controller-mutability policy as the immutable rules of the game.
3. Treat the machine-first benchmark and snapshot stack as the sacred evaluator; the loop must not redefine success.
4. Add an explicit shortfall-attribution and move-selection policy between benchmark misses and bounded patch proposals.
5. Keep the legal move set narrow and config-bounded until that policy is explicit and proven.
6. Treat later recursive-improvement architecture as attacker agent, defender agent, and independent judge rather than only two agent roles.
7. Add an episode archive or stepping-stone ledger before later LLM-backed recursive-improvement phases.
8. Carry the earlier `Human-only / private` reference-stance and run-to-homeostasis methodology forward as part of that game contract, not as a floating later idea.
9. Require later code-evolution work to optimize target stances while continuing to pass the strict reference stance as a regression anchor.

# Result

The right next step for Shuma's recursive-improvement architecture is **not** to widen autonomy immediately.

It is to explicitly define the game:

1. immutable rules,
2. sacred evaluator,
3. legal move set,
4. shortfall-attribution policy,
5. move-selection policy,
6. attacker/defender/judge role separation,
7. archive and memory shape,
8. and stop or homeostasis logic.

Only after that should later recursive-improvement phases reopen.

That will give Shuma a clearer game than it has today, and a stricter one than the released `HyperAgents` code currently demonstrates.
