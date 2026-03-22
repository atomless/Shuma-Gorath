# `OVR-AGENT-1` Shared-Host Agent Loop Post-Implementation Review

Date: 2026-03-21  
Status: complete

## Scope reviewed

`OVR-AGENT-1` was meant to land the first shared-host recommend-only agent harness over the now-truthful machine-first snapshot, benchmark, allowed-action, replay-promotion, and reconcile contracts, with one invocation contract shared between periodic and post-sim triggering.

Expected outcomes:

1. bounded shared-host agent-run persistence,
2. one agent execution contract reused by periodic and post-sim triggers,
3. bounded latest-run and recent-run status projection for later Monitoring and Tuning work,
4. explicit shared-host-only execution boundary,
5. and host-side wrapper support that keeps the first loop off the request path.

## What landed

1. `src/admin/oversight_agent.rs` now defines the persisted `oversight_agent_run_v1` contract, periodic and post-sim trigger kinds, bounded replay-once semantics for completed sim runs, latest/recent run loading, and the shared-host-only execution/status helpers.
2. `src/admin/oversight_api.rs`, `src/admin/api.rs`, and `src/admin/auth.rs` now expose `GET /admin/oversight/agent/status` plus `POST /internal/oversight/agent/run`, guarded by the internal `oversight-agent` supervisor trust boundary and backed by the same `execute_reconcile_cycle` contract used by the manual reconcile path.
3. `src/admin/adversary_sim_api.rs` now triggers the same agent execution contract immediately after qualifying sim completion on shared-host deployments, both for bounded internal-beat completions and admin-driven completion transitions.
4. `scripts/run_with_oversight_supervisor.sh` now chains the existing adversary-sim supervisor wrapper and adds bounded periodic `POST /internal/oversight/agent/run` calls on shared-host startup paths, while the `Makefile` dev and prod wrappers now use that chain by default.
5. `docs/api.md`, `docs/testing.md`, `docs/deployment.md`, and `docs/adversarial-operator-guide.md` now document the status surface, the focused verification gates, and the new wrapper/runtime contract.

## Verification performed

1. `make test-oversight-agent`
2. `make test-oversight-post-sim-trigger`
3. `make test-adversary-sim-runtime-surface`
4. `git diff --check`

## Shortfall found during review

### `OVR-AGENT-1-REVIEW-1`

Initial implementation exposed the first agent loop without an explicit shared-host-only runtime gate, which meant the internal route and post-sim helper could still execute on edge deployment profiles even though the plan forbids that.

Fix executed immediately:

1. added `shared_host_execution_available()` gating in `src/admin/oversight_agent.rs`,
2. made `POST /internal/oversight/agent/run` return `404` outside shared-host mode in `src/admin/oversight_api.rs`,
3. made post-sim trigger helpers no-op on edge profiles and added focused edge-profile regression tests,
4. reran the focused agent, post-sim, and live runtime-surface gates.

## Final assessment

`OVR-AGENT-1` now meets the plan intent:

1. periodic and post-sim invocation reuse one bounded backend contract,
2. the first agent loop is durable, inspectable, and recommend-only,
3. the shared-host-only execution boundary is now explicit in code and tests,
4. the existing adversary-sim runtime-surface proof still passes with the new wrapper and hook in place,
5. and no tranche-local shortfall remains open before `MON-OVERHAUL-1`.
