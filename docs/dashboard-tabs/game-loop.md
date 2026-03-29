# Dashboard Tab: Game Loop

Route: `#game-loop`  
Component: [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep recent loop rounds legible from the observer's point of view before the lower machine-first controller detail.
- Show the adversaries and defences from the last round without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Exposes the bounded Game Loop accountability sections:
  - `Recent Rounds`
  - `Adversaries In This Round`
  - `Defences In This Round`
  - `Round Outcome`
  - `Recent Loop Progress`
  - `Origin Leakage And Human Cost`
  - `Loop Actionability`
  - `Pressure Context`
  - `Trust And Blockers`
- Projects current machine-first feedback-loop reads from:
  - `operator_snapshot_v1`
  - `benchmark_results_v1`
  - `oversight_history_v1`
  - `oversight_agent_status_v1`
- Surfaces:
  - a top-level recent-round history built from completed judged episodes, with completion time, participating lanes, retained versus rolled-back result, bounded config family when available, and continue versus stop state,
  - completed-round observer casts now come from a compact durable `observer_round_archive` returned by `oversight_history_v1`, keyed by `episode_id` and written once at judged-round completion,
  - that durable archive is populated only from exact judged-run receipts present at archive-write time, so the page no longer reconstructs rounds by lane-plus-time coincidence or backfills empty rows from unrelated recent runs,
  - when one or more judged run receipts were unavailable at archive-write time, the archive preserves that absence explicitly via `basis_status` and `missing_run_ids`, and the top sections say the cast is unavailable instead of guessing,
  - recent Scrapling observer summaries now preserve explicit lane-owned `category_targets` from the worker receipt path, and every Scrapling worker tick now writes an observer receipt even when it produced no owned-surface receipt rows, so the page does not lose adversary-role labels simply because a later tick had empty surface evidence,
  - an observer-facing adversary cast built from exact judged-run category truth plus recent recognition-evaluation comparison rows, so the page shows which simulator categories appeared without inventing impossible lane/category pairings or turning those simulator labels into runtime truth,
  - an observer-facing defence cast built from the selected round’s receipt-backed Scrapling surface-contract checklist, so the top page stays surface-native and round-specific rather than reusing simulator labels as defence truth or borrowing broader breach-locus summaries into the selected round,
  - current benchmark overall status and improvement status plus separate current cards for terrain breach progress, evidence quality, exploit urgency, restriction confidence, abuse backstop, human-friction urgency, and top-level loop actionability,
  - bounded recent multi-loop oversight history rather than only the latest cycle,
  - completed judged-cycle lineage from the bounded episode archive, including retained versus rolled-back counts plus explicit homeostasis-break reasons and restart-baseline lineage,
  - true numeric budget usage for likely-human friction plus suspicious forwarded requests, bytes, and latency, with explicit wording that these are guardrails rather than proof of total attacker defeat,
  - a first-class `Terrain Breach Progress` panel showing terrain-local attacker advance separately from category posture,
  - named breach loci carrying explicit measured-vs-derived-vs-not-materialized truth for attempt counts, host-cost channels, repair families, and sample request or response evidence,
  - taxonomy rows as `Recognition Evaluation`, now rendered as recent per-category comparison rows instead of category-posture meters, explicitly described as the categorisation side quest rather than the primary adversary story, attacker surface-success proof, or bounded-tuning truth,
  - recognition summary counts showing exact matches, collapse to `unknown_non_human`, and still-not-materialized hostile categories separately from the main restriction quest,
  - a separate `Surface Contract Satisfaction` panel so compact Scrapling corroboration stays distinct from both exploit progress and category posture,
  - surface-contract blocking rows that now distinguish `attempted and blocked` from `required but unreached` when receipt-backed proof is present,
  - surface-contract blocker rows now carry dependency detail where available so the tab can distinguish an independent local miss from a downstream prerequisite miss,
  - explicit judge, restriction quest, recognition quest, grouped root-cause blockers, grouped controller outcomes, next-fix surfaces, move or escalation, config-ring, and code-evolution state inside `Loop Actionability`,
  - machine-first continuation status showing whether the loop is waiting on a fresh rerun, currently running that rerun, or stopped for an explicit reason before another bounded move can open,
  - recent config-change context from the operator snapshot,
  - and explicit trust or blocker rows for classification readiness, coverage, effective protected evidence, replay-lineage context, tuning eligibility, verified-identity guardrails, and the shared-path rule that simulator metadata does not count as category truth.
- Directs operators and contributors to `#diagnostics` for deep subsystem inspection and rawer contributor-facing telemetry.
- Keeps detailed adversary proof out of the tab:
  - `Red Team` is where operators verify Scrapling personas, categories, and owned-surface receipts,
  - `Game Loop` now starts with a compact round/adversary/defence box score, then shows bounded corroborating signals for the lower machine-first rails,
  - and the tab must still say clearly when a row is category posture math rather than direct attacker surface-contract truth, so attacker truth is visible without turning the tab into a forensic adversary surface.
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
  - recent Scrapling category presence is now preserved as explicit observer-only lane-owned category truth for judged runs, but the recognition side still remains a recent category evaluation rather than a per-request transcript,
  - so exact live recognition scoring for Scrapling-populated categories remains intentionally bounded by what Shuma itself can infer from real shared-path request or behavior evidence.
- The main current architecture limitation has now narrowed:
  - category posture no longer drives the top-level restriction status or bounded-tuning escalation when it is the only outside-budget family,
  - restriction urgency now explicitly carries `Restriction Confidence` and `Abuse Backstop` as separate machine-first states rather than flattening them into one urgency label,
  - controller diagnosis, recognition evaluation, and move selection are now explicit sibling benchmark surfaces rather than only implicit escalation-hint projections,
  - `Loop Actionability` now groups root causes, controller outcomes, and next-fix surfaces instead of flattening them into one blocker line,
  - the effective protected-evidence rail now admits strong live Scrapling runtime proof as a protected basis when the board-state evidence is localized, shared-path, and reproduced across the recent window, while replay lineage remains a separate provenance row rather than the only protected source,
  - and the remaining open Game Loop follow-on work has now moved to later controller-rail cleanup rather than still-missing restriction-confidence, abuse-backstop, or live protected-evidence semantics.
  - breach loci no longer render absent attempt counts as `0 attempts`, and missing board-state hints now stay labeled as `not materialized` instead of flattening into false certainty.
  - the shared-host strict loop now advances as `judge -> rerun -> judge -> next bounded move`; it no longer implies a one-episode stop or immediate patch chaining after a terminal judged cycle.

Refresh behavior:

- On Game Loop activation, the dashboard runtime now refreshes shared config plus the bounded machine-first accountability reads listed above.
- Completed judged-round casts are read from the bounded durable observer archive on `/admin/oversight/history`; `operator_snapshot.adversary_sim.recent_runs` remains a transient live window for current evidence, not the durable completed-round source.
- The tab now shares the top-level dashboard refresh bar:
  - manual refresh is available for on-demand accountability reloads,
  - auto-refresh is available when operators want the same live cadence used on the other active operational tabs.

Writes:

- Read-only tab (no config writes).
