# Dashboard Tab: Game Loop

Route: `#game-loop`  
Component: [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)

Purpose:

- Human-readable accountability surface for Shuma's closed feedback loop.
- Keep recent loop rounds legible from the observer's point of view.
- Show the adversaries and defences from the last round without collapsing into subsystem-forensics detail.
- Avoid mixing operator accountability with the deeper diagnostics workflow that now lives in `#diagnostics`.

Current behavior:

- Exposes three bounded observer-facing sections:
  - `Recent Rounds`
  - `Adversaries In This Round`
  - `Defences In This Round`
- Projects current machine-first feedback-loop reads from:
  - `operator_snapshot_v1`
  - `oversight_history_v1`
  - `oversight_agent_status_v1`
- Surfaces:
  - a top-level recent-round history built from completed judged episodes, with completion time, participating lanes, retained versus rolled-back result, bounded config family when available, and continue versus stop state,
  - the `Adversaries In This Round` and `Defences In This Round` panels now show the freshest exact observer evidence available, in this order:
    - current mixed-attacker required-run evidence when `candidate_window` or `continuation_run` names exact `follow_on_run_id` values that are still present in `operator_snapshot.adversary_sim.recent_runs`,
    - otherwise the single latest exact recent sim run from `operator_snapshot.adversary_sim.recent_runs`,
    - otherwise the latest completed judged round from the durable archive,
  - lane names shown to operators use the user-facing dashboard contract, so `bot_red_team` is rendered as `Agentic Traffic` here instead of leaking the backend lane id into the observer story,
  - completed-round observer casts still come from a compact durable `observer_round_archive` returned by `oversight_history_v1`, keyed by `episode_id` and written once at judged-round completion,
  - that durable archive is populated only from exact judged-run receipts present at archive-write time, so the page no longer reconstructs rounds by lane-plus-time coincidence or backfills empty rows from unrelated recent runs,
  - when one or more judged run receipts were unavailable at archive-write time, the archive preserves that absence explicitly via `basis_status` and `missing_run_ids`, and the top sections say the cast is unavailable instead of guessing,
  - recent Scrapling observer summaries now preserve explicit lane-owned `category_targets` from the worker receipt path, and every Scrapling worker tick now writes an observer receipt even when it produced no owned-surface receipt rows, so the page does not lose adversary-role labels simply because a later tick had empty surface evidence,
  - an observer-facing adversary cast built from exact judged-run category truth plus recent recognition-evaluation comparison rows, so the page shows which simulator categories appeared without inventing impossible lane/category pairings or turning those simulator labels into runtime truth,
  - and an observer-facing defence cast built from the selected round’s receipt-backed surface rows:
    - Scrapling rows come from `owned_surface_coverage.receipts`,
    - `Agentic Traffic` rows come from `llm_runtime_summary.latest_action_receipts` projected onto named defended surfaces,
    - so the page stays surface-native and round-specific rather than reusing simulator labels as defence truth.
- Directs operators and contributors to `#diagnostics` for deeper subsystem inspection and rawer contributor-facing telemetry.

Current limitation:

- The remaining Game Loop truthfulness limitation is category-specific:
  - non-verified suspicious automation still routes mostly through `unknown_non_human`,
  - recent Scrapling category presence is now preserved as explicit observer-only lane-owned category truth for judged runs, but the recognition side still remains a recent category evaluation rather than a per-request transcript,
  - so exact live recognition scoring for Scrapling-populated categories remains intentionally bounded by what Shuma itself can infer from real shared-path request or behavior evidence.
- Machine-first judge, benchmark, and controller detail still exists in the backend; this tab now intentionally chooses not to render most of that lower-level diagnostic furniture.

Refresh behavior:

- On Game Loop activation, the dashboard runtime refreshes shared config plus the bounded accountability reads listed above.
- Completed judged-round history is read from the bounded durable observer archive on `/admin/oversight/history`; `operator_snapshot.adversary_sim.recent_runs` remains the transient exact source for the top cast panels whenever fresher current observer evidence exists.
- The tab now shares the top-level dashboard refresh bar:
  - manual refresh is available for on-demand accountability reloads,
  - auto-refresh is available when operators want the same live cadence used on the other active operational tabs.

Writes:

- Read-only tab (no config writes).
