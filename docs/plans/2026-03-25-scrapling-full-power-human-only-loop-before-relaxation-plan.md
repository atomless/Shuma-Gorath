Date: 2026-03-25
Status: Proposed

Related context:

- [`../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md`](../research/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-review.md)
- [`../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
- [`../research/2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md`](../research/2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md)
- [`../research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md`](../research/2026-03-26-game-loop-scoring-and-diagnoser-audit.md)
- [`../research/2026-03-26-ideal-rsi-game-loop-scoring-review.md`](../research/2026-03-26-ideal-rsi-game-loop-scoring-review.md)
- [`../research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md`](../research/2026-03-26-game-loop-terrain-locality-and-breach-diagnosis-review.md)
- [`../research/2026-03-26-strict-loop-breach-signal-and-llm-resequence-review.md`](../research/2026-03-26-strict-loop-breach-signal-and-llm-resequence-review.md)
- [`2026-03-25-sim-scr-cap-1-capability-matrix-plan.md`](2026-03-25-sim-scr-cap-1-capability-matrix-plan.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`2026-03-25-scrapling-full-attacker-capability-principle-plan.md`](2026-03-25-scrapling-full-attacker-capability-principle-plan.md)
- [`2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Define the post-`STANCE-MODEL-1` execution order so Shuma does not relax stance or reopen LLM runtime work until:

1. Scrapling fully represents the non-agent or non-LLM adversary spectrum assigned to it through all attacker-relevant upstream Scrapling capability that belongs to that lane,
2. and the strict `human_only_private` loop has repeatedly improved through real config-change iteration.

# Core Decisions

1. `human_only_private` remains the first real operating stance after the stance-model redesign lands.
2. Scrapling must mature beyond the current request-native baseline before that strict stance is treated as operationally proven.
3. The proof bar is repeated config-change improvement across many completed cycles, not one canary cycle.
4. `humans_plus_verified_only` is a later gated second stance, not the next automatic move.
5. The remaining LLM attacker runtime should reopen after the first strict Scrapling-only proof, but before any relaxed verified-identity sweep.
6. The relaxed sweep should wait for a second strict-baseline proof that includes both Scrapling and LLM attacker pressure.
7. Before any of those tranches are described as complete, `VERIFY-GATE-1` must freeze the exact runtime, API, dashboard, and `make` proof required for closure.
8. During the strict sim-only phase, adversary-sim lanes are authoritative `100%` non-human traffic and should drive suspicious forwarded request, byte, and latency leakage toward zero or equivalent fail-closed suppression rather than the seeded mixed-site `10%` budgets.
9. Local loopback-hosted `/sim/public/*` pages are the first execution surface for strict-loop iteration when no real hosted site sits behind Shuma.
10. Human traversal against the discovered strict config is a later separate calibration ring, first local and then live, not something to infer from adversary-sim traffic.
11. Before strict-loop proof resumes, the judge and diagnoser must be upgraded so aggregate suspicious-origin suppression is no longer allowed to stand in for exploit defeat, low-confidence evidence cannot drive fine-grained config moves, and sudden new bypasses can break homeostasis immediately.
12. That scoring upgrade must treat the host site as terrain, preserve terrain-local breach loci, and prefer the smallest effective local repair rather than scattershot board-wide retuning.
13. The strict proof ring must not weaken `human_only_private` merely to manufacture a breach signal or recommendation; if a positive-control ring is needed, it must be explicit, separate, and not counted as proof that the strict stance itself is operationally proven.
14. Before further strict-loop claims, Shuma must explicitly answer whether the current Game Loop picture of Scrapling being fully blocked is truthful or a presentation artifact.
15. If full-power Scrapling plus truthful exploit-first scoring still cannot generate controller-grade breach signals under the unchanged strict stance, promoting `SIM-LLM-1C3` earlier becomes the correct next adversary-strength move.

# Execution Shape

## `SIM-SCR-FULL-1`: Full-power Scrapling for the non-agent adversary spectrum

This tranche should follow `STANCE-MODEL-1`.

It should treat the current request-native Scrapling lane as a truthful baseline, but not the maturity target.

### `SIM-SCR-FULL-1A`

Reopen and ratify the full attacker-relevant Scrapling capability matrix under the full-spectrum non-human adversary mandate.

Required contract:

1. evaluate every upstream Scrapling capability against whether it materially increases effective attack power against Shuma defenses or closes an uncovered non-human attacker gap,
2. keep such capability in scope for the active Scrapling lane unless there is an overt exclusion record,
3. treat browser or stealth classification alone as insufficient reason to assign a capability away from Scrapling,
4. allow exclusions only when the repo explicitly records that the capability does not increase adversary power, is already covered elsewhere with proof and no resulting gap, or would be unsafe or untruthful to claim without further runtime or receipt work,
5. and, if stronger Scrapling capability requires taxonomy or receipt expansion, define that expansion instead of weakening the lane by default.

### `SIM-SCR-FULL-1B`

Implement the remaining Scrapling capability required by that matrix.

Required contract:

1. Shuma should no longer rely on the current polite subset,
2. Scrapling should use every retained capability that materially strengthens attacks on Shuma-owned surfaces rather than stopping at the earlier request-native baseline,
3. Scrapling should be capable of touching and, where realistic, passing the defenses a real non-agent adversary should be able to pass,
4. and failing the defenses it should fail.

### `SIM-SCR-FULL-1C`

Add receipt-backed proof for the full-power Scrapling lane.

Required contract:

1. prove which defenses it touched,
2. which it passed where expected,
3. which it failed where expected,
4. and which non-human categories and defense surfaces it actually exercised.

### `SIM-SCR-FULL-1C3`

Audit the current operator picture of Scrapling performance before further strict-loop proof claims.

Required contract:

1. compare machine-first Scrapling receipts, Red Team truth, and Game Loop projection,
2. explicitly answer whether the current "fully blocked" reading is truthful or misleading,
3. and record any remaining projection gaps that could make Scrapling look weaker or stronger than the receipts support.

## `RSI-SCORE-2`: Exploit-first judge and diagnoser before strict-loop proof

This tranche should begin only after `SIM-SCR-FULL-1C1` and `SIM-SCR-FULL-1C2` have repaired the known evidence-truth gaps.

### `RSI-SCORE-2A`

Add terrain-local exploit-progress scoring so the judge can tell the difference between aggregate suppression and real attacker defeat, and can name where the adversary advanced.

### `RSI-SCORE-2B`

Add evidence-quality and diagnosis-confidence gates so low-confidence or poorly localized exploit evidence cannot drive fine-grained config tuning.

### `RSI-SCORE-2C`

Add urgency scoring and event-triggered homeostasis break so new bypasses interrupt a flat loop immediately.

### `RSI-SCORE-2D`

Separate judge, diagnoser, and move selector more sharply, require smallest-effective localized repair selection, and add an explicit config-ring exhaustion verdict for code-evolution referral.

### `RSI-SCORE-2E`

Project the richer judge truth in `Game Loop` so operators can distinguish guardrail pressure, exploit progress, evidence quality, urgency, named breach loci, and config-exhaustion or code-referral outcomes.

## `RSI-GAME-HO-1`: Strict `human_only_private` operational proof over repeated cycles

This tranche should begin only after `SIM-SCR-FULL-1` and `RSI-SCORE-2` are satisfied.

### `RSI-GAME-HO-1A`

Make `human_only_private` the actual active game-loop stance for the current machine-first loop.

Required contract:

1. verified non-human traffic remains denied under this stance,
2. benchmarks and Game Loop project the strict stance truthfully,
3. and the existing loop machinery runs against this corrected stance.
4. the strict view no longer presents the seeded mixed-site `10%` suspicious forwarded budgets as though they were the human-only target.

### `RSI-GAME-HO-1B`

Run repeated strict-baseline cycles until the loop is operationally proven.

Required contract:

1. Scrapling runs,
2. recommendations are generated,
3. bounded config changes are applied,
4. later Scrapling runs occur against the changed config,
5. watch windows judge retain or rollback truthfully,
6. and this repeats enough times to show non-noisy improvement toward the strict target rather than a single lucky cycle.
7. for this sim-only proof ring, the suspicious forwarded request, byte, and latency targets are zero or equivalent fail-closed suppression targets because the input traffic is known non-human adversary traffic.
8. the retained recommendations are terrain-localized and causally justified rather than broad multi-family config sweeps without a named breach locus.
9. the tranche does not rely on loosening `human_only_private` or any weaker policy baseline merely to create breach-positive evidence.

### `RSI-GAME-HO-1C`

Define the unlock condition for leaving the strict stance.

Required contract:

1. multiple completed cycles under `human_only_private`,
2. actual retained config changes,
3. measured improvement toward the strict target,
4. and clear evidence that the loop is not merely exercising rollback or recommendation plumbing.

## Human traversal calibration after strict-baseline discovery

This is a required follow-on proof ring after Shuma finds a strict config that suppresses adversary-sim traffic.

It is deliberately separate from the sim-only exclusion gate above.

Required contract:

1. real human-driven traversals run against the discovered strict config,
2. likely-human friction is measured from live human telemetry rather than adversary-sim inference,
3. local `/sim/public/*` remains the first fast calibration surface when there is no real hosted site behind Shuma,
4. and Linode or another shared-host target becomes the later realism and public-network verification surface.

## `SIM-LLM-1C3`: Remaining LLM attacker runtime proof closure

After `RSI-GAME-HO-1`, the default next meaningful runtime slice is the final LLM attacker proof closure.

Required contract:

1. runtime receipts are projected truthfully,
2. recent-run visibility is complete,
3. and the later LLM attacker becomes a real black-box runtime participant in the loop.
4. If `SIM-SCR-FULL-1C3` plus `RSI-SCORE-2` prove that full-power Scrapling still cannot generate controller-grade breach signals under the unchanged strict stance, promoting this tranche earlier is the correct evidence-driven exception to the default order.

## `RSI-GAME-HO-2`: Strict `human_only_private` proof with both Scrapling and LLM pressure

This later tranche should begin only after `SIM-LLM-1C3` is satisfied.

Required contract:

1. both Scrapling and the later LLM attacker now contribute loop pressure under `human_only_private`,
2. recommendations become bounded config changes,
3. later mixed-attacker runs occur against those changed configs,
4. watch windows retain or roll back truthfully,
5. and repeated retained changes show measured positive movement toward the strict target.

## `RSI-GAME-HV-1`: Later `humans_plus_verified_only` sweep

This is explicitly a later tranche.

It should remain blocked until `RSI-GAME-HO-2` is satisfied.

When it opens, it should:

1. compare against the proven strict baseline,
2. measure verified-identity handling against real prior strict-baseline data,
3. and not rely on hypothetical benefits.

# Backlog Integration

1. Keep `VERIFY-GATE-1` as the immediate next process prerequisite.
2. After `VERIFY-GATE-1`, keep `STANCE-MODEL-1` as the next design and implementation prerequisite.
3. After `STANCE-MODEL-1`, make `SIM-SCR-FULL-1` the next mainline instead of any LLM runtime slice.
4. Keep `SIM-SCR-FULL-1C3` as the explicit truth-audit gate before further strict-loop proof claims.
5. After `SIM-SCR-FULL-1`, make `RSI-SCORE-2` the next mainline so the judge and diagnoser are fit for exploit-first strict-loop proof.
6. After `RSI-SCORE-2`, make `RSI-GAME-HO-1` the next mainline by default.
7. Exception: if the `SIM-SCR-FULL-1C3` truth audit and `RSI-SCORE-2` together prove that full-power Scrapling still cannot generate controller-grade breach signals under the unchanged strict stance, promote `SIM-LLM-1C3` before further `RSI-GAME-HO-1` proof claims rather than weakening the strict baseline.
8. After `SIM-LLM-1C3`, add `RSI-GAME-HO-2` as the mixed Scrapling-plus-LLM strict-baseline proof.
9. Block `RSI-GAME-HV-1` until `RSI-GAME-HO-2` proves real repeated improvement.
10. Before claiming the strict baseline is human-safe or ready for broader operator use, add the separate human traversal calibration ring over the discovered strict config.

# Definition Of Done

This planning tranche is satisfied when:

1. the repo explicitly says the next post-stance-model work is full-power Scrapling, not LLM runtime,
2. the repo explicitly inserts exploit-first scoring and diagnoser work between full-power Scrapling evidence truth and strict-loop proof,
3. the repo explicitly forbids weakening `human_only_private` as a way to manufacture strict-loop proof,
4. the later LLM attacker runtime is sequenced before any relaxed verified-identity sweep, with an explicit evidence-driven promotion rule if full-power Scrapling still cannot generate breach signals,
5. the later `humans_plus_verified_only` sweep is blocked on the combined Scrapling-plus-LLM strict-baseline proof,
6. and relaxed stance work no longer doubles as loop verification.
