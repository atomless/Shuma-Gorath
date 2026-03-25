Date: 2026-03-25
Status: Completed

Related plan:

- [`../plans/2026-03-25-sim-scr-rn-1-request-native-fidelity-plan.md`](../plans/2026-03-25-sim-scr-rn-1-request-native-fidelity-plan.md)

# SIM-SCR-RN-1 Post-Implementation Review

## What landed

`SIM-SCR-RN-1` now closes the remaining request-native Scrapling fidelity gap without widening the lane into browser-runtime scope.

The worker now carries one explicit local request-native session contract in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py):

1. `impersonate="chrome"`,
2. `stealthy_headers=True`,
3. bounded timeout, retry, and redirect behavior,
4. and persona-appropriate `Accept` headers.

That helper is now reused by the crawler, bulk-scraper, and http-agent personas. At the same time, the worker no longer overwrites outward-facing traffic with the bespoke internal `ShumaScraplingWorker/...` `User-Agent`.

## Verification

- `make test-adversary-sim-scrapling-worker`
- `git diff --check`

## Outcome Against Plan

The tranche met the plan:

1. the new tests proved the real root cause first,
2. the worker now pins the request-native session contract explicitly rather than relying on upstream defaults alone,
3. live request emission no longer brands the attacker as an internal worker,
4. sim-tag telemetry headers remain intact,
5. and the fix stayed local to the request-native worker path.

## Remaining Gap

This slice does not reopen browser or stealth Scrapling ownership.

Those broader capabilities remain explicitly downstream of a later ownership decision in `SIM-SCR-CHALLENGE-2C` or `SIM-SCR-BROWSER-1`, not silently folded into the current request-native lane.

## Follow-On

The next backend mainline is to intentionally reopen `SIM-LLM-1C` over the now-landed black-box, episode-harness, and request-native attacker-fidelity contracts.
