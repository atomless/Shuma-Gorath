# RSI-SCORE-2D Move Selection And Config-Ring Exhaustion Post-Implementation Review

Date: 2026-03-26
Status: Closed

Related context:

- [`../../src/admin/oversight_patch_policy.rs`](../../src/admin/oversight_patch_policy.rs)
- [`../../src/admin/oversight_reconcile.rs`](../../src/admin/oversight_reconcile.rs)
- [`../../src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md`](../../docs/plans/2026-03-26-rsi-score-2-exploit-first-judge-and-diagnoser-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Scope Reviewed

This closeout reviewed `RSI-SCORE-2D`: separate judge state, diagnosis, and move selection more sharply, require smallest-effective localized repair selection, and emit explicit `config_ring_exhausted` or code-evolution referral when bounded safe config moves cannot credibly close the exploit gap.

# What Landed

1. Shuma now ranks bounded config candidates explicitly instead of collapsing directly to one opaque patch proposal.
2. `OversightReconcileResult` now carries three separate planes:
   - `judge`
   - `diagnosis`
   - `move_selection`
3. Move selection now preserves:
   - ranked candidate families,
   - the selected repair surface,
   - named breach-locus lineage,
   - explicit config-ring status,
   - and explicit code-evolution status.
4. The reconcile path now distinguishes:
   - `observe_longer`
   - `recommend_patch`
   - `config_ring_exhausted`
   - and `code_evolution_referral`
   without pretending those are all the same kind of controller outcome.
5. Repeated rolled-back bounded moves for the same top-ranked family now escalate to `config_ring_exhausted` instead of repeatedly retrying near-equivalent config moves.
6. A focused make path, `make test-rsi-score-move-selection`, now proves ranked move selection, localized repair lineage, explicit code-evolution referral, and config-ring exhaustion behavior end to end.

# Acceptance Review

`RSI-SCORE-2D` required sharper separation between judge, diagnoser, and move selector; smallest credible localized repair selection; and an explicit route from bounded config failure to exhaustion or code referral.

Those criteria are now satisfied:

1. judge state, diagnosis, and move-selection lineage are emitted as separate machine-readable structures;
2. the reconcile path ranks bounded candidates and chooses the smallest available repair surface instead of jumping straight to an opaque patch;
3. localized repair lineage now ties selected moves to named breach loci and explicit repair-surface candidates;
4. repeated failed bounded moves can produce `config_ring_exhausted`;
5. explicit code-gap cases can produce `code_evolution_referral`;
6. and the repo now has the required focused proof through:
   - `make test-rsi-score-move-selection`
   - `make test-oversight-reconcile`
   - `make test-controller-action-surface`

The key behavioral correction is this:

the controller no longer treats “there is benchmark pressure” and “here is the bounded move we should make” as the same step.

It now exposes the scoring state, the localized diagnosis, and the chosen smallest repair separately so later UI and operator surfaces can explain why a move was selected or why config tuning has been exhausted.

# Shortfalls Found

This slice does not yet project the richer judge truth clearly enough in the `Game Loop` tab.

The following planned work remains open:

1. `RSI-SCORE-2E` richer Game Loop projection for guardrails, exploit progress, evidence quality, urgency, breach loci, and exhaustion or code-referral state;
2. the later strict-loop proof tranches under `RSI-GAME-HO-1`.

So this tranche makes move selection controller-grade, but it does not yet make the operator-facing Game Loop projection equally clear.

# Verification

- `make test-rsi-score-move-selection`
- `git diff --check`
