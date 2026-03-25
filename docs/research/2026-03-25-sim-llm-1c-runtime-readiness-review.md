Date: 2026-03-25
Status: Proposed

Related context:

- [`2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md`](2026-03-25-sim-llm-1a-black-box-contract-post-implementation-review.md)
- [`2026-03-25-sim-llm-1b-episode-harness-post-implementation-review.md`](2026-03-25-sim-llm-1b-episode-harness-post-implementation-review.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md`](../plans/2026-03-24-mainline-resequence-scrapling-before-game-loop-plan.md)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs)
- [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
- [`../../scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# SIM-LLM-1C Runtime Readiness Review

## Question

Now that `SIM-LLM-1A` and `SIM-LLM-1B` are landed, is the repo actually ready to treat `SIM-LLM-1C` as one executable next implementation slice?

## Conclusion

No.

`SIM-LLM-1C` is still underplanned as a single runtime item.
The repo now has the black-box boundary and episode harness, but it still lacks two separate implementation layers:

1. a live frontier action-generation backend that produces real attacker actions under the settled black-box contract,
2. and a runtime dispatch/result path that can actually execute and ingest `bot_red_team` work rather than only emitting plans.

So the truthful next step is not "implement `SIM-LLM-1C`."
It is to decompose that umbrella into smaller runtime slices and reopen them deliberately.

## What is already real

The settled contract side is in place:

1. the black-box boundary is executable in:
   - [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json)
   - [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
   - [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
2. the episode harness and bounded-memory contract are also executable in those same surfaces,
3. and the internal beat payload already emits a typed `llm_fulfillment_plan`.

That is a good base, but it is still a contract base rather than a full runtime actor.

## The first missing seam: no live frontier action-generation backend

The current `bot_red_team` path still does not call a live provider-backed attack generator.

What exists today is:

1. provider metadata and provider-count scoring in [`../../scripts/tests/adversarial_runner/discovery_scoring.py`](../../scripts/tests/adversarial_runner/discovery_scoring.py),
2. contract loading and fulfillment-plan shaping in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
3. and the container black-box executor in [`../../scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py).

What does not yet exist is a live attacker-action generation adapter that:

1. selects a configured frontier provider,
2. generates bounded attacker actions from only host-derived observations,
3. validates those actions through the existing reject-by-default action contract,
4. and records receipt lineage that distinguishes real provider output from deterministic fallback.

So the repo currently has frontier metadata, not a live frontier attacker.

## The second missing seam: no `bot_red_team` execution and ingest path

The live lane runtime still stops at plan emission.

Evidence:

1. [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs) emits `summary.llm_fulfillment_plan = Some(plan)` for `RuntimeLane::BotRedTeam`,
2. [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs) returns `dispatch_mode = "llm_fulfillment_plan"` at the internal beat boundary,
3. but [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs) only dispatches the Scrapling worker path,
4. and [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs) only ingests `ScraplingWorkerResult` on the worker-result endpoint.

So the live `bot_red_team` lane still cannot:

1. hand off a plan to a real actor,
2. run the container black-box worker,
3. ingest the resulting actor report,
4. and materialize that runtime evidence into recent-run or monitoring truth.

## The third missing seam: no full runtime proof chain

Even after dispatch and ingest exist, the runtime still will not be complete until there is proof for:

1. run creation,
2. action execution,
3. result ingestion,
4. recent-run observability,
5. and coverage or category receipt projection.

Without that, `SIM-LLM-1C` would still risk becoming another status-only actor that claims runtime maturity without full-path proof.

## Decision

Do not reopen `SIM-LLM-1C` as one monolithic next task.

Instead:

1. keep the umbrella blocked,
2. split it into:
   - `SIM-LLM-1C1` live frontier action-generation backend
   - `SIM-LLM-1C2` supervisor dispatch and result-ingest runtime wiring
   - `SIM-LLM-1C3` runtime receipts, recent-run projection, and proof closure
3. make `SIM-LLM-1C1` the next backend mainline slice,
4. and keep `bot_red_team` visibly disabled until all three runtime slices are landed and verified.
