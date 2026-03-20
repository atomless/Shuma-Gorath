# Benchmark Fleet And Intelligence Enrichment Research Synthesis

Date: 2026-03-20
Status: Active synthesis for later benchmark and central-intelligence planning

Related context:

- [`2026-03-20-benchmark-suite-v1-research-synthesis.md`](./2026-03-20-benchmark-suite-v1-research-synthesis.md)
- [`2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md`](./2026-03-20-machine-first-operator-snapshot-and-feedback-loop-research-synthesis.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](./2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)
- [`../plans/2026-03-20-benchmark-suite-v1-design.md`](../plans/2026-03-20-benchmark-suite-v1-design.md)
- [`../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](../plans/2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)

## Purpose

Define how later fleet-level and shared-intelligence evidence should enrich Shuma's benchmark system without corrupting the machine-first local benchmark truth already established by:

1. `benchmark_suite_v1`,
2. `benchmark_results_v1`,
3. `operator_snapshot_v1`,
4. and the later controller and code-evolution loops.

## Research Question

What does current practice suggest about how Shuma should let fleet or central-intelligence evidence influence:

1. benchmark scenario selection,
2. benchmark family priority,
3. bounded weight bias between benchmark families,
4. and benchmark-driven escalation toward config tuning or later code evolution,

without:

1. replacing local benchmark truth,
2. letting advisory reputation become automatic enforcement,
3. or turning the Git repository into the dynamic transport for shared benchmark state?

## External Findings

### 1. Control loops should reconcile local current state against explicit desired state

Kubernetes controller guidance is clear that controllers observe current state, compare it to desired state, make bounded changes, and then report current state back so other loops can react. That is a strong signal that Shuma's future controller loop should treat fleet or intelligence evidence as additional context, not as a replacement for the local snapshot and local benchmark results.

Relevant source:

- [Kubernetes controllers](https://kubernetes.io/docs/concepts/architecture/controller/)

Implications:

1. Local `operator_snapshot_v1` remains the authoritative current-state contract for an instance.
2. Fleet or intelligence evidence should enrich benchmark emphasis, not rewrite local benchmark status.
3. Any later controller loop should still reconcile against local objectives and local benchmark outcomes first.

### 2. Eval systems work best when they stay structured and representative

OpenAI's evaluation best-practices guidance frames evals as structured tests used to measure accuracy, performance, and reliability. The same guidance supports the direction Shuma is already taking: explicit typed metrics and bounded result artifacts instead of narrative interpretation.

Relevant source:

- [OpenAI evaluation best practices](https://developers.openai.com/api/docs/guides/evaluation-best-practices)

Implications:

1. Fleet or intelligence enrichment should stay typed and machine-readable.
2. Shared evidence should influence which scenarios or benchmark families deserve attention, not turn into free-form "operator lore."
3. The later code-evolution loop should still be judged by explicit benchmark results, not commentary about what the fleet is "seeing."

### 3. Agent inputs must stay structured and must not elevate untrusted data into privileged control

OpenAI's agent-safety guidance warns against passing untrusted inputs into privileged control surfaces. That maps directly onto Shuma's later controller and code-evolution loops: community or fleet intelligence may be useful, but it must be admitted only through typed, bounded, explicitly governed fields.

Relevant source:

- [OpenAI safety in building agents](https://developers.openai.com/api/docs/guides/agent-builder-safety)

Implications:

1. Shared-intelligence inputs should enter later controller loops only as structured metadata.
2. Free-form community or fleet narratives must not be injected into controller prompts as authoritative instructions.
3. Source, confidence, freshness, and review posture need to travel with enrichment inputs.

### 4. Progressive-delivery systems use weighted analysis, but also keep hard failure and inconclusive states

Google Cloud Deploy's verify step and Argo Rollouts' analysis model both reinforce a staged verify-and-decide loop. Spinnaker's canary-judgment model adds an important additional insight: weights can be useful, but muted metrics still remain visible, and critical-failure conditions can override aggregate scoring.

Relevant sources:

- [Cloud Deploy verify your deployment](https://cloud.google.com/deploy/docs/verify-deployment)
- [Argo Rollouts analysis](https://argoproj.github.io/argo-rollouts/features/analysis/)
- [Spinnaker canary judgment](https://spinnaker.io/docs/guides/user/canary/judge/)
- [Spinnaker canary best practices](https://spinnaker.io/docs/guides/user/canary/best-practices/)

Implications:

1. Shuma can use bounded benchmark-family weighting later, but only with explicit caps and visibility.
2. Human-friction or safety-critical benchmark families should retain veto or blocker semantics even when weights shift.
3. "Inconclusive" or "observe longer" remains an important controller-grade state when evidence is weak or mixed.

### 5. Shared intelligence is most useful as advisory, source-scoped, and freshness-scoped input

The earlier Shuma research on recidive and central intelligence already established the right distinction: local truth and local bans are one thing, advisory or high-confidence shared intelligence is another. That earlier work aligns well with the benchmark problem here.

Relevant source:

- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md`](./2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-research-synthesis.md)

Implications:

1. Central intelligence should influence benchmark emphasis as advisory input first.
2. Benchmark enrichment needs the same provenance fields as intelligence itself: source, confidence, freshness, scope, and governance posture.
3. Fleet or shared intelligence must not silently create or erase local benchmark outcomes.

## Research Synthesis

Taken together, the evidence suggests one clean model:

1. `benchmark_suite_v1` stays the static local benchmark registry.
2. `benchmark_results_v1` stays the bounded local current-instance benchmark result contract.
3. Later fleet or central-intelligence evidence should arrive as a separate enrichment layer.
4. That enrichment layer may influence:
   - which scenario families Shuma should run or emphasize,
   - which benchmark families deserve higher review priority,
   - and which benchmark families receive a small bounded weight bias in later aggregate judgment.
5. That enrichment layer must not:
   - redefine local benchmark statuses,
   - bypass local human-friction or policy-safety budgets,
   - or become the transport for live fleet data via the Git repository.

## Recommended Enrichment Dimensions

### 1. Scenario-selection hints

Shared evidence should be able to say:

1. this scenario family is becoming more prevalent,
2. this adversary pattern is newly important,
3. or this beneficial-agent posture needs explicit benchmark coverage.

The output should be a hint to run, keep, or elevate named scenario families, not a free-form instruction.

### 2. Family-priority hints

Shared evidence should be able to say:

1. suspicious-origin cost now deserves more attention,
2. beneficial non-human posture deserves more attention because verified-agent traffic is rising,
3. or adversary-sim effectiveness should be elevated because a relevant attack family is changing quickly.

This affects ordering, alerting salience, or review cadence, not truth itself.

### 3. Bounded weight bias

Shared evidence may justify small bounded changes to how later aggregate benchmark judgments rank families, but only under strict rules:

1. weight bias is advisory and capped,
2. critical safety families still retain blocker semantics,
3. and the original family statuses remain visible and unchanged.

## Recommended Guardrails

1. Local objective budgets remain authoritative for current-instance tuning.
2. `likely_human_friction` must not be down-weighted below a hard safety floor because of fleet pressure.
3. Local policy stance must remain authoritative for `beneficial_non_human_posture`.
4. Shared-intelligence enrichment should never directly emit `config_tuning_candidate` or `code_evolution_candidate` by itself; it only enriches later aggregate judgment.
5. Git may store static benchmark definitions and planning docs, but it must not be the live transport or system of record for dynamic fleet benchmark state.

## Recommended Source Classes

Later enrichment should distinguish at least these source scopes:

1. `site_local_history`
2. `fleet_local`
3. `curated_intelligence`
4. `community_advisory`

Every enrichment record should carry:

1. `source_id`
2. `source_scope`
3. `confidence_class`
4. `freshness_ts`
5. `expires_at`
6. `review_required`
7. `evidence_basis`

## Recommended Architectural Consequence

Shuma should later define a separate `benchmark_enrichment_v1` contract rather than overloading either `benchmark_suite_v1` or `benchmark_results_v1`.

That contract should remain:

1. optional,
2. advisory-first,
3. bounded,
4. and tied to later central-intelligence architecture or a dedicated control plane,

while the local benchmark contracts continue to define current-instance truth.

## Recommended Next Step

Capture the fleet and intelligence enrichment contract as a dedicated design note now, so later central-intelligence architecture, controller planning, and code-evolution planning all inherit the same rules:

1. enrich benchmark emphasis,
2. do not redefine benchmark truth,
3. preserve safety vetoes,
4. and keep dynamic shared state out of the Git repository.
