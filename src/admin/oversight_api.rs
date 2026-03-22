use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use super::oversight_agent::{
    build_status_payload, execute_agent_cycle, OversightAgentTrigger,
    OversightAgentTriggerKind,
};
use super::oversight_decision_ledger::{
    load_recent_decisions, record_decision, OversightDecisionDraft,
    OversightDecisionEvidenceReference, OversightDecisionRecord,
};
use super::oversight_reconcile::{reconcile, OversightReconcileResult};

pub(crate) const OVERSIGHT_EXECUTION_SCHEMA_VERSION: &str = "oversight_execution_v1";
pub(crate) const OVERSIGHT_HISTORY_SCHEMA_VERSION: &str = "oversight_history_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchValidationIssue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchValidation {
    pub status: String,
    pub issues: Vec<OversightPatchValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightExecutionPayload {
    pub schema_version: String,
    pub decision: OversightDecisionRecord,
    pub reconcile: OversightReconcileResult,
    pub validation: OversightPatchValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AdminConfigValidationResponse {
    valid: bool,
    issues: Vec<OversightPatchValidationIssue>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct InternalOversightAgentRunRequest {
    trigger_kind: Option<String>,
    sim_run_id: Option<String>,
    sim_completion_reason: Option<String>,
}

pub(crate) fn execute_reconcile_cycle(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    trigger_source: &str,
) -> Result<OversightExecutionPayload, ()> {
    let now = crate::admin::now_ts();
    let snapshot = crate::observability::hot_read_projection::load_operator_snapshot_hot_read(
        store, site_id,
    );
    let objectives =
        crate::observability::operator_objectives_store::load_or_seed_operator_objectives(
            store, site_id, now,
        );

    let (reconcile_result, validation) = match snapshot {
        Some(snapshot) => {
            match crate::config::load_runtime_cached(store, site_id) {
                Ok(cfg) => {
                    let result = reconcile(&cfg, &snapshot.payload, trigger_source);
                    let validation = validate_reconcile_result(store, site_id, &result);
                    (result, validation)
                }
                Err(_) => (
                    OversightReconcileResult {
                        schema_version: crate::admin::oversight_reconcile::OVERSIGHT_RECONCILE_SCHEMA_VERSION
                            .to_string(),
                        generated_at: now,
                        trigger_source: trigger_source.to_string(),
                        outcome: "insufficient_evidence".to_string(),
                        summary: "Runtime config is unavailable, so recommend-only reconcile must fail closed.".to_string(),
                        objective_revision: snapshot.payload.objectives.revision.clone(),
                        benchmark_overall_status: snapshot.payload.benchmark_results.overall_status.clone(),
                        improvement_status: snapshot.payload.benchmark_results.improvement_status.clone(),
                        trigger_family_ids: snapshot
                            .payload
                            .benchmark_results
                            .escalation_hint
                            .trigger_family_ids
                            .clone(),
                        candidate_action_families: snapshot
                            .payload
                            .benchmark_results
                            .escalation_hint
                            .candidate_action_families
                            .clone(),
                        refusal_reasons: vec!["config_unavailable".to_string()],
                        proposal: None,
                        latest_sim_run_id: crate::admin::oversight_reconcile::latest_recent_sim_run_id(
                            &snapshot.payload,
                        ),
                        replay_promotion_availability: snapshot.payload.replay_promotion.availability.clone(),
                        snapshot_generated_at: snapshot.payload.generated_at,
                        evidence_references: vec![
                            crate::admin::oversight_reconcile::OversightEvidenceReference {
                                kind: "operator_snapshot".to_string(),
                                reference: format!("generated_at:{}", snapshot.payload.generated_at),
                                note: "Operator snapshot was materialized, but runtime config could not be loaded to shape a truthful proposal.".to_string(),
                            },
                        ],
                    },
                    OversightPatchValidation {
                        status: "skipped".to_string(),
                        issues: Vec::new(),
                    },
                ),
            }
        }
        None => (
            OversightReconcileResult {
                schema_version: crate::admin::oversight_reconcile::OVERSIGHT_RECONCILE_SCHEMA_VERSION
                    .to_string(),
                generated_at: now,
                trigger_source: trigger_source.to_string(),
                outcome: "insufficient_evidence".to_string(),
                summary: "Operator snapshot is not materialized yet, so recommend-only reconcile must fail closed.".to_string(),
                objective_revision: objectives.revision.clone(),
                benchmark_overall_status: "not_available".to_string(),
                improvement_status: "not_available".to_string(),
                trigger_family_ids: Vec::new(),
                candidate_action_families: Vec::new(),
                refusal_reasons: vec!["operator_snapshot_not_materialized".to_string()],
                proposal: None,
                latest_sim_run_id: None,
                replay_promotion_availability: "not_materialized".to_string(),
                snapshot_generated_at: 0,
                evidence_references: vec![crate::admin::oversight_reconcile::OversightEvidenceReference {
                    kind: "operator_snapshot".to_string(),
                    reference: "not_materialized".to_string(),
                    note: "Hot-read snapshot must exist before reconcile can safely reason about current budgets.".to_string(),
                }],
            },
            OversightPatchValidation {
                status: "skipped".to_string(),
                issues: Vec::new(),
            },
        ),
    };

    let recorded_result = apply_validation_to_result(reconcile_result, &validation);
    let decision = record_decision(
        store,
        site_id,
        OversightDecisionDraft {
            recorded_at_ts: now,
            trigger_source: trigger_source.to_string(),
            outcome: recorded_result.outcome.clone(),
            summary: recorded_result.summary.clone(),
            objective_revision: recorded_result.objective_revision.clone(),
            snapshot_generated_at: recorded_result.snapshot_generated_at,
            benchmark_overall_status: recorded_result.benchmark_overall_status.clone(),
            improvement_status: recorded_result.improvement_status.clone(),
            replay_promotion_availability: recorded_result.replay_promotion_availability.clone(),
            trigger_family_ids: recorded_result.trigger_family_ids.clone(),
            candidate_action_families: recorded_result.candidate_action_families.clone(),
            refusal_reasons: recorded_result.refusal_reasons.clone(),
            proposal: recorded_result.proposal.clone(),
            validation_status: validation.status.clone(),
            validation_issues: validation
                .issues
                .iter()
                .map(|issue| issue.message.clone())
                .collect(),
            latest_sim_run_id: recorded_result.latest_sim_run_id.clone(),
            evidence_references: recorded_result
                .evidence_references
                .iter()
                .map(|reference| OversightDecisionEvidenceReference {
                    kind: reference.kind.clone(),
                    reference: reference.reference.clone(),
                    note: reference.note.clone(),
                })
                .collect(),
        },
    )?;

    Ok(OversightExecutionPayload {
        schema_version: OVERSIGHT_EXECUTION_SCHEMA_VERSION.to_string(),
        decision,
        reconcile: recorded_result,
        validation,
    })
}

pub(crate) fn handle_admin_oversight_reconcile(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    match execute_reconcile_cycle(store, site_id, "manual_admin") {
        Ok(payload) => {
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        Err(()) => Response::new(500, "Key-value store error"),
    }
}

pub(crate) fn handle_admin_oversight_history(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let body = serde_json::to_string(&json!({
        "schema_version": OVERSIGHT_HISTORY_SCHEMA_VERSION,
        "rows": load_recent_decisions(store, site_id),
    }))
    .unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_admin_oversight_agent_status(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let payload = build_status_payload(store, site_id);
    let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_internal_oversight_agent_run(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    if !crate::admin::auth::is_internal_oversight_supervisor_request(req) {
        return Response::new(
            401,
            "Unauthorized: Internal oversight supervisor authorization required",
        );
    }

    let parsed = if req.body().is_empty() {
        InternalOversightAgentRunRequest::default()
    } else {
        match serde_json::from_slice::<InternalOversightAgentRunRequest>(req.body()) {
            Ok(parsed) => parsed,
            Err(_) => return Response::new(400, "Invalid oversight agent trigger payload"),
        }
    };
    let trigger_kind = parsed
        .trigger_kind
        .as_deref()
        .and_then(OversightAgentTriggerKind::parse)
        .unwrap_or(OversightAgentTriggerKind::PeriodicSupervisor);
    let trigger = OversightAgentTrigger {
        kind: trigger_kind,
        requested_at_ts: crate::admin::now_ts(),
        sim_run_id: parsed.sim_run_id,
        sim_completion_reason: parsed.sim_completion_reason,
    };
    match execute_agent_cycle(store, site_id, trigger) {
        Ok(payload) => {
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        Err(()) => Response::new(500, "Key-value store error"),
    }
}

fn validate_reconcile_result(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    result: &OversightReconcileResult,
) -> OversightPatchValidation {
    let Some(proposal) = result.proposal.as_ref() else {
        return OversightPatchValidation {
            status: "skipped".to_string(),
            issues: Vec::new(),
        };
    };
    let body = serde_json::to_vec(&proposal.patch).unwrap_or_default();
    let request = Request::builder()
        .method(Method::Post)
        .uri("/admin/config/validate")
        .body(body)
        .build();
    let response = super::config_api::handle_admin_config_validate(&request, store, site_id);
    if *response.status() != 200 {
        return OversightPatchValidation {
            status: "validation_error".to_string(),
            issues: vec![OversightPatchValidationIssue {
                field: None,
                message: String::from_utf8_lossy(response.body()).to_string(),
                expected: None,
                received: None,
            }],
        };
    }
    let payload = serde_json::from_slice::<AdminConfigValidationResponse>(response.body())
        .unwrap_or(AdminConfigValidationResponse {
            valid: false,
            issues: vec![OversightPatchValidationIssue {
                field: None,
                message: "Unable to parse config validation response.".to_string(),
                expected: None,
                received: None,
            }],
        });
    OversightPatchValidation {
        status: if payload.valid {
            "valid".to_string()
        } else {
            "invalid".to_string()
        },
        issues: payload.issues,
    }
}

fn apply_validation_to_result(
    mut result: OversightReconcileResult,
    validation: &OversightPatchValidation,
) -> OversightReconcileResult {
    if result.proposal.is_some() && validation.status == "invalid" {
        result.outcome = "refuse_invalid_patch".to_string();
        result.summary =
            "Reconcile produced a bounded candidate patch, but the existing config validator rejected it."
                .to_string();
        result.refusal_reasons = validation
            .issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect();
    } else if result.proposal.is_some() && validation.status == "validation_error" {
        result.outcome = "insufficient_evidence".to_string();
        result.summary =
            "Config validation failed unexpectedly, so recommend-only reconcile failed closed."
                .to_string();
        result.refusal_reasons = validation
            .issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        execute_reconcile_cycle, handle_admin_oversight_agent_status,
        handle_admin_oversight_history, handle_admin_oversight_reconcile,
        handle_internal_oversight_agent_run,
    };
    use crate::challenge::KeyValueStore;
    use crate::config::{defaults, serialize_persisted_kv_config};
    use crate::observability::hot_read_documents::{
        operator_snapshot_document_contract, operator_snapshot_document_key, HotReadDocumentEnvelope,
        HotReadDocumentMetadata, HotReadUpdateTrigger,
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
    use spin_sdk::http::{Method, Request};
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
    fn reconcile_cycle_records_insufficient_evidence_when_snapshot_missing() {
        let store = TestStore::new();

        let payload = execute_reconcile_cycle(&store, "default", "manual_admin")
            .expect("cycle succeeds");

        assert_eq!(payload.reconcile.outcome, "insufficient_evidence");
        assert_eq!(payload.validation.status, "skipped");
    }

    #[test]
    fn manual_reconcile_route_records_a_recommendation_and_history() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let request = Request::builder()
            .method(Method::Post)
            .uri("/admin/oversight/reconcile")
            .body(Vec::new())
            .build();
        let response = handle_admin_oversight_reconcile(&request, &store, "default");

        assert_eq!(*response.status(), 200);
        let payload: serde_json::Value =
            serde_json::from_slice(response.body()).expect("payload decodes");
        assert_eq!(payload["reconcile"]["outcome"], "recommend_patch");
        assert_eq!(payload["validation"]["status"], "valid");

        let history_request = Request::builder()
            .method(Method::Get)
            .uri("/admin/oversight/history")
            .body(Vec::new())
            .build();
        let history_response = handle_admin_oversight_history(&history_request, &store, "default");
        let history_payload: serde_json::Value =
            serde_json::from_slice(history_response.body()).expect("history decodes");
        assert_eq!(history_payload["schema_version"], "oversight_history_v1");
        assert_eq!(history_payload["rows"].as_array().expect("rows array").len(), 1);
    }

    #[test]
    fn reconcile_cycle_fails_closed_when_runtime_config_is_unavailable() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);
        store.delete("config:default").expect("config delete");

        let payload = execute_reconcile_cycle(&store, "default", "manual_admin")
            .expect("cycle succeeds");

        assert_eq!(payload.reconcile.outcome, "insufficient_evidence");
        assert!(payload
            .reconcile
            .refusal_reasons
            .contains(&"config_unavailable".to_string()));
        assert_eq!(payload.validation.status, "skipped");
    }

    #[test]
    fn internal_agent_route_records_periodic_run_and_status_surface() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "oversight-agent-test-key");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let internal_request = Request::builder()
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer oversight-agent-test-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "oversight-agent")
            .body(
                serde_json::to_vec(&serde_json::json!({
                    "trigger_kind": "periodic_supervisor"
                }))
                .expect("json body"),
            )
            .build();
        let response = handle_internal_oversight_agent_run(&internal_request, &store, "default");
        assert_eq!(*response.status(), 200);

        let status_request = Request::builder()
            .method(Method::Get)
            .uri("/admin/oversight/agent/status")
            .body(Vec::new())
            .build();
        let status_response =
            handle_admin_oversight_agent_status(&status_request, &store, "default");
        assert_eq!(*status_response.status(), 200);
        let payload: serde_json::Value =
            serde_json::from_slice(status_response.body()).expect("status decodes");
        assert_eq!(payload["latest_run"]["trigger_kind"], "periodic_supervisor");
        assert_eq!(
            payload["latest_run"]["execution"]["reconcile"]["outcome"],
            "recommend_patch"
        );

        std::env::remove_var("SHUMA_API_KEY");
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }
}
