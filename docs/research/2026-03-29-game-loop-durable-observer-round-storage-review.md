# 2026-03-29 Game Loop Durable Observer Round Storage Review

## Question

How do we keep the top-of-tab `Recent Rounds`, `Adversaries In This Round`, and `Defences In This Round` sections truthful and durable once a judged round is no longer represented in the bounded recent sim-run hot read?

## Why This Review Exists

The current exact-observer contract removed heuristic stitching, which was the right truth move, but it also exposed a durability gap:

- the Game Loop page now selects the latest judged round from `judged_run_ids`,
- then attempts to rehydrate those run ids from `operator_snapshot.adversary_sim.recent_runs`,
- and renders nothing for the cast when one or more judged run ids have already aged out of that bounded recent-run buffer.

The result is a real regression: Scrapling still emits category truth and surface receipts, but the page can no longer show them for the latest judged round once the transient recent-run window moves on.

## Investigated Code Paths

- Judged-round selection in [`dashboard/src/lib/components/dashboard/GameLoopTab.svelte`](../../dashboard/src/lib/components/dashboard/GameLoopTab.svelte)
- Recent sim-run hot-read projection in [`src/admin/api.rs`](../../src/admin/api.rs)
- Recent sim-run hot-read limits in [`src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)
- Operator snapshot hot-read projection in [`src/observability/hot_read_projection.rs`](../../src/observability/hot_read_projection.rs)
- Episode archive persistence in [`src/admin/oversight_api.rs`](../../src/admin/oversight_api.rs)
- Follow-on lane run identifiers in [`src/admin/oversight_follow_on_runs.rs`](../../src/admin/oversight_follow_on_runs.rs)
- Existing Scrapling observer truth in [`src/admin/adversary_sim_worker_plan.rs`](../../src/admin/adversary_sim_worker_plan.rs)

## Root Cause

### 1. Scrapling observer truth still exists

The current live operator snapshot still carries fresh Scrapling recent-run truth:

- explicit `observed_category_ids` such as `ai_scraper_bot`, `automated_browser`, `http_agent`, and `indexing_bot`,
- and non-empty `owned_surface_coverage` receipts for Scrapling runs.

So the regression is not that Scrapling stopped fulfilling labeled roles.

### 2. The Game Loop page now depends on a transient join

The top-of-tab casts are built from:

1. the latest judged round’s `judged_run_ids`, and
2. the current `recent_runs` hot-read buffer.

If a judged run id is missing from the current hot-read buffer, the page refuses to guess, which is correct, but the cast disappears entirely.

### 3. The bounded recent-run buffer is too short-lived to serve as round history

The current recent sim-run projection is capped by contract:

- `HOT_READ_RECENT_SIM_RUNS_MAX_RECORDS = 12` in [`src/observability/hot_read_documents.rs`](../../src/observability/hot_read_documents.rs)

In the live data examined during this review:

- the latest judged round pointed to Scrapling run `simrun-1774783491-238179a74c26d1fa`,
- the current recent-run buffer no longer contained that run id,
- but it did still contain newer Scrapling and Bot Red Team runs.

So the cast disappeared not because the observer truth was missing at write time, but because it was never durably attached to the judged round.

## Measured Payload Pressure

Measured against the live local payloads on 2026-03-29:

- one current Scrapling recent-run summary serialized to roughly `5,543` bytes,
- the full `adversary_sim.recent_runs` array serialized to roughly `26,689` bytes,
- the `operator_snapshot.episode_archive` section serialized to roughly `367,009` bytes,
- the full `/admin/operator-snapshot` response serialized to roughly `488,587` bytes,
- the full `/admin/oversight/history` response serialized to roughly `437,217` bytes.

These measurements matter because the operator-snapshot hot-read contract already aims to stay bounded, and the current snapshot is already large. This rules out “just stuff richer per-round observer payloads into every hot-read episode row” as a clean solution.

## Design Options Considered

### Option A: Raise the recent sim-run cap and keep joining from hot reads

Pros:

- smallest code change,
- no new storage model.

Cons:

- still incorrect in principle because the judged-round view remains hostage to a transient buffer,
- encourages ever-growing hot-read payloads,
- does not guarantee coverage once the round ages out further,
- couples Game Loop history correctness to unrelated recent-run retention tuning.

Verdict: reject.

### Option B: Replay raw event telemetry by `run_id` on every Game Loop read

Pros:

- storage-minimal,
- exact if implemented carefully.

Cons:

- read-expensive,
- pushes event-log replay into the dashboard read path,
- violates the repo’s hot-read/materialized-summary direction,
- makes the page slower and more operationally expensive exactly where the UI wants a simple summary.

Verdict: reject for normal dashboard reads.

### Option C: Persist full observer payloads directly inside the episode archive rows

Pros:

- durable,
- exact,
- simple dashboard join.

Cons:

- directly expands the already-large `episode_archive` embedded inside `operator_snapshot`,
- risks blowing the operator-snapshot hot-read budget,
- duplicates data into the hottest machine-first payload even though only Game Loop history needs it.

Verdict: reject as the primary design.

### Option D: Persist a separate compact judged-round observer archive and read it through `oversight/history`

Pros:

- durable and exact,
- keeps the operator snapshot compact,
- uses the Game Loop’s already-tab-scoped `oversight/history` read path,
- avoids raw event-log replay on normal dashboard refreshes,
- can store only the fields the top-of-tab observer surfaces actually need.

Cons:

- introduces one more bounded archive alongside the episode archive,
- requires a clean join between `episode_id` and the observer rows.

Verdict: recommended.

## Recommended Design

### Write path

Create a separate bounded store keyed by episode id, for example:

- `oversight_observer_round_archive:v1:<site>`

Populate it when a completed judged episode is recorded, while the exact judged run summaries are still present and easy to read.

### Stored payload shape

Store a compact observer-round summary rather than copying the full hot recent-run object:

- `episode_id`
- `completed_at_ts`
- `basis_status`
  - `exact_judged_run_receipts`
  - `partial_missing_run_receipts`
- `missing_run_ids`
- `run_rows`
  - `run_id`
  - `lane`
  - `profile`
  - `observed_fulfillment_modes`
  - `observed_category_ids`
  - `monitoring_event_count`
  - `defense_delta_count`
  - `ban_outcome_count`
- `scrapling_surface_rows`
  - `run_id`
  - `surface_id`
  - `surface_state`
  - `coverage_status`
  - `success_contract`
  - `dependency_kind`
  - `dependency_surface_ids`
  - `attempt_count`
  - `sample_request_method`
  - `sample_request_path`
  - `sample_response_status`

This preserves exact observer truth while avoiding repeated `surface_labels`, `canonical_surface_ids`, `latest_action_receipts`, and other fields the Game Loop top sections do not need.

### Read path

- `operator_snapshot` stays compact and continues to carry the current machine-first archive and hot recent-run state.
- `/admin/oversight/history` becomes the durable source for completed-round observer casts by returning the new observer archive alongside the episode archive.
- The Game Loop top sections render completed rounds from the durable observer archive, not from `operator_snapshot.adversary_sim.recent_runs`.
- Current in-flight evidence, if still shown, may continue to use exact `candidate_window` / `continuation_run` run ids plus current recent-run summaries because that is a live-now surface rather than archived round history.

## Why This Is The Best Read/Storage Trade

- It keeps the frequently refreshed hot snapshot from carrying repeated Scrapling surface receipts for every archived round.
- It writes observer data once, at the natural point where the judged round becomes durable.
- It keeps reads cheap: one tab-scoped history request, no event-log replay.
- It preserves exactness: the observer archive is populated from exact judged run summaries, and missing data is recorded explicitly rather than guessed.

## Non-Goals

- Do not reintroduce lane/time heuristics.
- Do not raise the recent-run cap as the primary fix.
- Do not let runtime defences consume simulator labels.
- Do not solve this by turning the operator snapshot into a larger history transport.

## Recommendation Summary

Implement a separate, compact, bounded observer-round archive keyed by `episode_id`, populate it when judged rounds are recorded, expose it through `/admin/oversight/history`, and make the Game Loop top sections read completed-round casts from that durable archive instead of rehydrating them from the transient recent sim-run hot read.
