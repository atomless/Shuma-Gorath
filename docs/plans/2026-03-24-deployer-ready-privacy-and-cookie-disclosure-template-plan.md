# Deployer-Ready Privacy And Cookie Disclosure Template Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a deployer-ready privacy and cookie disclosure template that operators can adapt for real Shuma deployments without reverse-engineering storage behavior from implementation docs.

**Architecture:** Keep this as a docs-only tranche. Reuse the existing privacy review and configuration reference as source truth, then add one operator-facing template doc and link it from the existing privacy and hardening guidance.

**Tech Stack:** Markdown docs, backlog closeout, and index updates.

---

## Guardrails

1. Do not present the template as legal advice.
2. Do not invent deployment behavior that Shuma does not actually implement.
3. Do not claim consent is never required; make clear that non-essential analytics or tracking changes the posture.
4. Keep the storage inventory aligned with current runtime behavior, including `SHUMA_EVENT_LOG_IP_STORAGE_MODE`.
5. Keep the deliverable deployer-ready rather than turning it into a generic legal explainer.

## Task 1: Add the research-backed template and supporting links

**Files:**
- Add: `docs/privacy-cookie-disclosure-template.md`
- Modify: `docs/privacy-gdpr-review.md`
- Modify: `docs/security-hardening.md`
- Modify: `docs/README.md`

**Work:**
1. Write a deployer-ready template with placeholders for controller identity, lawful basis, recipients, transfers, retention, and rights-handling contact.
2. Include a truthful cookie and browser-storage table for Shuma's default storage.
3. Include a truthful server-side storage and retention table for Shuma's default telemetry and session stores.
4. Link the template from the existing privacy review and hardening docs.

**Acceptance criteria:**
1. A deployer can start from one document instead of stitching together multiple implementation notes.
2. The template stays faithful to current Shuma storage behavior.

## Task 2: Close the planning and backlog trail

**Files:**
- Modify: `docs/research/README.md`
- Modify: `docs/plans/README.md`
- Modify: `todos/todo.md`
- Modify: `todos/completed-todo-history.md`
- Add: `docs/research/2026-03-24-sec-gdpr-4-deployer-ready-privacy-and-cookie-disclosure-template-post-implementation-review.md`

**Work:**
1. Add the new review and plan to the indexes.
2. Move `SEC-GDPR-4` to completed history.
3. Record the docs-only closeout and any remaining follow-on if exposed.

**Acceptance criteria:**
1. `SEC-GDPR-4` is visibly closed in the backlog.
2. The paper trail is complete and discoverable.

## Verification

1. `git diff --check`

Because this is docs-only, do not run behavior tests.

## Exit Criteria

This tranche is complete when:

1. the repo contains a deployer-ready privacy and cookie disclosure template,
2. that template reflects current Shuma storage and retention truth,
3. existing privacy and hardening docs point to it,
4. and the backlog and closeout trail show `SEC-GDPR-4` as delivered.
