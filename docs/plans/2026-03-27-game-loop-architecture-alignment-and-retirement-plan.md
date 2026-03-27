Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-architecture-alignment-gap-review.md`](../research/2026-03-27-game-loop-architecture-alignment-gap-review.md)
- [`../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](../plans/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../plans/2026-03-27-game-loop-board-state-refactor-plan.md`](../plans/2026-03-27-game-loop-board-state-refactor-plan.md)
- [`../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md`](../plans/2026-03-27-game-loop-scrapling-proof-and-rigor-repair-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Turn the architecture-alignment audit into execution-ready refactor guidance so the Game Loop implementation stops drifting between:

1. the newer board-state-first, restriction-vs-recognition design,
2. and the older family-first, category-posture-led architecture.

Implementation status update (2026-03-27):

1. `RSI-GAME-ARCH-1A` is landed: restriction-grade and recognition-evaluation evidence are now split in snapshot assembly.
2. `RSI-GAME-ARCH-1B` is landed: category posture is no longer a primary restriction optimization target or top-level restriction trigger for undeclared hostile traffic.
3. The next architecture-alignment step is now `RSI-GAME-ARCH-1C`.

# Core Decisions

1. Do not treat the current category-first benchmark stack as "dead"; replace it deliberately and then retire it.
2. Keep the runtime defense rail free of simulator labels.
3. Split recognition-evaluation from restriction scoring in snapshot and benchmark assembly.
4. Recenter restriction scoring on board progression, host cost, Shuma confidence, and human-friction guardrails.
5. Refactor controller inputs so reconcile does not depend on one monolithic escalation-hint oracle.
6. Make board-state loci and blocker types explicit enough that the Game Loop can read like a real game board instead of a blended benchmark dump.

# Execution Tranches

## `RSI-GAME-ARCH-1A`

### Split recognition-evaluation and restriction evidence in snapshot assembly

Required contract:

1. snapshot and hot-read contracts must distinguish:
   1. restriction-grade evidence,
   2. recognition-evaluation evidence,
   3. simulator ground-truth evaluation summaries,
2. projected recent-sim category presence must stop sharing the same receipt lane as restriction-grade current classification truth,
3. simulator ground truth may inform evaluation surfaces only after the fact.

Acceptance criteria:

1. `operator_snapshot_non_human` no longer exposes one mixed receipt stream that conflates projected sim category presence with restriction-grade current classification truth,
2. restriction scoring no longer needs to inspect degraded projected sim receipts to decide whether it is eligible to act,
3. recognition evaluation remains able to compare sim intent against Shuma inference,
4. runtime and tuning still remain free of simulator labels,
5. proof later exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-traffic-classification-contract`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-ARCH-1B`

### Replace category-posture-led restriction objectives with restriction-first score surfaces

Required contract:

1. undeclared hostile traffic must no longer be optimized primarily through exact hostile-category posture targets,
2. the primary restriction objective model must instead use:
   1. board progression,
   2. host cost,
   3. Shuma confidence,
   4. abuse backstop,
   5. human-friction guardrails,
3. category posture may remain as recognition evaluation or secondary diagnostics where appropriate.

Acceptance criteria:

1. `default_category_postures()` and `non_human_category_posture` are either demoted or explicitly re-scoped away from primary restriction scoring for undeclared hostile traffic,
2. `benchmark_suite_v1` and `benchmark_results_v1` expose the replacement restriction-first score surfaces,
3. the Game Loop no longer reads category posture as the main restriction scoreboard,
4. proof later exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-rsi-score-exploit-progress`
   3. `make test-rsi-score-evidence-quality`
   4. `make test-rsi-score-move-selection`
   5. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-ARCH-1C`

### Refactor controller contracts away from the monolithic escalation hint

Required contract:

1. judge, restriction diagnosis, recognition-evaluation status, and move selection must become explicit sibling surfaces,
2. reconcile must stop behaving like a thin wrapper around one benchmark escalation-hint object,
3. typed blockers and typed repair loci must become first-class controller inputs.

Acceptance criteria:

1. `oversight_reconcile` consumes explicit restriction-judge and move-selection contracts rather than mostly branching on `benchmark.escalation_hint.decision`,
2. `Loop Actionability` can be powered from typed controller state rather than a flat benchmark blocker list,
3. code-evolution referral, bounded config selection, and ring exhaustion remain explicit and testable,
4. proof later exists through:
   1. `make test-rsi-score-move-selection`
   2. `make test-rsi-game-mainline`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-ARCH-1D`

### Normalize board-state locus and blocker data models

Required contract:

1. breach loci must carry materialization truth, not only values,
2. missing `attempt_count`, host-cost, or repair fields must remain explicit,
3. blocker output must distinguish root causes from downstream symptoms.

Acceptance criteria:

1. `BenchmarkExploitLocus` no longer forces missing values through zero-like defaults,
2. API adapters and dashboard projection preserve materialized vs missing vs derived states,
3. actionability blockers are typed and grouped rather than emitted as one flattened string list,
4. proof later exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-rsi-score-exploit-progress`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-ARCH-1E`

### Retire replaced legacy category-first surfaces

Required contract:

1. once replacement contracts are live, the repo must remove or demote the older category-first surfaces cleanly,
2. docs, tests, API contracts, and dashboard projection must stop advertising the retired shape,
3. retirement must be full-path verified rather than assumed.

Acceptance criteria:

1. any retired objective, benchmark family, API field, adapter path, or UI section is proven replaced end-to-end before removal,
2. docs and tests stop referring to the older primary category-first restriction model,
3. removal notes explicitly distinguish retirement from still-live architecture,
4. proof later exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-dashboard-game-loop-accountability`
   3. `make test`
   4. and cited contract-path evidence in docs.

# Sequencing

1. keep `SIM-SCR-FULL-1C4` first so surface-exercise truth is known,
2. keep `SIM-SCR-FULL-1C5` second so dependency truth is known,
3. land `RSI-GAME-ARCH-1A` before or together with `RSI-SCORE-2F2`,
4. land `RSI-GAME-ARCH-1B` before or together with `RSI-SCORE-2F3`,
5. land `RSI-GAME-ARCH-1C` before or together with `RSI-GAME-BOARD-1F`,
6. land `RSI-GAME-ARCH-1D` before or together with `RSI-GAME-BOARD-1G`,
7. land `RSI-GAME-ARCH-1E` only after the replacement architecture is proven.

# Definition Of Done

This planning tranche is complete when:

1. the repo has one explicit architecture-alignment audit for the Game Loop stack,
2. the active plan chain includes a concrete modular refactor path rather than only local UI or scoring repairs,
3. the TODO queue now names the cross-cutting cleanup and retirement work explicitly,
4. retirement candidates are identified without falsely claiming they are already defunct,
5. and the repo now says one consistent thing about what must be refactored before the Game Loop can be considered architecturally aligned with the newer design.
