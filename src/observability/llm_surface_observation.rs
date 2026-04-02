use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::admin::adversary_sim_worker_plan::{
    LlmRuntimeActionReceipt, LlmRuntimeRecentRunSummary,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LlmSurfaceObservationRow {
    pub surface_id: String,
    pub surface_label: String,
    pub surface_state: String,
    pub coverage_status: String,
    pub success_contract: String,
    pub dependency_kind: String,
    pub dependency_surface_ids: Vec<String>,
    pub attempt_count: u64,
    pub sample_request_method: String,
    pub sample_request_path: String,
    pub sample_response_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmSurfaceCoverageReceipt {
    pub surface_id: String,
    pub coverage_status: String,
    pub surface_state: String,
    pub attempt_count: u64,
    pub sample_request_method: String,
    pub sample_request_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_response_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct LlmSurfaceCoverageSummary {
    pub overall_status: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub surface_labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub progress_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<LlmSurfaceCoverageReceipt>,
}

pub(crate) fn llm_receipt_surface_id(receipt: &LlmRuntimeActionReceipt) -> Option<&'static str> {
    let path = receipt.path.as_str();
    if crate::http_route_namespace::is_generated_public_site_path(path)
        || path.starts_with("/detail/")
        || path.starts_with("/search")
    {
        Some("public_path_traversal")
    } else if path.starts_with("/challenge") {
        Some("challenge_routing")
    } else if path.starts_with("/maze") {
        Some("maze_navigation")
    } else if path.starts_with("/pow") {
        Some("pow_verify_abuse")
    } else if path.starts_with("/tarpit") {
        Some("tarpit_progress_abuse")
    } else {
        None
    }
}

pub(crate) fn llm_receipt_indicates_exploit_progress(
    receipt: &LlmRuntimeActionReceipt,
) -> bool {
    receipt.error.is_none()
        && receipt
            .status
            .map(|status| (200..400).contains(&status))
            .unwrap_or(false)
}

pub(crate) fn llm_request_method(receipt: &LlmRuntimeActionReceipt) -> String {
    match receipt.action_type.as_str() {
        "http_get" | "browser_navigate" => "GET".to_string(),
        other => other.to_uppercase(),
    }
}

pub(crate) fn llm_surface_label(surface_id: &str) -> &'static str {
    match surface_id {
        "public_path_traversal" => "Public Path Traversal",
        "challenge_routing" => "Challenge Routing",
        "maze_navigation" => "Maze Navigation",
        "pow_verify_abuse" => "PoW Verify Abuse",
        "tarpit_progress_abuse" => "Tarpit Progress Abuse",
        _ => "Unknown Board Locus",
    }
}

fn llm_surface_state(receipt: &LlmRuntimeActionReceipt) -> &'static str {
    if llm_receipt_indicates_exploit_progress(receipt) {
        "leaked"
    } else if receipt.status.is_some() {
        "held"
    } else {
        "attempted"
    }
}

fn llm_surface_coverage_status(surface_state: &str) -> &'static str {
    match surface_state {
        "leaked" => "progress_observed",
        "held" => "response_observed",
        _ => "attempt_observed",
    }
}

fn merge_coverage_status(current: &str, next: &str) -> String {
    match (current, next) {
        ("progress_observed", _) | (_, "progress_observed") => "progress_observed".to_string(),
        ("response_observed", _) | (_, "response_observed") => "response_observed".to_string(),
        _ => "attempt_observed".to_string(),
    }
}

fn merge_surface_state(current: &str, next: &str) -> String {
    match (current, next) {
        ("leaked", _) | (_, "leaked") => "leaked".to_string(),
        ("held", _) | (_, "held") => "held".to_string(),
        _ => "attempted".to_string(),
    }
}

pub(crate) fn summarize_llm_surface_observations(
    summary: Option<&LlmRuntimeRecentRunSummary>,
) -> Vec<LlmSurfaceObservationRow> {
    let Some(summary) = summary else {
        return Vec::new();
    };

    let mut rows: BTreeMap<String, LlmSurfaceObservationRow> = BTreeMap::new();
    for receipt in &summary.latest_action_receipts {
        let Some(surface_id) = llm_receipt_surface_id(receipt) else {
            continue;
        };
        let request_method = llm_request_method(receipt);
        let surface_state = llm_surface_state(receipt);
        let entry = rows
            .entry(surface_id.to_string())
            .or_insert_with(|| LlmSurfaceObservationRow {
                surface_id: surface_id.to_string(),
                surface_label: llm_surface_label(surface_id).to_string(),
                surface_state: surface_state.to_string(),
                coverage_status: llm_surface_coverage_status(surface_state).to_string(),
                success_contract: "runtime_action_observed".to_string(),
                dependency_kind: "independent".to_string(),
                dependency_surface_ids: Vec::new(),
                attempt_count: 0,
                sample_request_method: request_method.clone(),
                sample_request_path: receipt.path.clone(),
                sample_response_status: receipt.status,
            });
        entry.attempt_count = entry.attempt_count.saturating_add(1);
        entry.surface_state = merge_surface_state(entry.surface_state.as_str(), surface_state);
        entry.coverage_status = merge_coverage_status(
            entry.coverage_status.as_str(),
            llm_surface_coverage_status(surface_state),
        );
        if entry.sample_request_path.is_empty() && !receipt.path.is_empty() {
            entry.sample_request_method = request_method;
            entry.sample_request_path = receipt.path.clone();
            entry.sample_response_status = receipt.status;
        } else if entry.sample_response_status.is_none() {
            entry.sample_response_status = receipt.status;
        }
    }

    rows.into_values().collect()
}

pub(crate) fn summarize_llm_surface_coverage(
    observations: &[LlmSurfaceObservationRow],
) -> Option<LlmSurfaceCoverageSummary> {
    if observations.is_empty() {
        return None;
    }

    let mut receipts_by_surface: BTreeMap<String, LlmSurfaceCoverageReceipt> = BTreeMap::new();
    let mut surface_labels: BTreeMap<String, String> = BTreeMap::new();

    for observation in observations {
        if observation.surface_id.trim().is_empty() {
            continue;
        }
        surface_labels
            .entry(observation.surface_id.clone())
            .or_insert_with(|| observation.surface_label.clone());
        let entry = receipts_by_surface
            .entry(observation.surface_id.clone())
            .or_insert_with(|| LlmSurfaceCoverageReceipt {
                surface_id: observation.surface_id.clone(),
                coverage_status: observation.coverage_status.clone(),
                surface_state: observation.surface_state.clone(),
                attempt_count: 0,
                sample_request_method: observation.sample_request_method.clone(),
                sample_request_path: observation.sample_request_path.clone(),
                sample_response_status: observation.sample_response_status,
            });
        entry.attempt_count = entry.attempt_count.saturating_add(observation.attempt_count);
        entry.surface_state =
            merge_surface_state(entry.surface_state.as_str(), observation.surface_state.as_str());
        entry.coverage_status = merge_coverage_status(
            entry.coverage_status.as_str(),
            observation.coverage_status.as_str(),
        );
        if entry.sample_request_path.is_empty() && !observation.sample_request_path.is_empty() {
            entry.sample_request_method = observation.sample_request_method.clone();
            entry.sample_request_path = observation.sample_request_path.clone();
            entry.sample_response_status = observation.sample_response_status;
        } else if entry.sample_response_status.is_none() {
            entry.sample_response_status = observation.sample_response_status;
        }
    }

    if receipts_by_surface.is_empty() {
        return None;
    }

    let receipts: Vec<LlmSurfaceCoverageReceipt> = receipts_by_surface.into_values().collect();
    let observed_surface_ids: Vec<String> = receipts
        .iter()
        .map(|receipt| receipt.surface_id.clone())
        .collect();
    let response_surface_ids: Vec<String> = receipts
        .iter()
        .filter(|receipt| receipt.coverage_status != "attempt_observed")
        .map(|receipt| receipt.surface_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let progress_surface_ids: Vec<String> = receipts
        .iter()
        .filter(|receipt| receipt.surface_state == "leaked")
        .map(|receipt| receipt.surface_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let overall_status = if progress_surface_ids.len() == observed_surface_ids.len() {
        "progress_observed"
    } else if !progress_surface_ids.is_empty() {
        "partial_progress"
    } else if !response_surface_ids.is_empty() {
        "response_observed"
    } else {
        "attempt_observed"
    };

    Some(LlmSurfaceCoverageSummary {
        overall_status: overall_status.to_string(),
        surface_labels,
        observed_surface_ids,
        response_surface_ids,
        progress_surface_ids,
        receipts,
    })
}
