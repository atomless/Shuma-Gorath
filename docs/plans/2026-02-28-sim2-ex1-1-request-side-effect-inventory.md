# SIM2-EX1-1 Request-Path Side-Effect Inventory

Date: 2026-02-28  
Status: Completed

Reference plans:

- [`docs/plans/2026-02-27-sim2-orchestration-capability-architecture-plan.md`](./2026-02-27-sim2-orchestration-capability-architecture-plan.md)
- [`docs/plans/2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md`](./2026-02-27-sim2-shortfall-1-black-box-capability-enforcement-plan.md)

## Scope and Method

Inventory scope: remaining direct request-path writes for:

1. `metrics`
2. `monitoring`
3. `event log`
4. `ban writes`

Scanned files:

1. `src/lib.rs`
2. `src/runtime/request_router.rs`
3. `src/runtime/policy_pipeline.rs`
4. `src/runtime/effect_intents.rs`
5. `src/runtime/shadow_mode/mod.rs`

Pattern family used for inventory:

1. `admin::log_event`
2. `observability::metrics::increment` and `observability::metrics::record_*`
3. `observability::monitoring::record_*`, `observability::monitoring::maybe_record_*`, `observability::monitoring::flush_pending_counters`
4. `ban_store_provider().ban_ip_with_fingerprint` and `ban::ban_ip_with_fingerprint`

## Callsite Summary

| File | Direct callsites (inventory pattern set) | Classification headline |
| --- | ---: | --- |
| `src/lib.rs` | 39 | `migrate` (except one operational epilogue `retain`) |
| `src/runtime/request_router.rs` | 41 | `migrate` |
| `src/runtime/policy_pipeline.rs` | 97 | `delete` (legacy dead-code handlers) + one `migrate` in active tranche path |
| `src/runtime/effect_intents.rs` | 8 | `retain` (canonical executor boundary) |
| `src/runtime/shadow_mode/mod.rs` | 1 | `migrate` |

## Classified Inventory

### `retain`

1. `src/runtime/effect_intents.rs:88-190`, `src/runtime/effect_intents.rs:963-1191`
   - Direct side-effect writes remain by design in the effect executor (`apply_metric_intent`, `apply_monitoring_intent`, `apply_event_log_intent`, `apply_ban_intent`).
   - Rationale: this is the intended imperative boundary for typed intents.
2. `src/lib.rs:1155-1160`
   - `spin_entrypoint` post-response `flush_pending_counters`.
   - Rationale: operational drain/flush epilogue, not policy decision logic.

### `migrate`

1. `src/lib.rs:644-905` (`serve_maze_with_tracking`)
   - Direct writes for maze metrics, event log, and maze-threshold ban write.
   - Target: route through typed effect intents or a dedicated maze effect executor invoked by response intent.
2. `src/lib.rs:946-1150` (`handle_bot_defence_impl`)
   - Orchestration shell still writes context/request/allow metrics and monitoring sample signals directly.
   - Target: pre-tranche and post-tranche intents so `src/lib.rs` stays routing/trust-boundary only.
3. `src/runtime/request_router.rs:98-158` (`enforce_tarpit_or_short_ban`)
   - Direct ban write + metrics + event log.
   - Target: request-router intent plan and capability-gated executor path.
4. `src/runtime/request_router.rs:160-179` (`record_sequence_violation_for_challenge_submit`)
   - Direct policy match + event log.
   - Target: challenge-submit intent plan.
5. `src/runtime/request_router.rs:181-363` (`handle_not_a_bot_submit`)
   - Direct monitoring submit record + metrics + event log across outcomes.
   - Target: typed challenge outcome intents.
6. `src/runtime/request_router.rs:440-750` (`maybe_handle_early_route` challenge handlers)
   - Direct challenge outcome metrics/monitoring/event log writes.
   - Target: move POST/GET challenge route outcome effects to intent executor.
7. `src/runtime/request_router.rs:763-776` (`/robots.txt` branch)
   - Direct request metric write.
   - Target: route-level intent for non-policy request accounting.
8. `src/runtime/policy_pipeline.rs:1385-1464` (`maybe_handle_policy_graph_second_tranche`)
   - Active path still writes `record_botness_visibility` before decision execution.
   - Target: explicit pre-decision effect intent or dedicated telemetry stage.
9. `src/runtime/shadow_mode/mod.rs:3-23` (`log_shadow_mode_event`)
   - Direct event log write in shadow-mode helper.
   - Target: shadow-mode effect intent adapter using same capability boundary.

### `delete`

Dead-code policy handlers in `src/runtime/policy_pipeline.rs` (`#[allow(dead_code)]`) still contain direct side-effect writes and are no longer request-path authorities:

1. `src/runtime/policy_pipeline.rs:29-509` (`maybe_handle_ip_range_policy`)
2. `src/runtime/policy_pipeline.rs:511-579` (`maybe_handle_honeypot`)
3. `src/runtime/policy_pipeline.rs:581-655` (`maybe_handle_rate_limit`)
4. `src/runtime/policy_pipeline.rs:657-697` (`maybe_handle_existing_ban`)
5. `src/runtime/policy_pipeline.rs:699-980` (`maybe_handle_geo_policy`)
6. `src/runtime/policy_pipeline.rs:1006-1254` (`maybe_handle_botness`)
7. `src/runtime/policy_pipeline.rs:1256-1302` (`maybe_handle_js`)

Supporting dead-code helper seams to remove with the above:

1. `src/runtime/policy_pipeline.rs:17-27` (`ip_range_signal_ids`, `ip_range_source_label`)

## Execution Order Derived From Inventory

1. EX1-2 should migrate `src/lib.rs` side-effect writes first (largest active orchestration seam).
2. EX1-4 should then remove the dead-code policy handlers in `policy_pipeline` as a separate atomic slice.
3. EX1-3 should split `effect_intents` only after active migrations land, so module extraction is mechanical and low-risk.

## Security and Resource Notes

1. Migration reduces convention-based privileged writes from orchestration entrypoints and narrows trusted write surfaces.
2. Deleting dead-code policy handlers removes duplicate side-effect paths and reduces accidental regression risk.
3. Keeping `effect_intents` as the write boundary preserves explicit capability gating and traceability.
