Date: 2026-03-18
Status: Settled and implemented foundation contract

Related context:

- [`2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md`](./2026-03-18-monitoring-telemetry-foundations-architectural-necessities.md)
- [`2026-03-18-monitoring-traffic-lane-and-denominator-contract.md`](./2026-03-18-monitoring-traffic-lane-and-denominator-contract.md)
- [`2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](./2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/policy_pipeline.rs`](../../src/runtime/policy_pipeline.rs)
- [`../../src/runtime/effect_intents/intent_types.rs`](../../src/runtime/effect_intents/intent_types.rs)
- [`../../src/runtime/effect_intents/intent_executor.rs`](../../src/runtime/effect_intents/intent_executor.rs)
- [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs)
- [`../../src/runtime/upstream_proxy.rs`](../../src/runtime/upstream_proxy.rs)
- [`../../src/runtime/capabilities.rs`](../../src/runtime/capabilities.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Define the concrete contract for the second telemetry-foundation prerequisite:

1. one authoritative request-outcome telemetry hook,
2. the structured outcome object that feeds it,
3. the boundary between response rendering and outcome emission,
4. and the rule that coarse request, byte, and funnel accounting must be emitted from that single canonical point rather than scattered across response branches.

This note is a design contract for `MON-TEL-1-2` and part of `MON-TEL-1-4`. It is not yet the implementation plan.

# Objectives

1. Give Shuma one canonical owner for coarse request-outcome telemetry.
2. Make forwarded-versus-short-circuited counts truthful across all major runtime branches.
3. Make response-byte accounting truthful enough for operator cost summaries without adding a new analytics subsystem.
4. Keep shadow-mode semantics explicit by recording both actual rendered outcome and intended action when they differ.
5. Fit the hook inside the existing request orchestration and capability model.

# Non-goals

1. Recording every fine-grained subsystem event from one mega-hook.
2. Replacing the existing event log or defence-specific counters.
3. Introducing a second parallel telemetry pipeline outside runtime orchestration.
4. Adding high-cardinality per-path or per-IP byte accounting.
5. Pretending an intended shadow action is the same thing as the actual rendered response.

# Current Shortfall

Shuma already records some request consequences, but not from one stable finalization point.

Today:

1. forwarded allow metrics are recorded in [`response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs),
2. challenge, maze, redirect, block, tarpit, and other responses are emitted through separate `ResponseIntent` branches,
3. `request_flow.rs` still has direct early returns for HTTPS rejection, early routes, static bypass, checkpoint, tarpit progress, maze traversal, sim public, and clean forwarded allow,
4. post-response processing only flushes buffered monitoring counters and is not a general request-outcome finalizer.

This means the current codebase does not yet have one place where Shuma can truthfully say, once per request:

1. what scope this request belongs to,
2. what final response class was actually rendered,
3. whether the result was forwarded or short-circuited,
4. how many response bytes were actually served,
5. and, in shadow mode, what would have happened versus what was actually served.

# Architectural Position

## Request flow must own final outcome emission

The authoritative request-outcome hook should live at the request-orchestration level, not inside individual response branches.

Recommended owner:

1. `request_flow.rs` finalization logic

Reason:

1. `request_flow.rs` sees all terminal request paths, including the branches that currently bypass `effect_intents`,
2. the response renderer only sees a subset of terminal paths,
3. and the final hook must be able to classify scope, origin, and final response across both policy-driven and non-policy branches.

## Response rendering should produce structured outcome evidence, not emit coarse counters directly

The response-rendering and intent-execution layers should continue to decide and render responses, but they should stop being the implicit owner of coarse request-outcome telemetry.

Instead they should supply structured outcome evidence back to request-flow finalization.

That keeps ownership clean:

1. response code renders,
2. request flow finalizes,
3. the outcome hook emits coarse telemetry once.

# Canonical Outcome Object

The canonical object should be a typed runtime-owned value, tentatively:

1. `RenderedRequestOutcome`
2. `RequestOutcomeTelemetry`
3. `RequestOutcomeRecord`

Recommended fields:

1. `response`
   - the final `Response`
2. `traffic_origin`
   - from the lane contract
   - `live` or `adversary_sim`
3. `measurement_scope`
   - from the lane contract
   - for example `ingress_primary`, `defence_followup`, `bypass_and_control`, `excluded`
4. `route_action_family`
   - coarse family from the lane contract
5. `execution_mode`
   - `enforced` or `shadow`
6. `traffic_lane`
   - optional until lane derivation is implemented in the telemetry tranche
7. `outcome_class`
   - coarse operator-facing outcome classification
8. `response_kind`
   - more specific rendered response type
9. `http_status`
10. `response_bytes`
   - exact buffered response length where available
11. `forward_attempted`
12. `forward_failure_class`
   - optional
13. `intended_action`
   - optional for shadow mode
14. `policy_source`
   - optional coarse source such as `clean_allow`, `policy_graph_first_tranche`, `policy_graph_second_tranche`, `early_route`, `sim_public`

The crucial rule is that this object represents the actual final rendered outcome, not just the planned intent.

# Outcome Taxonomy

## `outcome_class`

This should stay intentionally coarse:

1. `forwarded`
2. `short_circuited`
3. `control_response`

`forwarded`

Meaning:

1. the request was sent to origin and a canonicalized upstream response was returned

`short_circuited`

Meaning:

1. Shuma served the terminal response locally rather than forwarding to origin

`control_response`

Meaning:

1. internal or operational responses outside the primary site-traffic story

This coarse class supports the main operator summaries and suspicious-cost accounting.

## `response_kind`

This should carry the more useful rendered subtype:

1. `forward_allow`
2. `forward_failure_fallback`
3. `block_page`
4. `plain_text_block`
5. `redirect`
6. `drop_connection`
7. `challenge`
8. `not_a_bot`
9. `js_challenge`
10. `maze`
11. `tarpit`
12. `synthetic_shadow_allow`
13. `synthetic_shadow_action`
14. `checkpoint_response`
15. `control_plane_response`

This distinction is important because:

1. funnels need more than just forwarded versus short-circuited,
2. but top-level cost summaries should not depend on dozens of bespoke event strings.

# Shadow-Mode Rule

Shadow mode is where this hook becomes especially valuable.

The hook must record two truths at once:

1. what was actually rendered,
2. what Shuma intended to do under enforcement.

Examples:

1. synthetic shadow body that says "would challenge"
   - actual `response_kind`: `synthetic_shadow_action`
   - `intended_action`: `challenge`
   - `outcome_class`: `short_circuited`
2. shadow passthrough with upstream forwarding available
   - actual `response_kind`: `forward_allow`
   - `intended_action`: maybe `maze`, `block`, or `challenge`
   - `outcome_class`: `forwarded`

The hook must not collapse those into a single pseudo-fact.

That distinction is necessary for:

1. truthful operator monitoring,
2. future shadow-only summaries,
3. and later controller reasoning.

# Ownership By Runtime Layer

## `request_flow.rs`

Responsibility:

1. determine `traffic_origin`, `measurement_scope`, and `route_action_family`
2. ensure every terminal request path produces a `RenderedRequestOutcome`
3. call the one canonical `record_request_outcome(...)` hook before returning the final response

This is the sole owner of coarse request-outcome telemetry emission.

## `policy_pipeline.rs`

Responsibility:

1. keep building request facts and decisions
2. pass through structured outcome context when a policy-driven path produces a terminal response
3. not emit coarse forwarded or byte counters directly

## `effect_intents` and `response_renderer`

Responsibility:

1. render the actual response
2. annotate outcome evidence needed by finalization
3. preserve existing subsystem metrics and event-log behavior where still appropriate
4. stop being the de facto owner of coarse final-outcome accounting

## `upstream_proxy.rs`

Responsibility:

1. keep owning forwarding mechanics
2. surface forwarding success or failure class and final response
3. expose enough information for finalization to compute `forward_attempted`, `forward_failure_class`, and `response_bytes`

If necessary, `ForwardResult` should be extended to carry exact response-byte length explicitly rather than requiring later inference.

# Relationship To The Capability Model

This design should not invent a parallel telemetry side path.

Recommended rule:

1. the final outcome hook should consume the same runtime-owned capabilities already minted at the request boundary,
2. or a narrow wrapper derived from them,
3. but it must remain inside the capability lattice rather than bypassing it.

Practical implication:

1. no new ad hoc store writes from random branches,
2. no dashboard-oriented writes directly from UI or API code,
3. and no post-hoc reconstruction path pretending to be authoritative.

If a narrow wrapper is introduced, it should be a typed facade around the existing metrics and monitoring capabilities, not a new free-form write privilege.

## Buffered emission rule

This hook must not translate into one fresh KV mutation per request for every new counter family.

Required write-path rule:

1. coarse outcome counters must enter the same buffered monitoring write discipline Shuma already uses today,
2. or an explicitly equivalent batched delta path with the same or better write-amplification profile,
3. and hot-read refresh must continue to happen from buffered-flush or rebuild boundaries rather than from per-request projection mutation.

This is a hard architectural constraint, not an optimization idea. The hook is about canonical ownership of interpretation, not permission to bypass the telemetry-efficiency work.

## Single-emission invariant

Implementation must also preserve a strict single-emission rule:

1. each terminal request may produce at most one coarse request-outcome emission,
2. response branches may annotate or enrich the outcome object,
3. but they must not emit parallel coarse forwarded, byte, or funnel counters on their own.

# Coverage Requirements Across Request Paths

The final hook must cover all terminal branches that matter to operator summaries.

## Included in the canonical hook

1. clean forwarded allow
2. policy-driven challenge, not-a-bot, JS challenge, maze, redirect, drop, tarpit, and block responses
3. forward-failure fallback responses
4. static bypass responses
5. allowlist bypass responses
6. sim public responses that currently exist as fallback content-surface helpers for adversary-sim/demo scenarios
7. checkpoint and tarpit-progress responses where subsystem follow-up accounting is required

## Explicit exclusions or reduced accounting

1. admin and control-plane endpoints
2. metrics and health endpoints
3. fingerprint report endpoints
4. bootstrap failures where the store or config is unavailable before Shuma can perform meaningful monitored classification

These may still return a `RenderedRequestOutcome`, but they should resolve to `measurement_scope=excluded` or `bypass_and_control` so they do not pollute primary operator counters.

Important clarification:

1. fallback `/sim/public/*` pages are not a justification for treating adversary-generated traffic as likely-human or clean live ingress,
2. the critical split is `traffic_origin`, not dummy-surface versus real-surface path provenance,
3. so live-only inference signals must not be emitted for adversary-sim-origin requests even when the request would otherwise look like a clean allow.

# Important Implementation Consequence

The lane and denominator contract treats `static_asset` traffic as part of the primary ingress story.

Today, static bypass returns before store open in [`request_flow.rs`](../../src/runtime/request_flow.rs).

Therefore `MON-TEL-1-2` must not pick a static-bypass accounting strategy implicitly in code.

Before implementation, the plan must choose one of these explicitly:

1. keep static traffic in the primary ingress story and prove a telemetry-safe low-cost path,
2. keep static traffic visible only through a supporting or best-effort summary path,
3. or revise the denominator contract to exclude static bypass from exact primary ingress accounting for now.

The important requirement is explicit choice and proof. The wrong move here would be to silently drag store or config work into the static fast path and only discover the regression later.

# Byte Accounting Rule

The hook should use exact buffered response body length where Shuma actually has it.

Recommended source of truth:

1. `response.body().len()`

Use this for:

1. forwarded canonicalized responses
2. locally rendered challenge and block pages
3. maze and tarpit responses
4. synthetic shadow responses

Do not start with:

1. header-trusted `Content-Length`
2. path-level byte attribution
3. streaming or chunk-phase accounting across many branches

The phase-1 requirement is bounded, truthful, coarse byte accounting, not perfect packet accounting.

# Relationship To Existing Telemetry

This hook does not replace:

1. defence-specific counters
2. event-log rows
3. shadow event metadata
4. forward latency metrics

It complements them by adding the missing coarse finalization layer those existing signals cannot safely provide on their own.

# Impact On `MON-TEL-1`

## `MON-TEL-1-2`

Must use this hook as the sole owner of:

1. total requests seen by scope
2. forwarded versus short-circuited totals
3. response-byte totals

## `MON-TEL-1-4`

Must reuse the same outcome taxonomy so defence-funnel summaries are built from one shared interpretation of final outcomes instead of bespoke per-module logic.

# Definition Of Done

This prerequisite is complete when:

1. every relevant terminal request path yields one structured outcome object,
2. `request_flow` owns the one authoritative coarse outcome emission hook,
3. forwarded-versus-short-circuited totals come only from that hook,
4. shadow mode records actual rendered outcome and intended action separately,
5. response-byte accounting comes from one shared rule,
6. and the later telemetry tranche can add counters and funnels without sprinkling new branch-local counter writes across the runtime.
