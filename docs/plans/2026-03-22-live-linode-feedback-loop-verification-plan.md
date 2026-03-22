# Live Linode Feedback-Loop Verification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Prove the newly landed shared-host recommend-only feedback loop on the live Linode deployment using one canonical Make-backed verification path and durable receipts.

**Architecture:** Reuse the existing normalized ssh-managed remote receipt flow, the public admin read and control surfaces, and SSH loopback transport for the internal shared-host-only trigger boundary. Keep the verification slice focused on proof and tooling: no new controller semantics, no request-path execution, and no duplicate deploy workflow outside the existing `remote-update` path.

**Tech Stack:** Makefile targets, Python live-smoke tooling, existing remote SSH helpers, public admin APIs, shared-host systemd deployment, repo-native docs and TODO workflow.

---

## Task 1: Capture the exact live-proof contract

**Files:**
- Create: `docs/research/2026-03-22-live-linode-feedback-loop-verification-readiness-review.md`
- Modify: `docs/research/README.md`
- Modify: `todos/todo.md`

**Work:**
1. Record the current evidence gap between the older Linode receipts and the newer `OVR-RECON-1` / `OVR-AGENT-1` commits.
2. Freeze the live verification contract so the proof requires current `HEAD`, public status truth, internal shared-host trigger execution, and post-sim run linkage.
3. Add one execution-ready TODO entry for the live Linode feedback-loop proof.

**Acceptance criteria:**
1. The repo has a durable written reason for why the live proof is needed now.
2. The verification boundary is explicit before tooling work starts.

## Task 2: Add one truthful Make-backed live feedback-loop gate

**Files:**
- Create: `scripts/tests/live_feedback_loop_remote.py`
- Create: `scripts/tests/test_live_feedback_loop_remote.py`
- Modify: `Makefile`
- Modify: `docs/testing.md`
- Modify: `docs/deployment.md`
- Modify: `docs/adversarial-operator-guide.md`

**Work:**
1. Implement a focused live remote verifier that defaults to the active normalized ssh-managed target, reads public admin status from the deployed base URL, and uses SSH loopback for the internal shared-host-only trigger when required.
2. Verify public `GET /admin/oversight/agent/status` shape first.
3. Trigger one bounded internal agent run on the host and verify the status projection records it.
4. Drive one short adversary-sim run through `POST /admin/adversary-sim/control`, wait for completion, and verify a linked post-sim agent run appears.
5. Persist a bounded JSON proof receipt under `.spin/`.
6. Add unit coverage for the verifier so the Make target itself is regression-tested and truthful.

**Acceptance criteria:**
1. There is a single focused Make target for this live proof.
2. The target name matches its real scope: shared-host feedback-loop proof on a live ssh-managed remote.
3. The target leaves a durable local receipt describing what it proved.

**Verification:**
1. `make test-live-feedback-loop-remote`
2. `git diff --check`

## Task 3: Re-prove the live Linode target

**Files:**
- Modify: `.shuma/remotes/<active>.json` (through canonical remote-update helper metadata refresh)
- Create: `.spin/live_feedback_loop_remote.json` (or target-specific equivalent)
- Create: `docs/research/2026-03-22-live-linode-feedback-loop-proof.md`

**Work:**
1. Use the selected active normalized remote and push the current committed `HEAD` to it via `make remote-update`.
2. Run the focused live feedback-loop gate against that target.
3. Capture the deployed commit, target identity, verification commands, and the observed status/proposal evidence in a durable research note.

**Acceptance criteria:**
1. The live Linode receipt now points at the current feedback-loop `HEAD`.
2. The shared-host feedback loop is proved on the live target, not only locally.
3. The evidence is readable later without replaying the whole operation.

## Task 4: Review shortfalls and close the tranche

**Files:**
- Create: `docs/research/2026-03-22-live-linode-feedback-loop-post-verification-review.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`

**Work:**
1. Review the live proof against the readiness contract.
2. If the proof exposes any shortfall, add a follow-up TODO and execute it immediately.
3. Move the TODO entry into completed history with the exact evidence used.

**Acceptance criteria:**
1. The tranche ends with either a passing live proof or a truthful documented failure plus the next concrete follow-up.
2. No hidden verification gap remains between the recent shared-host feedback-loop work and the upcoming Monitoring overhaul.
