Date: 2026-03-25
Status: Completed

# SIM-SCR-CAP-1 Upstream Capability Matrix Review

## Question

What does upstream Scrapling actually expose today, what does Shuma currently use, and which capabilities should be adopted, assigned elsewhere, or explicitly excluded for Shuma's current Scrapling-owned surfaces?

## Current Shuma boundary

Shuma's current Scrapling ownership is still explicitly request-native:

- taxonomy ownership: `indexing_bot`, `ai_scraper_bot`, `http_agent` in [`src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- owned defense surfaces: `public_path_traversal`, `challenge_routing`, `rate_pressure`, `geo_ip_policy`, `not_a_bot_submit`, `puzzle_submit_or_escalation`, `pow_verify_abuse`, and `tarpit_progress_abuse` in [`src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- current runtime wiring: `FetcherSession`, `Request`, and `Spider` only in [`scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- current runtime bootstrap: readiness proves only `FetcherSession` in [`scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)

So the matrix must preserve one distinction:

1. request-native Scrapling-owned surfaces,
2. browser-class capability that belongs to a different lane or later ownership decision,
3. explicit exclusions for safety, cost, or scope reasons.

## Upstream capability matrix

Official upstream references used here:

- [HTTP requests - Scrapling](https://scrapling.readthedocs.io/en/latest/fetching/static/)
- [Fetching dynamic websites - Scrapling](https://scrapling.readthedocs.io/en/latest/fetching/dynamic.html)
- [Dynamic websites with hard protections - Scrapling](https://scrapling.readthedocs.io/en/latest/fetching/stealthy/)
- [Scrapling GitHub README](https://github.com/D4Vinci/Scrapling)

| Capability family | Upstream evidence | Current Shuma state | Matrix outcome |
| --- | --- | --- | --- |
| Request-native browser impersonation and realistic header shaping | Static fetcher docs show `impersonate='chrome'`, `stealthy_headers=True`, and proxy-capable GET/POST/PUT flows | Shuma uses `FetcherSession`, but does not currently enable impersonation or `stealthy_headers`; it also still emits `ShumaScraplingWorker/1.0 ...` as the visible `User-Agent` | **Adopt now** for the request-native lane |
| Request-native session and cookie continuity | Upstream static fetchers support session reuse and cookie persistence | Already used in the bulk-scraper and http-agent personas | **Keep adopted** |
| Request-native crawl and traversal mechanics | Upstream `Spider` + `Request` architecture supports bounded crawl/traversal | Already used in the crawler persona | **Keep adopted** |
| Request-native proxy support | Static fetcher docs show per-request proxy support | Shuma does not currently route Scrapling through proxies | **Explicitly exclude for now** on shared-host local mainline; revisit only with a concrete distributed-origin requirement |
| Dynamic browser automation (`DynamicFetcher` / `DynamicSession`) | Official docs expose real browser sessions, `page_action`, `network_idle`, `real_chrome`, shared browser sessions, and tab pools | Shuma does not currently wire any of this | **Assign to separate browser-class lane**, not the current request-native Scrapling contract |
| Stealth browser automation (`StealthyFetcher` / `StealthySession`) | Official docs expose `solve_cloudflare`, `block_webrtc`, `hide_canvas`, `allow_webgl`, and stealth page actions | Shuma does not currently wire any of this | **Assign to separate browser-class lane**, not the current request-native Scrapling contract |
| Cloudflare / Turnstile-style automatic challenge solving | Upstream stealth docs explicitly advertise automatic Cloudflare Turnstile and interstitial solving | Shuma has no truthful proof of this in its own attacker harness | **Do not claim; assign away for now** until Shuma owns an equivalent browser-class surface and proves it |
| Camoufox / alternative stealth browser engine shaping | Stealth docs describe optional Camoufox-backed persistent contexts and stealth launch options | Shuma does not use this | **Explicitly exclude for now**; too far beyond the current shared-host request-native remit |

## What this means

The matrix rules out two bad conclusions:

1. it is **wrong** to say “the current request-native lane already uses Scrapling fully,”
2. it is also **wrong** to say “therefore every upstream browser or stealth feature must immediately be bolted into the current Scrapling-owned request-native lane.”

The correct reading is:

- Shuma's current truthful gap is **not** “generic more Scrapling.”
- The immediate gap is **request-native attacker fidelity** inside the lane Shuma already says Scrapling owns.
- Browser or stealth Scrapling is real upstream power, but today it belongs to a separate browser-class ownership question unless Shuma explicitly re-ratifies its owned surfaces.

## Immediate consequence for implementation order

The next implementation tranche should be a request-native fidelity uplift, not a vague browser leap:

1. stop advertising the worker with an obviously internal `User-Agent`,
2. adopt request-native browser impersonation and realistic header shaping,
3. preserve signed sim-tag telemetry without making the attack traffic cosmetically non-attacker-like,
4. and extend focused worker proof so those attacker-faithful request-native signals are verified.

That tranche is the remaining honest prerequisite before the later fuller attacker runtime should reopen.

## Omission ledger outcome

As of this review:

- **adopt now:** request-native impersonation and realistic header shaping
- **keep adopted:** session persistence, cookies, crawl/traversal, hostile request-native submits
- **assign elsewhere:** `DynamicFetcher`, `StealthyFetcher`, browser automation, page actions, Cloudflare-style solving
- **explicitly exclude for now:** proxy routing on shared-host local mainline, Camoufox-specific runtime shaping, and any cloud-browser style capability not yet mapped to a ratified Shuma-owned surface
