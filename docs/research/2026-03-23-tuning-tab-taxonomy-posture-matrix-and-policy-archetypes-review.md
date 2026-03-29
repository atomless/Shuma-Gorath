# Tuning Tab Taxonomy Posture Matrix And Policy Archetypes Review

Date: 2026-03-23
Status: Historical planning driver (defunct on 2026-03-29)

Related context:

- [`2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md`](2026-03-22-canonical-non-human-taxonomy-and-sim-representativeness-review.md)
- [`2026-03-23-dashboard-operator-surfacing-gap-review.md`](2026-03-23-dashboard-operator-surfacing-gap-review.md)
- [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-review.md)
- [`2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md`](2026-03-23-karpathy-autoresearch-and-recursive-self-improvement-review.md)
- [`../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)
- [`../../docs/dashboard-tabs/policy.md`](../../docs/dashboard-tabs/policy.md)
- [`../../docs/dashboard-tabs/tuning.md`](../../docs/dashboard-tabs/tuning.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Retirement note (2026-03-29):

1. This review is preserved for auditability, but its recommended March 23-24 Tuning expansion path is now defunct.
2. The current Tuning contract remains the narrower shipped botness editor.

External references:

- [Cloudflare verified bot categories](https://developers.cloudflare.com/bots/concepts/bot/verified-bots/categories/)
- [Cloudflare manage AI crawlers](https://developers.cloudflare.com/ai-crawl-control/features/manage-ai-crawlers/)
- [Cloudflare Friendly Bots](https://blog.cloudflare.com/friendly-bots/)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)

# Purpose

Decide how Shuma should surface operator-owned non-human traffic posture in the dashboard: where it belongs, how it should be presented, and whether stance presets should exist alongside the per-category posture model.

# Findings

## 1. The category-posture editor is enforcement posture, not passive site policy

The current `Policy` tab owns:

1. `robots.txt` and AI crawl directives,
2. browser-policy declarations,
3. ban-duration configuration,
4. trusted path bypass rules.

That is a mostly declarative and rule-shaped surface. It describes what the site serves or exempts.

The category posture model is different.

It answers:

1. what kinds of non-human traffic the operator wants Shuma to allow, tolerate, cost-shift, restrict, or block,
2. and therefore how the closed feedback loop should optimize enforcement within explicit bounds.

That is closer to defense posture and controller intent than to `robots.txt`-style policy declaration.

Conclusion:

1. the category posture editor belongs in `Tuning`, not `Policy`,
2. even though it is a site-owned policy decision at the product level.

This lines up with the existing dashboard surfacing plan, which already reserves operator-objective and category-posture editing for `TUNE-SURFACE-1`.

## 2. The existing posture scale is already the right semantic model

The machine-first contract in [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs) already persists one canonical posture per category on a bounded five-point scale:

1. `allowed`
2. `tolerated`
3. `cost_reduced`
4. `restricted`
5. `blocked`

The benchmark layer in [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs) already judges current behavior against those posture rows.

This is strong evidence that the dashboard should not invent a different control vocabulary.

Conclusion:

1. the dashboard should project the exact same five-point scale,
2. and the persisted `category_postures` rows should remain the source of truth rather than a UI-only model.

## 3. A taxonomy posture matrix is clearer than independent fieldsets or stacked radio groups

The canonical taxonomy currently contains eight categories in [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs):

1. indexing bot
2. AI scraper bot
3. automated browser
4. HTTP agent
5. browser agent
6. agent on behalf of human
7. verified beneficial bot
8. unknown non-human

Semantically, each row requires exactly one choice from the same shared posture scale.

That is a matrix problem, not a many-fieldset problem.

Conclusion:

1. render one row per category,
2. render one column per posture value,
3. use row-local descriptions from the taxonomy catalog,
4. and make the current selection visually obvious.

This will be easier to scan, compare, and reason about than eight independent radio groups stacked vertically.

## 4. Presets are useful as starting archetypes, but they must not become a second policy system

The user's concern is right:

1. host operators will often want a recognizable starting stance,
2. but the full combinatorial state space is too large to model as an exhaustive preset catalog.

The most honest compromise is:

1. keep `category_postures` as the persisted source of truth,
2. offer a small number of optional stance archetypes that simply seed the matrix,
3. and collapse back to `custom` as soon as the operator edits rows manually.

Conclusion:

1. presets should exist only as a convenience for initializing the matrix,
2. not as a parallel persisted policy model,
3. and not as a replacement for row-level control.

## 5. The first version of presets should stay narrowly scoped to category posture rows

It would be tempting to make a preset also write:

1. verified-identity top-level stance,
2. `robots.txt` AI directives,
3. various threshold or patch-family defaults.

That would be too broad for the first UI slice.

It would create a cross-tab side effect surface that is hard to explain and easy to mistrust.

Conclusion:

1. first-wave presets should seed only the `operator_objectives.category_postures` matrix,
2. and should not silently mutate `Policy` or `Verification` settings.

Those other surfaces can later expose recommended pairings, but the first tuning contract should stay legible.

## 6. A small set of stance archetypes is enough

The current taxonomy and ecosystem patterns suggest a small, useful starter set:

1. `balanced_default`
   - search and clearly beneficial bots tolerated or cost-reduced,
   - unknown and higher-capability automation restricted.
2. `human_only_private`
   - most non-human categories restricted or blocked,
   - useful for private tools and internal apps.
3. `search_visible_ai_restricted`
   - indexing tolerated,
   - AI scraping and ambiguous automation strongly restricted.
4. `agent_friendly_scraper_hostile`
   - delegated or user-triggered agents treated more permissively,
   - scraping and bulk automation restricted or blocked.
5. `open_access_low_friction`
   - more permissive starting stance across multiple categories,
   - still leaves clearly abusive categories lower in posture.

These map well onto real operator intent patterns seen in current ecosystems:

1. Cloudflare distinguishes verified bot categories and allows category-level control.
2. Cloudflare also treats AI crawler control as a distinct operator concern.
3. Google distinguishes user-triggered fetchers from ordinary crawlers, which supports a dedicated "agent on behalf of human" stance rather than a single undifferentiated bot bucket.

Conclusion:

1. Shuma should offer a small starter set,
2. not an exhaustive preset universe.

# Decisions

1. `TUNE-SURFACE-1` should own the non-human category posture editor.
2. The section should be framed as active defense posture, not `robots.txt`-style site policy.
3. The primary UI should be a taxonomy posture matrix:
   - rows = canonical non-human categories,
   - columns = `allowed`, `tolerated`, `cost_reduced`, `restricted`, `blocked`.
4. Each row must include the stable category label and short description from the canonical taxonomy.
5. A small optional preset selector should sit above the matrix and seed row values only.
6. Presets must not become a second persisted model; `category_postures` remains the source of truth.
7. The first preset implementation should not silently mutate `Policy` or `Verification` tab state.
8. Manual edits after preset application should explicitly transition the UI to a `custom` state.

# Result

The clean operator model is:

1. `Policy` remains the place for declarative crawl rules, path exemptions, and related static site rules.
2. `Tuning` becomes the place where operators declare how aggressively Shuma should enforce against each non-human category.
3. The closed loop then tunes within those explicit category bounds rather than inferring operator intent from benchmark movement alone.

This keeps Shuma's recursive-improvement loop honest:

1. operator intent is explicit,
2. enforcement optimization is emergent,
3. and the dashboard does not need to invent a second policy language to bridge the two.
