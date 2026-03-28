use std::collections::BTreeSet;

use crate::observability::operator_snapshot::{
    OperatorSnapshotAdversarySim, OperatorSnapshotNonHumanTrafficSummary,
};

use super::benchmark_mixed_attacker_restriction_progress::{
    current_mixed_breach_locus_ids, latest_mixed_attacker_restriction_state,
    run_contains_breach_locus,
};
use super::benchmark_results::BenchmarkDiagnosisEvidenceQuality;

pub(crate) fn mixed_attacker_evidence_quality_assessment(
    adversary_sim: &OperatorSnapshotAdversarySim,
    non_human_traffic: &OperatorSnapshotNonHumanTrafficSummary,
) -> BenchmarkDiagnosisEvidenceQuality {
    let state = latest_mixed_attacker_restriction_state(adversary_sim);
    if state.latest_lane_ids.is_empty() {
        return BenchmarkDiagnosisEvidenceQuality {
            status: "insufficient_evidence".to_string(),
            diagnosis_confidence: "not_available".to_string(),
            attribution_status: "not_available".to_string(),
            sample_status: "missing_recent_run".to_string(),
            freshness_status: "missing_recent_run".to_string(),
            recent_window_support_status: "not_available".to_string(),
            locality_status: "not_localized".to_string(),
            breach_loci: Vec::new(),
            note: "No recent mixed-attacker lane evidence is visible, so exploit diagnosis cannot yet justify a bounded move.".to_string(),
        };
    }

    let breach_loci = state.exploit_loci.clone();
    let attribution_status = if breach_loci.is_empty() {
        "projected_or_incomplete"
    } else if non_human_traffic.restriction_readiness.status == "ready" {
        "category_and_surface_native"
    } else {
        "surface_native_shared_path"
    };
    let sample_status = if !breach_loci.is_empty()
        && breach_loci.iter().all(|locus| locus.attempt_count.unwrap_or(0) > 0)
    {
        "sufficient"
    } else {
        "insufficient"
    };
    let recent_window_support_status =
        recent_window_support_status(adversary_sim, current_mixed_breach_locus_ids(adversary_sim));
    let locality_status = if breach_loci.is_empty() {
        "not_localized"
    } else {
        "localized"
    };
    let high_confidence = matches!(
        attribution_status,
        "category_and_surface_native" | "surface_native_shared_path"
    ) && sample_status == "sufficient"
        && recent_window_support_status == "reproduced_recently"
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
        recent_window_support_status: recent_window_support_status.to_string(),
        locality_status: locality_status.to_string(),
        breach_loci: breach_loci.clone(),
        note: if high_confidence {
            format!(
                "Latest mixed-attacker exploit evidence is localized and supported across the recent board-state window at: {}.",
                breach_loci
                    .iter()
                    .map(|locus| locus.locus_label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!(
                "Latest mixed-attacker exploit evidence remains too weak for fine-grained tuning because attribution={}, samples={}, recent_window={}, locality={}.",
                attribution_status,
                sample_status,
                recent_window_support_status,
                locality_status
            )
        },
    }
}

fn recent_window_support_status(
    adversary_sim: &OperatorSnapshotAdversarySim,
    current_ids: BTreeSet<String>,
) -> &'static str {
    if current_ids.is_empty() {
        return "single_run_only";
    }
    let reproduced = adversary_sim.recent_runs.len() > 1
        && adversary_sim.recent_runs.iter().any(|run| {
            current_ids
                .iter()
                .any(|locus_id| run_contains_breach_locus(run, locus_id.as_str()))
        });
    if reproduced {
        "reproduced_recently"
    } else {
        "single_run_only"
    }
}
