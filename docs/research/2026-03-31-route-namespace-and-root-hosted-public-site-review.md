# Route Namespace And Root-Hosted Public Site Review

Date: 2026-03-31
Status: Active research driver

Related context:

- [`2026-03-30-contributor-generated-public-content-sim-site-review.md`](./2026-03-30-contributor-generated-public-content-sim-site-review.md)
- [`../plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md`](../plans/2026-03-30-contributor-generated-public-content-sim-site-plan.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../../src/runtime/sim_public.rs`](../../src/runtime/sim_public.rs)
- [`../../src/lib.rs`](../../src/lib.rs)
- [`../../src/admin/adversary_sim_corpus.rs`](../../src/admin/adversary_sim_corpus.rs)
- [`../../src/crawler_policy/robots.rs`](../../src/crawler_policy/robots.rs)

## Purpose

Decide whether the generated contributor public-content site should continue to live under `/sim/public/*`, and whether Shuma-owned control or operational routes should continue to occupy top-level origin paths such as `/admin/*`, `/dashboard/*`, `/health`, and `/metrics`.

## Current State

The current implementation preserves a simulation-specific public prefix:

1. `src/runtime/sim_public.rs` serves the generated contributor site only under `/sim/public/*`.
2. `config/sim_public_site/corpus.toml` hard-codes `root_prefix = "/sim/public"`.
3. `src/crawler_policy/robots.rs` emits the default sitemap path as `/sim/public/sitemap.xml`.
4. `src/admin/adversary_sim_corpus.rs` and the deterministic adversarial corpus default many path hints and public-search assumptions to `/sim/public/*`.
5. `src/lib.rs` reserves top-level control or special surfaces such as `/admin/*`, `/health`, `/metrics`, `/dashboard/*`, and `/sim/public/*` directly in request-path handling.

This architecture was understandable when the dummy site was a tiny, isolated simulation surface and when the contributor public site was intended as a safe replacement for that same isolated surface.

However, it is no longer the cleanest architecture now that the generated site is intended to stand in for the local protected host content that adversaries should explore from the origin root.

## External Standards And Security Guidance

`RFC 3986` explains that hierarchical URIs work by relative reference within a single hierarchical namespace and that designers should preserve that hierarchy unless they have compelling reasons not to do so. In practical terms, a protected host site that is supposed to live at the origin root should not need a simulation-only path prefix when the goal is realistic traversal and discovery. Source: [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986), especially the hierarchical-relative-reference discussion.

`RFC 8615` is even more directly relevant. It explains that minting fixed top-level "well-known locations" risks collisions with resources that the origin already has or wishes to create, and that doing so usurps the origin's control over its URI space. The standard solves that problem by reserving a narrowly defined top-level `/.well-known/` prefix for origin-wide metadata rather than encouraging arbitrary bespoke top-level claims. Source: [RFC 8615](https://www.rfc-editor.org/rfc/rfc8615), especially the introduction and Section 3.

OWASP's attack-surface guidance says web applications should explicitly distinguish buckets such as admin interfaces, operational command and monitoring interfaces, and anonymous public interfaces, and it calls out major architectural changes to trust relationships and interface shape as changes that require attack-surface review. Source: [OWASP Attack Surface Analysis Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Attack_Surface_Analysis_Cheat_Sheet.html).

## Why `/sim/public/*` Is Now The Wrong Model

The prefix has become architecturally expensive:

1. It splits the protected site's logical root from the first page a simulated adversary is expected to traverse.
2. It forces deterministic corpus, realism docs, benchmark sample paths, and worker hints to talk about a simulation-only path family instead of the origin root.
3. It lets Shuma's own control-plane paths remain mixed into the host root while pushing the public site into a subordinate subtree, which is the opposite of the cleaner ownership model.
4. It increases drift between local contributor realism and what a real hosted site behind Shuma should look like.

The earlier reason for the prefix was route isolation. That was a reasonable bootstrap choice, but it is no longer the stronger architecture now that the generated site is meant to be the local protected host content surface.

## Recommended Route Ownership Model

The cleaner architecture is:

1. host content and standard public metadata at the origin root:
   - `/`
   - ordinary generated content pages under `/*`
   - `/robots.txt`
   - `/sitemap.xml`
   - `/atom.xml`
2. Shuma-owned control and operational routes under `/shuma/*`:
   - `/shuma/admin/*`
   - `/shuma/dashboard/*`
   - `/shuma/health`
   - `/shuma/metrics`
   - `/shuma/internal/*`
3. public defence surfaces that intentionally participate in the protected site surface remain outside `/shuma/*` unless a separate design later changes that policy.

That last point is an explicit assumption for the corrective plan:

1. public challenge and defence interaction routes such as challenge, PoW, maze, honeypot, or similar public-facing surfaces are treated as part of the protected host surface, not as operator-only control routes;
2. this migration is specifically about moving Shuma's control-plane, dashboard, health, metrics, and internal command surfaces out of the host's public root.

If that assumption changes later, it should be a separate follow-on route-ownership review, not bundled into the current namespace correction.

## Consequences For Existing Work

The current contributor-generated site implementation is not rotten, but its prefix assumption is now architecturally superseded.

The correction is broad because `/sim/public/*` and top-level control paths are currently embedded in:

1. runtime routing,
2. robots and sitemap generation,
3. adversary deterministic and realism path hints,
4. benchmark or observer sample paths,
5. dashboard login and shell entry paths,
6. supervisor and operational scripts,
7. tests, docs, and operator guidance.

Because the repository is pre-launch, the preferred correction is a clean cut rather than compatibility aliases or dual-route shims.

## Recommendation

Adopt a root-hosted generated public site and a dedicated `/shuma/*` namespace for Shuma-owned control or operational routes.

Specifically:

1. stop extending `/sim/public/*`,
2. move the generated contributor site to root-hosted public pages,
3. move control-plane and operational surfaces under `/shuma/*`,
4. update deterministic or realism path hints to start from `/`,
5. and remove the legacy route families once the new path contract is fully proven.

This is a large enough change that it needs its own corrective implementation plan and atomic tranches rather than being folded casually into the remaining realism tasks.
