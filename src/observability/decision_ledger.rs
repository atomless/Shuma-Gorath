use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::challenge::KeyValueStore;

const OPERATOR_DECISION_LEDGER_SCHEMA_VERSION: &str = "operator_decision_ledger_v1";
const OPERATOR_DECISION_LEDGER_PREFIX: &str = "operator_decision_ledger:v1";
const OPERATOR_DECISION_LEDGER_MAX_ROWS: usize = 24;
const OPERATOR_DECISION_LEDGER_MAX_EVIDENCE_REFERENCES: usize = 3;
const OPERATOR_DECISION_LEDGER_MAX_TEXT_CHARS: usize = 160;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorDecisionEvidenceReference {
    pub kind: String,
    pub reference: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorDecisionRecord {
    pub decision_id: String,
    pub recorded_at_ts: u64,
    pub decision_kind: String,
    pub decision_status: String,
    pub source: String,
    pub changed_families: Vec<String>,
    pub targets: Vec<String>,
    pub objective_revision: String,
    pub watch_window_seconds: u64,
    pub expected_impact_summary: String,
    pub evidence_references: Vec<OperatorDecisionEvidenceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OperatorDecisionDraft {
    pub recorded_at_ts: u64,
    pub decision_kind: String,
    pub decision_status: String,
    pub source: String,
    pub changed_families: Vec<String>,
    pub targets: Vec<String>,
    pub objective_revision: String,
    pub watch_window_seconds: u64,
    pub expected_impact_summary: String,
    pub evidence_references: Vec<OperatorDecisionEvidenceReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OperatorDecisionLedgerState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OperatorDecisionRecord>,
}

fn operator_decision_ledger_key(site_id: &str) -> String {
    format!("{OPERATOR_DECISION_LEDGER_PREFIX}:{site_id}")
}

fn load_decision_ledger_state<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OperatorDecisionLedgerState {
    store
        .get(&operator_decision_ledger_key(site_id))
        .ok()
        .flatten()
        .and_then(|bytes| {
            serde_json::from_slice::<OperatorDecisionLedgerState>(bytes.as_slice()).ok()
        })
        .filter(|state| state.schema_version == OPERATOR_DECISION_LEDGER_SCHEMA_VERSION)
        .unwrap_or_else(|| OperatorDecisionLedgerState {
            schema_version: OPERATOR_DECISION_LEDGER_SCHEMA_VERSION.to_string(),
            updated_at_ts: 0,
            rows: Vec::new(),
        })
}

fn save_decision_ledger_state<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OperatorDecisionLedgerState,
) -> Result<(), ()> {
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store
        .set(&operator_decision_ledger_key(site_id), payload.as_slice())
        .map_err(|_| ())
}

pub(crate) fn record_decision<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    draft: OperatorDecisionDraft,
) -> Result<OperatorDecisionRecord, ()> {
    let record = OperatorDecisionRecord {
        decision_id: decision_id_for_draft(&draft),
        recorded_at_ts: draft.recorded_at_ts,
        decision_kind: draft.decision_kind,
        decision_status: draft.decision_status,
        source: draft.source,
        changed_families: draft.changed_families,
        targets: draft.targets,
        objective_revision: draft.objective_revision,
        watch_window_seconds: draft.watch_window_seconds,
        expected_impact_summary: truncate_text(draft.expected_impact_summary.as_str()),
        evidence_references: draft
            .evidence_references
            .into_iter()
            .take(OPERATOR_DECISION_LEDGER_MAX_EVIDENCE_REFERENCES)
            .map(|reference| OperatorDecisionEvidenceReference {
                kind: truncate_text(reference.kind.as_str()),
                reference: truncate_text(reference.reference.as_str()),
                note: truncate_text(reference.note.as_str()),
            })
            .collect(),
    };

    let mut state = load_decision_ledger_state(store, site_id);
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
    state.rows.truncate(OPERATOR_DECISION_LEDGER_MAX_ROWS);
    state.updated_at_ts = record.recorded_at_ts;
    state.schema_version = OPERATOR_DECISION_LEDGER_SCHEMA_VERSION.to_string();
    save_decision_ledger_state(store, site_id, &state)?;
    Ok(record)
}

pub(crate) fn load_recent_decision_map<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> HashMap<String, OperatorDecisionRecord> {
    load_decision_ledger_state(store, site_id)
        .rows
        .into_iter()
        .map(|row| (row.decision_id.clone(), row))
        .collect()
}

fn decision_id_for_draft(draft: &OperatorDecisionDraft) -> String {
    let mut hasher = DefaultHasher::new();
    draft.recorded_at_ts.hash(&mut hasher);
    draft.decision_kind.hash(&mut hasher);
    draft.source.hash(&mut hasher);
    draft.changed_families.hash(&mut hasher);
    draft.targets.hash(&mut hasher);
    draft.objective_revision.hash(&mut hasher);
    draft.expected_impact_summary.hash(&mut hasher);
    format!("decision-{}-{:016x}", draft.recorded_at_ts, hasher.finish())
}

fn truncate_text(value: &str) -> String {
    if value.chars().count() <= OPERATOR_DECISION_LEDGER_MAX_TEXT_CHARS {
        return value.to_string();
    }
    value
        .chars()
        .take(OPERATOR_DECISION_LEDGER_MAX_TEXT_CHARS.saturating_sub(3))
        .collect::<String>()
        + "..."
}

#[cfg(test)]
mod tests {
    use super::{
        load_recent_decision_map, record_decision, OperatorDecisionDraft,
        OperatorDecisionEvidenceReference,
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

    #[test]
    fn record_decision_persists_bounded_recent_rows() {
        let store = TestStore::new();
        let record = record_decision(
            &store,
            "default",
            OperatorDecisionDraft {
                recorded_at_ts: 1_700_000_000,
                decision_kind: "manual_config_patch".to_string(),
                decision_status: "applied".to_string(),
                source: "manual_admin".to_string(),
                changed_families: vec!["maze_core".to_string()],
                targets: vec!["maze".to_string()],
                objective_revision: "rev-1700000000".to_string(),
                watch_window_seconds: 86_400,
                expected_impact_summary: "Observe the standard objective window for suspicious-origin budget movement.".to_string(),
                evidence_references: vec![OperatorDecisionEvidenceReference {
                    kind: "operator_objectives".to_string(),
                    reference: "rev-1700000000".to_string(),
                    note: "Objective revision active when the manual config change was applied."
                        .to_string(),
                }],
            },
        )
        .expect("record persists");

        let decisions = load_recent_decision_map(&store, "default");
        let loaded = decisions.get(&record.decision_id).expect("decision exists");

        assert_eq!(loaded.decision_kind, "manual_config_patch");
        assert_eq!(loaded.objective_revision, "rev-1700000000");
        assert_eq!(loaded.evidence_references.len(), 1);
    }
}
