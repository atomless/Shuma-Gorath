Date: 2026-03-27
Status: proposed

Related context:

- [`../research/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md`](../research/2026-03-27-rsi-game-arch-1k-post-canary-candidate-window-review.md)
- [`../research/2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-post-implementation-review.md`](../research/2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-post-implementation-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../../src/admin/adversary_sim_api.rs`](../../src/admin/adversary_sim_api.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Objective

Turn the live Scrapling loop from “canary plus manual second run” into a real autonomous judged cycle by making post-canary candidate evidence materialize exactly once, under explicit lineage, without collapsing controller and adversary-generation responsibilities together.

# Core Decisions

1. Keep controller ownership and adversary-generation ownership separate.
2. Do not make the oversight agent directly generate attacker traffic.
3. Represent the need for post-canary candidate evidence as explicit machine state.
4. Let adversary-sim supervisor own the follow-on protected run.
5. Expose candidate-window lifecycle state in machine-first operator surfaces.

# Execution Tranche

## `RSI-GAME-ARCH-1K`

### Post-canary candidate-window materialization

Required contract:

1. when canary apply succeeds, the system must persist a single pending candidate-window request,
2. the request must be tied to the active canary and bounded to that watch window,
3. adversary-sim supervisor must honor the request exactly once by producing a protected post-change Scrapling run,
4. periodic judgment must consume the resulting candidate evidence when present,
5. and operator-visible surfaces must expose candidate-window request state explicitly.

Implementation steps:

1. Add failing tests first:
   - apply-side state tests proving canary apply creates a pending candidate-window request,
   - adversary-sim supervisor tests proving a pending request schedules exactly one follow-on protected run,
   - judgment tests proving the later periodic cycle consumes candidate evidence instead of failing as `candidate_window_not_materialized`,
   - operator-snapshot or status tests proving candidate-window lifecycle state is visible.
2. Add a dedicated candidate-window state record:
   - machine-readable,
   - linked to `canary_id`,
   - and owned separately from benchmark comparison payloads.
3. Update canary apply to create that state instead of only opening the watch window.
4. Update adversary-sim supervisor or lifecycle beat logic to consume the pending state and schedule the follow-on protected Scrapling run once.
5. Update post-sim trigger or result-ingest logic so the candidate request is marked materialized when the linked follow-on run finishes.
6. Thread the new lifecycle state into:
   - oversight status,
   - operator snapshot,
   - and any recent-change or archive surfaces that need to explain why judgment is still pending.
7. Add or refine focused Make targets so the proof path is narrow and truthful.
8. Run live local proof:
   - first run opens canary,
   - candidate request appears,
   - adversary sim automatically attacks the changed board,
   - periodic supervisor reaches a measured terminal outcome without manual rerun.

Acceptance criteria:

1. canary apply now creates explicit candidate-window request state,
2. adversary-sim supervisor materializes exactly one protected post-change Scrapling run for that request,
3. periodic supervisor can reach `improved` or `rollback_applied` from automatically materialized candidate evidence,
4. missing candidate evidence remains explicit and truthful when the request expires,
5. operator-visible machine-first surfaces expose candidate request lifecycle state,
6. and no component directly collapses controller logic into attacker-traffic generation.

Proof:

1. a new focused `make` target for candidate-window orchestration proof, if the current `Makefile` lacks one,
2. `make test-rsi-game-mainline`,
3. `make test-adversary-sim-runtime-surface`,
4. `make test`,
5. and authenticated live local evidence from:
   - `GET /admin/operator-snapshot`
   - `GET /admin/oversight/agent/status`
   - `GET /admin/oversight/history`

# Sequencing

1. Discuss and ratify the ownership decision before implementation.
2. Land `RSI-GAME-ARCH-1K` before reopening “fully working live Scrapling RSI” claims.
3. Only after this slice lands should the repo return to:
   - `RSI-GAME-ARCH-1E` retirement cleanup,
   - later `SIM-LLM-1C3`,
   - and the still-blocked frontier-LLM code-evolution ring.

# Definition Of Done

This tranche is complete when:

1. live Scrapling canary episodes automatically produce one protected post-change candidate window,
2. the next judgment is measured rather than missing-evidence fail-closed,
3. operator surfaces explain candidate-window lifecycle truth explicitly,
4. and the resulting ownership split still matches the board-game architecture:
   - controller changes the board,
   - adversary sim attacks the board,
   - judge compares the results.
