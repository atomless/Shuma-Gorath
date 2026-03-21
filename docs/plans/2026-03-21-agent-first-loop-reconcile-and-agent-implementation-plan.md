# Agent-First Loop Reconcile And Agent Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Land the first backend recommend-only reconcile engine and the first shared-host agent tweaker harness that reads machine-first evidence, produces typed proposals, supports both periodic and post-sim triggers, and stays strictly off the request path.

**Architecture:** Reuse the existing adversary-sim lease and idempotency model, the current hot-read snapshot and benchmark contracts, and the bounded `allowed_actions_v1` surface. Keep `OVR-RECON-1` pure and recommend-only. Keep `OVR-AGENT-1` as a shared-host control-plane harness that calls the same internal reconcile contract whether it is running periodically or immediately after a qualifying sim run.

**Tech Stack:** Rust admin modules, existing adversary-sim control plane, shared-host supervisor scripts, Makefile verification, repo-native docs and TODO workflow.

---

## Guardrails

1. No request-path model execution.
2. No edge-gateway execution for the first agent loop.
3. No config apply in this tranche; proposals only.
4. Periodic and post-sim triggers must route through one internal reconcile or agent contract, not two controller implementations.
5. Evidence-stale, contradictory, or degraded inputs must fail closed to no change.

## Task 0: Focused Verification Prep

**Files:**
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add a truthful focused `make` target for reconcile logic, for example `test-oversight-reconcile`.
2. Add a truthful focused `make` target for the first shared-host agent harness, for example `test-oversight-agent`.
3. Add a truthful focused `make` target for post-sim trigger behavior, for example `test-oversight-post-sim-trigger`.
4. Keep the targets specific about whether they verify pure reconcile logic, agent scheduling or triggering, or read-surface shaping.

**Acceptance criteria:**
1. Later controller work can be proved without overusing broad integration suites.

## Task 1: `OVR-RECON-1`

**Files:**
- Create: `src/admin/oversight_reconcile.rs`
- Create: `src/admin/oversight_patch_policy.rs`
- Create: `src/admin/oversight_decision_ledger.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/admin/mod.rs`
- Modify: `docs/api.md`
- Modify: `docs/configuration.md`

**Work:**
1. Implement a pure recommend-only reconcile engine that consumes `operator_snapshot_v1`, `benchmark_results_v1`, replay-promotion status, and `allowed_actions_v1`.
2. Implement patch-policy evaluation against the bounded config families already exposed by `allowed_actions_v1`.
3. Implement typed reconcile outputs that include patch family, patch body, expected impact, confidence, required verification, and explicit no-change or degraded-state outcomes.
4. Implement durable decision-ledger persistence for proposals and refusals.
5. Reuse the lease or idempotency patterns already proven in `src/admin/adversary_sim_control.rs` where ownership or repeat execution semantics matter.
6. Keep this tranche recommend-only: no config writes and no watch-window apply loop yet.

**Acceptance criteria:**
1. reconcile logic is unit-testable and pure enough to run without side effects,
2. patch families are bounded and typed,
3. stale or contradictory evidence yields explicit no-change outcomes,
4. and proposal lineage is durably recorded.

**Verification:**
1. `make test-oversight-reconcile`
2. `make test-runtime-preflight-unit`
3. `git diff --check`

## Task 2: `OVR-AGENT-1`

**Files:**
- Create: `src/admin/oversight_agent.rs`
- Create: `src/admin/oversight_api.rs`
- Modify: `src/admin/api.rs`
- Modify: `src/admin/mod.rs`
- Modify: `src/admin/adversary_sim.rs`
- Modify: `src/admin/adversary_sim_control.rs`
- Create: `scripts/run_with_oversight_supervisor.sh`
- Modify: `Makefile`
- Modify: `docs/deployment.md`
- Modify: `docs/testing.md`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Implement the first shared-host agent harness that calls the internal reconcile contract and records typed proposal results.
2. Support periodic execution through a host-side supervisor or timer wrapper.
3. Support immediate post-sim triggering through the same internal contract when a qualifying adversary-sim run completes.
4. Add bounded status or history read surfaces so operators and later Monitoring can inspect the latest agent run, trigger source, proposal, and refusal reasons.
5. Keep all execution off the request path and out of the edge runtime.
6. Keep this first agent loop recommend-only: it may propose, log, and request reruns, but must not mutate config in this tranche.

**Acceptance criteria:**
1. the same internal contract is reused for periodic and post-sim invocation,
2. the first agent loop can read live and sim evidence end to end,
3. proposal semantics are durable enough to become the truth source for later Monitoring and Tuning work,
4. and the shared-host-only execution boundary is explicit and tested.

**Verification:**
1. `make test-oversight-agent`
2. `make test-oversight-post-sim-trigger`
3. `make test-adversary-sim-runtime-surface`
4. `git diff --check`

## Exit Criteria

This plan is complete when:

1. Shuma has a pure recommend-only reconcile engine,
2. the first shared-host agent harness can invoke it periodically and after sim completion through one contract,
3. proposal and refusal lineage is durable and inspectable,
4. and Monitoring or Tuning can be designed against proven backend behavior rather than hypothetical controller semantics.
