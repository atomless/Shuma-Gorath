Date: 2026-03-27
Status: proposed

Related context:

- [`../research/2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-review.md`](../research/2026-03-27-rsi-game-arch-1j-runtime-dev-effective-watch-window-review.md)
- [`../research/2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-post-implementation-review.md`](../research/2026-03-27-rsi-game-arch-1i-runtime-dev-canary-seed-post-implementation-review.md)
- [`../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md`](../plans/2026-03-24-rsi-game-mainline-first-working-loop-plan.md)
- [`../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md`](../plans/2026-03-27-game-loop-architecture-alignment-and-retirement-plan.md)
- [`../../src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs)
- [`../../src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs)
- [`../../src/admin/oversight_agent.rs`](../../src/admin/oversight_agent.rs)
- [`../../docs/configuration.md`](../../docs/configuration.md)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Objective

Give runtime-dev a truthful accelerated oversight cadence so the local strict Scrapling loop can complete judged retain-vs-rollback cycles without waiting a production-shaped day, while keeping runtime-prod semantics unchanged and operator-visible surfaces honest about the effective cadence in use.

# Core Decisions

1. Do not change runtime-prod or default watch-window behavior in this slice.
2. Do not silently rewrite the operator-owned declared objective profile just to speed up runtime-dev.
3. Do not introduce a hidden local shortcut. If runtime-dev uses an accelerated effective cadence, the machine-first surfaces must say so.
4. Keep the rule-surface migration from hours to seconds deferred unless this narrower seam proves insufficient.

# Execution Tranche

## `RSI-GAME-ARCH-1J`

### Runtime-dev effective watch-window cadence

Required contract:

1. runtime-dev may use an env-only effective oversight watch-window override:
   - `SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS`
2. runtime-prod and default behavior must continue using declared objective policy without override,
3. the effective watch-window helper must be the single source of truth for:
   - canary apply,
   - watch-window-open state,
   - terminal judgment,
   - decision-ledger watch-window seconds,
   - recent-change timing surfaces,
   - and any other machine-first projection that already claims watch-window timing,
4. operator-visible surfaces must make the runtime-dev accelerated cadence explicit enough that the user can tell:
   - declared objective window,
   - effective local dev watch window,
   - and whether the override is active,
5. and the local strict Scrapling loop must be able to reach a real judged terminal outcome inside the accelerated cadence.

Implementation steps:

1. Add the failing tests first:
   - config/objectives helper tests for runtime-dev override behavior,
   - oversight apply or agent tests proving effective watch-window seconds come from the override in runtime-dev,
   - operator-snapshot or recent-change tests proving operator-visible timing surfaces show the effective cadence.
2. Add the env-only config variable and validation:
   - define it in `config/defaults.env`,
   - expose a config helper in `src/config/mod.rs`,
   - wire setup/bootstrap and docs per the repo’s env-variable lifecycle contract.
3. Thread a single effective watch-window helper through the controller surfaces:
   - `operator_snapshot_objectives`,
   - `oversight_apply`,
   - recent changes and decision ledger,
   - any operator-snapshot fields that already expose watch-window timing.
4. Make the operator truth explicit:
   - dashboard or machine-first payload must reveal the effective accelerated cadence,
   - and docs must explain that this is a runtime-dev local-iteration override rather than the production-faithful proof path.
5. Prove the live local loop:
   - open a canary,
   - let the accelerated effective watch window elapse,
   - trigger the next agent cycle,
   - and observe `improved` or `rollback_applied`.

Acceptance criteria:

1. `SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS` is accepted only in runtime-dev and ignored elsewhere,
2. focused tests prove:
   - runtime-dev effective watch-window override behavior,
   - runtime-prod/default no-override behavior,
   - controller and recent-change surfaces use the effective cadence consistently,
3. docs describe the new env-only local-dev cadence seam and its explicit limitation,
4. live local evidence shows the strict Scrapling loop can now complete a judged cycle under the accelerated runtime-dev cadence,
5. and the repo still keeps the earlier production-faithful watch-window proof notes intact rather than retroactively claiming the dev override is the same thing.

Proof:

1. `make test-operator-objectives-contract`
2. `make test-oversight-apply`
3. `make test-rsi-game-mainline`
4. `make test-adversary-sim-runtime-surface`
5. authenticated local API evidence from:
   - `GET /admin/operator-snapshot`
   - `GET /admin/oversight/agent/status`
   - and, if needed, `GET /admin/oversight/history`

# Sequencing

1. Land `RSI-GAME-ARCH-1J` before further live-loop completion claims.
2. Only after the local loop can complete real judged cycles at development cadence should the repo return to:
   - `RSI-GAME-ARCH-1E` retirement,
   - later LLM runtime reopening,
   - or later frontier-LLM code-evolution planning execution.

# Definition Of Done

This tranche is complete when:

1. runtime-dev has an explicit effective watch-window cadence override,
2. runtime-prod remains unchanged,
3. machine-first operator surfaces tell the truth about the accelerated local cadence,
4. the local strict Scrapling loop can progress from bounded mutation to a real judged outcome without waiting a full day,
5. and the repo docs and TODO chain describe this as a local-dev iteration seam rather than a production-proof shortcut.
