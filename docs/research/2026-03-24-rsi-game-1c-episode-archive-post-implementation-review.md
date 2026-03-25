# RSI-GAME-1C Post-Implementation Review

Date: 2026-03-24

## What landed

`RSI-GAME-1C` now exists as a machine-first bounded episode archive rather than a UI-only history idea.

The delivered contract now:

1. persists a bounded `oversight_episode_archive_v1` ledger in
   [`src/observability/oversight_episode_archive.rs`](../../src/observability/oversight_episode_archive.rs),
2. records stable `episode_id` and `proposal_id` lineage from the live oversight apply path in
   [`src/admin/oversight_apply.rs`](../../src/admin/oversight_apply.rs) and
   [`src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs),
3. stores target or evaluation context, proposed move, acceptance state, watch-window state, retain or rollback outcome, baseline and candidate scorecards, benchmark deltas, hard-guardrail triggers, and evidence references,
4. projects the same archive through
   [`/admin/oversight/history`](../../src/admin/oversight_api.rs),
   [`/admin/oversight/agent/status`](../../src/admin/oversight_agent.rs),
   and the operator-snapshot payload builder in
   [`src/observability/operator_snapshot.rs`](../../src/observability/operator_snapshot.rs),
5. and fixes the homeostasis-input contract to a concrete bounded archive with `last_10_completed_cycles` rather than vague trend-reading prose.

## Why this matters

Before this tranche, Shuma had:

1. the decision ledger,
2. the canary apply lifecycle,
3. and the recent-change ledger,

but it still lacked one machine-first answer to:

1. what episode was attempted,
2. what baseline it started from,
3. what move it proposed or applied,
4. whether it was retained or rolled back,
5. and what actually improved or regressed afterward.

That gap would have made later stepping-stone search, run-to-homeostasis stopping, and LLM player-side episode reasoning depend on ad hoc joins across separate ledgers.

The delivered archive closes that gap.

## Verification

The focused proof path is now:

1. `make test-oversight-episode-archive`
2. `make test-oversight-apply`
3. `git diff --check`

`make test-oversight-episode-archive` proves:

1. archive upsert and bounded retention semantics,
2. candidate-vs-baseline benchmark delta calculation,
3. history payload projection,
4. agent-status projection,
5. and a retained-canary lifecycle updating one stable episode row rather than fragmenting history.

## Notes

The current dashboard does not yet render the raw archive directly.
That is intentional for this tranche.
The archive is machine-first first; later Game Loop projection work can read from the settled archive rather than inventing a second history model.
