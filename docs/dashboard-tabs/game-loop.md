# Dashboard Tab: Game Loop

Route: `#game-loop`  
Component: [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep loop outcome and controller judgment visible without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Exposes the bounded Game Loop accountability sections:
  - top status cards and runtime posture rows with no extra framing pane
  - `Recent Loop Progress`
  - `Origin Leakage And Human Cost`
  - `Loop Actionability`
  - `Board State`
  - `Trust And Blockers`
- Projects current machine-first feedback-loop reads from:
  - `operator_snapshot_v1`
  - `benchmark_results_v1`
  - `oversight_history_v1`
  - `oversight_agent_status_v1`
- Surfaces:
  - current benchmark overall status and improvement status plus separate current cards for terrain breach progress, evidence quality, exploit urgency, restriction confidence, abuse backstop, human-friction urgency, and top-level loop actionability,
  - bounded recent multi-loop oversight history rather than only the latest cycle,
  - completed judged-cycle lineage from the bounded episode archive, including retained versus rolled-back counts plus explicit homeostasis-break reasons and restart-baseline lineage,
  - true numeric budget usage for likely-human friction plus suspicious forwarded requests, bytes, and latency, with explicit wording that these are guardrails rather than proof of total attacker defeat,
  - a first-class `Terrain Breach Progress` panel showing terrain-local attacker advance separately from category posture,
  - named breach loci carrying attempt counts, host-cost channels, repair families, and sample request or response evidence,
  - taxonomy rows as `Recognition Evaluation`, explicitly described as the categorisation side quest rather than attacker surface-success proof or bounded-tuning truth,
  - category rows that render as `Unscored` with no meter when exact shared-path category evidence is not available,
  - recognition summary counts showing exact matches, collapse to `unknown_non_human`, and still-not-materialized hostile categories separately from the main restriction quest,
  - a separate `Surface Contract Satisfaction` panel so compact Scrapling corroboration stays distinct from both exploit progress and category posture,
  - surface-contract blocking rows that now distinguish `attempted and blocked` from `required but unreached` when receipt-backed proof is present,
  - surface-contract blocker rows now carry dependency detail where available so the tab can distinguish an independent local miss from a downstream prerequisite miss,
  - explicit judge, restriction quest, recognition quest, grouped root-cause blockers, grouped controller outcomes, next-fix surfaces, move or escalation, config-ring, and code-evolution state inside `Loop Actionability`,
  - recent config-change context from the operator snapshot,
  - and explicit trust or blocker rows for classification readiness, coverage, protected replay status, tuning eligibility, verified-identity guardrails, and the shared-path rule that simulator metadata does not count as category truth.
- Directs operators and contributors to `#diagnostics` for deep subsystem inspection and rawer contributor-facing telemetry.
- Keeps detailed adversary proof out of the tab:
  - `Red Team` is where operators verify Scrapling personas, categories, and owned-surface receipts,
  - `Game Loop` only shows bounded corroborating signals for the detailed attacker receipts, and must say clearly when a row is category posture math rather than direct attacker surface-contract truth, so attacker truth is visible without turning the tab into a forensic adversary surface.
  - the tab now treats exploit progress as a separate judge plane from both category posture and compact Scrapling corroboration.

Current limitation:

- The seeded operator-objective profile now defaults to `human_only_private`, so the normal current Game Loop stance is the strict human-only reference profile from [`src/observability/operator_snapshot_objectives.rs`](../../src/observability/operator_snapshot_objectives.rs).
- Suspicious forwarded request, byte, and latency budgets now derive from adversary-sim scope when that strict profile is active.
- Legacy or explicitly selected `site_default_v1` payloads can still be rendered for comparison or test coverage, and when they appear the tab must label them as mixed-site defaults rather than the strict `human_only_private` target.
- The remaining strict-loop limitation is now tranche-level rather than seed-level:
  - adversary-sim lanes are treated as `100%` non-human traffic during the first strict loop,
  - suspicious forwarded request, byte, and latency leakage should therefore move toward zero or equivalent fail-closed suppression in that phase,
  - repeated judged config-change improvement on the local `/sim/public/*` surface is now proven through `make test-rsi-game-human-only-proof`,
  - and later human traversal calibration must remain a separate proof ring rather than something inferred from sim traffic alone.
- The remaining Game Loop truthfulness limitation is still category-specific:
  - non-verified suspicious automation still routes mostly through `unknown_non_human`,
  - recent Scrapling category presence can still appear only as degraded `projected_recent_sim_run` evidence,
  - so exact live recognition scoring for Scrapling-populated categories remains intentionally unscored until Shuma itself can infer those categories from real shared-path request or behavior evidence.
- The main current architecture limitation has now narrowed:
  - category posture no longer drives the top-level restriction status or bounded-tuning escalation when it is the only outside-budget family,
  - restriction urgency now explicitly carries `Restriction Confidence` and `Abuse Backstop` as separate machine-first states rather than flattening them into one urgency label,
  - controller diagnosis, recognition evaluation, and move selection are now explicit sibling benchmark surfaces rather than only implicit escalation-hint projections,
  - `Loop Actionability` now groups root causes, controller outcomes, and next-fix surfaces instead of flattening them into one blocker line,
  - and the remaining open Game Loop follow-on work has now moved to breach-locus missing-data honesty plus later controller-rail cleanup rather than still-missing restriction-confidence or abuse-backstop semantics.

Refresh behavior:

- On Game Loop activation, the dashboard runtime now refreshes shared config plus the bounded machine-first accountability reads listed above.
- The tab now shares the top-level dashboard refresh bar:
  - manual refresh is available for on-demand accountability reloads,
  - auto-refresh is available when operators want the same live cadence used on the other active operational tabs.

Writes:

- Read-only tab (no config writes).
