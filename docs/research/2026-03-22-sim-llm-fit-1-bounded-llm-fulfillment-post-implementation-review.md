# SIM-LLM-FIT-1 Post-Implementation Review

Date: 2026-03-22

## Scope reviewed

- `SIM-LLM-FIT-1`
- Plan reference: [`../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md)

## What landed

1. Shared-host adversary-sim heartbeat routing now emits a bounded `llm_fulfillment_plan` for the `bot_red_team` lane instead of the earlier unimplemented placeholder.
2. The contract is explicit and typed:
   - `browser_mode`
   - `request_mode`
   - `frontier_reference`
   - `local_candidate`
   - `configured` / `degraded` / `unavailable` backend state
3. The same bounded contract is now represented in the adversarial Python tooling through shared frontier-action and container-runtime nested contract sections, plus a focused loader and plan builder.
4. Focused verification is now first-class through `make test-adversarial-llm-fit`.

## Acceptance check

### 1. Shuma has a concrete bounded LLM fulfillment actor contract rather than a vague future lane

Passed.

- Runtime contract and planner live in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs).
- Internal beat payload surfaces the contract through [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs) and the admin test proof in [`../../src/admin/api.rs`](../../src/admin/api.rs).
- Python tooling mirrors the same contract in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py).

### 2. Browser-vs-request mode is explicit and testable

Passed.

- Runtime mode alternation and backend-state behavior are covered in [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs).
- Python contract loading and plan shaping are covered in [`../../scripts/tests/test_llm_fulfillment.py`](../../scripts/tests/test_llm_fulfillment.py).

### 3. Frontier is the initial reference backend for the high-capability categories

Passed, with truthful degradation semantics.

- `frontier_reference` is the current reference backend for both modes.
- `local_candidate` is exposed only as a supported later backend kind, not as an authoritative execution path yet.
- Single-provider frontier diversity is now explicitly reported as `degraded`, which is more truthful than the earlier over-optimistic `configured` expectation uncovered during implementation verification.

## Verification run

1. `make test-adversarial-llm-fit`
2. `make test-adversarial-runner-architecture`
3. `make test-adversary-sim-runtime-surface`
4. `git diff --check`

## Architectural review

The tranche stayed inside the intended boundary:

- it added planning contracts and payload truth,
- it did not add freeform LLM execution on the request path,
- it kept the later full `SIM-LLM-1` actor blocked,
- and it reused the existing frontier/container contract surfaces rather than introducing a parallel LLM-only schema family.

That is the right shape for the current stage because later fulfillment, coverage, and protected-evidence tranches now have a real, bounded LLM lane contract to build on without letting the system pretend it already has credible autonomous LLM traffic generation.

## Shortfalls found

No remaining tranche-local shortfall is left open.

One truthfulness issue was found and fixed during the tranche: single-provider frontier state was initially asserted as `configured` in tests, but the contract correctly treats reduced provider diversity as `degraded`. The tests and closeout record now reflect the truthful behavior.

## Next step

Proceed to `SIM-FULFILL-1`.
