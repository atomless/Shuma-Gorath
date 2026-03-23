Date: 2026-03-23
Status: Proposed

Related context:

- [`../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`](../plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md)
- [`../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md`](../plans/2026-03-22-canonical-non-human-taxonomy-and-lane-fulfillment-plan.md)
- [`../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md`](../plans/2026-03-22-lane-fulfillment-and-protected-evidence-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)
- [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)
- [`../../src/observability/non_human_coverage.rs`](../../src/observability/non_human_coverage.rs)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)

# Scrapling Non-Human Category Capability Review

## Question

Which canonical non-human traffic categories can Scrapling credibly fulfill on its own for Shuma, and which categories should remain outside Scrapling's near-term ownership?

## Current repo-grounded state

Shuma's current canonical taxonomy is:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `automated_browser`
4. `http_agent`
5. `browser_agent`
6. `agent_on_behalf_of_human`
7. `verified_beneficial_bot`
8. `unknown_non_human`

Source: [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)

The current lane-fulfillment contract maps only `indexing_bot` to Scrapling. `ai_scraper_bot` and `http_agent` are currently assigned to bounded LLM request mode, while the browser-like categories stay assigned to bounded LLM browser mode.

Source: [`../../src/observability/non_human_lane_fulfillment.rs`](../../src/observability/non_human_lane_fulfillment.rs)

The current Scrapling worker is a bounded shared-host crawler built around `FetcherSession` plus `Spider`, with:

1. operator-scoped seeds,
2. host and path fencing,
3. bounded depth, request, byte, and time budgets,
4. signed sim telemetry headers,
5. and no browser automation contract today.

Source: [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

The current repo-owned Scrapling runtime bootstrap is also intentionally narrow. It pins `scrapling[fetchers]==0.4.2` and only verifies request-fetcher readiness through `FetcherSession`, not dynamic browser automation dependencies.

Source: [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)

## External Scrapling capability findings

Scrapling's official HTTP-request surface is broader than Shuma's current worker use:

1. direct GET, POST, PUT, and DELETE support,
2. session management,
3. cookies,
4. proxies,
5. retries,
6. header customization,
7. browser-header and TLS impersonation,
8. and generic request-native scraping patterns such as form submission and pagination handling.

Source: [Scrapling HTTP requests docs](https://scrapling.readthedocs.io/en/latest/fetching/static/)

Scrapling also has browser-fetcher surfaces beyond the current Shuma worker:

1. `DynamicFetcher` for browser automation,
2. `StealthyFetcher` for browser automation with stronger anti-bot bypass,
3. `page_action` hooks using Playwright's page API,
4. session reuse,
5. and support aimed at JavaScript-heavy and protected sites.

Sources:

1. [Scrapling tutorial on robust scraping](https://scrapling.readthedocs.io/en/latest/tutorials/replacing_ai.html)
2. [Scrapling stealthy fetcher docs](https://scrapling.readthedocs.io/en/latest/fetching/stealthy/)

## Category-by-category judgment

### 1. `indexing_bot`

Strong yes.

This is already Scrapling's implemented and truthful role inside Shuma. The current bounded spider, seed inventory, redirect handling, and scope fence are all aligned with indexing-style discovery traffic.

### 2. `ai_scraper_bot`

Yes, as a near-term request-native Scrapling expansion.

Reasoning:

1. broad page retrieval and extraction are native Scrapling use cases,
2. Scrapling's HTTP fetchers already support the request features Shuma would need for bulk retrieval pressure,
3. and this category does not inherently require browser-agent reasoning.

This is an inference from Scrapling's official request-surface docs plus Shuma's current worker shape.

### 3. `http_agent`

Yes, with a bounded definition.

Reasoning:

1. Scrapling can already generate direct HTTP-layer traffic with mutable headers, cookies, proxies, request bodies, and multiple supported verbs,
2. so it can credibly fulfill request-native agent pressure,
3. but Shuma must keep the claim bounded to what Scrapling actually supports rather than imply arbitrary raw-protocol behavior.

This is also an inference from Scrapling's official request-surface docs.

### 4. `automated_browser`

Capability likely exists in Scrapling, but it is not implementation-ready for Shuma today.

Reasoning:

1. Scrapling's official browser-fetcher docs clearly support browser automation via Playwright-facing APIs,
2. but Shuma's current hosted worker, runtime bootstrap, tests, and deployment story only prove request-fetcher operation,
3. and this repo does not yet carry a truthful shared-host runtime contract for Patchright, Playwright, or equivalent browser dependencies inside the Scrapling lane.

So the capability is plausible, but Shuma should not yet reassign `automated_browser` away from the LLM browser lane until the runtime and evidence model are expanded and proven.

### 5. `browser_agent`

No, not as a near-term Scrapling-owned category.

A scripted browser tool and an agentic browser are not the same thing. Scrapling can automate browser actions, but the taxonomy category is about multi-step agentic behavior across flows, not only rendering JavaScript and clicking elements.

### 6. `agent_on_behalf_of_human`

No.

This category depends on delegated intent and human-proxy behavior, not only transport or rendering mechanics. Scrapling alone is not the right ownership boundary for that category.

### 7. `verified_beneficial_bot`

No.

This is an identity or provenance category, not an adversary-sim generation target for Scrapling.

### 8. `unknown_non_human`

No.

This remains an explicit classification gap, not a fulfillment target.

## Decision

Scrapling's truthful near-term ownership inside Shuma should expand from:

1. `indexing_bot`

to:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `http_agent`

but not yet to:

1. `automated_browser`
2. `browser_agent`
3. `agent_on_behalf_of_human`

## Why this boundary is the cleanest one

It aligns:

1. the canonical taxonomy,
2. Scrapling's official request-surface strengths,
3. the current shared-host worker and runtime bootstrap,
4. and the repo's requirement that representativeness and tuning claims stay receipt-backed rather than aspirational.

It also keeps the project from over-correcting too early. If Shuma moved `automated_browser` into Scrapling ownership now, it would be mixing a real request-native worker with an unproven browser-runtime expansion in the same truth contract.

## Required follow-on work

### 1. `SIM-SCR-FIT-1`

Freeze Scrapling's request-native category ownership in the lane-fulfillment and worker-plan contracts:

1. `indexing_bot`
2. `ai_scraper_bot`
3. `http_agent`

### 2. `SIM-SCR-FIT-2`

Implement bounded Scrapling request personas in the worker so the lane can actually generate those three categories distinctly enough for classification:

1. crawler persona,
2. bulk scraper persona,
3. direct HTTP-agent persona.

### 3. `SIM-SCR-COVER-2`

Extend classification and coverage receipts so Shuma can prove Scrapling-generated traffic lands in those categories strongly enough for later diagnosis and tuning inputs.

### 4. `SIM-SCR-BROWSER-1`

Keep browser-like Scrapling as a blocked later evaluation, not active implementation, until:

1. the request-native expansion lands,
2. the shared-host runtime and deploy story are widened truthfully for browser dependencies,
3. and category coverage can prove `automated_browser` without collapsing into a vague "browser-ish" claim.

## Result

The next Scrapling work should not be "make Scrapling do everything."

It should be:

1. expand Scrapling into the request-native categories it can genuinely own,
2. prove those categories with receipts,
3. keep browser-agent and delegated-agent categories on the LLM side,
4. and treat Scrapling browser automation as a separate later gate rather than a silent scope creep inside the current worker.
