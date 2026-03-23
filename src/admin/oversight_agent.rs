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
const POST_SIM_EVENT_EVIDENCE_LOOKBACK_HOURS: u64 = 2;

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

    let execution = crate::admin::oversight_api::execute_oversight_cycle_at(
        store,
        site_id,
        trigger.kind.as_str(),
        crate::admin::oversight_apply::OversightApplyMode::ExecuteCanary,
        now,
    )?;
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
    let sim_run_id = completed_sim_run_id_for_transition(previous_state, next_state)?;
    let had_generated_traffic = next_state.generated_tick_count > 0
        || next_state.generated_request_count > 0
        || previous_state.generated_tick_count > 0
        || previous_state.generated_request_count > 0
        || next_state.last_generated_at.is_some()
        || previous_state.last_generated_at.is_some();
    if !had_generated_traffic {
        return None;
    }
    Some(build_post_sim_trigger(next_state, requested_at_ts, sim_run_id))
}

fn completed_sim_run_id_for_transition(
    previous_state: &crate::admin::adversary_sim::ControlState,
    next_state: &crate::admin::adversary_sim::ControlState,
) -> Option<String> {
    if previous_state.phase == crate::admin::adversary_sim::ControlPhase::Off
        || next_state.phase != crate::admin::adversary_sim::ControlPhase::Off
    {
        return None;
    }
    next_state.last_run_id.clone()
}

fn build_post_sim_trigger(
    next_state: &crate::admin::adversary_sim::ControlState,
    requested_at_ts: u64,
    sim_run_id: String,
) -> OversightAgentTrigger {
    OversightAgentTrigger {
        kind: OversightAgentTriggerKind::PostAdversarySim,
        requested_at_ts,
        sim_run_id: Some(sim_run_id),
        sim_completion_reason: next_state.last_transition_reason.clone(),
    }
}

fn has_persisted_post_sim_event_evidence<S: KeyValueStore>(
    store: &S,
    requested_at_ts: u64,
    sim_run_id: &str,
) -> bool {
    crate::admin::api::load_recent_monitoring_event_records(
        store,
        requested_at_ts,
        POST_SIM_EVENT_EVIDENCE_LOOKBACK_HOURS,
    )
    .into_iter()
    .any(|record| record.is_simulation && record.sim_run_id.as_deref() == Some(sim_run_id))
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
    let trigger = post_sim_trigger_for_state_transition(previous_state, next_state, requested_at_ts)
        .or_else(|| {
            let sim_run_id = completed_sim_run_id_for_transition(previous_state, next_state)?;
            if !has_persisted_post_sim_event_evidence(store, requested_at_ts, sim_run_id.as_str()) {
                return None;
            }
            Some(build_post_sim_trigger(next_state, requested_at_ts, sim_run_id))
        });
    let Some(trigger) = trigger else {
        return Ok(None);
    };
    execute_agent_cycle(store, site_id, trigger).map(Some)
}

#[cfg(test)]
mod tests {
    use super::{
        execute_agent_cycle, load_latest_agent_run, maybe_trigger_post_sim_agent_cycle,
        post_sim_trigger_for_state_transition, OversightAgentTrigger, OversightAgentTriggerKind,
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
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 2_000,
                forwarded_upstream_latency_ms: None,
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

    fn seed_apply_ready_snapshot(store: &TestStore, cfg: crate::config::Config) {
        seed_candidate_snapshot(store, cfg, 1_700_000_200, 0.42, "outside_budget");
    }

    fn seed_candidate_snapshot(
        store: &TestStore,
        cfg: crate::config::Config,
        generated_at_ts: u64,
        suspicious_forwarded_request_rate: f64,
        overall_status: &str,
    ) {
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
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 2_000,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let summary = summarize_with_store(store, 24, 10);
        let mut payload = build_operator_snapshot_payload(
            store,
            "default",
            generated_at_ts,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            generated_at_ts,
            generated_at_ts,
            generated_at_ts,
        );
        payload.non_human_traffic.readiness.status = "ready".to_string();
        payload.non_human_traffic.readiness.blockers.clear();
        payload.non_human_traffic.readiness.live_receipt_count = 1;
        payload.non_human_traffic.readiness.adversary_sim_receipt_count = 1;
        payload.non_human_traffic.coverage.overall_status = "covered".to_string();
        payload.non_human_traffic.coverage.blocking_reasons.clear();
        payload.non_human_traffic.coverage.blocking_category_ids.clear();
        payload.non_human_traffic.coverage.mapped_category_count = 6;
        payload.non_human_traffic.coverage.covered_category_count = 6;
        payload.non_human_traffic.coverage.partial_category_count = 0;
        payload.non_human_traffic.coverage.stale_category_count = 0;
        payload.non_human_traffic.coverage.unavailable_category_count = 0;
        payload.non_human_traffic.coverage.uncovered_category_count = 2;
        payload.replay_promotion.availability = "materialized".to_string();
        payload.replay_promotion.evidence_status = "protected".to_string();
        payload.replay_promotion.tuning_eligible = true;
        payload.replay_promotion.protected_basis = "replay_promoted_lineage".to_string();
        payload.replay_promotion.protected_lineage_count = 1;
        payload.replay_promotion.eligibility_blockers.clear();
        payload.benchmark_results.coverage_status = "partial_support".to_string();
        payload.benchmark_results.generated_at = generated_at_ts;
        payload.benchmark_results.input_snapshot_generated_at = generated_at_ts;
        payload.benchmark_results.overall_status = overall_status.to_string();
        payload.benchmark_results.improvement_status = if overall_status == "inside_budget" {
            "improved".to_string()
        } else {
            "regressed".to_string()
        };
        payload.benchmark_results.non_human_classification =
            payload.non_human_traffic.readiness.clone();
        payload.benchmark_results.non_human_coverage =
            payload.non_human_traffic.coverage.compact_for_benchmark();
        payload.benchmark_results.tuning_eligibility.status = "eligible".to_string();
        payload.benchmark_results.tuning_eligibility.blockers.clear();
        payload.benchmark_results.escalation_hint.availability = "partial_support".to_string();
        payload.benchmark_results.escalation_hint.decision =
            "config_tuning_candidate".to_string();
        payload.benchmark_results.escalation_hint.review_status =
            "manual_review_required".to_string();
        payload.benchmark_results.escalation_hint.trigger_family_ids =
            vec!["suspicious_origin_cost".to_string()];
        payload.benchmark_results.escalation_hint.candidate_action_families =
            vec!["fingerprint_signal".to_string()];
        payload.benchmark_results.escalation_hint.blockers.clear();
        payload.benchmark_results.replay_promotion = payload.replay_promotion.clone();
        if let Some(row) = payload.budget_distance.rows.get_mut(0) {
            row.current = suspicious_forwarded_request_rate;
            row.delta = suspicious_forwarded_request_rate - row.target;
            row.status = overall_status.to_string();
        }
        if let Some(family) = payload
            .benchmark_results
            .families
            .iter_mut()
            .find(|family| family.family_id == "suspicious_origin_cost")
        {
            family.status = overall_status.to_string();
            family.comparison_status = if overall_status == "inside_budget" {
                "improved".to_string()
            } else {
                "regressed".to_string()
            };
            if let Some(metric) = family
                .metrics
                .iter_mut()
                .find(|metric| metric.metric_id == "suspicious_forwarded_request_rate")
            {
                metric.current = Some(suspicious_forwarded_request_rate);
                metric.delta =
                    metric.target.map(|target| suspicious_forwarded_request_rate - target);
                metric.status = overall_status.to_string();
                metric.comparison_status = family.comparison_status.clone();
            }
        }
        let document = HotReadDocumentEnvelope {
            metadata: HotReadDocumentMetadata {
                schema_version: operator_snapshot_document_contract()
                    .schema_version
                    .to_string(),
                site_id: "default".to_string(),
                generated_at_ts,
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

    fn seed_canary_only_objectives(store: &TestStore) {
        let mut profile =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(
                1_700_000_100,
            );
        profile.window_hours = 1;
        profile.rollout_guardrails.automated_apply_status = "canary_only".to_string();
        crate::observability::operator_objectives_store::save_operator_objectives(
            store,
            "default",
            &profile,
        )
        .expect("objectives save");
    }

    #[test]
    fn agent_cycle_records_periodic_supervisor_run_and_exposes_latest_run() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_apply_ready_snapshot(&store, cfg);

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
    fn post_sim_trigger_accepts_generation_evidence_from_previous_running_state() {
        let previous_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            run_id: Some("simrun-live-proof".to_string()),
            active_run_count: 1,
            active_lane_count: 2,
            generated_tick_count: 3,
            generated_request_count: 12,
            last_generated_at: Some(1_700_000_399),
            ..crate::admin::adversary_sim::ControlState::default()
        };
        let next_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Off,
            last_run_id: Some("simrun-live-proof".to_string()),
            last_transition_reason: Some("manual_off".to_string()),
            generated_tick_count: 0,
            generated_request_count: 0,
            ..crate::admin::adversary_sim::ControlState::default()
        };

        let trigger =
            post_sim_trigger_for_state_transition(&previous_state, &next_state, 1_700_000_400)
                .expect("post-sim trigger");
        assert_eq!(trigger.kind, OversightAgentTriggerKind::PostAdversarySim);
        assert_eq!(trigger.sim_run_id.as_deref(), Some("simrun-live-proof"));
        assert_eq!(trigger.sim_completion_reason.as_deref(), Some("manual_off"));
    }

    #[test]
    fn post_sim_agent_cycle_accepts_persisted_event_evidence_when_terminal_state_is_zeroed() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let sim_run_id = "simrun-persisted-evidence";
        let _guard = crate::runtime::sim_telemetry::enter(Some(
            crate::runtime::sim_telemetry::SimulationRequestMetadata {
                sim_run_id: sim_run_id.to_string(),
                sim_profile: "runtime_toggle".to_string(),
                sim_lane: "deterministic_black_box".to_string(),
            },
        ));
        crate::admin::log_event(
            &store,
            &crate::admin::EventLogEntry {
                ts: 1_700_000_399,
                event: crate::admin::EventType::Challenge,
                ip: Some("198.51.100.42".to_string()),
                reason: Some("simulated".to_string()),
                outcome: Some("served".to_string()),
                admin: None,
            },
        );
        drop(_guard);

        let previous_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Running,
            run_id: Some(sim_run_id.to_string()),
            active_run_count: 1,
            active_lane_count: 2,
            ..crate::admin::adversary_sim::ControlState::default()
        };
        let next_state = crate::admin::adversary_sim::ControlState {
            phase: crate::admin::adversary_sim::ControlPhase::Off,
            last_run_id: Some(sim_run_id.to_string()),
            last_transition_reason: Some("manual_off".to_string()),
            ..crate::admin::adversary_sim::ControlState::default()
        };

        let execution = maybe_trigger_post_sim_agent_cycle(
            &store,
            "default",
            &previous_state,
            &next_state,
            1_700_000_400,
        )
        .expect("execution succeeds")
        .expect("post-sim execution triggered");
        assert_eq!(execution.run.trigger_kind, "post_adversary_sim");
        assert_eq!(execution.run.sim_run_id.as_deref(), Some(sim_run_id));
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

    #[test]
    fn agent_cycle_refuses_canary_apply_when_rollout_guardrail_is_manual_only() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_apply_ready_snapshot(&store, cfg);
        let original_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("seeded config");

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

        let payload = serde_json::to_value(&execution.run.execution).expect("payload serializes");
        assert_eq!(payload["reconcile"]["outcome"], "recommend_patch");
        assert_eq!(payload["apply"]["stage"], "refused");
        assert_eq!(
            payload["apply"]["refusal_reasons"][0],
            "automated_apply_manual_only"
        );

        let persisted_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("persisted config");
        assert_eq!(persisted_config, original_config);
    }

    #[test]
    fn agent_cycle_can_apply_one_canary_when_rollout_guardrail_is_canary_only() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_canary_only_objectives(&store);
        seed_apply_ready_snapshot(&store, cfg);
        let original_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("seeded config");

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

        let payload = serde_json::to_value(&execution.run.execution).expect("payload serializes");
        assert_eq!(payload["reconcile"]["outcome"], "recommend_patch");
        assert_eq!(payload["apply"]["stage"], "canary_applied");
        assert_eq!(payload["apply"]["patch_family"], "fingerprint_signal");

        let persisted_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("persisted config");
        assert_ne!(persisted_config, original_config);
    }

    #[test]
    fn agent_cycle_reports_watch_window_open_before_candidate_window_ends() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_canary_only_objectives(&store);
        seed_apply_ready_snapshot(&store, cfg);

        let first = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_000_300,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("initial canary apply succeeds");
        assert_eq!(first.run.execution.apply.stage, "canary_applied");

        let canary_cfg =
            crate::config::Config::load(&store, "default").expect("canary config loads");
        seed_candidate_snapshot(&store, canary_cfg, 1_700_001_200, 0.30, "inside_budget");

        let second = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_003_899,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("watch-window cycle succeeds");

        assert_eq!(second.run.execution.apply.stage, "watch_window_open");
        assert_eq!(
            second.run.execution.apply.watch_window_end_at,
            Some(1_700_003_900)
        );
        assert_eq!(
            second.run.execution.apply.patch_family.as_deref(),
            Some("fingerprint_signal")
        );
        assert!(
            store
                .get_keys()
                .expect("keys load")
                .iter()
                .any(|key| key == "oversight_active_canary:v1:default")
        );
    }

    #[test]
    fn agent_cycle_rolls_back_canary_when_candidate_window_regresses() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_canary_only_objectives(&store);
        seed_apply_ready_snapshot(&store, cfg);
        let original_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("seeded config");

        let first = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_000_300,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("initial canary apply succeeds");
        assert_eq!(first.run.execution.apply.stage, "canary_applied");

        let canary_cfg =
            crate::config::Config::load(&store, "default").expect("canary config loads");
        seed_candidate_snapshot(&store, canary_cfg, 1_700_004_000, 0.55, "outside_budget");

        let second = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_004_000,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("rollback cycle succeeds");

        assert_eq!(second.run.execution.apply.stage, "rollback_applied");
        assert_eq!(
            second.run.execution.apply.comparison_status.as_deref(),
            Some("regressed")
        );
        assert_eq!(
            second.run.execution.apply.rollback_reason.as_deref(),
            Some("candidate_comparison_regressed")
        );
        assert_eq!(second.run.execution.decision.outcome, "rollback_applied");

        let persisted_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("rolled back config");
        assert_eq!(persisted_config, original_config);
        assert!(
            !store
                .get_keys()
                .expect("keys load")
                .iter()
                .any(|key| key == "oversight_active_canary:v1:default")
        );

        let decision_map =
            crate::observability::decision_ledger::load_recent_decision_map(&store, "default");
        assert!(decision_map.values().any(|decision| {
            decision.decision_kind == "oversight_canary_apply"
                && decision.decision_status == "applied"
        }));
        assert!(decision_map.values().any(|decision| {
            decision.decision_kind == "oversight_canary_rollback"
                && decision.decision_status == "rolled_back"
        }));

        let (recent_changes, _) =
            crate::admin::load_operator_snapshot_recent_changes(&store, "default", 1_700_004_000, 1, 6);
        assert!(recent_changes.rows.iter().any(|row| {
            row.decision_kind.as_deref() == Some("oversight_canary_apply")
                && row.decision_status.as_deref() == Some("applied")
        }));
        assert!(recent_changes.rows.iter().any(|row| {
            row.decision_kind.as_deref() == Some("oversight_canary_rollback")
                && row.decision_status.as_deref() == Some("rolled_back")
        }));
    }

    #[test]
    fn agent_cycle_keeps_canary_when_candidate_window_improves() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_canary_only_objectives(&store);
        seed_apply_ready_snapshot(&store, cfg);
        let original_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("seeded config");

        let first = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_000_300,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("initial canary apply succeeds");
        assert_eq!(first.run.execution.apply.stage, "canary_applied");

        let canary_cfg =
            crate::config::Config::load(&store, "default").expect("canary config loads");
        seed_candidate_snapshot(&store, canary_cfg, 1_700_004_100, 0.12, "inside_budget");

        let second = execute_agent_cycle(
            &store,
            "default",
            OversightAgentTrigger {
                kind: OversightAgentTriggerKind::PeriodicSupervisor,
                requested_at_ts: 1_700_004_100,
                sim_run_id: None,
                sim_completion_reason: None,
            },
        )
        .expect("retain cycle succeeds");

        assert_eq!(second.run.execution.apply.stage, "improved");
        assert_eq!(
            second.run.execution.apply.comparison_status.as_deref(),
            Some("improved")
        );
        assert_eq!(second.run.execution.decision.outcome, "improved");

        let persisted_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("retained config");
        assert_ne!(persisted_config, original_config);
        assert!(
            !store
                .get_keys()
                .expect("keys load")
                .iter()
                .any(|key| key == "oversight_active_canary:v1:default")
        );
    }
}
