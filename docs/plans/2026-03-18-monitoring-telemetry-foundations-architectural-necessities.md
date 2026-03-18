Date: 2026-03-18
Status: Settled and implemented prerequisite foundation

Related context:

- [`../research/2026-03-17-operator-decision-support-telemetry-audit.md`](../research/2026-03-17-operator-decision-support-telemetry-audit.md)
- [`../research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md`](../research/2026-03-18-agentic-era-operator-telemetry-research-synthesis.md)
- [`../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md`](../research/2026-03-18-cost-aware-operator-telemetry-gap-analysis.md)
- [`2026-03-12-unified-telemetry-hot-read-architecture-plan.md`](./2026-03-12-unified-telemetry-hot-read-architecture-plan.md)
- [`2026-03-13-compact-event-telemetry-implementation-plan.md`](./2026-03-13-compact-event-telemetry-implementation-plan.md)
- [`2026-03-18-monitoring-operator-summary-exactness-contract.md`](./2026-03-18-monitoring-operator-summary-exactness-contract.md)
- [`2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](./2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../src/runtime/request_flow.rs`](../../src/runtime/request_flow.rs)
- [`../../src/runtime/request_facts.rs`](../../src/runtime/request_facts.rs)
- [`../../src/runtime/policy_graph.rs`](../../src/runtime/policy_graph.rs)
- [`../../src/runtime/effect_intents/response_renderer.rs`](../../src/runtime/effect_intents/response_renderer.rs)
- [`../../src/runtime/capabilities.rs`](../../src/runtime/capabilities.rs)
- [`../../src/observability/hot_read_contract.rs`](../../src/observability/hot_read_contract.rs)
- [`../../src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)

# Purpose

Record the architectural necessities that should be settled before Shuma executes the `MON-TEL-1` telemetry-foundation tranche.

This note exists because the desired telemetry improvements are not just a matter of adding counters. Without a few explicit architectural decisions, the tranche risks:

1. duplicating telemetry logic across many response paths,
2. inventing inconsistent traffic-lane concepts,
3. overstating telemetry exactness,
4. and regressing hot-read and retained-footprint discipline.

# Necessity 1: Canonical Traffic-Lane Domain And Measurement Boundary

Shuma currently has rich request facts and defence-specific decisions, but it does not yet have one canonical coarse traffic-lane domain object.

Today:

1. `RequestFacts` carries raw request evidence such as botness score, GEO route, allowlist state, and not-a-bot marker validity.
2. `PolicyDecision` expresses defence outcomes such as `GeoChallenge`, `BotnessMaze`, or `ExistingBan`.
3. some requests return before the current request-total metric is even incremented, including early-route handlers, static bypasses, maze-path hits, checkpoint requests, and tarpit progress requests.

That means Shuma does not yet have one authoritative answer to:

1. which incoming requests are in scope for operator traffic denominators,
2. what lane each request belongs to,
3. and whether a lane was observed exactly or derived approximately.

This must be settled first.

Required outcome:

1. define one canonical coarse traffic-lane enum for current runtime use,
2. define exactly which request classes count toward operator denominators,
3. define which lanes are exact now versus reserved for future verified-identity work,
4. define the minimal route/action-family grouping that remains within cardinality guardrails.

Why this is a necessity:

1. lane summaries, suspicious-cost summaries, and human-friction rates all depend on the same denominator model,
2. and getting that model wrong early would poison every later operator chart and controller budget.

# Necessity 2: One Authoritative Request-Outcome Telemetry Hook

Shuma currently records some outcome telemetry at scattered points:

1. forwarded allows record metrics in one place,
2. block, challenge, maze, tarpit, redirect, and JS challenge responses are emitted through separate branches,
3. and post-response processing currently only flushes pending monitoring counters.

That is workable for the current subsystem metrics, but it is not a safe base for the new operator summaries.

The next tranche wants truthful totals for:

1. forwarded versus short-circuited requests,
2. response-byte totals,
3. suspicious-cost summaries,
4. and normalized defence funnels.

Those should not be assembled by sprinkling per-branch counter writes around every response path.

Required outcome:

1. introduce one authoritative request-outcome telemetry emission point,
2. make it responsible for the final request outcome class,
3. make it the single place where coarse forwarded/short-circuited/response-byte accounting is emitted,
4. keep it inside the existing runtime orchestration and capability model rather than inventing a parallel telemetry side path.

Why this is a necessity:

1. without it, `MON-TEL-1` will almost certainly duplicate logic across response branches,
2. and later changes to response behavior will silently drift away from telemetry truth.

# Necessity 3: Extend The Exactness Contract To Cover Operator Summaries

Shuma already has an explicit hot-read exactness model for several bootstrap components:

1. recent events tail,
2. recent sim runs,
3. security/privacy summary,
4. retention health,
5. active ban summary.

But the current bootstrap `summary: MonitoringSummary` does not itself participate in that exactness contract, even though it is central to operator interpretation.

That is acceptable for the current contributor-oriented Monitoring surface. It is not sufficient for the next operator-grade stage.

Required outcome:

1. extend the hot-read component contract so monitoring summary and new operator summaries have explicit exactness metadata,
2. define canonical source and projection model for each new summary,
3. distinguish exact, derived, and best-effort operator summaries clearly enough that future scheduled agents can consume them without pretending they are stronger than they are.

Concrete design contract:

1. [`2026-03-18-monitoring-operator-summary-exactness-contract.md`](./2026-03-18-monitoring-operator-summary-exactness-contract.md)

Why this is a necessity:

1. `MON-TEL-1` is explicitly about more decision-support telemetry,
2. and decision-support surfaces become dangerous if the exactness of their inputs is implied instead of declared.

# Necessity 4: Bootstrap Versus Supporting-Summary Ownership Must Stay Explicit

Shuma's recent telemetry work already proved something important:

1. hot-read documents are now a major part of retained footprint,
2. and bootstrap performance depends on keeping the default operator payload small and disciplined.

The next tranche proposes:

1. lane mix summaries,
2. suspicious-cost summaries,
3. normalized defence-funnel summaries.

Those are valuable, but they must not all be pushed straight into the bootstrap payload by default.

Required outcome:

1. decide which of the new operator summaries are bootstrap-critical,
2. decide which should live as separate supporting hot-read documents,
3. define clear rules for what belongs in bootstrap versus follow-up reads,
4. preserve the principle that raw tails and contributor diagnostics do not become fallback substitutes for missing operator summaries.

Concrete design contract:

1. [`2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md`](./2026-03-18-monitoring-bootstrap-and-supporting-summary-ownership-contract.md)

Why this is a necessity:

1. otherwise the tranche could succeed functionally while regressing the hot-read budget and retained-footprint goals it is supposed to respect.

# Recommended Sequencing

These necessities should be handled in this order:

1. define the canonical coarse lane and measurement boundary,
2. define the authoritative request-outcome telemetry hook,
3. extend the hot-read exactness contract to cover operator summaries,
4. settle bootstrap-versus-supporting-summary ownership,
5. only then implement the new counters, summaries, and read-surface wiring in `MON-TEL-1`.

# Impact On `MON-TEL-1`

This note does not replace `MON-TEL-1`. It narrows the safe way to execute it.

Practical implication:

1. `MON-TEL-1-1` should explicitly settle Necessity 1,
2. `MON-TEL-1-2` and `MON-TEL-1-4` should not proceed without Necessity 2,
3. `MON-TEL-1-5` should not proceed without Necessities 3 and 4,
4. and the later Monitoring UI overhaul should continue to remain blocked until this telemetry-foundation work lands on top of those decisions.

# Definition Of Done For The Prerequisite Slice

This prerequisite work is complete only when:

1. the lane taxonomy is explicit and shared,
2. request-outcome telemetry has one canonical owner,
3. operator-summary exactness is explicit in hot-read metadata,
4. bootstrap ownership rules for new summaries are documented and reflected in code contracts,
5. and the resulting `MON-TEL-1` implementation can proceed without inventing local patterns or implicit semantics.
