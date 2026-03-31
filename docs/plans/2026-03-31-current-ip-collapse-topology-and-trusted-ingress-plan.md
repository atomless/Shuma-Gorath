# Current IP-Collapse Topology And Trusted-Ingress Plan

**Goal:** Implement `SIM-REALISM-2I` and `SIM-REALISM-2J` from one exact trust-boundary contract so Shuma can gain realistic sim client-IP restoration without granting privileged header capabilities to attacker-plane workers.

**Architecture:** Preserve the existing runtime client-IP authorities in [`src/lib.rs`](../../src/lib.rs), prove the current topology matrix first, add an optional Shuma-owned trusted ingress or proxy path for sim traffic, and then expose explicit degraded-versus-trusted identity realism receipts and operator wording.

Related context:

- [`../research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md`](../research/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-review.md)
- [`../research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](../research/2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](./2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../../src/lib.rs`](../../src/lib.rs)
- [`../../docs/security-hardening.md`](../../docs/security-hardening.md)
- [`../../docs/deployment.md`](../../docs/deployment.md)
- [`../../docs/testing.md`](../../docs/testing.md)

## Scope Boundary

This addendum is intentionally narrower than full identity realism.

Included here:

1. current `unknown` collapse topology proof,
2. trusted-ingress design for sim traffic,
3. degraded identity realism receipts and wording,
4. and contributor-local truth about direct browser traffic.

Explicitly not included here:

1. residential/mobile/datacenter pool realism,
2. geo affinity or ASN realism,
3. proxy-pool churn policy,
4. or broader category-labeling work.

Those remain separate realism work after the trust-boundary slice is correct.

## Acceptance Criteria

1. Topology truth is frozen in focused proof: shared-host direct, shared-host trusted proxy, shared-host misconfigured proxy, edge trusted header path, edge missing trusted header path, and `/shuma/health` single-hop versus multi-hop behavior are all proven explicitly.
2. Trusted sim-ingress realism uses the same forwarded-header trust gate as external traffic. No attacker-plane worker can emit `x-shuma-forwarded-secret` or any equivalent privileged header.
3. When trusted ingress is configured, sim traffic can surface parseable client IPs through that ingress path.
4. When trusted ingress is absent, receipts and operator surfaces explicitly report degraded identity realism rather than silently presenting `unknown` or `h*` as meaningful source IPs.
5. Local contributor browsing under `make dev` remains supported even when its identity is degraded, and docs describe that state truthfully instead of treating it as representative IP realism.

## Task 1: Freeze The Current IP-Collapse Topology Contract

**Status:** Landed on 2026-03-31.

**Files:**
- Later code targets: focused extraction tests around [`../../src/lib.rs`](../../src/lib.rs), related `Makefile` targets, and docs that describe local identity behavior
- Later proof targets: a new focused Make target for client-IP topology

**Work:**
1. Add a focused proof target for the current matrix captured in the research addendum.
2. Prove ordinary request extraction separately from `/shuma/health` extraction.
3. Prove that missing forwarded-secret trust collapses to `unknown` even if forwarded-style headers are present.

**Acceptance criteria:**
1. The current topology matrix is executable rather than purely descriptive.
2. `/shuma/health` stricter single-hop behavior is independently asserted.
3. Any future regression that silently trusts untrusted forwarded identity fails fast.

**Proof:**
1. Add and pass `make test-client-ip-topology-contract`.

## Task 2: Add Trusted Sim-Ingress Without Worker Privilege Creep

**Files:**
- Later code targets: adversary-sim supervisor or ingress adapter surfaces, deployment wiring, runtime trust-boundary docs
- Later proof targets: attacker-plane contract tests, ingress realism tests, `Makefile`

**Work:**
1. Introduce an optional Shuma-owned trusted ingress or proxy path for sim traffic.
2. Keep workers forbidden from sending privileged trust headers.
3. Reuse the existing forwarded-header trust gate instead of adding a sim-only client-IP bypass.

**Acceptance criteria:**
1. Trusted sim ingress can restore parseable client IPs without changing the runtime trust model.
2. Workers remain unable to impersonate trusted ingress directly.
3. Misconfigured ingress still fails closed to degraded identity truth.

**Proof:**
1. Add and pass `make test-adversary-sim-trusted-ingress-ip-realism`.
2. Keep attacker-plane contract checks and forwarded-header security tests green.

## Task 3: Expose Degraded Identity Truth Explicitly

**Files:**
- Later code targets: sim receipts, admin/operator read models, dashboard wording, docs
- Later proof targets: machine-contract tests, rendered observer proofs, `Makefile`

**Work:**
1. Add compact receipt fields that record whether identity truth was:
   - trusted-ingress-backed,
   - pool-backed,
   - bucket-only,
   - or degraded.
2. Update operator wording so `unknown` and hashed bucket values are presented as degraded or bucketed identity rather than as client IPs.
3. Keep those fields observer-only.

**Acceptance criteria:**
1. Operators can distinguish realistic client-IP observation from degraded local identity.
2. UI wording stops implying that bucket values are source addresses.
3. Receipt shape stays compact and machine-readable.

**Proof:**
1. Add and pass `make test-adversary-sim-identity-observer-truth`.
2. Keep relevant machine-contract and rendered dashboard truth tests green.

## Recommended Execution Order

1. Task 1: freeze topology proof first.
2. Task 2: land trusted sim ingress second.
3. Task 3: land degraded identity receipts and wording immediately after the ingress slice.

Do not reorder this into "UI wording first" or "proxy-pool realism first". The trust-boundary proof must exist before either later refinement is meaningful.
