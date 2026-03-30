# SIM-REALISM-2 Wild Traffic Gap Closure Plan

**Goal:** Close the remaining field-grounded realism gaps after `SIM-REALISM-1C` and `SIM-REALISM-1D` so Shuma's adversary lanes no longer stop at local burst-shape realism and can instead represent the pressure, identity, transport, and recurrence patterns actually seen in current hostile traffic.

**Architecture:** Reuse the existing lane realism contract, lane planner, supervisor, and worker seams. Extend that contract with pressure-envelope, identity-envelope, transport-envelope, browser-secondary-traffic, and recurrence concepts. Preserve the existing public-discovery-only Scrapling boundary and the existing black-box Shuma-blind Agentic boundary. Do not widen host knowledge to achieve realism.

**Tech Stack:** Rust adversary-sim planner and state, Python Scrapling worker, Python LLM runtime worker and adversarial container worker, existing monitoring and observer read models, Makefile verification, backlog and sequencing docs.

**Related research:** [`../research/2026-03-30-adversary-lane-wild-traffic-gap-review.md`](../research/2026-03-30-adversary-lane-wild-traffic-gap-review.md)

---

## Guardrails

1. Do not fake proxy-pool realism when no pool is configured. Receipts and docs must say realism is degraded rather than inventing network identity churn.
2. Do not achieve higher pressure by widening host knowledge, introducing internal route catalogs, or leaking Shuma-specific hints.
3. Do not let simulator-only identity, proxy, or category metadata leak into defence truth or category truth.
4. Do not turn realism work into uncontrolled aggression. Every stronger behavior must remain bounded by an explicit contract and receipt-backed proof.

## Task 1: Refresh Backlog And Sequencing Truth

**Files:**
- Modify: `todos/todo.md`
- Modify: `todos/blocked-todo.md`
- Modify: `docs/plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`
- Modify: `docs/plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`

**Work:**
1. Make `SIM-REALISM-2A..2E` explicit as the field-grounded realism follow-on after `SIM-REALISM-1D`.
2. Stop implying that `SIM-REALISM-1D` alone reopens representative mixed-attacker or tuning-quality claims.
3. Record that `SIM-REALISM-1B` landed local pacing-shape realism, not the full wild-attacker realism contract.

**Acceptance criteria:**
1. The active backlog names the second realism chain explicitly.
2. The sequencing docs place `SIM-REALISM-2` ahead of any claim that adversary lanes are representative enough for mixed-attacker proof.

## Task 2: `SIM-REALISM-2A` Unclip Pressure Envelopes And Add Per-Persona Concurrency

**Files:**
- Later code targets: `src/admin/adversary_sim.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `src/admin/adversary_sim_realism_profile.rs`, `scripts/supervisor/scrapling_worker.py`, `src/admin/adversary_sim_llm_lane.rs`, `scripts/tests/adversarial_container/worker.py`
- Later proof targets: focused realism tests, `Makefile`

**Work:**
1. Replace the current one-size global request/time clipping behavior with explicit per-persona and per-mode pressure envelopes.
2. Allow request-native personas to exceed the current `8 requests / 2 seconds` ceiling when their realism contract calls for it, while preserving explicit bounded safety limits.
3. Add bounded concurrency semantics where they materially affect realism instead of forcing every persona through a single-file sequential shape.

**Acceptance criteria:**
1. Request-native personas no longer collapse to the same clipped pressure envelope.
2. Receipts prove actual peak per-burst counts, effective request cadence, and any concurrency grouping used.
3. Pressure remains explicitly bounded by contract and does not widen host knowledge.

**Proof:**
1. Add and pass `make test-adversary-sim-pressure-envelope-realism`.
2. Keep existing Scrapling and Agentic realism targets green.

## Task 3: `SIM-REALISM-2B` Add Proxy-Pool And Identity-Envelope Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_worker_plan.rs`, `src/admin/adversary_sim_lane_runtime.rs`, `scripts/supervisor/scrapling_worker.py`, `src/admin/adversary_sim_llm_lane.rs`, `scripts/supervisor/llm_runtime_worker.py`
- Later proof targets: contract tests, focused runtime tests, `Makefile`

**Work:**
1. Introduce a bounded identity-envelope contract that can represent:
   - residential versus mobile versus datacenter identity class,
   - geo affinity,
   - session stickiness,
   - reuse and churn cadence,
   - and truthful degraded mode when no proxy pool exists.
2. Support pool-backed identity rotation for both Scrapling and Agentic request traffic without claiming network diversity when only local process diversity exists.
3. Project identity-envelope truth into realism receipts without leaking those labels into Shuma defences.

**Acceptance criteria:**
1. Lanes can truthfully emit realistic identity rotation when backing infrastructure exists.
2. When that infrastructure does not exist, receipts explicitly state degraded identity realism rather than presenting synthetic session churn as full network realism.
3. Geo, locale, and session continuity can be tied to the chosen identity envelope.

**Proof:**
1. Add and pass `make test-adversarial-identity-envelope-contract`.
2. Keep recent-run projection and machine-contract tests green if receipt or observer shapes change.

## Task 4: `SIM-REALISM-2C` Add Coherent Header, Locale, And Transport Envelopes

**Files:**
- Later code targets: `scripts/supervisor/scrapling_worker.py`, `scripts/tests/adversarial_container/worker.py`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `src/admin/adversary_sim_realism_profile.rs`
- Later proof targets: focused runtime tests, `Makefile`

**Work:**
1. Replace the current mostly static locale and header posture with coherent persona and identity envelopes covering:
   - user-agent family,
   - Accept-Language,
   - Accept and related resource headers,
   - browser locale,
   - and transport or fingerprint posture where the current libraries expose it.
2. Keep those envelopes coherent with geo affinity and persona rather than randomizing headers independently.
3. Preserve the current same-origin, public-discovery-only, Shuma-blind boundaries.

**Acceptance criteria:**
1. Traffic no longer pins every request-native run to one default `en-GB` Chrome-like posture.
2. Header and locale variation is coherent enough to model the field patterns described in the research note.
3. Receipts or focused proofs can show which envelope was actually applied without turning it into defence truth.

**Proof:**
1. Add and pass `make test-adversary-sim-header-transport-realism`.
2. Keep relevant lane-contract and worker tests green.

## Task 5: `SIM-REALISM-2D` Capture Browser Secondary Traffic And Background Request Truth

**Files:**
- Later code targets: `scripts/supervisor/scrapling_worker.py`, browser-mode execution under `scripts/tests/adversarial_container/`, read-model and recent-run projection files if needed
- Later proof targets: focused runtime tests, machine-contract tests, `Makefile`

**Work:**
1. Capture and receipt meaningful browser secondary traffic:
   - subresource fetches,
   - background XHR or fetch activity,
   - and top-level versus background separation.
2. Use the already-upgraded upstream Scrapling capability where applicable instead of inventing a parallel capture surface.
3. Preserve compact observer truth so browser lanes are not understated as mere top-level page visits.

**Acceptance criteria:**
1. Browser personas no longer hide their emitted background traffic behind top-level-only receipts.
2. Recent-run or machine-facing truth can distinguish top-level actions from secondary background requests.
3. Proof remains compact and does not bloat the hot observer payload with raw browser traces.

**Proof:**
1. Add and pass `make test-adversary-sim-browser-secondary-traffic-realism`.
2. Keep relevant recent-run or machine-contract tests green if read models change.

## Task 6: `SIM-REALISM-2E` Add Long-Horizon Dormancy, Recurrence, And Re-Entry Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_state.rs`, `src/admin/adversary_sim_lane_runtime.rs`, relevant worker state or receipt files, later tuning or observer docs if timing truth changes
- Later proof targets: focused runtime tests, `Makefile`

**Work:**
1. Add a bounded recurrence model for adversary lanes so they can express:
   - re-entry after absence,
   - repeated narrow sessions over longer windows,
   - and longer-horizon persistence patterns that current short runs cannot represent alone.
2. Keep this separate from ban-policy tuning; the aim is attacker realism, not sanction logic.
3. Make any longer-horizon profile visible in machine receipts so later tuning and evaluation surfaces do not infer it from coincidence.

**Acceptance criteria:**
1. The realism contract can express more than one isolated short burst.
2. Longer-window attacker behavior is modeled explicitly rather than assumed from repeated beat execution.
3. The resulting state remains bounded and testable.

**Proof:**
1. Add and pass `make test-adversary-sim-recurrence-realism`.
2. Keep adversary-sim state and dispatch tests green.

## Recommended Execution Order

1. finish `SIM-REALISM-1C`
2. finish `SIM-REALISM-1D`
3. `SIM-REALISM-2A`
4. `SIM-REALISM-2B`
5. `SIM-REALISM-2C`
6. `SIM-REALISM-2D`
7. `SIM-REALISM-2E`
8. only then reopen representative mixed-attacker or tuning-quality claims
