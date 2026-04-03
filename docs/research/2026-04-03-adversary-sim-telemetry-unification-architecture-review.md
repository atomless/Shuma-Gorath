Date: 2026-04-03
Status: Active review

Related context:

- [`2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](./2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`2026-04-02-agentic-recent-run-coverage-gap-review.md`](./2026-04-02-agentic-recent-run-coverage-gap-review.md)
- [`../plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](../plans/2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`../plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](../plans/2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- [`../../src/observability/llm_surface_observation.rs`](../../src/observability/llm_surface_observation.rs)
- [`../../src/observability/scrapling_owned_surface.rs`](../../src/observability/scrapling_owned_surface.rs)
- [`../../scripts/run_with_adversary_sim_supervisor.sh`](../../scripts/run_with_adversary_sim_supervisor.sh)
- [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py)

# Adversary Sim Telemetry Unification Architecture Review

## Question

How should Shuma collect and present adversary-sim telemetry so that:

1. traffic events,
2. surfaces reached,
3. defence interactions,
4. and outcomes

come from the same observed telemetry spine used for real traffic, while still preserving the privileged simulator-only facts that real traffic cannot supply?

## Executive Summary

Three findings are now clear.

### 1. Shuma already has the right shared traffic-observation spine

The runtime already has a canonical request-outcome path:

1. [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs) builds traffic-origin context and always finalizes through one `RenderedRequestOutcome`,
2. [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs) defines that outcome shape for all traffic origins,
3. [`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs) provides signed sim tags so simulation requests can still be attributed without inventing a second ingress channel,
4. and [`../../src/admin/api.rs`](../../src/admin/api.rs) already consumes bounded monitoring/event records as the hot-read source of truth.

That means the architecture Shuma wants is not hypothetical. The repo already owns the main spine.

### 2. Recent adversary-sim presentation drifted into a hybrid evidence model

The current `Recent Red Team Runs` path mixes two evidence classes:

1. shared observed monitoring events for `monitoring_event_count` and `defense_delta_count`,
2. Scrapling-owned surface receipts for `owned_surface_coverage`,
3. LLM runtime receipts for `llm_surface_coverage`,
4. realism receipts for identity and transport,
5. and runtime lineage receipts for execution/provider detail.

That is why an Agentic row can currently show surface coverage while also showing `0 events · 0 defenses`: the row is combining receipt-backed coverage with monitoring-backed deltas from a stricter evidence class.

### 3. The next step must be architectural unification, not more receipt-shaped table patches

The user requirement is correct:

1. traffic behavior and penetration truth should be gathered by Shuma's generic telemetry path regardless of whether the traffic is real, Scrapling, or Agentic,
2. while simulator-only privileged metadata such as fulfillment modes, target categories, provider lineage, and realism envelopes should remain explicit sidecars.

The right model is therefore:

1. one shared observed-traffic evidence layer for request/outcome/surface/defence truth,
2. filtered views by traffic origin where needed,
3. and sim-only sidecars only for privileged attacker-internal facts.

## Current Architecture: What Is Already Shared

### Canonical request-outcome emission already exists

`handle_request(...)` in [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs) routes all requests through `finalize_request_outcome(...)`, which emits one `EffectIntent::RecordRequestOutcome`. That is the right collection seam for shared observed traffic truth.

Important consequence:

1. unification does not require a second telemetry store,
2. and it does not require simulation traffic to bypass Shuma's real monitoring model.

### Sim traffic already has shared attribution headers

[`../../src/runtime/sim_telemetry.rs`](../../src/runtime/sim_telemetry.rs) validates signed request headers and materializes `SimulationRequestMetadata`.

This is the correct place for simulation-specific attribution:

1. shared traffic collection can remain generic,
2. while `sim_run_id`, `sim_profile`, and `sim_lane` remain additive tags on the same observed request stream.

### Shared collection does not mean every summary must mix sim and real traffic

The architectural goal is shared collection, not identical dashboards.

Operator views can still:

1. filter real-only traffic,
2. filter sim-only traffic,
3. or compare the two,

as long as they are reading from one collected evidence spine rather than from separate lane-specific surrogate models.

## Where The Split Exists Today

### 1. Sim clean-allow monitoring is still explicitly suppressed

`clean_allow_monitoring_intents(...)` in [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs) returns an empty vector for `TrafficOrigin::AdversarySim`.

That means:

1. some shared monitoring intents are intentionally skipped for sim-origin traffic,
2. which may be appropriate for likely-human denominator sampling,
3. but is too coarse if the repo also expects sim traffic to produce the same observed allow-side evidence and downstream operator truth as real traffic.

The likely correct end state is not "force every live-traffic summary to include sim traffic." It is:

1. collect shared observed traffic truth for sim-origin requests too,
2. but keep summary filters and denominator rules origin-aware.

### 2. Recent-run aggregation currently promotes receipt evidence into traffic truth

`monitoring_recent_sim_run_summaries_filtered(...)` in [`../../src/admin/api.rs`](../../src/admin/api.rs) currently:

1. increments `monitoring_event_count` and `defense_delta_count` only for non-receipt external monitoring events,
2. accumulates `scrapling_activity_count` from Scrapling realism receipts,
3. builds `owned_surface_coverage` from Scrapling surface receipts,
4. builds `llm_surface_coverage` from LLM runtime receipt-derived observations,
5. and preserves the latest realism receipts for identity and transport.

This is the exact hybrid seam.

### 3. Hot-read recent-run shape now encodes the hybrid directly

[`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs) currently carries both:

1. shared-monitoring counts such as `monitoring_event_count` and `defense_delta_count`,
2. and lane-shaped coverage fields such as `owned_surface_coverage` and `llm_surface_coverage`.

That shape is what makes the dashboard look internally contradictory even when each individual field is truthful to its own evidence class.

### 4. The narrow Agentic coverage follow-on solved presentation pressure, not the root contract

The 2026-04-02 follow-on correctly observed that Agentic runs already had surface evidence and that the shared recent-run shape was Scrapling-shaped.

But the narrow fix direction there still kept:

1. `Coverage` receipt-backed for Agentic rows,
2. `Monitoring Deltas` monitoring-backed,
3. and the mixed evidence model intact.

That is now superseded by the stronger architectural requirement in this review.

## What Must Stay Sim-Specific

The user requirement is not "remove all sim-specific telemetry." Some facts are genuinely privileged and should remain sidecars because Shuma cannot infer them from ordinary inbound traffic alone.

Those include:

1. fulfillment modes used,
2. category targets and fulfilled categories,
3. generation source,
4. configured frontier provider/model lineage,
5. realism envelope details such as identity pool basis or degraded transport reason,
6. and worker-internal activity planning or targeting strategy.

These are valid simulator sidecars because:

1. they enrich explanation and calibration,
2. they help evaluate Shuma's classification and representativeness,
3. but they are not substitutes for observed traffic events, surfaces, or outcomes.

## Recommended Architecture

### Principle 1: One shared observed-traffic evidence layer

Anything Shuma can observe from inbound traffic should come from one shared runtime/event telemetry path for:

1. request outcomes,
2. defence interactions,
3. surface contact,
4. progression or denial outcomes,
5. and defence deltas.

This must hold regardless of whether the traffic origin is:

1. real traffic,
2. Scrapling sim traffic,
3. Agentic sim traffic,
4. or later mixed lanes.

### Principle 2: Sim-only receipts become additive sidecars

Sim receipts should only own facts the shared request telemetry cannot know, such as:

1. which persona mode the simulator chose,
2. which provider generated a plan,
3. which categories it was intentionally trying to satisfy,
4. and realism-envelope provenance details.

They should no longer be the authoritative source for:

1. surfaces reached,
2. outcomes on those surfaces,
3. or whether Shuma observed a defence interaction at all.

### Principle 3: Shared collection, filtered summaries

To avoid polluting live operator denominators:

1. collect sim and real traffic through the same evidence path,
2. tag sim traffic with shared metadata,
3. and filter live-only or sim-only summaries at read time.

This keeps collection architecture unified without collapsing all analytical views together.

### Principle 4: Exactness must remain explicit

If any panel still needs to render privileged simulator-only detail next to shared observed traffic truth, exactness must be obvious.

Examples:

1. `Coverage`, `Monitoring Deltas`, defence penetrations, and outcome counts should be shared observed truth.
2. `Modes`, `Categories`, frontier provider lineage, and realism envelopes should be explicitly simulator-sidecar truth.

## Separate But Related Defect: Frontier Provider Env Parity

The local `no_configured_frontier_provider` symptom is not an operator mistake.

Observed evidence from the live local process tree shows:

1. `.env.local` contains `SHUMA_FRONTIER_OPENAI_API_KEY`, `SHUMA_FRONTIER_ANTHROPIC_API_KEY`, and the corresponding model ids,
2. `make dev` passes those frontier values into `spin up` as `--env ...` bindings,
3. but [`../../scripts/run_with_adversary_sim_supervisor.sh`](../../scripts/run_with_adversary_sim_supervisor.sh) launches the host-side supervisor with only a narrow inline env set (`SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL`, exit flags, and related supervisor vars),
4. and [`../../scripts/supervisor/llm_runtime_worker.py`](../../scripts/supervisor/llm_runtime_worker.py) calls `generate_llm_frontier_actions(...)`, which reads configured providers from `os.environ`.

So the likely defect is:

1. the Spin runtime sees the frontier keys,
2. the host-side LLM worker does not,
3. which causes the worker to fall back to `no_configured_frontier_provider` even on a correctly configured contributor machine.

This is a runtime-env parity bug, not a reason to keep the split telemetry model.

## Recommendation

Proceed with a new telemetry-unification chain that:

1. restores shared observed traffic truth as the source for surfaces, outcomes, and defence deltas,
2. preserves sim-only receipts only for privileged internal attacker metadata,
3. makes read-time filtering origin-aware instead of collection-time split by lane,
4. and fixes the host-worker frontier env parity seam as part of the same follow-on because it directly affects Agentic execution truth.

## Definition Of Done For The Review

This review is useful only if the follow-on implementation chain preserves the following contract:

1. if Shuma can observe a traffic interaction from the request/response/follow-up path, it must not require a lane-specific receipt parser to surface that interaction,
2. if a fact is simulator-internal, it must remain explicit as a sidecar rather than being silently blended into shared traffic truth,
3. live operator denominators must remain origin-filterable without requiring a second collection architecture,
4. and `Recent Red Team Runs` must stop combining receipt-backed penetration claims with zero shared observed deltas in the same semantic column set.
