Date: 2026-03-28
Status: Proposed planning driver

Related context:

- [`2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md`](2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md)
- [`2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md`](2026-03-28-sim-llm-1c3-recent-run-projection-post-implementation-review.md)
- [`2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-post-implementation-review.md`](2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-post-implementation-review.md)
- [`../plans/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md`](../plans/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-plan.md)
- [`../plans/2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-plan.md`](../plans/2026-03-28-rsi-game-arch-1l-persistent-iterative-loop-plan.md)
- [`../../src/admin/adversary_sim_state.rs`](../../src/admin/adversary_sim_state.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/observability/benchmark_results.rs`](../../src/observability/benchmark_results.rs)
- [`../../src/observability/benchmark_scrapling_exploit_progress.rs`](../../src/observability/benchmark_scrapling_exploit_progress.rs)
- [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Audit whether the repo can truthfully execute `RSI-GAME-HO-2` now that `SIM-LLM-1C3` is landed, and identify the exact architectural changes still required before a combined Scrapling plus LLM strict-loop proof would mean anything.

# Findings

## 1. The control loop is still single-lane at episode and follow-on time

The current adversary-sim control state still models one desired lane and one active lane at a time.

That same single-lane assumption continues through:

1. canary candidate-window state,
2. continuation rerun state,
3. and the supervisor auto-start path.

The current machine-first loop can therefore do:

1. one bounded Scrapling canary,
2. one protected Scrapling follow-on run,
3. one retained or rolled-back judgment,
4. and one further Scrapling continuation rerun.

It cannot yet truthfully mean:

1. "both attackers contributed candidate evidence for this judgment",
2. or "the next bounded move improved the defended board under both attacker lanes".

## 2. Recent-run visibility for `bot_red_team` is now real, but controller-grade mixed-attacker scoring is not

`SIM-LLM-1C3` closed the visibility gap:

1. LLM runtime results now persist recent-run truth,
2. operator snapshot preserves that truth,
3. and Red Team renders the lane truthfully.

But the restriction-scoring spine is still mostly Scrapling-native:

1. `scrapling_exploit_progress`,
2. `scrapling_surface_contract`,
3. and the protected-evidence path that keys off strong live Scrapling runtime proof.

So today the repo can honestly say:

1. the LLM lane is visible,
2. but not that mixed-attacker pressure is part of the controller-grade restriction score.

## 3. LLM runtime receipts are not yet board-locus receipts

The current LLM runtime result carries:

1. action type,
2. path,
3. status,
4. error,
5. and category objective lineage.

That is enough for recent-run visibility and runtime lineage truth.

It is not yet enough for the same kind of board-state scoring the Scrapling lane already has, because the current loop still lacks:

1. named defense-surface coverage or breach receipts for LLM runtime actions,
2. a direct mapping from LLM action receipts into board loci,
3. and controller-grade host-cost or repair-family localization for those actions.

## 4. The current dev minimum is meaningful for Scrapling, but not automatically for the LLM lane

The runtime clamps adversary-sim runs to a minimum of `30s`, and runtime-dev candidate follow-on runs deliberately use that minimum as the shortest meaningful local proof window.

That is a sound Scrapling local-dev shortcut because:

1. the shared-host heartbeat cadence is `1s`,
2. Scrapling cycles five fulfillment modes,
3. and a `30s` run spans roughly thirty beats and about six full persona rotations if the worker keeps completing.

But the LLM lane currently declares higher per-tick time budgets:

1. `browser_mode` allows up to `90s`,
2. `request_mode` allows up to `120s`.

So a mixed-attacker loop cannot simply reuse a single `30s` follow-on duration and claim both lanes were given meaningful opportunity.

## 5. A truthful mixed-attacker proof therefore needs a broader architecture change, not just backlog unblocking

The smallest truthful combined-attacker design is not simultaneous active lanes.

It is:

1. one bounded episode that requires evidence from a defined set of attacker lanes,
2. sequential lane materialization within that episode,
3. lane-specific duration truth,
4. and a lane-native restriction score that can aggregate both attackers after the runs complete.

Without that, `RSI-GAME-HO-2` would only restate:

1. Scrapling is scored,
2. LLM is visible,
3. but the controller still acts on essentially Scrapling-only breach truth.

# Conclusions

## 1. `SIM-LLM-1C3` did remove the old blocker

The repo should stop saying `RSI-GAME-HO-2` is blocked by missing LLM runtime visibility, because that is no longer true.

## 2. The real blocker is now explicit

`RSI-GAME-HO-2` is blocked by three missing contracts:

1. multi-lane candidate-window and continuation orchestration,
2. lane-specific meaningful-duration truth,
3. and a mixed-attacker restriction score spine that is not Scrapling-only.

## 3. This is a whole-loop architecture change and should be approved explicitly before implementation

The next implementation step would touch:

1. adversary-sim state,
2. oversight candidate-window state,
3. continuation rerun state,
4. benchmark family assembly,
5. restriction diagnosis,
6. and Game Loop proof surfaces.

That is large enough that it should not be smuggled in as a "small follow-on".
