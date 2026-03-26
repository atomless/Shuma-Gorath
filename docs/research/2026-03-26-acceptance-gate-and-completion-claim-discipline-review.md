Date: 2026-03-26
Status: Completed

# Acceptance-Gate And Completion-Claim Discipline Review

## Question

How should Shuma prevent planning completions, baseline capability, and partial proof slices from being described as though larger feature tranches are already complete, especially for `STANCE-MODEL-1`, `SIM-SCR-FULL-1`, and `RSI-GAME-HO-1`?

## Findings

### 1. The backlog truth and the conversational completion story drifted apart

The active backlog still correctly shows the key tranches as open:

- `STANCE-MODEL-1` in [`../../todos/todo.md`](../../todos/todo.md)
- `SIM-SCR-FULL-1` in [`../../todos/todo.md`](../../todos/todo.md)
- `RSI-GAME-HO-1` in [`../../todos/todo.md`](../../todos/todo.md)

So the repository source of truth did not actually mark those tranches complete.

The failure was process discipline around how progress was described. Planning completions, baseline proof slices, and UI cleanup were allowed to sound too close to feature closure.

### 2. The current Scrapling implementation is still baseline capability, not full-power Scrapling

The live repo still distinguishes the current request-native Scrapling baseline from the later fuller capability target:

- [`2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md`](2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md) explicitly assigns `DynamicFetcher`, `StealthyFetcher`, and Cloudflare-style solving away from the currently landed request-native lane.
- [`../plans/2026-03-25-sim-scr-rn-1-request-native-fidelity-plan.md`](../plans/2026-03-25-sim-scr-rn-1-request-native-fidelity-plan.md) explicitly says not to widen that slice into browser-runtime capability.

So any phrasing that implies full-power Scrapling is already operational is inaccurate.

### 3. The strict `human_only_private` loop proof is not yet implemented just because a first loop exists

The intended bar for `RSI-GAME-HO-1` is much higher than “the loop ran once”:

- repeated Scrapling-driven cycles,
- bounded config recommendations,
- config changes applied,
- later runs against the changed config,
- retain or rollback truthfully judged,
- and measured improvement toward the strict target.

That bar is explicit in [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md).

So the first-working-loop slices remain baseline capability only.

### 4. The current Game Loop dashboard is not a valid closure oracle for those later tranches

The current `Game Loop` tab still renders legacy `non_human_category_posture` benchmark math:

- [`../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- [`../../src/observability/benchmark_non_human_categories.rs`](../../src/observability/benchmark_non_human_categories.rs)

That surface can expose useful pressure and mismatch, but until `STANCE-MODEL-1` lands it does not prove the strict `human_only_private` methodology is implemented. The UI therefore must not be used as an implicit completion oracle for those larger tranches.

### 5. Completion-history wording can still overstate future tranches if it is not carefully framed

The completion archive already separates planning completions from open TODO items, but some wording still says future tranches “prove” things rather than saying they are the tranches that should or must prove them once implemented.

That wording is enough to create false confidence and must be treated as a process defect.

## Decision

Shuma should add an immediate acceptance-discipline tranche before further mainline closure claims:

1. freeze explicit acceptance gates for the active mainline tranches,
2. identify the exact executable and operator-visible proof required for each gate,
3. prevent planning completions from being written in language that reads like feature closure,
4. and treat any mismatch between TODO state, code reality, dashboard reality, and completion claims as release-blocking.

## Practical Consequence

The immediate next process step is not another feature completion claim. It is an explicit `VERIFY-GATE-1` tranche that:

1. defines what counts as done for `STANCE-MODEL-1`, `SIM-SCR-FULL-1`, `RSI-GAME-HO-1`, and later `RSI-GAME-HO-2`,
2. makes those acceptance gates visible in the active backlog,
3. and corrects existing audit-trail wording that currently overstates what those future tranches are meant to prove.
