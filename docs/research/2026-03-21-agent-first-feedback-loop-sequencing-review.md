Date: 2026-03-21
Status: Sequencing review

Related context:

- [`../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md`](../plans/2026-03-21-feedback-loop-closure-and-architectural-restructuring-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-design.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md)
- [`../plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../plans/2026-03-15-agentic-era-oversight-implementation-plan.md)
- [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](./2026-03-20-adversary-evolution-loop-role-synthesis.md)
- [`2026-03-21-feedback-loop-and-architecture-debt-review.md`](./2026-03-21-feedback-loop-and-architecture-debt-review.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Question Reviewed

Should Shuma complete the first agent-driven tuning loop over adversary-sim cost feedback before `MON-OVERHAUL-1`, or should Monitoring still be treated as the prerequisite?

# Decision Summary

1. The first machine-first agent tweaker loop should come before `MON-OVERHAUL-1`.
2. That first loop should remain shared-host-first, bounded, and recommend-only at first.
3. Monitoring should follow the first working agent loop so it projects the real trigger, evidence, proposal, benchmark-delta, watch-window, and rollback semantics that the backend loop actually uses.
4. The first agent loop should not introduce a second controller model. Periodic scheduling and post-sim triggering should both call the same internal reconcile contract.
5. The later broader scheduled or autonomous agent planning should remain a separate follow-on once the first bounded agent loop is proven and the human projections are shaped by that proof.

# Evidence From The Current Planning Chain

## 1. The repo already says the first diagnosis harness can precede a full Monitoring UI

The current synthesis chain already points this way:

1. [`2026-03-20-adversary-evolution-loop-role-synthesis.md`](./2026-03-20-adversary-evolution-loop-role-synthesis.md) says the first diagnosis harness can be machine-first and recommend-only before a full human Monitoring UI exists.
2. [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md) defines the first real closed loop as benchmarkable telemetry, recommend-only diagnosis or tuning harness, reviewed config change, and replay promotion.
3. [`../plans/2026-03-15-agentic-era-oversight-design.md`](../plans/2026-03-15-agentic-era-oversight-design.md) and [`../plans/2026-03-15-agentic-era-oversight-implementation-plan.md`](../plans/2026-03-15-agentic-era-oversight-implementation-plan.md) already standardize on one internal reconcile contract with thin scheduler adapters rather than separate controller implementations.

So the repo's later and more mature planning already supports an agent-first sequence better than the remaining blocked TODO wording does.

## 2. The current blocker wording is now the stale part

The main inconsistency is not the architecture direction. It is the sequencing residue in older roadmap and blocker text.

Today, some planning still implies:

1. Monitoring must be redesigned before the serious agent loop can exist.
2. Tuning semantics are primarily defined by the Monitoring or dashboard contract.

That is now the wrong order for this codebase.

The more recent feedback-loop review already concluded that the missing truth is:

1. benchmark comparison,
2. persisted objectives,
3. decision evidence,
4. replay-promotion integration,
5. and the recommend-only reconciler.

Once those exist, the next missing piece is not a chart redesign. It is the first bounded agent loop that exercises those contracts end to end.

## 3. Monitoring learns from the first working loop

If `MON-OVERHAUL-1` happens first, the UI has to guess:

1. which trigger modes matter,
2. which proposal families the backend actually emits,
3. which benchmark deltas and adversary-cost fields are decision-relevant,
4. which degraded or no-change outcomes matter,
5. and which watch-window or rollback details operators really need to inspect.

If the first agent loop lands first, Monitoring can instead project proven semantics:

1. periodic versus post-sim invocation,
2. benchmark delta and cost evidence,
3. typed proposals,
4. decision-ledger lineage,
5. watch outcomes,
6. rollback outcomes,
7. and replay-promotion follow-through.

That yields a more truthful human surface and avoids baking speculative operator semantics into the UI too early.

# Architectural Consequences

## 1. Split the agent work into two levels

The current single blocked `OVR-AGENT-2` item conflates two different things:

1. the first working bounded agent tweaker loop,
2. and the later broader scheduled or autonomous planning with more mature fleet, central-intelligence, and possibly code-evolution concerns.

Those should become two different stages:

### `OVR-AGENT-1`

The first shared-host agent tweaker loop.

It should:

1. consume `operator_snapshot_v1`, `benchmark_results_v1`, `allowed_actions_v1`, decision evidence, and adversary-sim cost outcomes,
2. run on the shared-host control plane,
3. support both periodic and post-sim trigger paths through one reconcile contract,
4. emit typed recommend-only proposals and durable evidence references,
5. and fail closed to no-change when evidence is stale, insufficient, or contradictory.

### `OVR-AGENT-2`

The later expansion for broader always-on scheduling, autonomous apply growth, central-intelligence-aware behavior, hosted-worker ownership, and any future relation to code-evolution paths.

That later planning should remain blocked until the first bounded loop is proven and Monitoring/Tuning are shaped around that proof.

## 2. Monitoring and Tuning become downstream projections

`MON-OVERHAUL-1` should now follow:

1. structural decomposition,
2. benchmark and objective truth completion,
3. replay-promotion contract integration,
4. the recommend-only reconciler,
5. and the first working machine-first agent loop.

Any future broader Tuning re-expansion should likewise follow the first bounded agent loop so it exposes:

1. the config families the backend actually tunes,
2. the safety envelopes the patch-policy layer actually enforces,
3. and the evidence and rollback semantics the backend loop actually records.

That keeps the human surfaces as truthful projections rather than speculative design exercises.

## 3. The first agent loop should still stay narrow

This sequencing change is not a license to jump straight to a wide autonomous controller.

The first loop should stay:

1. local to one Shuma site,
2. config-only rather than code-changing,
3. bounded by `allowed_actions_v1`,
4. recommend-only first,
5. backed by benchmark comparison and adversary evidence,
6. and explicitly off the request path.

# Recommended Sequencing Correction

The active mainline should now be:

1. behavior-preserving structural decomposition,
2. benchmark, objective, verified-identity-summary, and decision-evidence truth completion,
3. replay-promotion lineage integration,
4. `OVR-RECON-1` recommend-only reconcile engine,
5. `OVR-AGENT-1` first shared-host agent tweaker loop,
6. `MON-OVERHAUL-1` Monitoring projection,
7. a future bounded human control projection, if reopened through fresh planning,
8. `OVR-AGENT-2` later broader scheduled or autonomous agent planning,
9. and only then the code-evolution loop.

# Conclusion

The system should work for agents first.

For Shuma, that means the next truthful milestone after reconcile is not the Monitoring overhaul. It is the first bounded agent-driven tuning loop over sim-cost and benchmark feedback, running on the shared-host control plane and feeding durable decision evidence. Monitoring should follow that loop, not try to define it in advance.
