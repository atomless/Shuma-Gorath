# Scrapling Challenge Interaction And Browser Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define the blocked later work needed to evaluate wider Scrapling challenge interaction and, only if proven, expand Shuma's Scrapling lane beyond today's request-native fetcher contract.

**Architecture:** Keep the current request-native Scrapling ownership truthful and unchanged while adding a separate blocked follow-on that evaluates defense-surface interaction coverage explicitly. First define a defense-surface representativeness matrix, then determine which missing interactions can stay request-native and which genuinely require Scrapling's browser or stealth runtime. Keep request-native public-network identity gaps, such as the current `geo_ip_policy` hole, on a separate source-diversification path rather than collapsing them into browser-runtime adoption. Keep browser-runtime adoption for `automated_browser` as a distinct later step rather than collapsing all widened Scrapling work into one vague bucket.

**Tech Stack:** Rust adversary-sim control plane, Python Scrapling worker, repo-owned Scrapling runtime bootstrap, canonical taxonomy and coverage receipts, Makefile verification, official Scrapling browser or stealth fetcher documentation.

---

## Guardrails

1. Do not retroactively claim that the current Scrapling worker already solves Shuma `not_a_bot`, puzzle, PoW, or browser-style challenge flows.
2. Do not widen the active Scrapling truth contract beyond `indexing_bot`, `ai_scraper_bot`, and `http_agent` in this planning slice.
3. Keep category ownership and defense-surface coverage as distinct concepts.
4. Keep any later browser-runtime adoption explicitly separate from simple request-native challenge interaction expansion.
5. Preserve the repo-wide requirement that coverage claims be receipt-backed rather than inferred from library marketing alone.
6. Judge all later adversary-lane expansion by attacker-faithfulness: if Shuma expects the lane to represent malicious automation against a surface, the lane must use the tool the way a real attacker would use it for that surface within Shuma's scope and safety boundaries.
7. Maintain a standing upstream Scrapling capability watch. When Scrapling adds attacker-relevant capability, refresh the research, plan, and backlog chain before claiming Shuma has or does not need that capability.

## Ongoing Hygiene Rule

Shuma should continuously track upstream Scrapling releases and official docs.

When a meaningful upstream delta appears:

1. compare it against Shuma's current runtime and worker integration,
2. judge it against the attacker-faithfulness rule,
3. decide whether it belongs in request-native expansion, `SIM-SCR-CHALLENGE-1`, or `SIM-SCR-BROWSER-1`,
4. and update the research, plan, and backlog docs explicitly rather than relying on tribal memory.

## Task 1: `SIM-SCR-CHALLENGE-1A`

**Files:**
- Modify: `docs/plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**
1. Add a blocked follow-on for Scrapling defense-surface challenge interaction after `SIM-SCR-FIT-1`, `SIM-SCR-FIT-2`, and `SIM-SCR-COVER-2`.
2. State explicitly that the current request-native plan does not yet prove Shuma `not_a_bot`, puzzle, or PoW interaction coverage.
3. Place the new follow-on before or alongside `SIM-SCR-BROWSER-1`, with clear sequencing language that challenge interaction evaluation comes before broad browser-runtime adoption.

**Acceptance criteria:**
1. The backlog clearly distinguishes current request-native ownership from later challenge-interaction evaluation.
2. `SIM-SCR-BROWSER-1` no longer carries the whole widened Scrapling burden by implication.

## Task 2: `SIM-SCR-CHALLENGE-1B`

**Files:**
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`
- Modify: `docs/research/2026-03-24-scrapling-challenge-surface-and-defense-coverage-review.md`
- Future implementation touchpoints to name explicitly in the plan:
  - `src/admin/adversary_sim_worker_plan.rs`
  - `src/admin/adversary_sim_lane_runtime.rs`
  - `scripts/supervisor/scrapling_worker.py`
  - `src/observability/non_human_coverage.rs`
  - `src/observability/non_human_lane_fulfillment.rs`

**Work:**
1. Define a canonical defense-surface matrix for Scrapling, separate from taxonomy ownership.
2. Include at minimum:
   - challenge routing interaction,
   - `not_a_bot` submit or fail paths,
   - puzzle submit or escalation paths,
   - PoW verify abuse,
   - rate pressure,
   - geo or IP policy interaction.
3. For each surface, classify whether the expected interaction should be:
   - request-native,
   - browser or stealth,
   - or intentionally out of Scrapling scope.
4. The first execution-ready follow-on should freeze the request-native owned-surface subset as `SIM-SCR-CHALLENGE-2A` before any wider runtime behavior is changed.

**Acceptance criteria:**
1. Shuma has a named representativeness matrix for defense surfaces, not only categories.
2. The matrix can explain why a missing Scrapling interaction is a real gap, not just an unstated expectation.
3. The first request-native owned-surface subset is frozen separately from later browser or stealth decisions.

## Task 3: `SIM-SCR-CHALLENGE-1C`

**Files:**
- Modify: `docs/plans/2026-03-24-scrapling-challenge-interaction-and-browser-expansion-plan.md`
- Future implementation touchpoints to name explicitly in the plan:
  - `scripts/bootstrap/scrapling_runtime.sh`
  - `scripts/supervisor/scrapling_worker.py`
  - `scripts/tests/test_scrapling_worker.py`
  - `scripts/tests/test_adversarial_lane_contract.py`
  - `scripts/tests/test_adversarial_coverage_contract.py`
  - `docs/testing.md`

**Work:**
1. Define the proof strategy for widened Scrapling interaction claims.
2. Require receipts for:
   - intended worker-plan or persona shape,
   - actual defense interaction observed,
   - resulting category or coverage effect,
   - and failure or fallback semantics when the interaction is unavailable.
3. State explicitly that upstream `StealthyFetcher` or `solve_cloudflare` support is insufficient evidence by itself.

**Acceptance criteria:**
1. Any later widened Scrapling claim must be proven at the runtime, API, and receipt surfaces Shuma actually uses.
2. The later implementation path already knows which focused Make targets and coverage fixtures need to exist.

## Task 4: `SIM-SCR-BROWSER-1`

**Files:**
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`

**Work:**
1. Refine the existing browser-runtime follow-on so it explicitly depends on the challenge-interaction evaluation outcome.
2. Keep its remit narrow:
   - truthful `automated_browser` ownership,
   - truthful browser-runtime deploy contract,
   - truthful cost and coverage receipts.
3. State explicitly that not all challenge-surface widening should automatically be solved by moving Scrapling into browser-runtime mode.

**Acceptance criteria:**
1. Browser-runtime adoption remains a separate, deliberate decision.
2. The roadmap can now distinguish:
   - request-native Scrapling,
   - widened challenge interaction,
   - and full browser-runtime Scrapling.

## Exit Criteria

This planning tranche is complete when:

1. the repo explicitly acknowledges that current Scrapling use is narrower than upstream Scrapling capability,
2. a new blocked `SIM-SCR-CHALLENGE-1` lane exists,
3. the current request-native contract remains truthful and unchanged,
4. and `SIM-SCR-BROWSER-1` is narrowed to the later browser-runtime question rather than absorbing every missing Scrapling capability by default.
