Date: 2026-03-23
Status: Research review

Related context:

- [`../README.md`](../README.md)
- [`../current-system-architecture.md`](../current-system-architecture.md)
- [`README.md`](README.md)
- [`../../README.md`](../../README.md)
- [`2026-03-23-documentation-audit-and-information-architecture-review.md`](2026-03-23-documentation-audit-and-information-architecture-review.md)
- [`2026-03-23-archive-directory-flattening-review.md`](2026-03-23-archive-directory-flattening-review.md)

# Purpose

Capture the follow-on shortfall in the documentation housekeeping work: the repository top-level `README.md` remained out of sync with the reorganized docs front doors, especially in its Documentation section.

# Findings

## 1. The main README remained outside the previous cleanup scope

The 2026-03-23 documentation cleanup tranches improved `docs/README.md`, `docs/research/README.md`, `docs/plans/README.md`, and related entry docs, but they did not revise the repository root `README.md`.

## 2. The README Documentation section still projects stale emphasis

The current Documentation section still:

1. links directly to `docs/research/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`,
2. surfaces deferred Akamai/Fermyon skills inline with the main shared-host-first path,
3. omits `docs/current-system-architecture.md`,
4. and does not point readers toward the new current-mainline documents that define the live-proven shared-host loop.

## 3. This is a real information-architecture gap, not a cosmetic one

For many readers, the top-level `README.md` is the actual front door. Leaving it structurally behind the rest of the docs reorganization weakens the whole cleanup effort and makes the repo feel less coherent than it now is.

# Implications

The README needs a small dedicated follow-on slice:

1. keep the Documentation section aligned to the shared-host-first mainline,
2. surface the current architecture and feedback-loop docs first,
3. keep deferred edge available but explicitly marked as later/deferred,
4. and route readers to the docs indexes instead of a stale one-off historical proof note.

# Recommended Slice

1. Add a short README-focused implementation plan.
2. Rewrite only the Documentation section of the top-level `README.md`.
3. Preserve useful links, but regroup them into current mainline, operator/product references, skills, and backlog.
4. Replace the direct historical Fermyon blocker link with `docs/deferred-edge-gateway.md`.
