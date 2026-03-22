use crate::challenge::KeyValueStore;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::oversight_api::OversightExecutionPayload;
use super::oversight_decision_ledger::{load_latest_decision, OversightDecisionRecord};

pub(crate) const OVERSIGHT_AGENT_RUN_SCHEMA_VERSION: &str = "oversight_agent_run_v1";
pub(crate) const OVERSIGHT_AGENT_EXECUTION_SCHEMA_VERSION: &str = "oversight_agent_execution_v1";
pub(crate) const OVERSIGHT_AGENT_STATUS_SCHEMA_VERSION: &str = "oversight_agent_status_v1";
pub(crate) const OVERSIGHT_AGENT_DEFAULT_INTERVAL_SECONDS: u64 = 300;
pub(crate) const OVERSIGHT_AGENT_INTERNAL_PATH: &str = "/internal/oversight/agent/run";

const OVERSIGHT_AGENT_HISTORY_PREFIX: &str = "oversight_agent_runs:v1";
const OVERSIGHT_AGENT_HISTORY_LIMIT: usize = 12;
const OVERSIGHT_AGENT_LEASE_PREFIX: &str = "oversight_agent:lease:v1:";

pub(crate) fn shared_host_execution_available() -> bool {
    matches!(
        crate::config::gateway_deployment_profile(),
        crate::config::GatewayDeploymentProfile::SharedServer
    )
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OversightAgentTriggerKind {
    PeriodicSupervisor,
    PostAdversarySim,
}

impl OversightAgentTriggerKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::PeriodicSupervisor => "periodic_supervisor",
            Self::PostAdversarySim => "post_adversary_sim",
        }
    }

    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "periodic_supervisor" => Some(Self::PeriodicSupervisor),
            "post_adversary_sim" => Some(Self::PostAdversarySim),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightAgentTrigger {
    pub kind: OversightAgentTriggerKind,
    pub requested_at_ts: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_completion_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightAgentRunRecord {
    pub schema_version: String,
    pub run_id: String,
    pub trigger_kind: String,
    pub requested_at_ts: u64,
    pub started_at_ts: u64,
    pub completed_at_ts: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sim_completion_reason: Option<String>,
    pub execution: OversightExecutionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightAgentExecutionResult {
    pub schema_version: String,
    pub status: String,
    pub replayed: bool,
    pub run: OversightAgentRunRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightAgentPeriodicTriggerContract {
    pub surface: String,
    pub wrapper_command: String,
    pub default_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightAgentPostSimTriggerContract {
    pub surface: String,
    pub qualifying_completion: String,
    pub dedupe_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightAgentStatusPayload {
    pub schema_version: String,
    pub execution_boundary: String,
    pub periodic_trigger: OversightAgentPeriodicTriggerContract,
    pub post_sim_trigger: OversightAgentPostSimTriggerContract,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_run: Option<OversightAgentRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_decision: Option<OversightDecisionRecord>,
    pub recent_runs: Vec<OversightAgentRunRecord>,
}

fn history_index_key(site_id: &str) -> String {
    format!("{}:{}:index", OVERSIGHT_AGENT_HISTORY_PREFIX, site_id)
}

fn history_run_key(site_id: &str, run_id: &str) -> String {
    format!("{}:{}:{}", OVERSIGHT_AGENT_HISTORY_PREFIX, site_id, run_id)
}

fn lease_key(site_id: &str) -> String {
    format!("{}{}", OVERSIGHT_AGENT_LEASE_PREFIX, site_id)
}

fn load_run_index<S: KeyValueStore>(store: &S, site_id: &str) -> Vec<String> {
    store
        .get(&history_index_key(site_id))
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_slice::<Vec<String>>(&raw).ok())
        .unwrap_or_default()
}

fn save_run_index<S: KeyValueStore>(store: &S, site_id: &str, run_ids: &[String]) -> Result<(), ()> {
    let encoded = serde_json::to_vec(run_ids).map_err(|_| ())?;
    store.set(&history_index_key(site_id), &encoded)
}

fn load_run_record<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    run_id: &str,
) -> Option<OversightAgentRunRecord> {
    let raw = store.get(&history_run_key(site_id, run_id)).ok().flatten()?;
    serde_json::from_slice::<OversightAgentRunRecord>(&raw).ok()
}

fn persist_run_record<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    run: &OversightAgentRunRecord,
) -> Result<(), ()> {
    let encoded = serde_json::to_vec(run).map_err(|_| ())?;
    store.set(&history_run_key(site_id, &run.run_id), &encoded)?;
    let mut run_ids = load_run_index(store, site_id);
    run_ids.retain(|candidate| candidate != &run.run_id);
    run_ids.insert(0, run.run_id.clone());
    if run_ids.len() > OVERSIGHT_AGENT_HISTORY_LIMIT {
        for stale_id in run_ids.drain(OVERSIGHT_AGENT_HISTORY_LIMIT..) {
            let _ = store.delete(&history_run_key(site_id, &stale_id));
        }
    }
    save_run_index(store, site_id, &run_ids)
}

fn load_agent_lease<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<crate::admin::adversary_sim_control::ControllerLease> {
    let raw = store.get(&lease_key(site_id)).ok().flatten()?;
    serde_json::from_slice::<crate::admin::adversary_sim_control::ControllerLease>(&raw).ok()
}

fn save_agent_lease<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    lease: &crate::admin::adversary_sim_control::ControllerLease,
) -> Result<(), ()> {
    let encoded = serde_json::to_vec(lease).map_err(|_| ())?;
    store.set(&lease_key(site_id), &encoded)
}

fn release_agent_lease<S: KeyValueStore>(store: &S, site_id: &str) {
    let _ = store.delete(&lease_key(site_id));
}

fn run_id(trigger: &OversightAgentTrigger) -> String {
    let ts = if trigger.requested_at_ts == 0 {
        crate::admin::now_ts()
    } else {
        trigger.requested_at_ts
    };
    let mut hasher = DefaultHasher::new();
    trigger.kind.hash(&mut hasher);
    trigger.requested_at_ts.hash(&mut hasher);
    trigger.sim_run_id.hash(&mut hasher);
    trigger.sim_completion_reason.hash(&mut hasher);
    format!("ovragent-{}-{:016x}", ts, hasher.finish())
}

fn matching_post_sim_run<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    sim_run_id: &str,
) -> Option<OversightAgentRunRecord> {
    load_recent_agent_runs(store, site_id).into_iter().find(|run| {
        run.trigger_kind == OversightAgentTriggerKind::PostAdversarySim.as_str()
            && run.sim_run_id.as_deref() == Some(sim_run_id)
    })
}

pub(crate) fn load_recent_agent_runs<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Vec<OversightAgentRunRecord> {
    load_run_index(store, site_id)
        .into_iter()
        .filter_map(|run_id| load_run_record(store, site_id, &run_id))
        .collect()
}

pub(crate) fn load_latest_agent_run<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<OversightAgentRunRecord> {
    load_recent_agent_runs(store, site_id).into_iter().next()
}

pub(crate) fn execute_agent_cycle<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    trigger: OversightAgentTrigger,
) -> Result<OversightAgentExecutionResult, ()> {
    if !shared_host_execution_available() {
        return Err(());
    }
    if trigger.kind == OversightAgentTriggerKind::PostAdversarySim {
        if let Some(sim_run_id) = trigger.sim_run_id.as_deref() {
            if let Some(existing) = matching_post_sim_run(store, site_id, sim_run_id) {
                return Ok(OversightAgentExecutionResult {
                    schema_version: OVERSIGHT_AGENT_EXECUTION_SCHEMA_VERSION.to_string(),
                    status: "replayed".to_string(),
                    replayed: true,
                    run: existing,
                });
            }
        }
    }

    let operation_id = run_id(&trigger);
    let now = if trigger.requested_at_ts == 0 {
        crate::admin::now_ts()
    } else {
        trigger.requested_at_ts
    };
    let current_lease = load_agent_lease(store, site_id);
    let lease = crate::admin::adversary_sim_control::acquire_controller_lease(
        now,
        operation_id.as_str(),
        Some(operation_id.as_str()),
        current_lease.as_ref(),
    )
    .map_err(|_| ())?;
    save_agent_lease(store, site_id, &lease)?;

    let execution =
        crate::admin::oversight_api::execute_reconcile_cycle(store, site_id, trigger.kind.as_str())?;
    let run = OversightAgentRunRecord {
        schema_version: OVERSIGHT_AGENT_RUN_SCHEMA_VERSION.to_string(),
        run_id: operation_id,
        trigger_kind: trigger.kind.as_str().to_string(),
        requested_at_ts: now,
        started_at_ts: now,
        completed_at_ts: now,
        sim_run_id: trigger.sim_run_id,
        sim_completion_reason: trigger.sim_completion_reason,
        execution,
    };
    let persist_result = persist_run_record(store, site_id, &run);
    release_agent_lease(store, site_id);
    persist_result?;

    Ok(OversightAgentExecutionResult {
        schema_version: OVERSIGHT_AGENT_EXECUTION_SCHEMA_VERSION.to_string(),
        status: "executed".to_string(),
        replayed: false,
        run,
    })
}

pub(crate) fn build_status_payload<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OversightAgentStatusPayload {
    OversightAgentStatusPayload {
        schema_version: OVERSIGHT_AGENT_STATUS_SCHEMA_VERSION.to_string(),
        execution_boundary: "shared_host_only".to_string(),
        periodic_trigger: OversightAgentPeriodicTriggerContract {
            surface: "host_supervisor_wrapper".to_string(),
            wrapper_command: "scripts/run_with_oversight_supervisor.sh".to_string(),
            default_interval_seconds: OVERSIGHT_AGENT_DEFAULT_INTERVAL_SECONDS,
        },
        post_sim_trigger: OversightAgentPostSimTriggerContract {
            surface: "internal_adversary_sim_completion_hook".to_string(),
            qualifying_completion: "transition_to_off_with_completed_run_id_and_generated_traffic"
                .to_string(),
            dedupe_key: "sim_run_id".to_string(),
        },
        latest_run: load_latest_agent_run(store, site_id),
        latest_decision: load_latest_decision(store, site_id),
        recent_runs: load_recent_agent_runs(store, site_id),
    }
}

pub(crate) fn post_sim_trigger_for_state_transition(
    previous_state: &crate::admin::adversary_sim::ControlState,
    next_state: &crate::admin::adversary_sim::ControlState,
    requested_at_ts: u64,
) -> Option<OversightAgentTrigger> {
    if previous_state.phase == crate::admin::adversary_sim::ControlPhase::Off
        || next_state.phase != crate::admin::adversary_sim::ControlPhase::Off
    {
        return None;
    }
    if next_state.generated_tick_count == 0 && next_state.generated_request_count == 0 {
        return None;
    }
    let sim_run_id = next_state.last_run_id.clone()?;
    Some(OversightAgentTrigger {
        kind: OversightAgentTriggerKind::PostAdversarySim,
        requested_at_ts,
        sim_run_id: Some(sim_run_id),
        sim_completion_reason: next_state.last_transition_reason.clone(),
    })
}

pub(crate) fn maybe_trigger_post_sim_agent_cycle<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    previous_state: &crate::admin::adversary_sim::ControlState,
    next_state: &crate::admin::adversary_sim::ControlState,
    requested_at_ts: u64,
) -> Result<Option<OversightAgentExecutionResult>, ()> {
    if !shared_host_execution_available() {
        return Ok(None);
    }
    let Some(trigger) =
        post_sim_trigger_for_state_transition(previous_state, next_state, requested_at_ts)
    else {
        return Ok(None);
    };
    execute_agent_cycle(store, site_id, trigger).map(Some)
}

#[cfg(test)]
mod tests {
    use super::{
        execute_agent_cycle, load_latest_agent_run, maybe_trigger_post_sim_agent_cycle,
        OversightAgentTrigger, OversightAgentTriggerKind,
    };
    use crate::challenge::KeyValueStore;
    use crate::config::{defaults, serialize_persisted_kv_config};
    use crate::observability::hot_read_documents::{
        operator_snapshot_document_contract, operator_snapshot_document_key,
        HotReadDocumentEnvelope, HotReadDocumentMetadata, HotReadUpdateTrigger,
    };
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
    use crate::observability::operator_snapshot::{
        build_operator_snapshot_payload, OperatorSnapshotRecentChanges,
    };
    use crate::runtime::effect_intents::ExecutionMode;
    use crate::runtime::request_outcome::{
        RenderedRequestOutcome, RequestOutcomeClass, RequestOutcomeLane, ResponseKind,
        TrafficOrigin,
    };
    use crate::runtime::traffic_classification::{
        MeasurementScope, PolicySource, RouteActionFamily, TrafficLane,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                map: Mutex::new(HashMap::new()),
            }
        }
    }

    impl KeyValueStore for TestStore {
        fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
            Ok(self.map.lock().expect("map lock").get(key).cloned())
        }

        fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
            self.map
                .lock()
                .expect("map lock")
                .insert(key.to_string(), value.to_vec());
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), ()> {
            self.map.lock().expect("map lock").remove(key);
            Ok(())
        }

        fn get_keys(&self) -> Result<Vec<String>, ()> {
            Ok(self.map.lock().expect("map lock").keys().cloned().collect())
        }
    }

    fn seed_snapshot(store: &TestStore, cfg: crate::config::Config) {
        store
            .set(
                "config:default",
                &serialize_persisted_kv_config(&cfg).expect("cfg serializes"),
            )
            .expect("config seed");
        record_request_outcome(
            store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 2_000,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let summary = summarize_with_store(store, 24, 10);
        let payload = build_operator_snapshot_payload(
            store,
            "default",
            1_700_000_200,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );
        let document = HotReadDocumentEnvelope {
            metadata: HotReadDocumentMetadata {
                schema_version: operator_snapshot_document_contract()
                    .schema_version
                    .to_string(),
                site_id: "default".to_string(),
                generated_at_ts: 1_700_000_200,
                trigger: HotReadUpdateTrigger::RepairRebuild,
            },
            payload,
        };
        store
            .set(
                operator_snapshot_document_key("default").as_str(),
                &serde_json::to_vec(&document).expect("document serializes"),
            )
            .expect("snapshot seed");
    }

    #[test]
    fn agent_cycle_records_periodic_supervisor_run_and_exposes_latest_run() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let execution = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_000_300,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("agent cycle succeeds");

        assert!(!execution.replayed);
        assert_eq!(execution.run.trigger_kind, "periodic_supervisor");
        assert_eq!(execution.run.execution.reconcile.outcome, "recommend_patch");

        let latest = load_latest_agent_run(&store, "default").expect("latest run");
        assert_eq!(latest.run_id, execution.run.run_id);
        assert_eq!(latest.trigger_kind, "periodic_supervisor");
    }

    #[test]
    fn post_sim_agent_cycle_replays_existing_run_for_same_completed_run_id() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let first = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PostAdversarySim,
                requested_at_ts: 1_700_000_300,
                sim_run_id: Some("simrun-001".to_string()),
                sim_completion_reason: Some("auto_window_expired".to_string()),
            },
        )
        .expect("first cycle succeeds");
        let second = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PostAdversarySim,
                requested_at_ts: 1_700_000_301,
                sim_run_id: Some("simrun-001".to_string()),
                sim_completion_reason: Some("auto_window_expired".to_string()),
            },
        )
        .expect("second cycle succeeds");

        assert!(!first.replayed);
        assert!(second.replayed);
        assert_eq!(second.run.run_id, first.run.run_id);
    }

    #[test]
    fn post_sim_agent_cycle_is_skipped_on_edge_profile() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");

        let store = TestStore::new();
        let previous_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            run_id: Some("simrun-edge-skip".to_string()),
            active_run_count: 1,
            active_lane_count: 2,
            ..crate::admin::adversary_sim::ControlState::default()
        };
        let next_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Off,
            last_run_id: Some("simrun-edge-skip".to_string()),
            last_transition_reason: Some("auto_window_expired".to_string()),
            generated_tick_count: 1,
            generated_request_count: 4,
            ..crate::admin::adversary_sim::ControlState::default()
        };

        let result =
            maybe_trigger_post_sim_agent_cycle(&store, "default", &previous_state, &next_state, 1_700_000_400)
                .expect("edge skip succeeds");
        assert!(result.is_none());

        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }
}
