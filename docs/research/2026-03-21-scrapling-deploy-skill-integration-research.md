# Scrapling Deploy Skill Integration Research

Date: 2026-03-21
Status: Active research note

Related context:

- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md`](../plans/2026-03-20-minimal-seed-and-telemetry-surface-discovery-design.md)
- [`../plans/2026-03-20-shared-host-scope-fence-contract.md`](../plans/2026-03-20-shared-host-scope-fence-contract.md)
- [`../plans/2026-03-20-shared-host-seed-contract.md`](../plans/2026-03-20-shared-host-seed-contract.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`../../skills/prepare-shared-host-on-linode/SKILL.md`](../../skills/prepare-shared-host-on-linode/SKILL.md)
- [`../../skills/deploy-shuma-on-linode/SKILL.md`](../../skills/deploy-shuma-on-linode/SKILL.md)
- [`../../skills/prepare-shuma-on-akamai-fermyon/SKILL.md`](../../skills/prepare-shuma-on-akamai-fermyon/SKILL.md)
- [`../../skills/deploy-shuma-on-akamai-fermyon/SKILL.md`](../../skills/deploy-shuma-on-akamai-fermyon/SKILL.md)
- [`../../scripts/deploy/linode_shared_host_setup.py`](../../scripts/deploy/linode_shared_host_setup.py)
- [`../../scripts/deploy/fermyon_akamai_edge_setup.py`](../../scripts/deploy/fermyon_akamai_edge_setup.py)
- [`../../scripts/deploy/remote_target.py`](../../scripts/deploy/remote_target.py)
- [`../../scripts/deploy_linode_one_shot.sh`](../../scripts/deploy_linode_one_shot.sh)
- [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)

## Question

How should Shuma turn the remaining Scrapling deployment steps:

1. provision a Python worker runtime,
2. define scope,
3. build seeds,
4. wire file paths into the supervisor environment,
5. harden egress externally,
6. verify,

into an agent-facing skill and helper surface that keeps operator input close to the current deploy baseline and keeps the simulation as realistic as possible?

## Current State

### 1. The architecture already rejects catalog-first discovery

The active codebase and plan chain are now clear that:

1. shared-host discovery for Scrapling should be `scope_fence -> minimal_seed_contract -> traversal_telemetry`,
2. telemetry is the map,
3. the reachable surface must come from observed traversal telemetry,
4. and deploy-time catalogs must not be treated as emergent runtime surface truth.

### 2. The deploy skills still encode the old setup story

The current Linode and Fermyon/Akamai setup skills still tell the agent to capture or generate `GATEWAY_SURFACE_CATALOG_PATH` as the main deploy-time discovery artifact.

That is still valid for gateway preflight and route-collision safety, but it is not the right operator contract for Scrapling runtime activation.

### 3. Shared-host deployment can actually host Scrapling today

The shared-host runtime path already has the hard runtime pieces:

1. `make setup-runtime` provisions the repo-owned `.venv-scrapling`,
2. `make prod-start` runs under `scripts/run_with_adversary_sim_supervisor.sh`,
3. the supervisor already consumes:
   - `ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH`
   - `ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH`
   - `ADVERSARY_SIM_SCRAPLING_CRAWLDIR`
   - optional `ADVERSARY_SIM_SCRAPLING_PYTHON`
4. and the worker already fails closed when those inputs are missing.

So shared-host does not need a new worker architecture. It needs deploy-time artifact and env automation.

### 4. The shared-host deploy path does not yet ship Scrapling artifacts

Today the shared-host deploy flow:

1. uploads the release bundle,
2. uploads the gateway surface catalog,
3. writes `.env.local`,
4. runs `make setup-runtime`,
5. and starts the runtime under the supervisor wrapper.

But it does not:

1. generate a Scrapling scope descriptor,
2. generate a minimal seed inventory,
3. upload those artifacts,
4. or persist the required `ADVERSARY_SIM_SCRAPLING_*` env values into the deployed runtime env.

That is the real automation gap.

### 5. Edge deploy is not a good fit for the full runtime architecture today

Fermyon/Akamai edge deploy can share the same scope-and-seed artifact generation, but it does not have a host-local Python worker plane in the current deploy contract.

The current docs are explicit that edge/no-local-process environments need an external supervisor service calling the same internal beat endpoint.

That means Fermyon is fine as:

1. an edge gateway deployment target,
2. a trusted-edge signal and gateway posture target,

but it is not currently a clean first-class home for the full Scrapling adversary runtime.

Given the current product stage and the desire to keep the simulation realistic, shared-host should be the primary supported target for full adversary-sim runtime operation.

## Design Constraints

### 1. Keep operator input irreducible

The operator should not need to hand-author:

1. a scope descriptor,
2. a seed inventory,
3. worker env values,
4. or a verification checklist,

when those can be inferred from the deploy context.

### 2. Keep the adversary starting point realistic

The simulation must not get discovery inputs a real attacker would not get by default.

So the first deploy-time seed should be minimal:

1. just the normalized public root URL by default,
2. no catalog-derived route list,
3. no pre-expanded site map,
4. and no automatic privileged or hidden path hints.

Optional `robots.txt` or extra seeds can exist later as explicit operator choices, but the default deploy-time setting should be just the root host.

### 3. Keep deploy-time catalog tooling for gateway safety only

`GATEWAY_SURFACE_CATALOG_PATH` still belongs to:

1. gateway collision preflight,
2. smoke path selection,
3. existing-site deployment evidence.

It must not be repurposed as the runtime Scrapling surface map.

### 4. Reuse the settled scope and seed contracts

The new helper must reuse:

1. `scripts/tests/shared_host_scope.py`
2. `scripts/tests/shared_host_seed_inventory.py`

instead of inventing a third deploy-only scope or seed format.

### 5. Fit the existing receipt pattern

The existing deploy skills already use durable local receipts in `.shuma/`.

The Scrapling automation should follow the same pattern:

1. create durable local artifacts,
2. create a durable local receipt,
3. let provider-specific deploy flows consume that receipt,
4. and keep operator-facing skills short because the helper does the work.

## Recommended Architecture

### 1. Add one shared Scrapling deploy-preflight helper

Create one shared helper that:

1. derives the default allowed host from the public base URL,
2. creates a fail-closed scope descriptor with the existing baseline denied path family,
3. creates the minimal seed inventory with the public base URL root as the only default start URL,
4. records the exact env values the supervisor runtime must see,
5. records external egress requirements,
6. records the relevant verification commands,
7. and writes a durable receipt under `.shuma/`.

This helper should not ask the operator to build a catalog or curate a route list.

### 2. Make shared-host consume that receipt automatically

For shared-host deploy:

1. the deploy path should generate or refresh the Scrapling receipt from the deploy context,
2. upload the generated scope and seed artifacts to the host,
3. write the `ADVERSARY_SIM_SCRAPLING_*` env vars into the deployed `.env.local`,
4. and store enough metadata in the normalized remote receipt that `make remote-update` can keep doing the same thing.

That makes Scrapling available on the host without a second manual setup flow.

### 3. Focus the full runtime contract on shared-host

Do not spend this tranche productizing Fermyon as a full Scrapling runtime target.

Instead:

1. keep Fermyon deploy docs truthful that it remains an edge gateway path,
2. explicitly say full adversary-sim runtime support is shared-host-first,
3. and defer any external-supervisor productization until there is a concrete reason to build it.

### 4. Add a dedicated agent-facing skill

Add one agent-facing skill that explains the shared workflow:

1. infer defaults from deployment context,
2. generate the Scrapling receipt and artifacts,
3. wire the result into the shared-host deploy path,
4. and treat traversal telemetry as the runtime surface map.

Then make the existing deploy skills depend on it where appropriate.

## Default Inference Rules

Recommended first defaults:

1. `allowed_hosts`:
   - exact host from the public base URL only.
2. `primary_start_url`:
   - normalized public base URL root (`https://host/`).
3. `extra_seed_urls`:
   - none by default.
4. `robots.txt`:
   - off by default for deploy-time prep.
5. `require_https=true`
6. `deny_ip_literals=true`
7. remote scope path:
   - `/opt/shuma-gorath/.shuma/adversary-sim/scrapling-scope.json`
8. remote seed path:
   - `/opt/shuma-gorath/.shuma/adversary-sim/scrapling-seed-inventory.json`
9. remote crawldir:
   - `/opt/shuma-gorath/.shuma/adversary-sim/scrapling-crawldir`

These defaults are narrow, deterministic, and require no new operator input.

## What Not To Do

The implementation should not:

1. replace `GATEWAY_SURFACE_CATALOG_PATH` for gateway preflight,
2. revive catalog-first Scrapling runtime guidance,
3. feed Scrapling a pre-expanded route set by default,
4. pretend edge deploy alone makes Scrapling fully operational,
5. require the operator to hand-write scope or seed files for the common case,
6. or introduce a second scope/seed contract unrelated to the active shared-host contracts.

## Recommended Execution Slices

1. Add the research-backed TODO split for this work inside `SIM-SCR-8`.
2. Add the shared helper plus tests and receipt schema first.
3. Integrate shared-host deploy/setup and remote-update wiring next.
4. Add the agent-facing skill and update the shared-host deploy skills to depend on it.
5. Update edge/Fermyon docs to mark full adversary-sim runtime support as shared-host-first and defer external-supervisor productization.

## Outcome

If implemented this way:

1. shared-host deploy can make Scrapling operational with inferred defaults,
2. the default adversary starting point is just the root host,
3. Fermyon remains a gateway target but not the primary full adversary-sim runtime target,
4. the operator burden drops back toward the existing deploy baseline,
5. and Shuma stays aligned with the rule that telemetry is the map.
