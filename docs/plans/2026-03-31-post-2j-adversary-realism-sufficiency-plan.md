# Post-2J Adversary Realism Sufficiency Plan

**Goal:** Close the realism gaps that would still remain after `SIM-REALISM-2F..2J` so Shuma can eventually claim representative attacker pressure for Game Loop and Tuning work without overstating what the adversary sim currently models.

**Architecture:** Reuse the existing planner, realism contract, worker, and observer seams, but extend them with mixed-lane parallel execution, stronger agentic action capability, long-window recurrence, deeper transport realism, and explicit representativeness gating. Preserve the black-box Shuma-blind and public-discovery-only boundaries.

**Tech Stack:** Rust adversary-sim planner and state, Python Scrapling worker, Python LLM runtime worker and adversarial container worker, dashboard and admin control surfaces, existing observer and monitoring read models, Makefile verification, backlog and sequencing docs.

---

## Guardrails

1. Do not widen host knowledge to make the lanes look more capable.
2. Do not let the new parallel mode become simulator-only choreography or a convenience-only harness path.
3. Do not grant privileged ingress headers or trust shortcuts to attacker-plane workers.
4. Do not overstate representativeness when required infrastructure is absent.
5. Do not add lane-selector wording that implies realism or maturity beyond what the receipts and topology can actually prove.

## Task 1: `SIM-REALISM-3A` Add Overlapping Multi-Lane And Multi-Identity Concurrency Realism

**Files:**
- Later code targets: `src/admin/adversary_sim.rs`, `src/admin/adversary_sim_state.rs`, `src/admin/adversary_sim_lane_runtime.rs`, relevant worker-plan/result contracts, dashboard lane-selector surfaces, Red Team docs, and observer read models
- Later proof targets: focused runtime and dashboard tests, `Makefile`

**Work:**
1. Add a bounded mixed-lane parallel execution mode that allows Scrapling and Agentic Traffic to run in the same episode window with truthful overlap instead of strict turn-taking.
2. Add a control-surface option for that mode in the Red Team lane selector so contributors can explicitly run the mixed-attacker path.
3. Preserve observer truth so recent-run, operator, and Game Loop surfaces can distinguish:
   - Scrapling-only,
   - Agentic-only,
   - and parallel mixed-lane pressure.
4. Keep concurrency bounded and receipt-backed so the system models overlapping attackers rather than uncontrolled worker fan-out.

**Acceptance criteria:**
1. Shuma can truthfully emit overlapping Scrapling and Agentic pressure rather than only serializing one lane at a time.
2. The Red Team lane dropdown exposes an explicit parallel mixed-lane option without implying more maturity than the underlying runtime supports.
3. Receipts and observer surfaces preserve per-lane contribution and overlap truth for the mixed-lane mode.

**Proof:**
1. Add and pass `make test-adversary-sim-parallel-lane-realism`.
2. Add and pass `make test-dashboard-red-team-lane-selector-contract`.
3. Keep mixed-attacker operator and dashboard truth targets green.

## Task 2: `SIM-REALISM-3B` Expand Agentic Action Capability And Degraded Fallback Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_llm_lane.rs`, `scripts/tests/adversarial_runner/llm_fulfillment.py`, `scripts/tests/adversarial_container/worker.py`, `scripts/supervisor/llm_runtime_worker.py`, related contract files, and observer receipts
- Later proof targets: focused LLM runtime tests, black-box contract tests, `Makefile`

**Work:**
1. Expand the bounded agentic action surface beyond `GET` plus `navigate/click` so it can cover realistic public-host hostile behaviors such as:
   - query variation,
   - form submission,
   - pagination or result walking,
   - repeated narrow extraction over focused page sets,
   - and other bounded multi-step public actions that real agentic abuse commonly performs.
2. Raise the degraded fallback floor so provider failure no longer collapses to a trivially polite `GET /` plus hints pattern.
3. Keep the black-box boundary intact: no repo knowledge, no hidden routes, no Shuma-only hints.

**Acceptance criteria:**
1. The agentic lane can perform realistic bounded public-host hostile behaviors beyond simple `GET` retrieval.
2. Provider degradation still leaves the lane meaningfully adversarial rather than collapsing to one polite fallback pattern.
3. The expanded action surface remains same-origin, public-hint-only, and Shuma-blind.

**Proof:**
1. Add and pass `make test-adversary-sim-agentic-action-realism`.
2. Keep LLM black-box contract, runtime dispatch, and rendered observer truth tests green.

## Task 3: `SIM-REALISM-3C` Add True Long-Window Dormancy And Return Realism

**Files:**
- Later code targets: `src/admin/adversary_sim_state.rs`, `src/admin/adversary_sim_lane_runtime.rs`, realism-profile contracts, recurrence receipts, and relevant docs
- Later proof targets: focused recurrence tests, `Makefile`

**Work:**
1. Extend recurrence beyond the current within-run short-gap model to cover:
   - hours-to-days dormancy,
   - later re-entry,
   - and bounded campaign-style return behavior.
2. Preserve explicit receipt truth for the planned and observed dormant windows so later loop surfaces do not infer campaign behavior from coincidence.
3. Keep the state bounded and contributor-usable; do not turn local runs into uncontrolled long-lived background processes.

**Acceptance criteria:**
1. The realism contract can express materially longer return windows than the current few-second re-entry.
2. Observer and receipt surfaces can distinguish one short burst from a bounded longer campaign.
3. The implementation remains testable and bounded for local contributor use.

**Proof:**
1. Add and pass `make test-adversary-sim-long-window-recurrence-realism`.
2. Keep state and dispatch tests green.

## Task 4: `SIM-REALISM-3D` Deepen Transport And Network Fingerprint Realism

**Files:**
- Later code targets: realism-profile contracts, request emission layers, browser emission layers, transport-envelope receipts, related docs
- Later proof targets: focused worker tests, contract tests, `Makefile`

**Work:**
1. Extend transport realism beyond coarse client-family posture to the deepest level the current stack can truthfully support, including protocol and fingerprint-relevant behavior where exposed.
2. Fail closed when the underlying runtime cannot support a claimed transport shape; do not pretend to have browser-grade network posture when the stack cannot actually emit it.
3. Preserve observer-only receipt truth about what transport realism level was actually achieved.

**Acceptance criteria:**
1. Transport realism moves beyond coarse `curl_impersonate` or `urllib_direct` naming where the current stack can support deeper proof.
2. Any limits of the underlying stack are exposed explicitly as degraded realism rather than silently hidden.
3. Observer surfaces can distinguish shallow transport posture from richer transport realism.

**Proof:**
1. Add and pass `make test-adversary-sim-transport-fingerprint-realism`.
2. Keep existing header, transport, and machine-contract tests green.

## Task 5: `SIM-REALISM-3E` Add A Representativeness Infrastructure Gate

**Files:**
- Later code targets: readiness and diagnostics surfaces, operator/admin snapshot wording, deployment and operator docs, Red Team help text, and blocker docs
- Later proof targets: focused rendered or API truth tests, `Makefile`

**Work:**
1. Add an explicit representativeness readiness contract that answers whether the current environment has the required backing infrastructure for realism claims, including:
   - trusted ingress,
   - pool-backed identities where required,
   - and any other ratified prerequisites for representative lane claims.
2. Expose that truth in operator and contributor surfaces so the system can fail closed to:
   - representative,
   - partially representative,
   - or degraded realism.
3. Use that readiness contract to prevent Game Loop and Tuning from overstating attacker realism in environments that do not actually support it.

**Acceptance criteria:**
1. Shuma no longer relies on implicit assumptions to decide whether realism claims are valid.
2. Operator and contributor surfaces can tell when the lanes are degraded by missing infrastructure.
3. Later Game Loop and Tuning work can key off an explicit representativeness contract instead of vague narrative claims.

**Proof:**
1. Add and pass `make test-adversary-sim-representativeness-readiness`.
2. Keep relevant dashboard and machine-contract truth tests green if rendered wording changes.
