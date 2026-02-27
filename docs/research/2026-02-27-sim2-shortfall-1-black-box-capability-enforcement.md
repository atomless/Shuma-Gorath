# SIM2 Shortfall Research 1: Black-Box Lane Capability Enforcement

Date: 2026-02-27  
Status: Completed

## Shortfall Statement

SIM2 established a black-box execution lane in policy, but current deterministic runner behavior still allows attacker-plane requests to inherit privileged capability material. This weakens the trust boundary because correctness depends on convention rather than construction.

## Current-State Evidence

1. Runner loads privileged secrets globally (`SHUMA_API_KEY`, `SHUMA_FORWARDED_IP_SECRET`, `SHUMA_CHALLENGE_SECRET`) in constructor context.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:276-280`.
2. Attacker-plane helper adds forwarded secret to generic request headers used by attacker scenarios.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:1517-1535`.
3. Attacker contract forbids several privileged headers, but does not forbid `X-Shuma-Forwarded-Secret`.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:116-122`, `1631-1645`.
4. Stale-token abuse simulation is produced by re-signing a seed with `SHUMA_CHALLENGE_SECRET`, which is not black-box behavior.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:1447-1467`.

## Research Findings

1. Black-box testing methodology requires external-observer posture and explicitly separates it from internal-knowledge approaches.  
   Source: OWASP WSTG, testing methodology and execution model  
   <https://owasp.org/www-project-web-security-testing-guide/>
2. PTES emphasizes explicit scope/assumptions and role separation to keep assessments trustworthy and repeatable.  
   Source: PTES methodology  
   <https://www.pentest-standard.org/index.php/Main_Page>
3. Capability systems reduce accidental authority by granting only the minimal rights needed to perform an operation.  
   Source: Capsicum capability model (USENIX Security)  
   <https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix>
4. Bearer credentials can be attenuated using caveats so each actor only receives scoped authority.  
   Source: Macaroons paper (NDSS)  
   <https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/>

## Addressing Options

1. Policy-only hardening (expand forbidden header list, add review checklist).
2. Structural hardening with capability-typed clients and explicit per-plane authority material.
3. Full process isolation for deterministic lane (separate process/user identity for attacker requests).

## Recommended Direction

Adopt option 2 now, then optionally add option 3 later if needed for stronger isolation.

Recommended control pattern:

1. Introduce explicit `ControlPlaneClient` and `AttackerPlaneClient` abstractions in runner code.
2. Make `AttackerPlaneClient` constructor refuse privileged headers and secrets by type/constructor contract.
3. Remove `X-Shuma-Forwarded-Secret` from attacker-plane request paths entirely.
4. Replace stale-token re-signing with black-box stale generation:
   - issue real seed,
   - wait/expire under low TTL (or server-provided stale fixture path in dev-only test harness),
   - submit unmodified seed.
5. Add contract test that fails if attacker lane can emit any privileged header or access admin endpoints.

## Success Signals

1. Attacker-plane scenarios execute without `SHUMA_API_KEY`, `SHUMA_CHALLENGE_SECRET`, and forwarded secret dependencies.
2. Replay/stale/ordering scenarios still pass using only public interface behavior.
3. CI contains a lane-contract check that fails on privilege regression.

## Source Links

1. OWASP WSTG: <https://owasp.org/www-project-web-security-testing-guide/>
2. PTES: <https://www.pentest-standard.org/index.php/Main_Page>
3. Capsicum (USENIX Security 2010): <https://www.usenix.org/conference/usenixsecurity10/capsicum-practical-capabilities-unix>
4. Macaroons (NDSS 2014): <https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/>
