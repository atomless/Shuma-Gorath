Date: 2026-03-26
Status: Proposed

Related context:

- [`../research/2026-03-26-acceptance-gate-and-completion-claim-discipline-review.md`](../research/2026-03-26-acceptance-gate-and-completion-claim-discipline-review.md)
- [`../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md`](../research/2026-03-26-strict-human-only-loop-and-human-traversal-calibration-review.md)
- [`2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md)
- [`2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md`](2026-03-25-canonical-non-human-stance-and-verified-identity-override-plan.md)
- [`2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/completed-todo-history.md`](../../todos/completed-todo-history.md)

# Objective

Add hard acceptance-gate discipline for the active mainline so Shuma does not again describe planning completions, baseline capability, or partial proof as though the larger Scrapling and Game Loop tranches are already complete.

# Core Decisions

1. `VERIFY-GATE-1` becomes the immediate next prerequisite before `STANCE-MODEL-1`.
2. `SIM-SCR-FULL-1` must not be described as complete until the full ratified Scrapling capability matrix is either implemented or explicitly excluded and receipt-backed.
3. `RSI-GAME-HO-1` must not be described as complete until repeated retained config-change improvement under the strict stance is proven, not merely because one loop or one apply path exists.
4. Planning completions must be written as planning completions only; they must not use language that reads like feature closure.
5. Dashboard or benchmark surfaces must not be treated as acceptance proof for a larger tranche unless the plan explicitly names them as one of the required proof surfaces and the underlying runtime contract is already landed.

# Execution Shape

## `VERIFY-GATE-1A`: Freeze explicit acceptance gates for the active mainline

Define tranche-level done criteria in the active backlog for:

1. `STANCE-MODEL-1`
2. `SIM-SCR-FULL-1`
3. `RSI-GAME-HO-1`
4. later `RSI-GAME-HO-2`

Each gate should answer, explicitly:

1. what runtime or config behavior must exist,
2. what API or snapshot truth must exist,
3. what operator-visible dashboard or admin truth must exist,
4. which focused `make` targets must pass,
5. and what evidence is still insufficient even if some precursor slice is already landed.

### Frozen closure gate: `STANCE-MODEL-1`

`STANCE-MODEL-1` may move to completed history only when all of the following are true:

1. Runtime/config truth:
   - one canonical non-human stance authority exists,
   - the independent verified-identity top-level stance is no longer authoritative,
   - and `human_only_private` plus `humans_plus_verified_only` are machine-readable and enforced through the same resolved policy contract.
2. API/snapshot truth:
   - the resolved effective policy is materialized in admin or snapshot surfaces in a machine-readable form,
   - and verified-identity override lineage is explicit rather than implied.
3. Dashboard/admin truth:
   - `Game Loop`, `Verification`, and any relevant operator control surfaces project the resolved policy rather than the old split model.
4. Focused proof:
   - focused `make` paths exist and pass for runtime policy behavior, API/snapshot projection, and rendered dashboard truth.
5. Insufficient evidence:
   - docs-only redesign,
   - config naming without runtime enforcement,
   - or legacy Game Loop posture rows still driven by the old dual-stance semantics.

### Frozen closure gate: `SIM-SCR-FULL-1`

`SIM-SCR-FULL-1` may move to completed history only when all of the following are true:

1. Runtime truth:
   - the full ratified Scrapling capability matrix for the non-agent or non-LLM lane is implemented,
   - or any omitted capability is explicitly excluded by the active matrix rather than silently unimplemented.
2. Receipt/API truth:
   - recent-run, snapshot, or equivalent machine-first surfaces carry receipt-backed evidence of categories fulfilled and defense surfaces touched,
   - including which surfaces were passed where expected and failed where expected.
3. Operator-visible truth:
   - `Red Team` projects that full-power evidence faithfully,
   - and `Game Loop` does not imply more attacker maturity than the backend actually proves.
4. Focused proof:
   - focused `make` paths exist and pass for worker/runtime behavior, category fulfillment, receipt-backed surface coverage, and operator-visible evidence.
5. Insufficient evidence:
   - request-native baseline only,
   - upstream Scrapling capability claims without Shuma-local proof,
   - or coverage claims that do not distinguish pass/fail expectations.

### Frozen closure gate: `RSI-GAME-HO-1`

`RSI-GAME-HO-1` may move to completed history only when all of the following are true:

1. Runtime/config truth:
   - the loop actually runs against `human_only_private`,
   - verified non-human traffic remains denied under that stance,
   - bounded config recommendations are generated,
   - config changes are applied,
   - later Scrapling runs occur against the changed config,
   - and retain or rollback judgments occur over repeated cycles rather than once.
2. API/snapshot truth:
   - recent changes, oversight history, and other machine-first surfaces show repeated cycle lineage, applied changes, and retained vs rolled-back outcomes.
3. Operator-visible truth:
   - `Game Loop` projects the strict stance truthfully,
   - does not keep showing the seeded mixed-site `10%` suspicious forwarded budgets as though they were the strict target,
   - shows the repeated change context,
   - shows measured movement toward the strict target rather than only recommendation plumbing or legacy mismatch rows,
   - and keeps later human traversal calibration explicit instead of implying that sim traffic itself proved likely-human safety.
4. Focused proof:
   - focused `make` paths exist and pass for strict-stance runtime behavior and repeated judged-cycle proof,
   - with local `/sim/public/*` or equivalent loopback sim-public proof accepted as the first truthful strict-loop surface,
   - while later live-host and human traversal calibration remain separate named follow-on proofs.
5. Insufficient evidence:
   - one successful loop,
   - one canary apply,
   - recommendation generation without retained improvement,
   - or unresolved dashboard/runtime mismatch where the operator surface still contradicts the claimed improvement.

### Frozen closure gate: `RSI-GAME-HO-2`

`RSI-GAME-HO-2` may move from blocked to completed only when all of the following are true:

1. Runtime/config truth:
   - both Scrapling and the later LLM attacker contribute pressure under `human_only_private`,
   - recommendations become bounded config changes,
   - later mixed-attacker runs occur against those changed configs,
   - and repeated retained changes show positive movement toward the strict target.
2. API/snapshot truth:
   - machine-first surfaces preserve mixed-attacker lineage and judged retain/rollback outcomes across repeated cycles.
3. Operator-visible truth:
   - `Red Team` and `Game Loop` truthfully show both lanes contributing to the strict-baseline proof,
   - without overstating mixed-attacker maturity from mere lane presence.
4. Focused proof:
   - focused `make` paths exist and pass for mixed-attacker repeated-cycle proof.
5. Insufficient evidence:
   - one mixed handoff,
   - both lanes merely appearing in recent runs,
   - or lack of repeated retained improvement under the strict stance.

## `VERIFY-GATE-1B`: Turn the gates into executable proof surfaces

After the gates are frozen, the next implementation work should wire any missing proof paths so completion is machine-checkable rather than conversational.

Required contract:

1. if a tranche claims runtime truth, it must have focused runtime or integration proof,
2. if a tranche claims dashboard truth, it must have rendered proof,
3. if a tranche claims repeated loop improvement, it must have proof over repeated judged cycles and retained changes rather than single-cycle plumbing,
4. and if a required proof path does not exist yet in `Makefile`, it must be added before the tranche is eligible for closure.
5. Strict sim-only proof paths should use the loopback-hosted `/sim/public/*` surface first when that is the truthful fast path already supported by the repo.
6. Human traversal calibration and live shared-host realism must be named as separate proof surfaces rather than being silently collapsed into the first strict sim-only gate.

## `VERIFY-GATE-1C`: Completion-language discipline and audit-trail cleanup

Correct misleading wording in the existing completion archive and set the rule for future entries.

Required contract:

1. planning or sequencing completions must say they landed planning or sequencing,
2. future-feature language must use “should prove”, “must prove”, or equivalent forward-looking phrasing until the implementation is actually delivered,
3. and no completion note may imply a feature tranche is closed while its TODO remains open.

# Sequencing Impact

The active mainline becomes:

1. `VERIFY-GATE-1`
2. `STANCE-MODEL-1`
3. `SIM-SCR-FULL-1`
4. `RSI-GAME-HO-1`
5. `SIM-LLM-1C3`
6. `RSI-GAME-HO-2`
7. only then `RSI-GAME-HV-1`

This is intentionally a process-first interruption. The point is to stop spending more engineering time under ambiguous closure standards.

# Definition Of Done

This tranche is satisfied when:

1. the active backlog makes `VERIFY-GATE-1` the immediate next item,
2. the sequencing docs reflect that new ordering,
3. the completion archive no longer uses misleading wording for the future Scrapling or strict-loop proof tranches,
4. and the repo explicitly records that future tranche closure requires executable acceptance proof rather than conversational progress summaries.
