use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::adversary_sim_corpus::deterministic_runtime_profile;
pub use super::adversary_sim_diagnostics::{
    generation_diagnostics, status_payload, supervisor_status_payload, LaneDiagnosticsState,
};
pub(crate) use super::adversary_sim_lane_runtime::{
    apply_scrapling_worker_result, run_autonomous_supervisor_ticks,
};
pub use super::adversary_sim_worker_plan::{
    AutonomousHeartbeatTickSummary, GenerationTickResult, ScraplingWorkerPlan,
    ScraplingWorkerResult,
};
#[cfg(test)]
use super::adversary_sim_corpus::{
    DETERMINISTIC_ATTACK_CORPUS, DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION,
};
pub use super::adversary_sim_state::{
    clamp_duration_seconds, control_surface_available, effective_active_lane,
    lane_reconciliation_needed, load_state, process_instance_id, project_effective_desired_state,
    reconcile_state, save_state, select_desired_lane, start_state, stop_state, ControlPhase,
    ControlState, RuntimeLane, StartError, Transition,
};

pub const MAX_CONCURRENT_RUNS: u32 = 1;
pub const MAX_CPU_MILLICORES: u32 = 1000;
pub const MAX_MEMORY_MIB: u32 = 512;
pub const QUEUE_POLICY: &str = "reject_new";
pub const STOP_TIMEOUT_SECONDS: u64 = 10;
pub const AUTONOMOUS_SHARED_SERVER_HEARTBEAT_INTERVAL_SECONDS: u64 = 1;
pub const AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION: u64 = 2;
pub const AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS: u64 = 60;
pub const AUTONOMOUS_EDGE_FERMYON_MAX_CATCHUP_TICKS_PER_INVOCATION: u64 = 1;
pub const AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE: &str =
    "staggered 5x cron set (one run per minute, each job every 5 minutes)";
pub(crate) const PRODUCTION_GENERATION_DEFAULT: &str = "off_until_explicit_enable";
pub const SCRAPLING_WORKER_PLAN_SCHEMA_VERSION: &str = "adversary-sim-scrapling-worker-plan.v1";
pub const SCRAPLING_WORKER_RESULT_SCHEMA_VERSION: &str = "adversary-sim-scrapling-worker-result.v1";
pub const SCRAPLING_SIM_PROFILE: &str = "scrapling_runtime_lane";
pub const SCRAPLING_MAX_REQUESTS_PER_TICK: u64 = 8;
pub const SCRAPLING_MAX_DEPTH_PER_TICK: u64 = 2;
pub const SCRAPLING_MAX_BYTES_PER_TICK: u64 = 262_144;
pub const SCRAPLING_MAX_MS_PER_TICK: u64 = 2_000;

#[cfg(test)]
pub fn state_key(site_id: &str) -> String {
    super::adversary_sim_state::state_key(site_id)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerFailureClass {
    Cancelled,
    Timeout,
    Transport,
    Http,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::adversary_sim_lane_runtime::{
        deterministic_generated_request_target_for_profile,
        deterministic_generated_request_target_for_tick, simulated_request_paths,
        supplemental_lanes_for_profile, FULL_SUPPLEMENTAL_LANES,
    };
    use crate::admin::adversary_sim_state::effective_desired_enabled;
    use crate::test_support::InMemoryStore;

    #[test]
    fn deterministic_attack_corpus_is_loaded_with_required_metadata() {
        let runtime = deterministic_runtime_profile();
        assert_eq!(
            DETERMINISTIC_ATTACK_CORPUS.schema_version,
            DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION
        );
        assert!(!DETERMINISTIC_ATTACK_CORPUS.corpus_revision.trim().is_empty());
        assert!(!DETERMINISTIC_ATTACK_CORPUS.taxonomy_version.trim().is_empty());
        assert!(runtime.active_lane_count >= 1);
        assert!(!runtime.primary_public_paths.is_empty());
        assert!(runtime.rate_burst.low > 0);
        assert!(!DETERMINISTIC_ATTACK_CORPUS.ci_oracle.drivers.is_empty());
    }

    #[test]
    fn start_and_stop_transitions_reach_off_state() {
        let now = 1_000u64;
        let (started, started_transitions) =
            start_state(now, 180, &ControlState::default()).expect("start");
        assert_eq!(started.phase, ControlPhase::Running);
        assert_eq!(started.active_run_count, 1);
        assert_eq!(started.active_lane_count, 2);
        assert_eq!(started_transitions.len(), 1);
        assert_eq!(started_transitions[0].reason, "manual_on");

        let (stopping, stop_transitions) = stop_state(now + 1, "manual_off", &started);
        assert_eq!(stopping.phase, ControlPhase::Stopping);
        assert_eq!(stopping.active_run_count, 0);
        assert_eq!(stopping.active_lane_count, 0);
        assert_eq!(stop_transitions.len(), 1);

        let (off, reconcile_transitions) = reconcile_state(now + 1, false, &stopping);
        assert_eq!(off.phase, ControlPhase::Off);
        assert_eq!(off.active_run_count, 0);
        assert_eq!(off.active_lane_count, 0);
        assert_eq!(reconcile_transitions.len(), 1);
        assert_eq!(reconcile_transitions[0].to, ControlPhase::Off);
    }

    #[test]
    fn start_and_stop_transitions_track_additive_lane_contract() {
        let now = 1_000u64;
        let (started, _) = start_state(now, 180, &ControlState::default()).expect("start");
        assert_eq!(started.desired_lane.as_str(), "synthetic_traffic");
        assert_eq!(
            started.active_lane.map(RuntimeLane::as_str),
            Some("synthetic_traffic")
        );
        assert_eq!(started.lane_switch_seq, 0);
        assert_eq!(started.last_lane_switch_at, None);
        assert_eq!(started.last_lane_switch_reason, None);

        let (stopping, _) = stop_state(now + 1, "manual_off", &started);
        assert_eq!(stopping.desired_lane.as_str(), "synthetic_traffic");
        assert_eq!(stopping.active_lane, None);
        assert_eq!(stopping.lane_switch_seq, 0);
        assert_eq!(stopping.last_lane_switch_at, None);
        assert_eq!(stopping.last_lane_switch_reason, None);
    }

    #[test]
    fn reconcile_expired_window_stops_and_turns_off() {
        let state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-expired".to_string()),
            started_at: Some(100),
            ends_at: Some(120),
            stop_deadline: None,
            active_run_count: 1,
            active_lane_count: 2,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 100,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(121, true, &state);
        assert_eq!(next.phase, ControlPhase::Off);
        assert_eq!(next.last_transition_reason.as_deref(), Some("auto_window_expired"));
        assert_eq!(next.last_run_id.as_deref(), Some("run-expired"));
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].to, ControlPhase::Stopping);
        assert_eq!(transitions[1].to, ControlPhase::Off);
    }

    #[test]
    fn forced_kill_timeout_transitions_to_safe_off_state() {
        let state = ControlState {
            phase: ControlPhase::Stopping,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-stuck".to_string()),
            started_at: Some(100),
            ends_at: Some(120),
            stop_deadline: Some(140),
            active_run_count: 1,
            active_lane_count: 1,
            last_transition_reason: Some("manual_off".to_string()),
            updated_at: 130,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(141, false, &state);
        assert_eq!(next.phase, ControlPhase::Off);
        assert_eq!(next.active_run_count, 0);
        assert_eq!(next.active_lane_count, 0);
        assert_eq!(
            next.last_terminal_failure_reason.as_deref(),
            Some("forced_kill_timeout")
        );
        assert_eq!(transitions.len(), 1);
        assert_eq!(transitions[0].reason, "forced_kill_timeout");
    }

    #[test]
    fn start_rejects_queue_full_when_run_is_active() {
        let state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-active".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            stop_deadline: None,
            active_run_count: MAX_CONCURRENT_RUNS,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 100,
            ..ControlState::default()
        };

        let result = start_state(150, 180, &state);
        assert_eq!(result, Err(StartError::QueueFull));
    }

    #[test]
    fn autonomous_supervisor_runs_initial_tick_when_running_without_history() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-supervisor".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };
        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 110);
        assert_eq!(summary.due_ticks, 1);
        assert_eq!(summary.executed_ticks, 1);
        assert_eq!(state.generated_tick_count, 1);
        assert_eq!(
            state.generated_request_count,
            deterministic_generated_request_target_for_tick(0)
        );
        assert_eq!(state.last_generated_at, Some(110));
        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn autonomous_supervisor_caps_catchup_ticks_per_invocation() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "shared-server");
        let store = InMemoryStore::default();
        let mut state = ControlState {
            phase: ControlPhase::Running,
            owner_instance_id: Some(process_instance_id().to_string()),
            run_id: Some("run-catchup".to_string()),
            started_at: Some(10),
            ends_at: Some(1000),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_generated_at: Some(10),
            ..ControlState::default()
        };
        let summary = run_autonomous_supervisor_ticks(&store, &mut state, 200);
        assert_eq!(
            summary.executed_ticks,
            AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
        assert_eq!(
            state.generated_tick_count,
            AUTONOMOUS_SHARED_SERVER_MAX_CATCHUP_TICKS_PER_INVOCATION
        );
        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn supervisor_status_payload_reports_off_state_inert_contract() {
        let state = ControlState::default();
        let payload = supervisor_status_payload(100, false, &state);
        assert_eq!(
            payload
                .get("heartbeat_active")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            payload
                .get("off_state_inert")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("deterministic_attack_corpus")
                .and_then(|value| value.get("schema_version"))
                .and_then(|value| value.as_str()),
            Some(DETERMINISTIC_ATTACK_CORPUS_SCHEMA_VERSION)
        );
    }

    #[test]
    fn supervisor_status_payload_reports_edge_cron_truthfully_before_first_tick() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let payload = supervisor_status_payload(130, true, &state);
        assert_eq!(
            payload
                .get("heartbeat_expected")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("heartbeat_active")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            payload
                .get("trigger_surface")
                .and_then(|value| value.as_str()),
            Some("edge_cron")
        );
        assert_eq!(
            payload
                .get("cadence_seconds")
                .and_then(|value| value.as_u64()),
            Some(AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS)
        );
        assert_eq!(
            payload
                .get("cron_schedule")
                .and_then(|value| value.as_str()),
            Some(AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE)
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn generation_diagnostics_waits_full_edge_interval_before_no_traffic() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let warming = generation_diagnostics(130, true, &state);
        assert_eq!(warming.health, "warming");
        assert_eq!(warming.reason, "waiting_for_first_edge_cron_tick");

        let no_traffic = generation_diagnostics(170, true, &state);
        assert_eq!(no_traffic.health, "no_traffic");
        assert_eq!(no_traffic.reason, "edge_cron_no_traffic_yet");

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn status_payload_surfaces_explicit_production_operating_envelope() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");

        let payload = status_payload(
            100,
            crate::config::RuntimeEnvironment::RuntimeProd,
            true,
            false,
            180,
            &ControlState::default(),
        );
        assert_eq!(
            payload
                .get("gateway_deployment_profile")
                .and_then(|value| value.as_str()),
            Some("edge-fermyon")
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("surface_available_by_default"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("generation_default"))
                .and_then(|value| value.as_str()),
            Some(PRODUCTION_GENERATION_DEFAULT)
        );
        assert_eq!(
            payload
                .get("guardrails")
                .and_then(|value| value.get("generation_requires_explicit_enable"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );

        let supervisor = supervisor_status_payload(100, false, &ControlState::default());
        assert_eq!(
            supervisor
                .get("deployment_profile")
                .and_then(|value| value.as_str()),
            Some("edge-fermyon")
        );
        assert_eq!(
            supervisor
                .get("trigger_surface")
                .and_then(|value| value.as_str()),
            Some("edge_cron")
        );
        assert_eq!(
            supervisor
                .get("cadence_seconds")
                .and_then(|value| value.as_u64()),
            Some(AUTONOMOUS_EDGE_FERMYON_HEARTBEAT_INTERVAL_SECONDS)
        );
        assert_eq!(
            supervisor
                .get("cron_schedule")
                .and_then(|value| value.as_str()),
            Some(AUTONOMOUS_EDGE_FERMYON_CRON_SCHEDULE)
        );

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn status_payload_exposes_additive_lane_migration_contract() {
        let state = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            run_id: Some("run-lane-contract".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            ..ControlState::default()
        };

        let payload = status_payload(
            150,
            crate::config::RuntimeEnvironment::RuntimeProd,
            true,
            true,
            180,
            &state,
        );
        assert_eq!(
            payload
                .get("desired_lane")
                .and_then(|value| value.as_str()),
            Some("synthetic_traffic")
        );
        assert_eq!(
            payload
                .get("active_lane")
                .and_then(|value| value.as_str()),
            Some("synthetic_traffic")
        );
        assert_eq!(
            payload
                .get("lane_switch_seq")
                .and_then(|value| value.as_u64()),
            Some(0)
        );
        assert_eq!(payload.get("last_lane_switch_at"), Some(&serde_json::Value::Null));
        assert_eq!(
            payload.get("last_lane_switch_reason"),
            Some(&serde_json::Value::Null)
        );
        assert_eq!(
            payload
                .get("lane_diagnostics")
                .and_then(|value| value.get("lanes"))
                .and_then(|value| value.get("scrapling_traffic"))
                .and_then(|value| value.get("beat_attempts"))
                .and_then(|value| value.as_u64()),
            Some(0)
        );
        assert_eq!(
            payload
                .get("lane_diagnostics")
                .and_then(|value| value.get("request_failure_classes"))
                .and_then(|value| value.get("cancelled"))
                .and_then(|value| value.get("count"))
                .and_then(|value| value.as_u64()),
            Some(0)
        );
    }

    #[test]
    fn effective_desired_enabled_uses_seed_before_first_control_write() {
        assert!(effective_desired_enabled(true, &ControlState::default()));
        assert!(!effective_desired_enabled(false, &ControlState::default()));
    }

    #[test]
    fn effective_desired_enabled_prefers_persisted_lifecycle_state_after_control_write() {
        let state = ControlState {
            desired_enabled: false,
            updated_at: 100,
            ..ControlState::default()
        };

        assert!(!effective_desired_enabled(true, &state));
    }

    #[test]
    fn reconcile_state_keeps_edge_runs_active_across_instance_changes() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");

        let current = ControlState {
            phase: ControlPhase::Running,
            desired_enabled: true,
            owner_instance_id: Some("simproc-other".to_string()),
            run_id: Some("run-edge".to_string()),
            started_at: Some(100),
            ends_at: Some(400),
            active_run_count: 1,
            active_lane_count: deterministic_runtime_profile().active_lane_count,
            last_transition_reason: Some("manual_on".to_string()),
            updated_at: 110,
            ..ControlState::default()
        };

        let (next, transitions) = reconcile_state(130, true, &current);
        assert_eq!(next.phase, ControlPhase::Running);
        assert!(transitions.is_empty());
        assert!(next.desired_enabled);
        assert_eq!(next.owner_instance_id.as_deref(), Some("simproc-other"));

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn deterministic_request_targets_cover_key_defense_surfaces() {
        let runtime_profile = deterministic_runtime_profile();
        let without_honeypot = simulated_request_paths("run-coverage", 1);
        assert!(
            without_honeypot
                .iter()
                .any(|path| path == runtime_profile.paths.pow.as_str())
        );
        assert!(without_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.not_a_bot_checkbox.as_str()));
        assert!(!without_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.honeypot.as_str()));
        assert!(without_honeypot
            .iter()
            .any(|path| path.starts_with(runtime_profile.paths.public_search.as_str())));
        assert!(without_honeypot
            .iter()
            .any(|path| path.starts_with(crate::maze::entry_path("").as_str())));

        let with_honeypot = simulated_request_paths("run-coverage", 5);
        assert!(with_honeypot
            .iter()
            .any(|path| path == runtime_profile.paths.honeypot.as_str()));
    }

    #[test]
    fn shared_server_generated_request_target_matches_batch_contract() {
        let runtime_profile = deterministic_runtime_profile();
        let burst = &runtime_profile.rate_burst;
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                0,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.high
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                1,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.low
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::SharedServer,
                3,
            ),
            runtime_profile.primary_request_count
                + runtime_profile.supplemental_request_count
                + burst.medium
        );
    }

    #[test]
    fn edge_fermyon_generated_request_target_stays_within_bounded_budget() {
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                0,
            ),
            6
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                1,
            ),
            4
        );
        assert_eq!(
            deterministic_generated_request_target_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                3,
            ),
            5
        );
    }

    #[test]
    fn edge_fermyon_supplemental_lane_rotation_covers_full_contract() {
        let mut observed = std::collections::BTreeSet::new();
        for tick in 0..FULL_SUPPLEMENTAL_LANES.len() as u64 {
            for lane in supplemental_lanes_for_profile(
                crate::config::GatewayDeploymentProfile::EdgeFermyon,
                tick,
            ) {
                observed.insert(lane);
            }
        }

        let expected = std::collections::BTreeSet::from(FULL_SUPPLEMENTAL_LANES);
        assert_eq!(observed, expected);
    }
}
