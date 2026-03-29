use std::collections::BTreeMap;

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

pub(crate) fn llm_receipt_surface_id(receipt: &LlmRuntimeActionReceipt) -> Option<&'static str> {
    let path = receipt.path.as_str();
    if path == "/"
        || path.starts_with("/sim/public/")
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
                coverage_status: "attempt_observed".to_string(),
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

