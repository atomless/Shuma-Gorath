# Agentic-Era Oversight Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a backend-owned oversight controller that reads Shuma telemetry through a bounded budget snapshot, proposes or applies low-risk config changes, verifies them with adversary evidence, and rolls back regressions without putting agents in the request path.

**Architecture:** Keep runtime request handling deterministic in Rust. Build a new oversight control and reconcile layer in the admin/observability domains, reuse existing hot-read document and adversary control patterns, and stage rollout from observe-only to bounded autonomous apply. Keep scheduling adapter-specific code thin so self-hosted, Fermyon, and future hosted-control-plane deployments share one internal contract.

**Tech Stack:** Rust admin and observability modules, Svelte dashboard, Makefile verification, Spin/Fermyon scheduler adapters, repo-native docs and ADRs.

---

## Implementation Principles

1. Reuse the existing adversary-sim control pattern for leases, idempotency, and audit.
2. Reuse existing hot-read document infrastructure for oversight snapshots.
3. Keep phase 1 limited to observe and recommend modes.
4. Make every autonomous change reversible.
5. Keep the first UI slice minimal and consistent with the current dashboard surface.
6. Add Makefile targets before introducing any new canonical workflow.
7. Controller outputs must remain typed policy patches with expected impact and confidence, not prose-only recommendations.
8. The first autonomous version must remain "LLM advisor, deterministic applier."

## Phase 0: Contract Capture and Governance

### Task 0.1: Capture the architecture decision

**Files:**
- Create: `docs/adr/0011-agentic-era-oversight-controller.md`
- Modify: `docs/adr/README.md`
- Modify: `docs/README.md`

**Work:**
1. Record the three-plane model (`request`, `evidence`, `oversight`).
2. Record the traffic-class split (`verified beneficial agents`, `declared crawlers`, `unverified suspicious automation`).
3. Record the decision that request-path logic stays agent-free.
4. Record the rollout policy (`observe -> recommend -> canary_apply -> autonomous`).

**Acceptance criteria:**
1. The ADR makes the no-agent-in-request-path rule explicit.
2. The ADR names telemetry truthfulness as a hard prerequisite for change application.

### Task 0.2: Add execution-ready backlog items

**Files:**
- Modify: `todos/todo.md`

**Work:**
1. Add `OVR-*` items covering snapshot, controller, scheduler adapters, low-risk applier, rollback, dashboard visibility, and verified-agent follow-on work.
2. Group them by tranche so the work is execution-ready rather than one umbrella idea.

**Acceptance criteria:**
1. The backlog reflects the staged rollout in the design doc.
2. High-risk and long-horizon work is separated from phase-1 controller work.

## Phase 1: Oversight Snapshot and Status Surfaces

### Task 1.1: Define the oversight budget document contract

**Files:**
- Modify: `src/observability/hot_read_documents.rs`
- Modify: `src/observability/hot_read_contract.rs`
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/observability/mod.rs`
- Test: `src/observability/hot_read_documents.rs`
- Test: `src/observability/hot_read_contract.rs`
- Test: `src/observability/hot_read_projection.rs`

**Work:**
1. Add `oversight_budget_snapshot` document contract metadata.
2. Define bounded payload shape and freshness/rebuild budgets.
3. Mark each budget component as exact or best-effort using the existing contract language.

**Acceptance criteria:**
1. The snapshot is bounded and cheap to read.
2. Exactness metadata is explicit for every budget family.

### Task 1.2: Materialize the first oversight snapshot

**Files:**
- Modify: `src/observability/hot_read_projection.rs`
- Modify: `src/admin/api.rs`
- Test: `src/admin/api.rs`

**Work:**
1. Build the snapshot from existing monitoring, config, adversary-sim, and security/privacy surfaces.
2. Reuse existing hot-read projection refresh triggers where possible.
3. Add a read-only status endpoint for the oversight plane.

**Acceptance criteria:**
1. The oversight status endpoint does not mutate state.
2. The snapshot includes human-friction, suspicious-cost, telemetry-truth, and simulation budget families.

### Task 1.3: Document and expose the status contract

**Files:**
- Modify: `docs/api.md`
- Modify: `docs/dashboard.md`
- Modify: `docs/dashboard-tabs/status.md`

**Work:**
1. Document the new oversight status payload.
2. Make freshness and degraded-state semantics explicit.
3. Document "no change" behavior when telemetry truth budgets fail.

**Acceptance criteria:**
1. Operators can understand why oversight is inactive or refusing to act.

## Phase 2: Backend-Owned Oversight Control Plane

### Task 2.1: Add lease-safe oversight control primitives

**Files:**
- Create: `src/admin/oversight_control.rs`
- Modify: `src/admin/mod.rs`
- Modify: `src/admin/api.rs`
- Test: `src/admin/oversight_control.rs`
- Test: `src/admin/api.rs`

**Work:**
1. Model idempotency, leases, audit records, and operation records after `src/admin/adversary_sim_control.rs`.
2. Add explicit operating-mode transitions.
3. Keep control-plane mutation separate from reconcile execution.

**Acceptance criteria:**
1. Only one oversight writer can own a site at a time.
2. Replay, payload mismatch, and throttling behavior are explicit and tested.

### Task 2.2: Add read-only history and last-decision visibility

**Files:**
- Modify: `src/admin/api.rs`
- Test: `src/admin/api.rs`
- Modify: `docs/api.md`

**Work:**
1. Add a bounded history read surface for recent oversight decisions.
2. Add last-decision summary to the status payload.

**Acceptance criteria:**
1. Operators can see the last proposed or applied change and why it happened.

## Phase 3: Deterministic Reconciler and Safe Applier

### Task 3.1: Add the deterministic reconcile engine

**Files:**
- Create: `src/admin/oversight_reconcile.rs`
- Modify: `src/admin/api.rs`
- Test: `src/admin/oversight_reconcile.rs`
- Test: `src/admin/api.rs`

**Work:**
1. Encode low-risk budget-breach rules without LLM dependence.
2. Produce outcomes such as:
   - `within_budget`,
   - `observe_only`,
   - `recommend_patch`,
   - `apply_patch`,
   - `rollback_required`,
   - `refuse_change_due_to_stale_evidence`.
3. Keep one config family per reconcile cycle.
4. Define a typed reconcile output shape that includes:
   - `patch_family`,
   - `patch`,
   - `expected_impact`,
   - `confidence`,
   - `required_verification`,
   - `rollback_window`.

**Acceptance criteria:**
1. Reconcile logic is pure enough to unit test directly.
2. Stale or contradictory evidence results in no change.
3. No reconcile outcome relies on prose-only guidance.

### Task 3.2: Add config-family allowlist and patch safety envelopes

**Files:**
- Create: `src/admin/oversight_patch_policy.rs`
- Modify: `src/admin/api.rs`
- Test: `src/admin/oversight_patch_policy.rs`
- Test: `src/admin/api.rs`
- Modify: `docs/configuration.md`

**Work:**
1. Define which config families are auto-tunable in phase 1.
2. Define max delta and cooldown rules.
3. Route all candidate patches through existing `POST /admin/config/validate` logic before writes.

**Acceptance criteria:**
1. Disallowed keys are rejected before any attempt to validate or write.
2. Delta caps are explicit and test-covered.

### Task 3.3: Add the first apply-and-watch loop

**Files:**
- Create: `src/admin/oversight_apply.rs`
- Modify: `src/admin/api.rs`
- Test: `src/admin/oversight_apply.rs`
- Test: `src/admin/api.rs`

**Work:**
1. Apply one validated low-risk config-family patch.
2. Record previous values for rollback.
3. Start a bounded watch window.
4. Trigger rollback when watch-window budgets are breached.

**Acceptance criteria:**
1. No patch can apply without recorded rollback state.
2. Breach handling restores last known-good config deterministically.

## Phase 4: Adversary Verification in the Change Loop

### Task 4.1: Integrate deterministic adversary exercise with reconcile

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `docs/adversarial-operator-guide.md`
- Modify: `docs/testing.md`
- Test: `src/admin/api.rs`
- Modify: `Makefile`

**Work:**
1. Require adversary evidence for designated patch families before apply.
2. Reuse the existing adversary-sim lifecycle and deterministic test corpus.
3. Add focused Make targets for oversight verification flows.

**Acceptance criteria:**
1. Oversight cannot claim success without adversary evidence for guarded families.
2. Make targets truthfully describe whether they validate controller logic, replay logic, or live reconcile behavior.

### Task 4.2: Add decision-ledger evidence shaping

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `docs/api.md`

**Work:**
1. Store evidence references per decision:
   - snapshot hash,
   - patch family,
   - validation result,
   - adversary exercise result,
   - watch outcome,
   - rollback outcome if any.

**Acceptance criteria:**
1. A later operator can reconstruct why a change happened.

## Phase 5: Scheduler Adapters

### Task 5.1: Add host-side scheduler support

**Files:**
- Create: `scripts/run_with_oversight_supervisor.sh`
- Modify: `Makefile`
- Modify: `docs/deployment.md`
- Modify: `docs/testing.md`

**Work:**
1. Add a host-side supervisor or timer wrapper that periodically calls the internal oversight reconcile endpoint.
2. Keep it off the request path and separate from dashboard lifecycle.

**Acceptance criteria:**
1. Self-hosted and shared-host deployments can run oversight without a browser session.

### Task 5.2: Add Fermyon scheduler guidance

**Files:**
- Modify: `docs/deployment.md`
- Modify: `docs/README.md`

**Work:**
1. Document how a cron-triggered Spin/Fermyon adapter should invoke the reconcile path.
2. Keep the contract identical to the host-side adapter.

**Acceptance criteria:**
1. Scheduler differences are deployment concerns, not contract differences.

### Task 5.3: Record canonical schedule tiers

**Files:**
- Modify: `docs/deployment.md`
- Modify: `docs/testing.md`
- Modify: `Makefile`

**Work:**
1. Add canonical schedule tiers for:
   - 5-minute deterministic budget sweeps,
   - hourly adversary smoke or focused replay,
   - daily bounded adjustment cycles,
   - weekly frontier corpus promotion review.
2. Keep these tiers as adapter schedules over one reconcile contract.

**Acceptance criteria:**
1. The schedule tiers are explicit in docs and make-target guidance.
2. No deployment adapter invents its own incompatible oversight cadence semantics.

## Phase 6: Dashboard Visibility

### Task 6.1: Add minimal oversight status rendering

**Files:**
- Modify: `dashboard/src/lib/domain/api-client.js`
- Modify: `dashboard/src/lib/domain/dashboard-state.js`
- Modify: `dashboard/src/lib/components/dashboard/StatusTab.svelte`
- Test: `e2e/dashboard.modules.unit.test.js`
- Test: `e2e/dashboard.smoke.spec.js`

**Work:**
1. Show oversight mode, last decision, next scheduled reconcile, and degraded-state reasons.
2. Keep the first UI slice read-only or lightly controlled.
3. Reuse existing dashboard primitives and state conventions.

**Acceptance criteria:**
1. Operators can tell whether oversight is active, idle, blocked, or in rollback.

### Task 6.2: Add bounded control toggles

**Files:**
- Modify: `dashboard/src/lib/components/dashboard/StatusTab.svelte`
- Modify: `dashboard/src/lib/domain/api-client.js`
- Test: `e2e/dashboard.modules.unit.test.js`
- Test: `e2e/dashboard.smoke.spec.js`

**Work:**
1. Add mode switching for `off`, `observe`, `recommend`, and `canary_apply`.
2. Keep mode writes on the new oversight control endpoint, not mixed into generic config writes.

**Acceptance criteria:**
1. Dashboard does not optimistically invent final state.
2. Backend status remains the source of truth.

## Phase 7: Verified-Agent and Low-Cost Content Follow-On

### Task 7.1: Add verified-agent identity contract design and status-only hooks

**Files:**
- Create: `docs/adr/0012-verified-agent-identity-lane.md`
- Modify: `src/crawler_policy/robots.rs`
- Modify: `docs/api.md`
- Modify: `docs/value-proposition.md`

**Work:**
1. Define how verified-agent identity is represented in Shuma.
2. Keep phase-1 implementation to status and policy wiring only if full enforcement is not yet ready.

**Acceptance criteria:**
1. Verified-agent handling stays separate from legacy crawler advisory policy.

### Task 7.2: Design low-cost agent representations

**Files:**
- Create: `docs/plans/2026-03-15-low-cost-agent-representation-plan.md`

**Work:**
1. Define when Shuma should serve cheaper agent-oriented content.
2. Keep it future-facing until verified-agent identity is mature enough to gate it safely.

**Acceptance criteria:**
1. The project has an explicit path to "cheaper good-automation handling," not only stronger blocking.

## Verification Strategy

1. Docs-only prerequisite slice:
   - no tests required while capturing the design and plan.
2. Contract and unit verification:
   - `make test-unit`
3. Integration verification:
   - `make dev`
   - `make test-integration`
4. Dashboard verification:
   - `make test-dashboard-e2e`
5. Full umbrella verification:
   - `make test`

Additional Make targets to add before they are used as canonical workflow:

1. `make test-oversight-contract`
2. `make test-oversight-reconcile`
3. `make test-oversight-live`

## Rollout Policy

1. Start with `observe`.
2. Promote to `recommend` only after operators trust the snapshot and decision quality.
3. Promote to `canary_apply` only for low-risk config families with rollback proof.
4. Promote to broader `autonomous` mode only after:
   - replay coverage is stable,
   - rollback is routine,
   - telemetry truth budgets are reliable,
   - and verified-agent handling is explicit enough to avoid harming beneficial automation.

## Definition of Done by Milestone

### Milestone A

1. Oversight snapshot exists.
2. Status and history are read-only and truthful.
3. No autonomous writes.

### Milestone B

1. Deterministic reconcile exists.
2. Recommend mode produces bounded proposals.
3. Decision ledger is live.

### Milestone C

1. Canary apply works for one low-risk config family.
2. Rollback is automatic and tested.
3. Adversary verification is integrated.

### Milestone D

1. Scheduler adapters are deployment-ready.
2. Dashboard visibility is operator-grade.
3. Hosted control-plane extraction is an architectural option, not a rewrite.
