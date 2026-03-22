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
4. then govern which evidence is protected and tuning-eligible,
5. and only then judge representativeness, diagnosis quality, and tuning readiness.

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

## 4. The actively evolving layer should be classification quality, not taxonomy breadth

The user is also right that the system should improve over time.

But the near-term priority is not to make taxonomy evolution itself a first-class gate.

The higher-priority adaptive layer is the fingerprinting, evidence, and categorization quality that decides whether traffic fits the existing categories well enough to act on.

That means the loop should first improve:

1. which signals it trusts,
2. how it weights or combines those signals,
3. how confident it is in a category assignment,
4. and how it distinguishes exact classification from best guess or insufficient evidence.

That classification layer should improve over time for both simulated and observed traffic without requiring category proliferation every time the signal model gets better.

Taxonomy expansion should stay a later contingency only if Shuma persistently encounters important non-human traffic that does not fit the current categories well enough.

## 5. The classification contract should map both live and simulated traffic into the same taxonomy

Before Shuma can judge whether Scrapling and frontier or LLM traffic are representative, it needs a bounded classification layer that can assign both real and simulated traffic into the same canonical category model.

That avoids circular reasoning:

1. categories are defined first,
2. observed and simulated traffic are both classified into those categories,
3. and lane representativeness is then judged by whether the generated traffic actually lands in the intended categories with the expected characteristics.

## 6. Scrapling and frontier or LLM lanes should jointly own category fulfillment

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

## 7. Diagnosis should judge defenses against the designed category set before auto-apply exists

Once Shuma has:

1. a canonical category list,
2. a classifier that maps simulated traffic into that list,
3. and lanes capable of generating those categories,

the recommend-only diagnoser can already inspect how the defenses perform against each simulated category and whether the cost imposed on the host stays inside budget.

That keeps the current diagnosis stage useful without prematurely enabling auto-apply.

## 8. Autonomous tuning should optimize only over categories that are both classifiable and represented

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

1. The first prerequisite is a seeded canonical non-human taxonomy, not an observed-traffic taxonomy.
2. The second prerequisite is a classification contract that can map both observed and simulated traffic into that taxonomy.
3. Scrapling and frontier or containerized LLM lanes jointly own category fulfillment and later representativeness against that taxonomy.
4. Protected tuning evidence eligibility must be explicit before autonomous tuning is reopened.
5. The actively evolving near-term layer should be fingerprinting and classification quality within the canonical taxonomy.
6. Taxonomy expansion is a later contingency only if important non-human traffic persistently falls outside the existing categories.
7. The recommend-only diagnoser should be extended to judge defenses against those simulated categories before autonomous apply is reopened.
8. Autonomous tuning remains blocked until category definition, classification confidence, protected-evidence rules, lane fulfillment, and representativeness are all machine-readable.

# Required Follow-On Work

1. `TRAFFIC-TAX-1` to define the canonical non-human traffic taxonomy Shuma intends to simulate and defend against.
2. `TRAFFIC-TAX-2` to add a classification contract and evidence receipts that can map both observed and simulated traffic into that taxonomy.
3. `SIM-FULFILL-1` to implement the category-to-lane fulfillment matrix across Scrapling and frontier or containerized LLM modes.
4. `SIM-COVER-1` to measure whether those lanes actually generate traffic that fits the intended categories.
5. `SIM-PROTECTED-1` to codify which classified adversary evidence is tuning-eligible and which remains advisory or harness-only.
6. `OPS-OBJECTIVES-3` to let operators declare stance and budget expectations per category.
7. `OPS-BENCH-3` to make benchmark and diagnosis outputs category-aware while preserving visibility into classification confidence.
8. `OVR-APPLY-1` to add bounded apply and rollback only after the above gates are real.

# Result

The corrected sequence is now:

1. define the categories Shuma wants to model,
2. build the classifier that can recognize those categories in both simulated and real traffic,
3. build lane behaviors designed to fulfill them,
4. define which evidence is protected enough to tune against,
5. improve the fingerprinting and classification quality that maps traffic into those categories,
6. judge defensive performance against those simulated categories and budgets,
7. then let the tuning loop recommend, apply, and repeat.

Later, if important non-human traffic persistently falls outside the existing categories, Shuma can add a governed taxonomy-expansion path, but that is not a first-loop priority.
