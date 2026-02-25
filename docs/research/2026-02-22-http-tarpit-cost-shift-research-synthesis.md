# HTTP Tarpit Cost-Shift Research Synthesis

Date: 2026-02-22  
Status: Proposed research gate closure for pending tarpit/low-rate and SSH tarpit paper TODOs

## Scope

This synthesis combines:

1. Existing Shuma tarpit research and plan work:
   - [`docs/research/tarpit-research-2026-02-11.md`](tarpit-research-2026-02-11.md)
   - [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](2026-02-14-maze-tarpit-research-synthesis.md)
   - [`docs/plans/archive/2026-02-13-http-tarpit-excellence-plan.md`](../plans/archive/2026-02-13-http-tarpit-excellence-plan.md)
2. Pending paper TODOs in `todos/todo.md` related to rate limiting/tarpit and SSH tarpit:
   - `R-RL-01`, `R-RL-03`, `R-RL-05`, `R-RL-06`, `R-RL-07`
   - `R-SSH-02`, `R-SSH-03`, `R-SSH-04`
3. Fresh GitHub implementation evidence from active projects (metadata and docs pulled on 2026-02-22).

Goal: identify the most defensible approaches for shifting attacker cost to visitors while keeping host cost bounded in Shuma’s Rust/Spin runtime.

## Existing Shuma Baseline (Before New Work)

- Provider seam exists (`MazeTarpitProvider::maybe_handle_tarpit`) but internal runtime behavior has been effectively maze-only in practice.
- Challenge abuse currently routes to tarpit-or-short-ban; when tarpit is unavailable, short-ban is applied.
- Maze already contains important shared primitives required by existing TODO constraints (`TP-C1`, `TP-C2`):
  - signed traversal token chain (`MZ-2` scope),
  - replay/issue checkpoints,
  - concurrency and response-cap budget controls (`MZ-7` scope),
  - deterministic fallback semantics.

## Paper Findings and Shuma Implications

### R-RL-01 Cloud Control with Distributed Rate Limiting (SIGCOMM 2007)

Key findings:

- Distributed limiters can enforce a global limit across multiple sites while approximating behavior of a single shared limiter.
- Designs can explicitly trade communication overhead for accuracy/scalability.
- Reported overhead was low in tested settings.

Shuma implications:

- Tarpit/budget counters must remain convergent across instances in enterprise mode.
- Keep a single semantic model for global and per-bucket caps, with documented drift and outage behavior.

### R-RL-03 Evaluation of a Low-Rate DoS Attack Against Application Servers (Computers & Security 2008)

Evidence quality note:

- Public metadata is available, but full abstract text is not broadly exposed via index APIs.
- The conclusion below is partially inferential from title/domain context and aligns with already-reviewed low-rate DoS literature (`R-RL-02`, `R-RL-04`, `R-RL-08`).

Likely implication (inference):

- Low-rate traffic can still starve application/server resources and should not be treated as safe because aggregate request-per-second is modest.

Shuma implication:

- Tarpit entry and escalation must not be keyed to volume alone; cadence, persistence, and protocol-behavior evidence remain required.

### R-RL-05 Mitigating Application-Level DoS on Web Servers (ACM TWEB 2008)

Key findings:

- Proposed twofold approach:
  - admission control to bound concurrent service,
  - adaptive congestion/priority control among admitted clients.
- Emphasis on low overhead while preserving resilience.

Shuma implications:

- Admission control maps directly to hard tarpit budgets (global/per-bucket).
- Congestion/priority concepts map to risk-tiered enforcement: maze first, then stronger cost-imposition paths.

### R-RL-06 Resisting SYN Flood DoS Attacks with a SYN Cache (BSDCon 2002)

Key findings:

- Defenses should avoid expensive per-connection allocation before trust/progress is demonstrated.
- SYN cache/cookie designs protect queue/backlog resources under connection floods.

Shuma implications:

- Keep tarpit state lightweight and short-lived until additional corroboration.
- Do not allocate deep per-flow state for every suspicious request.

### R-RL-07 SMARTCOOKIE (USENIX Security 2024)

Key findings:

- Split-proxy architecture can absorb large-scale flood pressure while reducing stress on software backends.
- Host-cost asymmetry improves when expensive validation work is moved away from overloaded software paths.

Shuma implications:

- Enterprise deployment should treat edge/perimeter controls as pressure reducers, while Shuma remains the policy source of truth.
- Internal tarpit logic still needs strict local budgets and deterministic fallback even when edge help exists.

### R-SSH-02 Fingerprinting Bots in a Hybrid Honeypot (SoutheastCon 2023)

Key findings:

- Hybrid honeypot routing can reserve high-interaction resources for higher-value traffic.
- Multi-signal classification helps avoid wasting expensive interaction tiers on known low-value bot traffic.

Shuma implications:

- Apply stronger tarpit tiers only on high-confidence abuse classes.
- Keep lower-cost deception paths as the default for uncertain cohorts.

### R-SSH-03 Adaptive Self-Guarded Honeypot vs Conventional Honeypots (Applied Sciences 2022)

Key findings:

- Adaptive strategy can improve intelligence collection while reducing compromise risk compared with static designs.
- Static low/medium interaction honeypots are easier to fingerprint.

Shuma implications:

- Static tarpit signatures are a long-term liability.
- Future tarpit mode should support controlled variability in timing/content envelopes without violating hard budgets.

### R-SSH-04 Agnostic OT Honeypot Fingerprinting (TMA 2025)

Key findings:

- Holistic fingerprinting (including TCP/IP stack behavior) can identify honeypots at Internet scale.
- Realism and variation are essential for deception durability.

Shuma implications:

- Avoid globally stable tarpit artifacts.
- Rotate response shapes and keep protocol behavior plausible.

## GitHub Landscape (Pulled 2026-02-22)

### Active projects examined

- `0x4D31/finch` (283 stars, pushed 2025-12-06)
- `JasonLovesDoggo/caddy-defender` (503 stars, pushed 2026-02-22)
- `amenyxia/Sarracenia` (10 stars, pushed 2026-01-19)
- `die-net/http-tarpit` (66 stars, pushed 2022-06-28)
- `skeeto/endlessh` (8408 stars, pushed 2024-06-03)
- `shizunge/endlessh-go` (1226 stars, pushed 2026-02-19)
- `p0pr0ck5/lua-resty-tarpit` (28 stars, pushed 2022-10-11)

### Pattern convergence

1. **Bounded slow streaming is common**
   - Explicit byte-rate and timeout knobs (`bytes_per_second`, timeout/drip delays).
2. **Concurrency caps are mandatory**
   - Finch explicitly limits concurrent tarpit responses.
3. **Budget/fallback controls matter more than payload novelty**
   - Mature projects emphasize operational limits and queue safety.
4. **Observability is part of viability**
   - Endlessh-go and similar projects expose bytes/connections metrics.
5. **Non-blocking runtime models are favored**
   - OpenResty/Nginx and specialized protocol tarpits rely on event-driven I/O.

### What shifts cost best to the visitor

Most effective patterns in practice:

- hold attacker connection state longer than defender compute state,
- cap defender memory/FD concurrency hard,
- degrade gracefully to deterministic fallback on saturation,
- apply tarpit only after higher-confidence gating signals.

Patterns to avoid:

- unbounded long-lived responses,
- tarpit activation on weak/uncertain evidence,
- no saturation telemetry,
- tarpit-only state systems that diverge from existing enforcement primitives.

## Synthesis: Design Requirements for Shuma

1. **Internal-first and shared primitives**
   - Reuse maze token/budget/fallback machinery (`TP-C1`, `TP-C2`).
2. **Mode progression**
   - `maze_only` baseline first; `maze_plus_drip` only with hard caps and rollback thresholds.
3. **Hard budget envelope**
   - global concurrent cap, per-IP-bucket cap, per-response duration cap, per-response byte cap.
4. **Deterministic fallback matrix**
   - on budget exhaustion: policy-driven fallback (`maze` or `block`) with explicit reason codes.
5. **Risk-gated escalation**
   - persistent abusive tarpit clients can escalate to short ban/block with false-positive guardrails.
6. **Observability parity**
   - activation, saturation, duration, bytes sent, fallback, escalation outcomes.
7. **Enterprise coherence**
   - distributed-state support for tarpit counters/budgets is required before authoritative multi-instance posture.

## `self_hosted_minimal` vs `enterprise_akamai` Mapping

- `self_hosted_minimal`:
  - internal tarpit execution required,
  - strict local budget caps and fallback mandatory,
  - no external runtime dependency.
- `enterprise_akamai`:
  - edge signals can influence eligibility,
  - Shuma still owns tarpit semantics, counters, and explainability,
  - distributed-state correctness needed for authoritative behavior.

## Proposed Next Step

Adopt a phased implementation:

1. **Phase 1**: internal tarpit availability via maze-backed mode (`maze_only`) with explicit observability.
2. **Phase 2**: bounded drip mode (`maze_plus_drip`) with strict concurrency/time/byte limits and deterministic fallback.
3. **Phase 3**: persistence-based escalation and distributed consistency.

## Sources

- Existing Shuma docs:
  - [`docs/research/tarpit-research-2026-02-11.md`](tarpit-research-2026-02-11.md)
  - [`docs/research/2026-02-14-maze-tarpit-research-synthesis.md`](2026-02-14-maze-tarpit-research-synthesis.md)
  - [`docs/plans/archive/2026-02-13-http-tarpit-excellence-plan.md`](../plans/archive/2026-02-13-http-tarpit-excellence-plan.md)
- Paper sources:
  - R-RL-01: https://www.microsoft.com/en-us/research/publication/cloud-control-with-distributed-rate-limiting/
  - R-RL-03: https://doi.org/10.1016/j.cose.2008.07.004
  - R-RL-05: https://research.ibm.com/publications/mitigating-application-level-denial-of-service-attacks-on-web-servers-a-client-transparent-approach
  - R-RL-06: https://www.usenix.org/legacy/publications/library/proceedings/bsdcon02/full_papers/lemon/lemon_html/index.html
  - R-RL-07: https://www.usenix.org/conference/usenixsecurity24/presentation/chen-zeyu
  - R-SSH-02: https://doi.org/10.1109/SoutheastCon51012.2023.10115143
  - R-SSH-03: https://doi.org/10.3390/app12105224
  - R-SSH-04: https://doi.org/10.23919/TMA66427.2025.11097018
- GitHub projects:
  - https://github.com/0x4D31/finch
  - https://github.com/JasonLovesDoggo/caddy-defender
  - https://github.com/amenyxia/Sarracenia
  - https://github.com/die-net/http-tarpit
  - https://github.com/skeeto/endlessh
  - https://github.com/shizunge/endlessh-go
  - https://github.com/p0pr0ck5/lua-resty-tarpit
