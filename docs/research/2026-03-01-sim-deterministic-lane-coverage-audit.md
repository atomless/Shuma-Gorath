# SIM Deterministic Lane Coverage Audit (Post-Heartbeat-Decoupling)

Date: 2026-03-01  
Status: Completed audit

## Scope

Audit the runtime-toggle deterministic adversary lane after autonomous heartbeat decoupling and confirm whether generated traffic now touches the intended Shuma detection/defence surfaces.

Primary implementation touchpoint:
- `src/admin/adversary_sim.rs` (`run_internal_generation_tick`)

## Deterministic Stimulus Matrix

1. `JS verification / generic challenge pressure`
   - Stimulus: repeated `GET` requests to `/sim/public/*` surfaces without solved challenge/verification state.
   - Expected effect: challenge/not-a-bot/JS-required decision paths can be exercised by risk posture.

2. `PoW`
   - Stimulus: `GET /pow` and malformed `POST /pow/verify`.
   - Expected effect: PoW issue + reject paths can be exercised.

3. `Challenge variants`
   - Stimulus: direct `GET /challenge/not-a-bot-checkbox`, malformed `POST /challenge/not-a-bot-checkbox`, malformed `POST /challenge/puzzle`.
   - Expected effect: challenge-serve and challenge-failure/reject paths can be exercised.

4. `Maze / tarpit`
   - Stimulus: maze entry probe via `crate::maze::entry_path(...)` and malformed `POST /tarpit/progress`.
   - Expected effect: maze progression and tarpit reject/escalation surfaces can be exercised.

5. `Rate limiting`
   - Stimulus: same-IP burst (`INTERNAL_RATE_BURST_IP`) against `/sim/public/search`.
   - Expected effect: rate-pressure and deny/escalation paths can be exercised.

6. `Fingerprint / scored bot signals`
   - Stimulus:
     - high-score `POST /cdp-report`,
     - UA/client-hints mismatch probe (`GET /sim/public/search?q=fingerprint-mismatch` with conflicting UA/CH headers).
   - Expected effect: botness/fingerprint signal accumulation and thresholds can be exercised.

7. `Ban paths`
   - Stimulus: honeypot hit via `/instaban` and abuse-style challenge/tarpit progression failures.
   - Expected effect: short/temporary deny and ban pathways can be exercised.

8. `GEO`
   - Stimulus: `x-geo-country: RU` on a deterministic subset of generated requests.
   - Expected effect: GEO signal/action path is stimulated when GEO policy lists are configured.

## Audit Result

1. Runtime-toggle deterministic generation now includes explicit stimuli for challenge variants, JS-required pressure, PoW, maze/tarpit, rate pressure, fingerprint/CDP, and ban paths.
2. Generation cadence and per-tick breadth are materially increased (`1s` heartbeat target, broader per-tick request mix including abuse probes and bursts).
3. GEO stimulation is present at request level but event emission remains config-dependent (default GEO policy lists are empty by default).

## Gaps and Follow-ups Opened

1. Config-dependent surfaces (`GEO`, optional IP-range policy actions) still need deterministic config-profiled verification to guarantee event emission in CI-style audits.
2. Runtime-toggle end-to-end category assertions are not yet a dedicated integration gate for every required surface category.

Follow-up TODOs were opened in `todos/todo.md` as `SIM-DET-2` and `SIM-DET-3`.
