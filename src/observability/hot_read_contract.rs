#![cfg_attr(not(test), allow(dead_code))]

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelemetryExactness {
    Exact,
    BestEffort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadCanonicalSource {
    ImmutableEventLog,
    DirectStateSnapshot,
    MutableCounter,
    MutableCatalog,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadProjectionModel {
    DeterministicRebuild,
    CommutativeAppendOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct HotReadComponentContract {
    pub key: &'static str,
    pub exactness: TelemetryExactness,
    pub canonical_source: HotReadCanonicalSource,
    pub projection_model: HotReadProjectionModel,
    pub note: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct HotReadProjectionContract {
    pub shared_read_modify_write_forbidden: bool,
    pub sqlite_split_forbidden: bool,
    pub external_database_required: bool,
}

const MONITORING_BOOTSTRAP_COMPONENTS: [HotReadComponentContract; 5] = [
    HotReadComponentContract {
        key: "runtime_posture_summary",
        exactness: TelemetryExactness::Exact,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Derived from current config/runtime state rather than accumulated telemetry.",
    },
    HotReadComponentContract {
        key: "recent_events_tail",
        exactness: TelemetryExactness::Exact,
        canonical_source: HotReadCanonicalSource::ImmutableEventLog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Must remain rebuildable from immutable event records so edge writers do not race on tail mutation.",
    },
    HotReadComponentContract {
        key: "security_privacy_summary",
        exactness: TelemetryExactness::BestEffort,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Current hourly security/privacy counters are shared mutable KV state and are not a safe exact source on non-atomic edge KV.",
    },
    HotReadComponentContract {
        key: "retention_health_summary",
        exactness: TelemetryExactness::BestEffort,
        canonical_source: HotReadCanonicalSource::MutableCatalog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Retention worker catalogs and state are currently maintained with read-modify-write over shared KV.",
    },
    HotReadComponentContract {
        key: "active_ban_summary",
        exactness: TelemetryExactness::BestEffort,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Direct state reads are cheap, but enterprise fallback semantics are still not strict enough to classify this summary as exact across all targets.",
    },
];

const CURRENT_PROJECTION_CONTRACT: HotReadProjectionContract = HotReadProjectionContract {
    shared_read_modify_write_forbidden: true,
    sqlite_split_forbidden: true,
    external_database_required: false,
};

pub(crate) fn monitoring_bootstrap_component_contracts() -> &'static [HotReadComponentContract] {
    &MONITORING_BOOTSTRAP_COMPONENTS
}

pub(crate) fn current_hot_read_projection_contract() -> HotReadProjectionContract {
    CURRENT_PROJECTION_CONTRACT
}

#[cfg(test)]
mod tests {
    use super::{
        current_hot_read_projection_contract, monitoring_bootstrap_component_contracts,
        HotReadCanonicalSource, HotReadComponentContract, HotReadProjectionModel,
        TelemetryExactness,
    };

    fn contract_for(key: &str) -> HotReadComponentContract {
        monitoring_bootstrap_component_contracts()
            .iter()
            .copied()
            .find(|component| component.key == key)
            .expect("contract exists")
    }

    #[test]
    fn recent_events_tail_is_exact_and_immutable() {
        let contract = contract_for("recent_events_tail");
        assert_eq!(contract.exactness, TelemetryExactness::Exact);
        assert_eq!(
            contract.canonical_source,
            HotReadCanonicalSource::ImmutableEventLog
        );
        assert_eq!(
            contract.projection_model,
            HotReadProjectionModel::DeterministicRebuild
        );
    }

    #[test]
    fn security_privacy_summary_is_best_effort_from_mutable_counters() {
        let contract = contract_for("security_privacy_summary");
        assert_eq!(contract.exactness, TelemetryExactness::BestEffort);
        assert_eq!(
            contract.canonical_source,
            HotReadCanonicalSource::MutableCounter
        );
    }

    #[test]
    fn projection_contract_forbids_shared_projection_mutation() {
        let contract = current_hot_read_projection_contract();
        assert!(contract.shared_read_modify_write_forbidden);
        assert!(contract.sqlite_split_forbidden);
        assert!(!contract.external_database_required);
    }
}
