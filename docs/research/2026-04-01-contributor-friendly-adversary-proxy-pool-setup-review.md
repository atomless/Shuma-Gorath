# Contributor-Friendly Adversary Proxy-Pool Setup Review

Date: 2026-04-01
Status: Current design driver for environment readiness after the landed adversary-realism chain

Related context:

- [`2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](./2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
- [`../plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](../plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../plans/2026-03-31-post-2j-adversary-realism-sufficiency-plan.md`](../plans/2026-03-31-post-2j-adversary-realism-sufficiency-plan.md)
- [`../../src/admin/adversary_sim_representativeness.rs`](../../src/admin/adversary_sim_representativeness.rs)
- [`../../src/admin/adversary_sim_identity_pool.rs`](../../src/admin/adversary_sim_identity_pool.rs)
- [`../../src/admin/adversary_sim_trusted_ingress.rs`](../../src/admin/adversary_sim_trusted_ingress.rs)
- [`../../scripts/supervisor/trusted_ingress_proxy.py`](../../scripts/supervisor/trusted_ingress_proxy.py)
- [`../../scripts/run_with_adversary_sim_supervisor.sh`](../../scripts/run_with_adversary_sim_supervisor.sh)

## Purpose

The adversary-realism implementation chain is now landed, but later Game Loop and Tuning work remain blocked in practice because target environments still need representative hostile-lane backing. Today that last mile is too operator-manual and too externalized: a contributor can run Shuma locally, but there is no first-class Shuma workflow for supplying real hostile proxy pools and validating that the environment is actually representative.

This review narrows the remaining problem:

1. what should stay outside the repo,
2. what Shuma should own directly,
3. whether that workflow should run locally or through a generic hosted backend,
4. and how an optional agent-facing runbook or skill adapter should relate to the canonical contributor path.

## Current State

Shuma already has the runtime and readiness primitives:

1. the representativeness gate requires trusted ingress plus hostile-lane proxy-pool backing in [`../../src/admin/adversary_sim_representativeness.rs`](../../src/admin/adversary_sim_representativeness.rs),
2. pool entries already have a normalized schema in [`../../src/admin/adversary_sim_identity_pool.rs`](../../src/admin/adversary_sim_identity_pool.rs),
3. trusted-ingress proxy URLs already have a strict adapter contract in [`../../src/admin/adversary_sim_trusted_ingress.rs`](../../src/admin/adversary_sim_trusted_ingress.rs),
4. and the local trusted-ingress sidecar already exists in [`../../scripts/supervisor/trusted_ingress_proxy.py`](../../scripts/supervisor/trusted_ingress_proxy.py) and is launched opportunistically by [`../../scripts/run_with_adversary_sim_supervisor.sh`](../../scripts/run_with_adversary_sim_supervisor.sh).

What is still missing is a contributor-owned setup workflow. The repo does not yet provide a canonical path for:

1. accepting provider credentials,
2. materializing normalized request/browser/agentic proxy pools,
3. validating health and readiness,
4. and surfacing a compact local status that tells a contributor whether the hostile lanes are truly representative.

## What Must Remain External

Shuma cannot create real hostile identities from localhost.

The following must remain outside the repo:

1. real residential, mobile, or datacenter proxy inventory,
2. vendor accounts, payment, and quota ownership,
3. any self-hosted exit infrastructure if a team chooses that route,
4. and any provider-specific compliance or commercial terms.

That is a hard boundary, not a tooling gap.

## What Shuma Should Own

Shuma should still own the contributor workflow around those external resources.

The repo should own:

1. secret intake and validation rules,
2. local normalized artifact generation,
3. health checks and readiness summaries,
4. trusted-ingress orchestration,
5. lane-to-pool mapping,
6. and contributor documentation and proofs.

If Shuma leaves those steps as free-form manual ops, the representativeness gate becomes truthful but impractical, and contributors cannot reliably reproduce a representative hostile environment.

## Options Considered

### Option 1: Keep Proxy-Pool Setup Entirely Outside The Repo

Rejected.

Why:

1. it makes environment readiness depend on undocumented operator improvisation,
2. it undermines reproducibility,
3. it leaves no canonical proof path for representative hostile readiness,
4. and it guarantees drift between contributor setups.

### Option 2: Shuma-Owned Repo Workflow Plus Local Sidecars

Recommended as the default path.

Shape:

1. contributor supplies provider credentials or provider-issued proxy seeds through a repo-owned config workflow,
2. Shuma materializes normalized pool artifacts under local generated state,
3. Shuma launches or consumes the existing trusted-ingress sidecar locally,
4. Shuma validates request, browser, and agentic pools through focused readiness checks,
5. and normal `make dev` or adversary-sim flows reuse those validated artifacts instead of rebuilding them every time.

Why:

1. it preserves the current trust boundary,
2. it keeps the setup contributor-friendly,
3. it keeps the canonical workflow tool-agnostic and available to every contributor,
4. and it fits the repo's existing separation between source and generated local state.

### Option 3: Generic Hosted BaaS As The Primary Setup Path

Rejected as the first implementation.

Why:

1. the main need is outbound hostile egress orchestration and trust-boundary preservation, not generic CRUD or hosted storage,
2. a generic BaaS would still leave Shuma to solve lane mapping, secret handling, readiness truth, and sidecar behavior,
3. it would add a second operational dependency before the local workflow is even standardized,
4. and it would make contributor-local verification harder, not easier.

### Option 4: Shuma-Owned Shared Broker For Team Staging

Reasonable later, but not as the first tranche.

Why:

1. a shared broker may be valuable later for staging or production-like rehearsal,
2. but it should build on top of a proven local workflow,
3. and it should stay a Shuma-owned narrow broker rather than a generic backend abstraction.

## Recommended Direction

Adopt a two-layer contract:

1. canonical path: repo-owned `make` workflow plus local sidecar processes,
2. optional convenience path: an agent-facing runbook or skill adapter that wraps the same repo workflow.

That runbook or skill adapter should not be the only path because:

1. contributors must be able to use the workflow without any specific assistant runtime,
2. `make` targets remain the canonical proof and documentation surface,
3. and the skill should be a guided wrapper over those targets rather than a separate hidden implementation.

## Proposed Responsibilities

### Repo-Owned Workflow

The repo should provide:

1. a local-state location for generated adversary proxy artifacts under `.shuma/`,
2. a provider-profile contract and validation rules,
3. explicit setup, refresh-if-stale, and validate commands,
4. a local broker or helper process that can transform provider credentials into normalized lane pools,
5. and a focused readiness summary that explains why a lane is still degraded or partial.

### Local Sidecar Topology

The first-class local topology should be:

1. running Shuma on the contributor machine,
2. running the trusted-ingress sidecar locally,
3. running a narrow local proxy-pool broker or generator locally,
4. feeding normalized pool entries to Scrapling request, Scrapling browser, and Agentic request lanes,
5. while leaving the actual egress IP inventory external through provider-backed proxies.

### Optional Agent-Facing Runbook Or Skill Adapter

The optional runbook or skill adapter should:

1. collect the required inputs,
2. call the repo-owned `make` targets,
3. summarize validation results,
4. and never bypass the canonical artifacts or readiness checks.

## Guardrails

1. Do not give attacker-plane workers `X-Shuma-Forwarded-Secret`.
2. Do not teach workers any privileged client-IP shortcut.
3. Do not check generated pool artifacts into the repo.
4. Do not make `make dev` regenerate hostile-pool artifacts on every run.
5. Do not make an agent-facing runbook or skill adapter the only supported setup path.
6. Do not treat a generic BaaS as the required first implementation.

## Consequence For The Roadmap

The adversary-realism implementation chain is complete, but environment readiness is not.

The next execution-ready work should therefore be a contributor-friendly setup chain that:

1. makes hostile proxy-pool readiness Shuma-owned and reproducible,
2. keeps the current trusted-ingress boundary intact,
3. keeps local development practical,
4. and makes the representativeness gate operational rather than aspirational.

## Standards And Prior Research

This direction continues the already-adopted trust-boundary model:

1. forwarded client identity remains trustworthy only when a trusted ingress owns it rather than arbitrary callers,
2. client-IP replacement remains bounded to explicitly trusted senders,
3. and operator surfaces should distinguish degraded from representative identity truth rather than blur the difference.

Relevant standards and primary references:

1. RFC 7239 Forwarded header semantics: <https://www.rfc-editor.org/rfc/rfc7239>
2. NGINX `set_real_ip_from` / `real_ip_header` trust-boundary guidance: <https://nginx.org/en/docs/http/ngx_http_realip_module.html>
3. Cloudflare origin IP-header guidance: <https://developers.cloudflare.com/fundamentals/reference/http-headers/>
