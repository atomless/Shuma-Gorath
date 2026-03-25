# Scrapling Geo/IP Policy Source Diversification Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the remaining request-native Scrapling-owned `geo_ip_policy` coverage gap using attacker-faithful public-network identity diversity rather than browser-stealth expansion.

**Architecture:** Keep the Scrapling lane black-box and request-native. Add bounded source-diversification or proxy-backed egress only for the owned `geo_ip_policy` surface, preserve the existing hostile request-native challenge behavior, and prove the result with receipts that stay grounded in observed telemetry.

**Tech Stack:** Rust adversary-sim worker-plan/runtime contracts, Python Scrapling worker, shared-host runtime bootstrap, focused Scrapling worker tests, operator-snapshot coverage receipts, Makefile verification.

---

## Guardrails

1. Do not solve this gap with trusted internal geo headers or any other privileged Shuma-only signal.
2. Do not widen Scrapling into browser or stealth mode unless an owned surface later proves that request-native behavior is insufficient for reasons other than source identity.
3. Keep the lane black-box: only public host knowledge and public-network identities are allowed.
4. Preserve bounded cost and explicit receipts for whichever source-diversification mechanism is adopted.

## Task 1: Freeze The Public-Network Identity Contract

**Files:**
- Modify: `docs/research/2026-03-24-scrapling-geo-ip-policy-source-diversification-review.md`
- Modify: `docs/plans/2026-03-24-scrapling-geo-ip-policy-source-diversification-plan.md`
- Modify: `todos/todo.md`

**Work:**
1. Freeze the rule that `geo_ip_policy` must be triggered only through attacker-faithful public-network identity diversity.
2. State explicitly which mechanisms are allowed:
   - bounded proxy-backed request-native egress,
   - bounded multi-source public identity rotation,
   - or equivalent public-network identity truth.
3. State explicitly which mechanisms are forbidden:
   - trusted internal geo headers,
   - Shuma-only privileged hints,
   - fake geo markers not available to outside attackers.

## Task 2: Add The Runtime And Worker Contract

**Files:**
- Modify: `src/admin/adversary_sim_worker_plan.rs`
- Modify: `src/admin/adversary_sim_lane_runtime.rs`
- Modify: `scripts/supervisor/scrapling_worker.py`
- Modify: `scripts/bootstrap/scrapling_runtime.sh`

**Work:**
1. Add a bounded public-network identity input contract for the Scrapling worker.
2. Keep it request-native and explicitly optional so unsupported deploys fail closed rather than pretending geo coverage exists.
3. Record receipts that show which identity class or proxy path was used when `geo_ip_policy` is touched.

## Task 3: Prove And Surface The Coverage

**Files:**
- Modify: `scripts/tests/test_scrapling_worker.py`
- Modify: `src/observability/scrapling_owned_defense_surfaces.rs`
- Modify: `src/observability/operator_snapshot_live_traffic.rs`
- Modify: `Makefile`
- Modify: `docs/testing.md`

**Work:**
1. Add focused worker proof that the request-native Scrapling lane can now touch `geo_ip_policy` truthfully.
2. Update the coverage summary so the current explicit gap assignment clears only when the geo surface is truly observed.
3. Keep the proof inside the existing receipt-backed coverage gate.

## Exit Criteria

This tranche is complete when:

1. `geo_ip_policy` is touched by attacker-faithful request-native Scrapling behavior,
2. the proof does not rely on privileged Shuma-only hints,
3. the operator snapshot no longer reports that surface as missing,
4. and `SIM-SCR-CHALLENGE-2C` remains conditional instead of becoming the default answer.
