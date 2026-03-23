# Monitoring And Diagnostics Tab Ownership Plan

Date: 2026-03-20

## Context

The backend telemetry foundation is now complete enough for the Monitoring overhaul to begin, but the current `Monitoring` tab is still the legacy subsystem-by-subsystem diagnostic surface. That legacy surface is valuable during the transition, but it is the wrong starting point for the operator decision surface Shuma needs next.

The existing research and telemetry plans already establish the target direction:

- `docs/research/2026-03-17-operator-decision-support-telemetry-audit.md`
- `docs/research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`
- `docs/research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`
- `docs/research/2026-03-19-pre-monitoring-overhaul-telemetry-foundation-closeout-review.md`

This note settles the immediate UI ownership question before `MON-OVERHAUL-1` begins.

## Decision

Shuma will split the current Monitoring surface into two distinct tab contracts:

1. `Monitoring`
   - Reserved for the new operator decision surface.
   - Starts as a clean, intentionally minimal placeholder rather than a hybrid with legacy diagnostics.
   - Must not inherit the current subsystem-by-subsystem layout.

2. `Diagnostics`
   - New tab placed after `Advanced`.
   - Receives the current Monitoring implementation largely intact as the transitional diagnostic surface.
   - Keeps contributor and deep-inspection value available while Monitoring is rebuilt properly.

As of 2026-03-23, the target is now more specific: Monitoring should become the human-readable accountability surface for the closed loop rather than a manual tuning cockpit, and Diagnostics should become more intentionally diagnostics-first rather than merely preserving the old Monitoring layout elsewhere.

## Ownership Contract

### Monitoring

`Monitoring` now owns only the future operator-facing contract:

- traffic mix and lane summaries,
- attacker-effectiveness versus human-friction visibility,
- loop verdict and benchmark status,
- change judgment and watch-window result,
- category-aware non-human outcome breakdowns,
- trust and actionability blockers,
- enforced versus shadow storytelling,
- bounded benchmark-grade summaries that later tuning and oversight loops can trust.

Until that overhaul lands, the tab should remain intentionally sparse and truthful about its status.

### Diagnostics

`Diagnostics` owns the legacy detailed surface during the transition:

- subsystem trend blocks,
- detailed recent events filtering,
- CDP, maze, tarpit, honeypot, challenge, PoW, rate, GEO, and IP-range detail sections,
- raw feed and low-level telemetry diagnostics,
- Prometheus helper and contributor-oriented deep inspection.

As the Monitoring overhaul progresses, Diagnostics should become more intentionally diagnostics-oriented in wording and sectioning rather than remaining a merely transplanted Monitoring page.

The current Monitoring implementation should move here with minimal behavioral change.

### Status

`Status` remains the operator health and control-plane truth surface:

- dashboard connectivity,
- telemetry delivery health,
- retention health,
- runtime performance telemetry.

It should not absorb the full legacy Monitoring surface.

### IP Bans

`IP Bans` remains the active-ban and ban-operations surface. It should continue to avoid low-level diagnostics in the main body.

## Refresh And Runtime Contract

During the transition:

- `Diagnostics` keeps the bounded monitoring refresh path but exposes manual refresh only.
- auto-refresh remains available only on `IP Bans` and `Red Team`.
- `Monitoring` no longer needs auto-refresh for the placeholder stage.
- the backend monitoring data source remains shared; the split is surface ownership, not a telemetry fork.
- dashboard request/runtime telemetry should describe `Diagnostics` truthfully where the legacy Monitoring surface now lives.

## Why This Is Better Than Wiping

- It preserves valuable diagnostic depth while the new Monitoring surface is designed carefully.
- It avoids a messy hybrid tab that mixes operator outcomes with contributor mechanics.
- It gives the Monitoring overhaul a genuinely clean slate.
- It keeps the section-ownership boundary explicit for later roadmap work and future test cleanup.

## Implementation Notes

1. Move the current `MonitoringTab.svelte` implementation to `DiagnosticsTab.svelte`.
2. Add a new minimal `MonitoringTab.svelte` placeholder using existing shared dashboard styles only.
3. Insert `Diagnostics` into the canonical tab registry after `Advanced`.
4. Route Diagnostics refresh through the existing bounded monitoring snapshot/delta flow.
5. Update docs and tests so the legacy diagnostic expectations point at `Diagnostics`, not `Monitoring`.

## Out Of Scope

- The substantive Monitoring operator redesign itself.
- New telemetry summaries or dashboard semantics beyond the ownership split.
- Reworking the legacy diagnostics surface beyond light copy cleanup needed for truthfulness.
