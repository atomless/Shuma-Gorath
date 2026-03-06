# Linode Shared-Host Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Harden the Linode shared-host deployment path so a real Shuma production deployment can be executed truthfully, repeatably, and then folded back into the Linode deployment skill.

**Architecture:** The canonical path remains Makefile-first and gateway-first. Deployment should run as local truthful preflight plus exact local `HEAD` bundle shipping, followed by remote bootstrap through canonical Make targets and post-start smoke. Browser login state is not part of the deploy path; the only browser-side dependency is one-time Linode token creation.

**Tech Stack:** Bash deployment wrapper, Python deploy/test helpers, Makefile orchestration, Spin runtime, systemd, Caddy, Linode API.

---

## Decisions Captured

1. The canonical Linode path must ship the exact checked-out local commit, not clone from GitHub on the VM.
2. The canonical first production pass must use domain/TLS from the start.
3. The Linode path must use production gateway guardrails before provisioning or cutover.
4. Successful deployment steps and crucial gotchas must be folded back into:
   - `skills/deploy-shuma-on-linode/SKILL.md`
   - `skills/deploy-shuma-on-linode/references/OPERATIONS.md`

## Prerequisites Before Real Linode Deployment

1. Fix Linode env generation so it matches the current `runtime-prod` contract.
2. Require the gateway production inputs in the Linode path:
   - `SHUMA_GATEWAY_UPSTREAM_ORIGIN`
   - `SHUMA_GATEWAY_DEPLOYMENT_PROFILE=shared-server`
   - `SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true`
   - `SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true`
   - `GATEWAY_SURFACE_CATALOG_PATH`
   - `SHUMA_GATEWAY_TLS_STRICT=true`
   - `SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true`
   - `SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true`
3. Make local deployment preflight authoritative before provisioning:
   - `make deploy-env-validate`
4. Replace VM-side `git clone` with exact local bundle shipping.
5. Run canonical remote deployment wrapper plus smoke after bootstrap.
6. Record the verified happy path and operator gotchas back into the Linode skill.

## Recommended Execution Order

1. Capture the Linode prerequisite gates and operator decisions in docs and backlog.
2. Add Linode-path tests for required production inputs and bundle generation.
3. Implement local bundle generation for exact `HEAD` deployment.
4. Update `scripts/deploy_linode_one_shot.sh` to:
   - require domain/TLS,
   - require truthful gateway/admin confirmations,
   - run local preflight before provisioning,
   - upload the local release bundle,
   - bootstrap from the uploaded bundle instead of `git clone`.
5. Update remote bootstrap/service start so restart paths do not depend on GitHub access.
6. Update `docs/deployment.md`, the Linode skill, and the Linode operations reference to match the new canonical path.
7. Run focused verification:
   - Linode deploy-path unit tests
   - gateway contract/path tests that do not require a live Linode account
8. Only then gather operator credentials and run the first real Linode deployment.

## Verification Expectations For This Tranche

1. The Linode path fails fast when required production env or attestations are missing.
2. The Linode path no longer depends on GitHub credentials on the VM.
3. The remote bootstrap uses canonical Make targets for validation/start/smoke.
4. Skill/docs text matches the real implemented path.
5. The first real deployment will still be required to capture environment-specific gotchas, but the repo path should be production-truthful before that run.
