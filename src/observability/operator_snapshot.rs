use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::challenge::KeyValueStore;
use crate::config::AllowedActionsSurface;
use crate::observability::benchmark_results::{
    build_benchmark_results_from_snapshot_sections, BenchmarkResultsPayload,
};
use crate::observability::hot_read_contract::{
    operator_snapshot_component_contracts, HotReadOwnershipTier, TelemetryBasis, TelemetryExactness,
};
use crate::observability::monitoring::{
    HumanFrictionSegmentRow, MonitoringSummary, RequestOutcomeLaneSummaryRow,
    RequestOutcomeScopeSummaryRow,
};
use super::operator_objectives_store::load_or_seed_operator_objectives;
use super::operator_snapshot_live_traffic::{
    adversary_sim_section, human_friction_row, lane_row, live_traffic_section, scope_row,
};
use super::operator_snapshot_objectives::DEFAULT_WINDOW_HOURS;
use super::operator_snapshot_runtime_posture::{runtime_posture, runtime_shadow_mode};
use super::operator_snapshot_verified_identity::verified_identity_summary;
use super::replay_promotion::load_replay_promotion_summary;

pub(crate) use super::operator_snapshot_live_traffic::{
    OperatorSnapshotAdversarySim, OperatorSnapshotLane, OperatorSnapshotLiveTraffic,
    OperatorSnapshotRecentSimRun, OperatorSnapshotShadowMode,
};
pub(crate) use super::benchmark_comparison::{
    BenchmarkComparableSnapshot, BenchmarkEpisodeFamilyDelta, BenchmarkHomeostasisSummary,
};
pub(crate) use super::operator_snapshot_objectives::{
    OperatorObjectiveBudget, OperatorObjectivesProfile, RecursiveImprovementGameContract,
};
pub(crate) use super::operator_snapshot_non_human::OperatorSnapshotNonHumanTrafficSummary;
pub(crate) use super::operator_snapshot_recent_changes::{
    OperatorSnapshotRecentChange, OperatorSnapshotRecentChanges,
};
pub(crate) use super::operator_snapshot_runtime_posture::OperatorSnapshotRuntimePosture;
pub(crate) use super::operator_snapshot_verified_identity::OperatorSnapshotVerifiedIdentitySummary;
pub(crate) use super::replay_promotion::ReplayPromotionSummary;

pub(crate) const OPERATOR_SNAPSHOT_SCHEMA_VERSION: &str = "operator_snapshot_v1";
const DEFAULT_RECENT_CHANGE_ROWS: usize = 6;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotWindow {
    pub start_ts: u64,
    pub end_ts: u64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotSectionMetadata {
    pub exactness: TelemetryExactness,
    pub basis: TelemetryBasis,
    pub ownership_tier: HotReadOwnershipTier,
    pub refreshed_at_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorBudgetDistanceRow {
    pub budget_id: String,
    pub metric: String,
    pub eligible_requests: u64,
    pub current: f64,
    pub target: f64,
    pub delta: f64,
    pub near_limit: f64,
    pub status: String,
    pub exactness: String,
    pub basis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct OperatorBudgetDistanceSummary {
    pub rows: Vec<OperatorBudgetDistanceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotEpisodeEvaluationContext {
    pub objective_revision: String,
    pub profile_id: String,
    pub subject_kind: String,
    pub comparison_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotEpisodeProposal {
    pub patch_family: String,
    pub patch: serde_json::Value,
    pub expected_impact: String,
    pub confidence: String,
    pub controller_status: String,
    pub canary_requirement: String,
    pub matched_group_ids: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotEpisodeRecord {
    pub episode_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    pub completed_at_ts: u64,
    pub trigger_source: String,
    pub evaluation_context: OperatorSnapshotEpisodeEvaluationContext,
    pub baseline_scorecard: BenchmarkComparableSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<OperatorSnapshotEpisodeProposal>,
    pub proposal_status: String,
    pub watch_window_result: String,
    pub retain_or_rollback: String,
    pub benchmark_deltas: Vec<BenchmarkEpisodeFamilyDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hard_guardrail_triggers: Vec<String>,
    pub cycle_judgment: String,
    pub homeostasis_eligible: bool,
    pub evidence_references: Vec<crate::observability::decision_ledger::OperatorDecisionEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotEpisodeArchive {
    pub schema_version: String,
    pub homeostasis: BenchmarkHomeostasisSummary,
    pub rows: Vec<OperatorSnapshotEpisodeRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotHotReadPayload {
    pub schema_version: String,
    pub generated_at: u64,
    pub window: OperatorSnapshotWindow,
    pub section_metadata: BTreeMap<String, OperatorSnapshotSectionMetadata>,
    pub objectives: OperatorObjectivesProfile,
    pub live_traffic: OperatorSnapshotLiveTraffic,
    pub shadow_mode: OperatorSnapshotShadowMode,
    pub adversary_sim: OperatorSnapshotAdversarySim,
    pub runtime_posture: OperatorSnapshotRuntimePosture,
    pub recent_changes: OperatorSnapshotRecentChanges,
    pub budget_distance: OperatorBudgetDistanceSummary,
    pub non_human_traffic: OperatorSnapshotNonHumanTrafficSummary,
    pub allowed_actions: AllowedActionsSurface,
    pub game_contract: RecursiveImprovementGameContract,
    pub episode_archive: OperatorSnapshotEpisodeArchive,
    pub benchmark_results: BenchmarkResultsPayload,
    pub verified_identity: OperatorSnapshotVerifiedIdentitySummary,
    pub replay_promotion: ReplayPromotionSummary,
}

pub(crate) fn operator_snapshot_watch_window_hours(summary_hours: u64) -> u64 {
    summary_hours.max(DEFAULT_WINDOW_HOURS)
}

pub(crate) fn operator_snapshot_recent_changes_limit() -> usize {
    DEFAULT_RECENT_CHANGE_ROWS
}

pub(crate) fn build_operator_snapshot_payload<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    generated_at_ts: u64,
    summary: &MonitoringSummary,
    recent_sim_runs: &[OperatorSnapshotRecentSimRun],
    recent_changes: OperatorSnapshotRecentChanges,
    summary_refreshed_at_ts: u64,
    recent_sim_runs_refreshed_at_ts: u64,
    recent_changes_refreshed_at_ts: u64,
) -> OperatorSnapshotHotReadPayload {
    let objectives = load_or_seed_operator_objectives(store, site_id, generated_at_ts);
    let window_hours = operator_snapshot_watch_window_hours(summary.hours);
    let live_scope = scope_row(summary, "live", "ingress_primary", "enforced").cloned();
    let sim_scope = scope_row(summary, "adversary_sim", "ingress_primary", "enforced").cloned();
    let likely_human_lane = lane_row(
        summary,
        "live",
        "ingress_primary",
        "enforced",
        "likely_human",
    )
    .cloned();
    let suspicious_lane = lane_row(
        summary,
        "live",
        "ingress_primary",
        "enforced",
        "suspicious_automation",
    )
    .cloned();
    let human_friction = human_friction_row(summary, "enforced", "likely_human").cloned();
    let window = snapshot_window(generated_at_ts, window_hours);
    let live_traffic = live_traffic_section(
        live_scope.as_ref(),
        likely_human_lane.as_ref(),
        suspicious_lane.as_ref(),
        human_friction.as_ref(),
    );
    let shadow_mode = OperatorSnapshotShadowMode {
        enabled: runtime_shadow_mode(store, site_id),
        total_actions: summary.shadow.total_actions,
        pass_through_total: summary.shadow.pass_through_total,
        actions: summary.shadow.actions.clone(),
    };
    let adversary_sim = adversary_sim_section(sim_scope.as_ref(), recent_sim_runs);
    let runtime_posture = runtime_posture(store, site_id);
    let budget_distance = budget_distance_summary(
        &objectives,
        live_scope.as_ref(),
        likely_human_lane.as_ref(),
        suspicious_lane.as_ref(),
        human_friction.as_ref(),
    );
    let non_human_traffic =
        super::operator_snapshot_non_human::non_human_traffic_summary(summary, recent_sim_runs);
    let allowed_actions = crate::config::allowed_actions_v1();
    let legal_move_ring = crate::config::controller_legal_move_ring_v1();
    let game_contract = super::operator_snapshot_objectives::recursive_improvement_game_contract_v1(
        &objectives,
        &legal_move_ring,
    );
    let (episode_archive, episode_archive_refreshed_at_ts) =
        crate::admin::load_oversight_episode_archive(store, site_id, &game_contract);
    let cfg = crate::config::load_runtime_cached(store, site_id)
        .unwrap_or_else(|_| crate::config::defaults().clone());
    let verified_identity =
        verified_identity_summary(summary, &cfg, non_human_traffic.receipts.as_slice());
    let (replay_promotion, replay_promotion_refreshed_at_ts) =
        load_replay_promotion_summary(store, site_id);
    let prior_window_reference =
        crate::observability::benchmark_history::load_prior_window_reference(
            store,
            site_id,
            generated_at_ts,
        );
    let benchmark_results_refreshed_at_ts =
        summary_refreshed_at_ts.min(recent_sim_runs_refreshed_at_ts);
    let benchmark_results = build_benchmark_results_from_snapshot_sections(
        generated_at_ts,
        generated_at_ts,
        &window,
        &objectives,
        &live_traffic,
        &adversary_sim,
        &non_human_traffic,
        &budget_distance,
        summary,
        &cfg,
        &allowed_actions,
        &replay_promotion,
        prior_window_reference.as_ref(),
    );

    OperatorSnapshotHotReadPayload {
        schema_version: OPERATOR_SNAPSHOT_SCHEMA_VERSION.to_string(),
        generated_at: generated_at_ts,
        window,
        section_metadata: operator_snapshot_section_metadata(
            generated_at_ts,
            objectives.updated_at_ts,
            summary_refreshed_at_ts,
            recent_sim_runs_refreshed_at_ts,
            recent_changes_refreshed_at_ts,
            benchmark_results_refreshed_at_ts,
            episode_archive_refreshed_at_ts,
            summary_refreshed_at_ts,
            summary_refreshed_at_ts,
            replay_promotion_refreshed_at_ts,
        ),
        objectives,
        live_traffic,
        shadow_mode,
        adversary_sim,
        runtime_posture,
        recent_changes,
        budget_distance,
        non_human_traffic,
        allowed_actions,
        game_contract,
        episode_archive,
        benchmark_results,
        verified_identity,
        replay_promotion,
    }
}

fn snapshot_window(generated_at_ts: u64, hours: u64) -> OperatorSnapshotWindow {
    let duration_seconds = hours.saturating_mul(3600);
    OperatorSnapshotWindow {
        start_ts: generated_at_ts.saturating_sub(duration_seconds.saturating_sub(1)),
        end_ts: generated_at_ts,
        duration_seconds,
    }
}

fn operator_snapshot_section_metadata(
    generated_at_ts: u64,
    objectives_refreshed_at_ts: u64,
    summary_refreshed_at_ts: u64,
    recent_sim_runs_refreshed_at_ts: u64,
    recent_changes_refreshed_at_ts: u64,
    benchmark_results_refreshed_at_ts: u64,
    episode_archive_refreshed_at_ts: u64,
    non_human_traffic_refreshed_at_ts: u64,
    verified_identity_refreshed_at_ts: u64,
    replay_promotion_refreshed_at_ts: u64,
) -> BTreeMap<String, OperatorSnapshotSectionMetadata> {
    operator_snapshot_component_contracts()
        .iter()
        .map(|component| {
            let refreshed_at_ts = match component.key {
                "objectives" => objectives_refreshed_at_ts,
                "live_traffic" | "shadow_mode" | "budget_distance" => summary_refreshed_at_ts,
                "adversary_sim" => recent_sim_runs_refreshed_at_ts,
                "recent_changes" => recent_changes_refreshed_at_ts,
                "game_contract" => objectives_refreshed_at_ts,
                "benchmark_results" => benchmark_results_refreshed_at_ts,
                "episode_archive" => episode_archive_refreshed_at_ts,
                "non_human_traffic" => non_human_traffic_refreshed_at_ts,
                "verified_identity" => verified_identity_refreshed_at_ts,
                "replay_promotion" => {
                    if replay_promotion_refreshed_at_ts == 0 {
                        generated_at_ts
                    } else {
                        replay_promotion_refreshed_at_ts
                    }
                }
                _ => generated_at_ts,
            };
            (
                component.key.to_string(),
                OperatorSnapshotSectionMetadata {
                    exactness: component.exactness,
                    basis: component.basis,
                    ownership_tier: component.ownership_tier,
                    refreshed_at_ts,
                },
            )
        })
        .collect()
}


fn budget_distance_summary(
    objectives: &OperatorObjectivesProfile,
    live_scope: Option<&RequestOutcomeScopeSummaryRow>,
    likely_human_lane: Option<&RequestOutcomeLaneSummaryRow>,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
    human_friction: Option<&HumanFrictionSegmentRow>,
) -> OperatorBudgetDistanceSummary {
    let mut rows = Vec::new();
    for budget in &objectives.budgets {
        let row = match budget.metric.as_str() {
            "likely_human_friction_rate" => {
                build_friction_budget_row(budget, likely_human_lane, human_friction)
            }
            "suspicious_forwarded_request_rate" => {
                build_suspicious_forwarded_request_budget_row(budget, suspicious_lane)
            }
            "suspicious_forwarded_byte_rate" => {
                build_suspicious_forwarded_byte_budget_row(budget, suspicious_lane)
            }
            "suspicious_forwarded_latency_share" => {
                build_suspicious_forwarded_latency_budget_row(budget, live_scope, suspicious_lane)
            }
            _ => None,
        };
        if let Some(row) = row {
            rows.push(row);
        }
    }
    OperatorBudgetDistanceSummary { rows }
}

fn build_friction_budget_row(
    budget: &OperatorObjectiveBudget,
    likely_human_lane: Option<&RequestOutcomeLaneSummaryRow>,
    human_friction: Option<&HumanFrictionSegmentRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let friction = human_friction?;
    let (exactness, basis) = if let Some(lane) = likely_human_lane {
        (lane.exactness.clone(), lane.basis.clone())
    } else {
        ("derived".to_string(), "observed".to_string())
    };
    Some(budget_row(
        budget,
        friction.denominator_requests,
        friction.friction_rate,
        exactness,
        basis,
    ))
}

fn build_suspicious_forwarded_request_budget_row(
    budget: &OperatorObjectiveBudget,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let lane = suspicious_lane?;
    let current = ratio(lane.forwarded_requests, lane.total_requests);
    Some(budget_row(
        budget,
        lane.total_requests,
        current,
        lane.exactness.clone(),
        lane.basis.clone(),
    ))
}

fn build_suspicious_forwarded_byte_budget_row(
    budget: &OperatorObjectiveBudget,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let lane = suspicious_lane?;
    let total_bytes = lane
        .forwarded_response_bytes
        .saturating_add(lane.short_circuited_response_bytes)
        .saturating_add(lane.control_response_bytes);
    let current = ratio(lane.forwarded_response_bytes, total_bytes);
    Some(budget_row(
        budget,
        lane.total_requests,
        current,
        lane.exactness.clone(),
        lane.basis.clone(),
    ))
}

fn build_suspicious_forwarded_latency_budget_row(
    budget: &OperatorObjectiveBudget,
    live_scope: Option<&RequestOutcomeScopeSummaryRow>,
    suspicious_lane: Option<&RequestOutcomeLaneSummaryRow>,
) -> Option<OperatorBudgetDistanceRow> {
    let scope = live_scope?;
    let lane = suspicious_lane?;
    let current = ratio(
        lane.forwarded_upstream_latency_ms_total,
        scope.forwarded_upstream_latency_ms_total,
    );
    Some(budget_row(
        budget,
        lane.forwarded_requests,
        current,
        lane.exactness.clone(),
        lane.basis.clone(),
    ))
}

fn budget_row(
    budget: &OperatorObjectiveBudget,
    eligible_requests: u64,
    current: f64,
    exactness: String,
    basis: String,
) -> OperatorBudgetDistanceRow {
    let near_limit = budget.target * budget.near_limit_ratio;
    let status = if eligible_requests == 0 {
        "insufficient_evidence".to_string()
    } else if current <= near_limit {
        "inside_budget".to_string()
    } else if current <= budget.target {
        "near_limit".to_string()
    } else {
        "outside_budget".to_string()
    };
    OperatorBudgetDistanceRow {
        budget_id: budget.budget_id.clone(),
        metric: budget.metric.clone(),
        eligible_requests,
        current,
        target: budget.target,
        delta: current - budget.target,
        near_limit,
        status,
        exactness,
        basis,
    }
}

fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_operator_snapshot_payload, operator_snapshot_watch_window_hours,
        OperatorSnapshotRecentChanges, OperatorSnapshotRecentSimRun,
        OPERATOR_SNAPSHOT_SCHEMA_VERSION,
    };
    use crate::challenge::KeyValueStore;
    use crate::observability::hot_read_documents::{
        operator_snapshot_document_contract, operator_snapshot_document_key, HotReadDocumentEnvelope,
        HotReadDocumentMetadata, HotReadUpdateTrigger,
    };
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
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

    #[test]
    fn snapshot_payload_uses_persisted_objective_profile_and_typed_verified_identity_summary() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let watch_window_hours = operator_snapshot_watch_window_hours(summary.hours);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_000,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "run_001".to_string(),
                lane: "deterministic_black_box".to_string(),
                profile: "fast_smoke".to_string(),
                observed_fulfillment_modes: Vec::new(),
                observed_category_ids: Vec::new(),
                first_ts: 1_699_999_900,
                last_ts: 1_700_000_000,
                monitoring_event_count: 3,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
            }],
            OperatorSnapshotRecentChanges {
                lookback_seconds: watch_window_hours.saturating_mul(3).saturating_mul(3600),
                watch_window_seconds: watch_window_hours.saturating_mul(3600),
                rows: Vec::new(),
            },
            1_700_000_000,
            1_700_000_000,
            1_700_000_000,
        );

        assert_eq!(payload.schema_version, OPERATOR_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(payload.objectives.profile_id, "site_default_v1");
        assert_eq!(payload.objectives.schema_version, "operator_objectives_v1");
        assert_eq!(payload.objectives.category_postures.len(), 8);
        assert_eq!(
            payload
                .objectives
                .category_postures
                .iter()
                .find(|row| row.category_id.as_str() == "verified_beneficial_bot")
                .expect("verified beneficial category posture")
                .posture,
            "allowed"
        );
        assert!(payload
            .budget_distance
            .rows
            .iter()
            .any(|row| row.metric == "likely_human_friction_rate"));
        assert!(payload.recent_changes.rows.is_empty());
        assert_eq!(
            payload.non_human_traffic.taxonomy.schema_version,
            "non_human_taxonomy_v1"
        );
        assert!(payload
            .non_human_traffic
            .taxonomy
            .categories
            .iter()
            .any(|category| category.category_id.as_str() == "agent_on_behalf_of_human"));
        assert_eq!(payload.allowed_actions.schema_version, "allowed_actions_v1");
        assert!(payload
            .allowed_actions
            .allowed_group_ids
            .contains(&"not_a_bot.policy".to_string()));
        assert_eq!(
            payload.benchmark_results.schema_version,
            "benchmark_results_v1"
        );
        assert_eq!(
            payload.benchmark_results.suite_version,
            "benchmark_suite_v1"
        );
        assert_eq!(
            payload.benchmark_results,
            crate::observability::benchmark_results::build_benchmark_results_from_snapshot_sections(
                payload.generated_at,
                1_700_000_000,
                &payload.window,
                &payload.objectives,
                &payload.live_traffic,
                &payload.adversary_sim,
                &payload.non_human_traffic,
                &payload.budget_distance,
                &summary,
                &crate::config::defaults(),
                &payload.allowed_actions,
                &payload.replay_promotion,
                None,
            )
        );
        assert_eq!(payload.verified_identity.availability, "supported");
        assert!(payload.verified_identity.enabled);
        assert_eq!(payload.verified_identity.attempts, 0);
        assert_eq!(payload.replay_promotion.availability, "not_materialized");
        assert_eq!(
            payload.game_contract.schema_version,
            "recursive_improvement_game_contract_v1"
        );
        assert_eq!(
            payload.episode_archive.schema_version,
            "oversight_episode_archive_v1"
        );
        assert!(payload.episode_archive.rows.is_empty());
        assert_eq!(
            payload.episode_archive.homeostasis.status,
            "not_enough_completed_cycles"
        );
        assert_eq!(
            payload.game_contract.legal_move_ring.allowed_actions_schema_version,
            "allowed_actions_v1"
        );
        assert!(payload
            .game_contract
            .safety_gates
            .iter()
            .any(|gate| gate.gate_id == "manual_review_guardrail"));
    }

    #[test]
    fn snapshot_payload_projects_scrapling_request_native_category_receipts() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        let summary = summarize_with_store(&store, 24, 10);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_050,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_040,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: None,
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_050,
            1_700_000_050,
            1_700_000_050,
        );

        assert_eq!(payload.non_human_traffic.readiness.status, "ready");
        assert_eq!(payload.non_human_traffic.coverage.covered_category_count, 3);
        assert!(payload
            .non_human_traffic
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "ai_scraper_bot"));
        assert!(payload
            .non_human_traffic
            .receipts
            .iter()
            .any(|receipt| receipt.category_id == "http_agent"));
    }

    #[test]
    fn snapshot_payload_projects_recent_run_owned_surface_coverage() {
        let store = TestStore::new();
        let summary = summarize_with_store(&store, 24, 10);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_060,
            &summary,
            &[OperatorSnapshotRecentSimRun {
                run_id: "simrun-request-native".to_string(),
                lane: "scrapling_traffic".to_string(),
                profile: "scrapling_runtime_lane".to_string(),
                observed_fulfillment_modes: vec![
                    "crawler".to_string(),
                    "bulk_scraper".to_string(),
                    "http_agent".to_string(),
                ],
                observed_category_ids: vec![
                    "indexing_bot".to_string(),
                    "ai_scraper_bot".to_string(),
                    "http_agent".to_string(),
                ],
                first_ts: 1_700_000_000,
                last_ts: 1_700_000_040,
                monitoring_event_count: 9,
                defense_delta_count: 2,
                ban_outcome_count: 0,
                owned_surface_coverage: Some(
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary {
                        overall_status: "covered".to_string(),
                        required_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        satisfied_surface_ids: vec![
                            "public_path_traversal".to_string(),
                            "challenge_routing".to_string(),
                        ],
                        blocking_surface_ids: Vec::new(),
                        receipts: vec![
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "public_path_traversal".to_string(),
                                success_contract: "should_pass_some".to_string(),
                                coverage_status: "pass_observed".to_string(),
                                satisfied: true,
                                attempt_count: 2,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/catalog?page=1".to_string(),
                                sample_response_status: Some(200),
                            },
                            crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                                surface_id: "challenge_routing".to_string(),
                                success_contract: "mixed_outcomes".to_string(),
                                coverage_status: "pass_observed".to_string(),
                                satisfied: true,
                                attempt_count: 3,
                                sample_request_method: "GET".to_string(),
                                sample_request_path: "/sim/public/search?q=scrapling".to_string(),
                                sample_response_status: Some(200),
                            },
                        ],
                    },
                ),
            }],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_060,
            1_700_000_060,
            1_700_000_060,
        );

        let recent_run = payload
            .adversary_sim
            .recent_runs
            .iter()
            .find(|row| row.run_id == "simrun-request-native")
            .expect("recent run");
        let owned_surface_coverage = recent_run
            .owned_surface_coverage
            .as_ref()
            .expect("owned surface coverage");
        assert_eq!(owned_surface_coverage.overall_status, "covered");
        assert_eq!(owned_surface_coverage.required_surface_ids.len(), 2);
        assert!(owned_surface_coverage.blocking_surface_ids.is_empty());
    }

    #[test]
    fn snapshot_payload_projects_suspicious_forwarded_latency_budget_row() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 90,
                forwarded_upstream_latency_ms: Some(30),
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Derived,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Mixed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 120,
                forwarded_upstream_latency_ms: Some(70),
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_075,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_075,
            1_700_000_075,
            1_700_000_075,
        );

        let row = payload
            .budget_distance
            .rows
            .iter()
            .find(|row| row.metric == "suspicious_forwarded_latency_share")
            .expect("latency-share budget row");

        assert_eq!(payload.live_traffic.forwarded_upstream_latency_ms_total, 100);
        assert_eq!(
            payload
                .live_traffic
                .suspicious_automation
                .as_ref()
                .expect("suspicious lane")
                .forwarded_upstream_latency_ms_total,
            70
        );
        assert_eq!(row.eligible_requests, 1);
        assert!((row.current - 0.7).abs() < 0.000_001);
        assert_eq!(row.status, "outside_budget");
    }

    #[test]
    fn snapshot_payload_uses_prior_operator_snapshot_as_benchmark_reference_when_available() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        let previous_summary = summarize_with_store(&store, 24, 10);
        let previous_payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_100,
            &previous_summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_100,
            1_700_000_100,
            1_700_000_100,
        );
        let previous_document = HotReadDocumentEnvelope {
            metadata: HotReadDocumentMetadata {
                schema_version: operator_snapshot_document_contract()
                    .schema_version
                    .to_string(),
                site_id: "default".to_string(),
                generated_at_ts: 1_700_000_100,
                trigger: HotReadUpdateTrigger::RepairRebuild,
            },
            payload: previous_payload,
        };
        store
            .set(
                &operator_snapshot_document_key("default"),
                &serde_json::to_vec(&previous_document).expect("snapshot document"),
            )
            .expect("seed prior operator snapshot");

        let fresh_store = TestStore::new();
        let fresh_summary = summarize_with_store(&fresh_store, 24, 10);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_200,
            &fresh_summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );

        assert_eq!(
            payload.benchmark_results.baseline_reference.status,
            "available"
        );
        assert_eq!(
            payload.benchmark_results.baseline_reference.reference_kind,
            "prior_window"
        );
        assert_eq!(
            payload.benchmark_results.baseline_reference.generated_at,
            Some(1_700_000_100)
        );
        assert_ne!(payload.benchmark_results.improvement_status, "not_available");
    }

    #[test]
    fn snapshot_payload_keeps_live_and_adversary_sim_sections_separate() {
        let store = TestStore::new();
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::LikelyHuman,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::ShortCircuited,
                response_kind: ResponseKind::NotABot,
                http_status: 200,
                response_bytes: 45,
                forwarded_upstream_latency_ms: None,
                forward_attempted: false,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            &store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Derived,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Mixed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 256,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );

        let summary = summarize_with_store(&store, 24, 10);
        let watch_window_hours = operator_snapshot_watch_window_hours(summary.hours);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_000,
            &summary,
            &[],
            OperatorSnapshotRecentChanges {
                lookback_seconds: watch_window_hours.saturating_mul(3).saturating_mul(3600),
                watch_window_seconds: watch_window_hours.saturating_mul(3600),
                rows: Vec::new(),
            },
            1_700_000_000,
            1_700_000_000,
            1_700_000_000,
        );

        assert_eq!(payload.live_traffic.traffic_origin, "live");
        assert_eq!(payload.live_traffic.total_requests, 1);
        assert_eq!(
            payload
                .live_traffic
                .likely_human
                .as_ref()
                .expect("likely human lane")
                .total_requests,
            1
        );
        assert!(payload.live_traffic.suspicious_automation.is_none());
        assert_eq!(payload.adversary_sim.traffic_origin, "adversary_sim");
        assert_eq!(payload.adversary_sim.total_requests, 1);
        assert_eq!(payload.non_human_traffic.taxonomy.categories.len(), 8);
        assert_eq!(
            payload
                .section_metadata
                .get("budget_distance")
                .expect("budget distance metadata")
                .exactness,
            crate::observability::hot_read_contract::TelemetryExactness::Derived
        );
        assert_eq!(
            payload
                .section_metadata
                .get("non_human_traffic")
                .expect("non human traffic metadata")
                .basis,
            crate::observability::hot_read_contract::TelemetryBasis::Mixed
        );
    }

    #[test]
    fn snapshot_payload_surfaces_materialized_replay_promotion_summary() {
        let store = TestStore::new();
        crate::observability::replay_promotion::persist_replay_promotion_payload(
            &store,
            "default",
            serde_json::from_value(serde_json::json!({
                "schema_version": "adversarial-promotion.v1",
                "generated_at_unix": 1_700_000_150u64,
                "frontier": {
                    "frontier_mode": "multi_provider_playoff",
                    "provider_count": 2,
                    "diversity_confidence": "higher"
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
                        "path": "/sim/public/search",
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
            }))
            .expect("replay payload parses"),
        )
        .expect("replay promotion persists");

        let summary = summarize_with_store(&store, 24, 10);
        let payload = build_operator_snapshot_payload(
            &store,
            "default",
            1_700_000_200,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );

        assert_eq!(payload.replay_promotion.availability, "materialized");
        assert_eq!(payload.replay_promotion.evidence_status, "advisory_only");
        assert!(!payload.replay_promotion.tuning_eligible);
        assert_eq!(payload.replay_promotion.protected_basis, "none");
        assert_eq!(payload.replay_promotion.replay_candidates, 1);
        assert_eq!(payload.replay_promotion.pending_owner_review_count, 1);
        assert_eq!(payload.replay_promotion.lineage.len(), 1);
        assert_eq!(payload.benchmark_results.replay_promotion, payload.replay_promotion);
        assert_eq!(
            payload
                .section_metadata
                .get("replay_promotion")
                .expect("replay promotion metadata")
                .refreshed_at_ts,
            1_700_000_150
        );
    }
}
