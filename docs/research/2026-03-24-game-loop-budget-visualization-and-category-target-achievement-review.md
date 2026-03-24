## Goal

Clarify how the Game Loop tab should express numeric objective budgets versus per-category taxonomy outcomes, so later UI work does not invent a fake category-budget model or keep relying on weak wording-only budget labels.

## Current machine-first truth

The current operator objective profile already contains real numeric ratio budgets for:

1. `likely_human_friction`
2. `suspicious_forwarded_requests`
3. `suspicious_forwarded_bytes`
4. `suspicious_forwarded_latency`

Those are persisted in `operator_objectives_v1.budgets`.

The taxonomy side is different. Canonical non-human categories are stored as posture targets in `operator_objectives_v1.category_postures`, not as separately configured numeric budget rows.

At the benchmark layer, however, the per-category posture family is already computed using target-vs-current ratio semantics:

1. one metric per category (`category_posture_alignment:<category_id>`)
2. `current`
3. `target`
4. `delta`
5. `inside_budget` / `near_limit` / `outside_budget`

So the category surface is already budget-like in evaluation shape, but it is not a first-class operator-configured budget model.

## Key distinction

The Game Loop tab should distinguish between:

1. **numeric budgets**
   - true operator-configured budget rows
   - best shown as budget consumption / target bars
2. **category target achievement**
   - per-category posture outcomes derived from classification receipts and the desired posture
   - best shown as achieved-vs-target ratio rows

This distinction matters because category posture semantics differ by posture:

1. `allowed` and `tolerated` are evaluated using forwarded ratio
2. `cost_reduced`, `restricted`, and `blocked` are evaluated using short-circuited ratio

Calling every category row "budget used" would therefore be semantically awkward even though the benchmark math is target-based.

## Product conclusion

The Game Loop surface should:

1. keep the high-level overall top line
2. demote wording like `inside_budget` to secondary metadata rather than the primary expression
3. show real numeric budgets with explicit current, target, near-limit threshold, and over-budget state
4. show taxonomy categories as `target achievement` or `posture conformance` rows rather than inventing per-category budget wording

Recommended operator-facing language:

1. `Budget usage` or `Budget consumption` for numeric objective budgets
2. `Category target achievement` for taxonomy categories

`Category target achievement` is preferred over `observed posture alignment`, which is technically correct but too vague for the operator-facing Game Loop tab.

## UX implication

The next Game Loop tranche should not stop at plain status words. It should expose:

1. human-friction and suspicious-origin budgets visually
2. target vs achieved ratio per non-human category
3. `inside budget` / `near limit` / `outside budget` only as supporting labels

## Why this is the right split

This keeps the tab faithful to the underlying machine-first contracts while making the operator view clearer:

1. no fake new policy model is introduced
2. no real configured budgets are hidden behind weak status text
3. category outcomes remain truthful to the actual benchmark math
4. the top line can stay high-level while the sections below become much more legible
