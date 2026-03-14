# Fermyon Blank-Slate Deploy and Edge-Signal Live Proof

Date: 2026-03-14

## Outcome

The Fermyon / Akamai edge baseline is now proven in the two places where the earlier proof was still incomplete:

1. a truly blank-slate app can be prepared, deployed, bootstrapped, and driven to a working adversary-sim state, and
2. the currently implemented trusted-edge signal surfaces are now proven through a Fermyon-native smoke path instead of only through the shared-host SSH proof.

This closes a real methodology gap. The earlier live proof established that the helper could publish and that the dashboard could converge on an existing edge app, but it did not yet prove:

- fresh-app creation from zero state,
- the full deployed adversary-sim path on a new app,
- or the real Fermyon edge signal contract using edge-native request identity semantics.

## Blank-Slate Deploy Proof

Fresh app:

- app name: `shuma-edge-fresh-20260314-112021`
- app id: `1f24a784-4585-42da-933c-7673ac5e25d8`
- base URL: `https://1f24a784-4585-42da-933c-7673ac5e25d8.fwf.app`

Precondition:

- `spin aka app status --app-name shuma-edge-fresh-20260314-112021 ...` returned `No app named ... found`

Preparation and deploy:

- `make prepare-fermyon-akamai-edge ... PREPARE_FERMYON_ARGS="--app-name shuma-edge-fresh-20260314-112021 --yes"`
- `make deploy-fermyon-akamai-edge ...`

Receipts:

- setup: `.shuma/shuma-edge-fresh-20260314-112021/fermyon-akamai-edge-setup.json`
- deploy: `.shuma/shuma-edge-fresh-20260314-112021/fermyon-akamai-edge-deploy.json`

What the fresh deploy proved:

- config bootstrap completed on the new app,
- dashboard/external smoke completed against the new edge URL,
- adversary sim enabled and produced real requests on the fresh app,
- a later autonomous follow-up tick arrived beyond the initial prime,
- the deploy receipt now truthfully records current app metadata and `git_head`.

## Edge-Signal Proof

The repo now has two distinct live proof targets because the transport and trust models are different:

- `make test-remote-edge-signal-smoke`
  - shared-host proof,
  - active `ssh_systemd` remote,
  - synthetic trusted forwarding over SSH loopback.
- `make test-fermyon-edge-signal-smoke`
  - Fermyon / Akamai proof,
  - current deploy receipt,
  - real edge identity semantics with no synthetic `X-Forwarded-For`.

The fresh Fermyon smoke passed:

- additive fingerprint ingestion,
- trusted GEO challenge,
- trusted GEO maze,
- trusted GEO block,
- authoritative fingerprint guardrail verification.

Report:

- `.spin/fermyon_edge_signal_smoke.json`

## Authoritative Fingerprint Truth on Fermyon

Authoritative fingerprint does not currently create a live ban on enterprise Fermyon when rate and ban state are still local-only. That is the correct current contract.

Live behavior:

- `POST /fingerprint-report` returned `503 Server configuration error`
- `spin aka logs` showed:
  - `enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative mode`

That means the smoke must treat this as a pass only when the logs prove the explicit distributed-state guardrail fired. Anything else would be dishonest:

- treating the `503` as success without log confirmation would hide a real outage,
- forcing a ban expectation would contradict the runtime guardrail,
- and reusing the shared-host synthetic-forwarding smoke would not exercise the real Fermyon trust path.

This stays the truthful live contract until the enterprise distributed-state tranche (`DEP-ENT-1..5`) lands.

## Telemetry Follow-Through

Telemetry evidence was refreshed against the same fresh app so the edge receipts no longer point at the older stale deploy head.

Fresh telemetry proof:

- `make telemetry-fermyon-edge-evidence ...`
- `make test-telemetry-hot-read-live-evidence ...`

Current telemetry receipt:

- `.spin/telemetry_fermyon_edge_evidence.json`

The fresh receipt reports:

- app id `1f24a784-4585-42da-933c-7673ac5e25d8`
- app name `shuma-edge-fresh-20260314-112021`
- deploy head `a19697ddb7ea8c6fb38cb6f54611f93fa76cce99`
- bootstrap `168.96 ms`
- delta `140.12 ms`

So the telemetry story is now exact on the same blank-slate app that proved the deploy path and the signal path.

## Methodology Corrections

This tranche changes the acceptance standard for Fermyon work:

1. Do not call Fermyon "working" from endpoint smoke alone; prove fresh setup, live deploy, and the runtime behavior that operators actually depend on.
2. Keep shared-host and Fermyon proof paths separate when the trust boundary or transport semantics differ.
3. When the correct behavior is a guardrail rather than a success response, acceptance must include live log evidence, not only HTTP status.
4. Refresh downstream evidence receipts after the deploy proof so telemetry notes do not keep stale-head caveats after the platform is actually fixed.

## Verification

- `make test-deploy-fermyon`
- `make test-deploy-linode`
- `make test-fermyon-edge-signal-smoke ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021`
- `make telemetry-fermyon-edge-evidence ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021`
- `make test-telemetry-hot-read-live-evidence ENV_LOCAL=/Users/jamestindall/Projects/Shuma-Gorath/.env.local SHUMA_LOCAL_STATE_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.worktrees/tel-evt-sparse-rows/.shuma/shuma-edge-fresh-20260314-112021 REMOTE_RECEIPTS_DIR=/Users/jamestindall/Projects/Shuma-Gorath/.shuma/remotes`
