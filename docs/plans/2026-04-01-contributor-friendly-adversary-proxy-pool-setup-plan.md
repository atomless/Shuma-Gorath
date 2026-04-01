# Contributor-Friendly Adversary Proxy-Pool Setup Plan

**Goal:** Make representative hostile-lane backing contributor-operable through a repo-owned setup workflow, local sidecars, and exact readiness proofs rather than leaving proxy-pool configuration as ad hoc external operator work.

**Architecture:** Keep the current runtime trust boundary intact, treat real hostile egress as an external dependency, and add a Shuma-owned local workflow that can intake provider credentials, materialize normalized request/browser/agentic pool artifacts under local generated state, validate them, and expose exact readiness truth through `make` targets.

Related context:

- [`../research/2026-04-01-contributor-friendly-adversary-proxy-pool-setup-review.md`](../research/2026-04-01-contributor-friendly-adversary-proxy-pool-setup-review.md)
- [`../research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](../research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
- [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](./2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../../src/admin/adversary_sim_representativeness.rs`](../../src/admin/adversary_sim_representativeness.rs)
- [`../../scripts/supervisor/trusted_ingress_proxy.py`](../../scripts/supervisor/trusted_ingress_proxy.py)

## Scope Boundary

Included here:

1. contributor-local hostile-proxy setup workflow,
2. generated local artifact layout and validation,
3. local sidecar or broker orchestration,
4. exact readiness reporting for request/browser/agentic pools,
5. and an optional agent-facing runbook or skill adapter that wraps the repo-owned workflow.

Explicitly not included here:

1. creating proxy vendors or external hostile egress out of thin air,
2. generic hosted BaaS as the default path,
3. automatic daily regeneration during ordinary `make dev`,
4. or widening attacker-plane worker privileges.

## Acceptance Criteria

1. A contributor can supply provider-backed hostile egress inputs through a documented Shuma workflow without hand-authoring raw pool JSON.
2. Normal contributor flows reuse existing validated local artifacts and do not regenerate hostile-pool state on every `make dev` or `make build`.
3. Generated proxy-pool artifacts live under local generated state, not under `src/`, `docs/`, or checked-in config.
4. Request, browser, and agentic hostile pools are validated separately, and readiness failures explain exactly which lane is still partial or degraded.
5. The trusted-ingress boundary stays unchanged: workers still must not emit `x-shuma-forwarded-secret`.
6. An optional agent-facing runbook or skill adapter may assist setup, but the canonical workflow remains repo-owned `make` targets plus docs.
7. A shared hosted broker remains explicitly deferred until the local path is landed and proven useful.

## Task 1: `SIM-REALISM-ENV-1A` Freeze The Setup Contract And Artifact Layout

**Files:**
- Later code targets: `config/`, `scripts/`, `.gitignore`, local-state docs, Makefile help text
- Later proof targets: `Makefile` contract target, focused unit validation, docs

**Work:**
1. Freeze the contributor-visible provider-profile input contract.
2. Freeze the generated local artifact location under `.shuma/`.
3. Freeze lane-specific artifact boundaries for Scrapling request, Scrapling browser, and Agentic request pools.
4. Keep secrets, generated pool manifests, and health receipts out of tracked source files.

**Acceptance criteria:**
1. Contributors have one canonical place to put provider-backed setup inputs.
2. Generated artifacts are clearly separated from source and documentation.
3. Lane-specific pool artifacts are explicit rather than one undifferentiated proxy list.

**Proof:**
1. Add and pass `make test-adversary-sim-proxy-setup-contract`.

## Task 2: `SIM-REALISM-ENV-1B` Add Make-Driven Setup, Refresh, And Validate Commands

**Files:**
- Later code targets: `Makefile`, setup scripts, bootstrap verification, contributor docs
- Later proof targets: focused Make-target contract and integration-style workflow test

**Work:**
1. Add canonical setup and validation targets for hostile proxy-pool readiness.
2. Support explicit refresh and refresh-if-stale behavior rather than automatic regeneration on every normal dev/build cycle.
3. Surface exact validation failures when provider inputs or generated pools are missing or invalid.

**Acceptance criteria:**
1. Contributors can bootstrap hostile-pool readiness through documented `make` targets.
2. The workflow is explicit and repeatable.
3. Normal `make dev` remains fast and reuses validated artifacts.

**Proof:**
1. Add and pass `make test-adversary-sim-proxy-setup-flow`.

## Task 3: `SIM-REALISM-ENV-1C` Add A Local Broker Or Helper Process For Pool Materialization And Health

**Files:**
- Later code targets: `scripts/` helper/broker code, health receipts, Makefile wiring, supervisor integration
- Later proof targets: focused local-broker contract and health-check verification

**Work:**
1. Add a narrow local helper or broker process that can transform provider-backed inputs into normalized lane pool artifacts.
2. Validate proxy reachability and identity metadata before lanes consume the pools.
3. Expose compact redacted health and readiness output without leaking raw secrets into dashboards or worker plans.

**Acceptance criteria:**
1. Contributors do not need to hand-maintain raw lane pool JSON.
2. Broken provider inputs fail during setup validation rather than during later lane execution.
3. Pool health state is available without turning the broker into a generic platform dependency.

**Proof:**
1. Add and pass `make test-adversary-sim-proxy-broker-contract`.

## Task 4: `SIM-REALISM-ENV-1D` Add Contributor Docs And An Optional Agent-Facing Runbook Or Skill Adapter

**Files:**
- Later code targets: operator docs, testing docs, setup docs, optional agent-facing runbook or skill assets
- Later proof targets: docs contract and optional runbook or skill verification

**Work:**
1. Add contributor-facing docs that explain local prerequisites, exact setup steps, and degraded-versus-representative outcomes.
2. Add an optional agent-facing runbook or skill adapter that wraps the canonical repo workflow.
3. Keep that adapter as a convenience layer rather than the only supported path.

**Acceptance criteria:**
1. A contributor can go from provider credentials to validated local readiness without reverse-engineering internal env names.
2. Any runbook or skill adapter uses the same `make` workflow and generated artifacts as manual contributors.
3. Docs explain clearly which parts remain external infrastructure.

**Proof:**
1. Add and pass `make test-adversary-sim-proxy-setup-docs-contract`.

## Deferred Follow-On: Shared Hosted Broker

Do not start this before `SIM-REALISM-ENV-1A..1D` are landed.

If later needed for team staging or production-like rehearsals, a Shuma-owned shared broker may be added as a separate follow-on. It must remain a narrow Shuma service over the same pool and readiness contract rather than a generic BaaS abstraction.

## Recommended Execution Order

1. `SIM-REALISM-ENV-1A`
2. `SIM-REALISM-ENV-1B`
3. `SIM-REALISM-ENV-1C`
4. `SIM-REALISM-ENV-1D`

Do not start with the optional runbook or skill adapter, and do not jump straight to a shared hosted broker. The canonical repo workflow and local generated-state contract must exist first.
