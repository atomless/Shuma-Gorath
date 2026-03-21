use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

pub(crate) const REPLAY_PROMOTION_SCHEMA_VERSION: &str = "replay_promotion_v1";
const REPLAY_PROMOTION_INGEST_SCHEMA_VERSION: &str = "adversarial-promotion.v1";
const REPLAY_PROMOTION_PREFIX: &str = "replay_promotion:v1";
const REPLAY_PROMOTION_MAX_LINEAGE_ROWS: usize = 12;
const REPLAY_PROMOTION_SUMMARY_MAX_LINEAGE_ROWS: usize = 4;
const REPLAY_PROMOTION_MAX_FAILURE_ROWS: usize = 4;
const REPLAY_PROMOTION_MAX_REVIEW_NOTES: usize = 3;
const REPLAY_PROMOTION_MAX_TEXT_CHARS: usize = 160;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReplayPromotionPersistError {
    InvalidPayload(&'static str),
    PersistFailed(&'static str),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionSource {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub report_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub attack_plan_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub manifest_path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub frontier_status_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionFrontier {
    #[serde(default)]
    pub frontier_mode: String,
    #[serde(default)]
    pub provider_count: u64,
    #[serde(default)]
    pub diversity_confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionLaneDescriptor {
    #[serde(default)]
    pub lane_id: String,
    #[serde(default)]
    pub release_blocking: bool,
    #[serde(default)]
    pub authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionLaneMetadata {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub contract_path: String,
    #[serde(default)]
    pub deterministic_conformance_lane: ReplayPromotionLaneDescriptor,
    #[serde(default)]
    pub emergent_exploration_lane: ReplayPromotionLaneDescriptor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionPipeline {
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub blocking_requires_deterministic_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ReplayPromotionPolicy {
    #[serde(default)]
    pub deterministic_oracle_authoritative: bool,
    #[serde(default)]
    pub single_provider_self_play_requires_owner_review: bool,
    #[serde(default)]
    pub multi_provider_playoff_requires_owner_review: bool,
    #[serde(default)]
    pub blocking_requires_deterministic_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ReplayPromotionHybridGovernance {
    #[serde(default)]
    pub thresholds_passed: bool,
    #[serde(default)]
    pub failures: Vec<String>,
    #[serde(default)]
    pub deterministic_confirmation_rate_percent: f64,
    #[serde(default)]
    pub false_discovery_rate_percent: f64,
    #[serde(default)]
    pub overdue_owner_review_count: u64,
}

impl Default for ReplayPromotionHybridGovernance {
    fn default() -> Self {
        Self {
            thresholds_passed: false,
            failures: Vec::new(),
            deterministic_confirmation_rate_percent: 0.0,
            false_discovery_rate_percent: 0.0,
            overdue_owner_review_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ReplayPromotionDiscoveryQualityMetrics {
    #[serde(default)]
    pub candidate_count: u64,
    #[serde(default)]
    pub generated_candidate_count: u64,
    #[serde(default)]
    pub novel_confirmed_regressions: u64,
    #[serde(default)]
    pub false_discovery_rate_percent: f64,
    #[serde(default)]
    pub provider_outage_impact_percent: f64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider_outage_status: String,
    #[serde(default)]
    pub blocking_requires_deterministic_confirmation: bool,
}

impl Default for ReplayPromotionDiscoveryQualityMetrics {
    fn default() -> Self {
        Self {
            candidate_count: 0,
            generated_candidate_count: 0,
            novel_confirmed_regressions: 0,
            false_discovery_rate_percent: 0.0,
            provider_outage_impact_percent: 0.0,
            provider_outage_status: String::new(),
            blocking_requires_deterministic_confirmation: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ReplayPromotionLineageRow {
    pub finding_id: String,
    pub candidate_id: String,
    pub scenario_id: String,
    pub source_lane: String,
    pub deterministic_replay_lane: String,
    pub classification: String,
    pub replay_status: String,
    pub release_blocking_authority: bool,
    pub scenario_family: String,
    pub path: String,
    pub expected_outcome: String,
    pub observed_outcome: String,
    pub severity: String,
    pub risk: String,
    pub generation_kind: String,
    pub mutation_class: String,
    pub behavioral_class: String,
    pub novelty_score: f64,
    pub owner_review_required: bool,
    pub owner_disposition: String,
    pub owner_disposition_due_at_unix: u64,
    pub blocking_regression: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promoted_scenario_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub review_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ReplayPromotionOutcomeSummary {
    pub total_findings: u64,
    pub replay_candidates: u64,
    pub confirmed_reproducible_count: u64,
    pub not_reproducible_count: u64,
    pub needs_manual_review_count: u64,
    pub pending_owner_review_count: u64,
    pub promoted_scenario_count: u64,
    pub blocking_required: bool,
    pub false_discovery_rate_percent: f64,
    pub provider_outage_impact_percent: f64,
    pub novel_confirmed_regression_count: u64,
}

impl Default for ReplayPromotionOutcomeSummary {
    fn default() -> Self {
        Self {
            total_findings: 0,
            replay_candidates: 0,
            confirmed_reproducible_count: 0,
            not_reproducible_count: 0,
            needs_manual_review_count: 0,
            pending_owner_review_count: 0,
            promoted_scenario_count: 0,
            blocking_required: false,
            false_discovery_rate_percent: 0.0,
            provider_outage_impact_percent: 0.0,
            novel_confirmed_regression_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct ReplayPromotionPayload {
    pub schema_version: String,
    pub generated_at_unix: u64,
    #[serde(default)]
    pub source: ReplayPromotionSource,
    #[serde(default)]
    pub frontier: ReplayPromotionFrontier,
    #[serde(default)]
    pub lane_metadata: ReplayPromotionLaneMetadata,
    #[serde(default)]
    pub promotion_pipeline: ReplayPromotionPipeline,
    #[serde(default)]
    pub policy: ReplayPromotionPolicy,
    #[serde(default)]
    pub hybrid_governance: ReplayPromotionHybridGovernance,
    #[serde(default)]
    pub discovery_quality_metrics: ReplayPromotionDiscoveryQualityMetrics,
    #[serde(default)]
    pub summary: ReplayPromotionOutcomeSummary,
    #[serde(default)]
    pub lineage: Vec<ReplayPromotionLineageRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub(crate) struct ReplayPromotionSummary {
    pub availability: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at_unix: Option<u64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub frontier_mode: String,
    #[serde(default)]
    pub provider_count: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub diversity_confidence: String,
    #[serde(default)]
    pub replay_candidates: u64,
    #[serde(default)]
    pub confirmed_reproducible_count: u64,
    #[serde(default)]
    pub not_reproducible_count: u64,
    #[serde(default)]
    pub needs_manual_review_count: u64,
    #[serde(default)]
    pub pending_owner_review_count: u64,
    #[serde(default)]
    pub promoted_scenario_count: u64,
    #[serde(default)]
    pub blocking_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thresholds_passed: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threshold_failures: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lineage: Vec<ReplayPromotionLineageRow>,
}

impl ReplayPromotionSummary {
    pub(crate) fn not_materialized() -> Self {
        Self {
            availability: "not_materialized".to_string(),
            generated_at_unix: None,
            frontier_mode: String::new(),
            provider_count: 0,
            diversity_confidence: String::new(),
            replay_candidates: 0,
            confirmed_reproducible_count: 0,
            not_reproducible_count: 0,
            needs_manual_review_count: 0,
            pending_owner_review_count: 0,
            promoted_scenario_count: 0,
            blocking_required: false,
            thresholds_passed: None,
            threshold_failures: Vec::new(),
            lineage: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReplayPromotionIngestPayload {
    pub schema_version: String,
    #[serde(default)]
    pub generated_at_unix: u64,
    #[serde(default)]
    pub source: ReplayPromotionSource,
    #[serde(default)]
    pub frontier: ReplayPromotionIngestFrontier,
    #[serde(default)]
    pub lane_metadata: ReplayPromotionLaneMetadata,
    #[serde(default)]
    pub promotion_pipeline: ReplayPromotionPipeline,
    #[serde(default)]
    pub policy: ReplayPromotionPolicy,
    #[serde(default)]
    pub hybrid_governance: ReplayPromotionIngestHybridGovernance,
    #[serde(default)]
    pub discovery_quality_metrics: ReplayPromotionIngestDiscoveryQualityMetrics,
    #[serde(default)]
    pub summary: ReplayPromotionIngestSummary,
    #[serde(default)]
    pub lineage: Vec<ReplayPromotionIngestLineageRow>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestFrontier {
    #[serde(default)]
    pub frontier_mode: String,
    #[serde(default)]
    pub provider_count: u64,
    #[serde(default)]
    pub diversity_confidence: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestHybridGovernance {
    #[serde(default)]
    pub thresholds_passed: bool,
    #[serde(default)]
    pub failures: Vec<String>,
    #[serde(default)]
    pub observed: ReplayPromotionIngestHybridGovernanceObserved,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestHybridGovernanceObserved {
    #[serde(default)]
    pub deterministic_confirmation_rate_percent: f64,
    #[serde(default)]
    pub false_discovery_rate_percent: f64,
    #[serde(default)]
    pub overdue_owner_review_count: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestDiscoveryQualityMetrics {
    #[serde(default)]
    pub candidate_count: u64,
    #[serde(default)]
    pub generated_candidate_count: u64,
    #[serde(default)]
    pub novel_confirmed_regressions: u64,
    #[serde(default)]
    pub false_discovery_rate_percent: f64,
    #[serde(default)]
    pub provider_outage_impact_percent: f64,
    #[serde(default)]
    pub provider_outage_status: String,
    #[serde(default)]
    pub blocking_requires_deterministic_confirmation: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestSummary {
    #[serde(default)]
    pub total_findings: u64,
    #[serde(default)]
    pub replay_candidates: u64,
    #[serde(default)]
    pub classification_counts: ReplayPromotionIngestClassificationCounts,
    #[serde(default)]
    pub confirmed_regression_count: u64,
    #[serde(default)]
    pub novel_confirmed_regression_count: u64,
    #[serde(default)]
    pub false_discovery_rate_percent: f64,
    #[serde(default)]
    pub provider_outage_impact_percent: f64,
    #[serde(default)]
    pub blocking_required: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestClassificationCounts {
    #[serde(default)]
    pub confirmed_reproducible: u64,
    #[serde(default)]
    pub not_reproducible: u64,
    #[serde(default)]
    pub needs_manual_review: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestLineageRow {
    #[serde(default)]
    pub finding_id: String,
    #[serde(default)]
    pub candidate_id: String,
    #[serde(default)]
    pub scenario_id: String,
    #[serde(default)]
    pub classification: String,
    #[serde(default)]
    pub source_lane: String,
    #[serde(default)]
    pub deterministic_replay_lane: String,
    #[serde(default)]
    pub release_blocking_authority: bool,
    #[serde(default)]
    pub generated_candidate: ReplayPromotionIngestGeneratedCandidate,
    #[serde(default)]
    pub candidate: ReplayPromotionIngestCandidate,
    #[serde(default)]
    pub deterministic_confirmation: ReplayPromotionIngestDeterministicConfirmation,
    #[serde(default)]
    pub promotion: ReplayPromotionIngestDecision,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestGeneratedCandidate {
    #[serde(default)]
    pub generation_kind: String,
    #[serde(default)]
    pub mutation_class: String,
    #[serde(default)]
    pub behavioral_class: String,
    #[serde(default)]
    pub novelty_score: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestCandidate {
    #[serde(default)]
    pub scenario_family: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub expected_outcome: String,
    #[serde(default)]
    pub observed_outcome: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub risk: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestDeterministicConfirmation {
    #[serde(default)]
    pub replay_status: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestDecision {
    #[serde(default)]
    pub owner_review_required: bool,
    #[serde(default)]
    pub owner_disposition: String,
    #[serde(default)]
    pub owner_disposition_due_at_unix: u64,
    #[serde(default)]
    pub blocking_regression: bool,
    #[serde(default)]
    pub promoted_scenario: ReplayPromotionIngestPromotedScenario,
    #[serde(default)]
    pub review_notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct ReplayPromotionIngestPromotedScenario {
    #[serde(default)]
    pub id: String,
}

pub(crate) fn load_replay_promotion_payload<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<ReplayPromotionPayload> {
    store
        .get(replay_promotion_state_key(site_id).as_str())
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<ReplayPromotionPayload>(bytes.as_slice()).ok())
        .filter(|payload| payload.schema_version == REPLAY_PROMOTION_SCHEMA_VERSION)
}

pub(crate) fn load_replay_promotion_summary<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> (ReplayPromotionSummary, u64) {
    match load_replay_promotion_payload(store, site_id) {
        Some(payload) => {
            let refreshed_at_ts = payload.generated_at_unix;
            (replay_promotion_summary(&payload), refreshed_at_ts)
        }
        None => (ReplayPromotionSummary::not_materialized(), 0),
    }
}

pub(crate) fn persist_replay_promotion_payload<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    request: ReplayPromotionIngestPayload,
) -> Result<ReplayPromotionPayload, ReplayPromotionPersistError> {
    if request.schema_version != REPLAY_PROMOTION_INGEST_SCHEMA_VERSION {
        return Err(ReplayPromotionPersistError::InvalidPayload(
            "Invalid replay-promotion payload: unsupported schema_version",
        ));
    }
    if request.generated_at_unix == 0 {
        return Err(ReplayPromotionPersistError::InvalidPayload(
            "Invalid replay-promotion payload: generated_at_unix must be > 0",
        ));
    }

    let summary = build_outcome_summary(&request);
    let payload = ReplayPromotionPayload {
        schema_version: REPLAY_PROMOTION_SCHEMA_VERSION.to_string(),
        generated_at_unix: request.generated_at_unix,
        source: ReplayPromotionSource {
            report_path: truncate_text(request.source.report_path.as_str()),
            attack_plan_path: truncate_text(request.source.attack_plan_path.as_str()),
            manifest_path: truncate_text(request.source.manifest_path.as_str()),
            frontier_status_path: truncate_text(request.source.frontier_status_path.as_str()),
        },
        frontier: ReplayPromotionFrontier {
            frontier_mode: truncate_text(request.frontier.frontier_mode.as_str()),
            provider_count: request.frontier.provider_count,
            diversity_confidence: truncate_text(request.frontier.diversity_confidence.as_str()),
        },
        lane_metadata: ReplayPromotionLaneMetadata {
            contract_path: truncate_text(request.lane_metadata.contract_path.as_str()),
            deterministic_conformance_lane: truncate_lane_descriptor(
                request.lane_metadata.deterministic_conformance_lane,
            ),
            emergent_exploration_lane: truncate_lane_descriptor(
                request.lane_metadata.emergent_exploration_lane,
            ),
        },
        promotion_pipeline: ReplayPromotionPipeline {
            steps: request
                .promotion_pipeline
                .steps
                .into_iter()
                .take(REPLAY_PROMOTION_MAX_FAILURE_ROWS)
                .map(|step| truncate_text(step.as_str()))
                .collect(),
            blocking_requires_deterministic_confirmation: request
                .promotion_pipeline
                .blocking_requires_deterministic_confirmation,
        },
        policy: request.policy,
        hybrid_governance: ReplayPromotionHybridGovernance {
            thresholds_passed: request.hybrid_governance.thresholds_passed,
            failures: request
                .hybrid_governance
                .failures
                .into_iter()
                .take(REPLAY_PROMOTION_MAX_FAILURE_ROWS)
                .map(|value| truncate_text(value.as_str()))
                .collect(),
            deterministic_confirmation_rate_percent: request
                .hybrid_governance
                .observed
                .deterministic_confirmation_rate_percent,
            false_discovery_rate_percent: request
                .hybrid_governance
                .observed
                .false_discovery_rate_percent,
            overdue_owner_review_count: request
                .hybrid_governance
                .observed
                .overdue_owner_review_count,
        },
        discovery_quality_metrics: ReplayPromotionDiscoveryQualityMetrics {
            candidate_count: request.discovery_quality_metrics.candidate_count,
            generated_candidate_count: request.discovery_quality_metrics.generated_candidate_count,
            novel_confirmed_regressions: request
                .discovery_quality_metrics
                .novel_confirmed_regressions,
            false_discovery_rate_percent: request
                .discovery_quality_metrics
                .false_discovery_rate_percent,
            provider_outage_impact_percent: request
                .discovery_quality_metrics
                .provider_outage_impact_percent,
            provider_outage_status: truncate_text(
                request.discovery_quality_metrics.provider_outage_status.as_str(),
            ),
            blocking_requires_deterministic_confirmation: request
                .discovery_quality_metrics
                .blocking_requires_deterministic_confirmation,
        },
        summary,
        lineage: request
            .lineage
            .into_iter()
            .take(REPLAY_PROMOTION_MAX_LINEAGE_ROWS)
            .map(truncate_lineage_row)
            .collect(),
    };

    let serialized = serde_json::to_vec(&payload).map_err(|_| {
        ReplayPromotionPersistError::PersistFailed("Failed to serialize replay-promotion payload")
    })?;
    store
        .set(replay_promotion_state_key(site_id).as_str(), serialized.as_slice())
        .map_err(|_| {
            ReplayPromotionPersistError::PersistFailed("Failed to persist replay-promotion payload")
        })?;
    Ok(payload)
}

pub(crate) fn replay_promotion_summary(payload: &ReplayPromotionPayload) -> ReplayPromotionSummary {
    ReplayPromotionSummary {
        availability: "materialized".to_string(),
        generated_at_unix: Some(payload.generated_at_unix),
        frontier_mode: payload.frontier.frontier_mode.clone(),
        provider_count: payload.frontier.provider_count,
        diversity_confidence: payload.frontier.diversity_confidence.clone(),
        replay_candidates: payload.summary.replay_candidates,
        confirmed_reproducible_count: payload.summary.confirmed_reproducible_count,
        not_reproducible_count: payload.summary.not_reproducible_count,
        needs_manual_review_count: payload.summary.needs_manual_review_count,
        pending_owner_review_count: payload.summary.pending_owner_review_count,
        promoted_scenario_count: payload.summary.promoted_scenario_count,
        blocking_required: payload.summary.blocking_required,
        thresholds_passed: Some(payload.hybrid_governance.thresholds_passed),
        threshold_failures: payload.hybrid_governance.failures.clone(),
        lineage: payload
            .lineage
            .iter()
            .take(REPLAY_PROMOTION_SUMMARY_MAX_LINEAGE_ROWS)
            .cloned()
            .collect(),
    }
}

fn build_outcome_summary(request: &ReplayPromotionIngestPayload) -> ReplayPromotionOutcomeSummary {
    let pending_owner_review_count = request
        .lineage
        .iter()
        .filter(|row| {
            row.promotion.owner_review_required
                && !matches!(
                    row.promotion.owner_disposition.as_str(),
                    "accepted" | "rejected" | "dismissed" | "not_required"
                )
        })
        .count() as u64;
    let promoted_scenario_count = request
        .lineage
        .iter()
        .filter(|row| !row.promotion.promoted_scenario.id.trim().is_empty())
        .count() as u64;
    ReplayPromotionOutcomeSummary {
        total_findings: request.summary.total_findings,
        replay_candidates: request.summary.replay_candidates,
        confirmed_reproducible_count: request
            .summary
            .classification_counts
            .confirmed_reproducible
            .max(request.summary.confirmed_regression_count),
        not_reproducible_count: request.summary.classification_counts.not_reproducible,
        needs_manual_review_count: request.summary.classification_counts.needs_manual_review,
        pending_owner_review_count,
        promoted_scenario_count,
        blocking_required: request.summary.blocking_required,
        false_discovery_rate_percent: request.summary.false_discovery_rate_percent,
        provider_outage_impact_percent: request.summary.provider_outage_impact_percent,
        novel_confirmed_regression_count: request.summary.novel_confirmed_regression_count,
    }
}

fn truncate_lane_descriptor(descriptor: ReplayPromotionLaneDescriptor) -> ReplayPromotionLaneDescriptor {
    ReplayPromotionLaneDescriptor {
        lane_id: truncate_text(descriptor.lane_id.as_str()),
        release_blocking: descriptor.release_blocking,
        authority: truncate_text(descriptor.authority.as_str()),
    }
}

fn truncate_lineage_row(row: ReplayPromotionIngestLineageRow) -> ReplayPromotionLineageRow {
    ReplayPromotionLineageRow {
        finding_id: truncate_text(row.finding_id.as_str()),
        candidate_id: truncate_text(row.candidate_id.as_str()),
        scenario_id: truncate_text(row.scenario_id.as_str()),
        source_lane: truncate_text(row.source_lane.as_str()),
        deterministic_replay_lane: truncate_text(row.deterministic_replay_lane.as_str()),
        classification: truncate_text(row.classification.as_str()),
        replay_status: truncate_text(row.deterministic_confirmation.replay_status.as_str()),
        release_blocking_authority: row.release_blocking_authority,
        scenario_family: truncate_text(row.candidate.scenario_family.as_str()),
        path: truncate_text(row.candidate.path.as_str()),
        expected_outcome: truncate_text(row.candidate.expected_outcome.as_str()),
        observed_outcome: truncate_text(row.candidate.observed_outcome.as_str()),
        severity: truncate_text(row.candidate.severity.as_str()),
        risk: truncate_text(row.candidate.risk.as_str()),
        generation_kind: truncate_text(row.generated_candidate.generation_kind.as_str()),
        mutation_class: truncate_text(row.generated_candidate.mutation_class.as_str()),
        behavioral_class: truncate_text(row.generated_candidate.behavioral_class.as_str()),
        novelty_score: row.generated_candidate.novelty_score,
        owner_review_required: row.promotion.owner_review_required,
        owner_disposition: truncate_text(row.promotion.owner_disposition.as_str()),
        owner_disposition_due_at_unix: row.promotion.owner_disposition_due_at_unix,
        blocking_regression: row.promotion.blocking_regression,
        promoted_scenario_id: if row.promotion.promoted_scenario.id.trim().is_empty() {
            None
        } else {
            Some(truncate_text(row.promotion.promoted_scenario.id.as_str()))
        },
        review_notes: row
            .promotion
            .review_notes
            .into_iter()
            .take(REPLAY_PROMOTION_MAX_REVIEW_NOTES)
            .map(|value| truncate_text(value.as_str()))
            .collect(),
    }
}

fn replay_promotion_state_key(site_id: &str) -> String {
    format!("{REPLAY_PROMOTION_PREFIX}:{site_id}")
}

fn truncate_text(value: &str) -> String {
    if value.chars().count() <= REPLAY_PROMOTION_MAX_TEXT_CHARS {
        return value.to_string();
    }
    value
        .chars()
        .take(REPLAY_PROMOTION_MAX_TEXT_CHARS.saturating_sub(3))
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::{
        load_replay_promotion_payload, load_replay_promotion_summary, persist_replay_promotion_payload,
        ReplayPromotionIngestPayload, ReplayPromotionPersistError,
    };
    use crate::challenge::KeyValueStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

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

    fn sample_ingest_payload() -> ReplayPromotionIngestPayload {
        serde_json::from_value(serde_json::json!({
            "schema_version": "adversarial-promotion.v1",
            "generated_at_unix": 1_700_000_200u64,
            "source": {
                "report_path": "scripts/tests/adversarial/latest_report.json",
                "attack_plan_path": "scripts/tests/adversarial/attack_plan.json",
                "manifest_path": "scripts/tests/adversarial/scenario_manifest.v2.json",
                "frontier_status_path": "scripts/tests/adversarial/frontier_lane_status.json"
            },
            "frontier": {
                "frontier_mode": "multi_provider_playoff",
                "provider_count": 2,
                "diversity_confidence": "higher"
            },
            "lane_metadata": {
                "contract_path": "scripts/tests/adversarial/hybrid_lane_contract.v1.json",
                "deterministic_conformance_lane": {
                    "lane_id": "deterministic_conformance",
                    "release_blocking": true,
                    "authority": "deterministic_replay_confirmation"
                },
                "emergent_exploration_lane": {
                    "lane_id": "emergent_exploration",
                    "release_blocking": false,
                    "authority": "discovery_only"
                }
            },
            "promotion_pipeline": {
                "steps": [
                    "generated_candidate",
                    "deterministic_replay_confirmation",
                    "owner_review_disposition",
                    "promoted_blocking_scenario"
                ],
                "blocking_requires_deterministic_confirmation": true
            },
            "policy": {
                "deterministic_oracle_authoritative": true,
                "single_provider_self_play_requires_owner_review": true,
                "multi_provider_playoff_requires_owner_review": true,
                "blocking_requires_deterministic_confirmation": true
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
                "candidate_count": 3,
                "generated_candidate_count": 2,
                "novel_confirmed_regressions": 1,
                "false_discovery_rate_percent": 0.0,
                "provider_outage_impact_percent": 0.0,
                "provider_outage_status": "healthy",
                "blocking_requires_deterministic_confirmation": true
            },
            "summary": {
                "total_findings": 3,
                "replay_candidates": 2,
                "classification_counts": {
                    "confirmed_reproducible": 1,
                    "not_reproducible": 1,
                    "needs_manual_review": 0
                },
                "confirmed_regression_count": 1,
                "novel_confirmed_regression_count": 1,
                "false_discovery_rate_percent": 0.0,
                "provider_outage_impact_percent": 0.0,
                "blocking_required": true
            },
            "lineage": [
                {
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
                            "multi_provider_playoff provides higher initial confidence, but owner review remains required."
                        ]
                    }
                },
                {
                    "finding_id": "simf-002",
                    "candidate_id": "cand-002",
                    "scenario_id": "sim_t4_b",
                    "classification": "not_reproducible",
                    "source_lane": "emergent_exploration",
                    "deterministic_replay_lane": "deterministic_conformance",
                    "release_blocking_authority": false,
                    "generated_candidate": {
                        "generation_kind": "seed",
                        "mutation_class": "seed",
                        "behavioral_class": "baseline",
                        "novelty_score": 0.0
                    },
                    "candidate": {
                        "scenario_family": "cdp_high_confidence_deny",
                        "path": "/",
                        "expected_outcome": "deny_temp",
                        "observed_outcome": "allow",
                        "severity": "medium",
                        "risk": "medium"
                    },
                    "deterministic_confirmation": {
                        "replay_status": "ok"
                    },
                    "promotion": {
                        "owner_review_required": false,
                        "owner_disposition": "not_required",
                        "owner_disposition_due_at_unix": 0,
                        "blocking_regression": false,
                        "promoted_scenario": {},
                        "review_notes": [
                            "deterministic replay did not confirm promotable candidate."
                        ]
                    }
                }
            ]
        }))
        .expect("sample payload")
    }

    #[test]
    fn replay_promotion_summary_reports_not_materialized_without_state() {
        let store = TestStore::default();

        let (summary, refreshed_at_ts) = load_replay_promotion_summary(&store, "default");

        assert_eq!(summary.availability, "not_materialized");
        assert!(summary.generated_at_unix.is_none());
        assert_eq!(refreshed_at_ts, 0);
    }

    #[test]
    fn persist_replay_promotion_payload_materializes_bounded_contract() {
        let store = TestStore::default();

        let payload = persist_replay_promotion_payload(&store, "default", sample_ingest_payload())
            .expect("payload persists");

        assert_eq!(payload.schema_version, "replay_promotion_v1");
        assert_eq!(payload.summary.confirmed_reproducible_count, 1);
        assert_eq!(payload.summary.not_reproducible_count, 1);
        assert_eq!(payload.summary.pending_owner_review_count, 1);
        assert_eq!(payload.summary.promoted_scenario_count, 1);
        assert_eq!(payload.lineage.len(), 2);
        assert_eq!(payload.lineage[0].classification, "confirmed_reproducible");

        let stored = load_replay_promotion_payload(&store, "default").expect("stored payload");
        assert_eq!(stored.generated_at_unix, 1_700_000_200);

        let (summary, refreshed_at_ts) = load_replay_promotion_summary(&store, "default");
        assert_eq!(summary.availability, "materialized");
        assert_eq!(summary.generated_at_unix, Some(1_700_000_200));
        assert_eq!(summary.replay_candidates, 2);
        assert_eq!(summary.pending_owner_review_count, 1);
        assert_eq!(summary.promoted_scenario_count, 1);
        assert_eq!(summary.lineage.len(), 2);
        assert_eq!(refreshed_at_ts, 1_700_000_200);
    }

    #[test]
    fn persist_replay_promotion_payload_rejects_unknown_schema() {
        let store = TestStore::default();
        let mut payload = sample_ingest_payload();
        payload.schema_version = "wrong".to_string();

        let err = persist_replay_promotion_payload(&store, "default", payload)
            .expect_err("schema must be rejected");

        assert_eq!(
            err,
            ReplayPromotionPersistError::InvalidPayload(
                "Invalid replay-promotion payload: unsupported schema_version"
            )
        );
    }
}
