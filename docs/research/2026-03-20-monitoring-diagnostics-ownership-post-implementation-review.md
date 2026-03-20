# Monitoring Diagnostics Ownership Post-Implementation Review

Date: 2026-03-20

## Scope Reviewed

- `dashboard/src/lib/components/dashboard/MonitoringTab.svelte`
- `dashboard/src/lib/components/dashboard/DiagnosticsTab.svelte`
- `dashboard/src/routes/+page.svelte`
- `dashboard/src/lib/runtime/dashboard-runtime-refresh.js`
- `dashboard/src/lib/domain/dashboard-state.js`
- `dashboard/src/lib/state/dashboard-store.js`
- `e2e/dashboard.modules.unit.test.js`
- `e2e/dashboard.smoke.spec.js`

## Objective Check

This tranche was meant to do one thing cleanly:

1. move the legacy subsystem-by-subsystem Monitoring surface into a truthful transitional home,
2. leave Monitoring as a genuinely clean slate for the later operator overhaul,
3. preserve the existing bounded monitoring data path rather than creating a second telemetry fork.

That objective is now met.

## What Landed Well

1. `Diagnostics` now owns the legacy monitoring surface with minimal semantic churn. The detailed charts, recent-event surface, raw-feed diagnostics, and contributor-oriented sections remain available without muddying the future Monitoring contract.
2. `Monitoring` is intentionally sparse and truthful. It no longer pretends that the legacy contributor surface is already the operator decision plane Shuma wants long-term.
3. The runtime split is clean. Diagnostics reuses the existing bounded monitoring refresh path, while Monitoring no longer owns auto-refresh or the legacy monitoring bootstrap flow.
4. The tab-state, hash-route, and auto-refresh contracts are now aligned with the new ownership model and are proven by rendered browser coverage.

## Review Findings

No new architectural shortfalls were found after implementation.

Two stale smoke-test assumptions were exposed during tranche-end full-suite verification:

1. one keyboard-navigation test still assumed `Advanced` was the last tab, and
2. one tab-state smoke test still tried to toggle auto-refresh from `Monitoring` instead of `Diagnostics`.

Those were corrected immediately and do not represent a runtime design shortfall.

## Existing Follow-On Work That Remains Valid

1. `MON-OVERHAUL-1` remains the next substantive task. The ownership split is complete, but the new Monitoring operator surface still needs its dedicated design discussion and implementation tranche.
2. `TEST-HYGIENE-2` remains valid. Full-suite verification still rewrites tracked SIM2 report artifacts, which required exact restoration after the pass. That is pre-existing test hygiene debt, not a regression introduced by this tranche.

## Evidence

- `make test-dashboard-unit`
- `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "monitoring tab is a clean-slate placeholder that points to diagnostics|dashboard clean-state renders explicit empty placeholders|auto refresh defaults off and is only available on diagnostics, ip-bans, and red-team tabs"'`
- `make test-dashboard-e2e PLAYWRIGHT_ARGS='--grep "tab keyboard navigation updates hash and selected state|tab states surface loading and data-ready transitions across all tabs"'`
- `make test`
- `git diff --check`
