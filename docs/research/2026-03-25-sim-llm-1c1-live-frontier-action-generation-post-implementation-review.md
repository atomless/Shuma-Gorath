Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)

# SIM-LLM-1C1 Post-Implementation Review

## What landed

`SIM-LLM-1C1` is now a real action-generation seam rather than a plan-shaped placeholder.

The Python LLM fulfillment helper in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py) now:

1. builds a host-root-only and Shuma-blind generation context from the settled fulfillment plan,
2. strips forbidden or internal-looking public hint paths before they ever enter the provider prompt,
3. selects configured frontier providers from the canonical provider inventory,
4. performs a real provider-backed generation attempt through a single canonical executor path,
5. validates generated actions against the existing frontier-action contract and the mode-specific capability envelope,
6. and records explicit provider-vs-fallback lineage instead of silently pretending degraded fallback is frontier output.

The shared frontier-action validator in [`../../scripts/tests/frontier_action_contract.py`](../../scripts/tests/frontier_action_contract.py) now also supports mode-specific tool overrides so the later browser-mode actor can validate bounded browser actions against the same contract family instead of growing a second one-off validator.

## Verification

- `make test-adversarial-llm-fit`
- `git diff --check`

## Outcome Against Plan

This tranche met the `SIM-LLM-1C1` acceptance bar:

1. the repo now has a real live generation adapter instead of only frontier metadata,
2. generation stays bounded by the existing black-box and episode contracts,
3. forbidden Shuma-specific knowledge is filtered from the prompt context,
4. provider-backed output is distinguishable from degraded fallback,
5. and the focused proof covers provider success, missing-key fallback, and mode-validation fallback.

One important improvement over the earlier deterministic default path is that degraded fallback no longer injects Shuma-specific public route guesses like `/sim/public/...`; it now stays anchored to the host root plus sanitized public hints only.

## Remaining Gap

This does **not** yet create a live executed `bot_red_team` actor.

The runtime still stops at generated actions:

1. the supervisor does not yet dispatch `llm_fulfillment_plan`,
2. the internal API does not yet ingest a typed LLM runtime result,
3. and the container runtime still does not execute browser-mode actions end to end.

Those remain the next two explicit slices:

1. `SIM-LLM-1C2` supervisor dispatch and typed result ingest,
2. `SIM-LLM-1C3` runtime proof closure and recent-run projection.

## Follow-On

The next backend mainline is now `SIM-LLM-1C2`.
