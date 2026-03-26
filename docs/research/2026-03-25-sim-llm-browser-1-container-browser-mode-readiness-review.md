Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-sim-llm-1c3-runtime-proof-closure-post-implementation-review.md`](../research/2026-03-25-sim-llm-1c3-runtime-proof-closure-post-implementation-review.md)
- [`../research/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md`](../research/2026-03-25-delay-humans-plus-verified-until-combined-attacker-proof-review.md)
- [`../plans/2026-03-24-llm-player-role-decomposition-plan.md`](../plans/2026-03-24-llm-player-role-decomposition-plan.md)
- [`../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](../plans/2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
- [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)
- [`../../scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py)
- [`../../scripts/tests/adversarial_browser_driver.mjs`](../../scripts/tests/adversarial_browser_driver.mjs)
- [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json)
- [`../../scripts/tests/adversarial/container_runtime_profile.v1.json`](../../scripts/tests/adversarial/container_runtime_profile.v1.json)

# SIM-LLM-BROWSER-1 Container Browser-Mode Readiness Review

## What is already true

The repo already declares browser mode as the intended fulfillment path for the browser-owned categories.

That contract already exists in:

1. [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
2. [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
3. [`../../scripts/tests/adversarial/frontier_action_contract.v1.json`](../../scripts/tests/adversarial/frontier_action_contract.v1.json)
4. [`../../scripts/tests/adversarial/container_runtime_profile.v1.json`](../../scripts/tests/adversarial/container_runtime_profile.v1.json)

So the repo already says:

1. `bot_red_team` alternates into `browser_mode`,
2. browser mode owns:
   - `automated_browser`
   - `browser_agent`
   - `agent_on_behalf_of_human`
3. browser automation is allowed,
4. and the LLM action contract allows:
   - `browser_navigate`
   - `browser_snapshot`
   - `browser_click`

## What is not yet true

The execution path still stops short of that contract.

Current gaps:

1. [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py) still fail-closes browser mode with `browser_mode_not_supported`.
2. [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py) still only executes urllib GET traffic.
3. [`../../scripts/tests/frontier_action_contract.py`](../../scripts/tests/frontier_action_contract.py) still defaults to request-only action types unless explicit overrides are supplied.
4. [`../../scripts/tests/adversarial_container/Dockerfile`](../../scripts/tests/adversarial_container/Dockerfile) does not yet provision a browser automation runtime.

So browser mode is currently:

1. modeled,
2. scheduled,
3. and projected,

but not actually executed.

## Design constraints for the next slice

The next slice should preserve the existing architecture rather than invent a parallel one.

Required constraints:

1. keep the existing capability-safe container boundary,
2. keep the host worker as the single orchestration seam,
3. keep the frontier action contract and capability-envelope path as the only action-admission path,
4. reuse existing repo browser automation patterns where they already exist,
5. and avoid introducing a second host-side browser runtime just for the LLM lane.

That means the right direction is:

1. extend the existing container black-box lane to execute browser actions,
2. reuse the existing repo Playwright/browser-driver patterns as design input,
3. and keep recent-run and operator-snapshot projection on the already-landed `SIM-LLM-1C3` path.

## Recommended implementation shape

`SIM-LLM-BROWSER-1` should land one bounded browser-mode execution path over the current contracts.

The minimal truthful slice is:

1. pass mode-specific allowed tool overrides from the host worker into the container worker,
2. teach the container worker to execute `browser_navigate`, `browser_snapshot`, and bounded `browser_click`,
3. keep execution inside the black-box container boundary,
4. emit bounded browser receipts through the existing `LlmRuntimeResult` shape,
5. and prove that a browser-mode run now reaches the recent-run/operator-snapshot path rather than fail-closing immediately.

## Proof expectation

This slice is only complete if the proof covers:

1. failing tests first for the unsupported browser-mode gap,
2. focused runtime dispatch proof,
3. container/browser contract proof,
4. recent-run projection proof,
5. and at least one rendered dashboard proof that `Red Team` can surface a `bot_red_team` browser-mode run without bespoke UI work.

Anything weaker would still leave the mixed-attacker strict-loop gate resting on a paper contract instead of an executable one.
