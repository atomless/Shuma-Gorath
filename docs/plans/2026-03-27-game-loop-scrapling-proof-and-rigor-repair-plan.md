Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../research/2026-03-27-game-loop-category-posture-scoring-audit.md`](../research/2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](../research/2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md`](../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md)
- [`2026-03-27-game-loop-board-state-refactor-plan.md`](2026-03-27-game-loop-board-state-refactor-plan.md)
- [`2026-03-27-game-loop-category-posture-truth-repair-plan.md`](2026-03-27-game-loop-category-posture-truth-repair-plan.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_scrapling_exploit_progress.rs`](../../src/observability/benchmark_scrapling_exploit_progress.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../src/observability/non_human_classification.rs`](../../src/observability/non_human_classification.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/traffic_classification.rs`](../../src/runtime/traffic_classification.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

# Objective

Close the most immediate Scrapling Game Loop rigor gaps that still prevent the loop from behaving like a repairable board-state system:

1. prove exactly which owned surfaces Scrapling is and is not exercising,
2. make surface dependencies or independence explicit,
3. stop letting current Scrapling categories float ambiguously between restriction scoring and recognition scoring,
4. replace flat blocker dumps with structured next-fix truth,
5. and make named breach loci exact enough to support fine-grained config repair.

This is still a Scrapling-first tranche.
The frontier-LLM attacker stays deferred until the Scrapling loop produces trustworthy board-state truth.

Current execution note:

1. `SIM-SCR-FULL-1C4`, `SIM-SCR-FULL-1C5`, `RSI-SCORE-2F2`, `RSI-SCORE-2F3`, `RSI-GAME-BOARD-1F`, `RSI-GAME-ARCH-1D`, and `RSI-GAME-BOARD-1G` are now landed.
2. The next active follow-on from this rigor chain is `RSI-GAME-ARCH-1E`.

# Core Decisions

1. Do not accept vague blocker lists as "actionability".
2. Do not accept vague or partially fabricated breach loci as "locality".
3. Do not accept a Game Loop where the restriction quest and recognition quest are still blurred together.
4. Do not use simulator labels, fulfillment modes, or recent-run category declarations as category truth.
5. Do not hide missing proof by coercing null or absent fields into zero.
6. If exact category inference for the current Scrapling categories requires a bigger hostile-category inference architecture than the current lane mapping can support, stop and discuss that architecture explicitly before faking progress.
7. Do not let simulator-known category labels become a shortcut that makes the recognition quest look solved or the restriction quest look smarter than it really is.

# Execution Tranche

## `SIM-SCR-FULL-1C4`

### Scrapling surface-exercise proof audit and repair

Required contract:

1. the repo must prove whether the current Scrapling worker truly exercises:
   1. `js_verification_execution`
   2. `browser_automation_detection`
   3. `pow_verify_abuse`
   4. `tarpit_progress_abuse`
   5. `maze_navigation`
2. the proof chain from worker receipt -> recent run -> owned-surface coverage -> benchmark -> Game Loop must stay intact and truthful,
3. and the operator must be able to tell whether a surface was:
   1. attempted and satisfied,
   2. attempted and blocked,
   3. required but not actually reached,
   4. or not required for the observed fulfillment modes.

Acceptance criteria:

1. worker-level tests or audit evidence prove the current request or browser steps for the owned surfaces above,
2. the coverage summary no longer drops or obscures those attempts when they are present,
3. the Game Loop or Red Team surfaces can explain the difference between "attempted and failed" and "never reached",
4. and focused proof exists through:
   1. `make test-adversary-sim-scrapling-coverage-receipts`
   2. `make test-adversary-sim-runtime-surface`
   3. `make test-dashboard-game-loop-accountability`

## `SIM-SCR-FULL-1C5`

### Owned-surface dependency and contract-rigor repair

Required contract:

1. the canonical owned-surface matrix must explicitly encode whether `pow_verify_abuse`, `tarpit_progress_abuse`, `maze_navigation`, and `js_verification_execution` are independent or prerequisite-linked,
2. if one blocking surface is only absent because an earlier prerequisite surface was not reached, the operator-facing truth must say so,
3. and the canonical owned-surface matrix must stay internally consistent with the worker and Game Loop projection.

Acceptance criteria:

1. the canonical surface contract documents dependency or independence explicitly,
2. required and blocking surface summaries distinguish "not reached because blocked earlier" from "not required" and from "attempted and failed",
3. Game Loop or Red Team no longer leaves the operator guessing why a surface is absent from the blocking list,
4. and focused proof exists through:
   1. `make test-adversary-sim-scrapling-coverage-receipts`
   2. `make test-dashboard-scrapling-evidence`
   3. `make test-dashboard-game-loop-accountability`

## `RSI-SCORE-2F3`

### Restriction scoring recentering around Shuma confidence, board-state cost, and the abuse backstop

Required contract:

1. the primary restriction score for current Scrapling traffic must be driven by:
   1. board progression,
   2. host cost,
   3. human-friction guardrails,
   4. and Shuma's own hostile or non-human confidence,
2. category posture for undeclared hostile traffic must not remain the main restriction score,
3. non-restriction of high-confidence hostile traffic must weigh more heavily than equally costly lower-confidence traffic,
4. low-confidence but high-cost traffic must still remain urgent through an anomaly or harm floor,
5. and if this requires a larger architecture change than the current judge model can support, execution must stop and escalate that architecture fork explicitly.

Acceptance criteria:

1. the repo explicitly names Shuma confidence and abuse-driven confidence escalation as part of restriction urgency,
2. benchmark and dashboard contracts stop implying that exact hostile-category posture is the main restriction score for undeclared Scrapling traffic,
3. the Game Loop projects category posture as a secondary recognition or diagnostic plane unless real shared-path evidence later justifies more,
4. and focused proof exists through:
   1. `make test-traffic-classification-contract`
   2. `make test-benchmark-results-contract`
   3. `make test-rsi-score-exploit-progress`
   4. `make test-rsi-score-evidence-quality`
   5. `make test-rsi-score-move-selection`
   6. `make test-dashboard-game-loop-accountability`

## `RSI-GAME-BOARD-1F`

### Loop Actionability blocker decomposition and next-fix surfacing

Required contract:

1. `Loop Actionability` must separate:
   1. shared classification blockers,
   2. evidence-quality blockers,
   3. specific surface-proof blockers,
   4. controller eligibility blockers,
   5. and the next exact fix surfaces,
2. it must preserve causal ordering rather than dump all blockers as one flat symptom string,
3. and it must tell the operator whether the loop is blocked by missing truth, missing coverage, or exhausted credible moves.

Acceptance criteria:

1. the machine payload preserves typed blocker groups or equivalent structured fields,
2. the dashboard projects those groups distinctly enough to show what must be fixed first,
3. the current flat blocker line is removed or demoted to a secondary raw-detail view,
4. and focused proof exists through:
   1. `make test-benchmark-results-contract`
   2. `make test-rsi-score-evidence-quality`
   3. `make test-dashboard-game-loop-accountability`

## `RSI-GAME-BOARD-1G`

### Named breach-locus exactness and missing-data honesty

Required contract:

1. each breach locus must name the exact owned surface and the exact request or browser sample that proved progress,
2. missing numeric or cost data must render as not materialized rather than zero,
3. host-cost and repair information must distinguish:
   1. measured,
   2. derived,
   3. and not yet materialized,
4. and the operator-facing breach view must stop conflating generic stage labels with the real defense surface that fell short.

Acceptance criteria:

1. the dashboard no longer coerces missing `attempt_count` or equivalent fields to `0`,
2. breach loci preserve surface-exact labels plus explicit data-materialization state,
3. the Game Loop can tell the operator which surface was breached and which repair surface is implicated without relying on vague generic wording,
4. and focused proof exists through:
   1. `make test-rsi-score-exploit-progress`
   2. `make test-benchmark-results-contract`
   3. `make test-dashboard-game-loop-accountability`

# Sequencing

1. Land `SIM-SCR-FULL-1C4` first so the repo knows what Scrapling is truly exercising today.
2. Land `SIM-SCR-FULL-1C5` second so required-surface and dependency truth is settled before further projection cleanup.
3. Land `RSI-SCORE-2F2` as the recognition-evaluation audit before using exact category inference for anything stronger than honesty.
4. Land `RSI-SCORE-2F3` after that so restriction scoring is recentered on confidence, board cost, and progression instead of category posture alone.
5. Land `RSI-GAME-BOARD-1F` after the scoring reset so actionability becomes a repair graph rather than a blocker dump.
6. Land `RSI-GAME-BOARD-1G` after that so breach locality becomes precise enough to guide bounded config changes.
7. Only after those slices should the repo reconsider reopening `SIM-LLM-1C3`.

# Definition Of Done

This rigor tranche is complete when:

1. the repo can prove exactly which current Scrapling-owned surfaces were exercised and how,
2. the owned-surface contract is explicit about dependency or independence where the operator needs that truth,
3. the repo has separated the recognition quest from the restriction quest instead of blending them through category posture,
4. the Game Loop actionability surface shows typed next-fix truth rather than one flat blocker list,
5. breach loci no longer fabricate zeroes for missing data and no longer describe attacker progress too vaguely,
6. the focused `make` proof targets above pass for each landed sub-slice,
7. and the active backlog clearly makes this Scrapling-first rigor repair the next mainline before later LLM attacker reopening.
