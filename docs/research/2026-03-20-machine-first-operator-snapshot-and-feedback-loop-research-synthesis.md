# Machine-First Operator Snapshot and Feedback Loop Research Synthesis

Date: 2026-03-20
Status: Active synthesis for Stage 2 planning

Related context:

- [`2026-03-15-agentic-era-oversight-research-synthesis.md`](./2026-03-15-agentic-era-oversight-research-synthesis.md)
- [`2026-03-17-operator-decision-support-telemetry-audit.md`](./2026-03-17-operator-decision-support-telemetry-audit.md)
- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](./2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-19-controller-readiness-telemetry-foundation-review.md`](./2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)

## Purpose

Determine the long-term destination for Shuma's monitoring surface if the primary operator is expected to become a scheduled frontier-model agent that:

1. reads telemetry periodically,
2. compares current outcomes to explicit bot-exclusion and human-friction goals,
3. proposes bounded config changes,
4. evaluates the next observation window,
5. and eventually escalates to code-change suggestions only when config-only tuning cannot close the gap.

The specific question is not "what charts should Monitoring show?" The question is "what bounded contract should Shuma provide so a future operator agent can act safely, truthfully, and cheaply?"

## Research Question

What do current state-of-the-art operating patterns suggest about:

1. control-loop design,
2. objective and budget modeling,
3. bot-traffic classification and verification,
4. machine-readable operator telemetry,
5. and the relationship between agent-facing control data and human-facing dashboards?

## External Findings

### 1. Mature control loops are desired-state reconciler systems, not narrative dashboards

Kubernetes documents the controller pattern as a system where a controller tracks desired state and works to bring current state closer to it. It also explicitly favors many small controllers rather than one large monolith, and notes that control loops may act on external systems as long as they report resulting state back into the system of record. See:

- [Kubernetes controllers](https://kubernetes.io/docs/concepts/architecture/controller/)

Key implications from the official docs:

1. Desired and current state should be explicit and typed.
2. Controllers should reconcile narrow concerns.
3. External actions are acceptable only when their results are fed back into the same authoritative state model.

### 2. Safe production change loops are budgeted, phased, and verified

Google Cloud's official guidance on service monitoring and deployment verification reinforces two ideas:

1. SLOs and error budgets are the right abstraction for deciding whether a system can tolerate additional change.
2. Risky changes should be phased and accompanied by explicit verification steps and retry or rollback posture.

Relevant docs:

- [Google Cloud service monitoring concepts](https://cloud.google.com/stackdriver/docs/solutions/slo-monitoring)
- [Cloud Deploy canary deployments for Cloud Run](https://cloud.google.com/deploy/docs/deployment-strategies/canary/cloud-run)
- [Cloud Deploy verify your deployment](https://cloud.google.com/deploy/docs/verify-deployment)

Important signals from those docs:

1. Error budgets are computed from eligible events over a compliance window, not from ad hoc intuition.
2. Canary rollout phases are explicitly percentage-based and staged.
3. Verification is a first-class deployment job, not an informal afterthought.

For Shuma this strongly suggests that autonomous tuning should optimize against explicit budgets such as:

1. likely-human friction budget,
2. suspicious-origin-cost budget,
3. and benchmark windows derived from live traffic plus adversary-sim evidence.

### 3. Bot telemetry in mature systems is class-based, source-aware, and API-backed

Cloudflare's official bot docs show a clear split between:

1. traffic type,
2. detection source,
3. top offender attributes,
4. and API-backed access to the same semantics.

Relevant docs:

- [Cloudflare Bot Analytics](https://developers.cloudflare.com/bots/bot-analytics/)
- [Cloudflare bot detection engines](https://developers.cloudflare.com/bots/concepts/bot-detection-engines/)

Important lessons:

1. Bot telemetry is segmented by traffic type and detection source, not a single flat score.
2. Detection engines are plural: heuristics, JavaScript detections, machine learning, anomaly detection.
3. Cloudflare exposes analytics data through an API as well as a dashboard.
4. Cloudflare's dashboard analytics are sampled in many cases, which is acceptable for human analytics but a bad foundation for an autonomous controller.

Inference:

Shuma should not make a future operator agent scrape or interpret chart-friendly, potentially sampled human dashboards. It should expose a bounded, exactness-tagged machine contract that the human dashboard also consumes.

### 4. Non-human traffic classes are already more granular than "bot"

Google's crawler docs distinguish among:

1. common crawlers that respect `robots.txt`,
2. special-case crawlers,
3. and user-triggered fetchers that generally ignore `robots.txt` because they act on behalf of a user.

Relevant docs:

- [Google crawler overview](https://developers.google.com/crawling/docs/crawlers-fetchers/overview-google-crawlers)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [Verify Google crawler requests](https://developers.google.com/crawling/docs/crawlers-fetchers/verify-google-requests)

Important lessons:

1. There is already a first-class distinction between autonomous crawling and user-triggered fetching.
2. Authenticity verification still matters; Google documents reverse-DNS plus forward-DNS verification rather than assuming user agents are truthful.
3. Policy communication surfaces such as `robots.txt` do not cover every kind of non-human request.

Combined with prior Shuma research on verified bots, signed agents, and Web Bot Auth, this reinforces that Monitoring and later control loops must preserve separate categories for:

1. likely-human interactive traffic,
2. suspicious automation,
3. declared crawlers,
4. user-triggered agents or fetchers,
5. verified bots,
6. signed agents,
7. and adversary-sim traffic as a separate origin rather than a peer traffic class.

### 5. LLM control loops should consume structured, bounded data and produce structured, bounded outputs

OpenAI's official agent safety guidance says untrusted data should not directly drive agent behavior and recommends structured outputs to constrain data flow. The same guidance explicitly recommends running evals and trace graders so model behavior is scored and understood rather than inferred from vibes. OpenAI's structured outputs guide further emphasizes reliable typed outputs and programmatic detectability.

Relevant docs:

- [OpenAI safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [OpenAI structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs)

Important lessons:

1. Structured outputs reduce prompt-injection risk and freeform ambiguity.
2. Evals and trace grading should be part of the control loop, not bolted on later.
3. A future Shuma operator agent should receive bounded JSON with typed fields, not raw text, raw events, or HTML intended for people.

## Research Synthesis

Taken together, the external evidence points toward one clear direction:

1. Monitoring should become a machine-first operator-state contract.
2. The human Monitoring tab should be a thin projection of that same contract.
3. Raw subsystem detail should stay in Diagnostics.
4. Autonomous tuning should operate on explicit objectives, budgets, and bounded action families.
5. Verification and rollback posture must be part of the feedback loop from the start.

An additional implication becomes clear when this is combined with Shuma's larger roadmap:

1. there are really two future feedback loops, not one,
2. and they must stay distinct even though they inform each other.

### Loop A: Instance tuning loop

This loop adjusts the config of one defended Shuma instance.

It should optimize against:

1. per-instance human-friction budgets,
2. per-instance suspicious-origin leakage or cost budgets,
3. per-instance adversary-sim benchmark outcomes,
4. and local policy stance for non-human traffic.

### Loop B: Project evolution loop

This loop does not adjust one site's config. It evaluates whether the Shuma project itself needs to evolve.

It should optimize against:

1. benchmark suites aggregated across adversary-sim, live traffic outcomes, and later central intelligence,
2. fleet-level evidence about which attack classes are becoming cheaper or more successful,
3. and measurable improvements in bot-cost asymmetry and human-friction reduction after code changes.

The first loop should mature earlier. The second loop should remain more bounded and review-heavy, because code and PR generation has a much larger blast radius than config tuning.

## Recommended Long-Term Destination

Shuma should converge on three related backend contracts:

1. `operator_objectives_v1`
   - explicit goals and budgets for human friction, suspicious-origin cost, desired bot exclusion, and desired allowed verified-agent posture.
2. `operator_snapshot_v1`
   - a bounded, exactness-tagged, machine-readable snapshot of current state over a defined window.
3. `allowed_actions_v1`
   - the bounded set of config families and thresholds that a scheduled controller is allowed to propose for adjustment.

It should also converge on two benchmark artifacts for the project-evolution loop:

4. `benchmark_suite_v1`
   - the canonical definitions of the benchmark families Shuma uses to decide whether the codebase itself has improved.
5. `benchmark_results_v1`
   - bounded benchmark outputs across adversary-sim, representative live-traffic summaries, and later central-intelligence evidence.

The future human Monitoring tab should not invent a separate semantic model. It should simply render selected parts of `operator_snapshot_v1`.

## What This Means For Shuma Now

The next Stage 2 work should not start with a chart-first Monitoring redesign.

Instead, the sequence should be:

1. define the machine-first objective contract,
2. define and materialize `operator_snapshot_v1`,
3. define the bounded action surface and recent-change ledger needed for tune-confirm-repeat loops,
4. define the benchmark families that will later tell Shuma whether code changes actually improved the arms race,
5. then build a thin Monitoring projection over those backend contracts,
6. then complete the Tuning tab against the same action model,
7. and only after that plan the scheduled recommend-or-apply agent in detail and the later code-evolution loop.

## Explicit Recommendations

1. Reframe `MON-OVERHAUL-1` as a thin Monitoring projection over a machine-first operator snapshot, not as a charting exercise.
2. Add a new active tranche for `OPS-SNAPSHOT-1`, covering objectives, snapshot materialization, exactness, change ledger, and allowed action families.
3. Add a later planning tranche for benchmark-grade project evolution so Shuma can judge code changes against explicit bot-cost and human-friction criteria instead of anecdotes.
4. Keep `OVR-AGENT-2` blocked until `OPS-SNAPSHOT-1` and the Tuning surface define a truthful controller input and action contract.
5. Keep code and PR generation out of the first controller loop; treat it as a second, more review-heavy loop that consumes benchmark results rather than only live per-instance telemetry.
6. Keep Diagnostics as the home for raw subsystem, transport, and drill-down detail.
7. Treat sampled or human-dashboard-only views as insufficient for future autonomous operation.

## Sources

Primary external sources used in this synthesis:

- [Kubernetes controllers](https://kubernetes.io/docs/concepts/architecture/controller/)
- [Google Cloud service monitoring concepts](https://cloud.google.com/stackdriver/docs/solutions/slo-monitoring)
- [Cloud Deploy canary deployments for Cloud Run](https://cloud.google.com/deploy/docs/deployment-strategies/canary/cloud-run)
- [Cloud Deploy verify your deployment](https://cloud.google.com/deploy/docs/verify-deployment)
- [Cloudflare Bot Analytics](https://developers.cloudflare.com/bots/bot-analytics/)
- [Cloudflare bot detection engines](https://developers.cloudflare.com/bots/concepts/bot-detection-engines/)
- [Google crawler overview](https://developers.google.com/crawling/docs/crawlers-fetchers/overview-google-crawlers)
- [Google user-triggered fetchers](https://developers.google.com/crawling/docs/crawlers-fetchers/google-user-triggered-fetchers)
- [Verify Google crawler requests](https://developers.google.com/crawling/docs/crawlers-fetchers/verify-google-requests)
- [OpenAI safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)
- [OpenAI structured outputs](https://developers.openai.com/api/docs/guides/structured-outputs)
