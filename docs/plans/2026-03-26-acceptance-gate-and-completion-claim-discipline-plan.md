Date: 2026-03-26
Status: Proposed

Related context:

- [`../research/2026-03-26-acceptance-gate-and-completion-claim-discipline-review.md`](../research/2026-03-26-acceptance-gate-and-completion-claim-discipline-review.md)
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

## `VERIFY-GATE-1B`: Turn the gates into executable proof surfaces

After the gates are frozen, the next implementation work should wire any missing proof paths so completion is machine-checkable rather than conversational.

Required contract:

1. if a tranche claims runtime truth, it must have focused runtime or integration proof,
2. if a tranche claims dashboard truth, it must have rendered proof,
3. if a tranche claims repeated loop improvement, it must have proof over repeated judged cycles and retained changes rather than single-cycle plumbing,
4. and if a required proof path does not exist yet in `Makefile`, it must be added before the tranche is eligible for closure.

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
