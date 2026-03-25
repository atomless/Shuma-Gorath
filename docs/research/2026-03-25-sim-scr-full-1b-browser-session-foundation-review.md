Date: 2026-03-25
Status: Proposed planning driver

Related context:

- [`2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md`](2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-review.md)
- [`../plans/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-plan.md`](../plans/2026-03-25-sim-scr-full-1a-non-agent-capability-matrix-refresh-plan.md)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../scripts/tests/test_scrapling_worker.py`](../../scripts/tests/test_scrapling_worker.py)
- [Scrapling dynamic fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
- [Scrapling stealth fetchers docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy.html)

# Question

What is the smallest truthful first coding slice for `SIM-SCR-FULL-1B` now that the refreshed matrix says dynamic and stealth Scrapling capability is in-scope for Scrapling-owned surfaces?

# Findings

## 1. The current worker is structurally request-native

[`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py) assumes request-native sessions:

1. `_import_scrapling()` imports only `FetcherSession`,
2. direct personas execute through `_execute_request_sequence(...)`,
3. and that executor assumes verb methods like `get`, `post`, and `put`.

That shape was fine for the earlier request-native lane, but it is not yet a truthful seam for dynamic or stealth sessions.

## 2. Browser sessions are not drop-in replacements

The official docs and the installed runtime both show that `DynamicSession` and `StealthySession` expose `fetch(...)` with browser-automation arguments like:

1. `page_action`,
2. `wait_selector`,
3. `network_idle`,
4. `solve_cloudflare`,
5. `block_webrtc`,
6. `hide_canvas`,
7. `allow_webgl`.

They do **not** expose the same `get/post/put/delete` API shape the current direct personas use.

So the first full-power implementation slice cannot honestly be “swap `FetcherSession` for `StealthySession` and keep the rest.”

## 3. The next safe move is a browser-session foundation slice

Before Shuma can truthfully perform browser-driven challenge or bypass interactions, it needs a first small bridge that:

1. makes dynamic and stealth classes part of the repo-owned runtime readiness contract,
2. makes them part of the worker import contract,
3. introduces an explicit session-strategy seam in the worker,
4. and proves which modes or surface sets should stay request-native versus which should later route through dynamic or stealth execution.

That is the smallest change that creates a real place for later browser behavior without faking the behavior itself.

# Result

`SIM-SCR-FULL-1B` should start with a foundation sub-slice:

1. `SIM-SCR-FULL-1B1` browser-session foundation

and only after that move to:

2. browser-driven challenge or bypass interaction implementation,
3. then receipt-backed proof closure.
