Date: 2026-03-28
Status: In Progress

Related context:

- [`../research/2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](../research/2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
- [`2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md`](2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md)
- [`2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-plan.md`](2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-plan.md)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Turn `RSI-GAME-HO-2` into a truthful combined-attacker strict-baseline proof by adding the smallest real architecture needed for:

1. both Scrapling and LLM attacker lanes to contribute episode evidence,
2. lane-native restriction scoring to judge that evidence,
3. and repeated bounded config moves to be retained or rolled back against mixed-attacker pressure instead of Scrapling-only pressure.

# Core Decisions

1. Do not try to fake combined proof with two simultaneously active lanes; the current runtime is single-lane and the truthful extension is sequential multi-run episode materialization.
2. Keep simulator labels out of runtime and tuning exactly as before.
3. Add no privileged LLM or simulator truth to the defense rail.
4. Make mixed-attacker orchestration explicit in machine-first state rather than inferred from recent-run coincidence.
5. Preserve the current `30s` runtime-dev shortcut only for Scrapling-sized windows; mixed-attacker episodes must carry lane-specific meaningful-duration truth.

# Execution Tranches

## `RSI-GAME-HO-2A1`

### Multi-lane episode and follow-on orchestration

Status: Implemented on 2026-03-28. See [`../research/2026-03-28-rsi-game-ho-2a1-multi-lane-episode-orchestration-post-implementation-review.md`](../research/2026-03-28-rsi-game-ho-2a1-multi-lane-episode-orchestration-post-implementation-review.md).

Required contract:

1. candidate windows and continuation reruns must be able to require more than one attacker lane,
2. materialization truth must be tracked per required lane,
3. follow-on runs may still execute one lane at a time,
4. and the judge must not treat the episode as complete until all required lanes have either materialized or expired truthfully.

Acceptance criteria:

1. active canary state no longer carries only one `requested_lane` and one `follow_on_run_id`,
2. continuation state can request a bounded ordered set of required lanes,
3. supervisor auto-start still materializes exactly one pending run at a time but can progress through the required-lane set without operator babysitting,
4. machine-first status surfaces show which required lanes are pending, running, materialized, or expired,
5. proof exists through:
   1. a focused oversight-agent make target,
   2. a focused RSI mixed-loop make target,
   3. and `GET /admin/oversight/agent/status`.

## `RSI-GAME-HO-2A2`

### Lane-native mixed-attacker restriction score spine

Status: Implemented on 2026-03-28. See [`../research/2026-03-28-rsi-game-ho-2a2-mixed-attacker-restriction-score-spine-post-implementation-review.md`](../research/2026-03-28-rsi-game-ho-2a2-mixed-attacker-restriction-score-spine-post-implementation-review.md).

Required contract:

1. the controller-grade restriction score must stop being effectively Scrapling-only,
2. Scrapling may keep its owned-surface exploit loci,
3. the LLM lane must gain its own board-locus or equivalent restriction-grade breach receipts,
4. and mixed-attacker pressure must then be aggregated without leaking simulator labels into runtime or tuning.

Acceptance criteria:

1. benchmark assembly exposes a mixed-attacker restriction family or equivalent sibling contract that can include both lanes truthfully,
2. LLM runtime action receipts are no longer recent-run visibility only; they map into restriction-grade breach evidence,
3. protected tuning evidence and tuning eligibility can become true because of mixed-attacker board-state evidence rather than only Scrapling-specific evidence,
4. proof exists through:
   1. `make test-benchmark-results-contract`
   2. a new focused mixed-attacker score-spine target
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-HO-2A3`

### Mixed-attacker proof projection

Required contract:

1. operator snapshot, oversight history, and the Game Loop must show both lanes contributing to the judged episode,
2. the UI must distinguish visibility from controller-grade scoring,
3. and the rendered proof must not overstate maturity from mere lane presence.

Acceptance criteria:

1. machine-first surfaces preserve mixed-attacker episode lineage and retain or rollback outcomes,
2. dashboard rendering shows which lanes contributed to the current candidate or continuation evidence set,
3. Game Loop wording makes clear when both lanes were judged versus merely recently visible,
4. proof exists through:
   1. focused API or route tests,
   2. `make test-dashboard-red-team-truth-basis`
   3. `make test-dashboard-game-loop-accountability`.

## `RSI-GAME-HO-2B`

### Repeated retained improvement under mixed pressure

Required contract:

1. recommendations become bounded config changes,
2. later mixed-attacker runs occur against changed config,
3. watch windows retain or roll back truthfully,
4. and repeated retained changes show positive movement toward the strict target.

Acceptance criteria:

1. at least two retained judged cycles are proven under mixed-attacker pressure,
2. at least one rollback path is still proven under mixed-attacker pressure,
3. the loop still stops only for explicit machine-readable reasons,
4. proof exists through focused mixed-loop make targets and `make test`.

## `RSI-GAME-HO-2C`

### Unlock condition for later stance relaxation

Required contract:

1. the repo must define what qualifies as enough mixed-attacker strict-baseline proof,
2. and `humans_plus_verified_only` must remain blocked until that proof exists.

Acceptance criteria:

1. the acceptance gate is explicit in docs and backlog,
2. the proof is stronger than lane visibility or one mixed handoff,
3. and later relaxed-stance work remains blocked until that gate is met.

# Sequencing

1. Land `RSI-GAME-HO-2A1` first because the current state model cannot express a truthful mixed episode.
2. Land `RSI-GAME-HO-2A2` second because the current score spine is still mostly Scrapling-only.
3. Land `RSI-GAME-HO-2A3` third so operators can see the new mixed-attacker truth.
4. Only then land `RSI-GAME-HO-2B` repeated retained-improvement proof.
5. Keep `RSI-GAME-HO-2C` as the later unlock gate for relaxed stance work.

# Definition Of Done

This planning tranche is complete when:

1. the repo explicitly states why `RSI-GAME-HO-2` is still blocked after `SIM-LLM-1C3`,
2. the blocker is no longer misdescribed as missing LLM runtime visibility,
3. the next implementation path names the exact orchestration and scoring changes required,
4. lane-specific meaningful-duration truth is part of the plan,
5. and the mixed-attacker proof path is frozen tightly enough that later implementation cannot quietly fake it.
