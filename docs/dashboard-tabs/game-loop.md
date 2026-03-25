# Dashboard Tab: Game Loop

Route: `#game-loop`  
Component: [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep loop outcome and controller judgment visible without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Exposes the bounded Game Loop accountability sections:
  - top status cards and runtime posture rows with no extra framing pane
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
  - true numeric budget usage for likely-human friction plus suspicious forwarded requests, bytes, and latency,
  - taxonomy rows as `Category Target Achievement` rather than fake configured per-category budgets,
  - benchmark escalation decision, candidate action families, and latest oversight apply or refusal context,
  - a bounded preview of remaining benchmark pressure plus recent config-change context from the operator snapshot,
  - and explicit trust or blocker rows for classification readiness, coverage, protected replay status, tuning eligibility, verified-identity guardrails, and the latest compact Scrapling evidence-readiness corroboration.
- Directs operators and contributors to `#diagnostics` for deep subsystem inspection and rawer contributor-facing telemetry.
- Keeps detailed adversary proof out of the tab:
  - `Red Team` is where operators verify Scrapling personas, categories, and owned-surface receipts,
  - `Game Loop` only shows a bounded corroborating signal so attacker truth is visible without turning the tab into a forensic adversary surface.

Refresh behavior:

- No tab-local manual or auto-refresh controls are exposed yet.
- On Game Loop activation, the dashboard runtime now refreshes shared config plus the bounded machine-first accountability reads listed above.

Writes:

- Read-only tab (no config writes).
