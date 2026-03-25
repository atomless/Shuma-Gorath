Date: 2026-03-25
Status: Completed

# `SIM-SCR-FULL-1B2B` Browser Challenge Interactions Review

## Context

`SIM-SCR-FULL-1B2A` made the repo-owned Scrapling runtime provision a real Playwright browser and fail readiness closed until that browser is actually executable.

That removed the runtime blocker, but the worker still drives all Scrapling-owned challenge surfaces through request-native `post(...)` calls. So the current lane is still under-using the newly landed browser seam exactly where request-native execution is least truthful: DOM-backed not-a-bot and puzzle interactions.

## Findings

1. The browser runtime is now executable in this worktree, not just importable.
   - After rerunning `make setup-runtime`, both `DynamicSession` and `StealthySession` can launch and fetch a real local page from `.venv-scrapling`.

2. Scrapling browser sessions are `fetch(url, ..., page_action=...)`, not `get/post`.
   - The browser path is therefore a distinct execution seam, not a drop-in replacement for the existing direct-request helper.

3. Existing Shuma browser attack logic already defines the truthful DOM paths we should mirror.
   - `scripts/tests/adversarial_browser_driver.mjs` already contains:
     - a human-like not-a-bot checkbox activation path,
     - a puzzle wrong-output path that routes to maze,
     - and the canonical selectors those flows depend on.

4. Browser requests can still carry Shuma sim-tag headers.
   - `DynamicSession` and `StealthySession` use page-level extra HTTP headers, and local experiments showed those headers survive form submission requests as well as the initial navigation.

5. Final browser response status is not enough to classify challenge outcomes truthfully.
   - A browser-driven not-a-bot pass may end on a `200` after redirect and should count as `pass_observed`.
   - A browser-driven puzzle wrong-answer flow may also end on a `200` maze page and should count as `fail_observed`.
   - So the worker must capture semantic browser evidence from the DOM path and submit response, not only the final HTTP code returned by `session.fetch(...)`.

## Implication

The first browser-backed Scrapling slice should:

1. make `not_a_bot_submit` browser-backed and capable of honest success where that defense is realistically bypassable,
2. make `puzzle_submit_or_escalation` browser-backed and capable of honest failure or escalation through the real DOM,
3. keep `pow_verify_abuse` and `tarpit_progress_abuse` on the current direct-request path for now,
4. and update the owned-surface contract so `required_transport` and `success_contract` reflect those real browser interactions.

## Next step

Implement `SIM-SCR-FULL-1B2B` as the first real browser-backed challenge interaction tranche over:

- `scripts/supervisor/scrapling_worker.py`
- `scripts/tests/test_scrapling_worker.py`
- `src/observability/scrapling_owned_surface.rs`

with focused proof that:

1. dynamic or stealth Scrapling can pass not-a-bot through the DOM,
2. dynamic or stealth Scrapling can drive puzzle failure or escalation through the DOM,
3. and owned-surface receipts classify those outcomes truthfully.
