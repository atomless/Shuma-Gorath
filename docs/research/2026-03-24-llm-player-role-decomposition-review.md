# LLM Player-Role Decomposition Review

Date: 2026-03-24
Status: Proposed

Related context:

- [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md)
- [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md)
- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-review.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Make the later three-role recursive-improvement architecture concrete enough to plan truthfully:

1. the attacker role in the sim harness should be explicitly LLM-backed,
2. the defender role in the diagnosis or tuning loop should be explicitly LLM-backed,
3. the judge should remain machine-first and non-LLM,
4. and the blocked backlog should decompose the two LLM player roles at roughly the same granularity that the judge role now has.

# External Signals

The current research base supports the direction but also suggests stronger role separation than the repo currently manifests in TODO granularity:

1. [AlphaZero](https://arxiv.org/abs/1712.01815) and [OpenAI competitive self-play](https://openai.com/index/competitive-self-play/) support adaptive player improvement inside fixed rules and fixed payoffs rather than through unconstrained freeform search.
2. [Anthropic's red-teaming guidance](https://www.anthropic.com/news/challenges-in-red-teaming-ai-systems) supports iterative red-team, mitigation, and retest loops rather than one-shot probing.
3. [Anthropic's evaluation note](https://www.anthropic.com/news/evaluating-ai-systems) reinforces that evaluation needs to stay distinct from the systems being optimized against it.
4. [RvB: Automating AI System Hardening via Iterative Red-Blue Games](https://arxiv.org/abs/2601.19726) is directionally relevant because it frames red-versus-blue adaptation as a sequential game, but Shuma still needs a stronger external judge than the paper's more integrated red-blue framing.
5. [Anthropic's AI for Cyber Defenders write-up](https://red.anthropic.com/2025/ai-for-cyber-defenders/) is a useful reminder that defensive agents are strongest when they work over structured evidence and bounded remediation tasks instead of vague end-to-end autonomy.

# Findings

## 1. The repo already means "LLM-backed agents" for the two player roles

The current active planning chain already implies that:

1. the sim-side attacker role is the later `SIM-LLM-1` actor,
2. the controller-side defender role is the later `OVR-AGENT-2` diagnosis or config harness,
3. and the judge is the machine-first benchmark and Monitoring projection rather than a third agent.

That direction is explicit in:

1. [`2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md`](2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-review.md),
2. [`../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md`](../plans/2026-03-24-recursive-self-improvement-game-loop-definition-and-move-selection-plan.md),
3. and the `OVR-AGENT-2` blocker in [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md).

So the missing step is not conceptual agreement. The missing step is backlog and plan decomposition.

## 2. The attacker role is directionally clear, but still too umbrella-shaped

The repo already has a strong bounded prelude in `SIM-LLM-FIT-1` and the later blocked `SIM-LLM-1` item.

The missing explicit attacker-agent contract is:

1. what observations the attacker gets,
2. what actions or tools it may use,
3. what the black-box boundary forbids,
4. what an episode looks like,
5. what receipts or traces are persisted,
6. and how strategy memory is bounded.

Without that split, `SIM-LLM-1` still sounds like a large future actor rather than a sequence of implementation slices.

## 3. The defender role is less concrete than the attacker role

`OVR-AGENT-2` currently says a great deal about prerequisites and almost nothing at tranche granularity about the agent itself.

The missing explicit defender-agent contract is:

1. what sacred input bundle it receives from the judge,
2. what bounded move schema it may emit,
3. when it must refuse to act,
4. when it may escalate to code-gap territory,
5. whether it begins as recommend-only,
6. and how it later participates in bounded run-to-homeostasis episodes.

This is the biggest current role-planning gap in the recursive-improvement chain.

## 4. The judge should stay non-LLM and machine-first

The most important thing to preserve is not "three roles" in the abstract. It is the asymmetry:

1. attacker and defender may be LLM-backed search policies,
2. but the judge must remain external to them.

That means the later system should not be modeled as three agents.
It should be modeled as:

1. two LLM-backed players,
2. and one non-LLM evaluator.

This is why Shuma's existing benchmark, objectives, protected-evidence, and Monitoring work is so important.

## 5. The right next decomposition is player-side, not judge-side

The judge side now has a credible blocked breakdown:

1. `RSI-GAME-1A`,
2. `RSI-GAME-1B`,
3. `RSI-GAME-1C`,
4. and `RSI-ROLES-1`.

The attacker and defender sides should now gain matching blocked decomposition so that:

1. the attacker role is not just "future big runtime actor",
2. the defender role is not just "future broad autonomous planner",
3. and both roles become subordinate to the same explicit judge contract.

# Decisions

1. Use "agent" explicitly for the two later LLM-backed player roles:
   1. attacker agent in the sim harness,
   2. defender agent in the diagnosis or tuning loop.
2. Do not use "agent" for the judge.
3. Decompose the later attacker track into:
   1. attacker-agent black-box contract,
   2. attacker-agent episode harness and receipts,
   3. later full first-class runtime actor.
4. Decompose the later defender track into:
   1. defender-agent sacred input and bounded output contract,
   2. recommendation-only defender runtime,
   3. later bounded autonomous episode controller.
5. Keep `OVR-CODE-1` separate from the defender-agent work.
6. Update the active planning chain and blocked backlog so the later three-role model is no longer clear only in prose.

# Required Follow-On Work

1. Write a companion plan that decomposes the player-side LLM roles into blocked execution-ready slices.
2. Update the main loop-closure plan so the active recursive-improvement chain names those player-side slices explicitly.
3. Update the blocked backlog so `SIM-LLM-1` and `OVR-AGENT-2` stop being single large placeholders.
4. Update the reference-stance and recursive-game plans so they state plainly that:
   1. attacker and defender are LLM-backed players,
   2. judge is machine-first and non-LLM.

# Result

The repo should now move toward a cleaner recursive-improvement framing:

1. `SIM-LLM-1*` for the LLM attacker-agent track,
2. `OVR-AGENT-2*` for the LLM defender-agent track,
3. `RSI-GAME-1*` for the judge and scorecard contract,
4. `RSI-ROLES-1` for preserving the role boundaries,
5. and `OVR-CODE-1` as a later, distinct code-evolution path rather than part of the defender role itself.
