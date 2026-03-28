# RSI-GAME-ARCH-1J Runtime-Dev Effective Watch-Window Post-Implementation Review

Date: 2026-03-27  
Status: implemented

## Scope

Give runtime-dev a truthful accelerated oversight cadence so the local strict Scrapling loop can complete judged retain-vs-rollback cycles without waiting a production-shaped day, while keeping runtime-prod semantics unchanged and operator-visible surfaces honest about the effective cadence in use.

## What Landed

1. A new env-only runtime-dev seam now exists:
   - `SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS`
2. The effective oversight watch-window helper now distinguishes:
   - declared objective window,
   - effective runtime-dev watch window,
   - and the active source of truth for that effective cadence.
3. Runtime-prod continues to use declared operator-objective policy and ignores the runtime-dev override.
4. Machine-first operator surfaces now expose the cadence truth explicitly:
   - `watch_window_seconds`
   - `declared_watch_window_seconds`
   - `watch_window_source`
5. The same effective cadence now drives:
   - canary apply,
   - watch-window-open state,
   - recent-change timing,
   - and terminal judgment timing.
6. Bootstrap, setup, docs, and focused Make targets now support the runtime-dev cadence seam end to end.

## Why This Matters

Before this slice, the live local strict Scrapling loop could already:

1. diagnose localized pressure,
2. recommend a bounded patch,
3. and apply a bounded canary.

But it still could not complete a judged live cycle at a practical development cadence because the operator-owned profile still declared a `24h` watch window. That meant the next blocker was no longer controller correctness but controller cadence.

This slice fixes that without pretending production semantics changed or hiding the local acceleration from operator-visible truth surfaces.

## Live Proof Outcome

After a clean local reset and runtime-dev start:

1. `GET /admin/operator-snapshot` and `GET /admin/oversight/agent/status` showed the expected explicit cadence split:
   - `watch_window_seconds=300`
   - `declared_watch_window_seconds=86400`
   - `watch_window_source=runtime_dev_override`
2. A fresh strict Scrapling run produced:
   - `protected_evidence.protected_basis=live_scrapling_runtime`
   - `tuning_eligibility.status=eligible`
   - `apply.stage=canary_applied`
3. After the accelerated effective watch window elapsed, the next periodic supervisor cycle reached a real terminal outcome:
   - `apply.stage=rollback_applied`
   - `rollback_reason=candidate_window_not_materialized`
4. That first terminal result is important because it proves the runtime-dev cadence seam is now real:
   - the controller no longer waits for a day,
   - and the live loop can progress from bounded mutation into terminal judgment.
5. A second live proof then materialized a post-canary candidate window by running Scrapling again while the canary was open.
6. The next periodic supervisor cycle then reached a meaningful measured terminal judgment:
   - `apply.stage=rollback_applied`
   - `comparison_status=neutral`
   - `rollback_reason=candidate_comparison_neutral`
   - and the episode archive recorded `cycle_judgment=flat`

So the cadence seam is now genuinely working. The next blocker is not watch-window speed anymore. It is automatic post-canary candidate-window materialization.

## Verification

- `make test-operator-objectives-contract`
- `make test-oversight-apply`
- `make test-rsi-game-mainline`
- `make test-adversary-sim-runtime-surface`
- live authenticated API evidence from:
  - `GET /admin/operator-snapshot`
  - `GET /admin/oversight/agent/status`
  - `GET /admin/oversight/history`
  - timed `POST /internal/oversight/agent/run`

## Remaining Follow-On

1. The next live RSI blocker is now explicit:
   - if no protected post-canary candidate window is materialized, terminal judgment fail-closes as `candidate_window_not_materialized`.
2. A truly autonomous local Scrapling RSI loop therefore still needs a way to produce candidate evidence after a canary opens without manual babysitting.
3. That follow-on is architecture-significant because it changes which component owns candidate-window materialization, so it should be discussed before implementation.
