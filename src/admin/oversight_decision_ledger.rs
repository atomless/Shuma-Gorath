use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

const OVERSIGHT_DECISION_LEDGER_SCHEMA_VERSION: &str = "oversight_decision_ledger_v1";
const OVERSIGHT_DECISION_LEDGER_PREFIX: &str = "oversight_decision_ledger:v1";
const OVERSIGHT_DECISION_LEDGER_MAX_ROWS: usize = 24;
const OVERSIGHT_DECISION_LEDGER_MAX_TEXT_CHARS: usize = 240;
const OVERSIGHT_DECISION_LEDGER_MAX_ISSUES: usize = 6;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OversightDecisionEvidenceReference {
    pub kind: String,
    pub reference: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightDecisionRecord {
    pub decision_id: String,
    pub recorded_at_ts: u64,
    pub trigger_source: String,
    pub outcome: String,
    pub summary: String,
    pub objective_revision: String,
    pub snapshot_generated_at: u64,
    pub benchmark_overall_status: String,
    pub improvement_status: String,
    pub replay_promotion_availability: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_family_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidate_action_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal: Option<crate::admin::oversight_patch_policy::OversightPatchProposal>,
    pub validation_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_issues: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_sim_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_references: Vec<OversightDecisionEvidenceReference>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OversightDecisionDraft {
    pub recorded_at_ts: u64,
    pub trigger_source: String,
    pub outcome: String,
    pub summary: String,
    pub objective_revision: String,
    pub snapshot_generated_at: u64,
    pub benchmark_overall_status: String,
    pub improvement_status: String,
    pub replay_promotion_availability: String,
    pub trigger_family_ids: Vec<String>,
    pub candidate_action_families: Vec<String>,
    pub refusal_reasons: Vec<String>,
    pub proposal: Option<crate::admin::oversight_patch_policy::OversightPatchProposal>,
    pub validation_status: String,
    pub validation_issues: Vec<String>,
    pub latest_sim_run_id: Option<String>,
    pub evidence_references: Vec<OversightDecisionEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OversightDecisionLedgerState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OversightDecisionRecord>,
}

fn ledger_key(site_id: &str) -> String {
    format!("{OVERSIGHT_DECISION_LEDGER_PREFIX}:{site_id}")
}

fn load_state<S: KeyValueStore>(store: &S, site_id: &str) -> OversightDecisionLedgerState {
    store
        .get(ledger_key(site_id).as_str())
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<OversightDecisionLedgerState>(&bytes).ok())
        .filter(|state| state.schema_version == OVERSIGHT_DECISION_LEDGER_SCHEMA_VERSION)
        .unwrap_or_else(|| OversightDecisionLedgerState {
            schema_version: OVERSIGHT_DECISION_LEDGER_SCHEMA_VERSION.to_string(),
            updated_at_ts: 0,
            rows: Vec::new(),
        })
}

fn save_state<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OversightDecisionLedgerState,
) -> Result<(), ()> {
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(ledger_key(site_id).as_str(), payload.as_slice())
}

pub(crate) fn record_decision<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    draft: OversightDecisionDraft,
) -> Result<OversightDecisionRecord, ()> {
    let record = OversightDecisionRecord {
        decision_id: decision_id(&draft),
        recorded_at_ts: draft.recorded_at_ts,
        trigger_source: draft.trigger_source,
        outcome: draft.outcome,
        summary: truncate_text(draft.summary.as_str()),
        objective_revision: truncate_text(draft.objective_revision.as_str()),
        snapshot_generated_at: draft.snapshot_generated_at,
        benchmark_overall_status: draft.benchmark_overall_status,
        improvement_status: draft.improvement_status,
        replay_promotion_availability: draft.replay_promotion_availability,
        trigger_family_ids: draft.trigger_family_ids,
        candidate_action_families: draft.candidate_action_families,
        refusal_reasons: draft.refusal_reasons,
        proposal: draft.proposal,
        validation_status: draft.validation_status,
        validation_issues: draft
            .validation_issues
            .into_iter()
            .take(OVERSIGHT_DECISION_LEDGER_MAX_ISSUES)
            .map(|issue| truncate_text(issue.as_str()))
            .collect(),
        latest_sim_run_id: draft.latest_sim_run_id,
        evidence_references: draft
            .evidence_references
            .into_iter()
            .map(|reference| OversightDecisionEvidenceReference {
                kind: truncate_text(reference.kind.as_str()),
                reference: truncate_text(reference.reference.as_str()),
                note: truncate_text(reference.note.as_str()),
            })
            .collect(),
    };

    let mut state = load_state(store, site_id);
    state
        .rows
        .retain(|existing| existing.decision_id != record.decision_id);
    state.rows.push(record.clone());
    state.rows.sort_by(|left, right| {
        right
            .recorded_at_ts
            .cmp(&left.recorded_at_ts)
            .then_with(|| left.decision_id.cmp(&right.decision_id))
    });
    state.rows.truncate(OVERSIGHT_DECISION_LEDGER_MAX_ROWS);
    state.updated_at_ts = record.recorded_at_ts;
    state.schema_version = OVERSIGHT_DECISION_LEDGER_SCHEMA_VERSION.to_string();
    save_state(store, site_id, &state)?;
    Ok(record)
}

pub(crate) fn load_recent_decisions<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Vec<OversightDecisionRecord> {
    load_state(store, site_id).rows
}

pub(crate) fn load_latest_decision<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<OversightDecisionRecord> {
    load_state(store, site_id).rows.into_iter().next()
}

fn decision_id(draft: &OversightDecisionDraft) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    draft.recorded_at_ts.hash(&mut hasher);
    draft.trigger_source.hash(&mut hasher);
    draft.outcome.hash(&mut hasher);
    draft.summary.hash(&mut hasher);
    draft.objective_revision.hash(&mut hasher);
    draft.snapshot_generated_at.hash(&mut hasher);
    draft.benchmark_overall_status.hash(&mut hasher);
    draft.improvement_status.hash(&mut hasher);
    draft.trigger_family_ids.hash(&mut hasher);
    draft.candidate_action_families.hash(&mut hasher);
    draft.refusal_reasons.hash(&mut hasher);
    if let Some(proposal) = draft.proposal.as_ref() {
        proposal.patch_family.hash(&mut hasher);
        proposal.patch.to_string().hash(&mut hasher);
    }
    format!("oversight-{}-{:016x}", draft.recorded_at_ts, hasher.finish())
}

fn truncate_text(value: &str) -> String {
    if value.chars().count() <= OVERSIGHT_DECISION_LEDGER_MAX_TEXT_CHARS {
        return value.to_string();
    }
    value
        .chars()
        .take(OVERSIGHT_DECISION_LEDGER_MAX_TEXT_CHARS.saturating_sub(3))
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::{
        load_latest_decision, load_recent_decisions, record_decision, OversightDecisionDraft,
        OversightDecisionEvidenceReference,
    };
    use crate::challenge::KeyValueStore;
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

    fn draft(recorded_at_ts: u64, outcome: &str) -> OversightDecisionDraft {
        OversightDecisionDraft {
            recorded_at_ts,
            trigger_source: "manual_admin".to_string(),
            outcome: outcome.to_string(),
            summary: format!("summary-{recorded_at_ts}"),
            objective_revision: "rev-1700000000".to_string(),
            snapshot_generated_at: recorded_at_ts,
            benchmark_overall_status: "outside_budget".to_string(),
            improvement_status: "regressed".to_string(),
            replay_promotion_availability: "not_materialized".to_string(),
            trigger_family_ids: vec!["suspicious_origin_cost".to_string()],
            candidate_action_families: vec!["fingerprint_signal".to_string()],
            refusal_reasons: Vec::new(),
            proposal: None,
            validation_status: "skipped".to_string(),
            validation_issues: Vec::new(),
            latest_sim_run_id: Some("simrun-001".to_string()),
            evidence_references: vec![OversightDecisionEvidenceReference {
                kind: "operator_snapshot".to_string(),
                reference: "snapshot-001".to_string(),
                note: "Bounded machine-first input.".to_string(),
            }],
        }
    }

    #[test]
    fn records_and_loads_latest_decision() {
        let store = TestStore::new();
        let record = record_decision(&store, "default", draft(1_700_000_000, "recommend_patch"))
            .expect("record persists");

        let latest = load_latest_decision(&store, "default").expect("latest exists");
        assert_eq!(latest.decision_id, record.decision_id);
        assert_eq!(latest.outcome, "recommend_patch");
    }

    #[test]
    fn recent_decisions_are_sorted_newest_first() {
        let store = TestStore::new();
        record_decision(&store, "default", draft(1_700_000_000, "observe_longer"))
            .expect("first record persists");
        record_decision(&store, "default", draft(1_700_000_010, "recommend_patch"))
            .expect("second record persists");

        let recent = load_recent_decisions(&store, "default");
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].recorded_at_ts, 1_700_000_010);
        assert_eq!(recent[1].recorded_at_ts, 1_700_000_000);
    }
}
