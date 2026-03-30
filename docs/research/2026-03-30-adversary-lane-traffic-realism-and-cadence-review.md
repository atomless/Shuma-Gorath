Date: 2026-03-30
Status: Proposed

Related context:

- [`../plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md`](../plans/2026-03-30-adversary-lane-traffic-realism-and-cadence-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md`](../plans/2026-03-25-sim-llm-1c-runtime-decomposition-plan.md)
- [`../research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md`](../research/2026-03-24-game-loop-sequencing-require-attacker-faithful-scrapling-review.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`../testing.md`](../testing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Freeze the next realism contract for Shuma's adversary lanes so Scrapling and Agentic Traffic apply characteristic real-world crawl or scrape pressure instead of one overly neat synthetic cadence. The goal is not "more aggressive by default." The goal is representative attacker behavior: not unrealistically polite, not theatrically reckless, and not choreographed for Shuma's convenience.

# Current Ground Truth

## 1. Both lanes are heartbeat-dispatched bounded bursts, not continuous streams

Shared-host adversary-sim currently beats every second and dispatches at most one pending worker tick at a time:

1. [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
2. [`../../src/admin/adversary_sim_state.rs`](../../src/admin/adversary_sim_state.rs)
3. [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs)
4. [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)

That bounded-beat architecture is acceptable. The realism gap is inside each tick.

## 2. Scrapling starts from the right knowledge boundary, but its pacing is still too uniform

The Scrapling lane currently does several things correctly:

1. it is fenced by an explicit shared-host scope descriptor,
2. it starts from a primary public start URL plus accepted public hint documents,
3. it does not receive an internal route catalog,
4. and its mode cycle already distinguishes `crawler`, `bulk_scraper`, `browser_automation`, `stealth_browser`, and `http_agent`.

References:

1. [`../../scripts/tests/shared_host_scope.py`](../../scripts/tests/shared_host_scope.py)
2. [`../../scripts/tests/shared_host_seed_inventory.py`](../../scripts/tests/shared_host_seed_inventory.py)
3. [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
4. [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)

But the current tick budgets are still very small and very flat:

1. `max_requests = 8`
2. `max_depth = 2`
3. `max_bytes = 262_144`
4. `max_ms = 2000`

Reference:

1. [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)

Inside the worker, the crawler is single-threaded with no download delay, and the direct-request personas loop sequentially until the request, byte, or time budget is exhausted:

1. [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

That means Scrapling is discoverability-faithful but still too close to "one bounded generic burst" rather than several attacker-characteristic shapes.

## 3. Agentic Traffic is still materially under-representative

The Agentic lane correctly stays black-box and Shuma-blind:

1. it alternates `browser_mode` and `request_mode`,
2. it is only given the normalized host root URL, public hint paths, and the bounded fulfillment plan,
3. and the contract explicitly forbids internal Shuma knowledge.

References:

1. [`../../src/admin/adversary_sim_llm_lane.rs`](../../src/admin/adversary_sim_llm_lane.rs)
2. [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
3. [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)

But it is not yet representative enough to support serious mixed-attacker claims:

1. `browser_mode` still returns `browser_mode_not_supported` instead of emitting real traffic,
2. `request_mode` is budgeted as `24` actions over `120` seconds but executes validated actions sequentially with no explicit burst or dwell model,
3. and degraded fallback can collapse to `GET /` plus public hint paths.

References:

1. [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)
2. [`../../scripts/tests/adversarial_runner/llm_fulfillment.py`](../../scripts/tests/adversarial_runner/llm_fulfillment.py)
3. [`../../scripts/tests/adversarial_container/worker.py`](../../scripts/tests/adversarial_container/worker.py)

# External Observations

## 1. Real scraper traffic is often distributed, low-per-identity, and relentless

The Glade write-up reports `6.8M` requests over `55` days against a tarpit-backed site, with requests arriving `24/7`, mostly from residential or mobile IP space, and mostly without JavaScript execution. It also reports that enabling a light proof-of-work challenge collapsed that traffic almost immediately.

Source:

1. [Glade Art: The bot situation on the internet is actually worse than you could imagine. Here's why](https://gladeart.com/blog/the-bot-situation-on-the-internet-is-actually-worse-than-you-could-imagine-heres-why)

Note:

1. as of 2026-03-30, the live Glade page is fronted by an Anubis JavaScript or proof-of-work challenge, so this review also relied on a locally captured copy of the article during research.

## 2. Large-scale scraping is frequently low-and-slow per IP rather than high-RPS from one identity

DataDome documents a week-long distributed scraper that made `1.1M` search requests using more than `45k` residential IPs, often at only `1-10` requests per day per IP, while still behaving coherently enough to look modern and legitimate at the header level.

Source:

1. [DataDome: Anatomy of a Distributed Scraping Attack](https://datadome.co/threat-research/anatomy-of-a-distributed-scraping-attack/)

## 3. Modern hostile automation hides in residential identity, browser impersonation, and low-and-slow pacing

Imperva's 2025 bot report describes the prevailing evasion pattern as:

1. residential proxy use,
2. browser impersonation,
3. identity churn,
4. human-mimicry signals,
5. and delayed or low-and-slow pacing chosen to avoid detection.

Source:

1. [Imperva 2025 Bad Bot Report](https://www.imperva.com/resources/wp-content/uploads/sites/6/reports/2025-Bad-Bot-Report.pdf)

## 4. Agentic abuse has a different shape: shorter, more coherent, and sometimes sharply bursty

DataDome reports both legitimate and spoofed AI-agent behavior that is materially different from site-wide crawler pressure:

1. AI-agent-identifying traffic surged rapidly around product launches,
2. spoofed "agentic" traffic reached about `22 req/s`,
3. and one observed session fetched `109` product pages in `5` seconds.

Sources:

1. [DataDome: AI Agents at the Gate](https://datadome.co/threat-research/ai-agents-llm-crawlers/)
2. [DataDome: AI Agent Identity Crisis](https://datadome.co/threat-research/ai-agent-identity-crisis/)

## 5. AI crawler pressure is already massive at internet scale

Cloudflare reports more than `50B` AI crawler requests per day across its network and separately highlights the large crawl-to-click imbalance from training-oriented crawlers.

Sources:

1. [Cloudflare: AI Labyrinth](https://blog.cloudflare.com/ai-labyrinth/)
2. [Cloudflare: The Crawl-to-Click Gap](https://blog.cloudflare.com/crawlers-click-ai-bots-training/)

# Findings

## 1. Representativeness is multi-profile, not one generic "bot rate"

The research does not support a single average rate target. It supports at least two distinct families Shuma must model:

1. distributed scraper pressure that is low per identity but persistent and wide,
2. and shorter goal-oriented agentic sessions that are coherent, response-aware, and sometimes sharply bursty.

## 2. Scrapling is directionally correct on knowledge boundary, but under-modeled on cadence and identity

The current Scrapling lane already respects the correct epistemic boundary: discovered public host knowledge only. The next gap is not more route hints. It is realistic behavior shape:

1. burst structure,
2. inter-request jitter,
3. browser dwell,
4. retry and pause rules,
5. and identity rotation patterns when proxy support exists.

## 3. Agentic Traffic is not yet representative enough for mixed-attacker or tuning claims

The current Agentic lane is still missing its browser half entirely, and its request half can degrade to an unrealistically narrow sequence. Until that changes, lane presence alone is not enough to claim representative agentic pressure.

## 4. Realism should be modeled as an executable profile contract

The right contract is not "increase the default request limit." It is a profile library that fixes:

1. cadence and burst size,
2. dwell and pause behavior,
3. identity rotation cadence,
4. JavaScript or browser propensity,
5. retry ceilings and hard-failure handling,
6. and session shape or focus area.

## 5. Proof must come from runtime receipts, not declared labels

Shuma should not claim a persona is realistic because the plan says so. It should record the actual emitted behavior needed to prove it:

1. requests attempted,
2. burst grouping and dwell timing,
3. identities or session handles used,
4. top-level navigation or interaction counts where relevant,
5. and terminal stop reasons.

# Proposed Target Profiles

## Scrapling

### `crawler` / `indexing_bot`

1. `6-14` requests per 30-second run,
2. `300-1200ms` jitter between requests,
3. depth `1-2`,
4. mostly breadth-first public traversal,
5. and identity rotation every `2-4` requests when a proxy pool exists.

### `bulk_scraper` / `ai_scraper_bot`

1. `18-45` requests per 30-second run,
2. micro-bursts of `2-5` requests,
3. `200-800ms` intra-burst gaps,
4. `1-3s` pauses between bursts,
5. mostly non-JavaScript extraction over discovered listing or detail pages,
6. and identity rotation every `1-3` requests when a proxy pool exists.

### `browser_automation` / `automated_browser`

1. `4-9` top-level navigations or interactions per 30-second run,
2. `800-2500ms` dwell between interactions,
3. one browser identity per session,
4. response-aware traversal from discovered entrypoints only,
5. and no bulk sitewide crawl behavior inside the session.

### `stealth_browser` / `automated_browser`

1. similar to `browser_automation`,
2. but with fewer navigations,
3. slightly longer dwell,
4. and stronger anti-detection posture rather than more raw volume.

### `http_agent` / `http_agent`

1. `10-24` direct requests per 30-second run,
2. bursts of `2-4` requests,
3. `100-500ms` intra-burst gaps,
4. `500-2000ms` pauses between bursts,
5. and identity rotation per burst when a proxy pool exists.

## Agentic Traffic

### `browser_mode` / `browser_agent` and `agent_on_behalf_of_human`

1. `4-8` top-level actions in 30 seconds,
2. `2-7s` dwell between actions,
3. one stable session identity,
4. narrow goal-driven traversal,
5. and no broad crawler-style discovery.

### `request_mode` / `http_agent` and `ai_scraper_bot`

1. `8-20` requests in 30 seconds,
2. short micro-bursts up to roughly `3-8 req/s`,
3. pauses of `1-4s`,
4. a small focused set of related pages rather than broad site traversal,
5. and response-aware continuation.

# Decisions

1. Shuma should treat traffic realism as a profile contract per lane and per mode, not one shared adversary-sim budget.
2. Scrapling should keep its current discoverable-host-only knowledge model; realism must not be achieved by route choreography or hidden site hints.
3. Agentic Traffic should keep its current Shuma-blind black-box boundary; realism must not come from leaking internal host knowledge or control-plane context.
4. Simulator labels and run metadata must remain observer-only and must not enter Shuma's defence truth or tuning truth.
5. Mixed-attacker proof, tuning-quality claims, and later stance-relaxation claims must not describe the current lanes as fully representative while the realism tranche remains open.

# Required Follow-On Work

1. `SIM-REALISM-1A` to freeze an executable realism profile contract and proof surface for both lanes.
2. `SIM-REALISM-1B` to implement Scrapling profile-driven pacing, dwell, and identity behavior while preserving the current public-discovery boundary.
3. `SIM-REALISM-1C` to implement Agentic request-mode profile-driven pacing and focused burst or pause behavior.
4. `SIM-REALISM-1D` to replace `browser_mode_not_supported` with a real bounded browser session actor and prove its emitted behavior.
5. sequencing updates so `RSI-GAME-HO-2` and later tuning claims depend on representative mixed-attacker behavior rather than mere lane presence.

# Result

The next adversary-sim maturity step is not "make the lanes louder." It is:

1. keep the current clean knowledge boundaries,
2. make the emitted crawl or scrape behavior characteristic of the attacker types those lanes claim to represent,
3. prove that behavior with runtime receipts,
4. and only then use mixed-attacker loop outcomes as representative tuning evidence.
