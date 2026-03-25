# MZ-T2 Live Maze Browser E2E Plan

Date: 2026-03-24
Status: Implemented
Parent plan: [`2026-02-25-maze-carry-forward-plan.md`](2026-02-25-maze-carry-forward-plan.md)

## Goal

Land the missing browser end-to-end proof for live maze behavior under real Chromium session semantics.

## Scope

Add a focused local gate that proves:

1. JavaScript-enabled maze traversal performs checkpointed hidden-link progression and replay rejection.
2. JavaScript-disabled traversal stays bounded and falls back deterministically once checkpoint is required.
3. Deep-tier micro-PoW is exercised through a real browser-managed maze link.
4. Repeated high-confidence maze violations escalate from challenge to block under the real browser path.

## Implementation shape

- Add a dedicated live browser gate at [`scripts/tests/maze_live_browser.py`](../../scripts/tests/maze_live_browser.py).
- Reuse [`scripts/tests/maze_live_traversal.py`](../../scripts/tests/maze_live_traversal.py) helpers for live config/event orchestration rather than duplicating admin/read/restore logic.
- Extend [`scripts/tests/adversarial_browser_driver.mjs`](../../scripts/tests/adversarial_browser_driver.mjs) with maze-specific browser actions instead of creating a new browser runner.
- Add focused helper tests at:
  - [`scripts/tests/test_maze_live_browser.py`](../../scripts/tests/test_maze_live_browser.py)
  - [`scripts/tests/test_adversarial_browser_driver.mjs`](../../scripts/tests/test_adversarial_browser_driver.mjs) for any new pure browser-driver helpers.
- Add canonical `make` targets:
  - `make test-maze-live-browser-unit`
  - `make test-maze-live-browser-contract`

## Coverage contract

The live browser gate should run these sub-scenarios:

1. **JS-enabled checkpoint + issue-links + replay**
   - discover the opaque public prefix through admin preview,
   - drive the browser through entry -> visible tokenized link -> hidden issued link,
   - assert checkpoint and issue-links requests were observed,
   - replay the first tokenized link until deterministic block fallback is observed,
   - assert persisted `maze_token_replay action=block`.

2. **JS-disabled checkpoint-missing fallback**
   - run with browser JavaScript disabled,
   - allow bounded progress up to the configured no-JS depth,
   - assert deeper traversal returns the challenge fallback,
   - assert persisted `maze_checkpoint_missing action=challenge`.

3. **JS-enabled micro-PoW**
   - enable maze micro-PoW at a shallow depth,
   - assert the maze link advertises browser-visible `data-pow-difficulty`,
   - assert click-driven traversal succeeds through the micro-PoW-protected link instead of challenge fallback.

4. **Repeated high-confidence escalation**
   - repeat the JS-disabled checkpoint-missing path from the same IP bucket over distinct opaque entry paths,
   - assert challenge on early attempts and block on the terminal attempt,
   - assert persisted `maze_checkpoint_missing action=block`.

## Guardrails

- Keep the browser driver black-box to admin internals: the live browser action should receive only the public maze entry path and runtime-safe options.
- Keep browser runtime setup on the shared Playwright bootstrap path in [`scripts/tests/playwright_runtime.py`](../../scripts/tests/playwright_runtime.py).
- Avoid broad integration-shell edits; keep this as a focused maze/browser gate with truthful `make` target names.
- Write the live report to `.spin/maze_live_browser.json` so the proof artifact remains local and bounded.
