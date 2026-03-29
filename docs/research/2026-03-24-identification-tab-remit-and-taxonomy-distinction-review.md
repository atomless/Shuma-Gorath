Date: 2026-03-24
Status: Historical planning driver (defunct on 2026-03-29)

Related context:

- [`../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte`](../../dashboard/src/lib/components/dashboard/FingerprintingTab.svelte)
- [`../../dashboard/src/lib/components/dashboard/TuningTab.svelte`](../../dashboard/src/lib/components/dashboard/TuningTab.svelte)
- [`../../src/runtime/non_human_taxonomy.rs`](../../src/runtime/non_human_taxonomy.rs)
- [`../research/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-review.md)
- [`../plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](../plans/2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)

Retirement note (2026-03-29):

1. This review is preserved for auditability, but the March 23-24 Identification rename chain is now defunct.
2. Do not treat this note as current dashboard roadmap.

# Objective

Refine the future `Fingerprinting` ownership plan so the tab is renamed to `Identification` and given a clearer remit:

1. show the signals Shuma uses to identify non-human traffic,
2. show how those signals distinguish the non-human taxonomy categories,
3. and stay a truthful explanatory or diagnostic surface rather than a second tuning editor.

# Findings

## 1. `Fingerprinting` is too narrow a label for what the tab should explain

The current tab already mixes more than browser or transport fingerprinting:

1. passive fingerprint signals,
2. corroborative inputs such as JS verification, GEO risk, and rate pressure,
3. provider-origin signals such as Akamai additive input,
4. and runtime scoring-definition material that is really about classification, not just fingerprints.

That means the current name undersells the real role of the tab and encourages a misleading mental model: operators could reasonably assume it is only about browser or transport fingerprints when the real question is broader, namely how Shuma identifies and separates non-human traffic.

## 2. The more useful operator question is “how does Shuma know what this traffic is?”

In the context of Shuma, the tab should help explain:

1. what evidence sources exist,
2. which signals are currently active in scoring or classification,
3. how those signals help distinguish categories in the canonical taxonomy,
4. and where the available evidence is weak or ambiguous.

That is a much clearer remit than a generic fingerprinting pane.

## 3. Category distinction is essential to the tab’s value

If the future tab only lists signals, it will still be incomplete.

For the tab to be genuinely useful, it should also show how those signals help separate:

1. `indexing_bot`,
2. `ai_scraper_bot`,
3. `automated_browser`,
4. `http_agent`,
5. `browser_agent`,
6. `agent_on_behalf_of_human`,
7. `verified_beneficial_bot`,
8. `unknown_non_human`.

Without that, the operator can see ingredients but not understand how the taxonomy is actually being differentiated.

## 4. The right category-distinction surface is bounded role-based explanation, not a noisy full matrix

The important thing is not to render every possible category-to-signal combination.

The cleanest first explanatory surface is a bounded role-based view where:

1. rows are canonical non-human taxonomy categories,
2. each row shows only the signal or evidence families that meaningfully help distinguish that category,
3. those visible families are labeled by explanatory role, for example:
   1. primary,
   2. supporting,
   3. disambiguating.

This is more informative than a single long signal list because it shows how Shuma’s category boundaries are made distinct, without cluttering the tab with explicit `not useful` combinations.

## 5. The tab should distinguish between available, active, and influential evidence

One important risk is that the dashboard could end up showing every theoretically available signal and imply that all are equally important.

The tab should instead distinguish:

1. signals available in principle,
2. signals currently present in the effective scoring or classification definition,
3. signals that are actually influential in distinguishing categories.

That will keep the surface honest and prevent it from becoming a decorative inventory rather than an explanatory tool.

## 6. Explicitly rendering `not useful` entries would add more noise than value in the first version

There is an internal modeling value in recognizing that some signal families are irrelevant for some categories.

But the operator-facing surface does not need to render that absence explicitly in the first version.

Leaving non-helpful combinations blank or unrendered is better because it:

1. keeps attention on the evidence that actually matters,
2. reduces visual clutter,
3. and avoids making the category-distinction surface feel like a sparse spreadsheet instead of an explanation.

# Recommended Direction

## 1. Rename the future tab to `Identification`

`Identification` is a better operator-facing frame because it can truthfully cover:

1. fingerprints,
2. corroborative passive signals,
3. provider-source signals,
4. verified identity evidence as a distinct explanatory input where relevant,
5. and taxonomy distinction logic.

## 2. Define the tab around three sections

The future `Identification` tab should aim to show:

1. `Signal Sources`
   - what evidence families Shuma can observe or ingest
2. `Effective Identification Signals`
   - what signals are currently active in scoring or classification
3. `Category Distinction`
   - how the available signals distinguish the canonical taxonomy categories

Later, if needed, it may also add:

4. `Identification Limits`
   - where category separation is weak, ambiguous, or underpowered

## 3. Keep editing ownership separate

Even after the rename, the tab should remain primarily explanatory and diagnostic.

Editable botness and fingerprint controls should still consolidate into `Tuning`, not move back into this tab. The rename improves the explanatory remit of the read surface; it does not change the previously settled ownership split.

# Conclusion

The correct refinement is not just to rename `Fingerprinting`, but to redefine it as `Identification`:

1. a place where operators can see the evidence Shuma uses to identify non-human traffic,
2. and how that evidence distinguishes the taxonomy categories,
3. while `Tuning` remains the home for editable posture and ratified control knobs.
