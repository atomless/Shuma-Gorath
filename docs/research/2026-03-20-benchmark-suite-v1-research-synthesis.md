# Benchmark Suite v1 Research Synthesis

Date: 2026-03-20
Status: Active synthesis for Stage 2 planning

Related context:

- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](./2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Purpose

Define the first benchmark families Shuma should use for `benchmark_suite_v1` so that:

1. a future instance-tuning loop can judge whether config changes improved outcomes,
2. a later project-evolution loop can judge whether the Shuma codebase itself needs to evolve,
3. those judgments are based on explicit budgets and measurable regressions,
4. and the human Monitoring surface later becomes a projection over the same benchmark semantics rather than a separate human-only interpretation layer.

## Research Question

What does current state-of-the-art practice suggest about:

1. how benchmark families should be shaped for controller-grade decision making,
2. which outcome classes matter most for modern bot-defence systems,
3. how beneficial or authenticated non-human traffic should be measured distinctly from hostile automation,
4. and how Shuma should compare config changes, code changes, and observation windows without drifting into narrative dashboard metrics?

## External Findings

### 1. Controller-grade benchmarks should be budget and ratio based, not count-only

Google's SRE material defines the error budget as the allowed failure fraction implied by an SLO and frames it as the control mechanism for trading innovation against stability. That is a strong signal that Shuma's benchmark families should be defined against eligible populations and target budgets, not as raw totals that grow with traffic volume.

Relevant source:

- [Google SRE workbook: Error Budget Policy](https://sre.google/workbook/error-budget-policy/)

Implications:

1. Each benchmark family should state its eligible population explicitly.
2. Each family should compute ratios or budget distance, not just counts.
3. Regression and improvement need to be judged over windows, not single anecdotes.

### 2. Safe change loops rely on explicit verification windows and baseline comparison

Google Cloud Deploy's official documentation treats verification as a first-class phase of rollout, rather than a manual afterthought. That reinforces that Shuma's benchmark results should compare a subject against a prior baseline or watch window and support staged judgment rather than one-shot mutation.

Relevant sources:

- [Cloud Deploy verify your deployment](https://cloud.google.com/deploy/docs/verify-deployment)
- [Cloud Deploy canary deployments for Cloud Run](https://cloud.google.com/deploy/docs/deployment-strategies/canary/cloud-run)

Implications:

1. `benchmark_results_v1` should include baseline and current subject references.
2. A benchmark result should carry watch-window semantics, not only present-tense counters.
3. Later config or code changes should be judged as verified improvement, neutral, or regression against prior evidence.

### 3. Modern bot-defence analytics are class-aware and source-aware

Cloudflare's official bot documentation emphasizes traffic type segmentation, multiple detection engines, and analytics APIs rather than one blended "bot blocked" metric. That suggests Shuma's benchmark families must not collapse hostile automation, declared crawlers, beneficial fetchers, and verified agents into one bucket.

Relevant sources:

- [Cloudflare Bot Analytics](https://developers.cloudflare.com/bots/bot-analytics/)
- [Cloudflare bot detection engines](https://developers.cloudflare.com/bots/concepts/bot-detection-engines/)
- [Cloudflare AI crawler reference](https://developers.cloudflare.com/ai-crawl-control/reference/bots/)

Implications:

1. Benchmark families should preserve traffic class and origin distinctions.
2. Detection-source or policy-source context matters for interpretation, even when it is not headline UI material.
3. Shuma should measure hostile cost-shifting separately from intended allowance of declared or verified automation.

### 4. Beneficial non-human traffic is already more granular than one "good bot" bucket

Google's crawler documentation explicitly separates common crawlers, special-case crawlers, and user-triggered fetchers, and its verification guidance emphasizes that user agents alone are not enough. That reinforces that Shuma needs a dedicated beneficial-non-human benchmark family, not just a side note inside hostile-traffic metrics.

Relevant sources:

- [Google crawler overview](https://developers.google.com/crawling/docs/crawlers-fetchers/overview-google-crawlers)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [Verify Google crawler requests](https://developers.google.com/crawling/docs/crawlers-fetchers/verify-google-requests)

Implications:

1. Shuma should benchmark declared crawlers, user-triggered agents, verified bots, and signed agents distinctly as the capabilities land.
2. Benchmarks must be stance-aware: some sites will intentionally deny all non-human traffic, while others will allow subsets.
3. Success for beneficial automation cannot be defined globally; it must be evaluated against local policy stance.

### 5. LLM-facing control artifacts should be structured, bounded, and explicitly typed

OpenAI's official guidance emphasizes using structured outputs and limiting agent interfaces to structured, bounded data. That strongly suggests Shuma's benchmark artifacts should be machine-readable, schema-versioned, and exactness-tagged rather than free-form dashboards or raw tails.

Relevant sources:

- [OpenAI structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs)
- [OpenAI safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)

Implications:

1. `benchmark_suite_v1` should be a typed contract, not an informal operator narrative.
2. `benchmark_results_v1` should be bounded, schema-versioned, and comparison-friendly.
3. Future controller logic should consume benchmark results directly, not chart screenshots or raw event feeds.

## Research Synthesis

Taken together, the evidence points to one clear direction:

1. Shuma should start `benchmark_suite_v1` with a very small number of benchmark families,
2. each family should be ratio and budget oriented,
3. each family should be grounded in typed traffic classes and explicit policy stance,
4. and benchmark results should compare current posture against a baseline or prior watch window rather than merely echoing present counters.

## Recommended First Benchmark Families

### 1. `suspicious_origin_cost`

Question answered:

1. How much suspicious or hostile traffic is still reaching the defended site, and how much cost is being shifted back onto Shuma instead?

Why this should be first:

1. It directly reflects Shuma's core value proposition: increase attacker cost while reducing defended-site cost.
2. It aligns with the controller-grade byte and outcome telemetry foundation already landed.
3. It can inform both instance tuning and project-level code evolution.

Expected first-wave metrics:

1. suspicious forwarded request rate,
2. suspicious forwarded byte rate,
3. suspicious short-circuit rate,
4. suspicious locally served byte share.

### 2. `likely_human_friction`

Question answered:

1. How much friction is Shuma imposing on likely-human or interactive traffic, and is that within budget?

Why this should be first:

1. It is the main counterbalance to hostile-cost shaping.
2. It gives the future controller a principled reason to loosen defences when friction rises too high.
3. It supports later project-evolution decisions when the current code cannot reduce bot leakage without excessive human pain.

Expected first-wave metrics:

1. likely-human friction rate,
2. interactive friction rate,
3. likely-human hard-block rate,
4. later, latency or cost proxies once Shuma collects them explicitly.

### 3. `representative_adversary_effectiveness`

Question answered:

1. Against known representative hostile scenarios, how effective is the current Shuma posture?

Why this should be first:

1. Live traffic alone is not enough to reveal blind spots or verify improvements.
2. Adversary-sim already exists as a first-class part of Shuma's operating model.
3. This family bridges the instance loop and the later code-evolution loop, because code changes can be benchmarked on representative scenarios before relying on live outcomes.

Expected first-wave metrics:

1. scenario goal-success rate,
2. origin-reach rate,
3. challenge or trap escalation rate,
4. scenario-family regression or improvement flags.

### 4. `beneficial_non_human_posture`

Question answered:

1. Is Shuma treating beneficial or authenticated non-human traffic in line with the site's declared stance?

Why this should be first even if some capabilities are not yet implemented:

1. The agentic era requires a clear stance on crawlers, user-triggered agents, verified bots, and signed agents.
2. The benchmark family needs to exist conceptually now so later verified-identity work lands into a prepared contract rather than inventing a parallel one.
3. This family gives Shuma a way to prove improvement for sites that intentionally allow certain automated traffic and also for sites that intentionally deny all non-human traffic.

Expected first-wave posture:

1. capability-gated until the verified-identity foundation lands,
2. stance-aware from the beginning,
3. and explicit about `not_applicable` or `not_yet_supported` rather than pretending coverage exists today.

## Recommended Common Family Rules

Every benchmark family should define:

1. the decision question it answers,
2. the eligible population,
3. the numerator and denominator,
4. the exactness and evidentiary basis,
5. the target or budget,
6. the tolerance band,
7. and the comparison modes it supports.

Those comparison modes should include, at minimum:

1. current versus prior window,
2. current versus baseline,
3. and later candidate code or config versus baseline.

## Recommended Result Contract Direction

`benchmark_results_v1` should not be a giant analytics dump. It should remain bounded and emphasize:

1. family status,
2. metric deltas,
3. improvement or regression classification,
4. benchmark eligibility and capability gates,
5. and whether the benchmark miss appears addressable by config tuning or suggests code evolution.

## What This Means For Shuma Now

The next planning step should be:

1. define `benchmark_suite_v1` as a typed contract over the four families above,
2. define `benchmark_results_v1` as the bounded comparison envelope,
3. define the explicit decision boundary between config tuning and code evolution,
4. and keep Monitoring blocked from inventing a human-only semantic model before those contracts are written down.

## Sources

Primary external sources used in this synthesis:

- [Google SRE workbook: Error Budget Policy](https://sre.google/workbook/error-budget-policy/)
- [Cloud Deploy verify your deployment](https://cloud.google.com/deploy/docs/verify-deployment)
- [Cloud Deploy canary deployments for Cloud Run](https://cloud.google.com/deploy/docs/deployment-strategies/canary/cloud-run)
- [Cloudflare Bot Analytics](https://developers.cloudflare.com/bots/bot-analytics/)
- [Cloudflare bot detection engines](https://developers.cloudflare.com/bots/concepts/bot-detection-engines/)
- [Cloudflare AI crawler reference](https://developers.cloudflare.com/ai-crawl-control/reference/bots/)
- [Google crawler overview](https://developers.google.com/crawling/docs/crawlers-fetchers/overview-google-crawlers)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [Verify Google crawler requests](https://developers.google.com/crawling/docs/crawlers-fetchers/verify-google-requests)
- [OpenAI structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs)
- [OpenAI safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
