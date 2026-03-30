# Contributor-Generated Public-Content Sim Site Implementation Plan

Date: 2026-03-30
Status: Active implementation plan

Related context:

- [`../research/2026-03-30-contributor-generated-public-content-sim-site-review.md`](../research/2026-03-30-contributor-generated-public-content-sim-site-review.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](./2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](./2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
- [`2026-03-20-shared-host-seed-contract.md`](./2026-03-20-shared-host-seed-contract.md)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the current hard-coded `/sim/public/*` dummy site with a build-time generated contributor content site that is richer, publicly discoverable, viewable on local dev even when sim is idle, and still faithful to the no-hidden-catalog discoverability doctrine.

**Architecture:** Generate a bounded site artifact from allowlisted markdown roots during contributor workflows, then serve that artifact through the existing `sim_public` runtime surface. Keep the runtime free of repo-walking behavior and heavy markdown-rendering logic. Preserve the `/sim/public/*` public traversal prefix, add root links plus feed and article pages plus `robots.txt` and sitemap documents, and delete the old five-page dummy-site implementation once the generated path is live. Keep the first profile contributor-only; defer any later public-hosted profile to a separate follow-on.

**Tech Stack:** existing Rust runtime `sim_public` surface, build-time site generator script, allowlisted markdown roots, semantic HTML generation, minimal shared stylesheet or browser-default rendering, focused `Makefile` generation and proof targets, docs/TODO bookkeeping.

---

## Acceptance Criteria

1. Local contributor workflows expose the richer `/sim/public/*` site even when adversary sim is idle; browsing the site must no longer depend on `adversary_sim_enabled`.
2. Runtime-only workflows do not silently generate or expose the contributor site artifact.
3. The generated site is derived from allowlisted markdown roots rather than duplicated content copies or runtime repo walking.
4. The generated site exposes root links, timeline-like feed pages, article pages, `robots.txt`, and sitemap documents that materially improve public discoverability.
5. The generated HTML is semantic and well structured, and the visual treatment remains extremely minimal and hypertext-like rather than dashboard-styled.
6. The old five-page hard-coded dummy site is removed once the generated site path is live; there must be only one canonical `/sim/public/*` surface model.
7. No worker receives hidden route catalogs or simulator-only discoverability hints as part of this work.

## Task 1: Freeze The Generated-Site Contract And Contributor-Only Scope

**Files:**
- Modify: `src/runtime/sim_public.rs`
- Modify: focused contract tests for `sim_public`
- Modify: `Makefile`
- Modify: docs covering contributor/runtime flow semantics

**Work:**
1. Freeze the generated-site contract around one canonical public prefix, contributor-only first profile, and truthful fallback behavior when the artifact is absent.
2. Make the availability boundary explicit: local contributor browsing depends on generated-artifact presence, not on adversary-sim control state.
3. Keep runtime-only workflows out of scope for auto-generation in the first tranche.

**Acceptance criteria:**
1. The contract explicitly forbids runtime repo walking.
2. The contract explicitly forbids tying site visibility to sim-run activity.
3. The contract explicitly distinguishes contributor flows from runtime-only flows.

**Proof:**
1. Add and pass `make test-sim-public-generated-site-contract`.

## Task 2: Build The Generator And Content Artifact

**Files:**
- Create: build-time generator script and related templates or helpers
- Create: generated artifact location under ignored/generated content output
- Modify: `Makefile`
- Modify: contributor docs

**Work:**
1. Generate the site from allowlisted markdown roots:
   - `README.md`
   - `docs/**/*.md`
   - `todos/todo.md`
   - `todos/blocked-todo.md`
   - `todos/completed-todo-history.md`
2. Exclude `todos/security-review.md` and any private or generated non-source content from the first profile.
3. Render:
   - one root page,
   - timeline-like feed pages,
   - article pages,
   - and compact metadata needed for sitemap generation and runtime serving.
4. Keep presentation intentionally minimal. Prefer browser defaults plus at most one tiny shared stylesheet for readability.

**Acceptance criteria:**
1. The artifact contains no duplicated source tree; it is a generated serving representation.
2. Feed ordering and excerpts are deterministic.
3. HTML structure is semantic and crawlable.
4. Visual styling remains minimal enough that the site reads like hypertext, not a custom app.

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
2. Ensure the root page and feed pages create meaningful public traversal depth.
3. Wire contributor flows so `make setup`, `make build`, and `make dev` generate or refresh the contributor site automatically.
4. Keep `make setup-runtime` and `make run-prebuilt` free from accidental contributor-site generation.

**Acceptance criteria:**
1. A contributor can browse the site locally on `make dev` without first running adversary sim.
2. The new site materially improves public discoverability through root links, feeds, `robots.txt`, and sitemap documents.
3. Runtime-only setup remains clean and unsurprising.

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
