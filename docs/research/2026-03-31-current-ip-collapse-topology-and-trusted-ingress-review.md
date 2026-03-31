# Current IP-Collapse Topology And Trusted-Ingress Review

Date: 2026-03-31
Status: Current design driver for `SIM-REALISM-2I` and `SIM-REALISM-2J`

Related context:

- [`2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md`](./2026-02-27-sim2-shortfall-4-sim-telemetry-authenticity.md)
- [`2026-03-30-adversary-lane-wild-traffic-gap-review.md`](./2026-03-30-adversary-lane-wild-traffic-gap-review.md)
- [`../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md`](../plans/2026-03-30-adversary-lane-wild-traffic-gap-plan.md)
- [`../plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md`](../plans/2026-03-31-current-ip-collapse-topology-and-trusted-ingress-plan.md)
- [`../../src/lib.rs`](../../src/lib.rs)
- [`../security-hardening.md`](../security-hardening.md)
- [`../deployment.md`](../deployment.md)
- [`../testing.md`](../testing.md)

## Purpose

Freeze the exact conditions under which Shuma currently collapses client identity to `unknown`, clarify how that differs across shared-host, edge, health-check, and contributor-local flows, and define the only acceptable trusted-ingress shape for later sim-IP realism work.

This review is intentionally narrow. It does not try to solve proxy-pool realism, geo realism, or residential/mobile identity class realism. It only answers:

1. when current requests collapse to `unknown`,
2. which paths can already produce parseable client IPs,
3. and how sim traffic can gain realistic client-IP restoration without being granted privileged trust headers that external traffic must never possess.

## Current Extraction Contract

Current runtime extraction in [`src/lib.rs`](../../src/lib.rs) is strict:

1. On edge deployments, `extract_client_ip()` first trusts `true-client-ip`.
2. On all deployments, `x-forwarded-for` and `x-real-ip` are trusted only when `forwarded_ip_trusted(req)` is true.
3. `forwarded_ip_trusted(req)` is true only when:
   - `SHUMA_FORWARDED_IP_SECRET` is configured, and
   - the request includes matching `x-shuma-forwarded-secret`.
4. If neither trusted path is available, the request falls back to `unknown`.
5. `/shuma/health` is stricter than ordinary traffic and intentionally rejects multi-hop `x-forwarded-for` chains even when the forwarded-secret trust gate is satisfied.

The current implementation still has no active `remote_addr` fallback. The code comments leave it as a future placeholder rather than live runtime truth.

## Current Topology Matrix

| Topology | Trusted client-IP source available? | Current observed result |
| --- | --- | --- |
| Local contributor browser to `make dev` shared-host runtime | No | `unknown` |
| Direct adversary-sim worker traffic to local/shared-host runtime | No | `unknown` |
| External human or attacker reaching shared-host origin directly | No | `unknown` |
| Shared-host deployment behind Shuma-owned trusted proxy that injects `x-forwarded-for` plus matching `x-shuma-forwarded-secret` | Yes | First `x-forwarded-for` hop is used when parseable and non-empty |
| Shared-host deployment behind proxy that omits or mismatches `x-shuma-forwarded-secret` | No | `unknown` |
| Edge deployment with `true-client-ip` present and non-empty | Yes | `true-client-ip` value is used |
| Edge deployment without `true-client-ip`, but with trusted `x-forwarded-for` or `x-real-ip` | Yes | Forwarded header value is used |
| Edge deployment without either trusted path | No | `unknown` |
| `/shuma/health` with trusted single-hop `x-forwarded-for` or trusted `x-real-ip` | Yes | That single trusted value is used |
| `/shuma/health` with trusted multi-hop `x-forwarded-for` | Intentionally rejected | `unknown` |

## Consequences Of The Current Contract

### Humans And Attackers Both Collapse The Same Way

Shuma does not have a special "human identity" extraction path. Before any later policy or scoring logic runs, both human and attacker traffic are subject to the same trusted-header gate. If the gate is not satisfied, either can collapse to `unknown`.

### Local Contributor Browsing Is Intentionally Degraded Today

A contributor running `make dev` and visiting the root-hosted protected site in a browser is not currently behind a trusted forwarding layer, so client identity collapses to `unknown`. This is acceptable only if the UI and docs describe it truthfully as degraded local identity rather than pretending it is a realistic network IP.

### Current Sim Traffic Still Does Not Have Realistic Client-IP Restoration

Even after the broader realism work, direct Scrapling or Agentic worker traffic still reaches the shared-host runtime without the trusted ingress contract required for realistic client-IP restoration. That is why sim traffic often appears as `unknown` today.

### `h382`-Style Values Are Buckets, Not IPs

When a non-parseable identifier later feeds into bucketed identity logic, values such as `h382` are not source addresses. They are coarse identity buckets and must never be presented as if they were real client IPs.

## Options Considered

### Option 1: Let Workers Emit Trusted Headers Directly

Rejected.

Why:

1. it gives attacker-plane workers privileges that external callers must never possess,
2. it collapses the trust boundary between simulation and ordinary hostile traffic,
3. and it would teach the runtime to trust convenience headers rather than a real ingress authority.

### Option 2: Add A Sim-Only Runtime Bypass For IP Restoration

Rejected.

Why:

1. it would create a second client-IP authority path that only simulation can use,
2. it would make realism claims depend on a hidden privilege rather than the production trust model,
3. and it would undermine the repo-wide rule that simulator truth must not leak into defence truth.

### Option 3: Restore Client IP Only Through A Shuma-Owned Trusted Ingress Or Proxy

Recommended.

Why:

1. it preserves one trust authority for both external and simulated traffic,
2. it keeps workers unprivileged,
3. it matches the existing forwarded-header security model,
4. and it allows graceful degraded truth when the ingress is absent instead of inventing fake realism.

## Recommended Direction

Treat later IP-realism work as a trusted-ingress problem, not a worker-header problem.

The recommended design is:

1. keep `extract_client_ip()` and `forwarded_ip_trusted()` as the only authorities for trusting forwarded client identity,
2. introduce an optional Shuma-owned trusted ingress or proxy path for adversary-sim traffic,
3. require that ingress, not the workers, to add any trusted forwarded client-IP headers and the matching forwarded secret,
4. explicitly mark sim identity realism as degraded when that ingress is absent,
5. and expose observer-only receipt fields that distinguish:
   - trusted-ingress-backed identity,
   - pool-backed identity,
   - bucket-only identity,
   - and degraded local identity.

## Guardrails

1. Do not give Scrapling or Agentic workers `x-shuma-forwarded-secret`.
2. Do not add a sim-only client-IP extraction shortcut in runtime code.
3. Do not present `unknown` or `h*` buckets as real source IPs in operator surfaces.
4. Do not treat local contributor direct browsing as representative IP realism.
5. Keep `/shuma/health` on its stricter single-hop trust contract.
6. Keep future proxy-pool realism separate from this first trust-boundary addendum.

## Standards And Prior Research

This addendum follows the same trust-boundary direction already established in earlier research:

1. forwarded identity headers only become trustworthy when an explicit trust boundary owns them, not when arbitrary callers can emit them,
2. proxy-owned client identity should be bounded to trusted infrastructure rather than accepted from the open internet,
3. and security telemetry remains useful only when the provenance of identity data is itself trustworthy.

Relevant sources already captured in the earlier authenticity research:

1. RFC 7239 Forwarded header semantics: <https://www.rfc-editor.org/rfc/rfc7239>
2. Cloudflare True-Client-IP guidance: <https://developers.cloudflare.com/fundamentals/reference/http-request-headers/#true-client-ip-enterprise-plan-only>
3. NGINX `real_ip` trust-boundary guidance: <https://nginx.org/en/docs/http/ngx_http_realip_module.html>
4. NIST SP 800-92 log trustworthiness guidance: <https://csrc.nist.gov/pubs/sp/800/92/final>

## Consequence For The Active Realism Chain

`SIM-REALISM-2I` and `SIM-REALISM-2J` should start by proving the topology matrix above rather than jumping straight to proxy-pool realism.

That means implementation should first:

1. freeze focused proof for current shared-host, edge, contributor-local, and health-check identity behavior,
2. add the trusted-ingress path without widening worker privilege,
3. and only then upgrade receipts and operator wording so degraded identity truth is explicit.
