# Dashboard Tab: Game Loop

Route: `#game-loop`  
Component: [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep loop outcome and controller judgment visible without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Shows a top-level `Closed-Loop Accountability` framing panel that makes the tab's role explicit.
- Exposes the bounded Game Loop accountability sections:
  - `Current Status`
  - `Recent Loop Progress`
  - `Outcome Frontier`
  - `What The Loop Decided`
  - `Where The Pressure Sits`
  - `Trust And Blockers`
- Projects current machine-first feedback-loop reads from:
  - `operator_snapshot_v1`
  - `benchmark_results_v1`
  - `oversight_history_v1`
  - `oversight_agent_status_v1`
- Surfaces:
  - current benchmark overall status, improvement status, tuning eligibility, and latest controller action,
  - bounded recent multi-loop oversight history rather than only the latest cycle,
  - suspicious-origin-cost versus likely-human-friction benchmark families as the first outcome-frontier slice,
  - benchmark escalation decision, candidate action families, and latest oversight apply or refusal context,
  - a bounded preview of remaining benchmark pressure plus recent config-change context from the operator snapshot,
  - and explicit trust or blocker rows for classification readiness, coverage, protected replay status, and tuning blockers.
- Full category-aware pressure breakdown and the richer final trust/actionability layer still belong to `MON-OVERHAUL-1C`.
- Directs operators and contributors to `#diagnostics` for deep subsystem inspection and rawer contributor-facing telemetry.

Refresh behavior:

- No tab-local manual or auto-refresh controls are exposed yet.
- On Game Loop activation, the dashboard runtime now refreshes shared config plus the bounded machine-first accountability reads listed above.

Writes:

- Read-only tab (no config writes).
