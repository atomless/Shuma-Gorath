Date: 2026-03-25
Status: Completed

Related context:

- [`2026-03-25-sim-llm-browser-1-container-browser-mode-readiness-review.md`](2026-03-25-sim-llm-browser-1-container-browser-mode-readiness-review.md)
- [`../plans/2026-03-25-sim-llm-browser-1-container-browser-mode-plan.md`](../plans/2026-03-25-sim-llm-browser-1-container-browser-mode-plan.md)

# SIM-LLM-BROWSER-1 Container Browser-Mode Post-Implementation Review

## What landed

`SIM-LLM-BROWSER-1` is now complete.

The live `bot_red_team` lane can now execute bounded browser-mode fulfillment through the existing containerized black-box seam instead of failing closed merely because the fulfillment plan chose browser actions.

The host orchestration path now keeps browser mode on the canonical LLM runtime seam in:

- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
- [`../../scripts/tests/adversarial_container_runner.py`](../../scripts/tests/adversarial_container_runner.py)
- [`../../scripts/tests/frontier_action_contract.py`](../../scripts/tests/frontier_action_contract.py)

That path now passes explicit allowed-tool capability inputs through the same frontier-action contract and into the existing container runner rather than creating a second browser-only validator.

The hardened container worker now executes a bounded browser action subset in:

- [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)
- [`../../scripts/tests/adversarial_container/Dockerfile`](../../scripts/tests/adversarial_container/Dockerfile)

The executed browser action set is intentionally narrow:

1. `browser_navigate`
2. `browser_snapshot`
3. `browser_click`

Receipts stay bounded and reuse the existing recent-run lineage shape rather than inventing a second browser-runtime reporting surface.

The Red Team dashboard path now proves the browser-mode seam through the existing recent-run presentation surface in:

- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)

That includes the route-level selectable-lane gate, which had still been silently rejecting `bot_red_team` even after the UI selector option and recent-run rendering path were ready. The focused dashboard test caught that stale gate truthfully, and the fix now admits `bot_red_team` through the same route control path as the other supported lanes.

## Verification

- `make test-adversarial-llm-browser-runtime`
- `git diff --check`

## Outcome against plan

This tranche met the planned acceptance bar:

1. `browser_mode` no longer fails closed just because it is browser mode,
2. the live `bot_red_team` lane can execute bounded browser actions through the existing capability-safe container boundary,
3. browser-owned categories now have executable runtime evidence instead of a paper-only runtime promise,
4. the existing recent-run and operator-snapshot projection seam remained the only truth surface,
5. and Red Team renders that evidence through the existing recent-run UI rather than a bespoke browser-runtime panel.

## Next active work

With the browser-mode blocker now closed, the next honest mainline tranche is:

1. `RSI-GAME-HO-2`

That follow-on should re-run the strict `human_only_private` Game Loop under combined Scrapling plus LLM attacker pressure and prove repeated retained config-change improvement before any later `humans_plus_verified_only` sweep opens.
