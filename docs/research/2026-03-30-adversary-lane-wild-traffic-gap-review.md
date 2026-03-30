Date: 2026-03-30
Status: Proposed

Related context:

- [`2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md`](2026-03-30-adversary-lane-traffic-realism-and-cadence-review.md)
- [`../plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md`](../plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
- [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
- [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
- [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Compare Shuma's current post-`SIM-REALISM-1B` adversary lanes against current field observations of hostile crawler, scraper, and agentic traffic so the next realism backlog is grounded in what attackers actually do, not in the narrower local success criteria of the first cadence-profile tranche.

# Current Implementation Check

## 1. Scrapling now has shaped cadence, but its pressure is still clipped hard

The first realism tranche improved persona shape correctly:

1. Scrapling rotates through `crawler`, `bulk_scraper`, `browser_automation`, `stealth_browser`, and `http_agent` in [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs).
2. Each mode now carries a persona-specific realism profile in [`../../src/admin/adversary_sim_realism_profile.rs`](../../src/admin/adversary_sim_realism_profile.rs).
3. The worker honors those profiles for burst sizing, pause or dwell timing, and bounded identity behavior in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

But the absolute per-tick ceiling is still:

1. `max_requests = 8`
2. `max_depth = 2`
3. `max_bytes = 262_144`
4. `max_ms = 2000`

from [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs), and the worker clips planned activity budgets against those ceilings in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

That means the current lane is more realistic in shape than before, but still materially conservative in pressure.

## 2. Identity realism is still shallow and often synthetic

The current worker plan exposes only one optional `request_proxy_url` and one optional `browser_proxy_url` in [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs).

Inside the worker:

1. identity rotation only happens when a proxy is configured,
2. rotation is modeled as session-handle churn such as `request-session-N`,
3. browser sessions are modeled as one stable local session handle,
4. and there is no pool contract for residential versus mobile versus datacenter origin classes, no geo affinity, and no session stickiness or reuse policy beyond the local worker process.

See [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

So the current realism receipts can prove local session rotation, but they cannot yet prove realistic network-identity rotation.

## 3. Header, locale, and transport posture is still too uniform

Request-native Scrapling currently pins:

1. `impersonate = "chrome"`,
2. `stealthy_headers = true`,
3. a single locale family around `accept-language: en-GB,en;q=0.8`,
4. and one general browser-like request posture

in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

Browser personas similarly pin locale to `en-GB` in that same worker file.

This is directionally better than custom Shuma-branded headers, but it is still far from the field patterns where attackers pair:

1. identity geography,
2. Accept-Language,
3. user-agent family,
4. resource headers,
5. and transport fingerprints

into coherent attack envelopes rather than one static local default.

## 4. Concurrency and browser background activity are still under-modeled

The crawler is explicitly single-threaded with:

1. `concurrent_requests = 1`
2. `concurrent_requests_per_domain = 1`

in [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py).

The request-native personas also execute sequentially through one local paced session wrapper in that same file. Browser receipts currently focus on top-level action count and dwell, but do not yet project browser secondary traffic such as subresource fetches or background XHR activity into realism receipts or recent-run truth.

## 5. Agentic Traffic is still materially short of field realism

The current agentic lane remains under-representative:

1. `browser_mode` still returns `browser_mode_not_supported` in [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py),
2. `request_mode` actions are still executed sequentially in [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py),
3. degraded fallback still collapses to `GET /` plus public hint paths in [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py),
4. and there is no current identity-envelope or transport-envelope realism for the agentic lane either.

# External Observations

## 1. Hostile scraping is often persistent, 24/7, mostly residential, and often non-JavaScript

The attached Glade article reports:

1. `6.8M` requests over `55` days,
2. requests arriving `24/7`,
3. the overwhelming majority from residential or mobile networks,
4. mostly without JavaScript execution,
5. and several hundred-thousand requests per day before a light proof-of-work gate collapsed the flow.

Sources:

1. local attached PDF: `/Users/jamestindall/Downloads/The bot situation on the internet is actually worse than you could imagine. Here's why: | Glade Art .pdf`
2. [Glade Art article](https://gladeart.com/blog/the-bot-situation-on-the-internet-is-actually-worse-than-you-could-imagine-heres-why)

## 2. Real distributed scrapers are often low-per-IP but massive in aggregate

DataDome documents a scraper that:

1. made `1.1M` search requests over one week,
2. used more than `45k` French residential IPs,
3. usually made only `1-10` requests per day per IP,
4. randomized modern user-agents,
5. kept `Accept-Language` coherent with the target site,
6. and used TLS fingerprints that were not trivially attributable to common HTTP client libraries.

Source:

1. [DataDome: Anatomy of a Distributed Scraping Attack](https://datadome.co/threat-research/anatomy-of-a-distributed-scraping-attack/)

## 3. Modern evasive bots explicitly use low-and-slow pacing and identity churn

Imperva's 2025 report describes evasive bots as using:

1. residential proxies,
2. identity changes,
3. human-behavior mimicry,
4. delayed requests,
5. and a `low and slow` pattern specifically to reduce noise and avoid detection.

Source:

1. [Imperva 2025 Bad Bot Report](https://www.imperva.com/resources/wp-content/uploads/sites/6/reports/2025-Bad-Bot-Report.pdf)

## 4. Agentic abuse has a different shape from broad scraping

DataDome reports that spoofed AI-agent traffic can look like:

1. a narrow focused session rather than a sitewide crawl,
2. about `22 requests per second`,
3. `109` product pages in `5` seconds,
4. and residential proxies paired with spoofed agent identity claims.

Source:

1. [DataDome: The AI Agent Identity Crisis](https://datadome.co/threat-research/ai-agent-identity-crisis/)

## 5. Legitimate agent identity is moving toward cryptographic proof, not self-declared user-agent strings

Cloudflare's current Web Bot Auth guidance emphasizes that trusted agent traffic should carry:

1. a stable identifier,
2. signed request metadata,
3. nonce-based replay protection,
4. and explicit browsing versus purchasing tags.

That matters for Shuma because the offensive lane should not casually claim trusted identity or inherit properties that in the wild now require cryptographic proof.

Source:

1. [Cloudflare: Securing agentic commerce](https://blog.cloudflare.com/secure-agentic-commerce/)

## 6. AI crawling pressure is enormous, and robots.txt is not a sufficient control

Cloudflare's current Radar reporting shows:

1. AI crawling at very high volume,
2. extreme crawl-to-refer ratios for some AI platforms,
3. and AI crawlers among the most frequently fully disallowed user agents in `robots.txt`.

Separately, the large-scale arXiv study on robots compliance found that some AI search crawlers rarely check `robots.txt` at all.

Sources:

1. [Cloudflare Radar 2025 Year in Review](https://blog.cloudflare.com/radar-2025-year-in-review/)
2. [Scrapers selectively respect robots.txt directives](https://arxiv.org/abs/2505.21733)

# Findings

## 1. `SIM-REALISM-1B` landed shape realism, not full attacker-pressure realism

The first Scrapling realism tranche was worth landing, but it should now be described precisely:

1. it improved burst and dwell shape,
2. it improved receipt-backed observability,
3. but it did not yet make Scrapling representative of the current hostile web on pressure, identity, or transport realism.

## 2. The next realism gap is not route knowledge; it is envelope realism

The current discovery boundary remains correct. The remaining gap is the envelope in which traffic is emitted:

1. pressure,
2. concurrency,
3. identity provenance,
4. geo and locale coherence,
5. transport and browser fingerprint coherence,
6. background browser noise,
7. and long-horizon recurrence.

## 3. Identity realism is the biggest missing attacker trait

The strongest mismatch between current code and field observations is not that Scrapling is too polite in one narrow pacing dimension. It is that the lane still lacks a truthful model of:

1. residential or mobile identity pools,
2. low-per-identity attack shaping,
3. session reuse and churn,
4. geo-targeting,
5. and honest receipts when that infrastructure is absent.

## 4. The agentic lane still needs both the currently planned work and a later field-grounded follow-on

`SIM-REALISM-1C` and `SIM-REALISM-1D` are still the right immediate next steps. But finishing them will still not close the whole realism gap unless the later chain also lands:

1. focused but burst-capable request-mode envelopes,
2. real browser-mode execution,
3. realistic identity envelopes,
4. and realistic transport, background traffic, and recurrence behavior.

## 5. Representative mixed-attacker proof should not reopen after `SIM-REALISM-1D` alone

The backlog and roadmap should stop implying that `SIM-REALISM-1A..1D` alone are enough for representative attacker-pressure claims. A second field-grounded realism chain is still required after `SIM-REALISM-1D`.

# Recommended Follow-On Chain

After `SIM-REALISM-1C` and `SIM-REALISM-1D`, Shuma should explicitly execute a second realism chain:

1. `SIM-REALISM-2A` pressure-envelope and concurrency realism
2. `SIM-REALISM-2B` proxy-pool and identity-envelope realism
3. `SIM-REALISM-2C` header, locale, and transport-envelope realism
4. `SIM-REALISM-2D` browser secondary-traffic and background-request realism
5. `SIM-REALISM-2E` long-horizon dormancy, recurrence, and re-entry realism

# Decisions

1. Keep the current immediate execution order as `SIM-REALISM-1C` then `SIM-REALISM-1D`.
2. Do not reopen representative mixed-attacker or tuning-quality claims after `SIM-REALISM-1D` alone.
3. Make the next realism chain explicit in backlog and sequencing truth now, before more completion claims flatten `SIM-REALISM-1B` into "problem solved".
