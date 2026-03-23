# Fermyon / Akamai Edge Live-Proof Blockers

**Date:** 2026-03-10  
**Status:** Historical blocker note

This document records the initial live-proof blockers encountered on 2026-03-10.
Those blockers have since been resolved and the completed live proof is now in [`2026-03-12-fermyon-akamai-edge-live-proof.md`](2026-03-12-fermyon-akamai-edge-live-proof.md).

## Summary

At that point, the in-repo Fermyon / Akamai edge setup and deploy helpers were implemented and the focused helper verification path passed, but the first real edge proof was blocked by Fermyon account access and an upstream CLI login defect.

## Observed Friction

### 1. PAT login path panics in `spin aka login`

Observed on this machine with a real Fermyon personal access token:

- `spin aka login --token ...`
- `spin aka login` with `SPIN_AKA_ACCESS_TOKEN` exported

Observed failure:

```text
thread 'main' panicked at /Users/runner/work/neutrino/neutrino/plugin/src/commands/login.rs:159:32:
index out of bounds: the len is 1 but the index is 1
```

This occurred on `aka` plugin `0.6.0`, and the same panic had already been reproduced earlier on `0.4.4`.

### 2. Device login path works as a fallback, but the account is rejected by provider access control

The helper now falls back to interactive device login when the token-login panic is detected.

Observed browser outcome after successful GitHub auth:

```text
We are sorry...
User is not allow-listed!
```

Interpretation:

- authentication succeeded,
- but the account is not currently enabled for Fermyon Wasm Functions on Akamai,
- so no real deploy-capable session is available yet.

## What This Means

- At that point, `FERM-SKILL-1` and `FERM-SKILL-2` were complete as implementation tranches.
- At that point, `FERM-SKILL-3` was blocked externally.
- The blocker was no longer “missing repo setup/deploy automation”; it was:
  1. upstream `spin aka` PAT-login instability, and
  2. provider-side Wasm Functions allowlisting for the authenticated account.

## Recovery Path

1. Ensure the Wasm Functions access request is approved.
2. Confirm the browser login is using the same identity that requested access.
3. If the device-login page still says `User is not allow-listed!`, contact Fermyon support / Discord and include the exact error.
4. Use the blocked setup receipt at `.shuma/fermyon-akamai-edge-setup.json` to see the recorded blocker and resume instruction instead of rediscovering the failure manually.
5. Once access is enabled, rerun:

```bash
make prepare-fermyon-akamai-edge PREPARE_FERMYON_ARGS="..."
make deploy-fermyon-akamai-edge
```

## Evidence

- `make test-deploy-fermyon`
- `skills/prepare-shuma-on-akamai-fermyon/SKILL.md`
- `skills/deploy-shuma-on-akamai-fermyon/SKILL.md`
- `scripts/deploy/fermyon_akamai_edge_setup.py`
- `scripts/deploy/fermyon_akamai_edge_deploy.py`
