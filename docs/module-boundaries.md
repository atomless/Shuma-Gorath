# Module Boundaries

This document defines the in-repo boundaries used to prepare future repo splits.

## Current Contract Layer

`src/boundaries/contracts.rs` is the explicit contract seam between request orchestration and domain modules.

- `ChallengeBoundary`
- `MazeBoundary`
- `AdminBoundary`

`src/boundaries/adapters.rs` provides default adapters that map those contracts to current modules:

- `crate::challenge`
- `crate::maze`
- `crate::admin`

`src/lib.rs` routes through `src/boundaries/mod.rs` instead of calling those domain modules directly.

## Target Split Direction

- Core policy/orchestration: `src/lib.rs` and `src/runtime/` helpers (`request_router`, `kv_gate`, `policy_pipeline`)
- Admin adapter domain: `src/admin/` (`api.rs` endpoint surface + `auth.rs` auth/session concerns)
- Config domain: `src/config/mod.rs` (+ `src/config/tests.rs`)
- Signal domain: `src/signals/` (browser/CDP/GEO/IP/JS/whitelist)
- Enforcement domain: `src/enforcement/` (ban/block/rate/honeypot)
- Maze/tarpit domain: `src/maze/` plus future tarpit implementation
- Challenge domain: `src/challenge/`
- Dashboard adapter: `dashboard/modules/` API/session/config adapters

## Defence Taxonomy (H3.6.1)

This taxonomy defines how modules participate in bot defense composition.

- `signal`: contributes evidence for scoring/risk decisions, no direct blocking by itself.
- `barrier`: enforces a defensive action (block/challenge/maze/tarpit/ban flow), may log metrics.
- `hybrid`: supports both signal contribution and direct enforcement, via explicit separate paths.

### Inventory

| Module Path | Class | Primary Role | Ownership | Dependency Direction |
| --- | --- | --- | --- | --- |
| `src/signals/geo/` | signal | Country extraction and GEO policy signal inputs | Signals | Can depend on config/input utils; must not depend on enforcement modules. |
| `src/signals/cdp/` | signal | Automation fingerprinting signal (CDP checks/reporting) | Signals | Can depend on config/shared types; direct enforcement calls are temporary and should migrate to policy/barrier orchestration. |
| `src/signals/js.rs` | signal | JS verification signal/challenge trigger inputs | Signals | Can depend on challenge/pow helpers for presentation, but should not own hard block decisions. |
| `src/signals/browser.rs` | signal | Browser capability/version signal | Signals | Independent utility signal source; no enforcement dependencies. |
| `src/signals/ip.rs` | signal | IP bucketing utility for telemetry/signal keys | Signals | Leaf utility; no enforcement dependencies. |
| `src/signals/whitelist/` | signal | Allow-list signal short-circuit inputs | Signals | Can depend on parsing/input modules; no enforcement dependencies. |
| `src/enforcement/honeypot.rs` | barrier | Honeypot path detection for immediate defensive action | Enforcement | May consume routing/config context; should not calculate botness directly. |
| `src/enforcement/ban/` | barrier | Ban persistence and ban-state enforcement primitives | Enforcement | May depend on storage/input sanitation; no direct dependence on signal module internals. |
| `src/enforcement/block_page.rs` | barrier | Block response rendering | Enforcement | Presentation-only enforcement utility. |
| `src/maze/` | barrier | Deception/maze barrier for suspicious traffic | Maze/Tarpit domain | Consumes policy decisions; should expose behavior via boundary adapters. |
| `src/challenge/` | barrier | Interactive challenge barrier flow | Challenge domain | Consumes policy decisions; should expose behavior via boundary adapters. |
| `src/enforcement/rate.rs` | hybrid | Rate telemetry + absolute limit enforcement | Enforcement | Must keep separate signal and enforcement paths (`signal` vs hard cap enforcement). |
| `src/pow.rs` | hybrid (candidate) | Cost-imposition challenge path + verification signal potential | Core policy/challenge integration | Current behavior is barrier-focused; future signal contribution should be explicit via normalized signal contract. |

### Dependency Rules for Composition

- Signal modules write evidence; they do not decide final blocking outcomes.
- Barrier modules consume policy outcomes; they do not silently mutate botness scoring internals.
- Hybrid modules MUST provide explicit split APIs for signal contribution and enforcement action.
- Runtime orchestration (`src/runtime/policy_pipeline.rs`) remains the ordering authority.

## Rules For New Work

- Keep `src/lib.rs` focused on orchestration and call domains via boundary adapters.
- Add new cross-domain behavior by extending a boundary contract first.
- Avoid adding direct `crate::<domain>` calls in `src/lib.rs` when a boundary exists.
- Keep boundary contracts behavior-focused and minimal to reduce split risk.
