Date: 2026-03-25
Status: Completed

# SIM-SCR-RN-1 Request-Native Fidelity Review

## Root cause

The current request-native Scrapling lane is closer to attacker-like than the earlier matrix phrasing implied, but it still has one major fidelity break:

1. upstream `FetcherSession` already defaults to `impersonate='chrome'` and `stealthy_headers=True`,
2. Shuma still overrides the visible `User-Agent` on every request with `ShumaScraplingWorker/1.0 ...`,
3. and Shuma relies on those upstream request-native defaults implicitly rather than locking them into an explicit local contract.

So the honest gap is:

- not “Shuma never uses request-native impersonation,”
- but “Shuma still brands the attacker traffic as internal and does not explicitly pin the request-native session contract it depends on.”

## Evidence

Upstream installed signature in the repo-local Scrapling runtime:

- `FetcherSession(... impersonate='chrome', stealthy_headers=True, ...)`

Current Shuma implementation:

- [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
  - `_signed_headers(...)` injects `user-agent: ShumaScraplingWorker/1.0 ...`
  - the worker uses `FetcherSession`, but does not make its attacker-fidelity session contract explicit

Live local request evidence from the current worker:

- browser-like headers such as `sec-ch-ua`, `sec-fetch-*`, `accept-language`, and Google referer are already present
- but the visible `user-agent` is still `ShumaScraplingWorker/1.0 lane=...`

## Consequence

The next request-native fidelity tranche should do two things and no more:

1. stop overriding the outward-facing `User-Agent` with an internal worker marker,
2. explicitly pin the request-native session contract Shuma expects:
   - `impersonate='chrome'`
   - `stealthy_headers=True`
   - bounded timeout/retry/follow-redirect behavior

That keeps the lane attacker-faithful without blurring into browser-class capability or taxonomy changes.
