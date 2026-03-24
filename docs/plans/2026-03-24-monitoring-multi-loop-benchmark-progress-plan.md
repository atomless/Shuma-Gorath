Date: 2026-03-24
Status: Proposed refinement

Related context:

- [`../research/2026-03-24-monitoring-multi-loop-benchmark-progress-review.md`](../research/2026-03-24-monitoring-multi-loop-benchmark-progress-review.md)
- [`2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md`](2026-03-23-monitoring-loop-accountability-and-diagnostics-focus-plan.md)
- [`2026-03-23-dashboard-operator-surfacing-sequencing-plan.md`](2026-03-23-dashboard-operator-surfacing-sequencing-plan.md)

# Objective

Make it explicit in Monitoring planning and backlog language that Shuma should show bounded progress over recent loops against benchmark families, not only the latest loop outcome.

# Required Planning Changes

## `MON-OVERHAUL-1A`

Freeze the information architecture so Monitoring clearly has both:

1. a `Current Status` layer,
2. and a `Recent Loop Progress` layer.

`1A` should establish that the top of Monitoring is a status board, while the next layer tells the recent multi-loop accountability story.

## `MON-OVERHAUL-1B`

Project bounded recent multi-loop progress explicitly.

At minimum, the plan should require:

1. current vs prior-window benchmark movement,
2. bounded recent history over the last meaningful handful of completed loops,
3. benchmark-family progress rather than one synthetic score,
4. and recent controller action history tied to those loop outcomes.

## `MON-OVERHAUL-1C`

When category breakdown lands, extend the same principle carefully:

1. category surfaces may show recent trend where the machine-first contract supports it,
2. but Monitoring must remain bounded and readable rather than turning into a raw historical browser.

# Definition Of Done

This refinement is complete when:

1. Monitoring planning and TODO language explicitly require bounded multi-loop benchmark progress,
2. current status and recent loop progress are both first-class Monitoring concepts,
3. and the docs make clear that Monitoring should show benchmark-family movement and controller history, not only the latest loop result.
