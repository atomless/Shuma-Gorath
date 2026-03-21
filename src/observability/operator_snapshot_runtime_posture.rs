use serde::{Deserialize, Serialize};

use crate::challenge::KeyValueStore;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OperatorSnapshotRuntimePosture {
    pub shadow_mode: bool,
    pub fail_mode: String,
    pub runtime_environment: String,
    pub gateway_deployment_profile: String,
    pub adversary_sim_available: bool,
}

pub(super) fn runtime_shadow_mode<S: KeyValueStore>(store: &S, site_id: &str) -> bool {
    crate::config::load_runtime_cached(store, site_id)
        .map(|cfg| cfg.shadow_mode)
        .unwrap_or(false)
}

pub(super) fn runtime_posture<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OperatorSnapshotRuntimePosture {
    OperatorSnapshotRuntimePosture {
        shadow_mode: runtime_shadow_mode(store, site_id),
        fail_mode: if crate::config::kv_store_fail_open() {
            "open".to_string()
        } else {
            "closed".to_string()
        },
        runtime_environment: crate::config::runtime_environment().as_str().to_string(),
        gateway_deployment_profile: crate::config::gateway_deployment_profile()
            .as_str()
            .to_string(),
        adversary_sim_available: crate::config::adversary_sim_available(),
    }
}
