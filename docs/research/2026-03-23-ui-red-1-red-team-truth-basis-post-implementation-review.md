# UI-RED-1 Post-Implementation Review

Date: 2026-03-23
Status: Closed

Related context:

- [`2026-03-23-dashboard-operator-surfacing-gap-review.md`](2026-03-23-dashboard-operator-surfacing-gap-review.md)
- [`../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](../plans/2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)
- [`../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte`](../../dashboard/src/lib/components/dashboard/RedTeamTab.svelte)
- [`../../dashboard/src/lib/domain/api-client.js`](../../dashboard/src/lib/domain/api-client.js)
- [`../../dashboard/src/lib/runtime/dashboard-adversary-sim.js`](../../dashboard/src/lib/runtime/dashboard-adversary-sim.js)

# Scope Reviewed

This closeout reviewed the delivered `UI-RED-1` slice:

1. Red Team surfacing of adversary-sim generation and lane-diagnostics truth basis,
2. bounded persisted-event evidence rendering when status truth was recovered from monitoring facts,
3. and the supporting dashboard adapter and runtime-normalization seams.

# What Landed

1. The dashboard API client now preserves:
   - `generation_diagnostics.truth_basis`,
   - `lane_diagnostics.truth_basis`,
   - and bounded `persisted_event_evidence`.
2. The dashboard adversary-sim runtime now normalizes those fields into the existing Red Team read model.
3. `Red Team` now renders a dedicated `Status Truth` block with:
   - generation counters basis,
   - lane diagnostics basis,
   - explicit lower-bound versus direct versus unavailable messaging,
   - and bounded persisted-event evidence when present.
4. The existing `Recent Red Team Runs` panel remained unchanged and still owns compact run-history projection.
5. A focused make gate now proves the adapter, runtime, and rendered lower-bound state together.

# Review Result

The delivered slice matches the intended ownership split:

1. local adversary-sim status truth now lives in `Red Team`,
2. the bounded evidence is subordinate to lifecycle and run-history surfaces,
3. and the work does not pull `operator_snapshot_v1`, `benchmark_results_v1`, or oversight projection forward out of `MON-OVERHAUL-1`.

The implementation also stayed aligned with the existing dashboard architecture:

- status truth still arrives through the existing adversary-sim status read path,
- normalization remains in the shared dashboard adversary-sim runtime module,
- and the UI reuses the current status-row and message patterns rather than inventing new local components.

# Shortfalls Found

One tranche-local shortfall appeared during implementation:

1. the first pass preserved `generation_diagnostics.truth_basis` and persisted-event evidence but still dropped `lane_diagnostics.truth_basis` inside `adaptLaneDiagnostics`.

That gap was caught by the focused unit test before completion and corrected in the same tranche. No further tranche-local shortfall remains open.

# Verification

- `make test-dashboard-red-team-truth-basis`
- `git diff --check`

# Operational Note

This slice intentionally stops at local status truth:

- compact run-history projection remains where it was,
- richer monitoring/benchmark/snapshot projection still belongs to `MON-OVERHAUL-1`,
- and Red Team is still not a substitute for the later Monitoring read-model projection work.
