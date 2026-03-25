# MZ-T2 Live Maze Browser E2E Post-Implementation Review

Date: 2026-03-24
Status: Complete

## Delivered

`MZ-T2` is now covered by a focused live Chromium gate in:

- [`scripts/tests/maze_live_browser.py`](../../scripts/tests/maze_live_browser.py)
- [`scripts/tests/test_maze_live_browser.py`](../../scripts/tests/test_maze_live_browser.py)
- [`scripts/tests/adversarial_browser_driver.mjs`](../../scripts/tests/adversarial_browser_driver.mjs)
- [`scripts/tests/test_adversarial_browser_driver.mjs`](../../scripts/tests/test_adversarial_browser_driver.mjs)
- [`Makefile`](../../Makefile) as:
  - `make test-maze-live-browser-unit`
  - `make test-maze-live-browser-contract`

The live browser gate now proves:

- JS-enabled maze traversal performs browser-managed checkpoint POST and hidden-link issuance,
- browser-managed progression continues through an issued hidden link,
- replayed maze traversal tokens deterministically block and persist `maze_token_replay action=block`,
- JS-disabled traversal falls back with persisted `maze_checkpoint_missing action=challenge`,
- deep-tier micro-PoW exposes a browser-visible protected link and still progresses through the maze,
- repeated checkpoint-missing browser attempts escalate from challenge to block with persisted `maze_checkpoint_missing action=block`.

## Important implementation notes

- The live gate keeps orchestration on the existing Python maze-live path and keeps browser execution on the existing Playwright adversarial driver path; no second browser harness was added.
- Public maze paths are still discovered from admin preview, but the high-confidence escalation attempts deliberately derive distinct public entry paths under the discovered opaque prefix so repeated checkpoint-missing proof does not collapse into token-replay proof.
- The browser driver now accepts explicit maze actions and optional `javaScriptEnabled` control so the JS and no-JS cohorts are proven through the same canonical browser runtime.

## Follow-on

- `MZ-T3` is still required for concurrency and soak coverage over maze state and budget primitives.
- `MZ-T4` is still required to promote the new live maze browser gate into broader canonical CI/full-suite paths once the remaining maze carry-forward work is complete.
