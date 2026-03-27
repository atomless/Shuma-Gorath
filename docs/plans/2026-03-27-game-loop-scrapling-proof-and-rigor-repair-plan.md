Date: 2026-03-27
Status: In progress

Related context:

- [`../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md`](../research/2026-03-27-game-loop-scrapling-proof-and-rigor-gap-review.md)
- [`../research/2026-03-27-game-loop-category-posture-scoring-audit.md`](../research/2026-03-27-game-loop-category-posture-scoring-audit.md)
- [`../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md`](../research/2026-03-27-game-loop-shared-path-locality-and-actionability-post-implementation-review.md)
- [`2026-03-27-game-loop-board-state-refactor-plan.md`](2026-03-27-game-loop-board-state-refactor-plan.md)
- [`2026-03-27-game-loop-category-posture-truth-repair-plan.md`](2026-03-27-game-loop-category-posture-truth-repair-plan.md)
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
3. turn current Scrapling categories from wholly unscored into exact shared-path blocked-share measurements,
4. replace flat blocker dumps with structured next-fix truth,
5. and make named breach loci exact enough to support fine-grained config repair.

This is still a Scrapling-first tranche.
The frontier-LLM attacker stays deferred until the Scrapling loop produces trustworthy board-state truth.

# Core Decisions

1. Do not accept vague blocker lists as "actionability".
2. Do not accept vague or partially fabricated breach loci as "locality".
3. Do not accept a Game Loop where all current Scrapling categories remain unscored.
4. Do not use simulator labels, fulfillment modes, or recent-run category declarations as category truth.
5. Do not hide missing proof by coercing null or absent fields into zero.
6. If exact category scoring for the current Scrapling categories requires a bigger hostile-category inference architecture than the current lane mapping can support, stop and discuss that architecture explicitly before faking progress.

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

### Exact shared-path category posture scoring for current Scrapling categories

Required contract:

1. `Category Posture Achievement` must emit exact shared-path blocked-share values for the current Scrapling-populated categories:
   1. `indexing_bot`
   2. `ai_scraper_bot`
   3. `automated_browser`
   4. `http_agent`
2. the exact receipts must come only from Shuma-side request, behavior, response, or browser evidence that real external traffic could also produce,
3. no simulator persona label, fulfillment mode, or recent-run category declaration may become category truth,
4. and if any category still cannot be inferred exactly without a larger architecture change, execution must stop and escalate that architecture fork explicitly.

Acceptance criteria:

1. the repo names the exact Shuma-side signals used for each newly exact category,
2. benchmark tests prove those categories can now score partial blocked-share values from exact receipts rather than degraded placeholders,
3. the Game Loop no longer renders all four current Scrapling categories as `Unscored`,
4. and focused proof exists through:
   1. `make test-traffic-classification-contract`
   2. `make test-adversary-sim-scrapling-category-fit`
   3. `make test-benchmark-results-contract`
   4. `make test-dashboard-game-loop-accountability`

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
3. Land `RSI-SCORE-2F3` third so category posture becomes useful instead of merely honest.
4. Land `RSI-GAME-BOARD-1F` fourth so actionability becomes a repair graph rather than a blocker dump.
5. Land `RSI-GAME-BOARD-1G` fifth so breach locality becomes precise enough to guide bounded config changes.
6. Only after those slices should the repo reconsider reopening `SIM-LLM-1C3`.

# Definition Of Done

This rigor tranche is complete when:

1. the repo can prove exactly which current Scrapling-owned surfaces were exercised and how,
2. the owned-surface contract is explicit about dependency or independence where the operator needs that truth,
3. the current Scrapling categories have exact shared-path posture scores or the repo has stopped for an explicit architecture discussion,
4. the Game Loop actionability surface shows typed next-fix truth rather than one flat blocker list,
5. breach loci no longer fabricate zeroes for missing data and no longer describe attacker progress too vaguely,
6. the focused `make` proof targets above pass for each landed sub-slice,
7. and the active backlog clearly makes this Scrapling-first rigor repair the next mainline before later LLM attacker reopening.
