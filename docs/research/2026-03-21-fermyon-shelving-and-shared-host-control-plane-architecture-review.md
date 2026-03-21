Date: 2026-03-21
Status: Architecture review

Related context:

- [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md)
- [`../deployment.md`](../deployment.md)
- [`../plans/2026-02-13-provider-externalization-design.md`](../plans/2026-02-13-provider-externalization-design.md)
- [`../plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md`](../plans/2026-02-20-deployment-paths-and-adversarial-simulation-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md)
- [`../../todos/todo.md`](../../todos/todo.md)
- [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md)

# Question Reviewed

What should Shuma keep, defer, and clean up after deciding to shelve Fermyon as a near-term full runtime target and instead pursue a shared-host-first control plane, with any later edge posture limited to gateway-only responsibilities?

# Decision Summary

1. Shared-host or other ordinary host/container compute should remain the supported full Shuma runtime for the pre-launch loop.
2. The dashboard, adversary-sim supervisor, Scrapling worker, benchmark loop, and later scheduled diagnosis/recommend/apply agents should all live on that shared-host control plane, not on Fermyon.
3. Fermyon/Akamai edge should no longer be treated as part of the near-term full-runtime or adaptive-loop story. It should remain a deferred gateway-only adapter path.
4. The first adaptive loop should therefore optimize for shared-host execution, benchmark truth, and telemetry-derived adversary surface discovery rather than edge cron ownership or multi-instance edge convergence.

# Repo Review

## What Is Already Aligned

Several important parts of the repository already match the shared-host-first direction:

1. [`../adversarial-operator-guide.md`](../adversarial-operator-guide.md) already says the real hosted Scrapling runtime is shared-host-first, with host-side supervisor ownership and a truthful boundary that external edge supervisor services remain deferred.
2. [`../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md`](../plans/2026-03-20-mature-adversary-sim-evolution-roadmap.md) already makes the first closed loop shared-host-first: production operating envelope, minimal scope/seed gate, Scrapling emergent lane, benchmarkable telemetry, then recommend-only diagnosis.
3. [`../../todos/blocked-todo.md`](../../todos/blocked-todo.md) already blocks `SIM-EDGE-RUNTIME-1`, which means the repo had already started to separate full adversary runtime support from the edge posture.
4. The shared-host Scrapling deploy path already converges on the right realism contract: minimal root-oriented seed, fail-closed scope fence, and "telemetry is the map" rather than catalog-first discovery.

## What Drifted Toward Fermyon And No Longer Fits

The main drift is now in framing, sequencing, and backlog priority rather than in the primary hosted runtime:

1. Public docs still overstate Fermyon as a current primary deployment/runtime path or as a live-proven baseline from which follow-on Akamai edge work should now proceed.
2. The pre-launch roadmap still presents edge-instance distributed-state correctness as a mainline prerequisite before verified identity and the shared-host-first adaptive loop.
3. The February deployment and provider-externalization plans still describe Akamai/Fermyon multi-instance posture as an active implementation target rather than a deferred long-term gateway track.
4. The active TODO queue still presents `DEP-ENT-2..5`, `OUT-1..3`, and `AK-RG-2..8` as execution-ready work, even though that work is no longer on the chosen pre-launch path.

## Code-Level Architecture Still Shaped By Fermyon

The codebase also contains vendor-shaped assumptions that are not immediate bugs, but no longer fit the long-term architectural purity now being chosen:

1. `GatewayDeploymentProfile` is still vendor-specific (`shared-server` vs `edge-fermyon`) rather than a more semantic shared-host-versus-edge-gateway model.
2. Adversary-sim autonomous execution profile, cadence, and bounded request-budget logic still contain explicit `EdgeFermyon` branches and edge-cron assumptions.
3. Dashboard request budgets still special-case `edge-fermyon` rather than a more generic delayed-edge or remote-gateway profile.
4. The top-level deploy bias still points at Fermyon in places such as `make deploy` help text and `spin.toml` description.
5. Several admin and dashboard help surfaces still mention `edge-fermyon` directly, which keeps the deferred platform name embedded in the visible product model.

Those items should not be "fixed" opportunistically inside a docs tranche, but they should be made explicit as later cleanup work instead of remaining an undocumented architectural residue.

# What To Keep

The right cleanup is not to pretend the edge work never existed. Shuma should keep:

1. the shared-host-first full runtime path as the supported hosted posture,
2. the narrow Akamai-compatible provider seams and trusted edge-signal handling,
3. the existing Fermyon/Akamai deploy helpers and historical proof notes as a deferred gateway-only track,
4. and the repo-wide rule that telemetry is the map for adversary discovery and later diagnosis.

# What To Defer

The work that should move off the pre-launch mainline is:

1. enterprise distributed-state follow-on work (`DEP-ENT-2..5`, `OUT-1..3`) while shared-host remains the supported control plane,
2. Akamai Rate/GEO expansion work (`AK-RG-2..8`) while the edge posture itself is not on the mainline,
3. any attempt to make Fermyon host the full hosted adversary runtime or the scheduled agent loop,
4. and any design that requires the first recommend/analyze/reconfigure loop to live on the edge.

# Follow-On Cleanup Backlog Needed

This review implies two explicit follow-on architecture items:

1. `EDGE-GW-ARCH-1`: plan the later thin edge-gateway plus shared-host control-plane split architecture, including state ownership, signed forwarding, deployment/day-2 model, and which enterprise distributed-state guarantees actually matter once the edge is only a gateway.
2. `EDGE-GW-ARCH-2`: after that plan exists, remove or generalize the vendor-shaped runtime assumptions that were introduced for Fermyon-specific cadence, request budgets, deploy naming, and dashboard timing behavior.

# Conclusion

The repo is already much closer to the desired architecture than the older roadmap language suggests.

The correct cleanup is:

1. re-center the roadmap on the shared-host-first adaptive loop,
2. move edge/Fermyon follow-on work out of the active queue,
3. relabel the public edge story as deferred gateway-only work rather than the near-term full-runtime path,
4. and capture the remaining vendor-shaped code assumptions as intentional later cleanup instead of leaving them implicit.
