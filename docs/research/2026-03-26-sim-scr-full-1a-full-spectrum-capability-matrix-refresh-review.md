Date: 2026-03-26
Status: Proposed

Related context:

- [`../research/2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md`](../research/2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md)
- [`../research/2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md`](../research/2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md)
- [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](../plans/2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`../plans/2026-03-26-sim-scr-full-spectrum-capability-implementation-plan.md`](../plans/2026-03-26-sim-scr-full-spectrum-capability-implementation-plan.md)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)

# SIM-SCR-FULL-1A Full-Spectrum Capability Matrix Refresh Review

## Question

Under the clarified full-spectrum adversary mandate, which Scrapling capabilities materially increase attack power against Shuma defenses and therefore belong in the active Scrapling lane now?

## Current repo-grounded state

The current Scrapling lane is still intentionally request-native:

1. the worker imports only `FetcherSession`, `Request`, and `Spider`,
2. the runtime cycles only `crawler`, `bulk_scraper`, and `http_agent`,
3. `automated_browser` remains mapped away from Scrapling,
4. and `maze_navigation`, `js_verification_execution`, and `browser_automation_detection` remain outside Scrapling ownership.

Sources:

- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)

## Pinned-runtime findings

The pinned local runtime is broader than the current Shuma worker uses.

Verified directly in the repo-owned `.venv-scrapling` runtime:

1. `DynamicFetcher`, `DynamicSession`, `StealthyFetcher`, and `StealthySession` import successfully.
2. Browser fetches execute successfully against a local bounded HTTP target.
3. `DynamicSession` and `StealthySession` support:
   - persistent browser sessions,
   - `page_action`,
   - `wait_selector`,
   - `network_idle`,
   - `load_dom`,
   - `extra_headers`,
   - optional `proxy`,
   - and optional `real_chrome`.
4. `StealthySession` additionally exposes:
   - `hide_canvas`,
   - `block_webrtc`,
   - `allow_webgl`,
   - `user_data_dir`,
   - and `solve_cloudflare`.
5. Browser-session responses retain cookies, which is enough to prove JS-verification execution truthfully from the worker.

So the relevant question is no longer whether browser or stealth capability exists. It does. The correct question is which of it materially strengthens attacks against Shuma rather than merely widening transport choices.

## Refreshed capability matrix

| Capability family | Pinned runtime evidence | Impact on Shuma defenses | Decision |
|---|---|---|---|
| Request-native sessions, cookies, method mix, traversal, pagination, impersonation, stealthy headers | Already wired and proven | Already increases attack power against public traversal, challenge routing, rate, geo, and request-native abuse surfaces | **Keep adopted** |
| Dynamic browser automation (`DynamicSession` / `DynamicFetcher`) | Present and locally executable | Materially increases attack power against JS verification, maze traversal, and browser-gated challenge flows | **Adopt now** |
| Browser `page_action`, `wait_selector`, `network_idle`, `load_dom`, and session reuse | Present and locally executable | Required to use the browser lane as a real attacker rather than a passive renderer | **Adopt now** |
| Stealth browser automation (`StealthySession` / `StealthyFetcher`) | Present and locally executable | Materially increases attack power against browser-detection and anti-automation surfaces | **Adopt now** |
| Stealth flags (`hide_canvas`, `block_webrtc`, `allow_webgl`, persistent browser profile) | Present in pinned runtime | Directly attacker-relevant for Shuma browser-detection pressure | **Adopt now** |
| `automated_browser` category ownership | Scrapling runtime now credibly supports it | Needed so the non-agent browser spectrum is not left as an artificial gap | **Adopt now** |
| `maze_navigation` surface | Browser interaction required; Scrapling can now do it | Direct hostile traversal pressure against Shuma maze | **Adopt now** |
| `js_verification_execution` surface | Browser execution and cookie state are now available | Direct hostile interaction with JS verification and PoW follow-up | **Adopt now** |
| `browser_automation_detection` surface | Shuma exposes `_checkCDPAutomation` inside JS verification pages | Direct hostile pressure against Shuma browser-detection logic | **Adopt now** |
| Proxy support | Present in pinned runtime | Materially increases attack power against geo, IP, rate, and ban defenses | **Adopt in the immediate follow-on implementation slice, not omit** |
| `solve_cloudflare` | Present in pinned runtime | Does not materially increase power against Shuma-native defenses because Shuma is not presenting Cloudflare Turnstile or Interstitial challenges | **Explicitly exclude for now** |
| `cdp_url` attach-to-existing-browser support | Present in pinned runtime | Changes transport topology, but does not itself add attacker power beyond the local browser or stealth capability being adopted | **Explicitly exclude for now** |
| `real_chrome` launch toggle | Present when a separately installed Chrome exists | Could help on some browser-detection paths, but the repo has not yet shown that bundled Chromium plus stealth is insufficient on current Shuma surfaces | **Explicitly defer unless receipts prove current browser bundle insufficient** |

## Governing reasoning for exclusions

The excluded items are not excluded because they are browser-like, expensive, or inconvenient.

They are excluded only because:

1. `solve_cloudflare` targets a protection family Shuma is not currently using, so adopting it now would not strengthen attacks against Shuma's own defenses,
2. `cdp_url` is a transport-control option rather than distinct attacker power,
3. and `real_chrome` has not yet been shown to improve outcomes on current Shuma browser surfaces relative to the bundled browser or stealth runtime already available in the pinned environment.

These exclusions must stay provisional. If later browser-surface receipts show the adopted browser or stealth path is still too weak, `real_chrome` must be re-opened immediately rather than left as a quiet omission.

## Immediate implementation consequence

`SIM-SCR-FULL-1A` is satisfied only if the repo now treats the active Scrapling implementation target as:

1. request-native personas,
2. plus browser automation personas,
3. plus stealth browser personas,
4. plus truthful `automated_browser` coverage,
5. with proxy support retained as immediate in-scope follow-on work rather than hidden deferral.

That means the next code tranche must:

1. widen the worker and worker-plan contracts,
2. widen lane fulfillment and owned-surface ownership,
3. add receipt-backed browser and stealth proof,
4. and stop describing browser power as merely assigned away.

## Decision

`SIM-SCR-FULL-1A` now closes under the stronger mandate with the following authoritative answer:

1. Shuma must adopt Scrapling browser and stealth capability for the active Scrapling lane now.
2. `automated_browser` is no longer a truthful reason to park non-agent browser power outside Scrapling.
3. Proxy capability remains attacker-relevant and must be implemented next rather than quietly excluded.
4. `solve_cloudflare`, `cdp_url`, and default `real_chrome` usage are explicit exclusions or deferrals, each with the precise reasoning above.
