Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-sim-llm-browser-1-container-browser-mode-readiness-review.md`](../research/2026-03-25-sim-llm-browser-1-container-browser-mode-readiness-review.md)
- [`2026-03-24-llm-player-role-decomposition-plan.md`](2026-03-24-llm-player-role-decomposition-plan.md)
- [`2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md`](2026-03-22-path-to-closed-loop-llm-adversary-and-diagnosis-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# SIM-LLM-BROWSER-1 Container Browser-Mode Implementation Plan

## Objective

Land bounded executed browser-mode fulfillment for the live `bot_red_team` lane without inventing a new runtime architecture.

## Core decisions

1. Browser mode must run through the existing containerized black-box lane, not a new host-only browser path.
2. The host `llm_runtime_worker.py` remains the single orchestration seam.
3. Mode-specific browser actions must continue to pass through the frontier action contract and capability-envelope validation path.
4. The existing `SIM-LLM-1C3` recent-run and snapshot projection path remains the output seam; this tranche only needs to feed it real browser receipts.
5. Red Team should render the browser-mode evidence through the existing recent-run presentation path rather than through a bespoke new panel.

## Task 1: add focused failing tests

Files:

1. [`../../scripts/tests/test_llm_runtime_worker.py`](../../scripts/tests/test_llm_runtime_worker.py)
2. [`../../scripts/tests/test_adversarial_container_worker.py`](../../scripts/tests/test_adversarial_container_worker.py)
3. [`../../scripts/tests/test_adversarial_container_runner.py`](../../scripts/tests/test_adversarial_container_runner.py)
4. [`../../scripts/tests/test_adversary_sim_make_targets.py`](../../scripts/tests/test_adversary_sim_make_targets.py)
5. dashboard rendered proof in:
   - [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
   - [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)

Add failing coverage that proves:

1. browser mode no longer short-circuits in the host worker,
2. browser actions can be admitted into the container worker only through explicit allowed-tool overrides,
3. the container command carries the browser-mode action admission inputs,
4. a focused Make target exists for the browser-mode proof path,
5. and the dashboard run surface can render a `bot_red_team` browser-mode row.

## Task 2: extend the existing black-box path for browser actions

Files:

1. [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
2. [`../../scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py)
3. [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)
4. [`../../scripts/tests/frontier_action_contract.py`](../../scripts/tests/frontier_action_contract.py)

Implementation requirements:

1. pass explicit mode-allowed tools from the fulfillment plan into the container worker,
2. reuse the same frontier action validation path with explicit overrides instead of a second validator,
3. execute:
   - `browser_navigate`
   - `browser_snapshot`
   - `browser_click`
4. keep receipts bounded and shaped like the existing request-mode receipts,
5. preserve fail-closed behavior for invalid actions or execution errors.

## Task 3: provision browser runtime inside the existing container lane

Files:

1. [`../../scripts/tests/adversarial_container/Dockerfile`](../../scripts/tests/adversarial_container/Dockerfile)
2. any runtime helper files required by the chosen implementation under:
   - [`../../scripts/tests/adversarial_container/`](../../scripts/tests/adversarial_container/)

Requirements:

1. keep the image non-root,
2. keep the runtime within the existing hardening profile,
3. and provision the minimum browser automation runtime needed for bounded Playwright execution.

## Task 4: prove the end-to-end seam

Files:

1. [`../../Makefile`](../../Makefile)
2. [`../../docs/testing.md`](../../docs/testing.md)
3. if needed, recent-run/output proof files already on the `SIM-LLM-1C3` path

Add a focused proof target that covers:

1. browser-mode dispatch,
2. container browser execution contract,
3. recent-run/operator-snapshot projection still receiving the result,
4. and dashboard rendering of the resulting browser-mode run summary.

## Acceptance criteria

`SIM-LLM-BROWSER-1` is complete only when:

1. `browser_mode` no longer fails closed just because it is browser mode,
2. the live `bot_red_team` lane can execute bounded browser actions through the existing containerized black-box worker,
3. browser-owned categories are now backed by executable runtime evidence rather than a paper contract,
4. recent-run and snapshot projection continue to work without a second projection path,
5. and Red Team can render the browser-mode evidence without any new design language or bespoke panel.
