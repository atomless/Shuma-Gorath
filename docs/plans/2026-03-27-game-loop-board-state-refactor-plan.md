Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](../research/2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`../research/2026-03-27-game-loop-category-posture-scoring-audit.md`](../research/2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](../research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
- [`../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
- [`2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md`](2026-03-27-ovr-code-1-frontier-llm-code-evolution-ring-plan.md)
- [`2026-03-27-human-friction-calibration-ring-plan.md`](2026-03-27-human-friction-calibration-ring-plan.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`2026-03-24-llm-player-role-decomposition-plan.md`](2026-03-24-llm-player-role-decomposition-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Refactor the Game Loop around an explicit board-state doctrine:

1. the host site is the board,
2. Shuma's defense surfaces are the movable pieces,
3. adversary-sim traffic and real traffic must share the same classification and scoring truth path,
4. the judge must emit breach-local cost and shortfall signals,
5. the config loop must remember failed moves,
6. and later code-change plus human-friction rings must be planned as explicit follow-ons rather than vague future ideas.

This tranche does not replace the earlier score-vector work.
It tightens and operationalizes it so the next iterations remain faithful to the board-state model and cannot drift back toward sim-privileged or aggregate-only reasoning.

Implementation status update (2026-03-27):

1. The first runtime or UI slice of this plan is now landed:
   - breach loci now preserve `attempt_count`, host-cost channels, and repair-family candidates,
   - localized high-confidence exploit progress can now become a bounded config-tuning candidate,
   - and the Game Loop UI now projects guardrails, board-state breach progress, surface-contract satisfaction, and loop actionability as distinct planes.
2. The later planning-only substeps for the frontier-LLM code-evolution ring and the real human-friction calibration ring are now being broken out into dedicated follow-on plans rather than left as vague future work.
3. The remaining open runtime truth gap inside this broader refactor is still the exact shared-path category-inference gap tracked separately in `RSI-SCORE-2F2`.
4. The later March 27 architecture clarification now tightens this plan further:
   - restriction scoring is the main quest,
   - recognition quality is a separate evaluation rail,
   - and abuse-driven confidence escalation is part of the board-state doctrine rather than an optional later heuristic.

# Core Decisions

1. Keep `RSI-SCORE-2F` as the immediate honesty repair and treat it as the first gate inside this broader refactor.
2. Forbid simulator metadata from becoming judge truth anywhere in category scoring, exploit scoring, or controller readiness.
3. Require the Game Loop to show three distinct truths at minimum:
   1. origin leakage and human-cost guardrails,
   2. terrain breach progress and host cost beyond the defended boundary,
   3. surface-contract satisfaction and tuning readiness.
4. Require the later recognition-evaluation rail to stay separate from restriction scoring, even when simulator-known labels are available after the fact.
5. Treat breach-local host-cost attribution as a first-class requirement for fine-grained config tuning.
6. Treat failed-move memory as part of controller correctness, not later polish.
7. Treat frontier-LLM code suggestion as a later bounded ring that begins only from explicit code-gap referral or config-ring exhaustion.
8. Treat human-friction measurement as a separate later calibration ring over real human evidence, not a relaxation of the strict sim-only loop.
9. Treat Shuma confidence as something that accumulates through defense layers and can also be raised by short-window abuse pressure even when explicit identity signals stay weak.

# Refactor Shape

## `RSI-GAME-BOARD-1A`

### Shared-path adversary truth and scoring honesty

Required contract:

1. Game Loop scoring for adversary traffic uses only Shuma-side observable evidence that real traffic could also produce.
2. Simulator metadata may remain available for harness control, replay, and offline audit, but it must not become category truth or score inputs.
3. Category or exploit rows with insufficient exact Shuma-side evidence remain explicitly unscored.
4. Game Loop wording makes the shared-path rule explicit enough that operators do not misread projected presence as measured category performance.
5. The refactor must remain compatible with the later three-rail split:
   1. defense rail,
   2. restriction-scoring rail,
   3. recognition-evaluation rail.

Acceptance criteria:

1. any exact category scoring for Scrapling-populated categories is proven to come from Shuma-side request or behavior inference rather than sim metadata,
2. unscored rows render as unscored rather than as `0%`,
3. repo docs explicitly state that sim and real traffic share the same judge path,
4. and focused proof exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-dashboard-game-loop-accountability`
   3. `make test-traffic-classification-contract` when new exact inference lands.

## `RSI-GAME-BOARD-1B`

### Board-state breach and host-cost locality

Required contract:

1. the judge preserves named breach loci on the host board,
2. each meaningful breach locus records the defended surface expected to stop, redirect, or degrade the attacker,
3. each locus records the cost consumed beyond that expected boundary where Shuma has truthful telemetry,
4. and the Game Loop projects those loci without collapsing them into one blended outcome line.

Acceptance criteria:

1. benchmark and comparison surfaces expose breach-local cost or resource-consumption fields where available,
2. controller-grade diagnosis names both the breach locus and the implicated repair surface,
3. Game Loop renders those planes distinctly enough that origin leakage does not read like total attacker defeat,
4. and focused proof exists through:
   1. `make test-rsi-score-exploit-progress`
   2. `make test-rsi-score-evidence-quality`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-BOARD-1C`

### Config trial memory, rollback discipline, and anti-repeat move selection

Required contract:

1. applied config episodes preserve the exact bounded move, named breach loci, measured effect, and retain or rollback outcome,
2. rolled-back moves are available to the move selector as failure memory,
3. near-equivalent failed moves are not reissued without new evidence that changes the diagnosis,
4. and the Game Loop exposes when the current state is blocked because no credible new bounded config move remains.

Acceptance criteria:

1. reconcile or move-selection outputs preserve prior failed-move lineage at the relevant repair surface,
2. repeated non-improving moves can lead to explicit `config_ring_exhausted` or equivalent state,
3. archive and Game Loop surfaces make rollback and anti-repeat reasoning visible,
4. and focused proof exists through:
   1. `make test-rsi-score-move-selection`
   2. `make test-rsi-game-mainline`
   3. `make test-oversight-episode-archive`.

## `RSI-GAME-BOARD-1D`

### Code-gap referral and later frontier-LLM code-evolution handoff

Required contract:

1. code evolution stays separate from bounded config tuning,
2. later frontier-LLM code suggestions may begin only from explicit code-gap referral or config-ring exhaustion plus sufficient evidence quality,
3. the LLM defender or code-evolution actor receives sacred machine-first inputs rather than ad hoc dashboard impressions,
4. and the later code ring preserves strict regression anchors including the human-only reference stance.

Acceptance criteria:

1. the repo has one canonical handoff from config-ring exhaustion or code-gap referral into later code-evolution planning,
2. blocked backlog items for `OVR-CODE-1` explicitly inherit the board-state doctrine and shared-path truth requirements,
3. the planning chain names the bounded inputs, outputs, and proof expectations for frontier-LLM code suggestions,
4. and the acceptance proof is documentation and backlog alignment rather than code execution in this tranche.

## `RSI-GAME-BOARD-1E`

### Human-friction measurement and calibration ring

Required contract:

1. human friction is measured from real human traversal or an explicitly human-operated test ring,
2. adversary-sim traffic never counts as human-friction evidence,
3. the loop records whether a human reached the intended content, what friction was imposed, and what latency or challenge burden was added,
4. and strict sim-only exclusion proof remains distinct from later human-calibration work.

Acceptance criteria:

1. the planning chain names a separate human-friction evidence contract,
2. operator docs stop implying that adversary-sim results can stand in for human-friction truth,
3. the later queue contains explicit human-calibration work rather than leaving it as an implicit aspiration,
4. and the acceptance proof is documentation and backlog alignment rather than runtime code in this tranche.

# Sequencing

1. Finish `RSI-SCORE-2F` first because the shared-path doctrine already fails if the Game Loop pretends to know exact category truth it does not possess.
2. Apply the later restriction-vs-recognition clarification before re-elevating category posture as a first-class Game Loop score for undeclared hostile traffic.
3. Land `RSI-GAME-BOARD-1A` before any renewed claim that Game Loop category or exploit scoring is shared-path truthful.
4. Land `RSI-GAME-BOARD-1B` before any renewed claim that the board-state model is materially reflected in controller-grade scoring.
5. Land `RSI-GAME-BOARD-1C` before any renewed claim that the config loop can avoid wasteful repeated failed moves.
6. Use `RSI-GAME-BOARD-1D` to reopen later `OVR-CODE-1` planning on explicit doctrine rather than vague future intent.
7. Use `RSI-GAME-BOARD-1E` to reopen later human-friction calibration work without weakening the strict non-human exclusion ring.

# Definition Of Done

This planning tranche is complete when:

1. the repo has one explicit doctrine document for the board-state model and shared-path truth rule,
2. the active plan chain has one execution-ready refactor tranche with named substeps for shared-path truth, board-state cost locality, failed-move memory, later code-evolution handoff, and later human-friction calibration,
3. each substep has explicit acceptance criteria and named proof surfaces,
4. the active and blocked TODO queues reflect that sequencing clearly,
5. and the backlog no longer leaves the frontier-LLM code ring or human-friction ring as vague unstated later work.
