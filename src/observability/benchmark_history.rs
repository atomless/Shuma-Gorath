use crate::challenge::KeyValueStore;

use super::benchmark_comparison::{
    comparable_snapshot_from_results, BenchmarkComparableSnapshot,
};
use super::hot_read_documents::{
    operator_snapshot_document_contract, operator_snapshot_document_key,
    OperatorSnapshotHotReadDocument,
};

pub(crate) fn load_prior_window_reference<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    current_generated_at: u64,
) -> Option<BenchmarkComparableSnapshot> {
    let bytes = store
        .get(&operator_snapshot_document_key(site_id))
        .ok()
        .flatten()?;
    let document =
        serde_json::from_slice::<OperatorSnapshotHotReadDocument>(bytes.as_slice()).ok()?;
    if document.metadata.schema_version != operator_snapshot_document_contract().schema_version {
        return None;
    }
    let snapshot = comparable_snapshot_from_results(&document.payload.benchmark_results);
    (snapshot.generated_at < current_generated_at).then_some(snapshot)
}
