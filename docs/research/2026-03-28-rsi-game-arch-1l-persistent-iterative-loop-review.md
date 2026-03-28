Date: 2026-03-28
Status: active

Related context:

- [`2026-03-24-reference-stance-and-run-to-homeostasis-review.md`](2026-03-24-reference-stance-and-run-to-homeostasis-review.md)
- [`2026-03-24-rsi-game-mainline-first-working-loop-review.md`](2026-03-24-rsi-game-mainline-first-working-loop-review.md)
- [`2026-03-27-game-loop-board-state-and-shared-path-truth-review.md`](2026-03-27-game-loop-board-state-and-shared-path-truth-review.md)
- [`2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md`](2026-03-27-game-loop-restriction-recognition-and-abuse-confidence-review.md)
- [`2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md`](2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md)
- [`2026-03-28-rsi-game-arch-1k-post-canary-candidate-window-post-implementation-review.md`](2026-03-28-rsi-game-arch-1k-post-canary-candidate-window-post-implementation-review.md)
- [`../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md`](../plans/2026-03-24-reference-stance-and-run-to-homeostasis-implementation-plan.md)
- [`../plans/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-plan.md`](../plans/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-plan.md)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../scripts/run_with_oversight_supervisor.sh`](../../scripts/run_with_oversight_supervisor.sh)

# Objective

Check whether the current shared-host controller actually satisfies the stronger RSI requirement:

1. run adversary sim,
2. observe and diagnose,
3. apply one bounded config change,
4. rerun adversary sim,
5. retain or roll back,
6. and keep iterating until the board is under budget or homeostasis is reached.

# Findings

## 1. The controller is now very close to the desired loop shape

Three important pieces are already present:

1. periodic oversight execution is already perpetual on shared-host via [`../../scripts/run_with_oversight_supervisor.sh`](../../scripts/run_with_oversight_supervisor.sh),
2. post-canary candidate evidence now auto-materializes exactly once through `RSI-GAME-ARCH-1K`,
3. and after a canary closes, the next stage of the loop can already re-enter controller judgment from a fresh adversary run rather than from a hand-maintained operator action.

That means the repo is no longer missing the basic ingredients for multi-episode iteration.

## 2. What is still missing is explicit proof and explicit contract language

The code now proves:

1. one canary can open,
2. one follow-on protected Scrapling run can materialize,
3. one terminal retain or rollback judgment can archive cleanly.

But it does not yet explicitly prove the stronger next-step behavior:

1. after `improved` but still outside budget, the loop must request and materialize another bounded Scrapling rerun automatically,
2. after `rollback_applied` and still outside budget, the loop must do the same,
3. only the later post-rerun oversight judgment should open the next bounded canary,
4. and the stop condition must be machine-explicit rather than implicit folklore.

## 3. The stop condition is now clear enough to freeze

The persistent loop should continue only while all of the following remain true:

1. the latest judged board state is still outside budget,
2. homeostasis is not yet reached,
3. bounded tuning or later code-evolution review is still warranted,
4. the adversary rerun request is still legal and bounded,
5. and the config ring is not exhausted.

The loop must stop when any of these becomes false and record why:

1. `inside_budget`,
2. `homeostasis`,
3. `config_ring_exhausted`,
4. `code_evolution_referral`,
5. or other fail-closed guardrails.

## 4. Runtime-dev speed has two separate cadence seams

The current local-fast path is already split cleanly:

1. the candidate-window adversary rerun is shortened to the shortest meaningful `30s` runtime-dev duration,
2. the effective runtime-dev watch window is `300s`.

That means correctness and cadence are now separate concerns. The next tranche should freeze and prove persistent continuation first. Only after that should the repo consider whether runtime-dev periodic supervisor cadence needs a separate faster seam.

# Recommendation

Treat the next step as `RSI-GAME-ARCH-1L`: persistent iterative episode continuity.

Its first responsibility is proof:

1. write focused failing tests that prove the loop requests and materializes a fresh rerun after retained and rolled-back terminal outcomes when the board is still over budget,
2. only let the later post-rerun oversight judgment open the next bounded canary,
3. add controller code only where that proof fails,
4. then expose the continuation and stop reasons explicitly in the machine-first contract and operator docs.

This keeps the next slice honest. If the architecture already satisfies the non-stop RSI requirement, the repo should say so with proof instead of adding redundant orchestration. If it still stops short, the new proof will show exactly where.
