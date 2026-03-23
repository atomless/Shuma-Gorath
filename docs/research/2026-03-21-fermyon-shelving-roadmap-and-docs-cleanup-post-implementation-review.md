Date: 2026-03-21
Status: Post-implementation review

Related context:

- [`2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`../plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](../plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../README.md`](../../README.md)
- [`../../docs/deployment.md`](../../docs/deployment.md)
- [`../../docs/bot-defence.md`](../../docs/bot-defence.md)
- [`../../docs/README.md`](../../docs/README.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Scope Reviewed

Shelve Fermyon as a near-term full-runtime target, re-center the pre-launch roadmap on a shared-host-first control plane, and clean up the docs, plans, and backlog to match that decision.

# What Landed

1. Added a repo-grounded architecture review capturing what should be kept, deferred, and cleaned up after the shared-host-first decision.
2. Added a direction-update plan that defines the shared-host control plane versus deferred edge gateway split.
3. Updated the master roadmap and older architecture plans so they no longer present edge/Fermyon work as the current pre-launch mainline.
4. Moved edge/Fermyon follow-on work out of the active TODO queue into blocked backlog items, and added explicit later architecture-cleanup blockers for the deferred edge gateway track.
5. Refreshed public docs and Akamai/Fermyon agent skills so they now describe the edge path as deferred gateway-only work rather than the near-term full-runtime destination.

# Verification Evidence

1. `git diff --check`

Verification note:

1. This was a docs/plans/backlog-only tranche, so behavior tests were intentionally not run.

# Plan Versus Implementation

The tranche met the plan:

1. the review and replacement plan were written before cleanup,
2. the roadmap and backlog were re-sequenced around the shared-host-first loop,
3. the public edge story was made truthful,
4. and the remaining code-level Fermyon-shaped assumptions were captured as later blocked cleanup rather than silently ignored.

# Shortfalls

No new tranche-local shortfall was found.

One intentional non-goal remains:

1. this tranche did not refactor the code-level `edge-fermyon` assumptions in runtime, dashboard, or Makefile surfaces. Those are now explicitly captured as deferred follow-on architecture work rather than being left as undocumented drift.

# Next Recommended Step

Continue the shared-host-first pre-launch mainline and do not reopen edge-runtime or edge-expansion work until the later edge gateway plus shared-host control-plane architecture is planned explicitly.
