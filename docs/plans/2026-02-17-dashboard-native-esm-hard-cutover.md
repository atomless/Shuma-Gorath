# Dashboard Native ESM Hard Cutover Decision

Date: 2026-02-17  
Scope: `dashboard/` JavaScript architecture modernization (`DSH-ESM-*`)  
Decision owner: project maintainers

## Decision

Use a **hard cutover** to native browser ESM for dashboard JavaScript.

- No temporary dual wiring (`window.ShumaDashboard*` + ESM in parallel).
- No compatibility shims retained only for pre-cutover globals.
- No build step introduced.

This is appropriate because the project is pre-launch and does not require backwards compatibility for legacy dashboard script loading.

## Why Hard Cutover

1. Reduces architectural complexity and maintenance overhead.
2. Avoids long-lived split-brain module contracts.
3. Forces one canonical dependency graph and import surface.
4. Aligns with modern browser capability baseline for target environments.

## Safety Constraints

1. Preserve existing dashboard behavior contracts:
   - hash-routed tabs (`#monitoring`, `#ip-bans`, `#status`, `#config`, `#tuning`)
   - config dirty-state/save semantics
   - monitoring card/table/chart rendering semantics
   - admin session/login/logout flow
2. Keep frameworkless, functional style (no class-based UI architecture for feature modules).
3. Keep runtime no-build operation (static served JS modules).
4. Land in small slices, each passing canonical verification (`make test` with Spin running).

## Migration Shape

1. `DSH-ESM-2`: lock behavior contracts and regression checklist.
2. `DSH-ESM-3`: extend tests before interface changes.
3. `DSH-ESM-4..6`: convert entrypoint + modules to ESM import graph.
4. `DSH-ESM-7..9`: tighten functional architecture boundaries.
5. `DSH-ESM-10..12`: enforce style and static guardrails.
6. `DSH-ESM-13..15`: full verification, docs, and final behavior audit.

## Supporting Contracts

- Behavior contract freeze: `docs/plans/2026-02-17-dashboard-esm-behavior-contracts.md`
- Module graph/layer contract: `docs/plans/2026-02-17-dashboard-esm-module-graph.md`
- Final no-net-behavior audit: `docs/plans/2026-02-17-dashboard-esm-no-net-behavior-audit.md`

## Non-Goals

1. No frontend framework migration in this cutover.
2. No build tooling introduction (bundlers/transpilers).
3. No behavior redesign of dashboard features as part of the ESM migration itself.
