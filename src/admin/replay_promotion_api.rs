use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use crate::observability::replay_promotion::{
    load_replay_promotion_payload, persist_replay_promotion_payload, replay_promotion_summary,
    ReplayPromotionIngestPayload, ReplayPromotionPersistError, REPLAY_PROMOTION_SCHEMA_VERSION,
};

pub(crate) fn handle_admin_replay_promotion(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    match *req.method() {
        Method::Get => handle_admin_replay_promotion_get(store, site_id),
        Method::Post => handle_admin_replay_promotion_post(req, store, site_id),
        _ => Response::new(405, "Method Not Allowed"),
    }
}

fn handle_admin_replay_promotion_get(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    match load_replay_promotion_payload(store, site_id) {
        Some(payload) => {
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        None => {
            let body = serde_json::to_string(&json!({
                "schema_version": REPLAY_PROMOTION_SCHEMA_VERSION,
                "error": "replay_promotion_not_materialized",
                "message": "Replay-promotion lineage is not materialized yet. Run the promotion triage lane first."
            }))
            .unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(503)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
    }
}

fn handle_admin_replay_promotion_post(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    let payload = match crate::request_validation::parse_json_body(
        req.body(),
        crate::request_validation::MAX_ADMIN_JSON_BYTES,
    ) {
        Ok(value) => value,
        Err(err) => {
            return Response::new(
                400,
                format!("Invalid replay-promotion payload: {}", err),
            )
        }
    };
    let request = match serde_json::from_value::<ReplayPromotionIngestPayload>(payload) {
        Ok(request) => request,
        Err(err) => {
            return Response::new(
                400,
                format!("Invalid replay-promotion payload: {}", err),
            )
        }
    };

    let persisted = match persist_replay_promotion_payload(store, site_id, request) {
        Ok(payload) => payload,
        Err(ReplayPromotionPersistError::InvalidPayload(err)) => return Response::new(400, err),
        Err(ReplayPromotionPersistError::PersistFailed(err)) => return Response::new(500, err),
    };
    crate::observability::hot_read_projection::refresh_after_admin_mutation(store, site_id);

    crate::admin::log_event(
        store,
        &crate::admin::EventLogEntry {
            ts: persisted.generated_at_unix,
            event: crate::admin::EventType::AdminAction,
            ip: None,
            reason: Some("replay_promotion_materialized".to_string()),
            outcome: Some(if persisted.summary.blocking_required {
                "blocking_required".to_string()
            } else {
                "materialized".to_string()
            }),
            admin: Some(crate::admin::auth::get_admin_id(req)),
        },
    );

    let body = serde_json::to_string(&json!({
        "updated": true,
        "replay_promotion": persisted,
        "summary": replay_promotion_summary(&load_replay_promotion_payload(store, site_id).expect("payload still present")),
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
    use super::handle_admin_replay_promotion;
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

    fn replay_request(method: Method, body: serde_json::Value) -> Request {
        let mut builder = Request::builder();
        builder
            .method(method)
            .uri("/admin/replay-promotion")
            .body(serde_json::to_vec(&body).expect("body serializes"));
        builder.build()
    }

    fn sample_body() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "adversarial-promotion.v1",
            "generated_at_unix": 1_700_000_200u64,
            "frontier": {
                "frontier_mode": "multi_provider_playoff",
                "provider_count": 2,
                "diversity_confidence": "higher"
            },
            "summary": {
                "total_findings": 2,
                "replay_candidates": 1,
                "classification_counts": {
                    "confirmed_reproducible": 1,
                    "not_reproducible": 0,
                    "needs_manual_review": 0
                },
                "confirmed_regression_count": 1,
                "novel_confirmed_regression_count": 1,
                "false_discovery_rate_percent": 0.0,
                "provider_outage_impact_percent": 0.0,
                "blocking_required": true
            },
            "hybrid_governance": {
                "thresholds_passed": true,
                "failures": [],
                "observed": {
                    "deterministic_confirmation_rate_percent": 100.0,
                    "false_discovery_rate_percent": 0.0,
                    "overdue_owner_review_count": 0
                }
            },
            "discovery_quality_metrics": {
                "candidate_count": 2,
                "generated_candidate_count": 1,
                "novel_confirmed_regressions": 1,
                "false_discovery_rate_percent": 0.0,
                "provider_outage_impact_percent": 0.0,
                "provider_outage_status": "healthy",
                "blocking_requires_deterministic_confirmation": true
            },
            "lineage": [{
                "finding_id": "simf-001",
                "candidate_id": "cand-001",
                "scenario_id": "sim_t4_a",
                "classification": "confirmed_reproducible",
                "source_lane": "emergent_exploration",
                "deterministic_replay_lane": "deterministic_conformance",
                "release_blocking_authority": true,
                "generated_candidate": {
                    "generation_kind": "mutation",
                    "mutation_class": "retry_strategy",
                    "behavioral_class": "timing_variation",
                    "novelty_score": 0.72
                },
                "candidate": {
                    "scenario_family": "cdp_high_confidence_deny",
                    "path": "/sim/public/",
                    "expected_outcome": "deny_temp",
                    "observed_outcome": "deny_temp",
                    "severity": "high",
                    "risk": "high"
                },
                "deterministic_confirmation": {
                    "replay_status": "ok"
                },
                "promotion": {
                    "owner_review_required": true,
                    "owner_disposition": "pending",
                    "owner_disposition_due_at_unix": 1_700_172_800u64,
                    "blocking_regression": true,
                    "promoted_scenario": {
                        "id": "frontier_regression_simf-001"
                    },
                    "review_notes": [
                        "owner review remains required."
                    ]
                }
            }]
        })
    }

    #[test]
    fn get_replay_promotion_returns_503_until_materialized() {
        let store = TestStore::default();
        let req = replay_request(Method::Get, serde_json::Value::Null);

        let resp = handle_admin_replay_promotion(&req, &store, "default");

        assert_eq!(*resp.status(), 503);
        let payload: serde_json::Value = serde_json::from_slice(resp.body()).expect("json body");
        assert_eq!(
            payload.get("error").and_then(|value| value.as_str()),
            Some("replay_promotion_not_materialized")
        );
    }

    #[test]
    fn post_replay_promotion_materializes_contract_and_get_returns_it() {
        let store = TestStore::default();
        let post_req = replay_request(Method::Post, sample_body());

        let post_resp = handle_admin_replay_promotion(&post_req, &store, "default");

        assert_eq!(*post_resp.status(), 200);
        let post_payload: serde_json::Value =
            serde_json::from_slice(post_resp.body()).expect("json body");
        assert_eq!(
            post_payload
                .get("summary")
                .and_then(|value| value.get("availability"))
                .and_then(|value| value.as_str()),
            Some("materialized")
        );
        assert_eq!(
            post_payload
                .get("summary")
                .and_then(|value| value.get("evidence_status"))
                .and_then(|value| value.as_str()),
            Some("advisory_only")
        );
        assert_eq!(
            post_payload
                .get("summary")
                .and_then(|value| value.get("tuning_eligible"))
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        let get_req = replay_request(Method::Get, serde_json::Value::Null);
        let get_resp = handle_admin_replay_promotion(&get_req, &store, "default");
        assert_eq!(*get_resp.status(), 200);
        let get_payload: serde_json::Value =
            serde_json::from_slice(get_resp.body()).expect("json body");
        assert_eq!(
            get_payload
                .get("schema_version")
                .and_then(|value| value.as_str()),
            Some("replay_promotion_v1")
        );
        assert_eq!(
            get_payload
                .get("summary")
                .and_then(|value| value.get("blocking_required"))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
    }
}
