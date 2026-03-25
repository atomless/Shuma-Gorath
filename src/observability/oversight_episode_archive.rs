use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

use super::benchmark_comparison::{
    BenchmarkComparableFamilyDelta, BenchmarkComparableSnapshot,
};

const OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION: &str = "oversight_episode_archive_v1";
const OVERSIGHT_EPISODE_ARCHIVE_PREFIX: &str = "oversight_episode_archive:v1";
const OVERSIGHT_EPISODE_ARCHIVE_MAX_ROWS: usize = 24;
pub(crate) const OVERSIGHT_EPISODE_HOMEOSTASIS_WINDOW: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightEpisodeEvidenceReference {
    pub kind: String,
    pub reference: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightEpisodeProposedMove {
    pub proposal_id: String,
    pub patch_family: String,
    pub patch: serde_json::Value,
    pub expected_impact_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightEpisodeArchiveRow {
    pub episode_id: String,
    pub latest_decision_id: String,
    pub recorded_at_ts: u64,
    pub trigger_source: String,
    pub objective_profile_id: String,
    pub objective_revision: String,
    pub evaluation_context: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_sim_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposed_move: Option<OversightEpisodeProposedMove>,
    pub acceptance_status: String,
    pub watch_window_status: String,
    pub retention_status: String,
    pub completion_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_scorecard: Option<BenchmarkComparableSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_scorecard: Option<BenchmarkComparableSnapshot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmark_deltas: Vec<BenchmarkComparableFamilyDelta>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hard_guardrail_triggers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<OversightEpisodeEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightEpisodeArchiveSummary {
    pub schema_version: String,
    pub homeostasis_window_size: usize,
    pub rows: Vec<OversightEpisodeArchiveRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OversightEpisodeArchiveDraft {
    pub episode_id: String,
    pub latest_decision_id: String,
    pub recorded_at_ts: u64,
    pub trigger_source: String,
    pub objective_profile_id: String,
    pub objective_revision: String,
    pub evaluation_context: String,
    pub latest_sim_run_id: Option<String>,
    pub proposed_move: Option<OversightEpisodeProposedMove>,
    pub acceptance_status: String,
    pub watch_window_status: String,
    pub retention_status: String,
    pub completion_status: String,
    pub baseline_scorecard: Option<BenchmarkComparableSnapshot>,
    pub candidate_scorecard: Option<BenchmarkComparableSnapshot>,
    pub benchmark_deltas: Vec<BenchmarkComparableFamilyDelta>,
    pub hard_guardrail_triggers: Vec<String>,
    pub evidence_references: Vec<OversightEpisodeEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OversightEpisodeArchiveState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OversightEpisodeArchiveRow>,
}

fn archive_key(site_id: &str) -> String {
    format!("{OVERSIGHT_EPISODE_ARCHIVE_PREFIX}:{site_id}")
}

fn load_state<S: KeyValueStore>(store: &S, site_id: &str) -> OversightEpisodeArchiveState {
    store
        .get(archive_key(site_id).as_str())
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<OversightEpisodeArchiveState>(&bytes).ok())
        .filter(|state| state.schema_version == OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION)
        .unwrap_or_else(|| OversightEpisodeArchiveState {
            schema_version: OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION.to_string(),
            updated_at_ts: 0,
            rows: Vec::new(),
        })
}

fn save_state<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OversightEpisodeArchiveState,
) -> Result<(), ()> {
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(archive_key(site_id).as_str(), payload.as_slice())
}

pub(crate) fn upsert_episode_archive_row<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    draft: OversightEpisodeArchiveDraft,
) -> Result<OversightEpisodeArchiveRow, ()> {
    let mut state = load_state(store, site_id);
    let updated_row = if let Some(existing) = state
        .rows
        .iter_mut()
        .find(|row| row.episode_id == draft.episode_id)
    {
        existing.latest_decision_id = draft.latest_decision_id.clone();
        existing.recorded_at_ts = draft.recorded_at_ts;
        existing.trigger_source = draft.trigger_source.clone();
        existing.objective_profile_id = draft.objective_profile_id.clone();
        existing.objective_revision = draft.objective_revision.clone();
        existing.evaluation_context = draft.evaluation_context.clone();
        existing.latest_sim_run_id = draft
            .latest_sim_run_id
            .clone()
            .or(existing.latest_sim_run_id.clone());
        if draft.proposed_move.is_some() {
            existing.proposed_move = draft.proposed_move.clone();
        }
        existing.acceptance_status = draft.acceptance_status.clone();
        existing.watch_window_status = draft.watch_window_status.clone();
        existing.retention_status = draft.retention_status.clone();
        existing.completion_status = draft.completion_status.clone();
        if draft.baseline_scorecard.is_some() {
            existing.baseline_scorecard = draft.baseline_scorecard.clone();
        }
        if draft.candidate_scorecard.is_some() {
            existing.candidate_scorecard = draft.candidate_scorecard.clone();
        }
        if !draft.benchmark_deltas.is_empty() {
            existing.benchmark_deltas = draft.benchmark_deltas.clone();
        }
        existing.hard_guardrail_triggers =
            compact_strings(existing.hard_guardrail_triggers.iter().cloned().chain(
                draft.hard_guardrail_triggers.clone().into_iter(),
            ));
        existing.evidence_references =
            compact_evidence(existing.evidence_references.clone(), &draft.evidence_references);
        existing.clone()
    } else {
        let row = OversightEpisodeArchiveRow {
            episode_id: draft.episode_id,
            latest_decision_id: draft.latest_decision_id,
            recorded_at_ts: draft.recorded_at_ts,
            trigger_source: draft.trigger_source,
            objective_profile_id: draft.objective_profile_id,
            objective_revision: draft.objective_revision,
            evaluation_context: draft.evaluation_context,
            latest_sim_run_id: draft.latest_sim_run_id,
            proposed_move: draft.proposed_move,
            acceptance_status: draft.acceptance_status,
            watch_window_status: draft.watch_window_status,
            retention_status: draft.retention_status,
            completion_status: draft.completion_status,
            baseline_scorecard: draft.baseline_scorecard,
            candidate_scorecard: draft.candidate_scorecard,
            benchmark_deltas: draft.benchmark_deltas,
            hard_guardrail_triggers: compact_strings(draft.hard_guardrail_triggers.into_iter()),
            evidence_references: compact_evidence(Vec::new(), &draft.evidence_references),
        };
        state.rows.push(row.clone());
        row
    };

    state.rows.sort_by(|left, right| {
        right
            .recorded_at_ts
            .cmp(&left.recorded_at_ts)
            .then_with(|| left.episode_id.cmp(&right.episode_id))
    });
    state.rows.truncate(OVERSIGHT_EPISODE_ARCHIVE_MAX_ROWS);
    state.updated_at_ts = updated_row.recorded_at_ts;
    state.schema_version = OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION.to_string();
    save_state(store, site_id, &state)?;
    Ok(updated_row)
}

pub(crate) fn load_episode_archive_summary<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OversightEpisodeArchiveSummary {
    let state = load_state(store, site_id);
    OversightEpisodeArchiveSummary {
        schema_version: OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION.to_string(),
        homeostasis_window_size: OVERSIGHT_EPISODE_HOMEOSTASIS_WINDOW,
        rows: state.rows,
    }
}

fn compact_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn compact_evidence(
    existing: Vec<OversightEpisodeEvidenceReference>,
    incoming: &[OversightEpisodeEvidenceReference],
) -> Vec<OversightEpisodeEvidenceReference> {
    let mut rows = existing;
    for reference in incoming {
        if rows.iter().any(|existing| existing == reference) {
            continue;
        }
        rows.push(reference.clone());
    }
    rows.sort_by(|left, right| {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.reference.cmp(&right.reference))
            .then_with(|| left.note.cmp(&right.note))
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::{
        load_episode_archive_summary, upsert_episode_archive_row,
        OversightEpisodeArchiveDraft, OversightEpisodeEvidenceReference,
        OversightEpisodeProposedMove, OVERSIGHT_EPISODE_HOMEOSTASIS_WINDOW,
    };
    use crate::challenge::KeyValueStore;
    use crate::observability::benchmark_comparison::{
        BenchmarkComparableFamily, BenchmarkComparableFamilyDelta, BenchmarkComparableMetric,
        BenchmarkComparableMetricDelta, BenchmarkComparableSnapshot,
    };
    use crate::observability::operator_snapshot::OperatorSnapshotWindow;
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

    fn snapshot(generated_at: u64, current: f64, status: &str) -> BenchmarkComparableSnapshot {
        BenchmarkComparableSnapshot {
            generated_at,
            subject_kind: "current_instance".to_string(),
            watch_window: OperatorSnapshotWindow {
                start_ts: generated_at.saturating_sub(99),
                end_ts: generated_at,
                duration_seconds: 100,
            },
            coverage_status: "supported".to_string(),
            overall_status: status.to_string(),
            families: vec![BenchmarkComparableFamily {
                family_id: "likely_human_friction".to_string(),
                status: status.to_string(),
                capability_gate: "supported".to_string(),
                metrics: vec![BenchmarkComparableMetric {
                    metric_id: "likely_human_friction_rate".to_string(),
                    status: status.to_string(),
                    current: Some(current),
                    capability_gate: "supported".to_string(),
                }],
            }],
        }
    }

    fn open_draft() -> OversightEpisodeArchiveDraft {
        OversightEpisodeArchiveDraft {
            episode_id: "episode-001".to_string(),
            latest_decision_id: "decision-001".to_string(),
            recorded_at_ts: 1_700_000_100,
            trigger_source: "periodic_supervisor".to_string(),
            objective_profile_id: "site_default_v1".to_string(),
            objective_revision: "rev-1700000000".to_string(),
            evaluation_context: "objective_profile:site_default_v1".to_string(),
            latest_sim_run_id: Some("simrun-001".to_string()),
            proposed_move: Some(OversightEpisodeProposedMove {
                proposal_id: "proposal-001".to_string(),
                patch_family: "fingerprint_signal".to_string(),
                patch: serde_json::json!({"fingerprint_signal_enabled": false}),
                expected_impact_summary: "Reduce likely-human friction".to_string(),
            }),
            acceptance_status: "accepted_canary".to_string(),
            watch_window_status: "open".to_string(),
            retention_status: "pending".to_string(),
            completion_status: "open".to_string(),
            baseline_scorecard: Some(snapshot(1_700_000_100, 0.20, "outside_budget")),
            candidate_scorecard: None,
            benchmark_deltas: Vec::new(),
            hard_guardrail_triggers: vec!["verified_identity_no_harm".to_string()],
            evidence_references: vec![OversightEpisodeEvidenceReference {
                kind: "operator_snapshot".to_string(),
                reference: "generated_at:1700000100".to_string(),
                note: "Baseline snapshot.".to_string(),
            }],
        }
    }

    #[test]
    fn episode_archive_upsert_preserves_baseline_and_updates_completion() {
        let store = TestStore::new();
        upsert_episode_archive_row(&store, "default", open_draft()).expect("open row persists");

        upsert_episode_archive_row(
            &store,
            "default",
            OversightEpisodeArchiveDraft {
                episode_id: "episode-001".to_string(),
                latest_decision_id: "decision-002".to_string(),
                recorded_at_ts: 1_700_000_400,
                trigger_source: "periodic_supervisor".to_string(),
                objective_profile_id: "site_default_v1".to_string(),
                objective_revision: "rev-1700000000".to_string(),
                evaluation_context: "objective_profile:site_default_v1".to_string(),
                latest_sim_run_id: Some("simrun-001".to_string()),
                proposed_move: None,
                acceptance_status: "accepted_canary".to_string(),
                watch_window_status: "improved".to_string(),
                retention_status: "retained".to_string(),
                completion_status: "completed".to_string(),
                baseline_scorecard: None,
                candidate_scorecard: Some(snapshot(1_700_000_400, 0.02, "inside_budget")),
                benchmark_deltas: vec![BenchmarkComparableFamilyDelta {
                    family_id: "likely_human_friction".to_string(),
                    baseline_status: "outside_budget".to_string(),
                    candidate_status: "inside_budget".to_string(),
                    comparison_status: "improved".to_string(),
                    metrics: vec![BenchmarkComparableMetricDelta {
                        metric_id: "likely_human_friction_rate".to_string(),
                        baseline_current: Some(0.20),
                        candidate_current: Some(0.02),
                        comparison_status: "improved".to_string(),
                    }],
                }],
                hard_guardrail_triggers: Vec::new(),
                evidence_references: vec![OversightEpisodeEvidenceReference {
                    kind: "watch_window".to_string(),
                    reference: "generated_at:1700000400".to_string(),
                    note: "Candidate snapshot.".to_string(),
                }],
            },
        )
        .expect("completed row persists");

        let summary = load_episode_archive_summary(&store, "default");
        assert_eq!(summary.schema_version, "oversight_episode_archive_v1");
        assert_eq!(
            summary.homeostasis_window_size,
            OVERSIGHT_EPISODE_HOMEOSTASIS_WINDOW
        );
        assert_eq!(summary.rows.len(), 1);
        assert_eq!(summary.rows[0].latest_decision_id, "decision-002");
        assert_eq!(summary.rows[0].completion_status, "completed");
        assert_eq!(summary.rows[0].retention_status, "retained");
        assert_eq!(
            summary.rows[0]
                .baseline_scorecard
                .as_ref()
                .expect("baseline scorecard")
                .generated_at,
            1_700_000_100
        );
        assert_eq!(
            summary.rows[0]
                .candidate_scorecard
                .as_ref()
                .expect("candidate scorecard")
                .generated_at,
            1_700_000_400
        );
        assert_eq!(summary.rows[0].benchmark_deltas[0].comparison_status, "improved");
        assert_eq!(summary.rows[0].evidence_references.len(), 2);
    }
}
