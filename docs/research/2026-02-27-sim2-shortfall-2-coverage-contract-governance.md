# SIM2 Shortfall Research 2: Coverage Contract Governance

Date: 2026-02-27  
Status: Completed

## Shortfall Statement

SIM2 introduced coverage gates, but coverage requirements are still defined as mutable profile data rather than enforced as a single immutable contract. This allows drift between plan commitments and executable gate policy.

## Current-State Evidence

1. SIM2 plan defines full-coverage contract that includes tarpit progression and event-stream requirements.  
   Evidence: `docs/plans/2026-02-26-adversarial-simulation-v2-plan.md:63-68`.
2. `full_coverage` profile gate set omits required rows from the plan contract (for example tarpit progression and recent event count), while still passing if declared requirements pass.  
   Evidence: `scripts/tests/adversarial/scenario_manifest.v2.json:173-190`.
3. Runner coverage evaluation only checks whichever keys happen to be declared in profile `coverage_requirements`; no hard requirement that profile keys match contract baseline.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:701-708`, `2461-2479`.

## Research Findings

1. Threat emulation guidance emphasizes objective-driven plans with explicit measurable outcomes, not ad hoc scenario lists.  
   Source: MITRE CTID adversary emulation guidance  
   <https://center-for-threat-informed-defense.github.io/adversary_emulation_library/>
2. OWASP testing methodology stresses traceability from objective to test evidence for reliable reporting.  
   Source: OWASP WSTG  
   <https://owasp.org/www-project-web-security-testing-guide/>
3. Quantitative gates are most reliable when thresholds are codified and versioned as first-class artifacts.  
   Source: k6 thresholds documentation  
   <https://grafana.com/docs/k6/latest/using-k6/thresholds/>
4. Policy-as-code approaches reduce drift by evaluating a single declarative contract against produced evidence.  
   Source: OPA policy model and testing  
   <https://www.openpolicyagent.org/docs/policy-language>

## Addressing Options

1. Keep profile-local `coverage_requirements` but improve review discipline.
2. Define canonical coverage contract file and require profiles to satisfy or explicitly supersede it.
3. Generate profile requirements from canonical contract as build artifact to eliminate manual duplication.

## Recommended Direction

Use option 2 now, with a path toward option 3 if maintenance overhead appears.

Recommended contract model:

1. Add `scripts/tests/adversarial/coverage_contract.v1.json` as canonical list of required coverage categories and minimums for `full_coverage`.
2. Update runner validation so `full_coverage` fails if any canonical key is absent, renamed, or lower than minimum.
3. Add drift check tying:
   - SIM2 plan contract rows,
   - canonical coverage contract file,
   - manifest profile coverage requirements.
4. Emit report section that includes contract version hash and per-key threshold source.

## Success Signals

1. Contract drift becomes impossible to merge silently.
2. Coverage gate failures identify exactly which canonical category is missing.
3. Plan and executable contract stay synchronized through automated checks.

## Source Links

1. MITRE CTID adversary emulation library: <https://center-for-threat-informed-defense.github.io/adversary_emulation_library/>
2. OWASP WSTG: <https://owasp.org/www-project-web-security-testing-guide/>
3. k6 thresholds: <https://grafana.com/docs/k6/latest/using-k6/thresholds/>
4. OPA policy language: <https://www.openpolicyagent.org/docs/policy-language>
