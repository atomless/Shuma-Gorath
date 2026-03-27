use std::collections::BTreeSet;

use crate::observability::operator_snapshot::{
    OperatorSnapshotAdversarySim, OperatorSnapshotNonHumanTrafficSummary,
};

use super::benchmark_results::{
    BenchmarkDiagnosisEvidenceQuality, BenchmarkExploitLocus,
};
use super::benchmark_scrapling_exploit_progress::host_cost_channels_for_surface;
use super::benchmark_scrapling_exploit_progress::repair_families_for_surface;
use super::benchmark_scrapling_exploit_progress::latest_scrapling_recent_run;

pub(crate) fn scrapling_evidence_quality_assessment(
    adversary_sim: &OperatorSnapshotAdversarySim,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
) -> BenchmarkDiagnosisEvidenceQuality {
    let Some(run) = latest_scrapling_recent_run(adversary_sim) else {
        return BenchmarkDiagnosisEvidenceQuality {
            status: "insufficient_evidence".to_string(),
            diagnosis_confidence: "not_available".to_string(),
            attribution_status: "not_available".to_string(),
            sample_status: "missing_recent_run".to_string(),
            freshness_status: "missing_recent_run".to_string(),
            persona_diversity_status: "not_available".to_string(),
            reproducibility_status: "not_available".to_string(),
            locality_status: "not_localized".to_string(),
            breach_loci: Vec::new(),
            note: "No recent Scrapling run is visible, so exploit diagnosis cannot yet justify a bounded config move.".to_string(),
        };
    };
    let Some(coverage) = run.owned_surface_coverage.as_ref() else {
        return BenchmarkDiagnosisEvidenceQuality {
            status: "insufficient_evidence".to_string(),
            diagnosis_confidence: "not_available".to_string(),
            attribution_status: "surface_receipts_missing".to_string(),
            sample_status: "missing_surface_receipts".to_string(),
            freshness_status: "fresh_recent_run".to_string(),
            persona_diversity_status: if run.observed_fulfillment_modes.is_empty() {
                "not_available".to_string()
            } else {
                "single_persona".to_string()
            },
            reproducibility_status: "not_available".to_string(),
            locality_status: "not_localized".to_string(),
            breach_loci: Vec::new(),
            note: format!(
                "Latest Scrapling run {} has no owned-surface coverage summary, so exploit diagnosis remains too weak for fine-grained tuning.",
                run.run_id
            ),
        };
    };

    let breach_loci: Vec<BenchmarkExploitLocus> = coverage
        .receipts
        .iter()
        .filter(|receipt| receipt.coverage_status == "pass_observed")
        .map(|receipt| BenchmarkExploitLocus {
            locus_id: receipt.surface_id.clone(),
            locus_label: coverage
                .surface_labels
                .get(receipt.surface_id.as_str())
                .cloned()
            .unwrap_or_else(|| receipt.surface_id.clone()),
            stage_id: stage_id(receipt.surface_id.as_str()).to_string(),
            evidence_status: "progress_observed".to_string(),
            attempt_count: receipt.attempt_count,
            cost_channel_ids: host_cost_channels_for_surface(receipt.surface_id.as_str())
                .iter()
                .map(|channel| (*channel).to_string())
                .collect(),
            sample_request_method: receipt.sample_request_method.clone(),
            sample_request_path: receipt.sample_request_path.clone(),
            sample_response_status: receipt.sample_response_status,
            repair_family_candidates: repair_families_for_surface(receipt.surface_id.as_str())
                .iter()
                .map(|family| (*family).to_string())
                .collect(),
        })
        .collect();
    let attribution_status = if breach_loci.is_empty() {
        "projected_or_incomplete"
    } else if non_human_traffic.restriction_readiness.status == "ready" {
        "category_and_surface_native"
    } else {
        "surface_native_shared_path"
    };
    let sample_status = if coverage.receipts.iter().all(|receipt| receipt.attempt_count > 0) {
        "sufficient"
    } else {
        "insufficient"
    };
    let persona_diversity_status = match run.observed_fulfillment_modes.len() {
        0 => "not_available",
        1 => "single_persona",
        _ => "multi_persona",
    };
    let reproducibility_status = reproducibility_status(
        adversary_sim,
        run.run_id.as_str(),
        breach_loci.as_slice(),
    );
    let locality_status = if breach_loci.is_empty() {
        "not_localized"
    } else {
        "localized"
    };
    let high_confidence = matches!(
        attribution_status,
        "category_and_surface_native" | "surface_native_shared_path"
    )
        && sample_status == "sufficient"
        && persona_diversity_status == "multi_persona"
        && reproducibility_status == "reproduced_recently"
        && locality_status == "localized";

    BenchmarkDiagnosisEvidenceQuality {
        status: if high_confidence {
            "high_confidence".to_string()
        } else {
            "low_confidence".to_string()
        },
        diagnosis_confidence: if high_confidence {
            "high".to_string()
        } else {
            "low".to_string()
        },
        attribution_status: attribution_status.to_string(),
        sample_status: sample_status.to_string(),
        freshness_status: "fresh_recent_run".to_string(),
        persona_diversity_status: persona_diversity_status.to_string(),
        reproducibility_status: reproducibility_status.to_string(),
        locality_status: locality_status.to_string(),
        breach_loci: breach_loci.clone(),
        note: if high_confidence {
            format!(
                "Latest Scrapling exploit evidence is localized, multi-persona, and reproduced at: {}.",
                breach_loci
                    .iter()
                    .map(|locus| locus.locus_label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!(
                "Latest Scrapling exploit evidence remains too weak for fine-grained tuning because attribution={}, samples={}, personas={}, reproducibility={}, locality={}.",
                attribution_status,
                sample_status,
                persona_diversity_status,
                reproducibility_status,
                locality_status
            )
        },
    }
}

fn reproducibility_status(
    adversary_sim: &OperatorSnapshotAdversarySim,
    current_run_id: &str,
    breach_loci: &[BenchmarkExploitLocus],
) -> &'static str {
    if breach_loci.is_empty() {
        return "single_run_only";
    }
    let current_ids: BTreeSet<_> = breach_loci.iter().map(|locus| locus.locus_id.as_str()).collect();
    let reproduced = adversary_sim
        .recent_runs
        .iter()
        .filter(|run| run.lane == "scrapling_traffic" && run.run_id != current_run_id)
        .filter_map(|run| run.owned_surface_coverage.as_ref())
        .any(|coverage| {
            coverage
                .receipts
                .iter()
                .filter(|receipt| receipt.coverage_status == "pass_observed")
                .any(|receipt| current_ids.contains(receipt.surface_id.as_str()))
        });
    if reproduced {
        "reproduced_recently"
    } else {
        "single_run_only"
    }
}

fn stage_id(surface_id: &str) -> &'static str {
    match surface_id {
        "public_path_traversal"
        | "challenge_routing"
        | "rate_pressure"
        | "geo_ip_policy" => "exposure",
        "not_a_bot_submit"
        | "puzzle_submit_or_escalation"
        | "maze_navigation"
        | "js_verification_execution" => "interactive",
        _ => "control_bypass",
    }
}
