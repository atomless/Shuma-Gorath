# Agentic-Era Oversight Design

Date: 2026-03-15
Status: Proposed

Related context:
- [`docs/research/2026-03-15-agentic-era-oversight-research-synthesis.md`](../research/2026-03-15-agentic-era-oversight-research-synthesis.md)
- [`docs/project-principles.md`](../project-principles.md)
- [`docs/module-boundaries.md`](../module-boundaries.md)
- [`docs/adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`docs/plans/2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md`](2026-03-01-adversary-sim-autonomous-heartbeat-implementation-plan.md)
- [`docs/plans/2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](2026-03-12-unified-telemetry-hot-read-architecture-plan.md)

## Objectives

1. Make Shuma fit the agentic era by distinguishing beneficial authenticated agent traffic from hostile or undeclared automation.
2. Reduce human admin burden by shifting routine review and low-risk tuning into a bounded oversight control loop.
3. Reduce host-site bandwidth, CPU, and origin work spent on suspicious automation.
4. Preserve or improve human UX and conversion-critical behavior by making human-friction budgets explicit and enforceable.
5. Keep all autonomous behavior auditable, reversible, and outside the request hot path.

## Non-goals

1. Running LLMs inside the request path.
2. Allowing autonomous production changes to auth, trust boundaries, or provider-backend selection.
3. Treating robots directives as enforcement.
4. Building a hosted multi-tenant Shuma control plane in the first tranche.
5. Replacing current adversary-sim or monitoring contracts instead of building on them.
6. Making an in-process app scheduler the primary production architecture.

## Current Baseline

Shuma already has:

1. backend-owned adversary-sim lifecycle and control-plane discipline,
2. hot-read monitoring documents and freshness/query-budget surfaces,
3. config validation and config patch APIs,
4. robots and AI-policy outputs,
5. a strong project principle set around low friction for humans and high cost for adversaries.

The missing capability is a backend-owned oversight controller that can:

1. read live state through one bounded budget snapshot,
2. decide whether the site is inside or outside desired operating budgets,
3. propose or apply small policy changes,
4. verify those changes with adversary evidence,
5. roll back regressions automatically,
6. and leave a durable audit trail.

## Architectural Model

Shuma should be treated as three coordinated planes.

### 1. Request Plane

This remains the current Rust request pipeline:

1. collect signals,
2. produce policy decisions,
3. execute effects,
4. render responses.

Requirements:

1. no LLM dependency,
2. no scheduler dependency,
3. no background mutation requirement,
4. existing boundary and capability rules remain intact.

### 2. Evidence Plane

This plane provides bounded, operator-truthful state for both humans and agents:

1. monitoring summaries,
2. recent event tails,
3. retention health,
4. security/privacy posture,
5. adversary-sim status and history,
6. future oversight budget snapshot and decision ledger.

Requirements:

1. bounded read cost,
2. explicit freshness semantics,
3. explicit exact vs best-effort labeling where needed,
4. no hidden write-on-read behavior.

### 3. Oversight Plane

This is the new backend-owned control loop:

1. scheduler triggers a reconcile cycle,
2. reconcile cycle loads the oversight budget snapshot,
3. deterministic rules decide whether a change is needed,
4. optional agent advisor proposes a bounded patch and rationale,
5. config patch is validated,
6. adversary checks or replay run,
7. approved patch is applied,
8. post-change watch window either confirms or rolls back,
9. decision and evidence are stored.

Requirements:

1. idempotent execution,
2. lease-safe single-writer behavior,
3. explicit mode gating,
4. bounded per-cycle blast radius,
5. durable auditability.

## Traffic Classification Model

Shuma should stop treating all automation as one policy class.

### Class A: Verified Beneficial Agents

Definition:

1. requests that present cryptographic bot identity or an equivalent verified-agent signal,
2. or traffic that can be mapped to a high-confidence authenticated agent contract.

Expected treatment:

1. low-friction allow path,
2. lower-cost content representation when available,
3. explicit observability and rate budgets,
4. no maze or tarpit by default unless behavior breaches declared policy.

### Class B: Declared Crawlers and Search Bots

Definition:

1. requests that identify as crawlers/search assistants through legacy means,
2. without strong cryptographic identity or verified-agent posture.

Expected treatment:

1. policy communication through robots and AI preference surfaces,
2. conservative cost budgets,
3. rate and scope constraints,
4. escalation when behavior diverges from declared posture.

### Class C: Unverified or Suspicious Automation

Definition:

1. undeclared automation,
2. spoofed or unverifiable bots,
3. automation that violates declared policy,
4. high-confidence malicious traffic.

Expected treatment:

1. early low-cost checks,
2. challenge and deception escalation,
3. bounded cost shifting through maze, tarpit, and related controls,
4. aggressive reduction of origin work and expensive response bytes.

## Oversight Control Contract

The oversight plane should use an explicit contract, not ad hoc automation.

### Operating Modes

1. `off`
   - no scheduled reconcile,
   - read-only status only.
2. `observe`
   - collect budget snapshots and decision candidates,
   - no proposals or writes.
3. `recommend`
   - generate bounded policy proposals and expected-impact notes,
   - do not persist config changes.
4. `canary_apply`
   - automatically apply low-risk changes within strict envelopes,
   - require watch-window confirmation and rollback support.
5. `autonomous`
   - same as `canary_apply`, but enabled for a broader low-risk family set only after tranche-specific signoff.

### Core Operations

1. `snapshot`
   - assemble one bounded budget document from monitoring, config, and adversary state.
2. `reconcile`
   - compare snapshot against desired budgets and produce a decision outcome.
3. `propose_patch`
   - generate a structured config patch for allowed keys only.
4. `validate_patch`
   - run the existing config validators before any write.
5. `exercise`
   - execute deterministic adversary checks or scenario replay required by the patch family.
6. `apply_patch`
   - persist one bounded config family change.
7. `watch`
   - observe the post-change budget window and confirm or roll back.
8. `rollback`
   - restore the last known-good snapshot when watch-window budgets are breached.
9. `record_decision`
   - append the proposal, validation result, evidence, and outcome to a durable ledger.

### Proposal Output Contract

The controller should behave like a reconciler, not a chatty analyst.

Every non-noop proposal should be structured and machine-consumable:

1. `patch_family`
2. `patch`
3. `expected_impact`
4. `confidence`
5. `required_verification`
6. `rollback_window`
7. `operator_notes`

Non-goal for this contract:

1. prose-only advice with no patch body,
2. free-form agent text standing in for a typed decision.

### Control-Plane Guarantees

1. One reconcile writer per site at a time.
2. Lease and fencing semantics follow the existing adversary-sim control pattern.
3. Every applied patch must record:
   - previous values,
   - new values,
   - reason,
   - expected budget impact,
   - evidence references,
   - rollback trigger window.
4. One config family per reconcile cycle.
5. Max patch count per day is capped by policy.

## Budget Schema

The controller needs one canonical schema that can be materialized into a hot-read document.

### Envelope

```json
{
  "schema_version": "oversight-budget.v1",
  "site_id": "default",
  "deployment_profile": "shared_host|gateway|edge",
  "operating_mode": "off|observe|recommend|canary_apply|autonomous",
  "generated_at_ts": 0,
  "site_profile": "docs|marketing|ecommerce|application|custom",
  "budgets": {}
}
```

### Budget Families

#### 1. Human Friction Budget

Purpose:

1. cap the human cost of defense.

Initial fields:

1. `max_human_path_challenge_rate_pct`
2. `max_human_path_block_rate_pct`
3. `max_added_latency_p95_ms`
4. `max_added_latency_p99_ms`
5. `max_conversion_path_intervention_rate_pct`
6. `max_verified_human_reauth_rate_pct`

Phase-1 note:

1. some of these will start as proxy metrics rather than perfect human ground truth.

#### 2. Suspicious-Traffic Cost Budget

Purpose:

1. cap what the defended site spends serving suspicious automation.

Initial fields:

1. `max_suspicious_allow_requests_per_hour`
2. `max_suspicious_origin_forward_bytes_per_hour`
3. `max_suspicious_response_bytes_per_hour`
4. `max_suspicious_cpu_cost_units_per_hour`
5. `min_high_confidence_cost_shift_rate_pct`
6. `max_unverified_bot_success_rate_pct`

#### 3. Verified-Agent Budget

Purpose:

1. protect beneficial agentic traffic and make it cheaper than generic browsing.

Initial fields:

1. `min_verified_agent_allow_rate_pct`
2. `max_verified_agent_challenge_rate_pct`
3. `max_verified_agent_added_latency_p95_ms`
4. `min_low_cost_representation_share_pct`
5. `max_verified_agent_origin_bytes_per_request`

#### 4. Telemetry Truth Budget

Purpose:

1. prevent the oversight loop from operating on stale or misleading evidence.

Initial fields:

1. `required_freshness_state`
2. `max_monitoring_lag_ms`
3. `required_query_budget_status`
4. `required_retention_health_state`
5. `max_unsampleable_event_drop_count`
6. `max_secret_canary_leak_count`

#### 5. Simulation Budget

Purpose:

1. ensure the controller remains red-teamed and evidence-backed.

Initial fields:

1. `min_daily_deterministic_runs`
2. `min_required_coverage_categories`
3. `max_live_sim_runtime_seconds_per_day`
4. `max_live_sim_cpu_budget_units_per_day`
5. `required_latest_run_state`

#### 6. Change Safety Budget

Purpose:

1. bound autonomous change behavior itself.

Initial fields:

1. `max_config_families_changed_per_day`
2. `max_single_change_delta_pct`
3. `cooldown_seconds_between_changes`
4. `watch_window_seconds`
5. `rollback_trigger_breach_count`
6. `requires_validation`
7. `requires_replay_before_apply`

### Initial Canonical Site Budget Keys

For the first implementation tranche, the controller should expose a short canonical key set that directly answers the question "is this site inside budget?":

1. `human_challenge_rate`
2. `human_p95_added_latency`
3. `suspicious_bytes_served`
4. `suspicious_cpu_cost`
5. `verified_agent_success_rate`
6. `monitoring_freshness`
7. `retention_health`
8. `unsampleable_event_drop_count`

These should be treated as the operator-facing summary layer over the richer budget-family internals above.

## Auto-Tunable vs Manual-Only Surfaces

### Auto-Tunable First

These are the best initial candidates because they are bounded, already operator-tuned today, and reversible:

1. rate limits,
2. ban durations,
3. botness weights,
4. challenge thresholds,
5. maze/tarpit thresholds and bounded resource controls,
6. low-risk defense modes where semantics are already explicit.

### Manual-Only Until Later

These should remain outside autonomous mutation in early phases:

1. auth and session controls,
2. trust-boundary and forwarded-header secrets,
3. provider backend selection,
4. fail-open / fail-closed posture,
5. retention windows,
6. path allowlists and emergency allowlists,
7. broad GEO deny lists,
8. robots/search policy that could materially affect SEO or external discovery posture.

## Scheduler and Deployment Model

The product architecture should not be tied to one scheduler.

### Near-Term Supported Adapters

1. host-side supervisor or timer for self-hosted and shared-host deployments,
2. Fermyon cron-triggered reconcile calls for edge deployments,
3. Kubernetes `CronJob` or equivalent for orchestrated deployments.

This is the best near-term approach because it matches Shuma's current adversary-sim heartbeat model and keeps scheduler ownership outside the request path.

### Long-Term Target

1. a Shuma-hosted control plane with per-site reconcile workers that owns schedules, decision ledgers, fleet learning, and model-provider access,
2. while tenant data planes continue to own request-path enforcement.

This is the best long-term product architecture because scheduling, policy audit, model access, and fleet learning belong outside tenant data planes.

### Rejected as Primary Architecture

1. An in-process scheduler inside the app is acceptable for local labs and experiments.
2. It should not be the primary production architecture because it muddies isolation boundaries, complicates model containment, and makes day-2 operations harder to reason about.

Recommendation:

1. standardize the internal reconcile contract first,
2. allow multiple scheduler adapters to call it,
3. defer hosted multi-tenant extraction until the contract is already proven on per-deployment schedulers.

## Low-Cost Beneficial-Agent Handling

Shuma should not only get better at rejecting bots. It should also get better at cheaply serving trusted automation.

Design direction:

1. keep current robots and Content-Signal surfaces,
2. add verified-agent identity handling,
3. add low-cost representations for eligible agents,
4. measure verified-agent success separately from suspicious automation.

This is how Shuma can protect host conversion and UX while still being "bot defense for the agentic era."

## Rollout Stages

### Stage 0: Design and Observe

1. Add the oversight budget snapshot and status surfaces.
2. Record decision candidates only.
3. No autonomous writes.

### Stage 1: Recommend

1. Produce bounded proposals and expected-impact notes.
2. Require explicit human approval for writes.

### Stage 2: Canary Apply

1. Autonomously apply low-risk config families only.
2. Enforce watch windows and rollback.

### Stage 3: Autonomous Bounded

1. Expand auto-tunable families only after evidence shows stable rollback and red-team coverage.
2. Keep high-risk families manual-only.

### Stage 4: Hosted Control Plane

1. Centralize schedule ownership, policy packs, model-provider integration, and fleet-level learning.
2. Keep request-path enforcement local and deterministic.

## Schedule Tiers

Once the controller exists, the default operating cadence should be tiered instead of monolithic:

1. Every 5 minutes:
   - deterministic budget sweep,
   - no more than one low-risk decision per cycle.
2. Hourly:
   - adversary-sim smoke or focused replay for recently touched config families.
3. Daily:
   - agent review and bounded config adjustment cycle.
4. Weekly:
   - frontier/red-team corpus expansion and promotion review into deterministic scenarios.

These tiers should be adapter-owned schedules calling one shared reconcile contract, not separate controller implementations.

## First-Autonomy Rule

The first autonomous version should explicitly be:

1. LLM advisor,
2. deterministic applier.

That means:

1. the agent can reason over the budget snapshot and produce a bounded proposal,
2. but Shuma validates, applies, watches, and rolls back through typed server-owned control paths.

## Security and Operational Requirements

1. The oversight plane must never weaken auth or trust boundaries.
2. Every control endpoint must be idempotent, lease-safe, and audited.
3. Telemetry truthfulness is a prerequisite for autonomous mutation.
4. The reconciler must fail closed to "no change" when evidence is stale, degraded, or contradictory.
5. The LLM advisor, when enabled, must output structured data only and remain outside the direct request path.

## Definition of Done for the First Real Tranche

The first production-meaningful oversight tranche is complete only when all of the following are true:

1. a bounded oversight budget snapshot exists and is cheap to read,
2. a backend-owned reconcile contract exists with lease/idempotency/audit guarantees,
3. observe and recommend modes are live,
4. canary-apply works for at least one low-risk config family with rollback,
5. adversary verification is part of the apply path,
6. operators can inspect why the system changed or refused to change,
7. request-path logic remains deterministic and agent-free.
