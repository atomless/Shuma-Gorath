Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)

# SIM-LLM-1C2 Post-Implementation Review

## What landed

`SIM-LLM-1C2` is now a real supervisor/runtime slice rather than another plan-only seam.

The host-side supervisor in [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs) now:

1. dispatches `dispatch_mode = "llm_fulfillment_plan"` alongside the existing Scrapling worker path,
2. routes that mode to a dedicated LLM runtime worker instead of overloading the Scrapling worker contract,
3. reads typed worker result files even when the worker exits non-zero, so fail-closed actor receipts are preserved instead of flattened into generic transport failures,
4. and posts those typed results back through the existing internal worker-result endpoint.

The new worker in [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py) now:

1. extracts the nested `llm_fulfillment_plan` from the internal beat payload,
2. reuses the landed live frontier generation seam from [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
3. keeps the attacker black-box by generating from only the host root plus bounded public hint seeds,
4. reuses the existing container black-box runner for request-mode execution,
5. emits a typed `adversary-sim-llm-runtime-result.v1` payload with provider lineage, counts, action receipts, and terminal-failure semantics,
6. and fails closed for browser-mode dispatch for now with an explicit typed result instead of pretending browser execution already exists.

On the Rust side, the live ingest path now exists too:

1. [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs) records pending LLM dispatch state and applies typed runtime results back into lane diagnostics and generation counters,
2. [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs) defines the typed runtime result and per-action receipt payloads,
3. [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs) accepts `adversary-sim-llm-runtime-result.v1` on the internal worker-result endpoint,
4. and [`../../src/admin/api.rs`](../../src/admin/api.rs) now proves the internal beat -> worker result -> persisted control-state loop for the `bot_red_team` lane.

## Verification

- `make test-adversarial-llm-runtime-dispatch`
- `make test-adversary-sim-make-target-contract`
- `make test-adversarial-python-unit`
- `git diff --check`

## Outcome Against Plan

This tranche met the `SIM-LLM-1C2` acceptance bar:

1. `bot_red_team` now has a real supervisor dispatch path rather than only a plan-shaped beat payload,
2. the runtime result contract is typed and separate from the Scrapling worker schema,
3. request-mode execution reuses the existing capability-safe container runner instead of inventing a second one-off executor,
4. fail-closed runtime outcomes preserve typed receipts and lineage instead of collapsing into generic transport noise,
5. and the focused proof now covers Rust ingest, supervisor dispatch knowledge, and the Python worker/runtime helper contract together.

## Remaining Gap

This still does **not** make the full attacker runtime operator-complete.

Two important gaps remain:

1. browser-mode execution still fails closed and is not yet a real executed browser actor,
2. and the recent-run / operator-snapshot / dashboard projection chain still does not surface the live `bot_red_team` runtime receipts end to end.

Those belong to `SIM-LLM-1C3`.

## Follow-On

The next backend mainline is now `SIM-LLM-1C3`.
