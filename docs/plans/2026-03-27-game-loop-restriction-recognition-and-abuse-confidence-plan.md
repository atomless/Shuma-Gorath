Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../research/2026-03-27-game-loop-category-posture-scoring-audit.md`](../research/2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md`](../plans/2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
- [`../plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md`](../plans/2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md)
- [`../plans/2026-03-27-human-friction-calibration-ring-plan.md`](../plans/2026-03-27-human-friction-calibration-ring-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Turn the new Game Loop architecture clarification into execution-ready guidance:

1. restriction scoring is the main quest,
2. recognition evaluation is a side quest,
3. simulator labels may inform evaluation but must not drive runtime or tuning,
4. Shuma confidence must accumulate through defense layers,
5. abuse-driven confidence escalation must exist as a backstop,
6. and the Game Loop UI and diagnoser must project these truths separately enough to support real bounded repair.

Implementation status update (2026-03-27):

1. The board-state doctrine and the shared-path rule are already written down in the March 27 board-state review and refactor plan.
2. The new clarification in this plan does not discard that work.
3. It tightens it by explicitly separating:
   1. the defense rail,
   2. the restriction-scoring rail,
   3. and the recognition-evaluation rail.
4. `RSI-SCORE-2F2` is now landed and keeps the recognition-evaluation rail honest about collapse to `unknown_non_human`.
5. `RSI-GAME-ARCH-1B` is now landed and stops category posture from acting like the primary restriction scoreboard.
6. `RSI-SCORE-2F3` is now landed: restriction urgency explicitly carries `Restriction Confidence` and `Abuse Backstop`, and the named proof path now executes dedicated urgency tests rather than merely compiling the new behavior.
7. `RSI-GAME-BOARD-1F` is now landed: `Loop Actionability` projects grouped root causes, controller outcomes, and next-fix surfaces from the typed controller contract.
8. The remaining open work in this chain is now:
   1. `RSI-GAME-BOARD-1G`,
   2. and the later controller and retirement cleanup in the architecture-alignment plan.

# Core Decisions

1. Treat the host board and the shared-path rule as already settled doctrine.
2. Make restriction scoring primary and recognition evaluation secondary.
3. Keep simulator labels completely out of runtime and tuning.
4. Explicitly allow simulator labels only in the recognition-evaluation rail.
5. Require Shuma's own confidence to influence restriction urgency.
6. Require abuse-driven confidence escalation so stealthy but expensive traffic still drives urgency.
7. Keep category posture for undeclared hostile traffic as a secondary diagnostic or evaluation plane unless real shared-path evidence later justifies a stronger role.

# Execution Tranche

## `RSI-SCORE-2F2`

### Recognition-evaluation rail and shared-path inference audit

Required contract:

1. the repo must define one explicit recognition-evaluation rail,
2. that rail must compare simulator-known category intent against Shuma's own inferred non-humanness and inferred hostile category,
3. runtime and restriction-scoring paths must stay free of simulator labels,
4. exact shared-path category inference may still be added where Shuma can really infer it from host-observable evidence,
5. and categories that remain only `unknown_non_human` must stay explicit rather than faked.

Acceptance criteria:

1. repo docs and backlog explicitly name the recognition-evaluation rail as distinct from restriction scoring,
2. the repo explicitly states which hostile categories are realistically inferable today from shared-path evidence and which are not,
3. no proposed implementation path uses `sim_profile`, `sim_lane`, worker fulfillment mode, or equivalent harness metadata as runtime or tuning truth,
4. if new exact inference later lands, it must be proven through:
   1. `make test-traffic-classification-contract`
   2. `make test-adversary-sim-scrapling-category-fit`
   3. `make test-benchmark-results-contract`
5. if exact inference is still not viable for one or more categories, the Game Loop remains honest and the recognition-evaluation rail records that gap rather than forcing fake category precision.

## `RSI-SCORE-2F3`

### Restriction scoring recentering and abuse-driven confidence escalation

Required contract:

1. the primary restriction score for undeclared hostile traffic must be driven by:
   1. board progression,
   2. host cost,
   3. human-friction guardrails,
   4. and Shuma's own hostile or non-human confidence,
2. non-restriction of high-confidence hostile traffic must weigh more heavily than equally costly lower-confidence traffic,
3. low-confidence but high-cost traffic must still remain urgent through an anomaly or harm floor,
4. and simulator-known categories must not directly drive bounded config tuning.

Acceptance criteria:

1. plan and backlog language no longer describe exact hostile-category posture as the primary restriction score for undeclared hostile traffic,
2. the restriction-scoring contract explicitly names confidence-weighted urgency plus an anomaly or harm floor,
3. implementation proof for the landed slice exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-rsi-score-exploit-progress`
   3. `make test-rsi-score-evidence-quality`
   4. `make test-rsi-score-urgency-and-homeostasis`
   5. `make test-rsi-score-move-selection`
   6. `make test-dashboard-game-loop-accountability`,
4. and any attempt to use simulator-known category labels in restriction scoring is treated as insufficient.

## `RSI-GAME-BOARD-1F`

### Loop Actionability projection under the three-rail model

Required contract:

1. `Loop Actionability` must explain restriction blockers in terms of:
   1. missing board-state truth,
   2. missing recognition quality,
   3. insufficient confidence,
   4. missing surface proof,
   5. or no credible bounded config move remaining,
2. it must stop flattening recognition-side gaps and restriction-side gaps into one untyped blocker string,
3. and it must tell the operator what needs to improve on the recognition quest versus what needs to improve on the restriction quest.

Acceptance criteria:

1. blocker groups distinguish recognition-evaluation gaps from restriction-tuning gaps,
2. actionability text can identify whether the next work is:
   1. better proof,
   2. better confidence accumulation,
   3. better bounded config moves,
   4. or later code-gap referral,
3. focused proof later exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-rsi-score-evidence-quality`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-BOARD-1G`

### Board-state and breach-locus projection under confidence-through-layers

Required contract:

1. breach loci must remain surface-exact and host-cost-local,
2. they must also expose the confidence context that made the locus urgent enough to matter,
3. and the Game Loop must stop implying that untrusted or low-confidence category posture rows are the main story when the real issue is board progression plus cost.

Acceptance criteria:

1. the board-state surface names:
   1. exact breach locus,
   2. exact sample,
   3. materialized or missing host-cost fields,
   4. and the relevant confidence or certainty context where available,
2. the Game Loop clearly distinguishes:
   1. origin leakage,
   2. board progression,
   3. recognition quality,
   4. and surface-contract truth,
3. focused proof later exists through:
   1. `make test-rsi-score-exploit-progress`
   2. `make test-benchmark-results-contract`
   3. `make test-dashboard-game-loop-accountability`.

## `OVR-CODE-1`

### Later code-evolution handoff under the new rails

Required contract:

1. the later frontier-LLM code-evolution ring must consume restriction and recognition outputs separately,
2. it must not be fed simulator labels as if they were runtime truth,
3. and it must treat the recognition-evaluation rail as supporting evidence rather than as a hidden runtime oracle.

Acceptance criteria:

1. blocked planning for `OVR-CODE-1` explicitly inherits the three-rail model,
2. code-evolution remains blocked until the Scrapling restriction loop is trusted,
3. and this planning slice records the doctrine change without reopening execution.

## `HUM-FRIC-1`

### Human-friction ring stays separate and real-human only

Required contract:

1. human-friction measurement remains a later separate ring over human-operated journeys,
2. adversary-sim traffic must never count as human-friction evidence,
3. and the new recognition or restriction rails must not blur that boundary.

Acceptance criteria:

1. blocked planning for `HUM-FRIC-1` explicitly points to this doctrine,
2. the repo still forbids adversary-sim contamination of human-friction evidence,
3. and this planning slice records the clarification without reopening execution.

# Sequencing

1. Keep `SIM-SCR-FULL-1C4` first so surface-exercise truth is settled.
2. Keep `SIM-SCR-FULL-1C5` second so dependency truth is settled.
3. Land `RSI-SCORE-2F2` next as the recognition-evaluation and shared-path inference audit.
4. Land `RSI-SCORE-2F3` after that so restriction scoring is recentered on cost, progression, confidence, and the abuse backstop.
5. Land `RSI-GAME-BOARD-1F` and `RSI-GAME-BOARD-1G` after the scoring reset so the UI and actionability surfaces project the new rails clearly.
6. Keep `SIM-LLM-1C3`, `OVR-CODE-1`, and `HUM-FRIC-1` blocked behind a trustworthy Scrapling-first restriction loop.

# Definition Of Done

This planning tranche is complete when:

1. the repo has one explicit doctrine note for restriction versus recognition and abuse-driven confidence escalation,
2. the active plan chain and backlog no longer treat exact hostile-category posture as the main restriction score for undeclared hostile traffic,
3. the open Scrapling-first tasks are reworded to follow the new three-rail model,
4. later blocked code-evolution and human-friction work explicitly inherit the same doctrine,
5. and the repo now says one consistent thing about what the Game Loop is actually optimizing.
