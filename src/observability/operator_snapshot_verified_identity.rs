use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OperatorSnapshotPlaceholderSection {
    pub availability: String,
    pub note: String,
}

pub(super) fn placeholder_section(
    availability: &str,
    note: &str,
) -> OperatorSnapshotPlaceholderSection {
    OperatorSnapshotPlaceholderSection {
        availability: availability.to_string(),
        note: note.to_string(),
    }
}

pub(super) fn verified_identity_placeholder_section() -> OperatorSnapshotPlaceholderSection {
    placeholder_section(
        "not_yet_supported",
        "Verified identity summaries land with the verified bot identity foundation.",
    )
}
