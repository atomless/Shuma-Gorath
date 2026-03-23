#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::non_human_classification::NonHumanClassificationReceipt;
use super::non_human_lane_fulfillment::{
    canonical_non_human_lane_fulfillment, NonHumanLaneFulfillmentRow,
};
use super::replay_promotion::ReplayPromotionSummary;

pub(crate) const NON_HUMAN_COVERAGE_SCHEMA_VERSION: &str = "non_human_coverage_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanCoverageReceipt {
    pub category_id: String,
    pub coverage_status: String,
    pub coverage_basis: String,
    pub adversary_sim_receipt_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanCoverageSummary {
    pub schema_version: String,
    pub overall_status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_category_ids: Vec<String>,
    pub mapped_category_count: usize,
    pub gap_category_count: usize,
    pub covered_category_count: usize,
    pub partial_category_count: usize,
    pub stale_category_count: usize,
    pub unavailable_category_count: usize,
    pub uncovered_category_count: usize,
    pub receipts: Vec<NonHumanCoverageReceipt>,
}

impl NonHumanCoverageSummary {
    pub(crate) fn compact_for_benchmark(&self) -> Self {
        let mut compact = self.clone();
        compact.receipts.clear();
        compact
    }

    pub(crate) fn mapped_categories_are_covered(&self) -> bool {
        self.overall_status == "covered"
    }

    pub(crate) fn protected_tuning_blockers(
        &self,
        replay_promotion: &ReplayPromotionSummary,
    ) -> Vec<String> {
        let mut blockers = Vec::new();
        if !self.mapped_categories_are_covered() {
            blockers.push("non_human_category_coverage_not_ready".to_string());
            blockers.extend(self.blocking_reasons.iter().cloned());
        }
        if !replay_promotion.tuning_eligible {
            blockers.push("protected_tuning_evidence_not_ready".to_string());
            blockers.extend(replay_promotion.eligibility_blockers.iter().cloned());
        }
        blockers
    }
}

pub(crate) fn summarize_non_human_coverage(
    receipts: &[NonHumanClassificationReceipt],
) -> NonHumanCoverageSummary {
    let fulfillment = canonical_non_human_lane_fulfillment();
    let mut coverage_receipts = Vec::new();
    let mut mapped_category_count = 0usize;
    let mut gap_category_count = 0usize;
    let mut covered_category_count = 0usize;
    let mut partial_category_count = 0usize;
    let mut stale_category_count = 0usize;
    let mut unavailable_category_count = 0usize;
    let mut uncovered_category_count = 0usize;
    let mut blocking_reasons = BTreeSet::new();
    let mut blocking_category_ids = Vec::new();

    for row in fulfillment.rows {
        let matching_live: Vec<&NonHumanClassificationReceipt> = receipts
            .iter()
            .filter(|receipt| {
                receipt.category_id == row.category_id && receipt.traffic_origin == "live"
            })
            .collect();
        let matching_sim: Vec<&NonHumanClassificationReceipt> = receipts
            .iter()
            .filter(|receipt| {
                receipt.category_id == row.category_id
                    && receipt.traffic_origin == "adversary_sim"
            })
            .collect();
        let (coverage_status, coverage_basis) =
            derive_coverage_status(&row, matching_live.as_slice(), matching_sim.as_slice());

        match coverage_status {
            "covered" => covered_category_count += 1,
            "partial" => partial_category_count += 1,
            "stale" => stale_category_count += 1,
            "unavailable" => unavailable_category_count += 1,
            "uncovered" => uncovered_category_count += 1,
            _ => {}
        }

        if row.assignment_status == "mapped" {
            mapped_category_count += 1;
            if coverage_status != "covered" {
                blocking_category_ids.push(row.category_id.clone());
                match coverage_status {
                    "partial" => {
                        blocking_reasons.insert(
                            "mapped_categories_have_partial_coverage".to_string(),
                        );
                    }
                    "stale" => {
                        blocking_reasons
                            .insert("mapped_categories_have_stale_coverage".to_string());
                    }
                    "unavailable" => {
                        blocking_reasons.insert(
                            "mapped_categories_have_unavailable_coverage".to_string(),
                        );
                    }
                    "uncovered" => {
                        blocking_reasons
                            .insert("mapped_categories_have_uncovered_coverage".to_string());
                    }
                    _ => {}
                }
            }
        } else {
            gap_category_count += 1;
        }

        coverage_receipts.push(NonHumanCoverageReceipt {
            category_id: row.category_id,
            coverage_status: coverage_status.to_string(),
            coverage_basis: coverage_basis.to_string(),
            adversary_sim_receipt_count: matching_sim.len(),
        });
    }

    let overall_status = if mapped_category_count == 0 {
        "unavailable".to_string()
    } else if blocking_category_ids.is_empty() {
        "covered".to_string()
    } else if stale_category_count > 0 {
        "stale".to_string()
    } else if covered_category_count > 0 || partial_category_count > 0 {
        "partial".to_string()
    } else if unavailable_category_count > 0 {
        "unavailable".to_string()
    } else {
        "uncovered".to_string()
    };

    NonHumanCoverageSummary {
        schema_version: NON_HUMAN_COVERAGE_SCHEMA_VERSION.to_string(),
        overall_status,
        blocking_reasons: blocking_reasons.into_iter().collect(),
        blocking_category_ids,
        mapped_category_count,
        gap_category_count,
        covered_category_count,
        partial_category_count,
        stale_category_count,
        unavailable_category_count,
        uncovered_category_count,
        receipts: coverage_receipts,
    }
}

fn derive_coverage_status(
    row: &NonHumanLaneFulfillmentRow,
    _matching_live: &[&NonHumanClassificationReceipt],
    matching_sim: &[&NonHumanClassificationReceipt],
) -> (&'static str, &'static str) {
    if row.assignment_status != "mapped" {
        return ("uncovered", "assignment_gap");
    }

    if matching_sim.iter().any(|receipt| {
        receipt.assignment_status == "classified" && receipt.degradation_status == "current"
    }) {
        return ("covered", "receipt_backed_current");
    }

    if matching_sim
        .iter()
        .any(|receipt| receipt.assignment_status != "classified")
    {
        return ("partial", "receipt_backed_insufficient_evidence");
    }

    if !matching_sim.is_empty() {
        return ("stale", "receipt_backed_degraded");
    }
    ("unavailable", "intent_only")
}

#[cfg(test)]
mod tests {
    use super::summarize_non_human_coverage;
    use crate::observability::non_human_classification::NonHumanClassificationReceipt;

    fn receipt(
        traffic_origin: &str,
        category_id: &str,
        assignment_status: &str,
        degradation_status: &str,
    ) -> NonHumanClassificationReceipt {
        NonHumanClassificationReceipt {
            traffic_origin: traffic_origin.to_string(),
            measurement_scope: "ingress_primary".to_string(),
            execution_mode: "enforced".to_string(),
            lane: "declared_crawler".to_string(),
            category_id: category_id.to_string(),
            category_label: "label".to_string(),
            assignment_status: assignment_status.to_string(),
            exactness: "exact".to_string(),
            basis: "observed".to_string(),
            degradation_status: degradation_status.to_string(),
            total_requests: 4,
            forwarded_requests: 3,
            short_circuited_requests: 1,
            evidence_references: vec!["recent_sim_runs:simrun-001".to_string()],
        }
    }

    #[test]
    fn coverage_summary_distinguishes_covered_unavailable_and_gap_categories() {
        let summary = summarize_non_human_coverage(&[
            receipt("adversary_sim", "indexing_bot", "classified", "current"),
            receipt("live", "verified_beneficial_bot", "classified", "current"),
        ]);

        assert_eq!(summary.schema_version, "non_human_coverage_v1");
        assert_eq!(summary.overall_status, "partial");
        assert_eq!(summary.mapped_category_count, 6);
        assert_eq!(summary.gap_category_count, 2);
        assert_eq!(summary.covered_category_count, 1);
        assert_eq!(summary.unavailable_category_count, 5);
        assert_eq!(summary.uncovered_category_count, 2);
        assert!(summary
            .blocking_reasons
            .contains(&"mapped_categories_have_unavailable_coverage".to_string()));
        assert!(summary
            .blocking_category_ids
            .contains(&"ai_scraper_bot".to_string()));

        let indexing = summary
            .receipts
            .iter()
            .find(|row| row.category_id == "indexing_bot")
            .expect("indexing receipt");
        assert_eq!(indexing.coverage_status, "covered");
        assert_eq!(indexing.coverage_basis, "receipt_backed_current");

        let browser = summary
            .receipts
            .iter()
            .find(|row| row.category_id == "automated_browser")
            .expect("browser receipt");
        assert_eq!(browser.coverage_status, "unavailable");
        assert_eq!(browser.coverage_basis, "intent_only");

        let beneficial = summary
            .receipts
            .iter()
            .find(|row| row.category_id == "verified_beneficial_bot")
            .expect("beneficial receipt");
        assert_eq!(beneficial.coverage_status, "uncovered");
        assert_eq!(beneficial.coverage_basis, "assignment_gap");
    }

    #[test]
    fn coverage_summary_marks_degraded_receipts_stale() {
        let summary = summarize_non_human_coverage(&[receipt(
            "adversary_sim",
            "indexing_bot",
            "classified",
            "degraded",
        )]);

        assert_eq!(summary.overall_status, "stale");
        assert_eq!(summary.stale_category_count, 1);
        assert!(summary
            .blocking_reasons
            .contains(&"mapped_categories_have_stale_coverage".to_string()));
        let indexing = summary
            .receipts
            .iter()
            .find(|row| row.category_id == "indexing_bot")
            .expect("indexing receipt");
        assert_eq!(indexing.coverage_status, "stale");
        assert_eq!(indexing.coverage_basis, "receipt_backed_degraded");
    }

    #[test]
    fn coverage_summary_marks_scrapling_request_native_categories_covered_when_receipts_exist() {
        let summary = summarize_non_human_coverage(&[
            receipt("adversary_sim", "indexing_bot", "classified", "current"),
            receipt("adversary_sim", "ai_scraper_bot", "classified", "current"),
            receipt("adversary_sim", "http_agent", "classified", "current"),
            receipt("live", "verified_beneficial_bot", "classified", "current"),
        ]);

        assert_eq!(summary.overall_status, "partial");
        assert_eq!(summary.covered_category_count, 3);
        assert!(summary
            .receipts
            .iter()
            .any(|row| row.category_id == "ai_scraper_bot" && row.coverage_status == "covered"));
        assert!(summary
            .receipts
            .iter()
            .any(|row| row.category_id == "http_agent" && row.coverage_status == "covered"));
    }
}
