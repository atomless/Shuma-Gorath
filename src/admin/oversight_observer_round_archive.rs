use serde::{Deserialize, Serialize};

use crate::observability::operator_snapshot::{
    OperatorSnapshotEpisodeRecord, OperatorSnapshotRecentSimRun,
};

pub(crate) const OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION: &str =
    "oversight_observer_round_archive_v1";

const OVERSIGHT_OBSERVER_ROUND_ARCHIVE_PREFIX: &str = "oversight_observer_round_archive:v1";
const OVERSIGHT_OBSERVER_ROUND_ARCHIVE_MAX_ROWS: usize = 24;
const BASIS_STATUS_EXACT: &str = "exact_judged_run_receipts";
const BASIS_STATUS_PARTIAL: &str = "partial_missing_run_receipts";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightObserverRoundRunRow {
    pub run_id: String,
    pub lane: String,
    pub profile: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_fulfillment_modes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_category_ids: Vec<String>,
    pub monitoring_event_count: u64,
    pub defense_delta_count: u64,
    pub ban_outcome_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightObserverRoundSurfaceRow {
    pub run_id: String,
    pub surface_id: String,
    pub surface_state: String,
    pub coverage_status: String,
    pub success_contract: String,
    pub dependency_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_surface_ids: Vec<String>,
    pub attempt_count: u64,
    pub sample_request_method: String,
    pub sample_request_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_response_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightObserverRoundRecord {
    pub episode_id: String,
    pub completed_at_ts: u64,
    pub basis_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_run_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_rows: Vec<OversightObserverRoundRunRow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scrapling_surface_rows: Vec<OversightObserverRoundSurfaceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightObserverRoundArchive {
    pub schema_version: String,
    pub rows: Vec<OversightObserverRoundRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OversightObserverRoundArchiveState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OversightObserverRoundRecord>,
}

fn observer_round_archive_key(site_id: &str) -> String {
    format!("{OVERSIGHT_OBSERVER_ROUND_ARCHIVE_PREFIX}:{site_id}")
}

fn load_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OversightObserverRoundArchiveState {
    store
        .get(&observer_round_archive_key(site_id))
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<OversightObserverRoundArchiveState>(&bytes).ok())
        .filter(|state| state.schema_version == OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION)
        .unwrap_or_else(|| OversightObserverRoundArchiveState {
            schema_version: OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION.to_string(),
            updated_at_ts: 0,
            rows: Vec::new(),
        })
}

fn save_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OversightObserverRoundArchiveState,
) -> Result<(), ()> {
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(&observer_round_archive_key(site_id), payload.as_slice())
}

pub(crate) fn load_oversight_observer_round_archive<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OversightObserverRoundArchive {
    let state = load_state(store, site_id);
    OversightObserverRoundArchive {
        schema_version: OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION.to_string(),
        rows: state.rows,
    }
}

pub(crate) fn record_oversight_observer_round<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    record: OversightObserverRoundRecord,
) -> Result<(), ()> {
    let mut state = load_state(store, site_id);
    state.rows.retain(|existing| existing.episode_id != record.episode_id);
    state.rows.push(record.clone());
    state.rows.sort_by(|left, right| {
        right
            .completed_at_ts
            .cmp(&left.completed_at_ts)
            .then_with(|| left.episode_id.cmp(&right.episode_id))
    });
    state.rows.truncate(OVERSIGHT_OBSERVER_ROUND_ARCHIVE_MAX_ROWS);
    state.updated_at_ts = record.completed_at_ts;
    state.schema_version = OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION.to_string();
    save_state(store, site_id, &state)
}

pub(crate) fn build_observer_round_record(
    episode_record: &OperatorSnapshotEpisodeRecord,
    recent_runs: &[OperatorSnapshotRecentSimRun],
) -> OversightObserverRoundRecord {
    let mut run_rows = Vec::new();
    let mut scrapling_surface_rows = Vec::new();
    let mut missing_run_ids = Vec::new();

    for run_id in dedupe_strings(episode_record.judged_run_ids.as_slice()) {
        let Some(run) = recent_runs.iter().find(|run| run.run_id == run_id) else {
            missing_run_ids.push(run_id);
            continue;
        };

        run_rows.push(OversightObserverRoundRunRow {
            run_id: run.run_id.clone(),
            lane: run.lane.clone(),
            profile: run.profile.clone(),
            observed_fulfillment_modes: run.observed_fulfillment_modes.clone(),
            observed_category_ids: run.observed_category_ids.clone(),
            monitoring_event_count: run.monitoring_event_count,
            defense_delta_count: run.defense_delta_count,
            ban_outcome_count: run.ban_outcome_count,
        });

        if run.lane == "scrapling_traffic" {
            if let Some(coverage) = run.owned_surface_coverage.as_ref() {
                scrapling_surface_rows.extend(coverage.receipts.iter().map(|receipt| {
                    OversightObserverRoundSurfaceRow {
                        run_id: run.run_id.clone(),
                        surface_id: receipt.surface_id.clone(),
                        surface_state: receipt.surface_state.clone(),
                        coverage_status: receipt.coverage_status.clone(),
                        success_contract: receipt.success_contract.clone(),
                        dependency_kind: receipt.dependency_kind.clone(),
                        dependency_surface_ids: receipt.dependency_surface_ids.clone(),
                        attempt_count: receipt.attempt_count,
                        sample_request_method: receipt.sample_request_method.clone(),
                        sample_request_path: receipt.sample_request_path.clone(),
                        sample_response_status: receipt.sample_response_status,
                    }
                }));
            }
        }
    }

    OversightObserverRoundRecord {
        episode_id: episode_record.episode_id.clone(),
        completed_at_ts: episode_record.completed_at_ts,
        basis_status: if missing_run_ids.is_empty() {
            BASIS_STATUS_EXACT.to_string()
        } else {
            BASIS_STATUS_PARTIAL.to_string()
        },
        missing_run_ids,
        run_rows,
        scrapling_surface_rows,
    }
}

fn dedupe_strings(values: &[String]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        let normalized = value.trim();
        if normalized.is_empty() || !seen.insert(normalized.to_string()) {
            continue;
        }
        rows.push(normalized.to_string());
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{
        build_observer_round_record, OversightObserverRoundArchive,
        OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION,
    };
    use crate::observability::benchmark_comparison::BenchmarkComparableSnapshot;
    use crate::observability::operator_snapshot::{
        BenchmarkHomeostasisRestartBaseline, OperatorSnapshotEpisodeEvaluationContext,
        OperatorSnapshotEpisodeRecord, OperatorSnapshotWindow,
    };
    use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;
    use crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageSummary;

    fn episode_record() -> OperatorSnapshotEpisodeRecord {
        OperatorSnapshotEpisodeRecord {
            episode_id: "episode-1".to_string(),
            proposal_id: None,
            completed_at_ts: 1_774_786_800,
            trigger_source: "periodic_supervisor".to_string(),
            evaluation_context: OperatorSnapshotEpisodeEvaluationContext {
                objective_revision: "rev-1".to_string(),
                profile_id: "human_only_private".to_string(),
                subject_kind: "site".to_string(),
                comparison_mode: "max_ratio_budget".to_string(),
            },
            baseline_scorecard: BenchmarkComparableSnapshot {
                generated_at: 1_774_786_700,
                subject_kind: "site".to_string(),
                watch_window: OperatorSnapshotWindow {
                    start_ts: 1_774_783_100,
                    end_ts: 1_774_786_700,
                    duration_seconds: 3_600,
                },
                coverage_status: "materialized".to_string(),
                overall_status: "outside_budget".to_string(),
                families: Vec::new(),
            },
            proposal: None,
            proposal_status: "accepted".to_string(),
            watch_window_result: "improved".to_string(),
            retain_or_rollback: "retained".to_string(),
            judged_lane_ids: vec!["scrapling_traffic".to_string(), "bot_red_team".to_string()],
            judged_run_ids: vec!["simrun-1".to_string(), "simrun-2".to_string()],
            benchmark_deltas: Vec::new(),
            hard_guardrail_triggers: Vec::new(),
            cycle_judgment: "continue".to_string(),
            homeostasis_eligible: true,
            benchmark_urgency_status: "steady".to_string(),
            homeostasis_break_status: "not_triggered".to_string(),
            homeostasis_break_reasons: Vec::new(),
            restart_baseline: BenchmarkHomeostasisRestartBaseline::default(),
            evidence_references: Vec::new(),
        }
    }

    fn scrapling_run() -> OperatorSnapshotRecentSimRun {
        OperatorSnapshotRecentSimRun {
            run_id: "simrun-1".to_string(),
            lane: "scrapling_traffic".to_string(),
            profile: "crawler".to_string(),
            observed_fulfillment_modes: vec!["crawler".to_string()],
            observed_category_ids: vec!["indexing_bot".to_string()],
            first_ts: 1,
            last_ts: 2,
            monitoring_event_count: 3,
            defense_delta_count: 2,
            ban_outcome_count: 1,
            owned_surface_coverage: Some(ScraplingOwnedSurfaceCoverageSummary {
                overall_status: "partial".to_string(),
                canonical_surface_ids: vec!["challenge_routing".to_string()],
                surface_labels: std::collections::BTreeMap::new(),
                required_surface_ids: vec!["challenge_routing".to_string()],
                satisfied_surface_ids: Vec::new(),
                blocking_surface_ids: vec!["challenge_routing".to_string()],
                receipts: vec![
                    crate::observability::scrapling_owned_surface::ScraplingOwnedSurfaceCoverageReceipt {
                        surface_id: "challenge_routing".to_string(),
                        success_contract: "required_then_enforced".to_string(),
                        dependency_kind: "independent".to_string(),
                        dependency_surface_ids: Vec::new(),
                        coverage_status: "required".to_string(),
                        surface_state: "required_but_unreached".to_string(),
                        satisfied: false,
                        blocked_by_surface_ids: Vec::new(),
                        attempt_count: 2,
                        sample_request_method: "GET".to_string(),
                        sample_request_path: "/catalog".to_string(),
                        sample_response_status: Some(403),
                    },
                ],
            }),
            llm_runtime_summary: None,
        }
    }

    fn llm_run() -> OperatorSnapshotRecentSimRun {
        OperatorSnapshotRecentSimRun {
            run_id: "simrun-2".to_string(),
            lane: "bot_red_team".to_string(),
            profile: "browser_mode".to_string(),
            observed_fulfillment_modes: vec!["browser_mode".to_string()],
            observed_category_ids: vec!["browser_agent".to_string()],
            first_ts: 3,
            last_ts: 4,
            monitoring_event_count: 5,
            defense_delta_count: 1,
            ban_outcome_count: 0,
            owned_surface_coverage: None,
            llm_runtime_summary: None,
        }
    }

    #[test]
    fn observer_round_record_is_exact_when_all_judged_runs_are_present() {
        let record = build_observer_round_record(&episode_record(), &[scrapling_run(), llm_run()]);
        assert_eq!(record.basis_status, "exact_judged_run_receipts");
        assert!(record.missing_run_ids.is_empty());
        assert_eq!(record.run_rows.len(), 2);
        assert_eq!(record.scrapling_surface_rows.len(), 1);
        assert_eq!(record.scrapling_surface_rows[0].surface_id, "challenge_routing");
    }

    #[test]
    fn observer_round_record_marks_missing_judged_runs_without_guessing() {
        let record = build_observer_round_record(&episode_record(), &[llm_run()]);
        assert_eq!(record.basis_status, "partial_missing_run_receipts");
        assert_eq!(record.missing_run_ids, vec!["simrun-1".to_string()]);
        assert_eq!(record.run_rows.len(), 1);
        assert!(record.scrapling_surface_rows.is_empty());
    }

    #[test]
    fn observer_round_archive_contract_uses_stable_schema_version() {
        let archive = OversightObserverRoundArchive {
            schema_version: OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION.to_string(),
            rows: Vec::new(),
        };
        assert_eq!(
            archive.schema_version,
            OVERSIGHT_OBSERVER_ROUND_ARCHIVE_SCHEMA_VERSION
        );
    }
}
