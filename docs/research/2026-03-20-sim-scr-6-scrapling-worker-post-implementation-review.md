# SIM-SCR-6 Scrapling Worker Post-Implementation Review

Date: 2026-03-20
Status: Completed

Related context:

- [`../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md`](../plans/2026-03-20-sim-scr-lane-1-runtime-migration-plan.md)
- [`../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](../plans/2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-20-sim-scr-1-lane-selection-post-implementation-review.md`](./2026-03-20-sim-scr-1-lane-selection-post-implementation-review.md)
- [`../../src/admin/adversary_sim.rs`](../../src/admin/adversary_sim.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../scripts/supervisor/adversary_sim_supervisor.rs`](../../scripts/supervisor/adversary_sim_supervisor.rs)
- [`../../scripts/supervisor/scrapling_worker.py`](../../scripts/supervisor/scrapling_worker.py)
- [`../../scripts/bootstrap/scrapling_runtime.sh`](../../scripts/bootstrap/scrapling_runtime.sh)
- [`../../Makefile`](../../Makefile)
- [`../../docs/api.md`](../../docs/api.md)
- [`../../docs/testing.md`](../../docs/testing.md)
- [`../../docs/adversarial-operator-guide.md`](../../docs/adversarial-operator-guide.md)

## Review Goal

Confirm that `SIM-SCR-6` landed as the real heartbeat-routing slice of `SIM-SCR-LANE-1`:

1. one runtime lane executes per beat,
2. `scrapling_traffic` now runs through a real bounded Scrapling worker,
3. hosted-scope and seed constraints remain fail-closed,
4. and the repo documents and verifies the real worker path truthfully.

## What Was Intended

This slice was meant to turn the earlier additive lane state into real runtime routing without jumping ahead to dashboard controls.

That means:

1. `synthetic_traffic` stays the deterministic baseline,
2. `scrapling_traffic` becomes a real worker-executed lane,
3. beat-boundary switching becomes real instead of planned-only,
4. and telemetry should come from bounded traversal, not from a precompiled catalog.

## What Landed

1. The internal beat path now routes by the selected active lane, keeping synthetic generation for `synthetic_traffic`, dispatching a bounded worker plan for `scrapling_traffic`, and failing closed for the still-unimplemented `bot_red_team` lane.
2. The host-side Rust supervisor now recognizes `scrapling_worker` dispatch mode, runs `scripts/supervisor/scrapling_worker.py`, and posts results to `POST /internal/adversary-sim/worker-result`.
3. The Python worker is real Scrapling, not a stub: it uses `FetcherSession`, bounded per-beat budgets, persistent `crawldir`, shared-host scope evaluation on every candidate and redirect, and signed simulation-tag headers on every emitted request.
4. Worker results now update persisted lane diagnostics through one internal contract, and stale/off-state results are rejected fail-closed instead of reviving an old run.
5. The focused verification surface is now real too: `make test-adversary-sim-scrapling-worker` proves beat routing, worker-result persistence, stale-result rejection, and real signed requests against a local hosted-scope fixture.
6. `make setup`, `make setup-runtime`, `make verify`, and `make verify-runtime` now know about the repo-owned `.venv-scrapling` runtime so the worker does not depend on an undeclared machine-global Python environment.

## Architectural Assessment

### 1. Desired versus active lane is now runtime truth, not planned-only metadata

This is the key value of the tranche.

Before `SIM-SCR-6`, the control plane could truthfully record `desired_lane=scrapling_traffic`, but the runtime always stayed synthetic.

Now the beat router actually converges the runtime onto the selected lane at the beat boundary, which makes `active_lane` operationally meaningful.

### 2. The emergent lane now follows the telemetry-as-map model

The worker starts from the minimal shared-host seed inventory and discovers additional in-scope pages by traversal.

That keeps the reachable surface derived from observed requests rather than from a rich precompiled catalog, which matches the newer 2026-03-20 migration and roadmap guidance.

### 3. The worker/runtime contract is now repo-owned instead of environment-implicit

Provisioning `.venv-scrapling` through the setup and verify flows is important architecture, not just convenience.

It turns the real worker into a declared part of the Shuma development and deployment contract, which reduces "works on one machine" drift for the next dashboard and rollout slices.

## Shortfalls Found During Review

Four real closeout issues were found and corrected before completion:

1. the first worker test fixture could hang on macOS because `HTTPServer.server_bind()` performed reverse DNS resolution during setup,
2. and the first worker session config used `retries=0`, which caused Scrapling request execution to fall through before any fetch attempt.
3. the setup verifier `make verify` was still reporting success even when `cargo test` failed because `scripts/bootstrap/verify-setup.sh` piped the test command through `tail` without preserving the exit code,
4. and two older Rust test expectations had drifted from the settled post-`SIM-DEPLOY-2` contract, which masked the real state of the suite until the verifier became truthful again.

All four were corrected and regression-covered inside this tranche.

One additional hardening gap remains open, but it is now explicit rather than hidden:

1. the older detailed Scrapling plan called for isolated outbound egress allowlisting at the worker-runtime level,
2. while this tranche currently enforces hosted scope in application logic and supervisor control flow,
3. and does not yet provide OS-level outbound sandboxing or deployment adapter enforcement for the worker process.

That gap does not block the 2026-03-20 runtime migration acceptance for `SIM-SCR-6`, but it should be closed as part of `SIM-SCR-8` rollout and operator guidance so the real deployment story matches the tighter historical hardening note.

## Result

Treat `SIM-SCR-6` as complete.

The next optimal tranche is `SIM-SCR-7`:

1. add dashboard lane controls,
2. project desired versus active lane honestly,
3. and keep `bot_red_team` visibly disabled while reusing the now-settled backend routing contract.
