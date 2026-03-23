# 🐙 Deferred Edge Gateway Track

Shuma’s current pre-launch mainline is the shared-host-first control plane. The first live-proven closed loop, hosted Scrapling runtime, bounded canary apply path, and upcoming Monitoring overhaul all target that shared-host path first.

The Fermyon/Akamai edge path is not deleted, but it is no longer a mainline runtime target for the adaptive loop. Treat it as a later gateway-only track.

## What Is Current

- Shared-host/Linode is the active deployment and control-plane path.
- The live closed-loop proof is on the shared-host target.
- `MON-OVERHAUL-1` and later operator-facing surfaces should reflect the shared-host-first loop, not the deferred edge path.

Primary current references:

- [`current-system-architecture.md`](current-system-architecture.md)
- [`plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`research/2026-03-22-live-linode-feedback-loop-proof.md`](research/2026-03-22-live-linode-feedback-loop-proof.md)

## What Is Deferred

The later edge track may still matter for:

- gateway-only request handling,
- trusted edge/header integrations,
- Akamai-specific operator controls,
- and later distributed-state or remote-gateway work once the shared-host-first loop is fully settled.

It is not the target for:

- the current closed tuning loop,
- hosted Scrapling runtime support,
- shared-host oversight agents,
- or the immediate Monitoring overhaul.

## Historical Edge Material

These docs were moved out of the active tree because they are historical receipts for a deferred track, not the right current entry points:

- [`research/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md`](research/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-live-proof-blockers.md)
- [`research/archive/outdated/deferred-edge/2026-03-12-fermyon-akamai-edge-live-proof.md`](research/archive/outdated/deferred-edge/2026-03-12-fermyon-akamai-edge-live-proof.md)
- [`research/archive/outdated/deferred-edge/2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md`](research/archive/outdated/deferred-edge/2026-03-14-fermyon-edge-signal-and-blank-slate-live-proof.md)
- [`plans/archive/outdated/deferred-edge/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md`](plans/archive/outdated/deferred-edge/2026-03-09-fermyon-akamai-edge-baseline-prerequisite-plan.md)
- [`plans/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md`](plans/archive/outdated/deferred-edge/2026-03-10-fermyon-akamai-edge-skill-implementation-plan.md)

## Later Edge References That Still Matter

- [`plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md`](plans/2026-03-21-shared-host-first-control-plane-and-deferred-edge-gateway-plan.md)
- [`research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md`](research/2026-03-21-fermyon-shelving-and-shared-host-control-plane-architecture-review.md)
- [`../todos/blocked-todo.md`](../todos/blocked-todo.md) under the deferred edge and enterprise distribution sections

## Skills And Commands

The edge-oriented skills and make targets still exist for later work and bounded experiments, but they should not be presented as the default pre-launch operating path.

Use:

- [`../skills/prepare-shuma-on-akamai-fermyon/SKILL.md`](../skills/prepare-shuma-on-akamai-fermyon/SKILL.md)
- [`../skills/deploy-shuma-on-akamai-fermyon/SKILL.md`](../skills/deploy-shuma-on-akamai-fermyon/SKILL.md)

only when you are explicitly working on the deferred edge gateway track.
