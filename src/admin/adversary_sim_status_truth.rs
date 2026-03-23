use crate::challenge::KeyValueStore;
use serde_json::json;

const CONTROL_STATE_TRUTH_BASIS: &str = "control_state";
const PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS: &str = "persisted_event_lower_bound";

pub(crate) struct AdversarySimStatusTruthProjection {
    pub(crate) projected_state: crate::admin::adversary_sim::ControlState,
    pub(crate) generation_truth_basis: &'static str,
    pub(crate) lane_diagnostics_truth_basis: &'static str,
    pub(crate) persisted_event_evidence: Option<serde_json::Value>,
}

pub(crate) fn project_status_truth<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    now: u64,
    state: &crate::admin::adversary_sim::ControlState,
) -> AdversarySimStatusTruthProjection {
    let mut projection = AdversarySimStatusTruthProjection {
        projected_state: state.clone(),
        generation_truth_basis: CONTROL_STATE_TRUTH_BASIS,
        lane_diagnostics_truth_basis: CONTROL_STATE_TRUTH_BASIS,
        persisted_event_evidence: None,
    };

    let Some(run_id) = state
        .run_id
        .as_deref()
        .or(state.last_run_id.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return projection;
    };

    let recent_runs = crate::observability::hot_read_projection::load_monitoring_recent_sim_runs_hot_read(
        store,
        site_id,
        now,
    );
    let Some(run) = recent_runs
        .payload
        .recent_sim_runs
        .iter()
        .find(|row| row.run_id == run_id && row.monitoring_event_count > 0)
    else {
        return projection;
    };

    let observed_request_lower_bound = run.monitoring_event_count;
    let observed_last_generated_at = run.last_ts;

    if projection.projected_state.generated_request_count < observed_request_lower_bound {
        projection.projected_state.generated_request_count = observed_request_lower_bound;
        projection.generation_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
    }
    if projection.projected_state.generated_tick_count == 0 {
        projection.projected_state.generated_tick_count = 1;
        projection.generation_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
    }
    if projection
        .projected_state
        .last_generated_at
        .unwrap_or(0)
        < observed_last_generated_at
    {
        projection.projected_state.last_generated_at = Some(observed_last_generated_at);
        projection.generation_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
    }

    if let Some(runtime_lane) = status_runtime_lane(state) {
        let counters = projection.projected_state.lane_diagnostics.lane_mut(runtime_lane);
        if counters.generated_requests < observed_request_lower_bound {
            counters.generated_requests = observed_request_lower_bound;
            projection.lane_diagnostics_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
        }
        if counters.beat_successes == 0 {
            counters.beat_successes = 1;
            projection.lane_diagnostics_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
        }
        if counters.beat_attempts < counters.beat_successes {
            counters.beat_attempts = counters.beat_successes;
            projection.lane_diagnostics_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
        }
        if counters.last_generated_at.unwrap_or(0) < observed_last_generated_at {
            counters.last_generated_at = Some(observed_last_generated_at);
            projection.lane_diagnostics_truth_basis = PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS;
        }
    }

    projection.persisted_event_evidence = Some(json!({
        "run_id": run.run_id,
        "lane": run.lane,
        "profile": run.profile,
        "monitoring_event_count": run.monitoring_event_count,
        "defense_delta_count": run.defense_delta_count,
        "ban_outcome_count": run.ban_outcome_count,
        "first_observed_at": run.first_ts,
        "last_observed_at": run.last_ts,
        "truth_basis": PERSISTED_EVENT_LOWER_BOUND_TRUTH_BASIS
    }));

    projection
}

fn status_runtime_lane(
    state: &crate::admin::adversary_sim::ControlState,
) -> Option<crate::admin::adversary_sim::RuntimeLane> {
    crate::admin::adversary_sim::effective_active_lane(state).or_else(|| {
        if state.run_id.is_some() || state.last_run_id.is_some() {
            Some(state.desired_lane)
        } else {
            None
        }
    })
}

