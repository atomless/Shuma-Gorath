use once_cell::sync::Lazy;
use spin_sdk::http::{Method, Request, Response};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

#[derive(Default)]
pub(crate) struct InMemoryStore {
    map: Mutex<HashMap<String, Vec<u8>>>,
}

impl crate::challenge::KeyValueStore for InMemoryStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        let map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.get(key).cloned())
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), ()> {
        let mut map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.remove(key);
        Ok(())
    }

    fn get_keys(&self) -> Result<Vec<String>, ()> {
        let map = self
            .map
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Ok(map.keys().cloned().collect())
    }
}

impl crate::maze::state::MazeStateStore for InMemoryStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, ()> {
        crate::challenge::KeyValueStore::get(self, key)
    }

    fn set(&self, key: &str, value: &[u8]) -> Result<(), ()> {
        crate::challenge::KeyValueStore::set(self, key, value)
    }
}

static ENV_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
    let guard = ENV_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    crate::config::clear_runtime_cache_for_tests();
    guard
}

pub(crate) fn request_with_headers(path: &str, headers: &[(&str, &str)]) -> Request {
    request_with_method_and_headers(Method::Get, path, headers)
}

pub(crate) fn request_with_method_and_headers(
    method: Method,
    path: &str,
    headers: &[(&str, &str)],
) -> Request {
    let mut builder = Request::builder();
    builder.method(method).uri(path);
    for (key, value) in headers {
        builder.header(*key, *value);
    }
    builder.build()
}

pub(crate) fn has_header(resp: &Response, name: &str) -> bool {
    resp.headers()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

pub(crate) fn seed_apply_ready_snapshot<S: crate::challenge::KeyValueStore>(
    store: &S,
    cfg: crate::config::Config,
) {
    seed_candidate_snapshot(store, cfg, 1_700_000_200, 0.42, "outside_budget");
}

pub(crate) fn seed_candidate_snapshot<S: crate::challenge::KeyValueStore>(
    store: &S,
    cfg: crate::config::Config,
    generated_at_ts: u64,
    suspicious_forwarded_request_rate: f64,
    overall_status: &str,
) {
    store
        .set(
            "config:default",
            &crate::config::serialize_persisted_kv_config(&cfg).expect("cfg serializes"),
        )
        .expect("config seed");
    crate::observability::monitoring::record_request_outcome(
        store,
        &crate::runtime::request_outcome::RenderedRequestOutcome {
            traffic_origin: crate::runtime::request_outcome::TrafficOrigin::Live,
            measurement_scope:
                crate::runtime::traffic_classification::MeasurementScope::IngressPrimary,
            route_action_family:
                crate::runtime::traffic_classification::RouteActionFamily::PublicContent,
            execution_mode: crate::runtime::effect_intents::ExecutionMode::Enforced,
            traffic_lane: Some(crate::runtime::request_outcome::RequestOutcomeLane {
                lane: crate::runtime::traffic_classification::TrafficLane::DeclaredCrawler,
                exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
            }),
            non_human_category: None,
            outcome_class: crate::runtime::request_outcome::RequestOutcomeClass::Forwarded,
            response_kind: crate::runtime::request_outcome::ResponseKind::ForwardAllow,
            http_status: 200,
            response_bytes: 2_000,
            forwarded_upstream_latency_ms: None,
            forward_attempted: true,
            forward_failure_class: None,
            intended_action: None,
            policy_source: crate::runtime::traffic_classification::PolicySource::CleanAllow,
        },
    );
    let summary = crate::observability::monitoring::summarize_with_store(store, 24, 10);
    let mut payload = crate::observability::operator_snapshot::build_operator_snapshot_payload(
        store,
        "default",
        generated_at_ts,
        &summary,
        &[],
        crate::observability::operator_snapshot::OperatorSnapshotRecentChanges::default(),
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
    payload.benchmark_results.non_human_classification = payload.non_human_traffic.readiness.clone();
    payload.benchmark_results.non_human_coverage =
        payload.non_human_traffic.coverage.compact_for_benchmark();
    payload.benchmark_results.tuning_eligibility.status = "eligible".to_string();
    payload.benchmark_results.tuning_eligibility.blockers.clear();
    payload.benchmark_results.escalation_hint.availability = "partial_support".to_string();
    payload.benchmark_results.escalation_hint.decision = "config_tuning_candidate".to_string();
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
            metric.delta = metric
                .target
                .map(|target| suspicious_forwarded_request_rate - target);
            metric.status = overall_status.to_string();
            metric.comparison_status = family.comparison_status.clone();
        }
    }
    let document = crate::observability::hot_read_documents::HotReadDocumentEnvelope {
        metadata: crate::observability::hot_read_documents::HotReadDocumentMetadata {
            schema_version: crate::observability::hot_read_documents::operator_snapshot_document_contract()
                .schema_version
                .to_string(),
            site_id: "default".to_string(),
            generated_at_ts,
            trigger: crate::observability::hot_read_documents::HotReadUpdateTrigger::RepairRebuild,
        },
        payload,
    };
    store
        .set(
            crate::observability::hot_read_documents::operator_snapshot_document_key("default")
                .as_str(),
            &serde_json::to_vec(&document).expect("document serializes"),
        )
        .expect("snapshot seed");
}

#[cfg(test)]
pub(crate) fn seed_canary_only_human_only_private_objectives<S: crate::challenge::KeyValueStore>(
    store: &S,
) {
    let mut profile =
        crate::observability::operator_snapshot_objectives::human_only_private_operator_objectives(
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
