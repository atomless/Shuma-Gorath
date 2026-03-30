#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SCRAPLING_OWNED_SURFACE_SCHEMA_VERSION: &str =
    "scrapling_owned_surface_contract_v1";

const SCRAPLING_OWNED_SURFACE_TARGETS: [&str; 11] = [
    "public_path_traversal",
    "challenge_routing",
    "rate_pressure",
    "geo_ip_policy",
    "not_a_bot_submit",
    "puzzle_submit_or_escalation",
    "pow_verify_abuse",
    "tarpit_progress_abuse",
    "maze_navigation",
    "js_verification_execution",
    "browser_automation_detection",
];
const SCRAPLING_CRAWLER_SURFACE_TARGETS: [&str; 4] = [
    "public_path_traversal",
    "challenge_routing",
    "rate_pressure",
    "geo_ip_policy",
];
const SCRAPLING_BULK_SCRAPER_SURFACE_TARGETS: [&str; 6] = [
    "public_path_traversal",
    "challenge_routing",
    "rate_pressure",
    "geo_ip_policy",
    "not_a_bot_submit",
    "puzzle_submit_or_escalation",
];
const SCRAPLING_HTTP_AGENT_SURFACE_TARGETS: [&str; 7] = [
    "challenge_routing",
    "rate_pressure",
    "geo_ip_policy",
    "not_a_bot_submit",
    "puzzle_submit_or_escalation",
    "pow_verify_abuse",
    "tarpit_progress_abuse",
];
const SCRAPLING_BROWSER_AUTOMATION_SURFACE_TARGETS: [&str; 4] = [
    "challenge_routing",
    "maze_navigation",
    "js_verification_execution",
    "browser_automation_detection",
];
const SCRAPLING_STEALTH_BROWSER_SURFACE_TARGETS: [&str; 4] = [
    "challenge_routing",
    "maze_navigation",
    "js_verification_execution",
    "browser_automation_detection",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedSurfaceRow {
    pub surface_id: String,
    pub surface_label: String,
    pub assignment_status: String,
    pub required_transport: String,
    pub interaction_requirement: String,
    pub success_contract: String,
    pub dependency_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fulfillment_modes: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedSurfaceSummary {
    pub schema_version: String,
    pub owned_surface_count: usize,
    pub other_lane_surface_count: usize,
    pub out_of_scope_surface_count: usize,
    pub rows: Vec<ScraplingOwnedSurfaceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingSurfaceObservationReceipt {
    pub surface_id: String,
    pub coverage_status: String,
    pub attempt_count: u64,
    pub sample_request_method: String,
    pub sample_request_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_response_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedSurfaceCoverageReceipt {
    pub surface_id: String,
    pub success_contract: String,
    pub dependency_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_surface_ids: Vec<String>,
    pub coverage_status: String,
    pub surface_state: String,
    pub satisfied: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocked_by_surface_ids: Vec<String>,
    pub attempt_count: u64,
    pub sample_request_method: String,
    pub sample_request_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_response_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct ScraplingOwnedSurfaceCoverageSummary {
    pub overall_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub surface_labels: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub satisfied_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_surface_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub receipts: Vec<ScraplingOwnedSurfaceCoverageReceipt>,
}

pub(crate) fn scrapling_owned_surface_targets() -> Vec<String> {
    SCRAPLING_OWNED_SURFACE_TARGETS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(crate) fn scrapling_owned_surface_targets_for_mode(mode: &str) -> Vec<String> {
    match mode {
        "crawler" => SCRAPLING_CRAWLER_SURFACE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "bulk_scraper" => SCRAPLING_BULK_SCRAPER_SURFACE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "browser_automation" => SCRAPLING_BROWSER_AUTOMATION_SURFACE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "stealth_browser" => SCRAPLING_STEALTH_BROWSER_SURFACE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "http_agent" => SCRAPLING_HTTP_AGENT_SURFACE_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn scrapling_owned_surface_targets_for_modes(modes: &[String]) -> Vec<String> {
    let mode_set: BTreeSet<_> = modes.iter().map(|value| value.as_str()).collect();
    canonical_scrapling_owned_surface_summary()
        .rows
        .into_iter()
        .filter(|row| row.assignment_status == "owned")
        .filter(|row| {
            row.fulfillment_modes
                .iter()
                .any(|mode| mode_set.contains(mode.as_str()))
        })
        .map(|row| row.surface_id)
        .collect()
}

pub(crate) fn summarize_scrapling_owned_surface_coverage(
    observed_modes: &[String],
    observations: &[ScraplingSurfaceObservationReceipt],
) -> Option<ScraplingOwnedSurfaceCoverageSummary> {
    let required_surface_ids = scrapling_owned_surface_targets_for_modes(observed_modes);
    if required_surface_ids.is_empty() {
        return None;
    }

    let summary = canonical_scrapling_owned_surface_summary();
    let canonical_surface_ids: Vec<String> = summary
        .rows
        .iter()
        .map(|row| row.surface_id.clone())
        .collect();
    let surface_labels: BTreeMap<String, String> = summary
        .rows
        .iter()
        .map(|row| (row.surface_id.clone(), row.surface_label.clone()))
        .collect();
    let row_by_id: BTreeMap<_, _> = summary
        .rows
        .into_iter()
        .map(|row| (row.surface_id.clone(), row))
        .collect();
    let mut observations_by_surface: BTreeMap<String, Vec<&ScraplingSurfaceObservationReceipt>> =
        BTreeMap::new();
    for observation in observations {
        observations_by_surface
            .entry(observation.surface_id.clone())
            .or_default()
            .push(observation);
    }

    let required_surface_set: BTreeSet<_> = required_surface_ids.iter().cloned().collect();
    let mut receipt_drafts = Vec::new();

    for surface_id in &required_surface_ids {
        let Some(row) = row_by_id.get(surface_id) else {
            continue;
        };
        let best_observation = observations_by_surface
            .get(surface_id)
            .and_then(|rows| best_surface_observation(row.success_contract.as_str(), rows.as_slice()));
        let coverage_status = best_observation
            .map(|value| value.coverage_status.clone())
            .unwrap_or_else(|| "unavailable".to_string());
        let satisfied = surface_status_satisfies_contract(
            row.success_contract.as_str(),
            coverage_status.as_str(),
        );
        let attempt_count = observations_by_surface
            .get(surface_id)
            .map(|rows| rows.iter().map(|row| row.attempt_count).sum())
            .unwrap_or(0);
        let (sample_request_method, sample_request_path, sample_response_status) =
            best_observation
                .map(|value| {
                    (
                        value.sample_request_method.clone(),
                        value.sample_request_path.clone(),
                        value.sample_response_status,
                    )
                })
                .unwrap_or_else(|| (String::new(), String::new(), None));

        receipt_drafts.push((
            surface_id.clone(),
            row.clone(),
            coverage_status,
            satisfied,
            attempt_count,
            sample_request_method,
            sample_request_path,
            sample_response_status,
        ));
    }

    let satisfied_surface_ids: Vec<String> = receipt_drafts
        .iter()
        .filter(|(_, _, _, satisfied, _, _, _, _)| *satisfied)
        .map(|(surface_id, _, _, _, _, _, _, _)| surface_id.clone())
        .collect();
    let satisfied_surface_set: BTreeSet<_> = satisfied_surface_ids.iter().cloned().collect();
    let mut receipts = Vec::new();
    let mut blocking_surface_ids = Vec::new();

    for (
        surface_id,
        row,
        coverage_status,
        satisfied,
        attempt_count,
        sample_request_method,
        sample_request_path,
        sample_response_status,
    ) in receipt_drafts
    {
        let blocked_by_surface_ids = blocked_prerequisite_surface_ids(
            row.dependency_kind.as_str(),
            row.dependency_surface_ids.as_slice(),
            &required_surface_set,
            &satisfied_surface_set,
        );
        let surface_state = coverage_receipt_state_from_parts(
            satisfied,
            attempt_count,
            blocked_by_surface_ids.as_slice(),
        )
        .to_string();
        if !satisfied {
            blocking_surface_ids.push(surface_id.clone());
        }
        receipts.push(ScraplingOwnedSurfaceCoverageReceipt {
            surface_id,
            success_contract: row.success_contract.clone(),
            dependency_kind: row.dependency_kind.clone(),
            dependency_surface_ids: row.dependency_surface_ids.clone(),
            coverage_status,
            surface_state,
            satisfied,
            blocked_by_surface_ids,
            attempt_count,
            sample_request_method,
            sample_request_path,
            sample_response_status,
        });
    }

    let overall_status = if receipts.iter().all(|receipt| receipt.satisfied) {
        "covered".to_string()
    } else if receipts.iter().any(|receipt| receipt.attempt_count > 0) {
        "partial".to_string()
    } else {
        "unavailable".to_string()
    };

    Some(ScraplingOwnedSurfaceCoverageSummary {
        overall_status,
        canonical_surface_ids,
        surface_labels,
        required_surface_ids,
        satisfied_surface_ids,
        blocking_surface_ids,
        receipts,
    })
}

pub(crate) fn coverage_receipt_state(receipt: &ScraplingOwnedSurfaceCoverageReceipt) -> &'static str {
    if !receipt.surface_state.trim().is_empty() {
        return match receipt.surface_state.as_str() {
            "satisfied" => "satisfied",
            "attempted_blocked" => "attempted_blocked",
            "blocked_by_prerequisite" => "blocked_by_prerequisite",
            "unreached" => "unreached",
            _ => "unreached",
        };
    }
    coverage_receipt_state_from_parts(
        receipt.satisfied,
        receipt.attempt_count,
        receipt.blocked_by_surface_ids.as_slice(),
    )
}

fn coverage_receipt_state_from_parts(
    satisfied: bool,
    attempt_count: u64,
    blocked_by_surface_ids: &[String],
) -> &'static str {
    if satisfied {
        "satisfied"
    } else if attempt_count > 0 {
        "attempted_blocked"
    } else if !blocked_by_surface_ids.is_empty() {
        "blocked_by_prerequisite"
    } else {
        "unreached"
    }
}

fn format_surface_label_list(
    surface_ids: &[String],
    surface_labels: Option<&BTreeMap<String, String>>,
) -> String {
    surface_ids
        .iter()
        .map(|surface_id| {
            surface_labels
                .and_then(|labels| labels.get(surface_id))
                .cloned()
                .unwrap_or_else(|| surface_id.clone())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn coverage_receipt_state_label(
    receipt: &ScraplingOwnedSurfaceCoverageReceipt,
) -> String {
    match coverage_receipt_state(receipt) {
        "satisfied" => "satisfied".to_string(),
        "attempted_blocked" => "attempted and blocked".to_string(),
        "blocked_by_prerequisite" => {
            if receipt.blocked_by_surface_ids.is_empty() {
                "blocked by prerequisite".to_string()
            } else {
                format!(
                    "blocked by prerequisite: {}",
                    format_surface_label_list(receipt.blocked_by_surface_ids.as_slice(), None)
                )
            }
        }
        "unreached" => "required but unreached".to_string(),
        _ => "state unavailable".to_string(),
    }
}

pub(crate) fn coverage_receipt_dependency_label(
    receipt: &ScraplingOwnedSurfaceCoverageReceipt,
    surface_labels: &BTreeMap<String, String>,
) -> String {
    match receipt.dependency_kind.as_str() {
        "independent" => "independent surface".to_string(),
        "co_materialized" => {
            if receipt.dependency_surface_ids.is_empty() {
                "co-materialized surface".to_string()
            } else {
                format!(
                    "co-materialized with {}",
                    format_surface_label_list(
                        receipt.dependency_surface_ids.as_slice(),
                        Some(surface_labels),
                    )
                )
            }
        }
        "requires_prior_surface_pass" => {
            if receipt.dependency_surface_ids.is_empty() {
                "requires prior surface".to_string()
            } else {
                format!(
                    "requires prior {}",
                    format_surface_label_list(
                        receipt.dependency_surface_ids.as_slice(),
                        Some(surface_labels),
                    )
                )
            }
        }
        _ => String::new(),
    }
}

pub(crate) fn coverage_receipt_operator_detail_label(
    receipt: &ScraplingOwnedSurfaceCoverageReceipt,
    surface_labels: &BTreeMap<String, String>,
) -> String {
    let state_label = coverage_receipt_state_label(receipt);
    let dependency_label = coverage_receipt_dependency_label(receipt, surface_labels);
    if dependency_label.is_empty() || coverage_receipt_state(receipt) == "blocked_by_prerequisite" {
        state_label
    } else {
        format!("{state_label} | {dependency_label}")
    }
}

pub(crate) fn canonical_scrapling_owned_surface_summary() -> ScraplingOwnedSurfaceSummary {
    let rows = vec![
        row(
            "public_path_traversal",
            "Public Path Traversal",
            "owned",
            "request_native",
            "must_touch",
            "should_pass_some",
            "independent",
            &[],
            &["crawler", "bulk_scraper"],
            "Crawler and bulk-scraper personas must be able to discover and retrieve ordinary public content on the attacked host.",
        ),
        row(
            "challenge_routing",
            "Challenge Routing",
            "owned",
            "request_native",
            "must_touch",
            "mixed_outcomes",
            "independent",
            &[],
            &["crawler", "bulk_scraper", "http_agent"],
            "Request-native Scrapling traffic must encounter Shuma's challenge-selection path rather than silently avoiding it.",
        ),
        row(
            "rate_pressure",
            "Rate Pressure",
            "owned",
            "request_native",
            "must_touch",
            "mixed_outcomes",
            "independent",
            &[],
            &["crawler", "bulk_scraper", "http_agent"],
            "Malicious request-native Scrapling should generate bursty access that can still pass some requests while also triggering rate-based pressure.",
        ),
        row(
            "geo_ip_policy",
            "Geo Or IP Policy",
            "owned",
            "request_native",
            "must_touch",
            "mixed_outcomes",
            "independent",
            &[],
            &["crawler", "bulk_scraper", "http_agent"],
            "Scrapling-owned request-native traffic should traverse the same geo and IP policy surfaces real hostile traffic would encounter.",
        ),
        row(
            "not_a_bot_submit",
            "Not-a-Bot Submit",
            "owned",
            "request_native",
            "must_touch",
            "should_fail",
            "independent",
            &[],
            &["bulk_scraper", "http_agent"],
            "Malicious request-native Scrapling must attempt the Not-a-Bot submit or fail path instead of leaving that defense untouched.",
        ),
        row(
            "puzzle_submit_or_escalation",
            "Puzzle Submit Or Escalation",
            "owned",
            "request_native",
            "must_touch",
            "should_fail",
            "independent",
            &[],
            &["bulk_scraper", "http_agent"],
            "When challenge routing escalates, Scrapling-owned malicious request-native traffic should attempt puzzle submission or puzzle escalation paths and fail honestly.",
        ),
        row(
            "pow_verify_abuse",
            "PoW Verify Abuse",
            "owned",
            "request_native",
            "must_touch",
            "should_fail",
            "independent",
            &[],
            &["http_agent"],
            "Direct-request Scrapling traffic should attempt PoW verification abuse where that surface belongs to the request-native malicious path.",
        ),
        row(
            "tarpit_progress_abuse",
            "Tarpit Progress Abuse",
            "owned",
            "request_native",
            "must_touch",
            "should_fail",
            "independent",
            &[],
            &["http_agent"],
            "If Scrapling owns the full request-native challenge-abuse path, the direct-request persona must also attempt tarpit progress abuse rather than leaving it to the deterministic lane forever.",
        ),
        row(
            "maze_navigation",
            "Maze Navigation",
            "owned",
            "browser_or_stealth",
            "must_touch",
            "should_pass_some",
            "independent",
            &[],
            &["browser_automation", "stealth_browser"],
            "Browser-capable Scrapling personas must attempt truthful maze traversal against the same maze path a hostile browser would encounter.",
        ),
        row(
            "js_verification_execution",
            "JavaScript Verification Execution",
            "owned",
            "browser_or_stealth",
            "must_touch",
            "should_pass_some",
            "independent",
            &[],
            &["browser_automation", "stealth_browser"],
            "Browser-capable Scrapling personas must execute the live JavaScript verification surface rather than leaving it untested.",
        ),
        row(
            "browser_automation_detection",
            "Browser CDP Automation Detection",
            "owned",
            "browser_or_stealth",
            "must_touch",
            "mixed_outcomes",
            "co_materialized",
            &["js_verification_execution"],
            &["browser_automation", "stealth_browser"],
            "Browser-capable Scrapling personas must pressure Shuma's browser-automation detection surface and receipt whether automation was detected or not.",
        ),
        row(
            "cdp_report_ingestion",
            "CDP Report Ingestion",
            "out_of_scope",
            "not_applicable",
            "must_not_touch",
            "outside_scrapling_scope",
            "independent",
            &[],
            &[],
            "A malicious attacker should not self-report CDP detection signals, so this surface is intentionally outside Scrapling ownership.",
        ),
        row(
            "verified_identity_attestation",
            "Verified Identity Attestation",
            "out_of_scope",
            "not_applicable",
            "must_not_touch",
            "outside_scrapling_scope",
            "independent",
            &[],
            &[],
            "Verified-identity attestation is not part of malicious Scrapling behavior and must not be claimed as Scrapling-owned adversary coverage.",
        ),
    ];

    let owned_surface_count = rows
        .iter()
        .filter(|row| row.assignment_status == "owned")
        .count();
    let other_lane_surface_count = rows
        .iter()
        .filter(|row| row.assignment_status == "other_lane")
        .count();
    let out_of_scope_surface_count = rows
        .iter()
        .filter(|row| row.assignment_status == "out_of_scope")
        .count();

    ScraplingOwnedSurfaceSummary {
        schema_version: SCRAPLING_OWNED_SURFACE_SCHEMA_VERSION.to_string(),
        owned_surface_count,
        other_lane_surface_count,
        out_of_scope_surface_count,
        rows,
    }
}

fn row(
    surface_id: &str,
    surface_label: &str,
    assignment_status: &str,
    required_transport: &str,
    interaction_requirement: &str,
    success_contract: &str,
    dependency_kind: &str,
    dependency_surface_ids: &[&str],
    fulfillment_modes: &[&str],
    notes: &str,
) -> ScraplingOwnedSurfaceRow {
    ScraplingOwnedSurfaceRow {
        surface_id: surface_id.to_string(),
        surface_label: surface_label.to_string(),
        assignment_status: assignment_status.to_string(),
        required_transport: required_transport.to_string(),
        interaction_requirement: interaction_requirement.to_string(),
        success_contract: success_contract.to_string(),
        dependency_kind: dependency_kind.to_string(),
        dependency_surface_ids: dependency_surface_ids
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        fulfillment_modes: fulfillment_modes
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        notes: notes.to_string(),
    }
}

fn blocked_prerequisite_surface_ids(
    dependency_kind: &str,
    dependency_surface_ids: &[String],
    required_surface_ids: &BTreeSet<String>,
    satisfied_surface_ids: &BTreeSet<String>,
) -> Vec<String> {
    if dependency_kind != "requires_prior_surface_pass" {
        return Vec::new();
    }
    dependency_surface_ids
        .iter()
        .filter(|surface_id| required_surface_ids.contains(*surface_id))
        .filter(|surface_id| !satisfied_surface_ids.contains(*surface_id))
        .cloned()
        .collect()
}

fn best_surface_observation<'a>(
    success_contract: &str,
    rows: &'a [&ScraplingSurfaceObservationReceipt],
) -> Option<&'a ScraplingSurfaceObservationReceipt> {
    rows.iter()
        .copied()
        .max_by(|left, right| {
            surface_observation_rank(success_contract, left.coverage_status.as_str())
                .cmp(&surface_observation_rank(
                    success_contract,
                    right.coverage_status.as_str(),
                ))
                .then_with(|| left.attempt_count.cmp(&right.attempt_count))
                .then_with(|| {
                    left.sample_response_status
                        .unwrap_or(0)
                        .cmp(&right.sample_response_status.unwrap_or(0))
                })
        })
}

fn surface_observation_rank(success_contract: &str, coverage_status: &str) -> u8 {
    if surface_status_satisfies_contract(success_contract, coverage_status) {
        return 3;
    }
    match coverage_status {
        "pass_observed" | "fail_observed" => 2,
        "transport_error" => 1,
        _ => 0,
    }
}

fn surface_status_satisfies_contract(success_contract: &str, coverage_status: &str) -> bool {
    match success_contract {
        "should_pass_some" => coverage_status == "pass_observed",
        "should_fail" => coverage_status == "fail_observed",
        "mixed_outcomes" => matches!(coverage_status, "pass_observed" | "fail_observed"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_scrapling_owned_surface_summary, coverage_receipt_dependency_label,
        coverage_receipt_operator_detail_label, coverage_receipt_state,
        coverage_receipt_state_label, scrapling_owned_surface_targets,
        scrapling_owned_surface_targets_for_mode, scrapling_owned_surface_targets_for_modes,
        summarize_scrapling_owned_surface_coverage, ScraplingSurfaceObservationReceipt,
        SCRAPLING_OWNED_SURFACE_SCHEMA_VERSION,
    };

    #[test]
    fn canonical_scrapling_owned_surface_summary_reports_owned_and_non_owned_rows_explicitly() {
        let summary = canonical_scrapling_owned_surface_summary();
        assert_eq!(
            summary.schema_version,
            SCRAPLING_OWNED_SURFACE_SCHEMA_VERSION
        );
        assert_eq!(summary.owned_surface_count, 11);
        assert_eq!(summary.other_lane_surface_count, 0);
        assert_eq!(summary.out_of_scope_surface_count, 2);

        let not_a_bot = summary
            .rows
            .iter()
            .find(|row| row.surface_id == "not_a_bot_submit")
            .unwrap();
        assert_eq!(not_a_bot.assignment_status, "owned");
        assert_eq!(not_a_bot.required_transport, "request_native");
        assert_eq!(not_a_bot.interaction_requirement, "must_touch");
        assert_eq!(not_a_bot.success_contract, "should_fail");
        assert_eq!(not_a_bot.dependency_kind, "independent");
        assert!(not_a_bot.dependency_surface_ids.is_empty());
        assert_eq!(
            not_a_bot.fulfillment_modes,
            vec!["bulk_scraper".to_string(), "http_agent".to_string()]
        );

        let maze = summary
            .rows
            .iter()
            .find(|row| row.surface_id == "maze_navigation")
            .unwrap();
        assert_eq!(maze.assignment_status, "owned");
        assert_eq!(maze.required_transport, "browser_or_stealth");
        assert_eq!(maze.interaction_requirement, "must_touch");
        assert_eq!(maze.success_contract, "should_pass_some");
        assert_eq!(maze.dependency_kind, "independent");
        assert!(maze.dependency_surface_ids.is_empty());
        assert_eq!(
            maze.fulfillment_modes,
            vec![
                "browser_automation".to_string(),
                "stealth_browser".to_string()
            ]
        );

        let browser_detection = summary
            .rows
            .iter()
            .find(|row| row.surface_id == "browser_automation_detection")
            .unwrap();
        assert_eq!(
            browser_detection.surface_label,
            "Browser CDP Automation Detection"
        );
        assert_eq!(browser_detection.assignment_status, "owned");
        assert_eq!(browser_detection.dependency_kind, "co_materialized");
        assert_eq!(
            browser_detection.dependency_surface_ids,
            vec!["js_verification_execution".to_string()]
        );

        let verified_identity = summary
            .rows
            .iter()
            .find(|row| row.surface_id == "verified_identity_attestation")
            .unwrap();
        assert_eq!(verified_identity.assignment_status, "out_of_scope");
        assert_eq!(verified_identity.required_transport, "not_applicable");
        assert!(verified_identity.fulfillment_modes.is_empty());
    }

    #[test]
    fn scrapling_owned_surface_target_helpers_match_full_spectrum_persona_contract() {
        assert_eq!(
            scrapling_owned_surface_targets(),
            vec![
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
                "pow_verify_abuse",
                "tarpit_progress_abuse",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
            ]
        );
        assert_eq!(
            scrapling_owned_surface_targets_for_mode("crawler"),
            vec![
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
            ]
        );
        assert_eq!(
            scrapling_owned_surface_targets_for_mode("bulk_scraper"),
            vec![
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
            ]
        );
        assert_eq!(
            scrapling_owned_surface_targets_for_mode("http_agent"),
            vec![
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
                "pow_verify_abuse",
                "tarpit_progress_abuse",
            ]
        );
        assert_eq!(
            scrapling_owned_surface_targets_for_mode("browser_automation"),
            vec![
                "challenge_routing",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
            ]
        );
        assert_eq!(
            scrapling_owned_surface_targets_for_mode("stealth_browser"),
            vec![
                "challenge_routing",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
            ]
        );
        assert!(scrapling_owned_surface_targets_for_mode("unknown_mode").is_empty());
        assert_eq!(
            scrapling_owned_surface_targets_for_modes(&[
                "crawler".to_string(),
                "browser_automation".to_string(),
                "http_agent".to_string()
            ]),
            vec![
                "public_path_traversal",
                "challenge_routing",
                "rate_pressure",
                "geo_ip_policy",
                "not_a_bot_submit",
                "puzzle_submit_or_escalation",
                "pow_verify_abuse",
                "tarpit_progress_abuse",
                "maze_navigation",
                "js_verification_execution",
                "browser_automation_detection",
            ]
        );
    }

    #[test]
    fn owned_surface_coverage_summary_marks_all_required_surfaces_covered_when_contracts_are_met() {
        let summary = summarize_scrapling_owned_surface_coverage(
            &[
                "bulk_scraper".to_string(),
                "browser_automation".to_string(),
                "http_agent".to_string(),
            ],
            &[
                ScraplingSurfaceObservationReceipt {
                    surface_id: "public_path_traversal".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 3,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/catalog?page=1".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "challenge_routing".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "rate_pressure".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "geo_ip_policy".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "not_a_bot_submit".to_string(),
                    coverage_status: "fail_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "POST".to_string(),
                    sample_request_path: "/challenge/not-a-bot-checkbox".to_string(),
                    sample_response_status: Some(400),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "puzzle_submit_or_escalation".to_string(),
                    coverage_status: "fail_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "POST".to_string(),
                    sample_request_path: "/challenge/puzzle".to_string(),
                    sample_response_status: Some(400),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "pow_verify_abuse".to_string(),
                    coverage_status: "fail_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "POST".to_string(),
                    sample_request_path: "/pow/verify".to_string(),
                    sample_response_status: Some(400),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "tarpit_progress_abuse".to_string(),
                    coverage_status: "fail_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "POST".to_string(),
                    sample_request_path: "/tarpit/progress".to_string(),
                    sample_response_status: Some(400),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "maze_navigation".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/maze/start".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "js_verification_execution".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/pow".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "browser_automation_detection".to_string(),
                    coverage_status: "fail_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/pow".to_string(),
                    sample_response_status: Some(200),
                },
            ],
        )
        .expect("coverage summary");

        assert_eq!(summary.overall_status, "covered");
        assert!(summary.blocking_surface_ids.is_empty());
        assert_eq!(summary.canonical_surface_ids.len(), 13);
        assert!(summary
            .canonical_surface_ids
            .contains(&"cdp_report_ingestion".to_string()));
        assert!(summary
            .canonical_surface_ids
            .contains(&"verified_identity_attestation".to_string()));
        assert_eq!(
            summary
                .surface_labels
                .get("browser_automation_detection")
                .map(String::as_str),
            Some("Browser CDP Automation Detection")
        );
        assert_eq!(
            summary
                .surface_labels
                .get("cdp_report_ingestion")
                .map(String::as_str),
            Some("CDP Report Ingestion")
        );
        assert_eq!(
            summary
                .surface_labels
                .get("verified_identity_attestation")
                .map(String::as_str),
            Some("Verified Identity Attestation")
        );
        assert_eq!(summary.required_surface_ids.len(), 11);
        assert_eq!(summary.satisfied_surface_ids.len(), 11);
        assert!(summary
            .receipts
            .iter()
            .all(|receipt| receipt.satisfied));
    }

    #[test]
    fn owned_surface_coverage_summary_leaves_missing_or_wrong_outcomes_blocking() {
        let summary = summarize_scrapling_owned_surface_coverage(
            &["stealth_browser".to_string(), "http_agent".to_string()],
            &[
                ScraplingSurfaceObservationReceipt {
                    surface_id: "challenge_routing".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "pow_verify_abuse".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "POST".to_string(),
                    sample_request_path: "/pow/verify".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "browser_automation_detection".to_string(),
                    coverage_status: "transport_error".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/pow".to_string(),
                    sample_response_status: None,
                },
            ],
        )
        .expect("coverage summary");

        assert_eq!(summary.overall_status, "partial");
        assert_eq!(summary.canonical_surface_ids.len(), 13);
        assert!(summary
            .canonical_surface_ids
            .contains(&"cdp_report_ingestion".to_string()));
        assert!(summary
            .canonical_surface_ids
            .contains(&"verified_identity_attestation".to_string()));
        assert_eq!(
            summary
                .surface_labels
                .get("browser_automation_detection")
                .map(String::as_str),
            Some("Browser CDP Automation Detection")
        );
        assert!(summary
            .blocking_surface_ids
            .contains(&"pow_verify_abuse".to_string()));
        assert!(summary
            .blocking_surface_ids
            .contains(&"not_a_bot_submit".to_string()));
        assert!(summary
            .blocking_surface_ids
            .contains(&"maze_navigation".to_string()));
        let pow = summary
            .receipts
            .iter()
            .find(|receipt| receipt.surface_id == "pow_verify_abuse")
            .expect("pow receipt");
        assert_eq!(pow.coverage_status, "pass_observed");
        assert!(!pow.satisfied);
        let detection = summary
            .receipts
            .iter()
            .find(|receipt| receipt.surface_id == "browser_automation_detection")
            .expect("browser detection receipt");
        assert_eq!(detection.coverage_status, "transport_error");
        assert!(!detection.satisfied);
        assert_eq!(coverage_receipt_state(detection), "attempted_blocked");
        assert_eq!(coverage_receipt_state_label(detection), "attempted and blocked");
        assert_eq!(
            coverage_receipt_dependency_label(detection, &summary.surface_labels),
            "co-materialized with JavaScript Verification Execution"
        );
        assert_eq!(
            coverage_receipt_operator_detail_label(detection, &summary.surface_labels),
            "attempted and blocked | co-materialized with JavaScript Verification Execution"
        );
        let maze = summary
            .receipts
            .iter()
            .find(|receipt| receipt.surface_id == "maze_navigation")
            .expect("maze receipt");
        assert_eq!(maze.coverage_status, "unavailable");
        assert_eq!(maze.attempt_count, 0);
        assert_eq!(coverage_receipt_state(maze), "unreached");
        assert_eq!(coverage_receipt_state_label(maze), "required but unreached");
        assert_eq!(
            coverage_receipt_dependency_label(maze, &summary.surface_labels),
            "independent surface"
        );
        assert_eq!(
            coverage_receipt_operator_detail_label(maze, &summary.surface_labels),
            "required but unreached | independent surface"
        );
    }
}
