use serde::{Deserialize, Serialize};

use crate::config::Config;

pub(crate) const OVERSIGHT_REQUIRED_RUN_STATUS_PENDING: &str = "pending";
pub(crate) const OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING: &str = "running";
pub(crate) const OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED: &str = "materialized";
pub(crate) const OVERSIGHT_REQUIRED_RUN_STATUS_EXPIRED: &str = "expired";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightRequiredLaneRun {
    pub lane: crate::admin::adversary_sim::RuntimeLane,
    pub status: String,
    pub requested_at_ts: u64,
    pub requested_duration_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_on_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_on_started_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub materialized_at_ts: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RequiredRunMaterialization {
    pub matched: bool,
    pub all_materialized: bool,
    pub has_pending: bool,
}

pub(crate) fn default_required_lane_runs(
    cfg: &Config,
    requested_at_ts: u64,
) -> Vec<OversightRequiredLaneRun> {
    let mut lanes = vec![crate::admin::adversary_sim::RuntimeLane::ScraplingTraffic];
    if crate::config::frontier_summary().provider_count > 0 {
        lanes.push(crate::admin::adversary_sim::RuntimeLane::BotRedTeam);
    }
    lanes
        .into_iter()
        .map(|lane| OversightRequiredLaneRun {
            lane,
            status: OVERSIGHT_REQUIRED_RUN_STATUS_PENDING.to_string(),
            requested_at_ts,
            requested_duration_seconds: follow_on_duration_seconds_for_lane(cfg, lane),
            follow_on_run_id: None,
            follow_on_started_at: None,
            materialized_at_ts: None,
        })
        .collect()
}

pub(crate) fn follow_on_duration_seconds_for_lane(
    cfg: &Config,
    lane: crate::admin::adversary_sim::RuntimeLane,
) -> u64 {
    let bounded =
        crate::admin::adversary_sim::clamp_duration_seconds(cfg.adversary_sim_duration_seconds);
    match lane {
        crate::admin::adversary_sim::RuntimeLane::ScraplingTraffic => {
            if crate::config::runtime_environment().is_dev() {
                bounded.min(crate::config::ADVERSARY_SIM_DURATION_SECONDS_MIN)
            } else {
                bounded
            }
        }
        crate::admin::adversary_sim::RuntimeLane::BotRedTeam => bounded.max(
            crate::admin::adversary_sim_llm_lane::minimum_meaningful_llm_window_seconds(),
        ),
        crate::admin::adversary_sim::RuntimeLane::SyntheticTraffic => bounded,
    }
}

pub(crate) fn overall_status(
    required_runs: &[OversightRequiredLaneRun],
    not_requested_status: &str,
    materialized_status: &str,
) -> String {
    if required_runs.is_empty() {
        return not_requested_status.to_string();
    }
    if required_runs
        .iter()
        .all(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED)
    {
        return materialized_status.to_string();
    }
    if required_runs
        .iter()
        .any(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING)
    {
        return OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING.to_string();
    }
    if required_runs
        .iter()
        .any(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_PENDING)
    {
        return OVERSIGHT_REQUIRED_RUN_STATUS_PENDING.to_string();
    }
    not_requested_status.to_string()
}

pub(crate) fn project_with_expiration(
    required_runs: &[OversightRequiredLaneRun],
    expired: bool,
) -> Vec<OversightRequiredLaneRun> {
    if !expired {
        return required_runs.to_vec();
    }
    required_runs
        .iter()
        .map(|run| {
            let mut projected = run.clone();
            if matches!(
                projected.status.as_str(),
                OVERSIGHT_REQUIRED_RUN_STATUS_PENDING | OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING
            ) {
                projected.status = OVERSIGHT_REQUIRED_RUN_STATUS_EXPIRED.to_string();
            }
            projected
        })
        .collect()
}

pub(crate) fn current_focus_run(
    required_runs: &[OversightRequiredLaneRun],
) -> Option<&OversightRequiredLaneRun> {
    required_runs
        .iter()
        .find(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING)
        .or_else(|| {
            required_runs
                .iter()
                .find(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_PENDING)
        })
        .or_else(|| required_runs.iter().rev().find(|run| run.materialized_at_ts.is_some()))
        .or_else(|| required_runs.last())
}

pub(crate) fn start_next_pending_run(
    required_runs: &mut [OversightRequiredLaneRun],
    now: u64,
    run_id: &str,
) -> Option<OversightRequiredLaneRun> {
    let next = required_runs
        .iter_mut()
        .find(|run| run.status == OVERSIGHT_REQUIRED_RUN_STATUS_PENDING)?;
    next.status = OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING.to_string();
    next.follow_on_run_id = Some(run_id.to_string());
    next.follow_on_started_at = Some(now);
    Some(next.clone())
}

pub(crate) fn mark_run_materialized(
    required_runs: &mut [OversightRequiredLaneRun],
    sim_run_id: &str,
    completed_at_ts: u64,
) -> RequiredRunMaterialization {
    let Some(run) = required_runs.iter_mut().find(|run| {
        run.status == OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING
            && run.follow_on_run_id.as_deref() == Some(sim_run_id)
    }) else {
        return RequiredRunMaterialization {
            matched: false,
            all_materialized: false,
            has_pending: required_runs
                .iter()
                .any(|candidate| candidate.status == OVERSIGHT_REQUIRED_RUN_STATUS_PENDING),
        };
    };
    run.status = OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED.to_string();
    run.materialized_at_ts = Some(completed_at_ts);
    let has_pending = required_runs
        .iter()
        .any(|candidate| candidate.status == OVERSIGHT_REQUIRED_RUN_STATUS_PENDING);
    let all_materialized = required_runs
        .iter()
        .all(|candidate| candidate.status == OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED);
    RequiredRunMaterialization {
        matched: true,
        all_materialized,
        has_pending,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_required_lane_runs, follow_on_duration_seconds_for_lane, mark_run_materialized,
        start_next_pending_run, OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED,
        OVERSIGHT_REQUIRED_RUN_STATUS_PENDING, OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING,
    };

    #[test]
    fn default_required_lane_runs_adds_bot_red_team_when_frontier_is_configured() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "frontier-key");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");

        let runs = default_required_lane_runs(crate::config::defaults(), 1_700_000_000);

        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].lane.as_str(), "scrapling_traffic");
        assert_eq!(runs[0].status, OVERSIGHT_REQUIRED_RUN_STATUS_PENDING);
        assert_eq!(runs[1].lane.as_str(), "bot_red_team");
        assert_eq!(runs[1].status, OVERSIGHT_REQUIRED_RUN_STATUS_PENDING);
        assert_eq!(runs[0].requested_duration_seconds, 30);
        assert!(runs[1].requested_duration_seconds >= 120);

        std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
    }

    #[test]
    fn mark_run_materialized_advances_pending_follow_on_sequence() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FRONTIER_OPENAI_API_KEY", "frontier-key");
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");

        let mut runs = default_required_lane_runs(crate::config::defaults(), 1_700_000_000);
        let started = start_next_pending_run(&mut runs, 1_700_000_010, "simrun-001")
            .expect("pending run");
        assert_eq!(started.status, OVERSIGHT_REQUIRED_RUN_STATUS_RUNNING);
        let first = mark_run_materialized(&mut runs, "simrun-001", 1_700_000_011);
        assert!(first.matched);
        assert!(!first.all_materialized);
        assert!(first.has_pending);
        assert_eq!(runs[0].status, OVERSIGHT_REQUIRED_RUN_STATUS_MATERIALIZED);
        assert_eq!(runs[1].status, OVERSIGHT_REQUIRED_RUN_STATUS_PENDING);

        let _second_started = start_next_pending_run(&mut runs, 1_700_000_012, "simrun-002")
            .expect("second pending run");
        let second = mark_run_materialized(&mut runs, "simrun-002", 1_700_000_013);
        assert!(second.matched);
        assert!(second.all_materialized);
        assert!(!second.has_pending);

        std::env::remove_var("SHUMA_FRONTIER_OPENAI_API_KEY");
        std::env::remove_var("SHUMA_RUNTIME_ENV");
    }

    #[test]
    fn llm_follow_on_duration_honors_meaningful_minimum() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_RUNTIME_ENV", "runtime-dev");
        let duration = follow_on_duration_seconds_for_lane(
            crate::config::defaults(),
            crate::admin::adversary_sim::RuntimeLane::BotRedTeam,
        );
        assert!(duration >= 120);
        std::env::remove_var("SHUMA_RUNTIME_ENV");
    }
}
