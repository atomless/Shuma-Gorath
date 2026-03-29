use serde::{Deserialize, Serialize};
use serde_json::json;
use spin_sdk::http::{Method, Request, Response};

use super::oversight_agent::{
    build_status_payload, execute_agent_cycle, OversightAgentTrigger,
    OversightAgentTriggerKind,
};
use super::oversight_apply::{evaluate_apply_cycle, OversightApplyMode, OversightApplyResult};
use super::oversight_decision_ledger::{
    load_recent_decisions, record_decision, OversightDecisionDraft,
    OversightDecisionEvidenceReference, OversightDecisionRecord,
};
use super::oversight_reconcile::{reconcile, OversightReconcileResult};
use crate::observability::benchmark_comparison::{
    benchmark_episode_delta_summary, classify_homeostasis, comparable_snapshot_from_results,
    unavailable_homeostasis_restart_baseline, BenchmarkCompletedCycleJudgment,
    BenchmarkHomeostasisRestartBaseline,
};
use crate::observability::decision_ledger::OperatorDecisionEvidenceReference;
use crate::observability::operator_snapshot::{
    OperatorSnapshotEpisodeArchive, OperatorSnapshotEpisodeEvaluationContext,
    OperatorSnapshotEpisodeProposal, OperatorSnapshotEpisodeRecord,
    RecursiveImprovementGameContract,
};

pub(crate) const OVERSIGHT_EXECUTION_SCHEMA_VERSION: &str = "oversight_execution_v1";
pub(crate) const OVERSIGHT_HISTORY_SCHEMA_VERSION: &str = "oversight_history_v1";
pub(crate) const OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION: &str = "oversight_episode_archive_v1";
const OVERSIGHT_EPISODE_ARCHIVE_PREFIX: &str = "oversight_episode_archive:v1";
const OVERSIGHT_EPISODE_ARCHIVE_MAX_ROWS: usize = 24;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchValidationIssue {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightPatchValidation {
    pub status: String,
    pub issues: Vec<OversightPatchValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct OversightExecutionPayload {
    pub schema_version: String,
    pub decision: OversightDecisionRecord,
    pub reconcile: OversightReconcileResult,
    pub validation: OversightPatchValidation,
    pub apply: OversightApplyResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct AdminConfigValidationResponse {
    valid: bool,
    issues: Vec<OversightPatchValidationIssue>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct InternalOversightAgentRunRequest {
    trigger_kind: Option<String>,
    sim_run_id: Option<String>,
    sim_completion_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct OversightEpisodeArchiveState {
    schema_version: String,
    updated_at_ts: u64,
    rows: Vec<OperatorSnapshotEpisodeRecord>,
}

fn episode_archive_key(site_id: &str) -> String {
    format!("{OVERSIGHT_EPISODE_ARCHIVE_PREFIX}:{site_id}")
}

fn load_episode_archive_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
) -> OversightEpisodeArchiveState {
    store
        .get(&episode_archive_key(site_id))
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

fn save_episode_archive_state<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    state: &OversightEpisodeArchiveState,
) -> Result<(), ()> {
    let payload = serde_json::to_vec(state).map_err(|_| ())?;
    store.set(&episode_archive_key(site_id), payload.as_slice())
}

pub(crate) fn load_oversight_episode_archive<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    game_contract: &RecursiveImprovementGameContract,
) -> (OperatorSnapshotEpisodeArchive, u64) {
    let state = load_episode_archive_state(store, site_id);
    let minimum_completed_cycles = game_contract
        .evaluator_scorecard
        .comparison_contract
        .minimum_completed_cycles_for_homeostasis;
    let judgments = state
        .rows
        .iter()
        .filter(|row| row.homeostasis_eligible)
        .take(minimum_completed_cycles as usize)
        .map(|row| BenchmarkCompletedCycleJudgment {
            episode_id: row.episode_id.clone(),
            judgment: row.cycle_judgment.clone(),
            urgency_status: row.benchmark_urgency_status.clone(),
            homeostasis_break_status: row.homeostasis_break_status.clone(),
            homeostasis_break_reasons: row.homeostasis_break_reasons.clone(),
            restart_baseline: row.restart_baseline.clone(),
        })
        .collect::<Vec<_>>();

    (
        OperatorSnapshotEpisodeArchive {
            schema_version: OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION.to_string(),
            homeostasis: classify_homeostasis(&judgments, minimum_completed_cycles),
            rows: state.rows,
        },
        state.updated_at_ts,
    )
}

fn record_completed_episode<S: crate::challenge::KeyValueStore>(
    store: &S,
    site_id: &str,
    record: OperatorSnapshotEpisodeRecord,
) -> Result<(), ()> {
    let mut state = load_episode_archive_state(store, site_id);
    state.rows.retain(|existing| existing.episode_id != record.episode_id);
    state.rows.push(record.clone());
    state.rows.sort_by(|left, right| {
        right
            .completed_at_ts
            .cmp(&left.completed_at_ts)
            .then_with(|| left.episode_id.cmp(&right.episode_id))
    });
    state.rows.truncate(OVERSIGHT_EPISODE_ARCHIVE_MAX_ROWS);
    state.updated_at_ts = record.completed_at_ts;
    state.schema_version = OVERSIGHT_EPISODE_ARCHIVE_SCHEMA_VERSION.to_string();
    save_episode_archive_state(store, site_id, &state)
}

pub(crate) fn execute_reconcile_cycle(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    trigger_source: &str,
) -> Result<OversightExecutionPayload, ()> {
    execute_oversight_cycle_at(
        store,
        site_id,
        trigger_source,
        OversightApplyMode::PreviewOnly,
        crate::admin::now_ts(),
    )
}

pub(crate) fn execute_oversight_cycle_at(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    trigger_source: &str,
    apply_mode: OversightApplyMode,
    now: u64,
) -> Result<OversightExecutionPayload, ()> {
    let snapshot = crate::observability::hot_read_projection::load_operator_snapshot_hot_read(
        store, site_id,
    );
    let objectives =
        crate::observability::operator_objectives_store::load_or_seed_operator_objectives(
            store, site_id, now,
        );

    let mut current_cfg: Option<crate::config::Config> = None;
    let (reconcile_result, validation) = match snapshot.as_ref() {
        Some(snapshot) => match crate::config::load_runtime_cached(store, site_id) {
            Ok(cfg) => {
                let result = reconcile(&cfg, &snapshot.payload, trigger_source);
                let validation = validate_reconcile_result(store, site_id, &result);
                current_cfg = Some(cfg);
                (result, validation)
            }
            Err(_) => (
                OversightReconcileResult {
                    schema_version: crate::admin::oversight_reconcile::OVERSIGHT_RECONCILE_SCHEMA_VERSION
                        .to_string(),
                    generated_at: now,
                    trigger_source: trigger_source.to_string(),
                    outcome: "insufficient_evidence".to_string(),
                    summary: "Runtime config is unavailable, so recommend-only reconcile must fail closed.".to_string(),
                    objective_revision: snapshot.payload.objectives.revision.clone(),
                    benchmark_overall_status: snapshot.payload.benchmark_results.overall_status.clone(),
                    improvement_status: snapshot.payload.benchmark_results.improvement_status.clone(),
                    problem_class: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .restriction_diagnosis
                        .problem_class
                        .clone(),
                    guidance_status: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .move_selection
                        .guidance_status
                        .clone(),
                    tractability: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .move_selection
                        .tractability
                        .clone(),
                    trigger_family_ids: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .move_selection
                        .trigger_family_ids
                        .clone(),
                    candidate_action_families: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .move_selection
                        .candidate_action_families
                        .clone(),
                    refusal_reasons: vec!["config_unavailable".to_string()],
                    proposal: None,
                    latest_sim_run_id: crate::admin::oversight_reconcile::latest_recent_sim_run_id(
                        &snapshot.payload,
                    ),
                    replay_promotion_availability: snapshot.payload.replay_promotion.availability.clone(),
                    snapshot_generated_at: snapshot.payload.generated_at,
                    judge: crate::admin::oversight_reconcile::OversightJudgeState {
                        overall_status: snapshot.payload.benchmark_results.overall_status.clone(),
                        improvement_status: snapshot
                            .payload
                            .benchmark_results
                            .improvement_status
                            .clone(),
                        urgency_status: snapshot.payload.benchmark_results.urgency.status.clone(),
                        evidence_quality_status: snapshot
                            .payload
                            .benchmark_results
                            .escalation_hint
                            .evidence_quality
                            .status
                            .clone(),
                        note: "Judge state copied from materialized benchmark results.".to_string(),
                    },
                    diagnosis: crate::admin::oversight_reconcile::OversightDiagnosis {
                        status: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .status
                            .clone(),
                        problem_class: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .problem_class
                            .clone(),
                        confidence: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .confidence
                            .clone(),
                        distributed_failure_status: "not_localized".to_string(),
                        repair_surface_status: "not_available".to_string(),
                        repair_surface_candidates: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .repair_surface_candidates
                            .clone(),
                        breach_loci: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .breach_loci
                            .clone(),
                        note: "Diagnosis remained aggregate because runtime config was unavailable."
                            .to_string(),
                    },
                    recognition_evaluation: snapshot
                        .payload
                        .benchmark_results
                        .controller_contract
                        .recognition_evaluation
                        .clone(),
                    move_selection: crate::admin::oversight_reconcile::OversightMoveSelection {
                        status: "blocked_by_guardrail".to_string(),
                        selected_family: None,
                        selected_breach_locus_ids: snapshot
                            .payload
                            .benchmark_results
                            .controller_contract
                            .restriction_diagnosis
                            .breach_loci
                            .iter()
                            .map(|locus| locus.locus_id.clone())
                            .collect(),
                        bounded_repair_surface: None,
                        ranked_candidates: Vec::new(),
                        config_ring_status: "not_evaluated".to_string(),
                        code_evolution_status: "not_required".to_string(),
                        note: "Move selection was skipped because runtime config could not be loaded."
                            .to_string(),
                    },
                    evidence_references: vec![
                        crate::admin::oversight_reconcile::OversightEvidenceReference {
                            kind: "operator_snapshot".to_string(),
                            reference: format!("generated_at:{}", snapshot.payload.generated_at),
                            note: "Operator snapshot was materialized, but runtime config could not be loaded to shape a truthful proposal.".to_string(),
                        },
                    ],
                },
                OversightPatchValidation {
                    status: "skipped".to_string(),
                    issues: Vec::new(),
                },
            ),
        },
        None => (
            OversightReconcileResult {
                schema_version: crate::admin::oversight_reconcile::OVERSIGHT_RECONCILE_SCHEMA_VERSION
                    .to_string(),
                generated_at: now,
                trigger_source: trigger_source.to_string(),
                outcome: "insufficient_evidence".to_string(),
                summary: "Operator snapshot is not materialized yet, so recommend-only reconcile must fail closed.".to_string(),
                objective_revision: objectives.revision.clone(),
                benchmark_overall_status: "not_available".to_string(),
                improvement_status: "not_available".to_string(),
                problem_class: "not_available".to_string(),
                guidance_status: "not_actionable_yet".to_string(),
                tractability: "insufficient_evidence".to_string(),
                trigger_family_ids: Vec::new(),
                candidate_action_families: Vec::new(),
                refusal_reasons: vec!["operator_snapshot_not_materialized".to_string()],
                proposal: None,
                latest_sim_run_id: None,
                replay_promotion_availability: "not_materialized".to_string(),
                snapshot_generated_at: 0,
                judge: crate::admin::oversight_reconcile::OversightJudgeState {
                    overall_status: "not_available".to_string(),
                    improvement_status: "not_available".to_string(),
                    urgency_status: "not_available".to_string(),
                    evidence_quality_status: "not_available".to_string(),
                    note: "Judge state is unavailable because the operator snapshot is not materialized."
                        .to_string(),
                },
                diagnosis: crate::admin::oversight_reconcile::OversightDiagnosis {
                    status: "aggregate_only".to_string(),
                    problem_class: "not_available".to_string(),
                    confidence: "not_available".to_string(),
                    distributed_failure_status: "not_localized".to_string(),
                    repair_surface_status: "not_available".to_string(),
                    repair_surface_candidates: Vec::new(),
                    breach_loci: Vec::new(),
                    note: "Diagnosis is unavailable because the operator snapshot is not materialized."
                        .to_string(),
                },
                recognition_evaluation:
                    crate::observability::benchmark_results::unavailable_benchmark_controller_contract()
                        .recognition_evaluation,
                move_selection: crate::admin::oversight_reconcile::OversightMoveSelection {
                    status: "not_selected".to_string(),
                    selected_family: None,
                    selected_breach_locus_ids: Vec::new(),
                    bounded_repair_surface: None,
                    ranked_candidates: Vec::new(),
                    config_ring_status: "not_evaluated".to_string(),
                    code_evolution_status: "not_required".to_string(),
                    note: "Move selection is unavailable because the operator snapshot is not materialized."
                        .to_string(),
                },
                evidence_references: vec![crate::admin::oversight_reconcile::OversightEvidenceReference {
                    kind: "operator_snapshot".to_string(),
                    reference: "not_materialized".to_string(),
                    note: "Hot-read snapshot must exist before reconcile can safely reason about current budgets.".to_string(),
                }],
            },
            OversightPatchValidation {
                status: "skipped".to_string(),
                issues: Vec::new(),
            },
        ),
    };

    let recorded_result = apply_validation_to_result(reconcile_result, &validation);
    let active_canary_episode_context =
        super::oversight_apply::load_active_canary_episode_context(store, site_id);
    let apply = evaluate_apply_cycle(
        store,
        site_id,
        now,
        snapshot.as_ref().map(|snapshot| &snapshot.payload),
        current_cfg.as_ref(),
        &recorded_result,
        &validation,
        apply_mode,
    )?;
    let decision_outcome = if apply.stage == super::oversight_apply::OVERSIGHT_APPLY_STAGE_REFUSED
        && recorded_result.outcome != "recommend_patch"
    {
        recorded_result.outcome.clone()
    } else {
        apply.stage.clone()
    };
    let decision_summary = if apply.stage == super::oversight_apply::OVERSIGHT_APPLY_STAGE_REFUSED
        && recorded_result.outcome != "recommend_patch"
    {
        recorded_result.summary.clone()
    } else {
        apply.summary.clone()
    };
    let decision = record_decision(
        store,
        site_id,
        OversightDecisionDraft {
            recorded_at_ts: now,
            trigger_source: trigger_source.to_string(),
            outcome: decision_outcome,
            summary: decision_summary,
            objective_revision: recorded_result.objective_revision.clone(),
            snapshot_generated_at: recorded_result.snapshot_generated_at,
            benchmark_overall_status: recorded_result.benchmark_overall_status.clone(),
            improvement_status: recorded_result.improvement_status.clone(),
            replay_promotion_availability: recorded_result.replay_promotion_availability.clone(),
            trigger_family_ids: recorded_result.trigger_family_ids.clone(),
            candidate_action_families: recorded_result.candidate_action_families.clone(),
            refusal_reasons: recorded_result.refusal_reasons.clone(),
            proposal: recorded_result.proposal.clone(),
            validation_status: validation.status.clone(),
            validation_issues: validation
                .issues
                .iter()
                .map(|issue| issue.message.clone())
                .collect(),
            apply: Some(apply.clone()),
            latest_sim_run_id: recorded_result.latest_sim_run_id.clone(),
            evidence_references: recorded_result
                .evidence_references
                .iter()
                .map(|reference| OversightDecisionEvidenceReference {
                    kind: reference.kind.clone(),
                    reference: reference.reference.clone(),
                    note: reference.note.clone(),
                })
            .collect(),
        },
    )?;
    if let Some(episode_record) = completed_episode_record(
        &objectives,
        snapshot.as_ref().map(|snapshot| &snapshot.payload),
        &decision,
        &recorded_result,
        &apply,
        active_canary_episode_context.as_ref(),
    ) {
        record_completed_episode(store, site_id, episode_record)?;
    }

    Ok(OversightExecutionPayload {
        schema_version: OVERSIGHT_EXECUTION_SCHEMA_VERSION.to_string(),
        decision,
        reconcile: recorded_result,
        validation,
        apply,
    })
}

pub(crate) fn handle_admin_oversight_reconcile(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    match execute_reconcile_cycle(store, site_id, "manual_admin") {
        Ok(payload) => {
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        Err(()) => Response::new(500, "Key-value store error"),
    }
}

pub(crate) fn handle_admin_oversight_history(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let now = crate::admin::now_ts();
    let objectives =
        crate::observability::operator_objectives_store::load_operator_objectives(store, site_id)
            .unwrap_or_else(|| {
                crate::observability::operator_snapshot_objectives::default_operator_objectives(
                    now,
                )
            });
    let game_contract =
        crate::observability::operator_snapshot_objectives::recursive_improvement_game_contract_v1(
            &objectives,
            &crate::config::controller_legal_move_ring_v1(),
        );
    let (episode_archive, _) = load_oversight_episode_archive(store, site_id, &game_contract);
    let body = serde_json::to_string(&json!({
        "schema_version": OVERSIGHT_HISTORY_SCHEMA_VERSION,
        "game_contract": game_contract,
        "episode_archive": episode_archive,
        "rows": load_recent_decisions(store, site_id),
    }))
    .unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_admin_oversight_agent_status(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Get {
        return Response::new(405, "Method Not Allowed");
    }
    let payload = build_status_payload(store, site_id);
    let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Cache-Control", "no-store")
        .body(body)
        .build()
}

pub(crate) fn handle_internal_oversight_agent_run(
    req: &Request,
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
) -> Response {
    if *req.method() != Method::Post {
        return Response::new(405, "Method Not Allowed");
    }
    if !crate::admin::oversight_agent::shared_host_execution_available() {
        return Response::new(404, "Not Found");
    }
    if !crate::admin::auth::is_internal_oversight_supervisor_request(req) {
        return Response::new(
            401,
            "Unauthorized: Internal oversight supervisor authorization required",
        );
    }

    let parsed = if req.body().is_empty() {
        InternalOversightAgentRunRequest::default()
    } else {
        match serde_json::from_slice::<InternalOversightAgentRunRequest>(req.body()) {
            Ok(parsed) => parsed,
            Err(_) => return Response::new(400, "Invalid oversight agent trigger payload"),
        }
    };
    let trigger_kind = parsed
        .trigger_kind
        .as_deref()
        .and_then(OversightAgentTriggerKind::parse)
        .unwrap_or(OversightAgentTriggerKind::PeriodicSupervisor);
    let trigger = OversightAgentTrigger {
        kind: trigger_kind,
        requested_at_ts: crate::admin::now_ts(),
        sim_run_id: parsed.sim_run_id,
        sim_completion_reason: parsed.sim_completion_reason,
    };
    match execute_agent_cycle(store, site_id, trigger) {
        Ok(payload) => {
            let body = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
            Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("Cache-Control", "no-store")
                .body(body)
                .build()
        }
        Err(()) => Response::new(500, "Key-value store error"),
    }
}

fn validate_reconcile_result(
    store: &impl crate::challenge::KeyValueStore,
    site_id: &str,
    result: &OversightReconcileResult,
) -> OversightPatchValidation {
    let Some(proposal) = result.proposal.as_ref() else {
        return OversightPatchValidation {
            status: "skipped".to_string(),
            issues: Vec::new(),
        };
    };
    let body = serde_json::to_vec(&proposal.patch).unwrap_or_default();
    let request = Request::builder()
        .method(Method::Post)
        .uri("/admin/config/validate")
        .body(body)
        .build();
    let response = super::config_api::handle_admin_config_validate(&request, store, site_id);
    if *response.status() != 200 {
        return OversightPatchValidation {
            status: "validation_error".to_string(),
            issues: vec![OversightPatchValidationIssue {
                field: None,
                message: String::from_utf8_lossy(response.body()).to_string(),
                expected: None,
                received: None,
            }],
        };
    }
    let payload = serde_json::from_slice::<AdminConfigValidationResponse>(response.body())
        .unwrap_or(AdminConfigValidationResponse {
            valid: false,
            issues: vec![OversightPatchValidationIssue {
                field: None,
                message: "Unable to parse config validation response.".to_string(),
                expected: None,
                received: None,
            }],
        });
    OversightPatchValidation {
        status: if payload.valid {
            "valid".to_string()
        } else {
            "invalid".to_string()
        },
        issues: payload.issues,
    }
}

fn apply_validation_to_result(
    mut result: OversightReconcileResult,
    validation: &OversightPatchValidation,
) -> OversightReconcileResult {
    if result.proposal.is_some() && validation.status == "invalid" {
        result.outcome = "refuse_invalid_patch".to_string();
        result.summary =
            "Reconcile produced a bounded candidate patch, but the existing config validator rejected it."
                .to_string();
        result.refusal_reasons = validation
            .issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect();
    } else if result.proposal.is_some() && validation.status == "validation_error" {
        result.outcome = "insufficient_evidence".to_string();
        result.summary =
            "Config validation failed unexpectedly, so recommend-only reconcile failed closed."
                .to_string();
        result.refusal_reasons = validation
            .issues
            .iter()
            .map(|issue| issue.message.clone())
            .collect();
    }
    result
}

fn completed_episode_record(
    objectives: &crate::observability::operator_snapshot_objectives::OperatorObjectivesProfile,
    snapshot: Option<&crate::observability::operator_snapshot::OperatorSnapshotHotReadPayload>,
    decision: &OversightDecisionRecord,
    reconcile: &OversightReconcileResult,
    apply: &OversightApplyResult,
    active_canary_episode_context: Option<
        &crate::admin::oversight_apply::OversightActiveCanaryEpisodeContext,
    >,
) -> Option<OperatorSnapshotEpisodeRecord> {
    let snapshot = snapshot?;
    if !matches!(
        apply.stage.as_str(),
        super::oversight_apply::OVERSIGHT_APPLY_STAGE_REFUSED
            | super::oversight_apply::OVERSIGHT_APPLY_STAGE_IMPROVED
            | super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED
    ) {
        return None;
    }

    let baseline_scorecard = match apply.stage.as_str() {
        super::oversight_apply::OVERSIGHT_APPLY_STAGE_IMPROVED
        | super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED => active_canary_episode_context
            .map(|context| context.baseline_snapshot.clone())
            .unwrap_or_else(|| comparable_snapshot_from_results(&snapshot.benchmark_results)),
        _ => comparable_snapshot_from_results(&snapshot.benchmark_results),
    };
    let episode_proposal = match apply.stage.as_str() {
        super::oversight_apply::OVERSIGHT_APPLY_STAGE_IMPROVED
        | super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED => active_canary_episode_context
            .map(|context| &context.proposal)
            .or(reconcile.proposal.as_ref()),
        _ => reconcile.proposal.as_ref(),
    };
    let proposal = episode_proposal.map(project_episode_proposal);
    let proposal_id =
        episode_proposal.map(|proposal| proposal_id(snapshot.generated_at, proposal));
    let (proposal_status, watch_window_result, retain_or_rollback, cycle_judgment, homeostasis_eligible) =
        match apply.stage.as_str() {
            super::oversight_apply::OVERSIGHT_APPLY_STAGE_IMPROVED => (
                "accepted",
                "improved",
                "retained",
                "improved",
                true,
            ),
            super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED => (
                "accepted",
                "rollback_applied",
                "rolled_back",
                rollback_cycle_judgment(apply),
                true,
            ),
            _ => (
                "refused",
                "not_opened",
                "not_applied",
                "guardrail_blocked",
                false,
            ),
        };
    let (
        benchmark_urgency_status,
        homeostasis_break_status,
        homeostasis_break_reasons,
        restart_baseline,
    ) = completed_episode_homeostasis_state(snapshot, apply, active_canary_episode_context);
    let judged_lane_ids = active_canary_episode_context
        .map(|context| {
            context
                .required_runs
                .iter()
                .map(|run| run.lane.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let judged_run_ids = active_canary_episode_context
        .map(|context| {
            context
                .required_runs
                .iter()
                .filter_map(|run| run.follow_on_run_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(OperatorSnapshotEpisodeRecord {
        episode_id: decision.decision_id.clone(),
        proposal_id,
        completed_at_ts: decision.recorded_at_ts,
        trigger_source: decision.trigger_source.clone(),
        evaluation_context: OperatorSnapshotEpisodeEvaluationContext {
            objective_revision: objectives.revision.clone(),
            profile_id: objectives.profile_id.clone(),
            subject_kind: snapshot.benchmark_results.subject_kind.clone(),
            comparison_mode: objectives
                .adversary_sim_expectations
                .comparison_mode
                .clone(),
        },
        baseline_scorecard,
        proposal,
        proposal_status: proposal_status.to_string(),
        watch_window_result: watch_window_result.to_string(),
        retain_or_rollback: retain_or_rollback.to_string(),
        judged_lane_ids,
        judged_run_ids,
        benchmark_deltas: benchmark_episode_delta_summary(&snapshot.benchmark_results),
        hard_guardrail_triggers: completed_episode_guardrail_triggers(snapshot, reconcile, apply),
        cycle_judgment: cycle_judgment.to_string(),
        homeostasis_eligible,
        benchmark_urgency_status,
        homeostasis_break_status,
        homeostasis_break_reasons,
        restart_baseline,
        evidence_references: decision
            .evidence_references
            .iter()
            .map(project_operator_evidence_reference)
            .collect(),
    })
}

fn project_episode_proposal(
    proposal: &crate::admin::oversight_patch_policy::OversightPatchProposal,
) -> OperatorSnapshotEpisodeProposal {
    OperatorSnapshotEpisodeProposal {
        patch_family: proposal.patch_family.clone(),
        patch: proposal.patch.clone(),
        expected_impact: proposal.expected_impact.clone(),
        confidence: proposal.confidence.clone(),
        controller_status: proposal.controller_status.clone(),
        canary_requirement: proposal.canary_requirement.clone(),
        matched_group_ids: proposal.matched_group_ids.clone(),
        note: proposal.note.clone(),
    }
}

fn project_operator_evidence_reference(
    reference: &OversightDecisionEvidenceReference,
) -> OperatorDecisionEvidenceReference {
    OperatorDecisionEvidenceReference {
        kind: reference.kind.clone(),
        reference: reference.reference.clone(),
        note: reference.note.clone(),
    }
}

fn proposal_id(
    snapshot_generated_at: u64,
    proposal: &crate::admin::oversight_patch_policy::OversightPatchProposal,
) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    snapshot_generated_at.hash(&mut hasher);
    proposal.patch_family.hash(&mut hasher);
    proposal.patch.to_string().hash(&mut hasher);
    format!("ovrproposal-{snapshot_generated_at}-{:016x}", hasher.finish())
}

fn rollback_cycle_judgment(apply: &OversightApplyResult) -> &'static str {
    match apply.comparison_status.as_deref() {
        Some("regressed") => "regressed",
        Some("neutral") | Some("not_available") => "flat",
        _ => "guardrail_blocked",
    }
}

fn completed_episode_homeostasis_state(
    snapshot: &crate::observability::operator_snapshot::OperatorSnapshotHotReadPayload,
    apply: &OversightApplyResult,
    active_canary_episode_context: Option<
        &crate::admin::oversight_apply::OversightActiveCanaryEpisodeContext,
    >,
) -> (
    String,
    String,
    Vec<String>,
    BenchmarkHomeostasisRestartBaseline,
) {
    let mut break_reasons = snapshot
        .benchmark_results
        .urgency
        .homeostasis_break_reasons
        .clone();
    if apply.stage == super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED
        && apply.comparison_status.as_deref() == Some("regressed")
        && !break_reasons.contains(&"candidate_baseline_regressed".to_string())
    {
        break_reasons.push("candidate_baseline_regressed".to_string());
    }
    let break_status = if break_reasons.is_empty() {
        "not_triggered".to_string()
    } else {
        "triggered".to_string()
    };
    let urgency_status = if break_status == "triggered"
        && snapshot.benchmark_results.urgency.status == "steady"
    {
        "critical".to_string()
    } else {
        snapshot.benchmark_results.urgency.status.clone()
    };
    let restart_baseline = match apply.stage.as_str() {
        super::oversight_apply::OVERSIGHT_APPLY_STAGE_IMPROVED => BenchmarkHomeostasisRestartBaseline {
            status: "available".to_string(),
            generated_at: Some(snapshot.benchmark_results.generated_at),
            subject_kind: Some(snapshot.benchmark_results.subject_kind.clone()),
            source: "retained_candidate".to_string(),
            note: "Latest retained candidate snapshot becomes the safe restart baseline.".to_string(),
        },
        super::oversight_apply::OVERSIGHT_APPLY_STAGE_ROLLBACK_APPLIED => active_canary_episode_context
            .map(|context| BenchmarkHomeostasisRestartBaseline {
                status: "available".to_string(),
                generated_at: Some(context.baseline_snapshot.generated_at),
                subject_kind: Some(context.baseline_snapshot.subject_kind.clone()),
                source: "pre_canary_baseline".to_string(),
                note: "Rollback re-entered the exact pre-canary safe baseline.".to_string(),
            })
            .unwrap_or_else(|| {
                unavailable_homeostasis_restart_baseline(
                    "Rollback completed without an active-canary baseline context.",
                )
            }),
        _ => unavailable_homeostasis_restart_baseline(
            "No accepted or re-entered safe baseline is recorded for this episode.",
        ),
    };

    (
        urgency_status,
        break_status,
        break_reasons,
        restart_baseline,
    )
}

fn completed_episode_guardrail_triggers(
    snapshot: &crate::observability::operator_snapshot::OperatorSnapshotHotReadPayload,
    reconcile: &OversightReconcileResult,
    apply: &OversightApplyResult,
) -> Vec<String> {
    let mut triggers = Vec::new();
    if snapshot.benchmark_results.tuning_eligibility.status != "eligible" {
        triggers.extend(snapshot.benchmark_results.tuning_eligibility.blockers.clone());
    }
    triggers.extend(reconcile.refusal_reasons.clone());
    triggers.extend(apply.refusal_reasons.clone());
    if let Some(reason) = apply.rollback_reason.as_ref() {
        if !matches!(
            reason.as_str(),
            "candidate_comparison_regressed"
                | "candidate_comparison_neutral"
                | "candidate_comparison_not_available"
        ) {
            triggers.push(reason.clone());
        }
    }
    triggers.sort();
    triggers.dedup();
    triggers
}

#[cfg(test)]
mod tests {
    use super::{
        execute_oversight_cycle_at, execute_reconcile_cycle, handle_admin_oversight_agent_status,
        handle_admin_oversight_history, handle_admin_oversight_reconcile,
        handle_internal_oversight_agent_run,
    };
    use crate::challenge::KeyValueStore;
    use crate::config::{defaults, serialize_persisted_kv_config};
    use crate::observability::hot_read_documents::{
        operator_snapshot_document_contract, operator_snapshot_document_key, HotReadDocumentEnvelope,
        HotReadDocumentMetadata, HotReadUpdateTrigger,
    };
    use crate::observability::monitoring::{record_request_outcome, summarize_with_store};
    use crate::observability::operator_snapshot::{
        build_operator_snapshot_payload, OperatorSnapshotRecentChanges,
    };
    use crate::runtime::effect_intents::ExecutionMode;
    use crate::runtime::request_outcome::{
        RenderedRequestOutcome, RequestOutcomeClass, RequestOutcomeLane, ResponseKind,
        TrafficOrigin,
    };
    use crate::runtime::traffic_classification::{
        MeasurementScope, PolicySource, RouteActionFamily, TrafficLane,
    };
    use spin_sdk::http::{Method, Request};
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

    fn seed_snapshot(store: &TestStore, cfg: crate::config::Config) {
        store
            .set(
                "config:default",
                &serialize_persisted_kv_config(&cfg).expect("cfg serializes"),
            )
            .expect("config seed");
        record_request_outcome(
            store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::SuspiciousAutomation,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 2_000,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphSecondTranche,
            },
        );
        record_request_outcome(
            store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::Live,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::VerifiedBot,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 512,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::PolicyGraphVerifiedIdentityTranche,
            },
        );
        record_request_outcome(
            store,
            &RenderedRequestOutcome {
                traffic_origin: TrafficOrigin::AdversarySim,
                measurement_scope: MeasurementScope::IngressPrimary,
                route_action_family: RouteActionFamily::PublicContent,
                execution_mode: ExecutionMode::Enforced,
                traffic_lane: Some(RequestOutcomeLane {
                    lane: TrafficLane::DeclaredCrawler,
                    exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                    basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
                }),
                non_human_category: None,
                outcome_class: RequestOutcomeClass::Forwarded,
                response_kind: ResponseKind::ForwardAllow,
                http_status: 200,
                response_bytes: 512,
                forwarded_upstream_latency_ms: None,
                forward_attempted: true,
                forward_failure_class: None,
                intended_action: None,
                policy_source: PolicySource::CleanAllow,
            },
        );
        let summary = summarize_with_store(store, 24, 10);
        let payload = build_operator_snapshot_payload(
            store,
            "default",
            1_700_000_200,
            &summary,
            &[],
            OperatorSnapshotRecentChanges::default(),
            1_700_000_200,
            1_700_000_200,
            1_700_000_200,
        );
        let document = HotReadDocumentEnvelope {
            metadata: HotReadDocumentMetadata {
                schema_version: operator_snapshot_document_contract()
                    .schema_version
                    .to_string(),
                site_id: "default".to_string(),
                generated_at_ts: 1_700_000_200,
                trigger: HotReadUpdateTrigger::RepairRebuild,
            },
            payload,
        };
        store
            .set(
                operator_snapshot_document_key("default").as_str(),
                &serde_json::to_vec(&document).expect("document serializes"),
            )
            .expect("snapshot seed");
    }

    fn seed_canary_only_objectives(store: &TestStore) {
        let mut profile =
            crate::observability::operator_snapshot_objectives::default_operator_objectives(
                1_700_000_100,
            );
        profile.window_hours = 1;
        profile.rollout_guardrails.automated_apply_status = "canary_only".to_string();
        crate::observability::operator_objectives_store::save_operator_objectives(
            store,
            "default",
            &profile,
        )
        .expect("objectives save");
    }

    fn mark_active_canary_candidate_window_materialized(
        store: &TestStore,
        materialized_at_ts: u64,
    ) {
        let mut active_canary: serde_json::Value = serde_json::from_slice(
            &store
                .get("oversight_active_canary:v1:default")
                .expect("active canary lookup")
                .expect("active canary present"),
        )
        .expect("active canary decodes");
        active_canary["candidate_window"]["status"] = serde_json::json!("materialized");
        active_canary["candidate_window"]["required_runs"] = serde_json::json!([
            {
                "lane": "scrapling_traffic",
                "status": "materialized",
                "requested_at_ts": materialized_at_ts.saturating_sub(2),
                "requested_duration_seconds": 30u64,
                "follow_on_run_id": "simrun-candidate-window-001",
                "follow_on_started_at": materialized_at_ts.saturating_sub(1),
                "materialized_at_ts": materialized_at_ts
            }
        ]);
        store
            .set(
                "oversight_active_canary:v1:default",
                &serde_json::to_vec(&active_canary).expect("active canary encodes"),
            )
            .expect("active canary save");
    }

    #[test]
    fn reconcile_cycle_records_insufficient_evidence_when_snapshot_missing() {
        let store = TestStore::new();

        let payload = execute_reconcile_cycle(&store, "default", "manual_admin")
            .expect("cycle succeeds");

        assert_eq!(payload.reconcile.outcome, "insufficient_evidence");
        assert_eq!(payload.validation.status, "skipped");
    }

    #[test]
    fn manual_reconcile_route_records_observe_longer_when_classification_is_not_ready() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let request = Request::builder()
            .method(Method::Post)
            .uri("/admin/oversight/reconcile")
            .body(Vec::new())
            .build();
        let response = handle_admin_oversight_reconcile(&request, &store, "default");

        assert_eq!(*response.status(), 200);
        let payload: serde_json::Value =
            serde_json::from_slice(response.body()).expect("payload decodes");
        assert_eq!(payload["reconcile"]["outcome"], "observe_longer");
        assert_eq!(payload["validation"]["status"], "skipped");

        let history_request = Request::builder()
            .method(Method::Get)
            .uri("/admin/oversight/history")
            .body(Vec::new())
            .build();
        let history_response = handle_admin_oversight_history(&history_request, &store, "default");
        let history_payload: serde_json::Value =
            serde_json::from_slice(history_response.body()).expect("history decodes");
        assert_eq!(history_payload["schema_version"], "oversight_history_v1");
        assert_eq!(history_payload["rows"].as_array().expect("rows array").len(), 1);
        assert_eq!(
            history_payload["game_contract"]["schema_version"],
            "recursive_improvement_game_contract_v1"
        );
        assert_eq!(
            history_payload["game_contract"]["legal_move_ring"]["legal_ring"],
            "controller_tunable"
        );
        assert_eq!(
            history_payload["episode_archive"]["schema_version"],
            "oversight_episode_archive_v1"
        );
        assert_eq!(
            history_payload["episode_archive"]["rows"][0]["proposal_status"],
            "refused"
        );
        assert_eq!(
            history_payload["episode_archive"]["rows"][0]["watch_window_result"],
            "not_opened"
        );
        assert_eq!(
            history_payload["episode_archive"]["rows"][0]["retain_or_rollback"],
            "not_applied"
        );
        assert_eq!(
            history_payload["episode_archive"]["homeostasis"]["status"],
            "not_enough_completed_cycles"
        );
    }

    #[test]
    fn reconcile_cycle_fails_closed_when_runtime_config_is_unavailable() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);
        store.delete("config:default").expect("config delete");

        let payload = execute_reconcile_cycle(&store, "default", "manual_admin")
            .expect("cycle succeeds");

        assert_eq!(payload.reconcile.outcome, "insufficient_evidence");
        assert!(payload
            .reconcile
            .refusal_reasons
            .contains(&"config_unavailable".to_string()));
        assert_eq!(payload.validation.status, "skipped");
    }

    #[test]
    fn internal_agent_route_records_periodic_run_and_fail_closed_status_surface() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "oversight-agent-test-key");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_snapshot(&store, cfg);

        let internal_request = Request::builder()
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer oversight-agent-test-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "oversight-agent")
            .body(
                serde_json::to_vec(&serde_json::json!({
                    "trigger_kind": "periodic_supervisor"
                }))
                .expect("json body"),
            )
            .build();
        let response = handle_internal_oversight_agent_run(&internal_request, &store, "default");
        assert_eq!(*response.status(), 200);

        let status_request = Request::builder()
            .method(Method::Get)
            .uri("/admin/oversight/agent/status")
            .body(Vec::new())
            .build();
        let status_response =
            handle_admin_oversight_agent_status(&status_request, &store, "default");
        assert_eq!(*status_response.status(), 200);
        let payload: serde_json::Value =
            serde_json::from_slice(status_response.body()).expect("status decodes");
        assert_eq!(payload["latest_run"]["trigger_kind"], "periodic_supervisor");
        assert_eq!(
            payload["latest_run"]["execution"]["reconcile"]["outcome"],
            "observe_longer"
        );

        std::env::remove_var("SHUMA_API_KEY");
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
    }

    #[test]
    fn internal_agent_route_is_unavailable_on_edge_profile() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_API_KEY", "oversight-agent-test-key");
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");
        std::env::set_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE", "edge-fermyon");
        let store = TestStore::new();

        let internal_request = Request::builder()
            .method(Method::Post)
            .uri("/internal/oversight/agent/run")
            .header("host", "localhost:3000")
            .header("authorization", "Bearer oversight-agent-test-key")
            .header("x-shuma-forwarded-secret", "test-forwarded-secret")
            .header("x-forwarded-proto", "https")
            .header("x-forwarded-for", "127.0.0.1")
            .header("x-shuma-internal-supervisor", "oversight-agent")
            .body(Vec::new())
            .build();
        let response = handle_internal_oversight_agent_run(&internal_request, &store, "default");
        assert_eq!(*response.status(), 404);

        std::env::remove_var("SHUMA_API_KEY");
        std::env::remove_var("SHUMA_FORWARDED_IP_SECRET");
        std::env::remove_var("SHUMA_GATEWAY_DEPLOYMENT_PROFILE");
    }

    #[test]
    fn manual_reconcile_route_exposes_apply_eligibility_without_mutating_config() {
        let store = TestStore::new();
        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        seed_canary_only_objectives(&store);
        crate::test_support::seed_apply_ready_snapshot(&store, cfg);
        let original_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("seeded config");

        let request = Request::builder()
            .method(Method::Post)
            .uri("/admin/oversight/reconcile")
            .body(Vec::new())
            .build();
        let response = handle_admin_oversight_reconcile(&request, &store, "default");

        assert_eq!(*response.status(), 200);
        let payload: serde_json::Value =
            serde_json::from_slice(response.body()).expect("payload decodes");
        assert_eq!(payload["reconcile"]["outcome"], "recommend_patch");
        assert_eq!(payload["apply"]["stage"], "eligible");

        let persisted_config = store
            .get("config:default")
            .expect("config lookup")
            .expect("persisted config");
        assert_eq!(persisted_config, original_config);
    }

    #[test]
    fn execute_oversight_cycle_at_records_ten_retained_improving_cycles_toward_strict_zero_leakage() {
        let store = TestStore::new();
        seed_canary_only_objectives(&store);

        let mut cfg = defaults().clone();
        cfg.fingerprint_signal_enabled = false;
        cfg.cdp_detection_enabled = false;
        cfg.pow_enabled = false;
        cfg.challenge_puzzle_enabled = false;
        cfg.not_a_bot_enabled = false;
        cfg.not_a_bot_risk_threshold = 1;
        cfg.maze_enabled = false;
        cfg.maze_auto_ban = false;
        cfg.js_required_enforced = false;
        crate::test_support::seed_candidate_snapshot_with_candidate_families(
            &store,
            cfg,
            1_700_000_200,
            0.42,
            "outside_budget",
            &["fingerprint_signal"],
        );

        let cycle_plan = [
            ("fingerprint_signal", "cdp_detection", 1_700_001_000, 0.34, "outside_budget"),
            ("cdp_detection", "proof_of_work", 1_700_002_000, 0.28, "outside_budget"),
            ("proof_of_work", "proof_of_work", 1_700_003_000, 0.22, "outside_budget"),
            ("proof_of_work", "challenge", 1_700_004_000, 0.18, "outside_budget"),
            ("challenge", "challenge", 1_700_005_000, 0.14, "outside_budget"),
            ("challenge", "not_a_bot", 1_700_006_000, 0.10, "outside_budget"),
            ("not_a_bot", "maze_core", 1_700_007_000, 0.07, "outside_budget"),
            ("maze_core", "maze_core", 1_700_008_000, 0.05, "outside_budget"),
            ("maze_core", "core_policy", 1_700_009_000, 0.03, "outside_budget"),
            ("core_policy", "core_policy", 1_700_010_000, 0.0, "inside_budget"),
        ];

        let start_ts = 1_700_100_000u64;
        for (index, (expected_family, next_family, generated_at, retained_rate, status)) in
            cycle_plan.iter().enumerate()
        {
            let post_sim = execute_oversight_cycle_at(
                &store,
                "default",
                "post_adversary_sim",
                crate::admin::oversight_apply::OversightApplyMode::ExecuteCanary,
                start_ts + (index as u64 * 10_000),
            )
            .expect("post-sim cycle succeeds");
            assert_eq!(post_sim.apply.stage, "canary_applied");
            assert_eq!(post_sim.apply.patch_family.as_deref(), Some(*expected_family));

            let canary_cfg =
                crate::config::Config::load(&store, "default").expect("canary config loads");
            mark_active_canary_candidate_window_materialized(&store, *generated_at);
            crate::test_support::seed_candidate_snapshot_with_candidate_families(
                &store,
                canary_cfg,
                *generated_at,
                *retained_rate,
                status,
                &[*next_family],
            );

            let periodic = execute_oversight_cycle_at(
                &store,
                "default",
                "periodic_supervisor",
                crate::admin::oversight_apply::OversightApplyMode::ExecuteCanary,
                start_ts + (index as u64 * 10_000) + 3_601,
            )
            .expect("periodic cycle succeeds");
            assert_eq!(periodic.apply.stage, "improved");
            assert_eq!(periodic.apply.patch_family.as_deref(), Some(*expected_family));
        }

        let final_cfg =
            crate::config::Config::load(&store, "default").expect("final config loads");
        assert!(final_cfg.fingerprint_signal_enabled);
        assert!(final_cfg.cdp_detection_enabled);
        assert!(final_cfg.pow_enabled);
        assert!(final_cfg.challenge_puzzle_enabled);
        assert!(final_cfg.not_a_bot_enabled);
        assert!(final_cfg.maze_enabled);
        assert!(final_cfg.maze_auto_ban);
        assert!(final_cfg.js_required_enforced);

        let history_request = Request::builder()
            .method(Method::Get)
            .uri("/admin/oversight/history")
            .body(Vec::new())
            .build();
        let history_response = handle_admin_oversight_history(&history_request, &store, "default");
        assert_eq!(*history_response.status(), 200);
        let history_payload: serde_json::Value =
            serde_json::from_slice(history_response.body()).expect("history decodes");

        let rows = history_payload["episode_archive"]["rows"]
            .as_array()
            .expect("episode archive rows");
        assert_eq!(rows.len(), 10);
        assert!(rows.iter().all(|row| {
            row["retain_or_rollback"].as_str() == Some("retained")
                && row["evaluation_context"]["profile_id"].as_str() == Some("human_only_private")
                && row["proposal_status"].as_str() == Some("accepted")
                && row["watch_window_result"].as_str() == Some("improved")
        }));

        let retained_families: std::collections::BTreeSet<String> = rows
            .iter()
            .filter_map(|row| row["proposal"]["patch_family"].as_str().map(str::to_string))
            .collect();
        assert!(retained_families.len() >= 7);

        let latest_request_current = rows[0]["benchmark_deltas"]
            .as_array()
            .and_then(|families| {
                families.iter().find(|family| {
                    family["family_id"].as_str() == Some("suspicious_origin_cost")
                })
            })
            .and_then(|family| family["metric_deltas"].as_array())
            .and_then(|metrics| {
                metrics.iter().find(|metric| {
                    metric["metric_id"].as_str() == Some("suspicious_forwarded_request_rate")
                })
            })
            .and_then(|metric| metric["current"].as_f64())
            .expect("latest request current");
        let oldest_baseline_request = rows[rows.len() - 1]["baseline_scorecard"]["families"]
            .as_array()
            .and_then(|families| {
                families.iter().find(|family| {
                    family["family_id"].as_str() == Some("suspicious_origin_cost")
                })
            })
            .and_then(|family| family["metrics"].as_array())
            .and_then(|metrics| {
                metrics.iter().find(|metric| {
                    metric["metric_id"].as_str() == Some("suspicious_forwarded_request_rate")
                })
            })
            .and_then(|metric| metric["current"].as_f64())
            .expect("oldest baseline request current");
        assert!(latest_request_current < oldest_baseline_request);
        assert_eq!(latest_request_current, 0.0);

        assert_eq!(
            history_payload["episode_archive"]["homeostasis"]["status"].as_str(),
            Some("improving")
        );
        assert_eq!(
            history_payload["episode_archive"]["homeostasis"]["break_status"].as_str(),
            Some("not_triggered")
        );
    }
}
