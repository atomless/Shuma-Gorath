# Route Namespace And Root-Hosted Public Site Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the `/sim/public/*` public-site model and top-level Shuma control routes with a root-hosted protected public site plus a dedicated `/shuma/*` namespace for Shuma-owned control and operational surfaces.

**Architecture:** Centralize route ownership in a canonical Shuma route-namespace contract, move the generated contributor public-content site to the origin root, move Shuma-owned control-plane, dashboard, health, metrics, and internal routes under `/shuma/*`, and then migrate adversary corpus, robots/sitemap, dashboard, scripts, docs, and tests onto that corrected path model. Keep public defence surfaces on the protected-site path space in this tranche unless a separate design later changes that ownership.

**Tech Stack:** Rust runtime routing, generated contributor site artifact, dashboard static and login routes, Python supervisory scripts, adversarial corpus and realism contracts, Makefile proof targets, docs and backlog.

Related context:

- [`../research/2026-03-31-route-namespace-and-root-hosted-public-site-review.md`](../research/2026-03-31-route-namespace-and-root-hosted-public-site-review.md)
- [`2026-03-30-contributor-generated-public-content-sim-site-plan.md`](./2026-03-30-contributor-generated-public-content-sim-site-plan.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](./2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs)
- [`../../src/lib.rs`](../../src/lib.rs)

---

## Route Ownership Assumption

This plan assumes:

1. Shuma-owned control or operational routes move under `/shuma/*`,
2. host content and standard public metadata move to root-hosted public paths,
3. public defence surfaces that intentionally participate in the protected host path space remain outside `/shuma/*` in this tranche.

Concretely, the target namespace is:

1. root-hosted public site:
   - `/`
   - generated content pages under `/*`
   - `/robots.txt`
   - `/sitemap.xml`
   - `/atom.xml`
2. Shuma-owned routes:
   - `/shuma/admin/*`
   - `/shuma/dashboard/*`
   - `/shuma/health`
   - `/shuma/metrics`
   - `/shuma/internal/*`

## Acceptance Criteria

1. The generated contributor site is served from the origin root, not from `/sim/public/*`.
2. `make dev` exposes that root-hosted public site even when adversary sim is disabled or idle.
3. Shuma-owned control, dashboard, health, metrics, and internal routes are served only from `/shuma/*`.
4. Top-level legacy routes `/admin/*`, `/dashboard/*`, `/health`, `/metrics`, and `/sim/public/*` no longer remain as active canonical paths.
5. Deterministic and adaptive adversary path hints start from `/` and standard public hints such as root `robots.txt` and root sitemap paths rather than `/sim/public/*`.
6. Docs, scripts, and tests stop advertising the legacy route families.
7. The migration lands without backward-compatibility aliases unless the user explicitly asks for them later.

## Task 1: Freeze The Canonical Route Namespace Contract

**Files:**
- Create: `src/http_route_namespace.rs`
- Modify: `src/lib.rs`
- Modify: route-contract tests in `tests/` or existing Rust unit suites
- Modify: `Makefile`

**Work:**
1. Introduce one canonical module for Shuma-owned route prefixes and well-known public metadata paths.
2. Stop scattering `/admin`, `/dashboard`, `/health`, `/metrics`, `/internal`, and `/sim/public` string literals across routing logic.
3. Freeze the new route contract in focused tests before wider migration.

**Acceptance criteria:**
1. A single canonical module defines Shuma-owned route prefixes and public metadata roots.
2. Runtime routing imports that module instead of embedding fresh literals.
3. Focused tests fail if legacy canonical route ownership returns.

**Proof:**
1. Add and pass `make test-shuma-route-namespace-contract`.

## Task 2: Move The Generated Contributor Site From `/sim/public/*` To Root-Hosted Public Paths

**Files:**
- Modify: `src/runtime/sim_public.rs`
- Modify: `scripts/build_sim_public_site.py`
- Modify: `scripts/sim_public_site/build.py`
- Modify: `config/sim_public_site/corpus.toml`
- Modify: generator and serving tests
- Modify: `Makefile`

**Work:**
1. Replace the `/sim/public/*` serving prefix with root-hosted public pages.
2. Serve the generated site at `/`, section roots, and article paths.
3. Serve standard public metadata at root:
   - `/robots.txt`
   - `/sitemap.xml`
   - `/atom.xml`
4. Remove the legacy `/sim/public/*` serving model after proof.

**Acceptance criteria:**
1. The generated public site is reachable from `/`.
2. Root `robots.txt`, `sitemap.xml`, and `atom.xml` are emitted and served correctly.
3. `make dev` still serves the site when the generated artifact exists and adversary sim is disabled or idle.
4. `/sim/public/*` is no longer the canonical generated-site surface.

**Proof:**
1. Add and pass `make test-shuma-root-public-site-serving`.
2. Keep `make test-sim-public-generator` green or replace it with a truthfully renamed focused target.

## Task 3: Move Control-Plane, Dashboard, Health, Metrics, And Internal Routes Under `/shuma/*`

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/admin/*` route handlers and auth/session helpers
- Modify: dashboard route handling and login/session surfaces
- Modify: supervisor and helper scripts that call admin or internal endpoints
- Modify: runtime tests and route-order tests

**Work:**
1. Move:
   - `/admin/* -> /shuma/admin/*`
   - `/dashboard/* -> /shuma/dashboard/*`
   - `/health -> /shuma/health`
   - `/metrics -> /shuma/metrics`
   - `/internal/* -> /shuma/internal/*`
2. Update auth, redirect, session, and supervisor polling paths accordingly.
3. Remove legacy top-level canonical ownership after proof.

**Acceptance criteria:**
1. All Shuma-owned control and operational surfaces are namespaced under `/shuma/*`.
2. Dashboard login and shell work from `/shuma/dashboard/*`.
3. Supervisor and helper scripts no longer poll top-level `/admin/*` or `/internal/*`.
4. Legacy top-level control routes do not remain active canonical paths.

**Proof:**
1. Add and pass `make test-shuma-control-route-migration`.
2. Keep focused dashboard path and auth targets green after they are updated.

## Task 4: Migrate Adversary Seed, Corpus, Realism, And Benchmark Assumptions To Root-Hosted Public Paths

**Files:**
- Modify: `src/admin/adversary_sim_corpus.rs`
- Modify: deterministic corpus JSON and related contract tests
- Modify: browser-driver and worker path-hint tests
- Modify: `src/crawler_policy/robots.rs`
- Modify: benchmark or observer sample-path fixtures that still point to `/sim/public/*`

**Work:**
1. Replace legacy `/sim/public/*` path hints with root-hosted equivalents.
2. Update robots and sitemap generation to advertise root-hosted public metadata paths.
3. Remove benchmark and observer sample-path assumptions that depend on the legacy prefix.

**Acceptance criteria:**
1. Deterministic and adaptive adversary paths begin from `/` and root-discoverable public pages.
2. Root `robots.txt` and root sitemap references become the canonical public hints.
3. Observer and benchmark fixtures no longer imply `/sim/public/*` is the host site's public root.

**Proof:**
1. Add and pass `make test-shuma-rooted-adversary-path-contract`.
2. Keep relevant realism and corpus-contract targets green.

## Task 5: Migrate Docs, Scripts, And Operator Guidance; Remove Legacy Route Families

**Files:**
- Modify: `docs/api.md`
- Modify: `docs/dashboard.md`
- Modify: `docs/testing.md`
- Modify: `docs/quick-reference.md`
- Modify: route-using operator or deployment docs
- Modify: TODO/history records as work lands

**Work:**
1. Rewrite docs and examples to use the new route namespace.
2. Remove stale `/sim/public/*`, `/admin/*`, `/dashboard/*`, `/health`, and `/metrics` examples where they describe canonical live paths.
3. Preserve explicit wording about what remains in public host space versus what is now under `/shuma/*`.

**Acceptance criteria:**
1. Docs no longer advertise the legacy route families as canonical.
2. Contributor guidance reflects root-hosted public-site browsing under `make dev`.
3. Operational guidance reflects the new `/shuma/*` control-plane contract.

**Proof:**
1. Keep docs/TODO references consistent.
2. Keep focused route-contract and dashboard-path targets green.

## Task 6: Review Loop And Removal Discipline

**Files:**
- Modify: any files still referencing legacy route families after migration
- Modify: `todos/completed-todo-history.md`

**Work:**
1. After each tranche, run a whole-repo route audit for:
   - `/sim/public/*`
   - `/admin/*`
   - `/dashboard/*`
   - `/health`
   - `/metrics`
   - `/internal/*`
2. Convert remaining legitimate references to the new contract or explicitly justify them.
3. Treat unreviewed legacy path remnants as incomplete migration evidence.

**Acceptance criteria:**
1. Route-audit evidence exists after each tranche.
2. Legacy route references are either removed or explicitly justified as non-canonical historical context.
3. The final state has one coherent route model rather than dual ownership.

**Proof:**
1. Use exact `rg` route-audit evidence in tranche completion notes.
2. Keep the new focused route-contract targets green.
