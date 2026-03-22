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

Freeze the corrected sequencing rule for autonomous tuning:

1. first define Shuma's own canonical non-human traffic taxonomy,
2. then build a classification contract that can map both observed and simulated traffic into that taxonomy,
3. then implement Scrapling and frontier or containerized LLM lane behaviors designed to fulfill those categories,
4. and only then judge representativeness, diagnosis quality, and tuning readiness.

# Findings

## 1. Shuma cannot wait for attackers before defining the category model

The earlier observed-first framing was too strict for a pre-launch security system.

Shuma has to be configurable before a site has accumulated enough adversary traffic to act as its own teacher. That means the first category model must be proactive:

1. research-backed,
2. product-owned,
3. and explicit about the non-human behaviors Shuma intends to distinguish and tune against.

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

## 3. The taxonomy should be canonical first, then calibrated by real traffic

Vendor taxonomies are useful seeds, but Shuma needs its own canonical model driven by:

1. the non-human behaviors it wants to simulate and defend against,
2. verified identity or provenance when available,
3. operator intent,
4. and the defensive controls that tuning can actually change.

Observed traffic still matters, but as a later calibration and confidence input, not as the first source of category existence.

## 4. The classification contract should map both live and simulated traffic into the same taxonomy

Before Shuma can judge whether Scrapling and frontier or LLM traffic are representative, it needs a bounded classification layer that can assign both real and simulated traffic into the same canonical category model.

That avoids circular reasoning:

1. categories are defined first,
2. observed and simulated traffic are both classified into those categories,
3. and lane representativeness is then judged by whether the generated traffic actually lands in the intended categories with the expected characteristics.

## 5. Scrapling and frontier or LLM lanes should jointly own category fulfillment

The right contract is not that each lane must independently represent every non-human category.

The better contract is:

1. Shuma defines the canonical non-human taxonomy,
2. Shuma classifies both observed and simulated traffic into that taxonomy,
3. Scrapling and frontier or containerized LLM lanes are implemented in modes explicitly designed to fulfill those categories,
4. representativeness is then measured as joint coverage against that taxonomy,
5. and the closed tuning loop can only use categories for which protected evidence exists.

That means:

1. Scrapling is likely the first strong source for crawl, scrape, and lower-complexity automated browsing pressure,
2. frontier or containerized LLM lanes likely fulfill browser-controlled, multi-step agentic, and higher-capability request behaviors,
3. and together they should aim to cover the category set the operator cares about.

## 6. Diagnosis should judge defenses against the designed category set before auto-apply exists

Once Shuma has:

1. a canonical category list,
2. a classifier that maps simulated traffic into that list,
3. and lanes capable of generating those categories,

the recommend-only diagnoser can already inspect how the defenses perform against each simulated category and whether the cost imposed on the host stays inside budget.

That keeps the current diagnosis stage useful without prematurely enabling auto-apply.

## 7. Autonomous tuning should optimize only over categories that are both classifiable and represented

Google's canary guidance is clear that synthetic or unrepresentative traffic is unsafe as a sole basis for rollout judgment.

Source:

1. [Google SRE Workbook: Canarying Releases](https://sre.google/workbook/canarying-releases/)

For Shuma that means a category is tuning-eligible only when:

1. Shuma has defined it in the canonical taxonomy,
2. Shuma can classify both simulated and, later, observed traffic into it with bounded confidence,
3. the operator has declared the desired stance for it,
4. and the protected adversary evidence set actually represents it.

Observed traffic should later refine confidence and weighting, but it should not be a prerequisite for category design or initial sim-lane fulfillment.

# Decisions

1. The first prerequisite is a canonical non-human taxonomy, not an observed-traffic taxonomy.
2. The second prerequisite is a classification contract that can map both observed and simulated traffic into that taxonomy.
3. Scrapling and frontier or containerized LLM lanes jointly own category fulfillment and later representativeness against that taxonomy.
4. The recommend-only diagnoser should be extended to judge defenses against those simulated categories before autonomous apply is reopened.
5. Autonomous tuning remains blocked until category definition, classification confidence, lane fulfillment, and representativeness are all machine-readable.

# Required Follow-On Work

1. `TRAFFIC-TAX-1` to define the canonical non-human traffic taxonomy Shuma intends to simulate and defend against.
2. `TRAFFIC-TAX-2` to add a classification contract and evidence receipts that can map both observed and simulated traffic into that taxonomy.
3. `SIM-FULFILL-1` to implement the category-to-lane fulfillment matrix across Scrapling and frontier or containerized LLM modes.
4. `SIM-COVER-1` to measure whether those lanes actually generate traffic that fits the intended categories.
5. `OPS-OBJECTIVES-3` to let operators declare stance and budget expectations per category.
6. `OPS-BENCH-3` to make benchmark and diagnosis outputs category-aware.
7. `OVR-APPLY-1` to add bounded apply and rollback only after the above gates are real.

# Result

The corrected sequence is now:

1. define the categories Shuma wants to model,
2. build the classifier that can recognize those categories in both simulated and real traffic,
3. build lane behaviors designed to fulfill them,
4. judge defensive performance against those simulated categories and budgets,
5. then let the tuning loop recommend, apply, and repeat.
