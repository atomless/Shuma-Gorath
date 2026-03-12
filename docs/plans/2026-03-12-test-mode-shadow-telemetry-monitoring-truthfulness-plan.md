# Test-Mode Shadow Telemetry and Monitoring Truthfulness Plan

Date: 2026-03-12  
Status: Proposed

Reference context:

- [`docs/configuration.md`](../configuration.md)
- [`docs/observability.md`](../observability.md)
- [`docs/dashboard-tabs/monitoring.md`](../dashboard-tabs/monitoring.md)
- [`src/runtime/test_mode/mod.rs`](../../src/runtime/test_mode/mod.rs)
- [`src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`src/admin/api.rs`](../../src/admin/api.rs)
- [`dashboard/src/lib/components/dashboard/MonitoringTab.svelte`](../../dashboard/src/lib/components/dashboard/MonitoringTab.svelte)
- [`dashboard/src/lib/components/dashboard/monitoring-view-model.js`](../../dashboard/src/lib/components/dashboard/monitoring-view-model.js)
- [`todos/todo.md`](../../todos/todo.md)

## Objective

Make `test_mode` a truthful long-running operator shadow mode:

1. no noisy per-request terminal logging by default,
2. first-class backend-authored shadow telemetry semantics,
3. monitoring that clearly distinguishes simulated "would act" outcomes from real enforcement,
4. bounded storage and query cost under real shared-host traffic.

## Current-State Shortfalls

1. Test mode still writes per-request stdout lines for most shadow decisions via `crate::log_line(...)`, which assumes short-lived local debugging rather than long-running hosted operator use.
2. The runtime records shadow meaning indirectly through free-text `reason`/`outcome` strings such as `"[TEST MODE]"` and `would_block`, rather than through explicit structured execution semantics.
3. Monitoring/event UI derives defense and outcome meaning heuristically from those strings, so aggregate monitoring can blur simulated actions with real enforcement.
4. Test mode currently under-represents clean-pass traffic: "would allow" is returned to the client, but no corresponding structured event is recorded.
5. Any fix that starts logging every clean pass as a raw event would create unacceptable event-log amplification on busy sites.

## Non-goals

1. Replacing the current monitoring storage architecture.
2. Redesigning test-mode activation source or lifecycle beyond what is necessary to keep this work compatible with the open `SIM2-R4-4` lifecycle tranche.
3. Changing normal enforcement semantics in non-test mode.

## Architecture Decisions

### 1. Reuse the normal policy graph and effect boundary

The cleanest design is not to keep expanding the current early `test_mode` short-circuit path. Today `test_mode` returns before the normal policy graph and effect execution pipeline runs, which means later ladder decisions are never evaluated through the same contract as normal enforcement.

That architecture is not good enough for a truthful long-running shadow mode. The runtime should instead:

1. evaluate the same `PolicyDecision` sequence it would in normal mode,
2. route those decisions through the existing plan/effect boundary,
3. execute them in either `enforced` or `shadow` mode,
4. emit telemetry from that boundary-authored execution result.

This keeps the system-path under inspection identical to the real runtime and avoids maintaining a parallel shadow-only decision tree.

### 2. Treat test mode as a first-class shadow-execution contract

Once shadow mode is anchored on the normal policy/effect path, the runtime must stop relying on free-text strings as the authoritative source of truth for test-mode behavior.

Monitoring/event surfaces should receive backend-authored execution semantics, for example:

- execution mode: `enforced` or `shadow`,
- intended action: `allow`, `challenge`, `maze`, `block`, `tarpit`, `ip_range_action`, etc,
- enforcement applied: `true` or `false`,
- shadow source: `test_mode`.

Pre-launch, the cleanest place to carry this is the stored/presented event record contract or a monitoring-specific backend enrichment layer authored from the shared decision/effect boundary. The dashboard must consume those explicit semantics, not infer them by parsing `"[TEST MODE]"` or `would_*`.

This must reuse the existing monitoring/event storage path rather than inventing a parallel shadow-specific storage family. The telemetry-efficiency tranche already established bucket-indexed reads, retention tiers, rollups, and query-budget accounting as the canonical path. Test-mode truthfulness must enrich that path, not bypass it.

### 3. Remove default per-request stdout logging from hosted test mode

Structured telemetry already exists and is the right operator surface.

Default stdout logging for every shadow decision is operationally noisy and scales with traffic volume. That is the wrong default for post-deploy shadow tuning on a real site. The default posture should therefore be:

- no per-request terminal/stdout test-mode decision logging,
- visibility through monitoring/event telemetry only.

If a local developer-only debug path is still desired later, it must be explicit and opt-in, not the default hosted behavior.

### 4. Keep raw-event storage bounded by separating "interesting shadow decisions" from aggregate shadow totals

The system must not compensate for under-modeled shadow telemetry by logging one raw event for every clean pass.

The clean shape is:

- raw event rows for shadow decisions that would have imposed meaningful friction or enforcement,
- aggregate counters/rollups for high-volume pass-through/no-op traffic,
- dashboard monitoring summaries that surface both shadow pressure and actual enforcement pressure.

This preserves observability while avoiding event-log amplification.

This also means:

- no new whole-keyspace scan path may be introduced for shadow-mode visibility,
- no new unbounded/high-cardinality dimensions may be introduced purely for test mode,
- shadow-mode counters and rollups must remain compatible with the existing bucket-indexed retention/query model.

### 5. Monitoring must render shadow and enforced outcomes as different operator truths

Monitoring should not show "blocks imposed" when test mode only simulated those blocks.

The operator-facing model should instead distinguish:

- `Enforced`
- `Would Challenge`
- `Would Maze`
- `Would Block`
- `Would Tarpit`
- `Would Apply IP Range Action`

Raw feeds should preserve the exact underlying payload, while summary cards/trend blocks/filter labels should render the shadow distinction explicitly.

### 6. Keep this work activation-source-agnostic

`SIM2-R4-4` is still open on whether `test_mode` remains persisted config or becomes narrower runtime/session state. This tranche must therefore avoid hard-coding assumptions about where activation comes from.

The shadow telemetry contract must hang off the resolved effective runtime flag for the current request, so it remains correct whether activation later comes from persisted config, an ephemeral operator session, or a narrower runtime toggle.

## Recommended Delivery Sequence

### Phase 1: Policy/Effect Boundary Refactor First

1. Rebase test mode on the normal policy graph and effect/plan boundary instead of the current early shadow short-circuit.
2. Ensure shadow mode can observe the same `PolicyDecision` sequence as normal enforcement without executing friction/block side effects.
3. Keep the existing response surface truthful while the telemetry contract migrates.

Acceptance criteria:

1. Test mode no longer depends on a parallel shadow-only decision tree for later ladder behavior.
2. The same policy decisions can be executed in either `enforced` or `shadow` mode.

### Phase 2: Backend Contract and Logging Discipline

1. Define the canonical shadow-enforcement telemetry vocabulary.
2. Add backend-authored shadow metadata to event/monitoring presentation paths.
3. Remove default stdout logging from the test-mode runtime path.
4. Decide which shadow outcomes warrant raw event rows and which belong only in aggregate counters.
5. Add/adjust counters so quiet pass-through traffic is represented without per-request raw event amplification.

Acceptance criteria:

1. No dashboard logic needs to detect test mode by parsing `"[TEST MODE]"`.
2. Long-running hosted test mode does not flood stdout/journald.
3. High-volume shadow-mode traffic remains storage-bounded.

### Phase 3: Monitoring UI Truthfulness

1. Update monitoring summaries, trend blocks, recent events, and raw feed helpers to render shadow outcomes explicitly.
2. Make test-mode monitoring language say "would have happened" instead of implying enforcement.
3. Keep raw feed fidelity while preventing summary charts from overstating enforcement.

Acceptance criteria:

1. Operators can tell at a glance whether a chart/table reflects shadow or enforced outcomes.
2. Monitoring no longer conflates "would block" with an actual block imposed.

### Phase 4: Docs and Verification

1. Update operator docs for long-running test mode usage.
2. Add unit, integration, and dashboard coverage for shadow semantics.
3. Tie the final lifecycle wording back into `SIM2-R4-4`.

Acceptance criteria:

1. Docs describe test mode as a credible long-running operator shadow posture.
2. Verification proves stdout quieting, shadow telemetry presence, and truthful monitoring rendering.

## Changes That Should Happen First or Alongside This Work

1. `SIM2-R4-4-4` should remain open until this shadow telemetry contract is defined, because test-mode semantics are not truthful end-to-end otherwise.
2. The current early `src/runtime/test_mode/mod.rs` short-circuit should not be expanded as the long-term solution; the first implementation step must be to move test-mode truthfulness onto the normal policy/effect boundary.
3. Dashboard changes should not happen before backend-authored shadow semantics exist; otherwise the UI will keep relying on fragile string parsing.
4. Any decision to preserve local-only stdout debug logging should be made explicitly during this tranche, not left implicit.
5. Verification must include real hosted-style traffic volume checks so the storage/logging claims are proven against operator use, not only local development.
6. Telemetry-efficiency guarantees from [`docs/plans/2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md`](./2026-03-11-telemetry-storage-query-efficiency-excellence-plan.md) remain release-blocking for this work:
   - normal monitoring reads must stay bucket-addressable,
   - no new whole-keyspace scans may be added for test-mode rendering,
   - shadow telemetry must not introduce a parallel storage/query path that escapes existing retention and query-budget governance.

## Verification Strategy

1. `make test-unit`
2. `make test-integration`
3. `make test-dashboard-e2e`
4. `make test`
5. Focused shared-host verification with `test_mode=true` and sustained traffic proving:
   - bounded stdout behavior,
   - shadow telemetry visibility,
   - truthful monitoring rendering,
   - no enforcement side effects.

## Definition of Done

1. Test mode reuses the normal policy/effect path rather than a parallel shadow-only decision tree.
2. Test mode no longer depends on default per-request stdout logging for operator visibility.
3. Monitoring and event payloads expose first-class shadow semantics.
4. Dashboard monitoring distinguishes simulated actions from enforced actions without heuristic string parsing.
5. Shadow-mode observability remains bounded under sustained traffic.
6. `SIM2-R4-4` can close its remaining semantics/docs items based on this delivered contract.
7. The tranche does not regress telemetry-efficiency guarantees: no added whole-keyspace scans, no unbounded shadow-specific cardinality growth, and no new storage/query path outside the existing bucket-indexed retention model.
