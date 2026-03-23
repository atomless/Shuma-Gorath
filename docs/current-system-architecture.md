# 🐙 Current System Architecture

This document describes the current landed Shuma-Gorath architecture on `main`.

It is a current-state reference, not an <abbr title="Architecture Decision Record">ADR</abbr>. Use [`docs/adr/`](adr/README.md) for architectural decisions and this document for the shape of the system as it exists today.

Primary reference sources:

- [`module-boundaries.md`](module-boundaries.md)
- [`project-principles.md`](project-principles.md)
- [`research/2026-03-22-live-linode-feedback-loop-proof.md`](research/2026-03-22-live-linode-feedback-loop-proof.md)
- [`research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md)

## 🐙 Architecture Summary

Current pre-launch Shuma is a shared-host-first Rust control plane with:

- a thin HTTP trust-boundary shell in [`src/lib.rs`](../src/lib.rs),
- functional-core request orchestration in [`src/runtime/`](../src/runtime),
- barrier and signal domains for defence execution,
- a machine-first observability layer in [`src/observability/`](../src/observability),
- an admin/control plane in [`src/admin/`](../src/admin),
- bounded provider seams in [`src/providers/`](../src/providers),
- and a live-proven bounded feedback loop for adversary simulation, diagnosis, bounded config canary apply, watch-window comparison, and rollback.

The current closed loop is for bounded config tuning. The later broader LLM diagnosis and code-evolution loops are intentionally not part of the landed architecture yet.

## 🐙 System Context

```mermaid
flowchart TB
  subgraph Host["Shared-host deployment (current live-proven shape)"]
    Operator["Operator browser"]
    Visitors["Live ingress traffic<br/>humans, non-human traffic, verified agents"]
    SimTraffic["Adversary-sim generated traffic<br/>public ingress with sim telemetry tags"]

    subgraph Supervisors["Host-side supervisors"]
      AdvSup["run_with_adversary_sim_supervisor.sh"]
      OvrSup["run_with_oversight_supervisor.sh"]
    end

    subgraph App["Shuma Spin HTTP component"]
      Entry["HTTP trust boundary<br/>src/lib.rs"]
      Runtime["Request/runtime orchestration<br/>src/runtime/*"]
      Admin["Admin + internal control plane<br/>src/admin/*"]
      Obs["Observability + hot-read projection<br/>src/observability/*"]
      ConfigProv["Config + provider registry<br/>src/config/* + src/providers/*"]
    end

    KV["Spin KV / persisted state"]
    Dashboard["Dashboard SPA<br/>dashboard/src/*"]
    Optional["Optional external adapters<br/>Redis, Akamai-style fingerprinting,<br/>verified-identity provider/discovery"]
  end

  Operator --> Dashboard
  Dashboard -->|"Admin API calls"| Admin
  Visitors --> Entry
  SimTraffic --> Entry
  AdvSup -->|"poll /admin/adversary-sim/status"| Admin
  OvrSup -->|"POST /internal/oversight/agent/run"| Admin

  Entry --> Runtime
  Runtime --> Obs
  Runtime --> ConfigProv
  Runtime --> KV

  Admin --> Obs
  Admin --> ConfigProv
  Admin --> KV

  Obs --> KV
  ConfigProv --> Optional
```

## 🐙 Request-Time Runtime

```mermaid
flowchart LR
  Req["Incoming HTTP request"] --> Entry["src/lib.rs"]
  Entry --> Router["request_router<br/>early route handling"]

  Router -->|early/admin/special route| Early["health, metrics, robots, sim public,<br/>admin/internal surfaces"]
  Router -->|normal request| KVGate["kv_gate + runtime config load"]

  KVGate --> Facts["request_facts / normalization"]
  Signals["Signals domain<br/>GEO, CDP, JS, allowlist, UA,<br/>rate pressure, IP identity"] --> Facts
  Identity["Verified identity<br/>src/bot_identity/* via provider registry"] --> Facts
  Config["Config + provider registry"] --> Facts

  Facts --> Graph["policy_graph<br/>pure ordered decisions"]
  Graph --> Pipeline["policy_pipeline<br/>stage wiring"]
  Pipeline --> Plan["effect_intents plan_builder"]
  Plan --> Exec["effect_intents intent_executor"]

  Exec --> Enforce["Enforcement and barriers<br/>ban, block, rate, challenge,<br/>maze, tarpit, upstream proxy"]
  Exec --> Telemetry["Request telemetry<br/>metrics, monitoring, event log,<br/>request outcome, verified identity"]
  Enforce --> Resp["Rendered response"]
```

## 🐙 Control And Feedback Loop

```mermaid
flowchart LR
  Live["Live request outcomes"] --> Mon["Monitoring + event records"]
  Sim["Adversary-sim runs + sim-tagged traffic"] --> Mon
  Sim --> SimStatus["Adversary-sim status truth<br/>persisted event lower-bound recovery"]

  Mon --> HotRead["Hot-read projection"]
  Objectives["Operator objectives store"] --> Snapshot["operator_snapshot_v1"]
  Replay["Replay promotion summary"] --> Snapshot
  Taxonomy["Non-human taxonomy,<br/>classification, coverage, lane fulfillment"] --> Snapshot
  HotRead --> Snapshot

  Snapshot --> Bench["benchmark_results_v1"]
  Snapshot --> Reconcile["oversight_reconcile"]
  Bench --> Reconcile
  Allowed["allowed_actions_v1"] --> Reconcile

  Periodic["Periodic supervisor trigger"] --> Agent["oversight_agent"]
  PostSim["Post-sim trigger"] --> Agent
  SimStatus --> PostSim
  Agent --> Reconcile

  Reconcile --> Patch["oversight_patch_policy<br/>+ config validation"]
  Patch --> Apply["oversight_apply"]

  Apply -->|"bounded canary apply"| ConfigState["Persisted runtime config"]
  Apply -->|"watch window + candidate comparison"| Bench
  Apply -->|"rollback if degraded"| ConfigState

  Apply --> Ledger["Decision ledger + recent changes"]
  ConfigState --> Live
```

## 🐙 Domain Map

- Request trust boundary: [`src/lib.rs`](../src/lib.rs)
- Runtime orchestration: [`src/runtime/`](../src/runtime)
- Admin/control plane: [`src/admin/`](../src/admin)
- Observability and hot-read contracts: [`src/observability/`](../src/observability)
- Config and allowed action surfaces: [`src/config/`](../src/config)
- Provider seams and backend selection: [`src/providers/`](../src/providers)
- Signals: [`src/signals/`](../src/signals)
- Enforcement and barriers: [`src/enforcement/`](../src/enforcement), [`src/challenge/`](../src/challenge), [`src/maze/`](../src/maze), [`src/tarpit/`](../src/tarpit), [`src/deception/`](../src/deception)
- Verified non-human identity: [`src/bot_identity/`](../src/bot_identity)
- Dashboard operator surface: [`dashboard/src/`](../dashboard/src)
- Shared-host supervisor wrappers: [`scripts/run_with_adversary_sim_supervisor.sh`](../scripts/run_with_adversary_sim_supervisor.sh), [`scripts/run_with_oversight_supervisor.sh`](../scripts/run_with_oversight_supervisor.sh)

## 🐙 Live-Proven Current State

The current live-proven shape on Linode is:

- shared-host execution is the active full control-plane deployment model,
- periodic oversight runs are triggered by the host-side oversight supervisor,
- adversary-sim runs generate traffic through the public ingress path,
- completed sim status can now recover truthful lower-bound counters from persisted simulation-tagged event evidence,
- and the first bounded config canary loop is live-proven end to end.

Operational proof references:

- [`research/2026-03-22-live-linode-feedback-loop-proof.md`](research/2026-03-22-live-linode-feedback-loop-proof.md)
- [`research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md`](research/2026-03-23-adv-diag-1-adversary-sim-status-truth-post-implementation-review.md)

## 🐙 Not Yet In The Landed Architecture

These are intentionally downstream and should not be mistaken for current-state architecture:

- the later broader LLM-backed diagnosis/config harness,
- benchmark-driven code evolution and optional PR generation,
- a mature central-intelligence service architecture,
- and a full Monitoring overhaul that replaces the older mixed monitoring surfaces with a thin operator projection over the machine-first contracts.
