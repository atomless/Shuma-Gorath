# 🐙 Testing Guide

## 🐙 Quick Commands (Official)

```bash
make test             # Full umbrella suite: unit + maze benchmark + Spin integration + mandatory fast adversarial matrix + dashboard e2e
make test-unit        # Unit tests only (native Rust)
make unit-test        # alias for make test-unit
make test-maze-benchmark # Deterministic maze asymmetry benchmark gate
make test-integration # Integration tests only (waits for existing Spin readiness)
make integration-test # alias for make test-integration
make test-gateway-harness # Gateway fixture/failure harness + deploy guardrail parser tests
make test-gateway-wasm-tls-harness # wasm32 TLS cert-failure matrix (expired/self-signed/hostname-mismatch; external egress required)
make test-gateway-origin-bypass-probe # Optional active direct-origin bypass probe (requires URL args)
make test-gateway-profile-shared-server # Shared-server gateway contract + forwarding checks
make test-gateway-profile-edge # Edge/Fermyon gateway contract + signed-header origin-auth checks
make smoke-gateway-mode # Fast gateway smoke (allow forward, enforcement-local, fail-closed outage)
make test-adversarial-manifest # Validate adversarial scenario manifest + fixture references
make test-adversarial-coverage-contract # Validate canonical full_coverage contract parity (plan + manifests + runner)
make test-adversarial-fast # Mandatory fast adversarial matrix (smoke + abuse + Akamai)
make test-adversarial-preflight # Validate required adversarial secrets/setup and ensure browser-lane Chromium readiness
make test-adversarial-smoke # Mandatory adversarial fast smoke profile (waits for existing Spin readiness)
make test-adversarial-abuse # Replay/stale/order-cadence abuse regression profile
make test-adversarial-akamai # Akamai fixture-driven simulation profile
make test-adversarial-coverage # Expanded adversarial coverage profile (pre-release gate)
make test-adversarial-sim-selftest # Minimal deterministic simulator mechanics self-test (no Spin server required)
make test-adversarial-soak # Deep adversarial soak gate alias for full_coverage
make test-adversarial-live # Loop adversarial profile for live monitoring demos (Ctrl+C to stop)
make test-adversarial-repeatability # Deterministic drift gate across smoke/abuse/coverage (N=3)
make test-adversarial-promote-candidates # Frontier finding triage + deterministic replay/promotion checks
make test-adversarial-container-isolation # Validate black-box container isolation contract (Docker required)
make test-adversarial-container-blackbox # Run containerized black-box adversary worker (Docker required)
make test-adversarial-frontier-attempt # Protected-lane frontier provider attempt probe (advisory/non-blocking)
make test-frontier-governance # Frontier artifact guard (forbidden keys + secret leak checks)
make test-frontier-unavailability-policy # Frontier degraded-threshold policy tracker + actionability artifact
make test-sim2-operational-regressions # SIM2 operational regressions for active deterministic profiles (retention/cost/security required; failure/prod checked when present)
make test-sim2-operational-regressions-strict # Strict SIM2 operational regressions (all failure/prod/retention/cost/security domains required)
make test-sim2-governance-contract # SIM2 hybrid lane + governance contract conformance diagnostics
make test-ip-range-suggestions # Focused IP-range suggestion regression gate (runtime + dashboard)
make test-coverage    # Unit coverage to lcov.info (requires cargo-llvm-cov)
make test-dashboard-unit # Dashboard module unit tests (Node `node:test`)
make test-dashboard-budgets # Dashboard /_app bundle-size ceilings report (warn-only by default)
make test-dashboard-budgets-strict # Dashboard /_app bundle-size ceilings (hard-fail)
make test-dashboard-e2e # Playwright dashboard smoke tests (waits for existing Spin readiness)
make seed-dashboard-data # Seed local dashboard sample records against running Spin
make test-dashboard   # Manual dashboard checklist
```

Notes:
- Use Makefile commands only (avoid running scripts directly)
- Integration tests require a running Spin server; targeted integration-only commands can run against `make dev` or `make dev-prod`, but the full umbrella `make test` contract requires `make dev` (`runtime-dev`).
- `make test`, `make test-integration`, and `make test-dashboard-e2e` wait for `/health` readiness before failing.
- `make test` now also checks `/admin/session` and fails fast if the running server is `runtime-prod`, because the full adversarial/dashboard contract is defined against `make dev`.
- `make test` includes maze asymmetry benchmark gating, the mandatory fast adversarial matrix (`smoke + abuse + Akamai`), SIM2 realtime gates, and Playwright dashboard e2e. If Docker is unavailable, the container black-box lane degrades to the advisory SIM2 verification matrix path instead of hard-failing the umbrella run.
- The container black-box runner chooses its own Docker reachability mode for loopback-hosted Spin instances: bridge + `host.docker.internal` on non-Linux hosts, and host-network mode on Linux when the target base URL is loopback-only. This keeps `make dev` bound to `127.0.0.1` while preserving container reachability in CI.
- Gateway profile gates are explicit and runnable independently:
  - `make test-gateway-profile-shared-server`
  - `make test-gateway-profile-edge`
  - `make smoke-gateway-mode`
- Gateway follow-on hardening gates:
  - `make test-gateway-wasm-tls-harness` runs a real wasm outbound TLS-failure matrix and requires external outbound HTTPS reachability.
  - `make test-gateway-origin-bypass-probe` is optional/operator-run and requires `GATEWAY_PROBE_GATEWAY_URL` + `GATEWAY_PROBE_ORIGIN_URL`.
    - optional strict mode: set `GATEWAY_PROBE_FAIL_ON_INCONCLUSIVE=1`.
- `make test-sim2-operational-regressions` enforces retention/cost/security domains and treats `failure_injection` + `prod_mode_monitoring` as optional when absent from the active deterministic profile artifact. Use `make test-sim2-operational-regressions-strict` when you need full-domain enforcement.
- `make test` keeps the fast adversarial matrix in the routine local/full-suite path and runs the SIM2 matrix in advisory mode against the resulting fast-profile artifact.
- `make test-adversarial-coverage` and `make test-adversarial-soak` remain the strict deterministic `full_coverage` oracle paths for deeper protected-lane verification.
- `make test-dashboard-e2e` now verifies the running Spin instance is serving the current `dist/dashboard/index.html` before Playwright runs; restart Spin after `make dashboard-build` if this check fails.
- `make test` now reseeds dashboard sample data at the end, so charts/tables stay populated for local inspection after the run.

## 🐙 Test Layers

This project uses six distinct test environments, each optimized for its scope:

1. Unit tests (native Rust)
2. Integration tests (Spin environment)
3. Adversarial simulation profiles (Spin environment + manifest-driven runner)
4. Dashboard module unit tests (Node `node:test`)
5. Dashboard e2e smoke tests (Playwright)
6. Dashboard checks (manual)

## 🐙 Test Layout Conventions

Rust test layout is now standardized as follows:

- Unit tests should live with the owning module, wired via `#[cfg(test)] mod tests;`.
- Module-specific test files should be placed under that module directory (for example `src/ban/tests.rs` or `src/allowlist/path_tests.rs`).
- Shared unit-test utilities belong in `src/test_support.rs` (request builders, env lock, in-memory <abbr title="Key-Value">KV</abbr> store fixtures).
- New black-box integration tests should be added in `tests/` when they can rely on public interfaces only.
- Cross-module crate-internal suites should live under `src/lib_tests/`.

## 🐙 Why Two Environments

Unit tests run natively in Rust and validate logic in isolation.
Integration tests must run in Spin because they require the <abbr title="Hypertext Transfer Protocol">HTTP</abbr> server, routing, Spin <abbr title="Key-Value">KV</abbr> storage, and real request headers.

## 🐙 Unit Tests (Native Rust)

Run with:

```bash
make test-unit
```

Unit tests validate core logic in isolation (ban logic, allowlist parsing, config defaults, <abbr title="Chrome DevTools Protocol">CDP</abbr> parsing, etc.).
Test counts may change as coverage evolves; rely on `make test-unit` output for the current total.
Coverage includes ban/unban flows, allowlists, maze generation, challenge logic, <abbr title="Chrome DevTools Protocol">CDP</abbr> parsing, and helper utilities.

## 🐙 Integration Tests (Spin Environment)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-integration
```

These tests exercise the full <abbr title="Hypertext Transfer Protocol">HTTP</abbr> + <abbr title="Key-Value">KV</abbr> runtime and are required for end-to-end validation.
If your Spin environment sets `SHUMA_FORWARDED_IP_SECRET`, export it before running integration tests so the curl requests include the matching `X-Shuma-Forwarded-Secret` header:

```bash
export SHUMA_FORWARDED_IP_SECRET="your-forwarded-ip-secret"
```

If you configured `SHUMA_HEALTH_SECRET`, export it too so health checks include `X-Shuma-Health-Secret`:

```bash
export SHUMA_HEALTH_SECRET="your-health-secret"
```

The integration suite is implemented in `scripts/tests/integration.sh` and is invoked by `make test-integration`.

Integration coverage includes:
1. Health endpoint and <abbr title="Key-Value">KV</abbr> availability
2. Root endpoint behavior (block page / <abbr title="JavaScript">JS</abbr> challenge)
3. Honeypot ban flow
4. Admin config + test-mode toggling
5. Challenge single-use behavior (`Incorrect` then replay `Expired`)
6. Metrics endpoint
7. <abbr title="Chrome DevTools Protocol">CDP</abbr> report ingestion and auto-ban flow
8. <abbr title="Chrome DevTools Protocol">CDP</abbr> stats counters in `/admin/cdp`
9. Monitoring summary endpoint in `/admin/monitoring`
10. Unban behavior

## 🐙 Adversarial Simulation Profiles (Manifest-Driven)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-adversarial-smoke
```

Available profiles:
- `make test-adversarial-fast` - mandatory fast matrix (`smoke + abuse + Akamai`)
- `make test-adversary-sim-runtime-surface` - runtime-toggle integration gate that verifies required deterministic defense-surface telemetry categories (challenge/JS/PoW/rate/GEO/maze-tarpit/fingerprint-CDP/ban) on a running server
- `make test-adversarial-smoke` - mandatory fast smoke gate (`SIM-T0`..`SIM-T4`)
- `make test-adversarial-abuse` - mandatory replay/stale/order-cadence abuse regressions
- `make test-adversarial-akamai` - mandatory Akamai signal fixture coverage
- `make test-adversarial-coverage` - expanded coverage contract profile (`full_coverage`) including PoW success/failure, puzzle-failure fallback, replay-to-tarpit bootstrap abuse, CDP deny path, rate-limit enforcement, and GEO block coverage
  - includes defense no-op detector checks (`coverage_gates.defense_noop_checks`) that fail when targeted defenses emit zero telemetry deltas
- PR CI and release-gate workflows use this target for strict deterministic coverage proof; routine `make test` remains on the fast/advisory path.
- `make test-adversarial-sim-selftest` - minimal deterministic simulator mechanics harness (seed/order/budget/retry/gate math/teardown), intentionally non-circular
- `make test-adversarial-soak` - deep soak alias for `full_coverage` (scheduled/manual gate)
- `make test-adversarial-manifest` - schema/fixture validation without server
- `make test-adversarial-lane-contract` - black-box attacker/control capability contract parity check across deterministic/container tooling
- `make test-adversarial-sim-tag-contract` - signed simulation-tag contract parity check across lane contract, runner, and container worker
- `make test-adversarial-coverage-contract` - canonical `full_coverage` contract parity check across SIM2 plan rows, manifests, and runner enforcement
- `make test-adversarial-live` - repeated live traffic generator for operator monitoring drills
- `make test-adversarial-repeatability` - deterministic replay consistency gate across `fast_smoke`, `abuse_regression`, and `full_coverage`
- `make test-adversarial-promote-candidates` - frontier finding normalization + deterministic replay triage + promotion lineage report
- `make test-adversarial-container-isolation` - container self-check gate for mount/env/identity/tooling hardening contract
- `make test-adversarial-container-blackbox` - containerized black-box worker run (separate complementary lane)
- `make test-adversarial-frontier-attempt` - protected-lane frontier provider probe attempt (advisory, non-blocking)
- `make test-frontier-governance` - fail-fast guard for forbidden frontier artifact fields and secret leaks
- `make test-frontier-unavailability-policy` - degraded-threshold policy evaluation and refresh-action artifact

Simulation realism pages are available at `/sim/public/landing`, `/sim/public/docs`, `/sim/public/pricing`, `/sim/public/contact`, and `/sim/public/search?q=...` only when both availability gates are true: `SHUMA_ADVERSARY_SIM_AVAILABLE=true` and KV `adversary_sim_enabled=true`.
Dashboard DOM-class contract for runtime/simulation affordances:
- `<html>` must include exactly one runtime environment class: `runtime-dev` or `runtime-prod` (derived from trusted runtime config).
- `<html>` connection state classes are heartbeat-owned: runtime boots in `disconnected`, flips to `connected` after successful heartbeat, enters `degraded` on heartbeat failures, and transitions to `disconnected` after configured hysteresis threshold (`N`) of consecutive heartbeat failures.
- `<body>` must include `adversary-sim` only when `adversary_sim_enabled=true`.
- These classes are presentational hooks only and must not alter defence/auth behavior directly.

Dashboard adversary-sim orchestration control contract:
- `POST /admin/adversary-sim/control` is the explicit admin-authenticated + CSRF-protected control path for ON/OFF transitions.
- Control submissions must include `Idempotency-Key`, pass strict origin/referer + fetch-metadata trust checks, and return `operation_id` + `decision`.
- `GET /admin/adversary-sim/status` is the operator/dashboard read path and returns lifecycle phase, fixed guardrails, desired/actual state, and controller reconciliation/lease metadata. Current implementation may also reconcile stale persisted state on read; strict non-mutating status remains an open contract cleanup item.
- `POST /internal/adversary-sim/beat` is an internal-only endpoint used by host-side supervisor workers; dashboard clients never call it directly.
- Host-side supervisor requests must satisfy trusted-forwarding (`X-Shuma-Forwarded-Secret`, loopback `X-Forwarded-For`, `X-Forwarded-Proto: https`) and send the internal supervisor marker header. Only `/admin/adversary-sim/status` and `/internal/adversary-sim/beat` bypass the public admin IP allowlist under that internal supervisor contract.
- Runtime generation cadence ownership is backend/supervisor-only: dashboard refresh cadence must not control traffic generation.
- Toggle-driven runs use `adversary_sim_duration_seconds` (default `180`, hard-bounded `30..900`) under backend autonomous heartbeat generation, and dashboard surfaces lifecycle state only (`off`, `running`, `stopping`) without procedural progress rendering.
- If no frontier provider keys are configured, OFF -> ON toggle attempts must show a warning dialog with two outcomes:
  - continue without frontier calls, or
  - cancel, add `SHUMA_FRONTIER_*_API_KEY` values to `.env.local`, restart `make dev`, then toggle on again.
- Runtime guardrails are hard-coded: `max_concurrent_runs=1`, `cpu_cap_millicores=1000`, `memory_cap_mib=512`, `queue_policy=reject_new`.
- Lifecycle split is explicit: `generation_active` controls producer state, while retained telemetry visibility is independent (`historical_data_visible=true` until retention expiry or explicit cleanup).

Host-side supervisor launch adapters:
- Local development (`make dev`, `make dev-prod`, `make run`, `make run-prebuilt`, `make prod`) wraps `spin up` with `scripts/run_with_adversary_sim_supervisor.sh`.
- Build/run helper targets:
  - `make adversary-sim-supervisor-build`
  - `make adversary-sim-supervisor`
- Single-host/systemd style deployment should use the same wrapper/runtime contract as `make prod-start`: launch `scripts/run_with_adversary_sim_supervisor.sh` around `spin up`, with `SHUMA_API_KEY` injected via service env/secret manager. That wrapper manages the `target/tools/adversary_sim_supervisor` worker and polls `GET /admin/adversary-sim/status` before sending `POST /internal/adversary-sim/beat`.
- Containerized deployment can run the same worker as a sidecar process sharing network reachability to the Shuma instance.
- Edge/no-local-process environments can run an external supervisor service that calls the same internal beat endpoint.

Live loop examples:

```bash
# Infinite fast-smoke loop until Ctrl+C
make test-adversarial-live

# Five abuse cycles with a 1-second pause between cycles
ADVERSARIAL_PROFILE=abuse_regression ADVERSARIAL_RUNS=5 ADVERSARIAL_PAUSE_SECONDS=1 make test-adversarial-live

# Akamai fixture profile with custom report output
ADVERSARIAL_PROFILE=akamai_smoke ADVERSARIAL_REPORT_PATH=scripts/tests/adversarial/live_akamai_report.json make test-adversarial-live

# Full coverage profile loop (bounded runtime is defined in manifest)
ADVERSARIAL_PROFILE=full_coverage ADVERSARIAL_RUNS=1 make test-adversarial-live

# Explicitly clear retained telemetry history (shared local keyspace; destructive)
make telemetry-clean
```

Live loop controls:
- `ADVERSARIAL_PROFILE` (default `fast_smoke`) must be one of `fast_smoke`, `abuse_regression`, `akamai_smoke`, `full_coverage`.
- `ADVERSARIAL_RUNS` (default `0`) controls cycle count; `0` means run until interrupted.
- `ADVERSARIAL_PAUSE_SECONDS` (default `2`) controls delay between cycles.
- `ADVERSARIAL_REPORT_PATH` (default `scripts/tests/adversarial/latest_report.json`) controls report output file.
- `ADVERSARIAL_CLEANUP_MODE` (default `0`) toggles preserve-vs-cleanup behavior per cycle:
  - `0`: preserve state by default for live observability loops.
  - `1`: force deterministic cleanup after each cycle.
- When cleanup mode is active (`SHUMA_ADVERSARIAL_PRESERVE_STATE=0`), the runner clears both ban state and retained telemetry history through `POST /admin/adversary-sim/history/cleanup` before and after the run.
- Resilience controls:
  - `ADVERSARIAL_FATAL_CYCLE_LIMIT` (default `3`) stops the loop only after N consecutive fatal cycles.
  - `ADVERSARIAL_TRANSIENT_RETRY_LIMIT` (default `4`) retries transient failures before converting to one fatal cycle.
  - `ADVERSARIAL_BACKOFF_BASE_SECONDS` / `ADVERSARIAL_BACKOFF_MAX_SECONDS` bound transient retry backoff.
- Live loop logs now include per-cycle failure classification (`transient` vs `fatal`), retry count, backoff, and terminal failure reason when exiting.
- Live loop enforces event-quality checks; admin-only noise is treated as a fatal cycle and logs a clear reason.
- Runner also emits `scripts/tests/adversarial/attack_plan.json` with frontier mode/provider metadata and sanitized candidate payloads.
- Promotion lane emits `scripts/tests/adversarial/promotion_candidates_report.json` with candidate -> replay -> promotion lineage and owner-review requirements.
- Frontier threshold lane emits `scripts/tests/adversarial/frontier_unavailability_policy.json` and can auto-open/assign model-refresh action when protected-lane degradation thresholds are exceeded.
  - If repository Issues are disabled, it must remain artifact-only and report that status in the output summary instead of failing the lane.
- Browser-realistic lane executes through Playwright (`scripts/tests/adversarial_browser_driver.mjs`) instead of HTTP emulation.
  - Browser runner controls:
  - `SHUMA_ADVERSARIAL_BROWSER_DRIVER_ENABLED` (default `true`) toggles browser-driver execution.
  - `SHUMA_ADVERSARIAL_BROWSER_RETRIES` (default `2`, clamped `1..3`) controls retry attempts for transient browser launch/network failures.
  - `SHUMA_ADVERSARIAL_BROWSER_TIMEOUT_MS` (default `15000`) bounds per-scenario browser execution timeout.
  - `SHUMA_ADVERSARIAL_BROWSER_SETTLE_MS` (default `200`) controls post-navigation settle delay.
  - Browser evidence is attached per scenario (`browser_js_executed`, `browser_dom_events`, `browser_storage_mode`, `browser_challenge_dom_path`, request-lineage correlation IDs) and enforced via `browser_execution_gates`.
- Deterministic and container black-box runners now stamp attacker-plane traffic with signed simulation headers:
  - `X-Shuma-Sim-Run-Id`
  - `X-Shuma-Sim-Profile`
  - `X-Shuma-Sim-Lane`
  - `X-Shuma-Sim-Ts`
  - `X-Shuma-Sim-Nonce`
  - `X-Shuma-Sim-Signature`
  The canonical lane contract is versioned in `scripts/tests/adversarial/lane_contract.v1.json`.
  The signing contract is versioned in `scripts/tests/adversarial/sim_tag_contract.v1.json`.
  Attacker-plane requests must not include privileged headers (including `X-Shuma-Forwarded-Secret`).
  Runtime tagging is accepted only when `SHUMA_ADVERSARY_SIM_AVAILABLE=true` and signature/timestamp/nonce verification succeeds under `SHUMA_SIM_TELEMETRY_SECRET`.
  Container black-box workers receive bounded pre-signed sim-tag envelopes from the host runner (no runtime signing secret is injected into the container).
- `latest_report.json` includes quantitative `gates` and separate `coverage_gates` sections with per-check `threshold_source`.
- `latest_report.json` `coverage_gates` section includes `defense_noop_checks` for defense-level telemetry presence validation in `full_coverage`.
- `latest_report.json` also includes `cohort_metrics` (persona-level collateral/latency summaries) and `ip_range_suggestions` seed evidence for `full_coverage`.
- `latest_report.json` includes `realism_metrics` and `realism_gates` proving runtime execution behavior for traffic-model pacing, retry envelopes, and state-mode handling (`stateless`, `stateful_cookie_jar`, `cookie_reset_each_request`).
- `latest_report.json` includes `plane_contract` guardrail metadata confirming attacker/control-plane separation checks are enforced.
- `latest_report.json` includes `coverage_contract` metadata (schema/version/hash + canonical category keys) for coverage-audit traceability.

`make test` runs `test-adversarial-fast` (which executes `test-adversarial-smoke`, `test-adversarial-abuse`, and `test-adversarial-akamai`) in sequence.
`make test-adversarial-soak` runs `test-adversarial-coverage` (`full_coverage`) for deeper scheduled/manual validation.
`test-adversarial-fast` enforces `test-adversarial-lane-contract`, `test-adversarial-sim-tag-contract`, and `test-adversarial-coverage-contract` before running profile lanes.
`test-adversarial-coverage` enforces `test-adversarial-sim-tag-contract`, `test-adversarial-coverage-contract`, and `test-frontier-governance` after artifact generation.
`test-adversarial-coverage` forces deterministic cleanup plus per-run scenario-IP rotation (`SHUMA_ADVERSARIAL_PRESERVE_STATE=0`, `SHUMA_ADVERSARIAL_ROTATE_IPS=1`) to avoid stale local cadence/persistence collisions.
Monitoring tab now includes explicit tarpit progression telemetry (activations, progression outcomes, budget fallbacks, escalation outcomes, and top active bucket) sourced from `/admin/monitoring`.
Current `full_coverage` proves tarpit bootstrap entry and event-stream minimums, but it does not yet claim advanced tarpit progress-walker telemetry; reintroduce strict `tarpit_progress_advanced` depth gates only alongside a dedicated progress-following scenario.
Container black-box controls:
- worker image path: `scripts/tests/adversarial_container/Dockerfile` (non-root user, no workspace mount, read-only rootfs at runtime)
- runtime guardrails: dropped capabilities + `no-new-privileges` + bounded CPU/memory/pids + tmpfs `/tmp`
- isolation report: `scripts/tests/adversarial/container_isolation_report.json`
- black-box run report: `scripts/tests/adversarial/container_blackbox_report.json`
Repeatability controls:
- default repeats: `ADVERSARIAL_REPEATABILITY_REPEATS=3`
- default profile set: `ADVERSARIAL_REPEATABILITY_PROFILES=fast_smoke,abuse_regression,full_coverage`
- summary report: `scripts/tests/adversarial/repeatability_report.json`
- drift policy: scenario pass/outcome vectors must match exactly; latency variance is bounded by `ADVERSARIAL_REPEATABILITY_LATENCY_TOLERANCE_MS` (default `250`).
CI policy is tiered:
- Push to `main`: `ci.yml` runs `make test` plus gateway profile gates (`make test-gateway-profile-shared-server`, `make test-gateway-profile-edge`, `make smoke-gateway-mode`).
- PR to `main`: `ci.yml` runs `make test`, then `make test-adversarial-coverage`, `make test-adversarial-frontier-attempt`, and `make test-adversarial-promote-candidates`.
- Release gate (`release-gate.yml`): blocks on gateway profile gates, `make test-adversarial-coverage`, and deterministic confirmed-regression triage (`make test-adversarial-promote-candidates`), and records `make test-adversarial-frontier-attempt` as advisory status.
- Scheduled/manual deep soak: `adversarial-soak.yml` runs `make test-adversarial-soak`, `make test-adversarial-container-isolation`, and `make test-adversarial-container-blackbox`.
Deterministic/container coexistence contract:
- Deterministic lanes remain canonical mandatory blockers until explicit parity sign-off is approved (`SIM-V2-15` policy).
- Containerized lanes remain complementary scheduled/manual coverage in this phase and must not silently replace deterministic protected-lane gates.
- Parity-signoff governance is tracked via ADR + checklist template:
  - `docs/adr/0005-adversarial-lane-coexistence-policy.md`
  - `docs/adr/adversarial-lane-parity-signoff-checklist.md`
Frontier lane policy:
- Local setup is optional (`make setup` can skip provider key entry).
- Protected-lane frontier attempt is mandatory to run (attempt status is always emitted), but degraded frontier status is advisory and does not override deterministic blocking gates.
- Deterministic replay/coverage remains the release-blocking oracle; stochastic one-off frontier anomalies do not block until deterministic replay confirms them.
- Degraded-threshold tracker (`make test-frontier-unavailability-policy`) opens/updates a refresh action when protected lanes remain degraded for 10 consecutive runs or 7 days.
Simulation telemetry read policy:
- `/admin/events`, `/admin/cdp/events`, and `/admin/monitoring` include simulation-tagged rows whenever tagged simulation traffic is present.
- Tagged rows remain identifiable via `sim_run_id`, `sim_profile`, `sim_lane`, and `is_simulation`.
- `POST /admin/adversary-sim/history/cleanup` is the explicit cleanup control path; auto-off is not a retention cleanup action.
  In `runtime-prod`, cleanup requires `X-Shuma-Telemetry-Cleanup-Ack: I_UNDERSTAND_TELEMETRY_CLEANUP` (the Make target sends this header).
`test-adversarial-akamai` is fixture-driven (local `/fingerprint-report` with canned payloads) and does not require a live Akamai edge instance.
`test-remote-edge-signal-smoke` is the live ssh-managed-host proof for the currently implemented trusted-edge surfaces. It runs against the active normalized remote, uses SSH loopback transport to `127.0.0.1:3000` on the host, and proves:
- additive `/fingerprint-report` ingestion,
- authoritative `/fingerprint-report` ban behavior,
- trusted GEO country-header routing for challenge, maze, and block.
It does not yet prove future Akamai-native rate or rich-geo augmentations; those remain separate backlog work.
`test-telemetry-storage` is the focused telemetry-storage regression target for this tranche. It proves:
- monitoring summary and delta reads stay on bucket-indexed paths,
- daily monitoring rollups are built and reused,
- retention honors separate raw-event and monitoring retention controls,
- density-aware query budgeting trips when a narrow window becomes too key-dense,
- the shared-host evidence harness shape remains stable.
`make telemetry-shared-host-evidence` captures a live shared-host evidence report for the active normalized remote at `.spin/telemetry_shared_host_evidence.json`. Use it after deploying the current committed `HEAD` to confirm:
- total key counts by telemetry family,
- keys per retained hour for monitoring, eventlog, and rollups,
- telemetry-adjacent monitoring-detail key counts (`maze_hits:*`, tarpit active-bucket state),
- retention health and lag from `/admin/monitoring`,
- payload sizes and latency for `/admin/monitoring`, `/admin/monitoring/delta`, and `/admin/monitoring/stream`,
- transport gzip benefit for the monitoring snapshot.
Operator interpretation and tuning workflow is documented in `docs/adversarial-operator-guide.md`.

Manifest and fixtures live under:
- `scripts/tests/adversarial/scenario_manifest.v1.json`
- `scripts/tests/adversarial/scenario_manifest.v2.json`
- `scripts/tests/adversarial/scenario_manifest.schema.json`
- `scripts/tests/fixtures/akamai/`

Both manifests enforce `execution_lane=black_box`; unsupported lane values fail validation before runs start.
Makefile simulation targets execute `scenario_manifest.v2.json`; `make test-adversarial-manifest` validates both `v1` and `v2`.

## 🐙 Dashboard <abbr title="End-to-End">E2E</abbr> Smoke Tests (Playwright)

Run with:

```bash
# Terminal 1
make dev

# Terminal 2
make test-dashboard-e2e
```

Behavior:
1. Installs pinned Playwright dependencies via `pnpm` (through `corepack`).
2. Uses repo-local Playwright browser cache for deterministic execution:
   - browser cache: `.cache/ms-playwright`
   - by default the runner uses repo-local browser `HOME`/config at `.cache/playwright-home`
   - optional: set `PLAYWRIGHT_FORCE_LOCAL_HOME=0` to keep system `HOME`
   - if Chromium launch fails with a known sandbox signature while local HOME is forced, the runner retries preflight with system HOME
   - if preflight still fails with repo-local browser cache, the runner automatically retries with system Playwright browser cache (when `PLAYWRIGHT_BROWSERS_PATH` was not explicitly set)
3. Runs a Chromium launch preflight and fails fast with actionable diagnostics when sandbox permissions block browser startup.
4. Runs dashboard module unit tests via `make test-dashboard-unit`.
5. Runs dashboard bundle-size budget reporting (`scripts/tests/check_dashboard_bundle_budget.js`) against `dist/dashboard/_app` (in the e2e flow this checks the currently served build without rebuilding first).
6. Verifies the running Spin instance is serving the current dashboard artifact (`dist/dashboard/index.html`) and fails fast if the server is stale.
7. Seeds deterministic dashboard data via `make seed-dashboard-data`.
8. Runs browser smoke checks for core dashboard behavior:
   - only browser smoke specs (`e2e/*.spec.js`) are executed in this stage; Node unit tests (`e2e/*.unit.test.js`) run in `make test-dashboard-unit`
   - page loads and refresh succeeds
   - runtime page errors or failed <abbr title="JavaScript">JS</abbr>/CSS loads fail the run
   - only one dashboard tab panel is visible at a time (panel exclusivity)
   - auto-refresh defaults OFF and is only exposed on Monitoring/<abbr title="Internet Protocol">IP</abbr> Bans
   - polling cadence assertions explicitly enable auto-refresh toggle (60s production cadence)
   - monitoring and <abbr title="Internet Protocol">IP</abbr>-bans tabs use cursor-delta refresh by default, prefer <abbr title="Server-Sent Events">SSE</abbr> when available, and surface explicit freshness state (`fresh`/`degraded`/`stale`)
   - native Monitoring polling request fan-out stays within bounded per-cycle budget during remount/steady-state loops
   - seeded events/tables are visible
   - clean-state <abbr title="Application Programming Interface">API</abbr> payloads render explicit empty placeholders (no crash/blank <abbr title="User Interface">UI</abbr>)
   - form validation/submit-state behavior works
   - tab hash/keyboard routing works
   - `/dashboard` canonical path redirects to `/dashboard/index.html`
   - tab-level error states surface backend failures
   - sticky table headers remain applied
9. `make test` executes a final dashboard seed step (`make seed-dashboard-data`) after e2e so local dashboards retain recent sample data.

Notes:
- Seeding is test-only and does not run during `make setup`.
- Seeded rows are operational test data and may appear in local dashboard history.
- Restricted sandbox escape hatch (local-only): set `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1` to skip dashboard e2e after a detected Chromium launch permission block.
- CI safeguard: when `CI` is set, `PLAYWRIGHT_SANDBOX_ALLOW_SKIP=1` is rejected and the run fails so mandatory e2e checks cannot silently downgrade to skip.
- Bundle budgets are warn-only by default to preserve development flow; set `SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=1` (or run `make test-dashboard-budgets-strict`) for hard-fail enforcement.

## 🐙 Build Mode Notes

The Makefile switches crate types between `rlib` (native tests) and `cdylib` (Spin <abbr title="WebAssembly">WASM</abbr>) via `scripts/set_crate_type.sh`.
Integration tests do not run `cargo clean`; this avoids interrupting an already-running `make dev` watcher session.
Integration <abbr title="Proof of Work">PoW</abbr>/challenge sequence checks use a fixed test user-agent plus timing guardrails/retries for deterministic behavior.
Use the Makefile targets rather than calling scripts directly.

## 🐙 Generated Directories

These directories are generated locally/<abbr title="Continuous Integration">CI</abbr> and should never be committed:

- `dist/wasm/` - built Spin component artifact (`shuma_gorath.wasm`)
- `target/` - Rust build cache/output
- `.spin/` - local Spin runtime data/logs
- `.spin/deploy/` - local deploy receipts and preflight reports
- `playwright-report/` - Playwright <abbr title="HyperText Markup Language">HTML</abbr> report output
- `test-results/` - Playwright test result artifacts
- `.cache/ms-playwright/` - repo-local Playwright browser cache
- `.cache/playwright-home/` - repo-local Playwright HOME/config sandbox

`make clean` removes core build/test outputs, including stale local `src/*.wasm` artifacts.
Use `make reset-local-state` when you intentionally want to wipe local `.spin` runtime/test state (SQLite KV, logs, local deploy receipts, and verification receipt) while preserving durable operator artifacts under `.shuma`.

## 🐙 Manual Test Sequence (Optional)

Use these steps to manually validate behavior. They mirror the integration suite but let you inspect responses in detail.
If `SHUMA_FORWARDED_IP_SECRET` is set, include the matching `X-Shuma-Forwarded-Secret` header on requests that use `X-Forwarded-For`.
If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret` on `/health`.
Start the server in another terminal with `make dev` before running these steps.

1. Health check (loopback only):
```bash
curl -H "X-Forwarded-For: 127.0.0.1" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -H "X-Shuma-Health-Secret: $SHUMA_HEALTH_SECRET" \
  http://127.0.0.1:3000/health
```
Expected: `OK`. If `SHUMA_DEBUG_HEADERS=true`, headers `X-KV-Status` and `X-Shuma-Fail-Mode` are also present.

2. Root endpoint (<abbr title="JavaScript">JS</abbr> challenge / block page):
```bash
curl -i -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/
```
Expected: an "Access Blocked" page or a <abbr title="JavaScript">JS</abbr> verification interstitial.
If `SHUMA_JS_REQUIRED_ENFORCED=true`, the interstitial is used when no valid `js_verified` cookie is present.
If `SHUMA_POW_ENABLED=true`, the interstitial performs a short proof‑of‑work step before `js_verified` is issued by `/pow/verify`.
If `SHUMA_POW_ENABLED=false`, the interstitial sets `js_verified` directly in browser <abbr title="JavaScript">JS</abbr>.
After a valid `js_verified` cookie is set, the originally requested page reloads and access is re-evaluated.
For browser checks, use a private window and confirm the cookie is set after the first visit.

3. Honeypot ban:
```bash
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/instaban > /dev/null
curl -s -H "X-Forwarded-For: 1.2.3.4" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/ | head -5
```
Expected: "Access Blocked" for the banned <abbr title="Internet Protocol">IP</abbr>.

4. Admin ban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","duration":3600}' \
  http://127.0.0.1:3000/admin/ban
```
Expected: a <abbr title="JavaScript Object Notation">JSON</abbr> response containing the new ban entry.
Optional: verify with `GET /admin/ban` to confirm the <abbr title="Internet Protocol">IP</abbr> is listed.

5. Admin unban:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=1.2.3.4"
```
Expected: the <abbr title="Internet Protocol">IP</abbr> removed from the ban list.
Optional: verify with `GET /admin/ban` that the entry is gone.

6. Test mode toggle:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"test_mode": true}' \
  http://127.0.0.1:3000/admin/config
```
Expected: a <abbr title="JavaScript Object Notation">JSON</abbr> response with `"test_mode":true`.

7. Metrics endpoint:
```bash
curl http://127.0.0.1:3000/metrics
```
Expected: Prometheus metrics output.

8. <abbr title="Chrome DevTools Protocol">CDP</abbr> report intake:
```bash
curl -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  http://127.0.0.1:3000/cdp-report
```
Expected: a success response and a <abbr title="Chrome DevTools Protocol">CDP</abbr> event recorded in analytics.

9. Challenge replay behavior:
```bash
challenge_page=$(curl -s -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  http://127.0.0.1:3000/challenge/puzzle)
seed=$(python3 -c 'import re,sys; m=re.search(r"name=\"seed\" value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
output=$(python3 -c 'import re,sys; m=re.search(r"name=\"output\"[^>]*value=\"([^\"]+)\"", sys.stdin.read()); print(m.group(1) if m else "")' <<< "$challenge_page")
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
curl -s -X POST \
  -H "X-Forwarded-For: 10.0.0.150" \
  -H "X-Shuma-Forwarded-Secret: $SHUMA_FORWARDED_IP_SECRET" \
  --data-urlencode "seed=$seed" \
  --data-urlencode "output=$output" \
  http://127.0.0.1:3000/challenge/puzzle
```
Expected: first submit returns `Incorrect.` with a new-challenge link; second submit returns `Expired` with the same link.

## 🐙 Complete Manual Test Sequence

Assumes the server is already running in another terminal via `make dev`.
If you are using `SHUMA_FORWARDED_IP_SECRET`, export it before running this sequence.

```bash
set -e
BASE_URL="http://127.0.0.1:3000"
if [[ -z "${SHUMA_API_KEY:-}" ]]; then
  SHUMA_API_KEY="$(grep -E '^SHUMA_API_KEY=' .env.local | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$//')"
fi
FORWARDED_SECRET_HEADER=()
if [[ -n "${SHUMA_FORWARDED_IP_SECRET:-}" ]]; then
  FORWARDED_SECRET_HEADER=(-H "X-Shuma-Forwarded-Secret: ${SHUMA_FORWARDED_IP_SECRET}")
fi
HONEYPOT_PATH="$(curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "Authorization: Bearer $SHUMA_API_KEY" "$BASE_URL/admin/config" | python3 -c 'import json,sys; d=json.loads(sys.stdin.read()); print((d.get("honeypots") or ["/instaban"])[0])')"

echo "1) Health"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 127.0.0.1" "$BASE_URL/health"
echo ""

echo "2) Root (JS challenge / block page)"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "3) Honeypot ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL$HONEYPOT_PATH" > /dev/null
curl -s "${FORWARDED_SECRET_HEADER[@]}" -H "X-Forwarded-For: 1.2.3.4" "$BASE_URL/" | head -5
echo ""

echo "4) Admin ban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"ip":"10.20.30.40","reason":"manual_test","duration":3600}' \
  "$BASE_URL/admin/ban"
echo ""

echo "5) Admin unban"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "$BASE_URL/admin/unban?ip=1.2.3.4"
echo ""

echo "6) Test mode on, then off"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"test_mode": true}' \
  "$BASE_URL/admin/config"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  -H "Content-Type: application/json" -d '{"test_mode": false}' \
  "$BASE_URL/admin/config"
echo ""

echo "7) Metrics"
curl -s "$BASE_URL/metrics" | head -20
echo ""

echo "8) CDP report"
curl -s "${FORWARDED_SECRET_HEADER[@]}" -X POST -H "Content-Type: application/json" \
  -H "X-Forwarded-For: 10.0.0.200" \
  -d '{"cdp_detected":true,"score":0.5,"checks":["webdriver"]}' \
  "$BASE_URL/cdp-report"
echo ""
```

## 🐙 Local Testing Notes

- If you visit `/instaban` in a browser without `X-Forwarded-For`, your <abbr title="Internet Protocol">IP</abbr> is detected as `unknown`.
- To unban yourself locally:
```bash
curl -X POST -H "Authorization: Bearer $SHUMA_API_KEY" \
  "http://127.0.0.1:3000/admin/unban?ip=unknown"
```

## 🐙 Additional Manual Checks

- Allowlist: add your <abbr title="Internet Protocol">IP</abbr> via `/admin/config` and confirm access is always allowed
- Rate limit: send a burst of requests and confirm auto-ban
- Browser policy signal: send a low-version User-Agent (example: `Chrome/50`) and confirm botness signal output reflects `browser_outdated`
- <abbr title="Geolocation">GEO</abbr> policy: set `geo_*` lists via `/admin/config`, then send `X-Geo-Country` with a trusted forwarded-secret request and verify `allow/challenge/maze/block` routing precedence
- Ban list: `GET /admin/ban` and confirm entries match recent actions

## 🐙 Troubleshooting

Problem: `/health` returns 403
- Ensure you passed `X-Forwarded-For: 127.0.0.1`
- If `SHUMA_FORWARDED_IP_SECRET` is set, include `X-Shuma-Forwarded-Secret`
- If `SHUMA_HEALTH_SECRET` is set, include `X-Shuma-Health-Secret`
- Confirm the server is running with `make status`

Problem: Admin calls fail with 401/403
- Confirm `SHUMA_API_KEY` is correct
- If `SHUMA_ADMIN_IP_ALLOWLIST` is set, ensure your <abbr title="Internet Protocol">IP</abbr> is included

Problem: `make test` failed preflight (server not ready)
- Start the server with `make dev`
- Re-run with `make test`
- If startup is slow, increase wait timeout: `make test SPIN_READY_TIMEOUT_SECONDS=180`

Problem: Unsure what <abbr title="Internet Protocol">IP</abbr> the bot defence detected
- Query the ban list:
```bash
curl -H "Authorization: Bearer $SHUMA_API_KEY" \
  http://127.0.0.1:3000/admin/ban
```

## 🐙 Dashboard Manual Check

Open:
- `http://127.0.0.1:3000/dashboard/index.html`

Verify:
- Stats update on refresh
- Charts render correctly
- Ban/unban controls work
- Test mode toggle updates banner
- Fail-open/closed indicator matches deployment policy
- Login key should match `make api-key-show` (or your deployed `SHUMA_API_KEY`)
- Use the dashboard Ban <abbr title="Internet Protocol">IP</abbr> and Unban actions to validate the admin <abbr title="Application Programming Interface">API</abbr> wiring

## 🐙 Tips

Use browser developer tools to inspect:
- Network tab: headers, cookies, redirects
- Application tab: `js_verified` cookie
- Console: <abbr title="JavaScript">JS</abbr> errors
