# Scorecard, Player-Protocol, And Held-Out Evaluation Separation Review

Date: 2026-03-24
Status: Proposed

Related context:

- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`2026-03-24-llm-player-role-decomposition-review.md`](2026-03-24-llm-player-role-decomposition-review.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Identify the remaining protocol-level underplanning on the path to Shuma's full recursive-improvement game and turn it into explicit follow-on contracts:

1. a canonical judge scorecard,
2. canonical player protocol schemas,
3. and a canonical separation between protected training evidence and held-out evaluation evidence.

# External Signals

Several external sources point in the same direction:

1. [Anthropic: Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents) emphasizes that when evaluating an agent, the harness and model are evaluated together, which supports making the player protocol a first-class contract rather than an implicit implementation detail.
2. [Anthropic: Challenges in evaluating AI systems](https://www.anthropic.com/news/evaluating-ai-systems) emphasizes repeatable quantitative evaluations and mitigations rather than vague impressions, which supports freezing a canonical judge scorecard.
3. [Google DeepMind: Evaluating Frontier Models for Dangerous Capabilities](https://deepmind.google/research/publications/evaluating-frontier-models-for-dangerous-capabilities/) reinforces the value of structured evaluation programs over ad hoc probing.
4. [Google DeepMind: Introducing the Frontier Safety Framework](https://deepmind.google/discover/blog/introducing-the-frontier-safety-framework/) reinforces the need to separate capability evaluation from mitigation decisions and to keep critical evaluation criteria explicit.
5. [Anthropic: Designing AI-resistant technical evaluations](https://www.anthropic.com/engineering/AI-resistant-technical-evaluations) is a useful reminder that evaluation surfaces degrade if the optimizing system can overfit to them too directly, which supports a held-out evaluation ring distinct from player-visible evidence.

# Findings

## 1. The judge exists, but the episode scorecard is still not explicit enough

The current recursive-improvement planning chain already says that Shuma needs:

1. fixed payoffs,
2. an independent judge,
3. and run-to-homeostasis logic.

But the repo still does not have one later explicit scorecard contract that says:

1. which metrics are optimization targets,
2. which metrics are hard no-harm gates,
3. which contexts are regression anchors only,
4. what counts toward homeostasis,
5. and which metrics are only explanatory diagnostics rather than reward signals.

That is the remaining judge-side planning gap.

## 2. The player roles are decomposed, but the wire format between players and judge is not

The new player-role work correctly decomposes:

1. the attacker into black-box, episode, and runtime slices,
2. and the defender into contract, recommendation-only, and bounded-autonomy slices.

But Shuma still lacks one canonical player-protocol layer describing:

1. observation envelopes,
2. action or proposal envelopes,
3. refusal and escalation envelopes,
4. trace and receipt envelopes,
5. and which parts of those schemas are shared across the attacker and defender roles.

Without that layer, the player roles are now conceptually correct but still not protocol-complete.

## 3. Protected evidence is planned, but held-out evaluation is still too implied

The repo is already strong on one boundary:

1. raw synthetic or one-off LLM outputs are not tuning truth,
2. replay-promoted or equivalently confirmed lineage becomes protected evidence.

But the full game also needs a second boundary:

1. what evidence the players may train or plan against,
2. what evidence the judge may use for scoring but keep withheld,
3. and what contexts remain regression anchors rather than optimization surfaces.

That second ring matters because a self-improving loop can quietly overfit even a well-designed protected-evidence surface if all of the judge's evaluation basis becomes player-visible.

## 4. The three underplanned areas are cross-cutting prerequisites, not optional polish

These are not cosmetic refinements.
They are what stops the later game from devolving into:

1. fuzzy rewards,
2. ad hoc player wiring,
3. and evaluation leakage.

So they should be treated as blocked prerequisites for the later fully working game loop, not as late cleanup.

# Decisions

1. Add `RSI-SCORE-1` for the canonical judge scorecard.
2. Add `RSI-PROTO-1` for canonical attacker and defender protocol schemas.
3. Add `RSI-EVAL-1` for protected-vs-held-out evaluation separation.
4. Make `SIM-LLM-1A`, `OVR-AGENT-2A`, `OVR-AGENT-2B`, `OVR-AGENT-2C`, `RSI-METH-1`, and `OVR-CODE-1` depend on the appropriate subset of those contracts.
5. Keep these items blocked and downstream of the current mainline operator-facing work; do not let them pull `MON-OVERHAUL-1` or any future broader Tuning re-expansion out of order.

# Required Follow-On Work

1. Write a companion plan that defines `RSI-SCORE-1`, `RSI-PROTO-1`, and `RSI-EVAL-1`.
2. Update the recursive-improvement game plan and player-role plan so they explicitly consume those contracts.
3. Update the main loop-closure plan and blocked backlog so those items become visible blockers on the path to the full game loop.

# Result

After this follow-on, the remaining underplanned path should be much narrower:

1. judge score semantics,
2. player wire protocols,
3. and evaluation visibility boundaries

rather than any broader architectural confusion about what the game is supposed to be.
