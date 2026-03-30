# Contributor-Generated Public-Content Sim Site Review

Date: 2026-03-30
Status: Active research driver

Related context:

- [`2026-03-30-adversary-lane-wild-traffic-gap-review.md`](./2026-03-30-adversary-lane-wild-traffic-gap-review.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md`](../plans/2026-03-29-observed-telemetry-truth-and-scrapling-discoverability-plan.md)
- [`../plans/2026-03-20-shared-host-seed-contract.md`](../plans/2026-03-20-shared-host-seed-contract.md)
- [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs)
- [`../../Makefile`](../../Makefile)
- [`../../Cargo.toml`](../../Cargo.toml)

## Purpose

Decide how to replace the current thin `/sim/public/*` dummy surface with a richer site that is:

1. materially deeper and more publicly discoverable for adversary realism,
2. contributor-visible on local `make dev` even when adversary sim is idle,
3. faithful to the no-hidden-catalog discoverability doctrine,
4. cheap to serve and maintain,
5. and clean enough that the old hard-coded dummy site can be removed rather than linger beside it.

## Current State

The current dummy site is intentionally tiny.

`src/runtime/sim_public.rs` hard-codes five pages:

1. `landing`
2. `docs`
3. `pricing`
4. `contact`
5. `search`

That graph is useful as a seed harness, but it is too shallow for the richer traversal and scrape behavior now expected by `SIM-REALISM-2F..2H`.

The current availability contract is also wrong for contributor browsing. `sim_public` only serves when adversary sim is available and enabled, so the site disappears when a contributor is simply running the dev server without sim activity. That couples a content surface to the sim-control plane in a way that does not match the intended use.

Finally, the runtime currently has no markdown-rendering dependency in `Cargo.toml`. That is a good thing. It means the clean solution should avoid teaching the Spin runtime to walk the repo and render markdown dynamically.

## Requirements Clarified By The Latest Direction

The requested site should:

1. act as a richer traversal target for Scrapling and later Agentic Traffic,
2. be derived from existing project content instead of duplicating it,
3. publish timeline-oriented feeds and article pages from allowlisted markdown roots,
4. use modern semantic HTML,
5. keep CSS and visual design extremely minimal, closer to early hypertext than to a styled dashboard,
6. be available to contributors on local dev even when sim is not running,
7. and remove the old five-page dummy site rather than preserving both models.

One further scope clarification matters: this is best treated as contributor-only first. A future public profile may still be worthwhile, but it should not be bundled into the first implementation.

## Options Considered

### Option 1: Runtime repo walker and live markdown renderer

Have the Spin runtime walk allowlisted directories and render markdown on request.

Rejected.

Why:

1. it couples request handling to repo filesystem shape,
2. it makes runtime behavior differ between contributor and runtime-only environments,
3. it encourages accidental exposure of more content than intended,
4. and it would require either new runtime dependencies or ad hoc markdown logic in the serving path.

This is the opposite of the repo’s current direction toward bounded, deterministic contributor tooling and truthful runtime surfaces.

### Option 2: Build-time generated contributor site served by runtime

Generate a compact site artifact from allowlisted markdown roots during contributor flows, then have `sim_public` serve that artifact.

Recommended.

Why:

1. source content remains the existing markdown files,
2. the runtime serves a deterministic artifact rather than walking the repo,
3. contributor flows can generate it automatically while runtime-only flows can omit it,
4. the generated site can expose root links, feeds, article pages, `robots.txt`, and sitemap documents for richer discoverability,
5. and the old hard-coded dummy site can be deleted cleanly once the new artifact path lands.

### Option 3: Separate external static-site pipeline first

Build a separate site outside the runtime and use that as the richer traversal target.

Not recommended for the first tranche.

Why:

1. it adds deployment and content-governance work before we have validated the content shape locally,
2. it does not solve the immediate contributor-local realism need,
3. and it risks bundling public-content policy questions into what should first be a contributor-only replacement for the dummy site.

## Recommendation

Replace the current hard-coded dummy site with a build-time generated contributor content site.

Recommended first profile:

1. `README.md` rendered as a separate `About` page
2. dated research entries in `docs/research/2026-*.md`
3. dated plan entries in `docs/plans/2026-*.md`
4. dated completion history from `todos/completed-todo-history.md`

Recommended exclusions for the first contributor profile:

1. `todos/todo.md`
2. `todos/blocked-todo.md`
3. `todos/security-review.md`
4. undated general docs outside the explicit first-profile allowlist
5. any secret-bearing local files
6. any generated artifacts that are not source-of-truth markdown

The generated site should:

1. keep the `/sim/public/*` serving prefix so existing discovery and realism work can build on one canonical public surface,
2. use the homepage as a dated latest or all-entries feed rather than as the `README` or About page,
3. expose section feed pages for research, plans, and completed work, plus article pages, `robots.txt`, and sitemap documents,
3. render semantic HTML using elements like `main`, `header`, `nav`, `section`, `article`, `aside`, `time`, and `footer`,
4. use either no CSS or one extremely small shared stylesheet that mostly preserves browser defaults,
5. and derive excerpts and ordering at generation time.

## Availability And Environment Contract

The first implementation should be contributor-only by default.

That means:

1. contributor flows such as `make setup`, `make build`, and `make dev` should generate or refresh the contributor site automatically,
2. runtime-only flows such as `make setup-runtime` and `make run-prebuilt` should not silently generate it,
3. and `sim_public` availability for contributor browsing should be tied to generated-site presence, not to `adversary_sim_enabled`.

This keeps the contributor traversal surface available locally without granting runtime-only operators a surprise content site they did not build or ask for.

## Architectural Guardrails

1. No runtime repo walking.
2. No hidden worker route catalogs.
3. No simulator-only hints added to emitted traffic.
4. Generated site content must come from allowlisted source markdown, not duplicated copies.
5. The homepage must be a chronology-driven feed, while `README.md` remains a separate About page rather than becoming the root index.
6. The runtime must fail closed truthfully when the artifact is absent: either a minimal explicit unavailable response or no route, but never a fabricated richer site.
7. The current five-page dummy site should be removed once the generated path is live, so there is only one public traversal model.

## Consequence For The Realism Chain

`SIM-REALISM-2H` should no longer be interpreted as “make the current five-page blog slightly bigger.” The better implementation vehicle is now explicit:

1. contributor-generated site artifact,
2. richer public discoverability through root links, feeds, `robots.txt`, and sitemap documents,
3. semantic hypertext-style presentation,
4. and a clean replacement of the old hard-coded dummy surface.

Any later public or production-hosted profile should be a separate, curated follow-on after the contributor profile proves useful locally.
