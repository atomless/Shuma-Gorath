use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::challenge::KeyValueStore;
use crate::config::Config;
use crate::observability::benchmark_comparison::{
    apply_candidate_comparison, comparable_snapshot_from_results, BenchmarkComparableSnapshot,
};
use crate::observability::decision_ledger::{
    record_decision as record_operator_decision, OperatorDecisionDraft,
    OperatorDecisionEvidenceReference,
};
use crate::observability::operator_snapshot::OperatorSnapshotHotReadPayload;
use crate::observability::operator_snapshot_objectives::operator_objectives_watch_window_seconds;

use super::oversight_api::OversightPatchValidation;
use super::oversight_patch_policy::OversightPatchProposal;
use super::oversight_reconcile::{
    contradictory_evidence_reasons, stale_evidence_reasons, OversightReconcileResult,
};
use super::recent_changes_ledger::{
    operator_snapshot_config_patch_recent_change_row,
    operator_snapshot_recent_change_with_decision_id, record_operator_snapshot_recent_change_rows,
};

pub(crate) const OVERSIGHT_APPLY_STAGE_ELIGIBLE: &str = "eligible";
pub(crate) const OVERSIGHT_APPLY_STAGE_CANARY_APPLIED: &str = "canary_applied";
pub(crate) const OVERSIGHT_APPLY_STAGE_WATCH_WINDOW_OPEN: &str = "watch_window_open";
pub(crate) const OVERSIGHT_APPLY_STAGE_IMPROVED: &str = "improved";
pub(crate) const OVERSIGHT_APPLY_STAGE_REFUSED: &str = "refused";
pub(crate) const OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED: &str = "rollback_applied";

const OVERSIGHT_APPLY_SCHEMA_VERSION: &str = "oversight_apply_v1";
const OVERSIGHT_ACTIVE_CANARY_SCHEMA_VERSION: &str = "oversight_active_canary_v1";
const OVERSIGHT_ACTIVE_CANARY_PREFIX: &str = "oversight_active_canary:v1";
const OVERSIGHT_CONTROLLER_ADMIN_ID: &str = "controller:oversight_canary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OversightApplyMode {
    PreviewOnly,
    ExecuteCanary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightApplyResult {
    pub schema_version: String,
    pub stage: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watch_window_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watch_window_started_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watch_window_end_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_generated_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_generated_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rollback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OversightActiveCanaryState {
    schema_version: String,
    canary_id: String,
    opened_at_ts: u64,
    trigger_source: String,
    objective_revision: String,
    watch_window_seconds: u64,
    watch_window_end_at: u64,
    proposal: OversightPatchProposal,
    baseline_snapshot: BenchmarkComparableSnapshot,
    previous_config: Config,
}

pub(crate) fn evaluate_apply_cycle<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    now: u64,
    snapshot: Option<&OperatorSnapshotHotReadPayload>,
    current_cfg: Option<&Config>,
    reconcile: &OversightReconcileResult,
    validation: &OversightPatchValidation,
    mode: OversightApplyMode,
) -> Result<OversightApplyResult, ()> {
    let Some(snapshot) = snapshot else {
        return Ok(refused_result(
            "Oversight cannot evaluate bounded canary apply because the operator snapshot is not materialized.",
            vec!["operator_snapshot_not_materialized".to_string()],
            reconcile.proposal.as_ref(),
        ));
    };

    if let Some(active_canary) = load_active_canary(store, site_id) {
        return continue_active_canary(
            store,
            site_id,
            now,
            snapshot,
            current_cfg,
            &active_canary,
            mode,
        );
    }

    let refusal_reasons =
        apply_refusal_reasons(snapshot, current_cfg, reconcile, validation, false);
    if !refusal_reasons.is_empty() {
        return Ok(refused_result(
            "Current oversight cycle is not allowed to mutate config automatically.",
            refusal_reasons,
            reconcile.proposal.as_ref(),
        ));
    }

    let Some(current_cfg) = current_cfg else {
        return Ok(refused_result(
            "Current runtime config is unavailable, so canary apply must fail closed.",
            vec!["config_unavailable".to_string()],
            reconcile.proposal.as_ref(),
        ));
    };
    let Some(proposal) = reconcile.proposal.as_ref() else {
        return Ok(refused_result(
            "No bounded patch proposal is available for automatic apply.",
            vec!["proposal_missing".to_string()],
            None,
        ));
    };

    if mode == OversightApplyMode::PreviewOnly {
        return Ok(OversightApplyResult {
            schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
            stage: OVERSIGHT_APPLY_STAGE_ELIGIBLE.to_string(),
            summary: "This bounded proposal is eligible for shared-host canary apply, but the manual reconcile surface remains preview-only.".to_string(),
            refusal_reasons: Vec::new(),
            patch_family: Some(proposal.patch_family.clone()),
            watch_window_seconds: Some(operator_objectives_watch_window_seconds(
                &snapshot.objectives,
            )),
            watch_window_started_at: None,
            watch_window_end_at: None,
            baseline_generated_at: Some(snapshot.benchmark_results.generated_at),
            candidate_generated_at: None,
            comparison_status: None,
            rollback_reason: None,
        });
    }

    let updated_cfg = match crate::config::apply_persisted_patch(current_cfg, &proposal.patch) {
        Ok(cfg) => cfg,
        Err(reason) => {
            return Ok(refused_result(
                "The bounded controller patch could not be merged into persisted config safely.",
                vec![format!("patch_apply_invalid:{reason}")],
                Some(proposal),
            ));
        }
    };
    let watch_window_seconds = operator_objectives_watch_window_seconds(&snapshot.objectives);
    let active_canary = OversightActiveCanaryState {
        schema_version: OVERSIGHT_ACTIVE_CANARY_SCHEMA_VERSION.to_string(),
        canary_id: canary_id(now, proposal.patch_family.as_str(), &proposal.patch),
        opened_at_ts: now,
        trigger_source: reconcile.trigger_source.clone(),
        objective_revision: snapshot.objectives.revision.clone(),
        watch_window_seconds,
        watch_window_end_at: now.saturating_add(watch_window_seconds),
        proposal: proposal.clone(),
        baseline_snapshot: comparable_snapshot_from_results(&snapshot.benchmark_results),
        previous_config: current_cfg.clone(),
    };
    save_active_canary(store, site_id, &active_canary)?;
    if let Err(err) = persist_config_change(
        store,
        site_id,
        now,
        current_cfg,
        &updated_cfg,
        &proposal.patch,
        "oversight_canary_apply",
        "applied",
        &snapshot.objectives.revision,
        watch_window_seconds,
        "Bounded oversight canary apply changed one controller-approved config family and opened a protected watch window.",
        vec![
            operator_evidence_reference(
                "oversight_canary_id",
                active_canary.canary_id.as_str(),
                "Active canary identifier for the first bounded autonomous tuning loop.",
            ),
            operator_evidence_reference(
                "benchmark_baseline_generated_at",
                snapshot.benchmark_results.generated_at.to_string().as_str(),
                "Benchmark baseline captured immediately before canary apply.",
            ),
        ],
    ) {
        clear_active_canary(store, site_id);
        return Err(err);
    }

    Ok(OversightApplyResult {
        schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
        stage: OVERSIGHT_APPLY_STAGE_CANARY_APPLIED.to_string(),
        summary: "Oversight applied one bounded config canary and opened the protected watch window.".to_string(),
        refusal_reasons: Vec::new(),
        patch_family: Some(proposal.patch_family.clone()),
        watch_window_seconds: Some(watch_window_seconds),
        watch_window_started_at: Some(now),
        watch_window_end_at: Some(active_canary.watch_window_end_at),
        baseline_generated_at: Some(snapshot.benchmark_results.generated_at),
        candidate_generated_at: None,
        comparison_status: None,
        rollback_reason: None,
    })
}

fn continue_active_canary<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    now: u64,
    snapshot: &OperatorSnapshotHotReadPayload,
    current_cfg: Option<&Config>,
    active_canary: &OversightActiveCanaryState,
    mode: OversightApplyMode,
) -> Result<OversightApplyResult, ()> {
    if now < active_canary.watch_window_end_at {
        return Ok(OversightApplyResult {
            schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
            stage: OVERSIGHT_APPLY_STAGE_WATCH_WINDOW_OPEN.to_string(),
            summary: "A bounded canary is still inside its protected watch window; no additional mutation is allowed yet.".to_string(),
            refusal_reasons: Vec::new(),
            patch_family: Some(active_canary.proposal.patch_family.clone()),
            watch_window_seconds: Some(active_canary.watch_window_seconds),
            watch_window_started_at: Some(active_canary.opened_at_ts),
            watch_window_end_at: Some(active_canary.watch_window_end_at),
            baseline_generated_at: Some(active_canary.baseline_snapshot.generated_at),
            candidate_generated_at: Some(snapshot.benchmark_results.generated_at),
            comparison_status: None,
            rollback_reason: None,
        });
    }

    let rollback_reason = rollback_reason(snapshot, active_canary);
    let comparison_status = candidate_comparison_status(snapshot, &active_canary.baseline_snapshot);
    if mode == OversightApplyMode::PreviewOnly {
        return Ok(OversightApplyResult {
            schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
            stage: if rollback_reason.is_none() && comparison_status.as_deref() == Some("improved")
            {
                OVERSIGHT_APPLY_STAGE_IMPROVED.to_string()
            } else {
                OVERSIGHT_APPLY_STAGE_REFUSED.to_string()
            },
            summary: "A bounded canary has finished its watch window, but the manual reconcile surface does not finalize retain or rollback decisions.".to_string(),
            refusal_reasons: rollback_reason
                .clone()
                .into_iter()
                .collect::<Vec<_>>(),
            patch_family: Some(active_canary.proposal.patch_family.clone()),
            watch_window_seconds: Some(active_canary.watch_window_seconds),
            watch_window_started_at: Some(active_canary.opened_at_ts),
            watch_window_end_at: Some(active_canary.watch_window_end_at),
            baseline_generated_at: Some(active_canary.baseline_snapshot.generated_at),
            candidate_generated_at: Some(snapshot.benchmark_results.generated_at),
            comparison_status,
            rollback_reason,
        });
    }

    if rollback_reason.is_none() && comparison_status.as_deref() == Some("improved") {
        clear_active_canary(store, site_id);
        return Ok(OversightApplyResult {
            schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
            stage: OVERSIGHT_APPLY_STAGE_IMPROVED.to_string(),
            summary: "The bounded canary improved the protected benchmark baseline and will be retained.".to_string(),
            refusal_reasons: Vec::new(),
            patch_family: Some(active_canary.proposal.patch_family.clone()),
            watch_window_seconds: Some(active_canary.watch_window_seconds),
            watch_window_started_at: Some(active_canary.opened_at_ts),
            watch_window_end_at: Some(active_canary.watch_window_end_at),
            baseline_generated_at: Some(active_canary.baseline_snapshot.generated_at),
            candidate_generated_at: Some(snapshot.benchmark_results.generated_at),
            comparison_status,
            rollback_reason: None,
        });
    }

    let Some(current_cfg) = current_cfg else {
        return Err(());
    };
    let rollback_reason = rollback_reason.unwrap_or_else(|| {
        format!(
            "candidate_comparison_{}",
            comparison_status
                .clone()
                .unwrap_or_else(|| "not_available".to_string())
        )
    });
    persist_config_change(
        store,
        site_id,
        now,
        current_cfg,
        &active_canary.previous_config,
        &active_canary.proposal.patch,
        "oversight_canary_rollback",
        "rolled_back",
        &active_canary.objective_revision,
        active_canary.watch_window_seconds,
        "Bounded oversight canary rolled back to the exact pre-canary config after the watch window failed closed.",
        vec![
            operator_evidence_reference(
                "oversight_canary_id",
                active_canary.canary_id.as_str(),
                "Active canary identifier being rolled back.",
            ),
            operator_evidence_reference(
                "rollback_reason",
                rollback_reason.as_str(),
                "Fail-closed rollback reason captured after protected watch-window evaluation.",
            ),
        ],
    )?;
    clear_active_canary(store, site_id);

    Ok(OversightApplyResult {
        schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
        stage: OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED.to_string(),
        summary: "The bounded canary was rolled back to the exact pre-canary config.".to_string(),
        refusal_reasons: Vec::new(),
        patch_family: Some(active_canary.proposal.patch_family.clone()),
        watch_window_seconds: Some(active_canary.watch_window_seconds),
        watch_window_started_at: Some(active_canary.opened_at_ts),
        watch_window_end_at: Some(active_canary.watch_window_end_at),
        baseline_generated_at: Some(active_canary.baseline_snapshot.generated_at),
        candidate_generated_at: Some(snapshot.benchmark_results.generated_at),
        comparison_status,
        rollback_reason: Some(rollback_reason),
    })
}

fn apply_refusal_reasons(
    snapshot: &OperatorSnapshotHotReadPayload,
    current_cfg: Option<&Config>,
    reconcile: &OversightReconcileResult,
    validation: &OversightPatchValidation,
    active_canary_exists: bool,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if current_cfg.is_none() {
        reasons.push("config_unavailable".to_string());
    }
    if active_canary_exists {
        reasons.push("active_canary_already_open".to_string());
    }
    if snapshot.objectives.rollout_guardrails.automated_apply_status != "canary_only" {
        reasons.push("automated_apply_manual_only".to_string());
    }
    if reconcile.outcome != "recommend_patch" || reconcile.proposal.is_none() {
        reasons.push(format!("reconcile_outcome_{}", reconcile.outcome));
    }
    if let Some(reason) = proposal_apply_refusal_reason(reconcile) {
        reasons.push(reason);
    }
    if validation.status != "valid" {
        reasons.push(format!("proposal_validation_{}", validation.status));
    }
    if snapshot.benchmark_results.tuning_eligibility.status != "eligible" {
        reasons.push("benchmark_tuning_ineligible".to_string());
        reasons.extend(snapshot.benchmark_results.tuning_eligibility.blockers.clone());
    }
    reasons
}

fn proposal_apply_refusal_reason(reconcile: &OversightReconcileResult) -> Option<String> {
    let proposal = reconcile.proposal.as_ref()?;
    (proposal.controller_status != "allowed")
        .then_some("proposal_not_controller_tunable".to_string())
}

fn rollback_reason(
    snapshot: &OperatorSnapshotHotReadPayload,
    active_canary: &OversightActiveCanaryState,
) -> Option<String> {
    let stale_reasons = stale_evidence_reasons(snapshot);
    if !stale_reasons.is_empty() {
        return Some(stale_reasons.join(","));
    }
    let contradictions = contradictory_evidence_reasons(snapshot);
    if !contradictions.is_empty() {
        return Some(contradictions.join(","));
    }
    if snapshot.benchmark_results.tuning_eligibility.status != "eligible" {
        return Some(
            snapshot
                .benchmark_results
                .tuning_eligibility
                .blockers
                .join(","),
        );
    }
    if snapshot.generated_at <= active_canary.baseline_snapshot.generated_at {
        return Some("candidate_window_not_materialized".to_string());
    }
    None
}

fn candidate_comparison_status(
    snapshot: &OperatorSnapshotHotReadPayload,
    baseline_snapshot: &BenchmarkComparableSnapshot,
) -> Option<String> {
    let mut families = snapshot.benchmark_results.families.clone();
    let (_, improvement_status) =
        apply_candidate_comparison(snapshot.generated_at, families.as_mut_slice(), Some(baseline_snapshot));
    (improvement_status != "not_available").then_some(improvement_status)
}

fn refused_result(
    summary: &str,
    refusal_reasons: Vec<String>,
    proposal: Option<&OversightPatchProposal>,
) -> OversightApplyResult {
    OversightApplyResult {
        schema_version: OVERSIGHT_APPLY_SCHEMA_VERSION.to_string(),
        stage: OVERSIGHT_APPLY_STAGE_REFUSED.to_string(),
        summary: summary.to_string(),
        refusal_reasons,
        patch_family: proposal.map(|proposal| proposal.patch_family.clone()),
        watch_window_seconds: None,
        watch_window_started_at: None,
        watch_window_end_at: None,
        baseline_generated_at: None,
        candidate_generated_at: None,
        comparison_status: None,
        rollback_reason: None,
    }
}

fn active_canary_key(site_id: &str) -> String {
    format!("{OVERSIGHT_ACTIVE_CANARY_PREFIX}:{site_id}")
}

fn load_active_canary<S: KeyValueStore>(
    store: &S,
    site_id: &str,
) -> Option<OversightActiveCanaryState> {
    let raw = store.get(&active_canary_key(site_id)).ok().flatten()?;
    let state = serde_json::from_slice::<OversightActiveCanaryState>(&raw).ok()?;
    (state.schema_version == OVERSIGHT_ACTIVE_CANARY_SCHEMA_VERSION).then_some(state)
}

fn save_active_canary<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OversightActiveCanaryState,
) -> Result<(), ()> {
    let encoded = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(&active_canary_key(site_id), &encoded)
}

fn clear_active_canary<S: KeyValueStore>(store: &S, site_id: &str) {
    let _ = store.delete(&active_canary_key(site_id));
}

fn canary_id(opened_at_ts: u64, patch_family: &str, patch: &serde_json::Value) -> String {
    let mut hasher = DefaultHasher::new();
    opened_at_ts.hash(&mut hasher);
    patch_family.hash(&mut hasher);
    patch.to_string().hash(&mut hasher);
    format!("ovrcanary-{}-{:016x}", opened_at_ts, hasher.finish())
}

fn persist_config_change<S: KeyValueStore>(
    store: &S,
    site_id: &str,
    changed_at_ts: u64,
    old_cfg: &Config,
    new_cfg: &Config,
    patch: &serde_json::Value,
    decision_kind: &str,
    decision_status: &str,
    objective_revision: &str,
    watch_window_seconds: u64,
    expected_impact_summary: &str,
    evidence_references: Vec<OperatorDecisionEvidenceReference>,
) -> Result<(), ()> {
    let encoded = crate::config::serialize_persisted_kv_config(new_cfg).map_err(|_| ())?;
    store.set(format!("config:{site_id}").as_str(), &encoded)?;

    if let Some(change_row) = operator_snapshot_config_patch_recent_change_row(
        old_cfg,
        new_cfg,
        patch,
        OVERSIGHT_CONTROLLER_ADMIN_ID,
        changed_at_ts,
    ) {
        let decision = record_operator_decision(
            store,
            site_id,
            OperatorDecisionDraft {
                recorded_at_ts: changed_at_ts,
                decision_kind: decision_kind.to_string(),
                decision_status: decision_status.to_string(),
                source: "scheduled_controller".to_string(),
                changed_families: change_row.changed_families.clone(),
                targets: change_row.targets.clone(),
                objective_revision: objective_revision.to_string(),
                watch_window_seconds,
                expected_impact_summary: expected_impact_summary.to_string(),
                evidence_references,
            },
        )
        .ok();
        let rows = match decision {
            Some(decision) => vec![operator_snapshot_recent_change_with_decision_id(
                &change_row,
                decision.decision_id.as_str(),
            )],
            None => vec![change_row],
        };
        record_operator_snapshot_recent_change_rows(store, site_id, rows.as_slice(), changed_at_ts);
    }

    crate::config::invalidate_runtime_cache(site_id);
    crate::observability::hot_read_projection::refresh_after_admin_mutation(store, site_id);
    Ok(())
}

fn operator_evidence_reference(
    kind: &str,
    reference: &str,
    note: &str,
) -> OperatorDecisionEvidenceReference {
    OperatorDecisionEvidenceReference {
        kind: kind.to_string(),
        reference: reference.to_string(),
        note: note.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::proposal_apply_refusal_reason;
    use crate::admin::oversight_patch_policy::OversightPatchProposal;
    use crate::admin::oversight_reconcile::OversightReconcileResult;
    use serde_json::json;

    fn reconcile_with_controller_status(controller_status: &str) -> OversightReconcileResult {
        OversightReconcileResult {
            schema_version: "oversight_reconcile_v1".to_string(),
            generated_at: 1_700_000_000,
            trigger_source: "test".to_string(),
            outcome: "recommend_patch".to_string(),
            summary: "test".to_string(),
            objective_revision: "rev-1".to_string(),
            benchmark_overall_status: "outside_budget".to_string(),
            improvement_status: "regressing".to_string(),
            trigger_family_ids: vec!["likely_human_friction".to_string()],
            candidate_action_families: vec!["challenge".to_string()],
            refusal_reasons: Vec::new(),
            proposal: Some(OversightPatchProposal {
                patch_family: "challenge".to_string(),
                patch: json!({ "challenge_puzzle_enabled": true }),
                expected_impact: "test".to_string(),
                confidence: "medium".to_string(),
                required_verification: Vec::new(),
                controller_status: controller_status.to_string(),
                canary_requirement: "required".to_string(),
                matched_group_ids: vec!["challenge.policy".to_string()],
                note: "test".to_string(),
            }),
            latest_sim_run_id: None,
            replay_promotion_availability: "available".to_string(),
            snapshot_generated_at: 1_700_000_000,
            evidence_references: Vec::new(),
        }
    }

    #[test]
    fn apply_refuses_non_tunable_proposals_even_if_one_is_present() {
        assert_eq!(
            proposal_apply_refusal_reason(&reconcile_with_controller_status("forbidden")),
            Some("proposal_not_controller_tunable".to_string())
        );
    }

    #[test]
    fn apply_accepts_allowed_proposals() {
        assert_eq!(
            proposal_apply_refusal_reason(&reconcile_with_controller_status("allowed")),
            None
        );
    }
}
