Date: 2026-03-16
Status: Roadmap capture

Related context:

- [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
- [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
- [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)
- [`../observability.md`](../observability.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Purpose

Capture the major pre-launch work that Shuma still needs, but that is not yet fully planned in execution-ready detail, so implementation can be sequenced deliberately rather than opportunistically.

This note is intentionally a roadmap and sequencing capture, not an implementation-ready plan for every item listed.

# What Is Already Partially Planned

## 1. Adversary-sim maturation is started, but not complete

Already captured:

1. shared-host discovery first,
2. Scrapling surface catalog work,
3. blocked Scrapling runtime lane,
4. blocked containerized LLM lane,
5. deterministic oracle governance,
6. frontier data-governance work.

Current references:

1. [`2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md`](2026-03-04-scrapling-surface-catalog-and-emergent-lane-implementation-plan.md)
2. `SIM-SH-SURFACE-1` in [`../../todos/todo.md`](../../todos/todo.md)
3. `SIM-SCR-LANE-1` and `SIM-LLM-1` in [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

Gap:

1. there is not yet one mature end-state roadmap that ties deterministic, Scrapling, and containerized frontier lanes to the future tuning and oversight loop as one operating system.

## 2. Oversight-controller direction is planned, but the operator surfaces it depends on are not all ready

Already captured:

1. bounded oversight controller,
2. budget snapshots,
3. recommend/canary/autonomous rollout model.

Current references:

1. [`2026-03-15-agentic-era-oversight-design.md`](2026-03-15-agentic-era-oversight-design.md)
2. [`2026-03-15-agentic-era-oversight-implementation-plan.md`](2026-03-15-agentic-era-oversight-implementation-plan.md)

Gap:

1. monitoring and tuning surfaces are not yet planned as the operator-grade inputs and outputs of that controller.

## 3. Central intelligence, ban jitter, and local recidive now have design direction

Already captured:

1. banded ban jitter,
2. local repeat-offender ladder,
3. central intelligence classes and controller fit.

Current references:

1. [`2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md`](2026-03-16-agentic-era-ban-jitter-recidive-and-central-intelligence-design.md)

Gap:

1. storage architecture, governance, and sequencing relative to monitoring, tuning, and oversight are not yet broken out as their own roadmap work.

# Major Missing Planning Tracks

## A. Mature Adversary-Sim As An Operating Input, Not Just A Contributor Tool

Shuma still needs a coherent mature adversary-sim roadmap that ends with:

1. deterministic oracle lane,
2. Scrapling crawler/scraper lane,
3. containerized frontier-driven adversary lane,
4. credible representation of current-era automation, crawlers, scrapers, and agentic browsing behavior,
5. and clear integration into tuning and oversight loops.

This should explicitly answer:

1. what remains deterministic and release-blocking,
2. what remains emergent and advisory,
3. what telemetry from each lane is trustworthy enough to drive policy changes,
4. and how run results become tuning evidence rather than mere diagnostics.

## B. Tuning Surface Completion

Shuma still needs a plan for completing the Tuning tab as the operator surface for:

1. route thresholds,
2. defence thresholds,
3. ban families and duration policy,
4. recidive policy,
5. intelligence influence thresholds,
6. and the future controller-tunable config families.

This is not a simple UI cleanup task. It is a config-governance and operator-contract task.

The plan must define:

1. which thresholds belong in Tuning,
2. which belong elsewhere,
3. which are read-only diagnostics,
4. and which are safe for future autonomous recommendation or bounded auto-apply.

## C. Monitoring Overhaul For Operators, Not Contributors

Shuma still needs a monitoring redesign focused on operator questions:

1. where are attackers being effectively intercepted,
2. where are attackers probably getting through,
3. what is the apparent human-friction cost,
4. what did shadow mode predict would happen,
5. what actually happened under enforced mode,
6. and how should those two views stay clearly separated.

This is a foundational prerequisite for autonomous tuning. If monitoring remains contributor-diagnostic rather than operator-decisional, the controller will lack the right evidence surface.

## D. Adversary-Sim Telemetry Retention And Disposal

Shuma still needs a distinct lifecycle policy for adversary-sim telemetry.

Today, the repo has telemetry retention planning in general, but it does not yet fully settle:

1. whether adversary-sim telemetry should retain on the same timescales as real traffic,
2. when sim telemetry is considered actioned and disposable,
3. whether sim telemetry should have separate hot-read and raw retention windows,
4. and what evidence should remain after cleanup for audit without carrying full sim payload history.

This should be planned as its own operating model because the economics and value profile are different from real-traffic telemetry.

## E. Central-Intelligence Storage And Service Architecture

Shuma still needs a dedicated architecture plan for where central intelligence lives.

Questions still open:

1. repo-linked artifact, separate service, or provider-backed managed store,
2. publish/subscribe or periodic snapshot fetch,
3. what must be signed or authenticated,
4. whether data is site-local, fleet-local, or community-shared,
5. how removal and false-positive governance works,
6. and how Shuma consumes the data without coupling runtime decisions to a fragile external dependency.

This must be planned before implementation because the storage, trust, and governance model will shape every later API and telemetry contract.

## F. Scheduled Agent Analyzer, Recommender, And Reconfigurer

Shuma still needs a full plan for the scheduled agentic operator loop:

1. how it is scheduled,
2. which model/runtime stack it uses,
3. what data it reads,
4. what it is allowed to propose,
5. what it is allowed to apply automatically,
6. and whether code-change suggestions or pull requests are part of the same system or a separate one.

This must clearly separate:

1. config tuning recommendations,
2. config auto-apply,
3. code-change recommendations,
4. and code-change execution or PR creation.

Those are not the same risk class and should not be treated as one automation mode.

# Recommended Sequencing

## Stage 1: Monitoring And Tuning Foundations

1. Monitoring overhaul for operator decision-making.
2. Clear shadow vs enforced telemetry separation.
3. Tuning tab completion and config-governance alignment.

Reason:

1. without truthful operator monitoring and a complete tuning surface, neither human operators nor future scheduled agents have a solid control plane.

## Stage 2: Mature Adversary-Sim As A Tuning Input

1. Shared-host discovery baseline.
2. Scrapling lane.
3. Containerized frontier lane as a bounded emergent actor.
4. Explicit mapping from each lane's evidence to tuning confidence.

Reason:

1. Shuma needs realistic attacker input before automated tuning can be trusted to optimize against the actual agentic threat landscape.

## Stage 3: Sim-Telemetry Lifecycle

1. Separate retention and disposal policy for sim telemetry.
2. Clear distinction between actioned and unactioned sim evidence.
3. Audit residue kept minimal but sufficient.

Reason:

1. once emergent and frontier lanes exist, sim telemetry volume and cost will matter much more.

## Stage 4: Central Intelligence Architecture

1. Storage and service architecture.
2. Governance and false-positive process.
3. Observe-only ingest first.
4. Advisory usage before stronger enforcement.

Reason:

1. external and shared memory should not be wired into runtime policy before its trust and blast-radius model is explicit.

## Stage 5: Scheduled Agent Operator Loop

1. Recommend-only scheduled agent.
2. Narrow config auto-apply with canary and rollback.
3. Separate code-change recommendation path.
4. Only later, if ever, a PR-generating path with stricter review gates.

Reason:

1. the agent loop should stand on truthful monitoring, mature sim evidence, tuned config surfaces, and explicit central-intelligence governance.

# Recommended Design Calls To Lock Early

1. Keep request-path logic deterministic and Rust-only.
2. Treat monitoring overhaul as a prerequisite for serious autonomous tuning.
3. Treat tuning-tab completion as a control-plane contract, not a cosmetic dashboard task.
4. Keep adversary-sim telemetry retention distinct from real-traffic retention.
5. Treat central intelligence as a separate service or data plane concern, not a side effect of the Git repository.
6. Keep config auto-tuning and code-change/PR generation as separate systems with separate permissions and review paths.

# Roadmap Outcome

This roadmap suggests that the next pre-launch excellence sequence should be:

1. operator-grade monitoring,
2. tuning-surface completion,
3. mature adversary-sim lanes,
4. sim-telemetry retention lifecycle,
5. central-intelligence architecture,
6. scheduled agent analyzer and reconfigurer.

That order makes the future autonomous loop far more likely to be truthful, low-risk, and actually useful.
