#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

pub(crate) const SCRAPLING_OWNED_DEFENSE_SURFACE_SCHEMA_VERSION: &str =
    "sim-scrapling-owned-defense-surfaces.v1";
pub(crate) const SCRAPLING_OWNED_DEFENSE_SURFACE_COVERAGE_SCHEMA_VERSION: &str =
    "scrapling_owned_defense_surface_coverage_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedDefenseSurfaceRow {
    pub surface_id: String,
    pub runtime_requirement: String,
    pub interaction_requirement: String,
    pub success_contract: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedDefenseSurfaceCatalog {
    pub schema_version: String,
    pub rows: Vec<ScraplingOwnedDefenseSurfaceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedDefenseSurfaceCoverageReceipt {
    pub surface_id: String,
    pub runtime_requirement: String,
    pub interaction_requirement: String,
    pub success_contract: String,
    pub coverage_status: String,
    pub coverage_basis: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub gap_assignment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ScraplingOwnedDefenseSurfaceCoverageSummary {
    pub schema_version: String,
    pub overall_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_surface_ids: Vec<String>,
    pub owned_surface_count: usize,
    pub recent_scrapling_run_count: usize,
    pub covered_surface_count: usize,
    pub uncovered_surface_count: usize,
    pub receipts: Vec<ScraplingOwnedDefenseSurfaceCoverageReceipt>,
}

impl Default for ScraplingOwnedDefenseSurfaceCoverageSummary {
    fn default() -> Self {
        Self {
            schema_version: SCRAPLING_OWNED_DEFENSE_SURFACE_COVERAGE_SCHEMA_VERSION.to_string(),
            overall_status: "unavailable".to_string(),
            blocking_reasons: Vec::new(),
            blocking_surface_ids: Vec::new(),
            owned_surface_count: 0,
            recent_scrapling_run_count: 0,
            covered_surface_count: 0,
            uncovered_surface_count: 0,
            receipts: Vec::new(),
        }
    }
}

impl ScraplingOwnedDefenseSurfaceCoverageSummary {
    pub(crate) fn owned_surfaces_are_covered(&self) -> bool {
        self.overall_status == "covered"
    }
}

pub(crate) fn canonical_scrapling_owned_defense_surfaces() -> ScraplingOwnedDefenseSurfaceCatalog {
    ScraplingOwnedDefenseSurfaceCatalog {
        schema_version: SCRAPLING_OWNED_DEFENSE_SURFACE_SCHEMA_VERSION.to_string(),
        rows: vec![
            surface_row(
                "honeypot",
                "request_native",
                "must_touch",
                "must_fail_or_escalate",
                "Request-native Scrapling crawler and scraper personas should be expected to encounter hostile lure paths and trip them rather than cleanly bypass them in this first tranche.",
            ),
            surface_row(
                "rate_limit",
                "request_native",
                "must_touch",
                "must_touch",
                "Request-native Scrapling personas must be able to generate the pressure that reaches rate enforcement, but passing the limiter is not required by this first owned-surface contract.",
            ),
            surface_row(
                "geo_ip_policy",
                "request_native",
                "must_touch",
                "must_touch",
                "Request-native Scrapling must be able to hit geo and IP policy routing decisions so Shuma can observe challenge or block behavior against hostile request-native traffic.",
            ),
            surface_row(
                "challenge_routing",
                "request_native",
                "must_touch",
                "must_touch",
                "Scrapling-owned request-native traffic must be able to trigger challenge-family routing decisions before later browser or stealth adoption is considered.",
            ),
            surface_row(
                "not_a_bot",
                "request_native",
                "must_touch",
                "must_fail_or_escalate",
                "The first attacker-faithful request-native contract requires explicit hostile fail or escalate interaction with not-a-bot flows, not a claim that Scrapling can already solve them.",
            ),
            surface_row(
                "challenge_puzzle",
                "request_native",
                "must_touch",
                "must_fail_or_escalate",
                "Puzzle interaction is owned at the request-native hostile-failure level first. Clean puzzle solving should remain a later black-box proof obligation only if an outside attacker could plausibly do it with public host knowledge.",
            ),
            surface_row(
                "proof_of_work",
                "request_native",
                "must_touch",
                "must_fail_or_escalate",
                "PoW coverage is owned first as verify-abuse and hostile-failure interaction. Passing PoW should only be required later if Shuma proves that a black-box attacker-faithful Scrapling path can do so.",
            ),
        ],
    }
}

pub(crate) fn summarize_scrapling_owned_defense_surface_coverage(
    recent_runs: &[crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun],
) -> ScraplingOwnedDefenseSurfaceCoverageSummary {
    let catalog = canonical_scrapling_owned_defense_surfaces();
    let scrapling_runs: Vec<_> = recent_runs
        .iter()
        .filter(|run| run.lane == "scrapling_traffic")
        .collect();
    let mut observed_surface_ids = std::collections::BTreeSet::new();
    for run in &scrapling_runs {
        for surface_id in &run.observed_defense_keys {
            observed_surface_ids.insert(surface_id.clone());
        }
    }

    let mut receipts = Vec::new();
    let mut blocking_surface_ids = Vec::new();
    let mut covered_surface_count = 0usize;
    let mut uncovered_surface_count = 0usize;
    let has_recent_scrapling_runs = !scrapling_runs.is_empty();

    for row in catalog.rows {
        let covered = observed_surface_ids.contains(&row.surface_id);
        if covered {
            covered_surface_count += 1;
        } else {
            uncovered_surface_count += 1;
            blocking_surface_ids.push(row.surface_id.clone());
        }
        receipts.push(ScraplingOwnedDefenseSurfaceCoverageReceipt {
            surface_id: row.surface_id.clone(),
            runtime_requirement: row.runtime_requirement.clone(),
            interaction_requirement: row.interaction_requirement.clone(),
            success_contract: row.success_contract.clone(),
            coverage_status: if covered {
                "covered".to_string()
            } else if has_recent_scrapling_runs {
                "uncovered".to_string()
            } else {
                "unavailable".to_string()
            },
            coverage_basis: if covered {
                "recent_sim_run_observed_defense_keys".to_string()
            } else if has_recent_scrapling_runs {
                "recent_sim_run_receipts_missing_surface".to_string()
            } else {
                "no_recent_scrapling_runs".to_string()
            },
            gap_assignment: if covered {
                String::new()
            } else {
                default_gap_assignment_for_surface(row.surface_id.as_str()).to_string()
            },
        });
    }

    let overall_status = if receipts.is_empty() || !has_recent_scrapling_runs {
        "unavailable"
    } else if uncovered_surface_count == 0 {
        "covered"
    } else if covered_surface_count > 0 {
        "partial"
    } else {
        "uncovered"
    };

    let blocking_reasons = match overall_status {
        "covered" => Vec::new(),
        "unavailable" => vec!["scrapling_recent_sim_runs_not_observed".to_string()],
        _ => vec!["scrapling_owned_surfaces_missing_receipts".to_string()],
    };

    ScraplingOwnedDefenseSurfaceCoverageSummary {
        schema_version: SCRAPLING_OWNED_DEFENSE_SURFACE_COVERAGE_SCHEMA_VERSION.to_string(),
        overall_status: overall_status.to_string(),
        blocking_reasons,
        blocking_surface_ids,
        owned_surface_count: receipts.len(),
        recent_scrapling_run_count: scrapling_runs.len(),
        covered_surface_count,
        uncovered_surface_count,
        receipts,
    }
}

fn surface_row(
    surface_id: &str,
    runtime_requirement: &str,
    interaction_requirement: &str,
    success_contract: &str,
    notes: &str,
) -> ScraplingOwnedDefenseSurfaceRow {
    ScraplingOwnedDefenseSurfaceRow {
        surface_id: surface_id.to_string(),
        runtime_requirement: runtime_requirement.to_string(),
        interaction_requirement: interaction_requirement.to_string(),
        success_contract: success_contract.to_string(),
        notes: notes.to_string(),
    }
}

fn default_gap_assignment_for_surface(surface_id: &str) -> &'static str {
    match surface_id {
        "geo_ip_policy" => "request_native_proxy_or_source_ip_diversification",
        _ => "unassigned_gap",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_scrapling_owned_defense_surfaces,
        summarize_scrapling_owned_defense_surface_coverage,
    };
    use crate::observability::operator_snapshot_live_traffic::OperatorSnapshotRecentSimRun;

    fn recent_run(observed_defense_keys: Vec<&str>) -> OperatorSnapshotRecentSimRun {
        OperatorSnapshotRecentSimRun {
            run_id: "simrun-scrapling-http-agent".to_string(),
            lane: "scrapling_traffic".to_string(),
            profile: "scrapling_runtime_lane".to_string(),
            observed_fulfillment_modes: vec!["http_agent".to_string()],
            observed_category_ids: vec!["http_agent".to_string()],
            observed_defense_keys: observed_defense_keys
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
            first_ts: 1_700_000_000,
            last_ts: 1_700_000_100,
            monitoring_event_count: 8,
            defense_delta_count: 6,
            ban_outcome_count: 1,
        }
    }

    #[test]
    fn canonical_scrapling_owned_defense_surface_catalog_stays_frozen() {
        let catalog = canonical_scrapling_owned_defense_surfaces();
        assert_eq!(catalog.schema_version, "sim-scrapling-owned-defense-surfaces.v1");
        assert_eq!(catalog.rows.len(), 7);
        assert!(catalog.rows.iter().any(|row| row.surface_id == "geo_ip_policy"));
        assert!(catalog.rows.iter().all(|row| row.runtime_requirement == "request_native"));
    }

    #[test]
    fn scrapling_owned_surface_coverage_summary_makes_geo_gap_explicit() {
        let summary = summarize_scrapling_owned_defense_surface_coverage(&[recent_run(vec![
            "challenge_routing",
            "rate_limit",
            "honeypot",
            "not_a_bot",
            "challenge_puzzle",
            "proof_of_work",
        ])]);

        assert_eq!(summary.schema_version, "scrapling_owned_defense_surface_coverage_v1");
        assert_eq!(summary.overall_status, "partial");
        assert_eq!(summary.recent_scrapling_run_count, 1);
        assert_eq!(summary.covered_surface_count, 6);
        assert_eq!(summary.uncovered_surface_count, 1);
        assert_eq!(summary.blocking_surface_ids, vec!["geo_ip_policy".to_string()]);
        assert!(summary
            .blocking_reasons
            .contains(&"scrapling_owned_surfaces_missing_receipts".to_string()));
        let geo = summary
            .receipts
            .iter()
            .find(|row| row.surface_id == "geo_ip_policy")
            .expect("geo surface receipt");
        assert_eq!(geo.coverage_status, "uncovered");
        assert_eq!(
            geo.gap_assignment,
            "request_native_proxy_or_source_ip_diversification"
        );
    }

    #[test]
    fn scrapling_owned_surface_coverage_summary_reports_complete_coverage() {
        let summary = summarize_scrapling_owned_defense_surface_coverage(&[recent_run(vec![
            "challenge_routing",
            "rate_limit",
            "honeypot",
            "not_a_bot",
            "challenge_puzzle",
            "proof_of_work",
            "geo_ip_policy",
        ])]);

        assert!(summary.owned_surfaces_are_covered());
        assert_eq!(summary.overall_status, "covered");
        assert!(summary.blocking_surface_ids.is_empty());
        assert_eq!(summary.covered_surface_count, 7);
    }
}
