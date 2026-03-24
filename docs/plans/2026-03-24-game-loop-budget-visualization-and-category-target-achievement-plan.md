## Goal

Refine the next Game Loop tranche so it expresses numeric objective budgets and taxonomy-category outcomes truthfully and legibly, without collapsing both into the same vague `inside_budget` wording.

## Decisions

1. Keep the existing high-level overall top line in Game Loop.
2. Treat numeric objective budgets and category posture outcomes as two distinct surface types.
3. Use stronger visual target-vs-current expression for both, but with different semantics:
   - numeric budgets -> budget usage / consumption bars
   - category posture -> category target-achievement rows
4. Keep `inside_budget`, `near_limit`, and `outside_budget` as secondary status labels rather than the primary presentation.

## Why

The current objective profile contains real numeric budgets, while category posture is stored as desired posture rows. The benchmark layer already evaluates category posture as target-vs-current ratios, but that still does not make category posture a first-class configured budget surface.

So the UI should reuse the target-vs-current clarity without lying about the underlying model.

## Scope impact on active work

### `MON-OVERHAUL-1C`

Refine the tranche scope so Game Loop adds:

1. a numeric-budget section or sub-surface for:
   - likely human friction
   - suspicious forwarded requests
   - suspicious forwarded bytes
   - suspicious forwarded latency
2. category rows expressed as target achievement rather than a generic "alignment" label
3. supporting status text for budget state, not wording-only primary display

### Out of scope for this slice

1. inventing new operator-editable per-category budgets
2. changing the persisted objective model
3. collapsing category posture and numeric budgets into one generic widget type if that makes the semantics less clear

## Recommended Game Loop shape

1. **Overall top line**
   - keep the current high-level rollup
2. **Budget usage**
   - compact target-vs-current bars for the true numeric budgets
3. **Category target achievement**
   - one row per taxonomy category
   - show desired posture
   - show achieved ratio
   - show support status / evidence readiness where relevant
4. **Trust and blockers**
   - keep as the bounded explanation layer

## Backlog implication

Update `MON-OVERHAUL-1C` so the next Game Loop work explicitly distinguishes:

1. numeric budget visualization
2. category target-achievement visualization
3. secondary status labels vs primary visual signal

## Acceptance direction for the later tranche

The later Game Loop implementation should satisfy all of these:

1. the overall top line remains visible and high-level
2. numeric objective budgets are readable without relying on text-only status wording
3. category rows do not pretend to be separately configured budgets
4. the operator can tell, at a glance, both:
   - how much real numeric budget is being used
   - how close each taxonomy category is to its desired posture outcome
