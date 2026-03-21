use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use crate::observability::decision_ledger::{
    record_decision, OperatorDecisionDraft, OperatorDecisionEvidenceReference,
};
use crate::observability::operator_objectives_store::{
    load_or_seed_operator_objectives, save_operator_objectives,
};
use crate::observability::operator_snapshot_objectives::{
    operator_objectives_watch_window_seconds, persisted_operator_objectives_from_request,
    OperatorObjectivesUpsertRequest,
};

pub(crate) fn handle_admin_operator_objectives(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    match *req.method() {
        Method::Get => {
            let profile = load_or_seed_operator_objectives(store, site_id, crate::admin::now_ts());
            let body = serde_json::to_string(&profile).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        Method::Post => handle_admin_operator_objectives_update(req, store, site_id),
        _ => Response::new(405, "Method Not Allowed"),
    }
}

fn handle_admin_operator_objectives_update(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if !crate::config::admin_config_write_enabled() {
        return Response::new(
            403,
            "Operator-objectives updates are disabled when SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false",
        );
    }

    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(value) => value,
        Err(err) => return Response::new(400, format!("Invalid operator-objectives payload: {}", err)),
    };
    let request = match serde_json::from_value::<OperatorObjectivesUpsertRequest>(payload) {
        Ok(request) => request,
        Err(err) => {
            return Response::new(
                400,
                format!("Invalid operator-objectives payload: {}", err),
            )
        }
    };

    let updated_at_ts = crate::admin::now_ts();
    let admin_id = crate::admin::auth::get_admin_id(req);
    let profile = match persisted_operator_objectives_from_request(
        request,
        updated_at_ts,
        "manual_admin_profile",
    ) {
        Ok(profile) => profile,
        Err(err) => return Response::new(400, err),
    };

    if save_operator_objectives(store, site_id, &profile).is_err() {
        return Response::new(500, "Failed persisting operator objectives");
    }

    let decision = match record_decision(
        store,
        site_id,
        OperatorDecisionDraft {
            recorded_at_ts: updated_at_ts,
            decision_kind: "operator_objectives_update".to_string(),
            decision_status: "applied".to_string(),
            source: if admin_id.starts_with("controller:") {
                "scheduled_controller".to_string()
            } else {
                "manual_admin".to_string()
            },
            changed_families: vec!["operator_objectives".to_string()],
            targets: vec!["operator_objectives.profile".to_string()],
            objective_revision: profile.revision.clone(),
            watch_window_seconds: operator_objectives_watch_window_seconds(&profile),
            expected_impact_summary:
                "Updated site objectives; later benchmark and watch-window judgments should use the new objective revision."
                    .to_string(),
            evidence_references: vec![
                OperatorDecisionEvidenceReference {
                    kind: "operator_objectives_revision".to_string(),
                    reference: profile.revision.clone(),
                    note: "Server-assigned objective revision persisted for later reconcile and rollback reasoning."
                        .to_string(),
                },
                OperatorDecisionEvidenceReference {
                    kind: "operator_objectives_profile".to_string(),
                    reference: profile.profile_id.clone(),
                    note: "Profile identifier updated through the admin operator-objectives endpoint."
                        .to_string(),
                },
            ],
        },
    ) {
        Ok(decision) => decision,
        Err(_) => return Response::new(500, "Failed persisting operator decision"),
    };

    let recent_change = super::recent_changes_ledger::operator_snapshot_recent_change_with_decision_id(
        &super::recent_changes_ledger::operator_snapshot_manual_change_row(
            updated_at_ts,
            "operator_objectives_update",
            &["operator_objectives"],
            &["operator_objectives.profile"],
            admin_id.as_str(),
            "operator objectives updated",
        ),
        decision.decision_id.as_str(),
    );
    super::recent_changes_ledger::record_operator_snapshot_recent_change_rows(
        store,
        site_id,
        &[recent_change],
        updated_at_ts,
    );
    crate::observability::hot_read_projection::refresh_after_admin_mutation(store, site_id);

    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: updated_at_ts,
            event: crate::admin::EventType::AdminAction,
            ip: None,
            reason: Some("operator_objectives_update".to_string()),
            outcome: Some(profile.revision.clone()),
            admin: Some(admin_id),
        },
    );

    let body = serde_json::to_string(&json!({
        "updated": true,
        "decision_id": decision.decision_id,
        "objectives": profile,
    }))
    .unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

#[cfg(test)]
mod tests {
    use super::handle_admin_operator_objectives;
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use spin_sdk::http::{Method, Request};

    #[derive(Default)]
    struct TestStore {
        map: Mutex<HashMap<String, Vec<u8>>>,
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

    fn objectives_request(method: Method, body: serde_json::Value) -> Request {
        let mut builder = Request::builder();
        builder
            .method(method)
            .uri("/admin/operator-objectives")
            .body(serde_json::to_vec(&body).expect("body serializes"));
        builder.build()
    }

    #[test]
    fn get_operator_objectives_seeds_and_returns_persisted_contract() {
        let store = TestStore::default();
        let req = objectives_request(Method::Get, serde_json::Value::Null);

        let resp = handle_admin_operator_objectives(&req, &store, "default");

        assert_eq!(*resp.status(), 200);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).expect("json body");
        assert_eq!(
            payload.get("schema_version").and_then(|value| value.as_str()),
            Some("operator_objectives_v1")
        );
        assert_eq!(
            payload.get("profile_id").and_then(|value| value.as_str()),
            Some("site_default_v1")
        );
    }

    #[test]
    fn post_operator_objectives_persists_revisioned_profile_and_decision_id() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED", "true");
        let store = TestStore::default();
        let req = objectives_request(
            Method::Post,
            serde_json::json!({
                "profile_id": "custom_profile",
                "window_hours": 12,
                "compliance_semantics": "max_ratio_budget",
                "non_human_posture": "allow_verified_by_category",
                "budgets": [{
                    "budget_id": "likely_human_friction",
                    "metric": "likely_human_friction_rate",
                    "comparator": "max_ratio",
                    "target": 0.03,
                    "near_limit_ratio": 0.8,
                    "eligible_population": "live:ingress_primary:enforced:likely_human"
                }],
                "adversary_sim_expectations": {
                    "comparison_mode": "prior_window",
                    "max_goal_success_rate": 0.0,
                    "min_escalation_rate": 0.5,
                    "regression_status_required": "no_goal_successes"
                },
                "rollout_guardrails": {
                    "automated_apply_status": "manual_only",
                    "code_evolution_status": "review_required"
                }
            }),
        );

        let resp = handle_admin_operator_objectives(&req, &store, "default");

        assert_eq!(*resp.status(), 200);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).expect("json body");
        assert_eq!(
            payload
                .get("updated")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert!(payload
            .get("decision_id")
            .and_then(|value| value.as_str())
            .is_some());
        assert_eq!(
            payload
                .get("objectives")
                .and_then(|value| value.get("profile_id"))
                .and_then(|value| value.as_str()),
            Some("custom_profile")
        );
        std::env::remove_var("SHUMA_ADMIN_CONFIG_WRITE_ENABLED");
    }
}
