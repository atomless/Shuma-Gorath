# Telemetry-As-Map Adversary Surface Discovery Synthesis

Date: 2026-03-20
Status: Active synthesis for shared-host and emergent-lane planning

Related context:

- [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](./2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`2026-02-25-llm-adversarial-testing-research-synthesis.md`](./2026-02-25-llm-adversarial-testing-research-synthesis.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)

## Purpose

Reassess Shuma's current shared-host discovery plan in light of the intended mature adversary loop, and answer one specific question:

Should Shuma precompute a rich public-surface catalog before emergent adversary lanes run, or should the reachable surface emerge from adversary telemetry itself?

## Research Question

For a realistic crawler or agent adversary harness, what should be considered necessary:

1. a full precomputed public-surface catalog,
2. a minimal seed and scope contract,
3. or a telemetry-emergent reachable-surface map?

## External Findings

### 1. Real crawlers primarily discover through crawlable links

Google's crawl guidance makes links the core discovery mechanism. That strongly suggests the most realistic adversary harness should behave like a bounded crawler starting from a small entry set and following reachable links, not like a privileged actor handed an authoritative site map.

Relevant source:

- [Google crawlable links](https://developers.google.com/search/docs/crawling-indexing/links-crawlable)

Implications:

1. Link-following should be the default discovery behavior.
2. A rich precomputed catalog is not the most realistic starting model.
3. The relevant map is what the crawler actually reaches.

### 2. Sitemaps are hints, not the primary discovery truth

Google's sitemap documentation explicitly says a sitemap submission is merely a hint and does not guarantee crawling or use for discovery. That reinforces that Shuma should treat `sitemap.xml` as optional seed help, not as the adversary's core knowledge model.

Relevant source:

- [Build and submit a sitemap](https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap)

Implications:

1. `robots.txt` and `sitemap.xml` can be useful cheap inputs.
2. They should not become the authoritative public-surface model.
3. A precomputed catalog derived from them risks over-weighting hints into false ground truth.

### 3. Real crawler frameworks start from small seed sets and scope fences

Scrapy's model is simple and revealing: `start_urls` define where the spider begins, while `allowed_domains` constrains scope. That is a closer match to the realistic adversary harness Shuma wants than a heavyweight prebuilt catalog pipeline.

Relevant source:

- [Scrapy spiders](https://docs.scrapy.org/en/latest/topics/spiders.html)

Implications:

1. The essential ingredients are bounded start points plus scope rules.
2. The crawler's discovered reachable surface should emerge from traversal.
3. A large catalog builder is not needed to begin realistic crawling.

## Synthesis

Taken together, these sources support a much stricter and simpler planning rule:

1. Shuma should keep a fail-closed scope fence.
2. Shuma should require only minimal operator-defined seeds:
   - homepage or primary public URL,
   - optional `robots.txt`,
   - optional small manual seed list.
3. The emergent adversary lane should discover the reachable surface by traversal.
4. The telemetry produced by that traversal should become the only map Shuma cares about for the adaptive loop.

## Why This Is Better

### 1. Higher fidelity

The harness then reflects what a bounded adversary actually found, not what Shuma guessed it might find.

### 2. Less wasted machinery

This avoids building and maintaining a separate discovery-and-catalog subsystem that may add little value and distort behavior.

### 3. Better deterministic memory later

If deterministic replay memory is promoted from observed telemetry traces, the deterministic oracle preserves real discovered exploit paths rather than synthetic catalog entries.

## Recommended Planning Consequence

Shuma should replace the old "shared-host discovery first" concept with a narrower model:

1. `scope_fence`
2. `minimal_seed_contract`
3. `observed_reachable_surface` emerging from telemetry

The plan should explicitly reject the idea that Shuma needs a rich precomputed public-surface catalog before emergent lanes can be realistic.

If export or curation tooling is later needed for deterministic replay promotion, it should be derived from observed traversal telemetry, not from a separately maintained discovery product.
