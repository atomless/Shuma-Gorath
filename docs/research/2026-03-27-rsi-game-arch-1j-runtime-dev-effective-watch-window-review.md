Date: 2026-03-27
Status: proposed

Related context:

- [`2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-post-implementation-review.md`](2026-03-27-rsi-game-arch-1h-live-protected-evidence-stale-guard-post-implementation-review.md)
- [`2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-post-implementation-review.md`](2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-post-implementation-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)

# Scope

Decide how the local strict Scrapling RSI loop should progress from a truthful `canary_applied` or `watch_window_open` state into a truthful judged terminal outcome without waiting a full production-shaped day inside runtime-dev.

# Findings

1. The live local strict Scrapling loop is now genuinely past diagnosis:
   - protected evidence can be `live_scrapling_runtime`,
   - bounded config mutation can be `canary_applied`,
   - and the current live local loop is now sitting in `watch_window_open`.
2. The current local operator-owned objectives profile still declares:
   - `source=manual_admin_profile`
   - `rollout_guardrails.automated_apply_status=canary_only`
   - `window_hours=24`
3. `operator_objectives_watch_window_seconds()` still derives watch-window cadence purely from whole-hour objective policy:
   - `window_hours * 3600`
4. The earlier stronger proof plan was correct to avoid pretending that production-faithful live proof can always close a real watch window inside a short smoke budget.
5. That does not solve the current local development problem:
   - the live local RSI loop can now mutate config,
   - but it still cannot iterate on retain-vs-rollback judgment at a useful development cadence.
6. There are two plausible fixes:
   - widen the operator-objectives rule surface from hour-granularity to second-granularity,
   - or add an explicit runtime-dev-only effective watch-window override for oversight judgment cadence.
7. The first option is architecturally cleaner long-term, but it is a broader rule-surface migration:
   - API contract,
   - snapshot contract,
   - dashboard adapters,
   - tests,
   - and persisted objective validation all currently speak in hours.
8. The second option is narrower and better aligned to the current urgent blocker if and only if it stays explicit:
   - runtime-prod must ignore it,
   - runtime-dev must surface it as an effective cadence override rather than silently rewriting the declared rule surface,
   - and machine-first operator surfaces must show that the effective watch window is locally accelerated.

# Decision

Take the narrower next step now:

1. Add an explicit env-only runtime-dev oversight cadence override:
   - `SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS`
2. Honor it only when:
   - `SHUMA_RUNTIME_ENV=runtime-dev`
   - and the value is positive and valid.
3. Keep runtime-prod and default behavior unchanged:
   - declared `operator_objectives_v1.window_hours` remains the governing watch-window input there.
4. Thread the effective watch-window seconds consistently through:
   - oversight apply,
   - decision ledger,
   - recent changes,
   - and operator-visible machine-first surfaces that already expose watch-window timing.
5. Make the override explicit in operator truth:
   - the repo must not leave a hidden mismatch where the declared objectives still imply a 24-hour watch window while the controller is secretly judging on a five-minute cadence.
6. Defer the larger "seconds-granularity objective profile" redesign unless this explicit runtime-dev seam proves insufficient or too confusing.

# Why This Is The Right Next Move

1. The current blocker is no longer controller correctness; it is controller cadence.
2. The current local development need is not a new production policy. It is a way to iterate truthfully on judged local cycles.
3. The repo already separates runtime-dev from runtime-prod posture, so a runtime-dev-only cadence seam is a smaller and safer local fix than a whole rule-surface migration.
4. The override is only acceptable if the system tells the truth about it. Hidden cadence acceleration would make the Game Loop harder to trust, not easier.

# Acceptance Direction

This follow-on is complete only when:

1. runtime-dev can use an explicit effective watch-window override shorter than the declared objective hours,
2. runtime-prod and default behavior remain unchanged,
3. operator-visible machine-first surfaces reveal the effective accelerated cadence rather than hiding it,
4. the live local strict Scrapling loop can progress from `canary_applied` or `watch_window_open` into `improved` or `rollback_applied` within the accelerated runtime-dev cadence,
5. and the repo docs make clear that this is a runtime-dev local-iteration aid rather than a production-faithful watch-window proof replacement.
