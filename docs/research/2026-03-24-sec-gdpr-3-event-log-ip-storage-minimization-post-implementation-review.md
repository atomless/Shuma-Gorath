Date: 2026-03-24
Status: Completed

Related implementation:

- [`../../src/config/mod.rs`](../../src/config/mod.rs)
- [`../../src/admin/api.rs`](../../src/admin/api.rs)
- [`../../config/defaults.env`](../../config/defaults.env)
- [`../../Makefile`](../../Makefile)
- [`../../docs/privacy-gdpr-review.md`](../../docs/privacy-gdpr-review.md)

# SEC-GDPR-3 Event-Log IP Storage Minimization Post-Implementation Review

## What landed

`SEC-GDPR-3` now gives deployments an explicit env-only choice for how newly written event-log IPs are stored:

1. `raw`
2. `masked`
3. `pseudonymized`

The write-time mode is now applied in the canonical event-log persistence path, and each stored row records its immutable `ip_storage_mode` so mixed historical rows remain understandable after a deployment changes the env variable.

## Why this closes the tranche

Before this slice, Shuma only minimized event-log IPs at presentation time. Raw IPs always remained at rest, and forensic acknowledgement in admin views implicitly assumed raw data still existed. That was not enough for privacy-sensitive deployments.

After this slice:

1. storage-level minimization is configurable without changing the default behavior,
2. masked and keyed-pseudonymized rows minimize new data at rest,
3. forensic truth is no longer overstated when raw IPs are unavailable for new rows,
4. and the config/setup/deploy/runtime inventory path knows about the new env-only variable.

## Verification evidence

- `make test-event-log-ip-storage-mode`
- `make test-config-lifecycle`
- `git diff --check`

## Remaining follow-on

`SEC-GDPR-4` remains open. This tranche closes the runtime/storage control, but deployer-facing disclosure templates for lawful basis, retention inventory, and rights-handling workflow are still a separate documentation follow-on.
