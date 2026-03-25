#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const SCRAPLING_OWNED_SURFACE_SCHEMA_VERSION: &str =
    "scrapling_owned_surface_contract_v1";

const SCRAPLING_OWNED_SURFACE_TARGETS: [&str; 8] = [
    "public_path_traversal",
    "challenge_routing",
    "rate_pressure",
    "geo_ip_policy",
    "not_a_bot_submit",
    "puzzle_submit_or_escalation",
    "pow_verify_abuse",
    "tarpit_progress_abuse",
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedSurfaceRow {
    pub surface_id: String,
    pub surface_label: String,
    pub assignment_status: String,
    pub required_transport: String,
    pub interaction_requirement: String,
    pub success_contract: String,
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
    pub coverage_status: String,
    pub satisfied: bool,
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

    let mut receipts = Vec::new();
    let mut satisfied_surface_ids = Vec::new();
    let mut blocking_surface_ids = Vec::new();

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

        if satisfied {
            satisfied_surface_ids.push(surface_id.clone());
        } else {
            blocking_surface_ids.push(surface_id.clone());
        }
        receipts.push(ScraplingOwnedSurfaceCoverageReceipt {
            surface_id: surface_id.clone(),
            success_contract: row.success_contract.clone(),
            coverage_status,
            satisfied,
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
        required_surface_ids,
        satisfied_surface_ids,
        blocking_surface_ids,
        receipts,
    })
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
            &["http_agent"],
            "If Scrapling owns the full request-native challenge-abuse path, the direct-request persona must also attempt tarpit progress abuse rather than leaving it to the deterministic lane forever.",
        ),
        row(
            "maze_navigation",
            "Maze Navigation",
            "other_lane",
            "browser_or_stealth",
            "must_not_touch",
            "outside_scrapling_scope",
            &[],
            "Meaningful maze traversal remains a browser-class interaction and belongs to a browser-capable lane unless reassigned explicitly later.",
        ),
        row(
            "js_verification_execution",
            "JavaScript Verification Execution",
            "other_lane",
            "browser_or_stealth",
            "must_not_touch",
            "outside_scrapling_scope",
            &[],
            "Executing JavaScript verification truthfully is browser-class behavior, not current request-native Scrapling ownership.",
        ),
        row(
            "browser_automation_detection",
            "Browser Automation Detection",
            "other_lane",
            "browser_or_stealth",
            "must_not_touch",
            "outside_scrapling_scope",
            &[],
            "Browser automation and anti-automation detection belong to browser-capable adversary lanes, not the request-native Scrapling lane.",
        ),
        row(
            "cdp_report_ingestion",
            "CDP Report Ingestion",
            "out_of_scope",
            "not_applicable",
            "must_not_touch",
            "outside_scrapling_scope",
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
        fulfillment_modes: fulfillment_modes
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        notes: notes.to_string(),
    }
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
        canonical_scrapling_owned_surface_summary, scrapling_owned_surface_targets,
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
        assert_eq!(summary.owned_surface_count, 8);
        assert_eq!(summary.other_lane_surface_count, 3);
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
        assert_eq!(
            not_a_bot.fulfillment_modes,
            vec!["bulk_scraper".to_string(), "http_agent".to_string()]
        );

        let maze = summary
            .rows
            .iter()
            .find(|row| row.surface_id == "maze_navigation")
            .unwrap();
        assert_eq!(maze.assignment_status, "other_lane");
        assert_eq!(maze.required_transport, "browser_or_stealth");
        assert_eq!(maze.interaction_requirement, "must_not_touch");
        assert_eq!(maze.success_contract, "outside_scrapling_scope");

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
    fn scrapling_owned_surface_target_helpers_match_request_native_persona_contract() {
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
        assert!(scrapling_owned_surface_targets_for_mode("unknown_mode").is_empty());
        assert_eq!(
            scrapling_owned_surface_targets_for_modes(&[
                "crawler".to_string(),
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
            ]
        );
    }

    #[test]
    fn owned_surface_coverage_summary_marks_all_required_surfaces_covered_when_contracts_are_met() {
        let summary = summarize_scrapling_owned_surface_coverage(
            &["bulk_scraper".to_string(), "http_agent".to_string()],
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
                    sample_request_path: "/sim/public/search?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "rate_pressure".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/search?q=test".to_string(),
                    sample_response_status: Some(200),
                },
                ScraplingSurfaceObservationReceipt {
                    surface_id: "geo_ip_policy".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 2,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/search?q=test".to_string(),
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
            ],
        )
        .expect("coverage summary");

        assert_eq!(summary.overall_status, "covered");
        assert!(summary.blocking_surface_ids.is_empty());
        assert_eq!(summary.required_surface_ids.len(), 8);
        assert_eq!(summary.satisfied_surface_ids.len(), 8);
        assert!(summary
            .receipts
            .iter()
            .all(|receipt| receipt.satisfied));
    }

    #[test]
    fn owned_surface_coverage_summary_leaves_missing_or_wrong_outcomes_blocking() {
        let summary = summarize_scrapling_owned_surface_coverage(
            &["http_agent".to_string()],
            &[
                ScraplingSurfaceObservationReceipt {
                    surface_id: "challenge_routing".to_string(),
                    coverage_status: "pass_observed".to_string(),
                    attempt_count: 1,
                    sample_request_method: "GET".to_string(),
                    sample_request_path: "/sim/public/search?q=test".to_string(),
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
            ],
        )
        .expect("coverage summary");

        assert_eq!(summary.overall_status, "partial");
        assert!(summary
            .blocking_surface_ids
            .contains(&"pow_verify_abuse".to_string()));
        assert!(summary
            .blocking_surface_ids
            .contains(&"not_a_bot_submit".to_string()));
        let pow = summary
            .receipts
            .iter()
            .find(|receipt| receipt.surface_id == "pow_verify_abuse")
            .expect("pow receipt");
        assert_eq!(pow.coverage_status, "pass_observed");
        assert!(!pow.satisfied);
    }
}
