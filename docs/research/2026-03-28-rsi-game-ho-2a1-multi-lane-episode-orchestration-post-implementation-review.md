Date: 2026-03-28
Status: Implemented

Related context:

- [`2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md`](2026-03-28-rsi-game-ho-2-combined-attacker-architecture-gap-review.md)
- [`../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md`](../plans/2026-03-28-rsi-game-ho-2-combined-attacker-orchestration-plan.md)
- [`../../src/admin/oversight_follow_on_runs.rs`](../../src/admin/oversight_follow_on_runs.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../Makefile`](../../Makefile)

# Objective

Land `RSI-GAME-HO-2A1` by making mixed-attacker candidate windows and loop continuations truthful machine-first episode state instead of one-lane-at-a-time folklore.

# What Landed

1. Added a shared ordered follow-on run contract in [`../../src/admin/oversight_follow_on_runs.rs`](../../src/admin/oversight_follow_on_runs.rs).
   - required runs now carry lane, duration, and per-lane status (`pending`, `running`, `materialized`, `expired`) instead of one global requested-lane placeholder.
2. Active canary candidate windows now persist ordered `required_runs`.
   - with frontier configured, the default set is `scrapling_traffic` then `bot_red_team`;
   - without frontier, the set remains Scrapling-only.
3. Loop continuation state now uses the same required-run contract.
   - terminal `improved` or `rollback_applied` judgments that stay outside budget now request a mixed ordered rerun set instead of only Scrapling.
4. Supervisor orchestration still starts exactly one run at a time.
   - after the first required lane materializes, the state returns to `pending` for the next lane;
   - the next beat then starts that next required lane automatically.
5. Post-sim oversight is now suppressed until the final required lane materializes.
   - the first completed lane in a mixed episode no longer triggers premature judgment.
6. Lane-specific duration truth is now explicit.
   - Scrapling keeps the runtime-dev short meaningful window;
   - `bot_red_team` now honors at least the LLM lane’s meaningful minimum window instead of inheriting the Scrapling shortcut.

# Acceptance Check

## State truth

Passed.

- Candidate windows and loop continuations no longer rely on one requested lane plus one follow-on run id.
- `GET /admin/oversight/agent/status` now exposes `required_runs` arrays for both `candidate_window` and `continuation_run`.

## Orchestration

Passed.

- The supervisor still dispatches one lane at a time.
- After a materialized Scrapling run, the next beat advances to the pending `bot_red_team` run instead of treating the episode as finished.

## Judgment discipline

Passed.

- The post-sim trigger now suppresses oversight execution when a matched required run materializes but more required lanes are still pending.

## Proof surfaces

Passed.

- Mixed-episode route proof:
  - `make test-rsi-game-mixed-episode-orchestration`
- Shared helper and status proof:
  - `make test-oversight-agent`
- Regression guard on prior Scrapling-only mainline:
  - `make test-rsi-game-mainline`

# Remaining Gaps

`RSI-GAME-HO-2` is still not complete.

The next blockers are now narrower and cleaner:

1. `RSI-GAME-HO-2A2`
   - controller-grade restriction scoring is still mostly Scrapling-native.
2. `RSI-GAME-HO-2A3`
   - operator/admin and dashboard projection still need explicit mixed-attacker judged-episode truth instead of inferred lane coincidence.

This tranche intentionally did not claim mixed-attacker loop success from lane visibility alone. It only landed the episode-state and sequencing truth required before that later proof can mean anything.

# Verification

- `make test-rsi-game-mixed-episode-orchestration`
- `make test-oversight-agent`
- `make test-rsi-game-mainline`
- `git diff --check`
