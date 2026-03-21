# SIM-SCR-7 Dashboard Lane Controls Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-sim-scr-6-scrapling-worker-post-implementation-review.md`](./2026-03-20-sim-scr-6-scrapling-worker-post-implementation-review.md)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/runtime/dashboard-adversary-sim.js`](../../dashboard/src/lib/runtime/dashboard-adversary-sim.js)
- [`../../dashboard/src/lib/runtime/dashboard-red-team-controller.js`](../../dashboard/src/lib/runtime/dashboard-red-team-controller.js)
- [`../../dashboard/src/routes/+page.svelte`](../../dashboard/src/routes/+page.svelte)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../e2e/dashboard.modules.unit.test.js`](../../e2e/dashboard.modules.unit.test.js)
- [`../../e2e/dashboard.smoke.spec.js`](../../e2e/dashboard.smoke.spec.js)
- [`../../Makefile`](../../Makefile)
- [`../../docs/adversarial-operator-guide.md`](../../docs/adversarial-operator-guide.md)
- [`../../docs/testing.md`](../../docs/testing.md)

## Review Goal

Confirm that `SIM-SCR-7` landed the operator-facing side of the settled lane contract:

1. the Dashboard Red Team tab exposes lane selection without replacing the global ON/OFF control,
2. desired lane versus active lane stays honest in the rendered surface,
3. `bot_red_team` remains visible but disabled,
4. and verification now includes both focused module contracts and one rendered browser proof.

## What Was Intended

This tranche was meant to project the backend lane migration into the dashboard after the control path and real Scrapling worker were already stable.

That means:

1. no new dashboard state architecture,
2. no speculative extra lane model beyond the backend contract,
3. and no new visual language for the controls.

The right implementation had to reuse the existing red-team controller, the same control endpoint, and canonical dashboard form/status styles.

## What Landed

1. The dashboard API adapter and runtime normalizer now preserve `desired_lane`, `active_lane`, switch metadata, and bounded `lane_diagnostics`.
2. The Red Team controller gained one narrow sync hook so the page can apply accepted lane-only control responses without inventing a second polling path.
3. The Red Team tab now renders:
   - the lane selector,
   - desired lane,
   - active lane,
   - switch metadata,
   - and lane diagnostics for the active lane when running or the desired lane while off.
4. The lane selector writes through the existing `POST /admin/adversary-sim/control` path, keeps top-level enablement semantics intact, and rejects `bot_red_team` as not yet available.
5. The dashboard now has a truthful focused module gate, `make test-dashboard-adversary-sim-lane-contract`.
6. The focused Playwright adversary-sim gate now also proves the rendered lane selector contract, including off-state desired-versus-active truth and the disabled `bot_red_team` option.

## Architectural Assessment

### 1. The dashboard now matches the settled backend contract instead of translating it away

This is the main value of the tranche.

Before this slice, the backend could truthfully report lane intent and lane execution, but the dashboard still looked like a toggle-only surface.

Now the operator view preserves the same desired-versus-active split the runtime actually uses.

### 2. The implementation stayed inside existing dashboard patterns

The selector reuses the existing `input-row`, `input-field`, `control-label`, and `status-item` surfaces.

That kept the tranche aligned with the repo-wide shared-style and no-new-pattern rules.

### 3. Lane writes remain part of the existing control-plane truth

No new endpoint, route-local state machine, or shadow persistence path was introduced.

Lane selection still uses the existing adversary-sim control endpoint and read-after-write status surface, which keeps the dashboard subordinate to the same server-authoritative contract as the toggle.

## Shortfalls Found During Review

Four closeout gaps were found and corrected before completion:

1. the first pass only had source and module-contract proof for the new operator surface, but this tranche changes rendered UI and needed at least one browser-level assertion,
2. and the dashboard did not yet have a truthful focused Make target for the lane selector contract.
3. the first rendered proof seeded off-state lane state through the bearer admin helper and then immediately tried to change lane through the dashboard session, which created a short-lived cross-actor controller-lease conflict unrelated to the rendered selector contract itself,
4. and the adversary-sim Playwright restore helper was restoring enabled state and config but not `desired_lane`, which could leak Scrapling lane state into later smoke cases and make telemetry proofs flaky.

Both are now closed:

1. `make test-dashboard-e2e-adversary-sim` includes the rendered lane-selector proof,
2. and `make test-dashboard-adversary-sim-lane-contract` covers the focused adapter/runtime/controller/source-contract path.
3. the rendered proof now waits for the direct admin lease it created to expire before exercising the dashboard session write path,
4. and the restore helper now restores both desired enablement and desired lane so later dashboard smokes start from truthful state.

No new architectural shortfall remains inside `SIM-SCR-7`.

The remaining open follow-on is still `SIM-SCR-8`, where operator rollout/rollback guidance and hosted-scope deployment egress hardening notes need to be closed around the now-real lane surface.

## Result

Treat `SIM-SCR-7` as complete.

The next optimal tranche remains `SIM-SCR-8`:

1. finalize operator workflow docs,
2. keep Make targets truthful as rollout evidence accumulates,
3. and close the deployment-level hosted-scope egress guidance that is still explicitly open from `SIM-SCR-6`.
