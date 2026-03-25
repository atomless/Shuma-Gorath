#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelemetryExactness {
    Exact,
    Derived,
    BestEffort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TelemetryBasis {
    Observed,
    Policy,
    Verified,
    Residual,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadOwnershipTier {
    BootstrapCritical,
    SupportingSummary,
    DiagnosticOrDrilldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadCanonicalSource {
    ImmutableEventLog,
    DirectStateSnapshot,
    MutableCounter,
    MutableCatalog,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HotReadProjectionModel {
    DeterministicRebuild,
    CommutativeAppendOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct HotReadComponentContract {
    pub key: &'static str,
    pub exactness: TelemetryExactness,
    pub basis: TelemetryBasis,
    pub ownership_tier: HotReadOwnershipTier,
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

const MONITORING_BOOTSTRAP_COMPONENTS: [HotReadComponentContract; 7] = [
    HotReadComponentContract {
        key: "runtime_posture_summary",
        exactness: TelemetryExactness::Exact,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Derived from current config/runtime state rather than accumulated telemetry.",
    },
    HotReadComponentContract {
        key: "monitoring_summary",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Current monitoring summary is rebuilt from mutable counters and remains best-effort until newer operator summaries land.",
    },
    HotReadComponentContract {
        key: "recent_events_tail",
        exactness: TelemetryExactness::Exact,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::ImmutableEventLog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Must remain rebuildable from immutable event records so edge writers do not race on tail mutation.",
    },
    HotReadComponentContract {
        key: "recent_sim_runs_summary",
        exactness: TelemetryExactness::Exact,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::ImmutableEventLog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Compact recent run history must remain rebuildable from immutable simulation-tagged event records rather than mutable run-state caches.",
    },
    HotReadComponentContract {
        key: "security_privacy_summary",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Current hourly security/privacy counters are shared mutable KV state and are not a safe exact source on non-atomic edge KV.",
    },
    HotReadComponentContract {
        key: "retention_health_summary",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::MutableCatalog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Retention worker catalogs and state are currently maintained with read-modify-write over shared KV.",
    },
    HotReadComponentContract {
        key: "active_ban_summary",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Direct state reads are cheap, but enterprise fallback semantics are still not strict enough to classify this summary as exact across all targets.",
    },
];

const OPERATOR_SNAPSHOT_COMPONENTS: [HotReadComponentContract; 14] = [
    HotReadComponentContract {
        key: "objectives",
        exactness: TelemetryExactness::Exact,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Objective profile is a persisted site-owned direct-state contract with server-assigned revision metadata, seeded conservatively when missing and then read as the operator-facing truth.",
    },
    HotReadComponentContract {
        key: "live_traffic",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Live-traffic summary is currently derived from bounded monitoring counters and request-outcome summaries.",
    },
    HotReadComponentContract {
        key: "shadow_mode",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Shadow section combines runtime posture and shadow-action counters into one bounded snapshot view.",
    },
    HotReadComponentContract {
        key: "adversary_sim",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::ImmutableEventLog,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Adversary-sim section uses bounded recent-run summaries and request-outcome aggregates while remaining separate from live ingress.",
    },
    HotReadComponentContract {
        key: "runtime_posture",
        exactness: TelemetryExactness::Exact,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Runtime posture is read directly from current config and runtime environment state.",
    },
    HotReadComponentContract {
        key: "budget_distance",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Budget-distance rows are derived from bounded summaries and backend-owned objective budgets rather than raw counters alone.",
    },
    HotReadComponentContract {
        key: "non_human_traffic",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Non-human traffic section exposes the seeded canonical taxonomy, the classifier decision chain, bounded category receipts, and readiness blockers on the same operator-facing contract used later by category-aware objective, coverage, and tuning gates.",
    },
    HotReadComponentContract {
        key: "benchmark_results",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Mixed,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::MutableCounter,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Benchmark results are a bounded machine-first projection over operator snapshot sections, currently materialize prior-window comparison against the last snapshot, and keep the same comparison contract reusable for later baseline or candidate subjects.",
    },
    HotReadComponentContract {
        key: "recent_changes",
        exactness: TelemetryExactness::BestEffort,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Recent-change ledger is maintained from meaningful admin/controller mutation writes and exposed as a bounded change-plus-decision summary with watch-window state and durable evidence references.",
    },
    HotReadComponentContract {
        key: "allowed_actions",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Allowed-actions surface enumerates the bounded controller write contract, including allowed, manual-only, and forbidden config groups plus canary guardrails.",
    },
    HotReadComponentContract {
        key: "game_contract",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Policy,
        ownership_tier: HotReadOwnershipTier::BootstrapCritical,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Game-contract surface names the immutable rules, evaluator boundary, legal move ring, safety gates, and regression anchors that later recursive-improvement phases must inherit.",
    },
    HotReadComponentContract {
        key: "episode_archive",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Episode-archive surface persists bounded completed loop outcomes, baseline scorecards, benchmark deltas, and explicit homeostasis judgments for later recursive-improvement memory.",
    },
    HotReadComponentContract {
        key: "verified_identity",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Verified,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Verified-identity summary is a bounded typed projection over current verified-identity telemetry plus live policy stance and capability counts, rather than a placeholder note.",
    },
    HotReadComponentContract {
        key: "replay_promotion",
        exactness: TelemetryExactness::Derived,
        basis: TelemetryBasis::Observed,
        ownership_tier: HotReadOwnershipTier::SupportingSummary,
        canonical_source: HotReadCanonicalSource::DirectStateSnapshot,
        projection_model: HotReadProjectionModel::DeterministicRebuild,
        note: "Replay-promotion summary is a bounded typed projection over persisted emergent-finding and deterministic-replay lineage materialized by the promotion triage lane, rather than a sidecar JSON artifact.",
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

pub(crate) fn operator_snapshot_component_contracts() -> &'static [HotReadComponentContract] {
    &OPERATOR_SNAPSHOT_COMPONENTS
}

pub(crate) fn current_hot_read_projection_contract() -> HotReadProjectionContract {
    CURRENT_PROJECTION_CONTRACT
}

#[cfg(test)]
mod tests {
    use super::{
        current_hot_read_projection_contract, monitoring_bootstrap_component_contracts,
        operator_snapshot_component_contracts, HotReadCanonicalSource, HotReadComponentContract,
        HotReadOwnershipTier, HotReadProjectionModel, TelemetryBasis, TelemetryExactness,
    };
    use serde_json::Value;

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
        assert_eq!(contract.basis, TelemetryBasis::Observed);
        assert_eq!(
            contract.ownership_tier,
            HotReadOwnershipTier::SupportingSummary
        );
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
    fn recent_sim_runs_summary_is_exact_and_immutable() {
        let contract = contract_for("recent_sim_runs_summary");
        assert_eq!(contract.exactness, TelemetryExactness::Exact);
        assert_eq!(contract.basis, TelemetryBasis::Observed);
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
        assert_eq!(contract.basis, TelemetryBasis::Mixed);
        assert_eq!(
            contract.canonical_source,
            HotReadCanonicalSource::MutableCounter
        );
    }

    #[test]
    fn monitoring_summary_is_bootstrap_critical_best_effort_component() {
        let contract = contract_for("monitoring_summary");
        assert_eq!(contract.exactness, TelemetryExactness::BestEffort);
        assert_eq!(contract.basis, TelemetryBasis::Mixed);
        assert_eq!(
            contract.ownership_tier,
            HotReadOwnershipTier::BootstrapCritical
        );
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

    #[test]
    fn component_contracts_serialize_basis_and_ownership_tier() {
        let contract = contract_for("recent_events_tail");
        let value = serde_json::to_value(contract).expect("component contract serializes");
        let object = value.as_object().expect("contract is object");
        assert_eq!(
            object.get("basis"),
            Some(&Value::String("observed".to_string()))
        );
        assert_eq!(
            object.get("ownership_tier"),
            Some(&Value::String("supporting_summary".to_string()))
        );
    }

    #[test]
    fn operator_snapshot_contracts_include_budget_distance_and_runtime_posture() {
        let keys: Vec<_> = operator_snapshot_component_contracts()
            .iter()
            .map(|component| component.key)
            .collect();
        assert!(keys.contains(&"objectives"));
        assert!(keys.contains(&"live_traffic"));
        assert!(keys.contains(&"budget_distance"));
        assert!(keys.contains(&"non_human_traffic"));
        assert!(keys.contains(&"benchmark_results"));
        assert!(keys.contains(&"runtime_posture"));
        assert!(keys.contains(&"game_contract"));
        assert!(keys.contains(&"episode_archive"));
        assert!(keys.contains(&"replay_promotion"));
    }
}
