use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::runtime::request_outcome::ObservedRequestOutcomeSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ObservedSurfaceObservationRow {
    pub surface_id: String,
    pub surface_label: String,
    pub coverage_status: String,
    pub surface_state: String,
    pub attempt_count: u64,
    pub sample_response_kind: String,
    pub sample_http_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ObservedSurfaceCoverageReceipt {
    pub surface_id: String,
    pub coverage_status: String,
    pub surface_state: String,
    pub attempt_count: u64,
    pub sample_response_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_http_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ObservedSurfaceCoverageSummary {
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
    pub receipts: Vec<ObservedSurfaceCoverageReceipt>,
}

fn observed_surface_row(
    surface_id: &str,
    coverage_status: &str,
    sample_response_kind: &str,
    sample_http_status: Option<u16>,
) -> ObservedSurfaceObservationRow {
    ObservedSurfaceObservationRow {
        surface_id: surface_id.to_string(),
        surface_label: observed_surface_label(surface_id).to_string(),
        coverage_status: coverage_status.to_string(),
        surface_state: if coverage_status == "progress_observed" {
            "leaked".to_string()
        } else {
            "held".to_string()
        },
        attempt_count: 1,
        sample_response_kind: sample_response_kind.to_string(),
        sample_http_status,
    }
}

fn normalize_token(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .replace('-', "_")
        .replace(' ', "_")
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

pub(crate) fn observed_surface_label(surface_id: &str) -> &'static str {
    match surface_id {
        "public_ingress" => "Public Ingress",
        "challenge_puzzle" => "Challenge Puzzle",
        "not_a_bot" => "Not A Bot",
        "js_verification" => "JS Verification",
        "maze" => "Maze",
        "tarpit" => "Tarpit",
        "rate_pressure" => "Rate Pressure",
        "geo_ip_policy" => "Geo IP Policy",
        "honeypot" => "Honeypot",
        "pow_verify" => "PoW Verify",
        "ban_path" => "Ban Path",
        "browser_automation_detection" => "Browser Automation Detection",
        _ => "Observed Surface",
    }
}

pub(crate) fn summarize_observed_request_outcome_surface_observations(
    observed: &ObservedRequestOutcomeSummary,
) -> Vec<ObservedSurfaceObservationRow> {
    let response_kind = normalize_token(observed.response_kind.as_str());
    let http_status = Some(observed.http_status);
    match response_kind.as_str() {
        "forward_allow"
        | "forward_failure_fallback"
        | "synthetic_shadow_allow"
        | "synthetic_shadow_action"
        | "sim_public_response" => vec![observed_surface_row(
            "public_ingress",
            "progress_observed",
            response_kind.as_str(),
            http_status,
        )],
        "challenge" | "checkpoint_response" | "defence_followup_response" => {
            vec![observed_surface_row(
                "challenge_puzzle",
                "response_observed",
                response_kind.as_str(),
                http_status,
            )]
        }
        "not_a_bot" => vec![observed_surface_row(
            "not_a_bot",
            "response_observed",
            response_kind.as_str(),
            http_status,
        )],
        "js_challenge" => vec![observed_surface_row(
            "js_verification",
            "response_observed",
            response_kind.as_str(),
            http_status,
        )],
        "maze" => vec![observed_surface_row(
            "maze",
            "response_observed",
            response_kind.as_str(),
            http_status,
        )],
        "tarpit" => vec![observed_surface_row(
            "tarpit",
            "progress_observed",
            response_kind.as_str(),
            http_status,
        )],
        "block_page" | "plain_text_block" | "redirect" | "drop_connection" => {
            vec![observed_surface_row(
                "ban_path",
                "response_observed",
                response_kind.as_str(),
                http_status,
            )]
        }
        _ => Vec::new(),
    }
}

pub(crate) fn summarize_monitoring_event_surface_observations(
    event_type: &str,
    reason: Option<&str>,
    outcome_code: Option<&str>,
    outcome: Option<&str>,
) -> Vec<ObservedSurfaceObservationRow> {
    let combined = format!(
        "{} {} {} {}",
        normalize_token(event_type),
        normalize_token(reason.unwrap_or_default()),
        normalize_token(outcome_code.unwrap_or_default()),
        normalize_token(outcome.unwrap_or_default())
    );
    if combined.contains("challenge_puzzle_pass") {
        return vec![observed_surface_row(
            "challenge_puzzle",
            "progress_observed",
            "challenge_puzzle_pass",
            None,
        )];
    }
    if combined.contains("not_a_bot_pass") {
        return vec![observed_surface_row(
            "not_a_bot",
            "progress_observed",
            "not_a_bot_pass",
            None,
        )];
    }
    if combined.contains("honeypot") {
        return vec![observed_surface_row(
            "honeypot",
            "progress_observed",
            "honeypot",
            None,
        )];
    }
    if combined.contains("tarpit") {
        return vec![observed_surface_row(
            "tarpit",
            "progress_observed",
            "tarpit",
            None,
        )];
    }
    if combined.contains("pow") || combined.contains("proof") {
        return vec![observed_surface_row(
            "pow_verify",
            "progress_observed",
            "pow_verify",
            None,
        )];
    }
    if combined.contains("rate") {
        return vec![observed_surface_row(
            "rate_pressure",
            "progress_observed",
            "rate_pressure",
            None,
        )];
    }
    if combined.contains("geo") {
        return vec![observed_surface_row(
            "geo_ip_policy",
            "progress_observed",
            "geo_ip_policy",
            None,
        )];
    }
    if combined.contains("cdp") || combined.contains("fingerprint") {
        return vec![observed_surface_row(
            "browser_automation_detection",
            "response_observed",
            "browser_automation_detection",
            None,
        )];
    }
    if combined.contains("maze") {
        return vec![observed_surface_row(
            "maze",
            "response_observed",
            "maze",
            None,
        )];
    }
    if combined.contains("not_a_bot") {
        return vec![observed_surface_row(
            "not_a_bot",
            "response_observed",
            "not_a_bot",
            None,
        )];
    }
    if combined.contains("challenge") {
        return vec![observed_surface_row(
            "challenge_puzzle",
            "response_observed",
            "challenge_puzzle",
            None,
        )];
    }
    if combined.contains("ban")
        || combined.contains("deny_temp")
        || combined.contains("block")
        || combined.contains("drop_connection")
    {
        return vec![observed_surface_row(
            "ban_path",
            "response_observed",
            "ban_path",
            None,
        )];
    }
    Vec::new()
}

pub(crate) fn summarize_observed_surface_coverage(
    observations: &[ObservedSurfaceObservationRow],
) -> Option<ObservedSurfaceCoverageSummary> {
    if observations.is_empty() {
        return None;
    }

    let mut receipts_by_surface: BTreeMap<String, ObservedSurfaceCoverageReceipt> =
        BTreeMap::new();
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
            .or_insert_with(|| ObservedSurfaceCoverageReceipt {
                surface_id: observation.surface_id.clone(),
                coverage_status: observation.coverage_status.clone(),
                surface_state: observation.surface_state.clone(),
                attempt_count: 0,
                sample_response_kind: observation.sample_response_kind.clone(),
                sample_http_status: observation.sample_http_status,
            });
        entry.attempt_count = entry.attempt_count.saturating_add(observation.attempt_count);
        entry.surface_state =
            merge_surface_state(entry.surface_state.as_str(), observation.surface_state.as_str());
        entry.coverage_status = merge_coverage_status(
            entry.coverage_status.as_str(),
            observation.coverage_status.as_str(),
        );
        if entry.sample_response_kind.is_empty() && !observation.sample_response_kind.is_empty() {
            entry.sample_response_kind = observation.sample_response_kind.clone();
        }
        if entry.sample_http_status.is_none() {
            entry.sample_http_status = observation.sample_http_status;
        }
    }

    if receipts_by_surface.is_empty() {
        return None;
    }

    let receipts: Vec<ObservedSurfaceCoverageReceipt> = receipts_by_surface.into_values().collect();
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
        .filter(|receipt| receipt.coverage_status == "progress_observed")
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

    Some(ObservedSurfaceCoverageSummary {
        overall_status: overall_status.to_string(),
        surface_labels,
        observed_surface_ids,
        response_surface_ids,
        progress_surface_ids,
        receipts,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_monitoring_event_surface_observations,
        summarize_observed_request_outcome_surface_observations,
        summarize_observed_surface_coverage,
    };
    use crate::runtime::request_outcome::ObservedRequestOutcomeSummary;

    #[test]
    fn observed_request_outcomes_materialize_shared_surface_progress_and_response() {
        let observations = vec![
            summarize_observed_request_outcome_surface_observations(&ObservedRequestOutcomeSummary {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                route_action_family: "public_content".to_string(),
                execution_mode: "enforced".to_string(),
                outcome_class: "forwarded".to_string(),
                response_kind: "forward_allow".to_string(),
                policy_source: "clean_allow".to_string(),
                http_status: 200,
                traffic_lane: Some("suspicious_automation".to_string()),
                lane_exactness: Some("exact".to_string()),
                lane_basis: Some("policy".to_string()),
                non_human_category_id: None,
                non_human_assignment_status: None,
            }),
            summarize_observed_request_outcome_surface_observations(&ObservedRequestOutcomeSummary {
                traffic_origin: "adversary_sim".to_string(),
                measurement_scope: "ingress_primary".to_string(),
                route_action_family: "public_content".to_string(),
                execution_mode: "enforced".to_string(),
                outcome_class: "short_circuited".to_string(),
                response_kind: "challenge".to_string(),
                policy_source: "policy_graph_second_tranche".to_string(),
                http_status: 403,
                traffic_lane: Some("suspicious_automation".to_string()),
                lane_exactness: Some("exact".to_string()),
                lane_basis: Some("policy".to_string()),
                non_human_category_id: None,
                non_human_assignment_status: None,
            }),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let summary =
            summarize_observed_surface_coverage(observations.as_slice()).expect("coverage");
        assert_eq!(summary.overall_status, "partial_progress");
        assert_eq!(
            summary.observed_surface_ids,
            vec!["challenge_puzzle".to_string(), "public_ingress".to_string()]
        );
        assert_eq!(
            summary.progress_surface_ids,
            vec!["public_ingress".to_string()]
        );
    }

    #[test]
    fn monitoring_event_tokens_project_shared_surface_rows() {
        let observations = summarize_monitoring_event_surface_observations(
            "challenge",
            Some("challenge_puzzle_pass"),
            None,
            Some("pass"),
        );
        let summary =
            summarize_observed_surface_coverage(observations.as_slice()).expect("coverage");
        assert_eq!(summary.overall_status, "progress_observed");
        assert_eq!(
            summary.progress_surface_ids,
            vec!["challenge_puzzle".to_string()]
        );
    }
}
