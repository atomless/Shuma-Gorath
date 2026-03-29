Date: 2026-03-29
Status: Completed on 2026-03-29

Related context:

- [`../research/2026-03-29-tuning-surface-defunct-review.md`](../research/2026-03-29-tuning-surface-defunct-review.md)
- [`2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md`](2026-03-23-tuning-surface-taxonomy-posture-matrix-implementation-plan.md)
- [`2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md`](2026-03-24-tuning-surface-visibility-and-fingerprint-control-ownership-plan.md)
- [`2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md`](2026-03-24-identification-tab-rename-and-taxonomy-distinction-plan.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)
- [`../../todos/completed-todo-history.md`](../../todos/completed-todo-history.md)

# Objective

Retire the defunct March 23-24 Tuning expansion chain cleanly so the backlog, indexes, roadmap notes, and current tab docs all stop presenting it as active work.

# Acceptance Criteria

1. `todos/blocked-todo.md` no longer contains `TUNE-SURFACE-1`, `TUNE-SURFACE-1A`, `TUNE-SURFACE-1B`, or `TUNE-SURFACE-1C`.
2. The three March 23-24 Tuning plan docs and their companion research notes are explicitly marked `Defunct` or `Historical ... defunct` at the top of each file.
3. `docs/plans/README.md` and `docs/research/README.md` no longer list that chain in active sections and instead list it under historical or defunct sections.
4. Active roadmap and dependency docs no longer treat `TUNE-SURFACE-1` as an executable future tranche.
5. [`../dashboard-tabs/tuning.md`](../dashboard-tabs/tuning.md) describes the current narrow Tuning contract and explicitly notes that the broader March 23-24 expansion chain is retired.
6. The retirement is recorded in [`../../todos/completed-todo-history.md`](../../todos/completed-todo-history.md) as a docs-only backlog and planning cleanup rather than shipped feature work.

# Verification

Because this tranche is docs-only, verification is:

1. `git diff --check`

# Execution Summary

1. Add a dated retirement review and this retirement plan.
2. Remove the blocked `TUNE-SURFACE-1*` backlog items.
3. Mark the associated March 23-24 Tuning and Identification docs as defunct for audit only.
4. Move those docs out of active index sections and into historical or defunct sections.
5. Clean the remaining active roadmap and backlog references so they no longer point at retired tranche names.
