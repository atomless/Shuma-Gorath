Date: 2026-03-22
Status: Proposed

Related context:

- [`../plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md`](../plans/2026-03-22-autonomous-tuning-safety-gates-implementation-plan.md)
- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../research/2026-03-20-adversary-evolution-loop-role-synthesis.md`](../research/2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md`](../research/2026-03-22-autonomous-tuning-safety-and-sim-representativeness-review.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Freeze the sequencing rule that Shuma must first become good at identifying and categorizing the non-human traffic it actually encounters before it can truthfully judge whether the Scrapling and frontier or LLM adversary lanes are representative enough to drive autonomous tuning.

# Findings

## 1. The taxonomy should be grounded in observed traffic, not lane assumptions

The representativeness contract should not start by asking whether Scrapling or the frontier or LLM lane "looks realistic enough" in the abstract.

It should start by asking:

1. what non-human traffic categories the site actually sees,
2. which of those categories are desired, tolerated, restricted, or unwanted,
3. and how confidently Shuma can classify requests into those categories.

That aligns with the repo-wide rule that telemetry is the map.

## 2. External ecosystems already distinguish multiple meaningful non-human categories

Google explicitly separates:

1. common crawlers,
2. special-case crawlers,
3. user-triggered fetchers.

Cloudflare also distinguishes verified bot categories and maintains a separate AI crawler reference surface.

Sources:

1. [Google crawler overview](https://developers.google.com/search/docs/crawling-indexing/overview-google-crawlers)
2. [Google user-triggered fetchers](https://developers.google.com/search/docs/crawling-indexing/google-user-triggered-fetchers)
3. [Cloudflare verified bot categories](https://developers.cloudflare.com/bots/concepts/bot/verified-bots/categories/)
4. [Cloudflare AI crawler reference](https://developers.cloudflare.com/ai-crawl-control/reference/bots/)

Implication for Shuma:

1. a single generic "bot" bucket is not enough for tuning,
2. and the taxonomy should explicitly make room for crawler, fetcher, AI crawler or scraper, automated browser, and agent-on-behalf behavior.

## 3. The site-local taxonomy must still be Shuma's own, not a mirror of vendor labels

Vendor taxonomies are useful seeds, but Shuma needs a site-local model driven by:

1. observed request behavior,
2. verified identity or provenance when available,
3. operator intent,
4. and the defensive behaviors that tuning can actually change.

So the question is not "does this request match Cloudflare or Google's naming exactly?"

The question is:

1. what category is operationally relevant for this site,
2. what confidence does Shuma have in that categorization,
3. and what evidence supports it?

## 4. Classification confidence must precede lane representativeness

Before Shuma can judge whether Scrapling and frontier or LLM traffic are representative, it needs bounded receipts showing that its own classification layer is trustworthy enough to distinguish categories in the first place.

Otherwise the system risks circular reasoning:

1. lane traffic is judged against categories,
2. but the categories themselves are still unproven,
3. so representativeness claims become anecdotal rather than machine-checkable.

## 5. Scrapling and frontier or LLM lanes should jointly own representativeness

The right contract is not that each lane must independently represent every non-human category.

The better contract is:

1. Shuma defines the protected non-human taxonomy from observed traffic,
2. Shuma records classification confidence and coverage receipts against that taxonomy,
3. Scrapling and frontier or LLM lanes are jointly evaluated against that taxonomy,
4. and the closed tuning loop can only use categories for which protected evidence exists.

That means:

1. Scrapling is likely the first strong source for crawl, scrape, and lower-complexity automated browsing pressure,
2. frontier or LLM lanes likely deepen multi-step agentic and higher-capability browser behaviors,
3. and together they should aim to cover the category set the operator cares about.

## 6. Autonomous tuning should optimize only over categories that are both classifiable and represented

Google's canary guidance is clear that synthetic or unrepresentative traffic is unsafe as a sole basis for rollout judgment.

Source:

1. [Google SRE Workbook: Canarying Releases](https://sre.google/workbook/canarying-releases/)

For Shuma that means a category is tuning-eligible only when:

1. Shuma can identify it with bounded confidence,
2. the operator has declared the desired stance for it,
3. and the protected adversary evidence set actually represents it.

# Decisions

1. The next closed-loop prerequisite is not only protected evidence. It is observed-traffic taxonomy plus bounded classification confidence.
2. `SIM-COVER-1` must be redefined so lane representativeness is evaluated against Shuma's observed traffic taxonomy, not against lane-owned assumptions alone.
3. Scrapling and frontier or LLM lanes jointly own the representativeness contract.
4. Autonomous tuning remains blocked until category identification, category confidence, and category representation are all machine-readable.

# Required Follow-On Work

1. `TRAFFIC-TAX-1` to define the observed non-human traffic taxonomy and materialize it into machine-first contracts.
2. `TRAFFIC-TAX-2` to add category-confidence and evidence receipts for the observed classification layer.
3. `SIM-COVER-1` to measure Scrapling and frontier or LLM representativeness jointly against that taxonomy.
4. `OPS-OBJECTIVES-3` to let operators declare stance per category.
5. `OPS-BENCH-3` to make benchmark and tuning eligibility category-aware and taxonomy-aware.

# Result

The sequence is now:

1. learn the traffic categories Shuma actually sees,
2. become confident in categorizing them,
3. measure whether Scrapling and frontier or LLM traffic jointly cover them,
4. and only then let those lanes participate in autonomous tuning decisions.
