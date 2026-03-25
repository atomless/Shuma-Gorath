Date: 2026-03-24
Status: Completed

Related context:

- [`../2026-03-24-rsi-game-mainline-1b-shared-host-verifier-review.md`](2026-03-24-rsi-game-mainline-1b-shared-host-verifier-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../../scripts/tests/live_feedback_loop_remote.py`](../../scripts/tests/live_feedback_loop_remote.py)
- [`../../scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py)
- [`../../Makefile`](../../Makefile)
- [`../../docs/testing.md`](../../docs/testing.md)

# RSI-GAME-MAINLINE-1B Shared-Host Verifier Post-Implementation Review

## What landed

`RSI-GAME-MAINLINE-1B` now proves the first working game loop at the next truthful operational layer above the local Rust route tests.

The shared-host feedback-loop verifier behavior now proves:

1. the post-sim hook opens the bounded canary path,
2. a later periodic supervisor judgment can close that path at a terminal apply stage,
3. and the completed episode archive is visible through the same machine-first shared-host surfaces.

## What changed

1. [`scripts/tests/live_feedback_loop_remote.py`](../../scripts/tests/live_feedback_loop_remote.py)
   - added terminal apply-stage recognition
   - added episode-archive validation helpers
   - added an explicit verifier mode for terminal post-sim follow-on judgment
2. [`scripts/tests/test_live_feedback_loop_remote.py`](../../scripts/tests/test_live_feedback_loop_remote.py)
   - added the stronger shared-host verifier behavior proof for periodic terminal judgment plus episode-archive persistence
3. [`Makefile`](../../Makefile)
   - extended `make test-rsi-game-mainline` to include the shared-host verifier behavior proof
4. [`docs/testing.md`](../../docs/testing.md)
   - updated the testing contract to describe the broader mainline proof truthfully

## Important design choice

The stronger proof intentionally lives in the unitized shared-host verifier harness, not the default live remote command.

That keeps the proof honest because the protected watch-window contract is still at least one hour, so a short live smoke should not pretend it can always force a real terminal canary judgment without mutating operator objectives or violating the guarded watch-window rule.

## Result

Shuma now has:

1. a narrow local route-level proof in `src/admin/api.rs`,
2. and a stronger shared-host verifier-layer proof in `scripts/tests/live_feedback_loop_remote.py`.

That is enough to say the first working game loop is no longer only inferred from neighboring tests.

## Verification

1. `make test-live-feedback-loop-remote-unit`
2. `make test-rsi-game-mainline`
3. `make test-oversight-agent`
4. `git diff --check`
