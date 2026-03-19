Date: 2026-03-19
Status: Active follow-on implementation plan

Related context:

- [`../research/2026-03-19-controller-readiness-telemetry-foundation-review.md`](../research/2026-03-19-controller-readiness-telemetry-foundation-review.md)
- [`2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md`](./2026-03-18-monitoring-request-outcome-telemetry-hook-contract.md)
- [`2026-03-18-monitoring-operator-summary-exactness-contract.md`](./2026-03-18-monitoring-operator-summary-exactness-contract.md)
- [`2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](./2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md)
- [`2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](./2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs)
- [`../../src/observability/monitoring.rs`](../../src/observability/monitoring.rs)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Turn the 2026-03-19 controller-readiness review into the next concrete Stage 1 execution order.

This note is intentionally narrow. It does not reopen the broader telemetry architecture. It records the three remaining precision items that must now be treated as first-order foundation work before the Monitoring UI overhaul:

1. byte attribution by outcome,
2. richer bounded backend summary shapes,
3. and fuller terminal-path coverage.

# Why This Is The Next Work

The current telemetry foundation already gives Shuma:

1. one request-outcome object,
2. one live-versus-adversary-sim origin split,
3. one hot-read-safe scope summary,
4. and one hot-read-safe lane summary.

That is enough to keep moving forward, but not enough yet for benchmark-grade operator or controller inputs.

The next work must therefore improve truthfulness at the telemetry layer itself, not jump to Monitoring UI work. If Shuma redesigns Monitoring before these three items land, the UI will still be explaining an incomplete backend contract.

# Precision Item 1: Byte Attribution By Outcome

## Goal

Distinguish total bytes from outcome-class-attributed bytes so Shuma can later benchmark:

1. suspicious bytes that still reached origin,
2. locally served friction or block cost,
3. and control or fail-path bytes that should not be confused with either.

## Required contract

For each bounded request-outcome cohort that is already tracked, Shuma should retain:

1. total response bytes,
2. forwarded response bytes,
3. short-circuited response bytes,
4. control-response bytes.

This should exist at least for:

1. scope rows,
2. lane rows where a lane exists.

## Design rule

This must extend the existing request-outcome counter family rather than creating a second analytics path.

The outcome hook in [`src/runtime/request_outcome.rs`](../../src/runtime/request_outcome.rs) and [`src/observability/monitoring.rs`](../../src/observability/monitoring.rs) remains the source of truth.

## Execution order

This is the first code slice because later summary and benchmark work depends on it.

# Precision Item 2: Bounded Benchmark Summary Shapes

## Goal

Surface the semantics that Shuma already records but does not yet expose in `MonitoringSummary`:

1. `response_kind`,
2. `policy_source`,
3. `route_action_family`.

## Required contract

The summary surface must stay bounded and low-cardinality.

The preferred shape is compact grouped rows rather than ad hoc map blobs, so the backend contract remains stable and operator-friendly.

Each summary family must also carry the correct ownership tier:

1. bootstrap-critical only if needed for the primary Monitoring narrative,
2. supporting-summary if useful but not required for first paint,
3. never silently promoted from diagnostics into operator truth.

## Design rule

Do not wait for the Monitoring overhaul to invent these semantics in the UI.

First expose them as backend summaries, then let the Monitoring redesign consume that stable contract.

## Execution order

This is the second code slice, after byte attribution and before UI work.

# Precision Item 3: Terminal-Path Coverage Matrix

## Goal

Remove ambiguity about which terminal responses are covered by request-outcome telemetry, which are intentionally excluded, and why.

## Required contract

Every terminal path in [`src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs) should fall into one of three classes:

1. emits request-outcome telemetry today,
2. must be brought under the request-outcome hook,
3. remains intentionally excluded with explicit reason.

## Current desired disposition

| Terminal path | Current state | Desired state |
| --- | --- | --- |
| HTTPS-required rejection | early return | include as control-path outcome |
| early routes | early return | include as control-path outcome |
| static asset bypass | early return | remain excluded until a low-cost accounting path is chosen |
| store-open failure | early return | include as fail/control outcome |
| config-load failure | early return | include as fail/control outcome |
| handled forwarded allow | covered | keep covered |
| handled defence follow-up responses | covered | keep covered |
| handled sim-public responses | covered | keep covered with adversary-sim origin truth |

## Design rule

This is not primarily a UI requirement. It is a telemetry truth-boundary requirement for both operators and future bounded controllers.

## Execution order

This is the third code slice, unless a simple inclusion change naturally falls out of the byte-attribution work earlier.

# Recommended Implementation Order

1. Add outcome-attributed byte counters and summary fields.
2. Add bounded `response_kind`, `policy_source`, and `route_action_family` summary rows.
3. Close or explicitly codify the remaining terminal-path gaps.
4. Refresh focused verification and then conduct a post-implementation review before touching the Monitoring UI.

# Verification Strategy

During implementation:

1. use `make test-monitoring-telemetry-foundation-unit` as the primary surgical loop,
2. add narrower assertions at the boundary actually being changed,
3. only widen verification when a boundary change crosses runtime, hot-read, and dashboard surfaces.

Before claiming the pre-overhaul telemetry foundation tranche complete:

1. run `make test`,
2. refresh `.spin/last-full-test-pass.json`,
3. conduct a post-implementation review against this plan and the 2026-03-19 review addendum,
4. immediately convert any shortfalls into follow-on TODOs and execute them before moving to the Monitoring overhaul.
