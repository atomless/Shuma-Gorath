# Linode Shared-Host Live Proof

Date: 2026-03-06

## Goal

Prove the canonical same-host Linode path end-to-end against a real instance using `dummy_static_site` as the first static HTML acid test.

## Outcome

Success.

The canonical `make deploy-linode-one-shot` path completed against a prepared existing Linode instance with:

- a same-host upstream origin listening on `http://127.0.0.1:8080`,
- a TLS-capable public FQDN from the start,
- a generated surface catalog from `dummy_static_site`,
- remote bootstrap through the normal Make targets,
- post-start smoke passing for:
  - `/health`,
  - `/admin/config` auth posture,
  - `/metrics`,
  - reserved-route ownership,
  - forwarded public-path parity on `/css/style.css`,
  - challenge-route sanity.

## Proven Path

1. Run the setup helper and produce the handoff receipt:

   ```bash
   make prepare-linode-shared-host PREPARE_LINODE_ARGS="--docroot /Users/jamestindall/Projects/dummy_static_site --site-mode static-html-docroot"
   ```

2. Stage the protected origin on the prepared Linode host.

   Proof boundary:

   - this was outside current Shuma skill ownership,
   - the proof used a minimal same-host static origin rooted at `dummy_static_site`,
   - the origin listened on `127.0.0.1:8080`.

3. Attach Shuma to the prepared host with the canonical deploy path:

   ```bash
   SHUMA_GATEWAY_UPSTREAM_ORIGIN=http://127.0.0.1:8080 \
   SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true \
   SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true \
   SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true \
   SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true \
   make deploy-linode-one-shot DEPLOY_LINODE_ARGS="--existing-instance-id <linode-id> --domain <fqdn>"
   ```

4. Confirm `make smoke-single-host` passes on the remote host.

5. Confirm `systemd` runtime health and public TLS reachability.

## Crucial Gotchas Found During The Live Proof

1. Admin smoke checks must use an allowlisted forwarded IP, not loopback.
   Fixed by deriving the admin-route smoke IP from `SHUMA_ADMIN_IP_ALLOWLIST` by default and allowing override via `SHUMA_SMOKE_ADMIN_FORWARDED_IP`.
   Commit: `15e2aee`.

2. Shared-host parity smoke must prefer obvious static assets over HTML pages.
   HTML pages can legitimately trigger Shuma challenge flow, which makes them a poor deterministic forwarding oracle.
   Commit: `4ecb7ec`.

3. Gateway forwarding must canonicalize absolute request URIs before composing the upstream target path.
   Without this, the origin can receive paths like `/http://127.0.0.1:3000/css/style.css` and return a false `404`.
   Commit: `05a0376`.

4. The Linode deploy path ships committed `HEAD` only.
   Dirty worktree warnings were truthful. Uncommitted fixes were not part of the deployed bundle until committed and pushed.

5. Same-host local HTTP upstreams remain a special contract.
   The canonical Linode deploy path handles the local HTTP allowlist overlay automatically; ad hoc starts outside that path must set the equivalent gateway allowance explicitly.

6. Same-host origin staging is still a separate concern from Shuma attach.
   The setup and deploy skills now form a clean handoff, but they do not claim to generically stage arbitrary origin applications.

## Evidence Surface

- `make test-deploy-linode`
- `make test-gateway-profile-shared-server`
- live `make deploy-linode-one-shot` success against a real prepared Linode instance
- updated Linode setup/deploy skills and operations references

## Carry-Forward Interpretation

The shared-host Linode deployment tranche is now real, not hypothetical:

- the setup helper can create the deploy-ready receipt,
- the deploy helper can attach to a prepared same-host instance,
- the canonical smoke checks are now truthful against the production admin and forwarding contracts.

The remaining boundary is deliberate: generic origin staging is still site-specific and should stay outside Shuma's core skills unless a future generic hosting adapter is explicitly designed.
