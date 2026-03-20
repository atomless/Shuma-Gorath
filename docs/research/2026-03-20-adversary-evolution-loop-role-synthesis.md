# Adversary Evolution Loop Role Synthesis

Date: 2026-03-20
Status: Active synthesis for mature adversary-sim planning

Related context:

- [`2026-02-25-llm-adversarial-testing-research-synthesis.md`](./2026-02-25-llm-adversarial-testing-research-synthesis.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`2026-03-20-benchmark-suite-v1-research-synthesis.md`](./2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)

## Purpose

Clarify the role of deterministic, Scrapling, and frontier-agent adversary lanes now that Shuma's goal has shifted from contributor diagnostics toward a real adaptive feedback loop:

1. emergent attack traffic finds weaknesses,
2. benchmarkable telemetry measures those weaknesses,
3. a diagnosis and tuning agent proposes mitigation,
4. and successful emergent findings are promoted into deterministic memory so the system learns.

## Research Question

How should Shuma split roles between:

1. deterministic scripted traffic,
2. Scrapling-style emergent crawling,
3. frontier-agent adversary traffic,
4. and the later diagnosis/tuning harness,

so that the feedback loop is adaptive without becoming too noisy to trust?

## Findings

### 1. Deterministic traffic is still valuable, but not as the primary adaptive lane

The earlier LLM adversarial research already made the right distinction: adaptive or emergent attacker behavior is the discovery engine, while deterministic scenarios are what make comparisons stable enough to trust across runs and releases.

Relevant sources:

- [`2026-02-25-llm-adversarial-testing-research-synthesis.md`](./2026-02-25-llm-adversarial-testing-research-synthesis.md)
- [OpenAI evaluation best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices)

Implications:

1. Deterministic traffic should remain the oracle and comparator.
2. It should not be treated as the primary source of adaptive diagnosis.
3. Emergent lanes should discover new exploit patterns that deterministic scenarios later preserve as memory.

### 2. The first real adaptive loop should be driven by emergent lanes

Kubernetes-style controller loops and Shuma's own machine-first snapshot work point in the same direction: the useful control loop starts with current observed behavior, not a static regression script alone.

Relevant sources:

- [Kubernetes controllers](https://kubernetes.io/docs/concepts/architecture/controller/)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)

Implications:

1. Scrapling and later frontier-agent lanes should be the primary adaptive feedback inputs.
2. Deterministic traffic should validate and preserve what those lanes discover.
3. The first diagnosis harness can be machine-first and recommend-only before a full human Monitoring UI exists.

### 3. Shared-host discovery is still needed, but only as a safety and seed contract

The earlier Scrapling plan treated full shared-host discovery as the gate before the emergent lane. That is now too heavy for the loop Shuma wants.

Implications:

1. Shuma still needs a fail-closed scope contract and initial seed discovery.
2. But full discovery artifact generation should not remain the primary gate before useful Scrapling execution starts.
3. Continuous discovery should become a byproduct of the emergent harness itself.
4. Observed telemetry should become the authoritative map of what the emergent harness actually reached.

### 4. Emergent findings should feed deterministic memory through reviewed promotion

This is the key new synthesis.

The right long-term loop is:

1. emergent lane surfaces a meaningful exploit,
2. diagnosis agent proposes a config change,
3. diagnosis agent also proposes a deterministic replay candidate,
4. the replay is reduced to a stable representative sequence,
5. the deterministic corpus absorbs it after review,
6. and future config or code changes are judged against that promoted scenario.

Implications:

1. Deterministic sim becomes curated memory, not merely a static baseline.
2. Emergent lanes become discovery engines.
3. The diagnosis harness becomes the bridge between discovery and memory.

## Synthesis

Shuma should stop thinking of the deterministic lane as "the sim" and instead treat it as one role inside a broader adversary evolution loop:

1. `deterministic_oracle`
   - release-blocking regression authority
   - before/after comparator
   - curated memory of known exploits
2. `scrapling_emergent`
   - first adaptive discovery lane
   - lower-cost and more controllable than full frontier agents
3. `frontier_agent`
   - later high-capability discovery lane
   - strategic but noisier and costlier
4. `diagnosis_and_tuning_agent`
   - reads machine-first snapshot and benchmark results
   - proposes config diffs
   - proposes deterministic replay candidates

## Recommended Planning Consequence

The mature adversary-sim roadmap should be rewritten so that:

1. deterministic traffic is explicitly demoted from "primary tuning signal" to oracle/comparator,
2. Scrapling and later frontier-agent lanes are the primary adaptive inputs,
3. shared-host discovery is narrowed to a minimal scope and seed gate,
4. observed telemetry is treated as the authoritative surface map for emergent lanes,
5. and promotion from emergent exploit to deterministic scenario becomes an explicit roadmap concept.
