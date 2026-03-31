# Contributor-Generated Public-Content Sim Site Implementation Plan

Date: 2026-03-30
Status: Partially superseded on 2026-03-31 by the route-namespace correction plan

Related context:

- [`2026-03-31-route-namespace-and-root-hosted-public-site-plan.md`](./2026-03-31-route-namespace-and-root-hosted-public-site-plan.md)
- [`../research/2026-03-30-contributor-generated-public-content-sim-site-review.md`](../research/2026-03-30-contributor-generated-public-content-sim-site-review.md)
- [`../research/2026-03-30-generated-public-content-site-standards-and-generator-pattern-review.md`](../research/2026-03-30-generated-public-content-site-standards-and-generator-pattern-review.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](./2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](./2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
- [`2026-03-20-shared-host-seed-contract.md`](./2026-03-20-shared-host-seed-contract.md)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

> **Correction note (2026-03-31):** this plan remains relevant for the generated-site build discipline, contributor-local artifact model, and minimal semantic rendering requirements, but its assumption that the generated site should remain under `/sim/public/*` is now superseded by [`2026-03-31-route-namespace-and-root-hosted-public-site-plan.md`](./2026-03-31-route-namespace-and-root-hosted-public-site-plan.md). Future implementation should follow the newer root-hosted public-site plus `/shuma/*` control-plane namespace contract.

**Goal:** Replace the current hard-coded `/sim/public/*` dummy site with a build-time generated contributor content site that is richer, publicly discoverable, viewable on local dev even when sim is idle, and still faithful to the no-hidden-catalog discoverability doctrine.

**Architecture:** Generate a bounded site artifact from allowlisted dated markdown roots during contributor workflows, then serve that artifact through the existing `sim_public` runtime surface. Keep the runtime free of repo-walking behavior and heavy markdown-rendering logic. Use a small build-time CLI plus shared helper module, following the repo’s existing deterministic generator pattern. Preserve the `/sim/public/*` public traversal prefix, use the homepage as a chronology-driven latest or all-entries feed, render `README.md` as a separate `About` page, add section feeds plus article pages plus `robots.txt`, sitemap documents, and an Atom feed, and delete the old five-page dummy-site implementation once the generated path is live. Keep the first profile contributor-only; defer any later public-hosted profile to a separate follow-on.

**Tech Stack:** existing Rust runtime `sim_public` surface, build-time site generator script, allowlisted markdown roots, semantic HTML generation, minimal shared stylesheet or browser-default rendering, focused `Makefile` generation and proof targets, docs/TODO bookkeeping.

---

## Canonical Directory And File Contract

The first implementation must keep the generated-site toolchain and artifacts visibly separate from Shuma internals.

Canonical locations:

1. build-time CLI entrypoint: `scripts/build_sim_public_site.py`
2. generator implementation package: `scripts/sim_public_site/`
3. corpus allowlist and section policy: `config/sim_public_site/corpus.toml`
4. generated contributor-local artifact root: `.shuma/sim-public-site/`
5. runtime serving adapter: `src/runtime/sim_public.rs`

Canonical generated artifact layout:

1. `.shuma/sim-public-site/manifest.json`
2. `.shuma/sim-public-site/freshness.json`
3. `.shuma/sim-public-site/site/index.html`
4. `.shuma/sim-public-site/site/about/index.html`
5. `.shuma/sim-public-site/site/research/...`
6. `.shuma/sim-public-site/site/plans/...`
7. `.shuma/sim-public-site/site/work/...`
8. `.shuma/sim-public-site/site/sitemap.xml`
9. `.shuma/sim-public-site/site/atom.xml`

Non-goals for this tranche:

1. no generated HTML committed under `docs/`
2. no generated site files under `src/`
3. no contributor-site output under `dist/`
4. no runtime repo walking
5. no runtime markdown rendering

This contract keeps the generated site obviously identifiable as contributor-local public terrain rather than Shuma core runtime code.

## Acceptance Criteria

1. Local contributor workflows expose the richer `/sim/public/*` site even when adversary sim is idle or disabled; browsing the site must no longer depend on `adversary_sim_enabled`, and contributors must be able to use that surface under `make dev` as the local human-friction assessment terrain for current config.
2. Runtime-only workflows do not silently generate or expose the contributor site artifact.
3. The generated site is derived from allowlisted markdown roots rather than duplicated content copies or runtime repo walking.
4. The generated site exposes a chronology-driven homepage feed, section feeds, article pages, `robots.txt`, and sitemap documents that materially improve public discoverability.
5. The generated HTML is semantic and well structured, and the visual treatment remains extremely minimal and hypertext-like rather than dashboard-styled.
6. The old five-page hard-coded dummy site is removed once the generated site path is live; there must be only one canonical `/sim/public/*` surface model.
7. No worker receives hidden route catalogs or simulator-only discoverability hints as part of this work.
8. Feed pages and archives are reachable through ordinary `<a href>` links and paginated URLs rather than JS-only navigation.
9. Entry and feed pages emit absolute canonical URLs, and timestamps are rendered with semantic `<time datetime>` markup.
10. Markdown rendering is performed by a real CommonMark-conforming build-time parser, not ad hoc string or regex transforms in the runtime.
11. Normal contributor builds and dev restarts do not regenerate the site artifact by default; refresh happens only on explicit command or bounded stale-check policy.
12. Generator source, corpus policy, generated artifacts, and runtime serving code live only in the canonical locations defined above rather than being mixed into `docs/`, `src/`, or `dist/`.

## Task 1: Freeze The Generated-Site Contract And Contributor-Only Scope

**Files:**
- Create: `scripts/build_sim_public_site.py`
- Create: `scripts/sim_public_site/`
- Create: `config/sim_public_site/corpus.toml`
- Modify: `src/runtime/sim_public.rs`
- Modify: focused contract tests for `sim_public`
- Modify: `Makefile`
- Modify: docs covering contributor/runtime flow semantics

**Work:**
1. Freeze the generated-site contract around one canonical public prefix, contributor-only first profile, and truthful fallback behavior when the artifact is absent.
2. Make the availability boundary explicit: local contributor browsing depends on generated-artifact presence, not on adversary-sim control state.
3. Keep runtime-only workflows out of scope for auto-generation in the first tranche.
4. Freeze the URL and markup contract:
   - chronology-driven root feed
   - `About` page from `README.md`
   - section feeds
   - entry pages
   - paginated archives
   - canonical URLs
   - semantic timestamps
5. Freeze the repository-separation contract:
   - generator source under `scripts/`
   - corpus policy under `config/`
   - generated site artifact under `.shuma/`
   - runtime serving adapter under `src/runtime/sim_public.rs`
   - no generated pages under `docs/`, `src/`, or `dist/`

**Acceptance criteria:**
1. The contract explicitly forbids runtime repo walking.
2. The contract explicitly forbids tying site visibility to sim-run activity.
3. The contract explicitly distinguishes contributor flows from runtime-only flows.
4. The contract explicitly requires ordinary crawlable anchor links for pagination and archive traversal.
5. The contract explicitly requires canonical URLs and semantic `time` markup.
6. The contract explicitly forbids unconditional regeneration on every `make build` or `make dev` restart.
7. The contract explicitly freezes the canonical generator, config, artifact, and runtime locations named above.
8. The contract explicitly forbids generated site files from being stored under `docs/`, `src/`, or `dist/`.

**Proof:**
1. Add and pass `make test-sim-public-generated-site-contract`.

## Task 2: Build The Generator And Content Artifact

**Files:**
- Create: `scripts/build_sim_public_site.py`
- Create: `scripts/sim_public_site/`
- Create: `config/sim_public_site/corpus.toml`
- Create: generated artifact location under `.shuma/sim-public-site/`
- Modify: `Makefile`
- Modify: contributor docs

**Work:**
1. Generate the site from allowlisted source content:
   - `README.md` as a separate `About` page
   - dated research entries in `docs/research/2026-*.md`
   - dated plan entries in `docs/plans/2026-*.md`
   - dated completion history from `todos/completed-todo-history.md`
2. Exclude active backlog, blocked backlog, security review material, and undated general docs from the first profile.
3. Render:
   - one chronology-driven root page,
   - section feed pages,
   - article pages,
   - one `About` page,
   - one site Atom feed and optional section feeds,
   - and compact metadata needed for sitemap generation and runtime serving.
4. Keep presentation intentionally minimal. Prefer browser defaults plus at most one tiny shared stylesheet for readability.
5. Use a CommonMark-conforming parser at build time rather than ad hoc markdown-to-HTML transforms.

**Acceptance criteria:**
1. The artifact contains no duplicated source tree; it is a generated serving representation.
2. Feed ordering and excerpts are deterministic.
3. The root page is a dated feed and `README.md` is exposed separately as `About`.
4. HTML structure is semantic and crawlable.
5. Visual styling remains minimal enough that the site reads like hypertext, not a custom app.
6. Atom feed output is standards-based and reflects the latest chronology stream.
7. The generated artifact lives entirely under `.shuma/sim-public-site/` with no rendered pages committed under `docs/`, `src/`, or `dist/`.

**Proof:**
1. Add and pass `make test-sim-public-generator`.

## Task 3: Serve The Generated Site Through `sim_public` And Remove The Old Dummy Site

**Files:**
- Modify: `src/runtime/sim_public.rs`
- Remove or replace: old hard-coded five-page dummy-site logic
- Modify: tests and docs that still assume `landing/docs/pricing/contact/search`

**Work:**
1. Teach `sim_public` to serve the generated artifact.
2. Remove the old hard-coded page enum, legacy graph, and legacy content copy once the generated path is working.
3. Keep the runtime fallback truthful when the generated artifact is absent.

**Acceptance criteria:**
1. `/sim/public/*` serves the generated contributor site when the artifact exists.
2. The old five-page dummy site no longer exists as a parallel public surface.
3. Any unavailable state is explicit and truthful rather than silently falling back to the old fake site.
4. Runtime serving code remains a narrow adapter over the generated artifact rather than absorbing markdown-rendering or repo-walking responsibilities.

**Proof:**
1. Keep `make test-sim-public-generated-site-contract` green.
2. Add and pass `make test-sim-public-runtime-serving`.

## Task 4: Add Discoverability Artifacts And Local Dev Flow Wiring

**Files:**
- Modify: generator outputs
- Modify: `Makefile`
- Modify: contributor docs
- Modify: discoverability and shared-host proof targets as needed

**Work:**
1. Generate `robots.txt` and sitemap documents for the new site.
2. Ensure the root page and section feeds create meaningful public traversal depth.
3. Add explicit contributor refresh commands, for example:
   - `make sim-public-refresh`
   - `make sim-public-refresh-if-stale`
4. Store generation metadata so the stale-check path can skip rebuilds until a bounded threshold or source-change rule is crossed.
5. Wire contributor flows so `make dev` and `make build` serve the existing artifact without forcing regeneration on every invocation.
6. Keep `make setup-runtime` and `make run-prebuilt` free from accidental contributor-site generation.
7. Ensure archive and pagination pages remain link-driven and crawlable without JavaScript.

**Acceptance criteria:**
1. A contributor can browse the site locally on `make dev` without first running adversary sim, including when adversary sim is disabled or idle, so the generated `/sim/public/*` surface can be used for local human-friction assessment against the active config.
2. The new site materially improves public discoverability through a dated root feed, section feeds, `robots.txt`, and sitemap documents.
3. Runtime-only setup remains clean and unsurprising.
4. Feed traversal, archive traversal, and entry traversal all work through ordinary hyperlinks alone.
5. Default contributor build/dev paths do not trigger expensive regeneration when a usable artifact already exists.
6. Contributors have an explicit refresh path and an optional bounded stale-refresh path instead of an always-rebuild policy.

**Proof:**
1. Add and pass `make test-sim-public-build-flow-contract`.
2. Keep `make test-sim-public-discoverability-contract` green.
3. Keep shared-host seed-contract proof green.

## Task 5: Align The Realism And Observer Contracts

**Files:**
- Modify: adversary-sim docs and realism docs
- Modify: any observer or receipt docs/tests touched by the richer site

**Work:**
1. Update the realism chain so `SIM-REALISM-2H` is explicitly implemented through the generated site rather than through ad hoc page accretion.
2. Keep the no-hidden-catalog rule explicit.
3. Keep the public-content site framed as contributor-only in this tranche.

**Acceptance criteria:**
1. Docs and backlog stop describing the dummy-site work as a generic “bigger fake blog.”
2. The contributor-only scope and later-public deferral are explicit.
3. Discoverability remains public-content-derived rather than worker-hint-derived.

**Proof:**
1. Update and keep docs/TODO references consistent.
2. Keep the focused sim-public and shared-host proof targets green when implementation lands.

## Deferred Follow-On

Any later public or production-hosted profile should be a separate tranche after the contributor profile proves useful.

That later work should answer:

1. which content roots are safe and valuable on a public Shuma property,
2. whether a separate hostname is preferable,
3. and what deploy-time gating or curation contract should replace the contributor allowlist.
