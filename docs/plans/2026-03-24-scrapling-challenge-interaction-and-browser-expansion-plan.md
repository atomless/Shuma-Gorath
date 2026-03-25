# Scrapling Challenge Interaction And Browser Expansion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Define the work needed to make Shuma adopt attacker-relevant upstream Scrapling capability by default for Scrapling-owned surfaces, while keeping broader category-ownership questions separate.

**Architecture:** Keep the current request-native Scrapling ownership truthful as the baseline, but stop treating fuller attacker-relevant capability as a merely reluctant contingency. The earlier `SIM-SCR-CAP-1` matrix is now only a request-native baseline; `SIM-SCR-FULL-1A` supersedes it for the full non-agent remit. Determine which additional owned-surface interactions stay request-native and which genuinely require Scrapling's dynamic or stealth runtime. Keep browser-runtime adoption for `automated_browser` as a distinct later step rather than collapsing all widened Scrapling work into one vague bucket.

**Tech Stack:** Rust adversary-sim control plane, Python Scrapling worker, repo-owned Scrapling runtime bootstrap, canonical taxonomy and coverage receipts, Makefile verification, official Scrapling browser or stealth fetcher documentation.

---

## Guardrails

1. Do not retroactively claim that the current Scrapling worker already solves Shuma `not_a_bot`, puzzle, PoW, or browser-style challenge flows.
2. Do not widen category ownership beyond `indexing_bot`, `ai_scraper_bot`, and `http_agent` unless that ownership is explicitly re-ratified.
3. Keep category ownership and defense-surface coverage as distinct concepts.
4. Keep any later browser-runtime adoption explicitly separate from simple request-native challenge interaction expansion.
5. Preserve the repo-wide requirement that coverage claims be receipt-backed rather than inferred from library marketing alone.
6. Judge all later adversary-lane expansion by attacker-faithfulness: if Shuma expects the lane to represent malicious automation against a surface, the lane must use the tool the way a real attacker would use it for that surface within Shuma's scope and safety boundaries.
7. Treat attacker-relevant upstream Scrapling capability for Scrapling-owned surfaces as a default-adopt expectation; any omission must be explicit and justified.
8. Maintain a standing upstream Scrapling capability watch. When Scrapling adds attacker-relevant capability, refresh the research, plan, and backlog chain before claiming Shuma has or does not need that capability.
9. Do not preserve stale wording that assigns dynamic or stealth Scrapling away by default merely because the earlier request-native baseline was already receipt-backed.

## Ongoing Hygiene Rule

Shuma should continuously track upstream Scrapling releases and official docs.

When a meaningful upstream delta appears:

1. compare it against Shuma's current runtime and worker integration,
2. judge it against the attacker-faithfulness rule,
3. decide whether it belongs in request-native expansion, `SIM-SCR-CHALLENGE-1`, or `SIM-SCR-BROWSER-1`,
4. and update the research, plan, and backlog docs explicitly rather than relying on tribal memory.

## Task 1: `SIM-SCR-CAP-1A`

**Files:**
- Modify: `docs/plans/2026-03-23-scrapling-request-native-category-fulfillment-implementation-plan.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`
- Modify: `todos/blocked-todo.md`

**Work:**
1. Add an active follow-on for the upstream attacker-capability matrix and explicit omission ledger over Scrapling-owned surfaces.
2. State explicitly that the current request-native plan is a truthful baseline, not the end state of Scrapling maturity.
3. Place the capability-matrix tranche before broader browser-runtime adoption, with clear sequencing language that explicit adoption or exclusion comes before passive deferral.

**Acceptance criteria:**
1. The backlog clearly distinguishes current request-native ownership from active attacker-capability maintenance.
2. `SIM-SCR-BROWSER-1` no longer carries the whole widened Scrapling burden by implication.

## Task 2: `SIM-SCR-CAP-1B`

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
1. Define a canonical attacker-capability matrix for Scrapling, separate from taxonomy ownership.
2. Include at minimum:
   - challenge routing interaction,
   - `not_a_bot` submit or fail paths,
   - puzzle submit or escalation paths,
   - PoW verify abuse,
   - rate pressure,
   - geo or IP policy interaction.
3. For each attacker-relevant upstream capability, classify whether it should be:
   - adopted in the current Scrapling-owned surface set,
   - assigned to another lane,
   - or explicitly excluded with a recorded reason.
4. For each adopted surface interaction, classify whether the expected interaction should be:
   - request-native,
   - browser or stealth,
   - or intentionally out of Scrapling scope.

**Acceptance criteria:**
1. Shuma has a named attacker-capability matrix for Scrapling-owned surfaces, not only categories.
2. The matrix can explain why a missing Scrapling interaction is an explicit exclusion rather than an unstated omission.

## Task 3: `SIM-SCR-CAP-1C`

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
1. Define the proof strategy for widened Scrapling interaction claims and explicit omissions.
2. Require receipts for:
   - intended worker-plan or persona shape,
   - actual defense interaction observed,
   - resulting category or coverage effect,
   - and failure or fallback semantics when the interaction is unavailable.
3. State explicitly that upstream `StealthyFetcher` or `solve_cloudflare` support is insufficient evidence by itself.
4. Require an explicit omission ledger entry whenever an attacker-relevant upstream capability is not adopted.

**Acceptance criteria:**
1. Any later widened Scrapling claim must be proven at the runtime, API, and receipt surfaces Shuma actually uses.
2. Any omitted attacker-relevant capability must leave an explicit recorded reason and a reconsideration trigger.
3. The later implementation path already knows which focused Make targets and coverage fixtures need to exist.

## Task 4: `SIM-SCR-BROWSER-1`

**Files:**
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`
- Modify: `docs/plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`

**Work:**
1. Refine the existing browser-runtime follow-on so it explicitly depends on the capability-matrix and omission-ledger outcome.
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
2. an active capability-matrix and omission-ledger lane exists,
3. the current request-native contract remains truthful and unchanged as a baseline,
4. and `SIM-SCR-BROWSER-1` is narrowed to the later browser-runtime question rather than absorbing every missing Scrapling capability by default.
