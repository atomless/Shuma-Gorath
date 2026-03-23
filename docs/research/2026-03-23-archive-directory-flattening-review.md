Date: 2026-03-23
Status: Research review

Related context:

- [`2026-03-23-documentation-audit-and-information-architecture-review.md`](2026-03-23-documentation-audit-and-information-architecture-review.md)
- [`../plans/2026-03-23-documentation-audit-and-reorganization-plan.md`](../plans/2026-03-23-documentation-audit-and-reorganization-plan.md)
- [`../README.md`](../README.md)
- [`README.md`](README.md)
- [`../plans/README.md`](../plans/README.md)
- [`../deferred-edge-gateway.md`](../deferred-edge-gateway.md)

# Purpose

Review the follow-on docs housekeeping question: now that clearly defunct deferred-edge material has been moved out of the active front door, should `docs/research/` and `docs/plans/` remain as nested folders, or should dated research and plan docs return to flat top-level directories?

# Findings

## 1. The dated filename convention already carries the chronological grouping signal

Every document in `docs/research/` and `docs/plans/` is already date-prefixed. The extra `archive/` nesting does not add meaningful discoverability for readers who are primarily scanning by date and topic.

## 2. The nested archive layout adds path complexity without enough navigational value

The current structure forces readers and backlinks to distinguish between:

1. `docs/research/`
2. `docs/research/`
3. `docs/research/archive/outdated/`
4. `docs/plans/`
5. `docs/plans/`
6. `docs/plans/archive/outdated/`

That distinction made sense for the first cleanup slice, but it now creates a second discoverability problem: people have to remember both the date and a special subdirectory even though the real meaningful distinction is semantic, not structural.

## 3. The semantic distinction still matters

The project still benefits from distinguishing:

1. current planning and research drivers,
2. historical delivered baselines,
3. and outdated deferred-edge notes that are kept only as historical receipts.

What matters is preserving that status in indexes and entry docs, not preserving it as nested filesystem levels.

## 4. The indexes are now strong enough to carry that meaning

`docs/README.md`, `docs/research/README.md`, `docs/plans/README.md`, and `docs/deferred-edge-gateway.md` can carry the “current vs historical vs outdated deferred-edge” split without needing separate archive folders. That keeps the filesystem flat while still keeping the information architecture explicit.

# Implications

The cleanest next docs slice is:

1. move all dated docs from `docs/research/**` into `docs/research/`,
2. move all dated docs from `docs/plans/**` into `docs/plans/`,
3. delete the now-redundant archive README files and empty archive directories,
4. and rewrite the top-level indexes so they explicitly call out historical baselines and outdated deferred-edge notes in-place.

# Recommended Slice

1. Add one small execution plan dedicated to archive flattening.
2. Move all archived dated docs up one directory level.
3. Update links repo-wide so no active docs still point at `research/archive` or `plans/archive`.
4. Remove the archive subdirectories entirely once the link graph is repaired.
5. Record the cleanup in completed TODO history as a second docs-housekeeping tranche.
