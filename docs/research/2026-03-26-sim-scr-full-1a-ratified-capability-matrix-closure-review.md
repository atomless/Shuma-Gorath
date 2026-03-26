Date: 2026-03-26
Status: Superseded

# SIM-SCR-FULL-1A Ratified Capability Matrix Closure Review

Superseded later on 2026-03-26 by [`2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md`](2026-03-26-sim-scr-full-spectrum-adversary-mandate-review.md).

This note captured the narrower request-native ownership interpretation that appeared to close `SIM-SCR-FULL-1A` before the clarified full-spectrum adversary mandate reopened the matrix under a stronger source-of-truth requirement.

## Question

Is `SIM-SCR-FULL-1A` still an open capability-matrix research task, or has the repository already frozen the ratified Scrapling capability matrix and simply drifted in how the later backlog names it?

## Findings

### 1. The matrix-freeze work is already landed under `SIM-SCR-CAP-1`

The repository already contains the matrix-freeze tranche that `SIM-SCR-FULL-1A` asks for:

- [`2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md`](2026-03-25-sim-scr-cap-1-upstream-capability-matrix-review.md)
- [`../plans/2026-03-25-sim-scr-cap-1-capability-matrix-plan.md`](../plans/2026-03-25-sim-scr-cap-1-capability-matrix-plan.md)
- [`../../todos/completed-todo-history.md`](../../todos/completed-todo-history.md)

That earlier tranche already froze the official-upstream-vs-Shuma matrix, the omission ledger, and the explicit adopt versus assign versus exclude outcomes for the current Scrapling-owned lane.

### 2. The ratified matrix is already explicit about what Scrapling does and does not own today

The settled matrix currently says:

1. adopt now: request-native impersonation and realistic header shaping,
2. keep adopted: request-native session continuity, cookies, and crawl or traversal mechanics,
3. assign away from the current request-native lane: dynamic browser automation, stealth browser automation, and Cloudflare-style challenge solving,
4. explicitly exclude for now: proxy routing on the shared-host local mainline and Camoufox-style shaping.

So `SIM-SCR-FULL-1A` is not waiting on discovery of the matrix. It is waiting on the repo to acknowledge that the matrix was already ratified and to sequence the remaining work truthfully.

### 3. The real drift is backlog and plan wording, not missing research

Despite the settled matrix above:

- [`../../todos/todo.md`](../../todos/todo.md) still leaves `SIM-SCR-FULL-1A` open as though the matrix freeze were still pending,
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md) still blocks `SIM-SCR-CHALLENGE-2C` on a future `SIM-SCR-FULL-1A` refresh,
- and [`../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md`](../plans/2026-03-25-scrapling-full-power-human-only-loop-before-relaxation-plan.md) still reads as though `1A` were a fresh open sub-tranche rather than a satisfied prerequisite.

That wording creates the false impression that the capability-ownership question remains unresolved when the repo has already answered it.

### 4. Closing `SIM-SCR-FULL-1A` does not close `SIM-SCR-FULL-1`

This review does not convert baseline Scrapling into full-power Scrapling.

The remaining open work is still:

1. `SIM-SCR-FULL-1B`: implement the remaining power required by the already-ratified matrix for the lane Shuma says Scrapling owns,
2. `SIM-SCR-FULL-1C`: add receipt-backed proof showing which defenses and categories the matured lane actually exercised, passed, and failed,
3. and only then `RSI-GAME-HO-1`: prove the strict `human_only_private` loop over that matured Scrapling lane.

So the parent `SIM-SCR-FULL-1` tranche remains open even though the matrix-freeze prerequisite is already satisfied.

## Decision

Treat `SIM-SCR-FULL-1A` as already satisfied by the settled `SIM-SCR-CAP-1` matrix and omission-ledger work.

Practical consequences:

1. remove `SIM-SCR-FULL-1A` from the active execution queue,
2. keep `SIM-SCR-FULL-1B` and `SIM-SCR-FULL-1C` as the real active Scrapling maturity slices,
3. keep `SIM-SCR-CHALLENGE-2C` blocked under the current ratified matrix unless a later owned-surface decision or request-native insufficiency proof deliberately reopens broader browser or stealth adoption,
4. and keep `SIM-SCR-FULL-1` itself open until the implementation and receipt-backed proof slices are complete.

## Verification

This slice was docs-only.

Evidence:

- `git diff --check`
