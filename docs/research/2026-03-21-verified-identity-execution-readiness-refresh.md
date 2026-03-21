Date: 2026-03-21
Status: Readiness refresh

Related context:

- [`2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md`](2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-research-synthesis.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-design.md)
- [`../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md`](../plans/2026-03-16-agentic-era-verified-bot-identity-and-webbotauth-implementation-plan.md)
- [`../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md`](../plans/2026-03-16-pre-launch-roadmap-gap-capture-and-sequencing.md)
- [`../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md`](../plans/2026-03-20-machine-first-operator-snapshot-and-feedback-loop-implementation-plan.md)
- [`../../todos/todo.md`](../../todos/todo.md)

# Purpose

Refresh the verified-identity planning chain so it matches the current shared-host-first roadmap and exposes execution-ready backlog slices.

# Findings

1. The research, design, and implementation docs for verified identity already exist and are still directionally sound.
2. The main drift is sequencing: the older implementation plan still places verified identity after the human Monitoring overhaul and Tuning surface completion.
3. The refreshed roadmap now places verified identity after the delivered controller-grade telemetry and machine-first snapshot/benchmark foundations, but before mature adversary-sim expansion and before the later Monitoring/Tuning projection work.
4. That means human Monitoring/Tuning UI completion is no longer a blocker for starting verified identity.
5. The correct first execution slices remain the narrow observe-only foundations:
   - `WB-0.1` canonical identity domain
   - `WB-0.2` config placeholders and validation
   - `WB-1.1` provider seam normalization
   - `WB-1.2` observe-only telemetry
   - `WB-1.3` request-path annotations without routing change
6. The product stance should now be stated more explicitly: verified identity is primarily about exact restriction and exception management for non-human traffic, not about granting bots preferential treatment by default.

# Decision

1. Refresh the verified-identity implementation plan so its roadmap-fit section matches the current roadmap.
2. Add execution-ready `WB-0.*` and `WB-1.*` items to the active TODO queue.
3. Keep later policy/UI/service-profile phases in the implementation plan, but do not treat them as the first execution slice.
4. Treat `WB-0.1` as the next atomic implementation tranche after any final design discussion about adjacent inputs or inspirations.
5. Make the restrictive-default stance explicit in the planning docs so later policy and UI work does not drift toward "verified therefore favored" semantics.

# Why This Is Safe

1. These first phases are contract-and-observability work, not trust-boundary auto-allow work.
2. They preserve the repo rule that authenticated identity must not imply authorization.
3. They prepare the later Monitoring/Tuning projection and mature adversary-sim tranches instead of waiting on them.
4. They keep the primary operator value clear: better non-human restriction with explicit named exceptions.
