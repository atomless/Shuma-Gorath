# Blocked TODO Roadmap

Last updated: 2026-03-14

This file holds gated, contingent, or explicitly deferred work that is not execution-ready.
Move an item back into `todos/todo.md` only when its blocking condition is cleared.
Completed work lives in `todos/completed-todo-history.md`.
Security finding validity and closure status live in `todos/security-review.md`.

## P0 Blocked by Telemetry Live-Evidence Deployment Gate

- [ ] TEL-EVT-1-5 Extend live telemetry evidence to capture representative persisted-row bytes, recent-events-tail document bytes, and bootstrap payload bytes, and prove the compact event contract improves storage/payload weight while preserving analysis and dashboard usability; treat any regression in the current `TEL-HOT` live budget envelope as tranche-blocking and treat failure to achieve a material challenge-heavy sample size reduction as a review gate.
  Blocker: wait until the compact-event telemetry changes are deployed to the shared-host and Fermyon evidence targets, then re-run `make test-telemetry-hot-read-live-evidence` against the updated deployments so the measured row/document/payload sizes reflect the new schema rather than pre-change receipts.

- [ ] TEL-EVT-1-6 Once `TEL-EVT-1` lands with live size evidence, write the follow-on retention/lifecycle plan and active TODO tranche that re-evaluates raw event, summarized hot-read, and rollup retention windows in light of the new compact schema, preserving automatic purge/default-on lifecycle governance.
  Blocker: keep blocked until `TEL-EVT-1-5` produces live post-deploy size evidence for the compact schema; the retention/lifecycle reassessment must be based on the measured deployed footprint, not pre-deploy estimates.

## P0 Blocked by Shared-Host Discovery and Runtime-Safety Gates

- [ ] SIM-SCR-LANE-1 Three-lane runtime migration and Scrapling integration.
  Blocker: do not start until `SIM-SH-SURFACE-1` is complete with real shared-host evidence per [`docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../docs/plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md).

- [ ] SIM-LLM-1 Full LLM-orchestrated, instruction-driven, containerized adversary lane as a first-class runtime actor.
  Blocker: keep blocked until shared-host discovery is complete, the runtime lane model is proven safe, and the active `SIM-DEPLOY-2` operating-envelope tranche establishes acceptable production bounds.

## P1 Blocked by Roadmap Reprioritisation After Deployment Baseline

- [ ] SIM-BREACH-REPLAY-1 External breach to replayable attack pipeline.
  Blocker: defer until shared-host deployment readiness and discovery baseline are complete, then re-assess capture, retention, and replay governance against the deployed operating model.

## P1 Blocked by Enterprise Baseline Maturity

- [ ] DEP-ENT-6 Optional asynchronous mirror of high-confidence bans to Akamai Network Lists.
  Blocker: wait until `DEP-ENT-1..5` establish the authoritative enterprise distributed-state baseline.

- [ ] OUT-4 ADR for non-Redis external integrations (for example webhook notifications or cross-service sync) that defines the approved pattern in Spin (`allowed_outbound_hosts` expansion vs sidecar/bridge service).
  Blocker: wait until a concrete non-Redis integration target is approved.

- [ ] OUT-5 External transport design for non-stub `challenge_engine=external` and `maze_tarpit=external`.
  Blocker: wait until there is an approved external provider path after the baseline deployment work is complete.
