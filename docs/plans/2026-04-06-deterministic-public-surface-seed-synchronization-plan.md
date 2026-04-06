# Deterministic Public-Surface Seed Synchronization Plan

Date: 2026-04-06
Status: Proposed

Related context:

- [`2026-03-31-route-namespace-and-root-hosted-public-site-plan.md`](./2026-03-31-route-namespace-and-root-hosted-public-site-plan.md)
- [`2026-03-30-contributor-generated-public-content-sim-site-plan.md`](./2026-03-30-contributor-generated-public-content-sim-site-plan.md)
- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](./2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](./2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
- [`2026-04-06-adversary-objective-separation-and-asymmetric-loop-plan.md`](./2026-04-06-adversary-objective-separation-and-asymmetric-loop-plan.md)
- [`../research/2026-03-31-route-namespace-and-root-hosted-public-site-review.md`](../research/2026-03-31-route-namespace-and-root-hosted-public-site-review.md)
- [`../../scripts/build_sim_public_site.py`](../../scripts/build_sim_public_site.py)
- [`../../scripts/build_site_surface_catalog.py`](../../scripts/build_site_surface_catalog.py)
- [`../../scripts/site_surface_catalog.py`](../../scripts/site_surface_catalog.py)
- [`../../src/admin/adversary_sim_lane_runtime.rs`](../../src/admin/adversary_sim_lane_runtime.rs)
- [`../../src/admin/adversary_sim_corpus.rs`](../../src/admin/adversary_sim_corpus.rs)
- [`../../scripts/tests/adversarial/deterministic_attack_corpus.v1.json`](../../scripts/tests/adversarial/deterministic_attack_corpus.v1.json)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the synthetic or deterministic adversary lane seed from the exact currently mounted generated public site, not from a stale fixed route list.

**Architecture:** Treat the generated public-site artifact under `.shuma/sim-public-site/` as the single source of truth for the public surface Shuma is actually serving. Extend the existing site refresh flow so one explicit regeneration step writes the served site, the semantic site manifest, and a sibling deterministic surface-seeding catalog derived from that same docroot. Then move deterministic-lane path selection off the fixed `primary_public_paths` list and onto the generated artifact metadata, while preserving deterministic `run_id + tick + slot` selection and the existing supplemental defense probes.

**Tech Stack:** existing sim-public generator and artifact manifest, existing site-surface catalog compiler, Rust adversary-sim runtime and corpus loaders, focused `Makefile` proof targets, Python and Rust contract tests, docs/TODO bookkeeping.

---

## Problem Statement

The route-namespace migration corrected the public site from `/sim/public/*` to the origin root, and the deterministic lane now reflects that prefix correction. However, the deterministic lane still samples from a fixed shallow list of roots rather than from the richer generated public site it is meant to confront.

Today:

1. the generated site artifact contains deep article routes, section pagination, root and section feeds, and sitemap documents,
2. the public-route matcher already recognizes those broader surfaces,
3. but deterministic runtime selection still pulls from a fixed `primary_public_paths` list,
4. and the baked deterministic corpus still freezes only six public paths (`/`, `/about/`, `/research/`, `/plans/`, `/work/`, `/atom.xml`).

That leaves the synthetic lane root-host aware but still traversal-shallow. The deterministic lane is therefore confronting a simplified approximation of the public site rather than the generated site currently mounted by Shuma.

## Core Decision

Shuma should not solve this by introducing a separately maintained manual surface catalog. The synchronization burden should instead be collapsed into the existing explicit public-site regeneration flow.

This plan locks in the following rule:

1. whenever the generated public site is refreshed,
2. the deterministic surface-seeding artifact must be refreshed from that same exact docroot in the same bounded workflow,
3. and deterministic-lane seeding must consume that derived artifact rather than a repo-authored fixed path list.

In other words, sync must be structural rather than procedural.

## Decisions Locked In

1. The served site artifact under `.shuma/sim-public-site/site/` remains the source of truth for what the synthetic lane is allowed to treat as current public terrain.
2. The existing public-site `manifest.json` remains the semantic site-generation receipt.
3. A sibling `surface-catalog.json` should be emitted from the same explicit refresh flow rather than maintained by a separate manual process.
4. The deterministic lane keeps deterministic selection (`run_id + tick + slot`) for reproducibility.
5. The deterministic lane keeps existing supplemental defense probes (`pow`, challenge, maze, honeypot, tarpit, CDP, rate) separate from public-surface route seeding.
6. The deterministic lane must not silently claim deep public-surface coverage if the generated seeding artifact is missing, stale, or narrower than the mounted site.
7. Runtime and operator surfaces must report truthful degradation when deterministic public-surface seeding falls back to a reduced or incomplete seed set.

## Explicit Assumptions

1. The repository remains pre-launch, so no backward-compatibility shim is needed for older deterministic corpus formats unless explicitly requested later.
2. Contributor and local-dev flows continue to use explicit `make sim-public-refresh` and `make sim-public-refresh-if-stale` commands rather than unconditional regeneration on every build or dev restart.
3. The deterministic lane remains a bounded oracle or comparator lane rather than becoming a crawler; the goal is richer seeded traversal, not emergent discovery.

## Non-goals

1. Replacing Scrapling as the primary adaptive attacker lane.
2. Introducing runtime repo walking or runtime site-surface compilation.
3. Making the deterministic lane perform open-ended crawl discovery.
4. Adding a second independent freshness policy for site-surface seeding.
5. Reclassifying judge-owned realism envelopes as attacker-fitness objectives.

## Artifact Truth Contract

After this plan lands, the generated public-site artifact root must contain:

1. `.shuma/sim-public-site/site/` - the exact served docroot,
2. `.shuma/sim-public-site/manifest.json` - semantic site-generation receipt,
3. `.shuma/sim-public-site/freshness.json` - bounded regeneration freshness receipt,
4. `.shuma/sim-public-site/surface-catalog.json` - deterministic route-seeding artifact derived from the exact current docroot.

The relationship between them is:

1. `site/` is authoritative for runtime serving,
2. `manifest.json` is authoritative for semantic generation metadata,
3. `surface-catalog.json` is authoritative for deterministic public-surface seeding,
4. and all three must be regenerated together by the same explicit refresh command.

## Deterministic Seeding Contract

The synthetic lane must stop treating public-surface selection as a flat list of six roots.

Instead, the route-seeding artifact must at minimum distinguish:

1. `root_feed`
2. `section_feed`
3. `archive_page`
4. `entry_page`
5. `about_page`
6. `atom_feed`
7. `sitemap_index`
8. `sitemap_leaf`

The deterministic lane should then deterministically sample from those route families while keeping the total request budget bounded and preserving existing defense-surface probes.

The result should be a deterministic lane that can confront:

1. root or section feed surfaces,
2. pagination and archive depth,
3. deep article routes,
4. and public metadata routes such as `atom.xml` and `sitemaps/*.xml`,

without turning into a real crawler.

## Truthful Degradation Rule

If the site artifact exists but the deterministic seeding artifact is missing, unreadable, or stale, the runtime must not pretend it still has full generated-site coverage.

Allowed behavior:

1. report deterministic seeded-surface coverage as degraded or partial,
2. fall back only to a reduced, explicitly declared seed surface,
3. expose that reduced mode through status or diagnostics metadata,
4. keep operator-facing wording exact about what was actually seeded.

Forbidden behavior:

1. silently using an outdated fixed `primary_public_paths` list while presenting the lane as if it were traversing the current generated site,
2. claiming route-family or depth coverage that was not actually seeded,
3. deriving apparent current-route coverage from repo state instead of the mounted artifact.

## Task 1: Freeze The Public-Site And Deterministic-Seeding Artifact Contract

**Files:**
- Modify: `scripts/build_sim_public_site.py`
- Modify: `scripts/sim_public_site/build.py`
- Modify: `scripts/sim_public_site/__init__.py`
- Modify: `Makefile`
- Modify: generator and adversary-sim docs

**Work:**
1. Freeze the rule that deterministic public-surface seeding is derived from the mounted generated site artifact, not from source markdown directly and not from a separately refreshed manual cache.
2. Define the canonical sibling artifact path for the seeded surface catalog.
3. Record the artifact relationship and refresh semantics in the generator contract and contributor docs.
4. Make the refresh commands the sole canonical entrypoint for keeping the site and its deterministic seed surface in sync.

**Acceptance criteria:**
1. The contract explicitly states that `site/`, `manifest.json`, `freshness.json`, and `surface-catalog.json` are regenerated together.
2. The contract explicitly forbids a manual out-of-band catalog refresh path.
3. The contract explicitly states that deterministic public-surface seeding is mounted-artifact based rather than repo-state based.

**Proof:**
1. Docs and help text reflect the sibling artifact contract.
2. Focused generator-contract tests prove the canonical paths and refresh semantics.

## Task 2: Emit A Sibling Surface Catalog During Public-Site Refresh

**Files:**
- Modify: `scripts/build_sim_public_site.py`
- Modify: `scripts/sim_public_site/build.py`
- Reuse or modify: `scripts/build_site_surface_catalog.py`
- Reuse or modify: `scripts/site_surface_catalog.py`
- Modify: `Makefile`
- Modify: `scripts/tests/test_build_sim_public_site.py`
- Modify: `scripts/tests/test_build_site_surface_catalog.py`

**Work:**
1. After the site docroot is written, compile a deterministic `surface-catalog.json` from that exact docroot as part of the same build flow.
2. Ensure the output contains route inventory for deep article pages, archive pages, and metadata routes exposed by the generated site.
3. Record catalog metadata in `manifest.json`, at minimum path and hash or equivalent deterministic version signal.
4. Keep the compiler bounded to the local generated docroot rather than performing discovery beyond the mounted artifact.

**Acceptance criteria:**
1. `make sim-public-refresh` and `make sim-public-refresh-if-stale` always produce a synchronized site artifact and surface catalog together.
2. The catalog includes deep generated-site routes rather than only section roots.
3. `manifest.json` carries enough metadata to identify the exact catalog paired to that site generation.
4. The catalog can be deterministically reproduced from the same generated docroot.

**Proof:**
1. Focused Python tests prove the catalog is emitted during refresh.
2. Focused tests prove the emitted catalog includes entry routes, archive routes, and sitemap-driven metadata routes from the generated site shape.

## Task 3: Replace Fixed Deterministic Public Paths With Artifact-Derived Seeding

**Files:**
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `src/admin/adversary_sim_corpus.rs`
- Modify: `src/admin/adversary_sim.rs`
- Modify: `scripts/tests/adversarial/deterministic_attack_corpus.v1.json`
- Modify: relevant adversary-sim tests

**Work:**
1. Remove the assumption that deterministic public-surface seeding is a fixed six-path list.
2. Load deterministic public-surface seed candidates from the generated sibling artifact.
3. Keep deterministic selection reproducible via `run_id + tick + slot`.
4. Preserve existing supplemental defense probes and lane budgets.
5. Update the corpus contract so the deterministic lane truthfully represents artifact-derived public-surface seeding rather than hard-coded shallow roots.

**Acceptance criteria:**
1. The deterministic lane samples from the generated site artifact’s route families rather than only from the current six-path root list.
2. The lane can seed archive depth, deep entry pages, and metadata routes while remaining deterministic.
3. Existing bounded request-count and supplemental-lane contracts remain intact.
4. Tests no longer freeze the deterministic lane to the legacy six-path seed set.

**Proof:**
1. Focused Rust tests prove deterministic seeded paths can include archive and entry routes from the generated artifact.
2. Existing deterministic repeatability and adversary-sim regression gates remain green.

## Task 4: Add Truthful Diagnostics For Seeded-Surface Coverage And Degradation

**Files:**
- Modify: `src/admin/api.rs`
- Modify: `src/admin/adversary_sim_*` status or diagnostics surfaces as needed
- Modify: dashboard or operator docs only if already showing deterministic coverage claims
- Modify: relevant tests

**Work:**
1. Report the active deterministic surface-seeding artifact identity, at minimum hash or version.
2. Report whether deterministic public-surface seeding is full, reduced, or degraded.
3. Expose route-coverage or family-coverage diagnostics without fabricating unobserved depth.
4. When fallback is necessary, make the reduced seed surface explicit to operators.

**Acceptance criteria:**
1. Status and diagnostics make it possible to tell which public-surface artifact the deterministic lane is using.
2. Operators are not misled into believing deep generated-site coverage exists when only a reduced seed set is available.
3. Presentation surfaces remain faithful to recorded evidence and actual loaded artifacts.

**Proof:**
1. Focused API or runtime tests prove the catalog identity and degradation state are surfaced truthfully.

## Task 5: Tighten Makefile Verification And Documentation Around The New Contract

**Files:**
- Modify: `Makefile`
- Modify: `docs/adversarial-operator-guide.md`
- Modify: `docs/plans/README.md`
- Modify: any targeted sim-public or adversary-sim operator docs

**Work:**
1. Add or refine focused `make` targets for the synchronized site plus catalog build contract and the deterministic-lane seed-surface contract.
2. Document the one true refresh path for contributors.
3. Keep docs explicit that Shuma’s deterministic lane attacks the currently mounted public artifact, not an inferred repo surface.

**Acceptance criteria:**
1. Contributors have one documented refresh path for keeping the public site and deterministic seeding in sync.
2. The new plan and operator docs clearly distinguish semantic site generation from deterministic attacker seeding.
3. Verification can be run through focused `make` targets rather than broad unrelated suites.

**Proof:**
1. New or refined focused `make` targets exist and are documented.
2. Docs and plan index reflect the new artifact-sync contract.

## Suggested Implementation Order

1. Freeze the artifact contract and sibling catalog location.
2. Emit the sibling catalog during `sim-public-refresh`.
3. Add focused tests for synchronized build outputs.
4. Move deterministic public-surface seeding off fixed roots and onto the artifact-derived catalog.
5. Add truthful diagnostics for artifact identity and degraded seeding.
6. Close out docs and focused `make` verification wiring.

## Success Condition

This plan is complete when the deterministic lane no longer attacks a simplified remembered public surface, but instead attacks the exact generated public site Shuma is currently serving, and does so without hidden sync debt, fabricated coverage claims, or a second independently drifting refresh cycle.
