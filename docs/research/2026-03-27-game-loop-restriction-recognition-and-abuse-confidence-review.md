Date: 2026-03-27
Status: Proposed planning driver

Related context:

- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-27-game-loop-current-state-and-gap-review.md`](2026-03-27-game-loop-current-state-and-gap-review.md)
- [`2026-03-27-game-loop-category-posture-scoring-audit.md`](2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md`](../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`../plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md`](../plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md)
- [`../plans/2026-03-27-human-friction-calibration-ring-plan.md`](../plans/2026-03-27-human-friction-calibration-ring-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)

# Purpose

Freeze the next Game Loop architecture clarification before more Scrapling or scoring work lands.

The March 27 discussions exposed that the repo is still carrying two partially conflicting instincts:

1. category posture as if it were a first-class restriction score for undeclared hostile traffic,
2. and board-state exploit or cost scoring as if that were the real game.

The clearer answer is that Shuma needs both recognition work and restriction work, but they are not the same rail and they must not be allowed to contaminate each other.

# The Main Realisation

The Game Loop does not need to know exact non-humanness or exact hostile category at ingress in order to be useful.

The Game Loop must also not become a parallel fake classifier that pretends to know more than Shuma itself can know from runtime evidence.

What it does need is:

1. a truthful record of how much cost or board progression the current traffic caused,
2. a truthful record of how confident Shuma itself became that the traffic was hostile or non-human as it passed through the defense layers,
3. and a clean separation between:
   1. runtime defense behavior,
   2. restriction tuning and scoring,
   3. and offline evaluation of how well Shuma recognized what the simulator really sent.

That means `human_only_private` remains the right strict reference stance for the Game Loop, but it must be understood as policy intent and evaluation anchor rather than a claim that Shuma can know the exact human or non-human truth at the first request.

# The Two Quests

## 1. Restriction quest

This is the main quest.

It asks:

1. how much costly hostile traffic reached the board,
2. how deep did it progress,
3. what host cost did it impose,
4. what human friction did the current defenses impose,
5. and which bounded config or later code changes reduce that hostile cost or progression while respecting human-friction guardrails.

This quest must use only Shuma-side runtime evidence and outcomes.

## 2. Recognition quest

This is a side quest.

It asks:

1. how well did Shuma infer non-humanness,
2. how well did it infer the right hostile category where such inference is realistically possible,
3. and where did its certainty rise too slowly, stay too weak, or collapse into `unknown_non_human`.

This quest may use simulator-known labels after the fact, but only as harness-evaluation truth.
It must not tell the defenses what the visitor "really is" while the visit is happening.

This is the same general pattern as privileged-information-for-evaluation rather than privileged-information-for-deployment.
The evaluator is allowed to know more than the deployed runtime, but that extra knowledge must stay outside the runtime decision loop.

# The Three Rails

## 1. Defense rail

This is the sacred runtime rail.

Only host-observable evidence may enter:

1. request shape,
2. headers,
3. challenge behavior,
4. browser execution signals,
5. traversal behavior,
6. rate or byte pressure,
7. defense outcomes,
8. and any verified or signed identity evidence that a real visitor actually presented.

Simulator-known category labels must never enter this rail.

## 2. Restriction-scoring rail

This is the rail that should drive config suggestions and later code-gap referral.

It should score:

1. board progression,
2. host cost,
3. human-friction guardrails,
4. Shuma's own evolving confidence that the visitor is hostile or non-human,
5. and an anomaly or harm floor for low-certainty but still expensive traffic.

This rail may consume Shuma's runtime confidence outputs, but not simulator labels.

## 3. Recognition-evaluation rail

This is where simulator truth is allowed.

It should compare:

1. simulator-known persona or category intent,
2. Shuma's inferred non-humanness confidence,
3. Shuma's inferred hostile category when present,
4. and the specific surfaces or signals that raised or failed to raise confidence.

This rail is allowed to use privileged harness truth because it is evaluation-only.
It must not leak back into defense decisions or restriction-tuning scores.

That separation is exactly the important guardrail:

1. the evaluator may know the simulator's exact category,
2. the runtime must not see it,
3. and the restriction scorer must not be told what to do by it.

# Confidence Must Accumulate Through Layers

The right model is not "botness is known first and then the defenses act."

The right model is:

1. early weak priors,
2. more evidence as the visitor encounters more defenses,
3. rising or falling confidence through those layers,
4. and routing or restriction that becomes more forceful as confidence and harm justify it.

This is closer to how layered bot mitigation and risk-based systems are described publicly:

1. Cloudflare describes multiple parallel detection mechanisms such as heuristics, machine learning, behavioral analysis, and JavaScript detections rather than one magical upfront truth source:
   - [Cloudflare Bot Management: machine learning and more](https://blog.cloudflare.com/cloudflare-bot-management-machine-learning-and-more/)
   - [Cloudflare JavaScript Detections](https://developers.cloudflare.com/cloudflare-challenges/challenge-types/javascript-detections/)
2. NIST identity guidance describes risk and fraud indicators as part of confidence or risk adjustment rather than as standalone proof of identity:
   - [NIST SP 800-63B](https://pages.nist.gov/800-63-4/sp800-63b.html)

# Abuse-Driven Confidence Escalation Is Required

The Game Loop needs a backstop.

If a stealthy attacker evades all explicit hostile fingerprints but still consumes abnormal cost quickly, Shuma should still become more confident that the visitor is hostile, abusive, or non-human-adjacent.

That backstop should be explicit and should use short-window signals such as:

1. request rate,
2. bytes transferred,
3. page or asset fetch velocity,
4. challenge retries,
5. verification abuse,
6. concurrency,
7. and other direct cost or abuse measures.

This does not mean "humans become bots."
It means abuse pressure itself is evidence that the current visitor should be treated more cautiously or more restrictively.

Two implications matter:

1. non-restriction of high-confidence hostile traffic should weigh more heavily than equally costly lower-confidence traffic,
2. but low-confidence traffic that is still imposing large host cost must still remain urgent through an anomaly or harm floor.

Otherwise a stealthy adversary can hide in the low-certainty region while still draining resources.

# What This Means For The Current Repo

The repo is not yet cleanly aligned to this model.

## 1. Category posture is still too central

`default_category_postures()` in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) still seeds every canonical non-human category as a blocked target.
That is still useful as policy intent, but it is too strong as a primary Game Loop restriction score for undeclared hostile traffic.

## 2. Restriction and recognition are still partially blended

`family_priority()` and `classify_problem()` in [`../../src/observability/benchmark_results_comparison.rs`](../../src/observability/benchmark_results_comparison.rs) already partly acknowledge that category posture misses are not bounded config-tuning signals.
But the repo still carries the older shape where `non_human_category_posture` looks like part of the main attacker-restriction scoreboard.

## 3. Runtime classification is still coarse for undeclared hostile traffic

`non_human_category_assignment_for_lane()` in [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs) still collapses `SuspiciousAutomation` to `unknown_non_human`.
That is honest for the current runtime, but it means the current Game Loop cannot depend on exact hostile-category scoring for undeclared traffic yet.

# Decisions

1. Keep simulator-known labels out of runtime defense behavior.
2. Keep simulator-known labels out of restriction scoring and config-change selection.
3. Explicitly allow simulator-known labels in a separate recognition-evaluation rail.
4. Recenter primary Game Loop scoring on:
   1. board progression,
   2. host cost,
   3. Shuma's own confidence,
   4. and human-friction guardrails.
5. Demote category posture for undeclared hostile traffic from primary restriction score to secondary diagnostic or evaluation surface unless and until Shuma has real shared-path certainty strong enough to justify a tighter role.
6. Add abuse-driven confidence escalation as an explicit backstop so stealthy but expensive traffic still matters.
7. Preserve the strict `human_only_private` reference stance as policy intent and regression anchor.
8. Keep the later frontier-LLM code-evolution ring blocked until it can consume these rails separately and explicitly.
9. Keep human-friction calibration as a separate later ring over real human evidence only.

# Why Harness Truth Is Still Valuable

Keeping simulator labels out of runtime and tuning does not mean they are useless.

They are valuable for:

1. confirming that each sim persona really exercised the surfaces it was meant to exercise,
2. measuring whether Shuma's recognition improved,
3. building confusion matrices between sim intent and Shuma inference,
4. and discovering whether missing exact category scoring is caused by weak Shuma inference or weak persona expression.

This mirrors the general evaluation separation used in information-retrieval and benchmark systems:

1. training or evaluation labels help measure the system,
2. but the deployed system is not allowed to see those answers while operating.

Reference:

- [TREC How To](https://trec.nist.gov/howto.html)
- [Learning by Cheating](https://arxiv.org/abs/1912.12294)
- [Asymmetric DQN](https://proceedings.mlr.press/v180/baisero22a.html)

# Planning Implications

The open March 27 Game Loop tasks should now be interpreted as follows:

1. `SIM-SCR-FULL-1C4` and `SIM-SCR-FULL-1C5` remain unchanged in spirit because they are about proof and rigor of what Scrapling really exercised on the board.
2. `RSI-SCORE-2F2` should become the recognition-evaluation rail and shared-path inference audit, not a demand that category posture become the main restriction score.
3. `RSI-SCORE-2F3` should become the restriction-scoring refactor:
   1. board cost and progression first,
   2. Shuma confidence weighting second,
   3. abuse-driven confidence backstop third,
   4. category posture secondary for undeclared hostile traffic.
4. `RSI-GAME-BOARD-1F` and `RSI-GAME-BOARD-1G` should project these new rails clearly enough that operators can tell:
   1. what the board state is,
   2. what Shuma believed,
   3. what the harness later proved,
   4. and why a config move is or is not credible.

# Bottom Line

The Game Loop should not become a fake omniscient classifier.

It should become:

1. a restriction loop that optimizes against hostile cost, hostile progression, and human burden using only Shuma-side evidence,
2. a recognition loop that measures how well Shuma learned to identify hostile traffic and categories using simulator truth after the fact,
3. and a layered confidence system where explicit hostile signals and raw abuse pressure both raise urgency.

That is the architecture most likely to produce a real RSI loop instead of a neat-looking but overfitted simulator game.
