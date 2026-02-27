# SIM2 Shortfall Research 3: Traffic-Model Execution Realism

Date: 2026-02-27  
Status: Completed

## Shortfall Statement

Manifest v2 defines realistic traffic metadata, but runtime execution remains mostly imperative per-driver request scripts. `traffic_model` currently influences validation/report metadata far more than actual request behavior.

## Current-State Evidence

1. Manifest requires `traffic_model` fields (`persona`, think-time, retry strategy, cookie behavior).  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:2233-2266`.
2. Runner validates and exports traffic metadata but does not apply a general execution engine that enforces those knobs across scenario drivers.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:2128-2172`.
3. Request path uses direct `urllib` calls with driver-specific imperative flows and no generalized persona scheduler or deterministic think-time runtime model.  
   Evidence: `scripts/tests/adversarial_simulation_runner.py:1537-1583`.

## Research Findings

1. Realistic load testing requires behavior models (user classes/tasks/wait-time), not only endpoint invocation.  
   Source: Locust writing locustfiles and wait-time modeling  
   <https://docs.locust.io/en/stable/writing-a-locustfile.html>
2. Scenario composition and executor models should express arrival pattern and pacing explicitly for reproducible realism.  
   Source: k6 scenarios/executors documentation  
   <https://grafana.com/docs/k6/latest/using-k6/scenarios/>
3. Automated-threat testing is strongest when traffic families map to distinct abuse behavior classes.  
   Source: OWASP Automated Threats to Web Applications  
   <https://owasp.org/www-project-automated-threats-to-web-applications/>

## Addressing Options

1. Keep current drivers and only annotate reports with traffic-model metadata.
2. Introduce a deterministic traffic execution runtime that consumes `traffic_model` and wraps all drivers.
3. Replace custom runner with external load-testing toolchain and adapter wrappers.

## Recommended Direction

Adopt option 2 to preserve current deterministic scenario framework while making `traffic_model` execution-real.

Recommended model:

1. Add shared execution policy application layer per scenario:
   - deterministic think-time from scenario seed,
   - retry behavior (`single_attempt`, `bounded_backoff`, `retry_storm`),
   - cookie strategy (`stateful_cookie_jar`, `stateless`, `cookie_reset_each_request`).
2. Add profile-level cohort scheduler that enforces persona mix rather than purely ordered scenario loops for selected profiles.
3. Keep deterministic mode by seeding all timing/retry randomness with scenario seed.
4. Add realism telemetry in report:
   - effective sleeps,
   - retries attempted,
   - cookie persistence mode applied.

## Success Signals

1. Changing `traffic_model` knobs changes observable runtime behavior deterministically.
2. Full-coverage profile can prove per-persona pacing/cookie/retry envelope evidence.
3. Report includes execution evidence, not only declarative metadata.

## Source Links

1. Locust user behavior modeling: <https://docs.locust.io/en/stable/writing-a-locustfile.html>
2. k6 scenarios and executors: <https://grafana.com/docs/k6/latest/using-k6/scenarios/>
3. OWASP Automated Threat Handbook: <https://owasp.org/www-project-automated-threats-to-web-applications/>
