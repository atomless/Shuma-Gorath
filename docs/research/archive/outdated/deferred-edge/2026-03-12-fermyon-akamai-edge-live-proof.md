# Fermyon / Akamai Edge Live Proof

Date: 2026-03-12

Update (2026-03-14):

This note records the first live Fermyon baseline. It is extended by [`2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`](2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md), which adds:

- a fresh blank-slate app setup and deploy proof,
- a Fermyon-native trusted-edge signal proof path,
- and a refreshed telemetry evidence pass tied to the current deploy receipt head.

## Outcome

The Fermyon / Akamai edge setup and deploy path is now proven live.

- Setup receipt: `.shuma/fermyon-akamai-edge-setup.json`
- Deploy receipt: `.shuma/fermyon-akamai-edge-deploy.json`
- Account: `atomless` (`2d1ba909-1579-483c-bd75-f521eaabf8e9`)
- App: `shuma-edge-prod` (`79b823de-37b6-4a85-b3cc-16a40738c5a7`)
- Public URLs:
  - `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app`
  - `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.aka.fermyon.tech`

## Verified Success Criteria

The following live checks passed after deploy:

- `GET /dashboard/login.html` returned `200` HTML
- `GET /index.html` returned `200`
- authenticated `GET /admin/config` returned `200`
- live dashboard readiness completed in roughly `2.1s - 2.3s` on the deployed edge app
- the live Monitoring tab rendered its first real feed rows in roughly `2.6s`
- toggling Shadow Mode from the real dashboard UI converged successfully on the deployed edge app
- toggling Adversary Sim from the real dashboard UI converged successfully on the deployed edge app
- enabling adversary sim immediately produced a bounded first tick on the live edge app
- a later cron-driven follow-up tick arrived after enable without manual intervention
- the canonical external dashboard smoke passed against the deployed edge app, including a fresh simulation event observed in monitoring
- deploy receipt captured the live app id, account metadata, primary URL, and managed edge-cron metadata

## Real Friction and Resolutions

### 1. `spin aka login --token` panics on this machine

Observed behavior:

- `spin aka login --token ...` with `spin-aka 0.6.0` hit an upstream panic in `plugin/src/commands/login.rs`

Resolution:

- the setup/deploy helpers now treat this as a known upstream failure,
- interactive runs fall back to Fermyon device login instead of pretending PAT login worked,
- non-interactive runs stop with a truthful blocker.

### 2. Device login UX is not self-explanatory

Observed behavior:

- the browser page says “enter the code displayed on your device”
- the “device” is the terminal session running the helper, not a phone or separate workstation

Resolution:

- the setup guidance now makes that explicit and expects the agent to surface the current user code clearly.

### 3. Wasm Functions access approval can lag after request submission

Observed behavior:

- the first real auth attempt ended with `User is not allow-listed!`

Resolution:

- setup receipts now persist blocked progress state and exact rerun guidance,
- once provider access is granted, rerunning setup/deploy resumes cleanly from recorded state.

### 4. Fermyon deploy helpers were not inheriting canonical repo defaults

Observed behavior:

- the Python deploy helper only read `.env.local`
- `make`-level normalized defaults such as `SHUMA_MONITORING_RETENTION_HOURS` were not exported into the helper
- the rendered edge manifest declared missing variables with `default = ""`
- runtime then panicked on empty-string numeric values

Resolution:

- deploy now merges `config/defaults.env` with `.env.local` before shaping Spin variables,
- the edge deploy path now passes the same canonical defaults the rest of the repo already depends on.

### 5. Edge request trust needed explicit Fermyon/Akamai handling

Observed behavior:

- admin IP allowlist and HTTPS trust could not work correctly through the edge path without using Fermyon/Akamai request metadata

Resolution:

- runtime now trusts `true-client-ip` for client IP extraction and `spin-full-url` for HTTPS detection when `gateway_deployment_profile=edge-fermyon`.

### 6. First edge deploy needed explicit config bootstrap

Observed behavior:

- a fresh edge app returned missing-config `500`s after deploy because the KV config had not been seeded yet
- posting the full config JSON to `POST /admin/config` was not valid because that endpoint accepts the narrower patch schema

Resolution:

- added `POST /admin/config/bootstrap` for full-config seeding when config is absent,
- deploy helper now exports canonical seeded config JSON and uses the bootstrap endpoint before live smoke.

### 7. Fermyon cron cadence cannot run every minute for a single job

Observed behavior:

- `spin aka cron create` rejected the original once-per-minute adversary-sim schedule.
- the platform currently requires each individual cron job to run no more frequently than every five minutes.

Resolution:

- deploy now provisions a managed staggered cron set with five jobs (`shuma-adversary-sim-beat-0..4`),
- each job runs every five minutes, but the combined set yields an effective once-per-minute cadence,
- the deploy receipt records the cron job prefix, job count, and the exact schedules used.

### 8. Edge cron beat auth had to match the real transport shape

Observed behavior:

- the original internal adversary-sim beat path assumed the host-side supervisor contract: authenticated `POST` only.
- real Fermyon edge cron calls the route as `GET`, so cron jobs were present but generated no traffic.

Resolution:

- edge-fermyon now authorizes `GET /internal/adversary-sim/beat?edge_cron_secret=...` over HTTPS when the deployment profile is edge and the secret matches.
- host-side supervisor beats remain `POST`; the bypass stays scoped only to the internal beat endpoint.

### 9. Edge adversary sim needed a truthful first-tick contract

Observed behavior:

- even after cron was working, operator experience and deploy smoke were still weak because first visible traffic depended entirely on waiting for the next cron window.
- status could look `running` before any edge beat had actually occurred.

Resolution:

- enabling adversary sim on `edge-fermyon` now primes one bounded autonomous tick immediately at control acceptance,
- supervisor/generation diagnostics now report edge-cron cadence truthfully before and after the first tick,
- deploy smoke now requires both the initial prime and a later follow-up tick beyond that baseline, so the proof covers real autonomous generation instead of a one-off manual kick.

### 10. Edge dashboard writes needed explicit timeout budgets and retry-aware convergence

Observed behavior:

- the first edge proof was still incomplete even after cron generation was fixed, because the real dashboard UI could still look broken.
- global Shadow Mode and Adversary Sim toggles used the same short write budgets as shared-host/local flows.
- on the live edge app, adversary-sim control could legitimately take longer than the default dashboard timeout and could transiently return controller-lease `409` responses with `Retry-After`.
- the UI then rolled back toggles even though the backend finished enabling shortly afterwards, which made the dashboard appear non-responsive and hid real monitoring activity behind a misleading client-side failure.

Resolution:

- the dashboard now derives request budgets from `gateway_deployment_profile`,
- edge-fermyon uses longer write/status budgets than shared-server/local flows,
- the API client now preserves `Retry-After` on control failures,
- adversary-sim control now retries bounded `409`/`429` edge write failures instead of treating them as final,
- the canonical Fermyon deploy helper now runs an external live dashboard smoke that proves:
  - dashboard readiness,
  - global toggle convergence from the real UI,
  - monitoring visibility of a fresh simulation event,
  - and healthy reload behavior while adversary sim is already active.

## Evidence

- `make test-deploy-fermyon`
- `make deploy-fermyon-akamai-edge`
- standalone external smoke:
  - `SHUMA_BASE_URL=https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app SHUMA_API_KEY=<local-key> node scripts/tests/dashboard_external_live_smoke.mjs`
- live authenticated `POST /admin/adversary-sim/control` on the deployed app returned status with `generation.tick_count >= 1`, `generation.request_count > 0`, and `supervisor.heartbeat_active=true`
- live authenticated polling of `/admin/adversary-sim/status` after enable showed a later tick and request-count increase beyond that primed baseline
- live dashboard interaction proved:
  - Shadow Mode toggle on -> off converged from the real UI,
  - Adversary Sim toggle off -> on converged from the real UI without client rollback,
  - Monitoring loaded fresh simulation rows after enable,
  - and a reload while adversary sim was still active preserved truthful control/feed state
- live probes against:
  - `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app/dashboard/login.html`
  - `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app/index.html`
  - `https://79b823de-37b6-4a85-b3cc-16a40738c5a7.fwf.app/admin/config`

## Follow-on

With the edge baseline now proven, the Akamai edge control expansion tranche (`AK-RG-2..8`) can move back into the active queue.
