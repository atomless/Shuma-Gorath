#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, NonHumanCategoryId,
};

pub(crate) const NON_HUMAN_LANE_FULFILLMENT_SCHEMA_VERSION: &str =
    "non_human_lane_fulfillment_v1";

const SCRAPLING_CATEGORY_TARGETS: [&str; 1] = ["indexing_bot"];
const LLM_BROWSER_CATEGORY_TARGETS: [&str; 3] = [
    "automated_browser",
    "browser_agent",
    "agent_on_behalf_of_human",
];
const LLM_REQUEST_CATEGORY_TARGETS: [&str; 2] = ["http_agent", "ai_scraper_bot"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanLaneFulfillmentRow {
    pub category_id: String,
    pub category_label: String,
    pub assignment_status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runtime_lane: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fulfillment_mode: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanLaneFulfillmentSummary {
    pub schema_version: String,
    pub mapped_category_count: usize,
    pub gap_category_count: usize,
    pub rows: Vec<NonHumanLaneFulfillmentRow>,
}

pub(crate) fn scrapling_category_targets() -> Vec<String> {
    SCRAPLING_CATEGORY_TARGETS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(crate) fn llm_category_targets_for_mode(mode: &str) -> Vec<String> {
    match mode {
        "browser_mode" => LLM_BROWSER_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "request_mode" => LLM_REQUEST_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn canonical_non_human_lane_fulfillment() -> NonHumanLaneFulfillmentSummary {
    let taxonomy = canonical_non_human_taxonomy();
    let mut rows = Vec::new();
    let mut mapped_category_count = 0usize;
    let mut gap_category_count = 0usize;

    for descriptor in taxonomy.categories {
        let (assignment_status, runtime_lane, fulfillment_mode, notes) =
            lane_assignment_for_category(descriptor.category_id);
        if assignment_status == "mapped" {
            mapped_category_count += 1;
        } else {
            gap_category_count += 1;
        }
        rows.push(NonHumanLaneFulfillmentRow {
            category_id: descriptor.category_id.as_str().to_string(),
            category_label: descriptor.label,
            assignment_status: assignment_status.to_string(),
            runtime_lane: runtime_lane.to_string(),
            fulfillment_mode: fulfillment_mode.to_string(),
            notes: notes.to_string(),
        });
    }

    NonHumanLaneFulfillmentSummary {
        schema_version: NON_HUMAN_LANE_FULFILLMENT_SCHEMA_VERSION.to_string(),
        mapped_category_count,
        gap_category_count,
        rows,
    }
}

fn lane_assignment_for_category(
    category_id: NonHumanCategoryId,
) -> (&'static str, &'static str, &'static str, &'static str) {
    match category_id {
        NonHumanCategoryId::IndexingBot => (
            "mapped",
            "scrapling_traffic",
            "scrapling_worker",
            "Shared-host Scrapling is the intended crawler/indexing fulfillment lane.",
        ),
        NonHumanCategoryId::AiScraperBot => (
            "mapped",
            "bot_red_team",
            "request_mode",
            "Bounded LLM request mode is the intended fulfillment lane for direct request scraping pressure.",
        ),
        NonHumanCategoryId::AutomatedBrowser => (
            "mapped",
            "bot_red_team",
            "browser_mode",
            "Bounded LLM browser mode is the intended fulfillment lane for browser automation pressure.",
        ),
        NonHumanCategoryId::HttpAgent => (
            "mapped",
            "bot_red_team",
            "request_mode",
            "Bounded LLM request mode is the intended fulfillment lane for direct HTTP agent behavior.",
        ),
        NonHumanCategoryId::BrowserAgent => (
            "mapped",
            "bot_red_team",
            "browser_mode",
            "Bounded LLM browser mode is the intended fulfillment lane for multi-step browser-agent behavior.",
        ),
        NonHumanCategoryId::AgentOnBehalfOfHuman => (
            "mapped",
            "bot_red_team",
            "browser_mode",
            "Bounded LLM browser mode is the intended initial fulfillment lane for delegated browser agents.",
        ),
        NonHumanCategoryId::VerifiedBeneficialBot => (
            "gap",
            "",
            "",
            "No current adversary lane credibly simulates verified beneficial bot traffic yet.",
        ),
        NonHumanCategoryId::UnknownNonHuman => (
            "gap",
            "",
            "",
            "Unknown non-human traffic stays an explicit gap until recurring traffic warrants a clearer category mapping.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_non_human_lane_fulfillment, llm_category_targets_for_mode,
        scrapling_category_targets,
    };

    #[test]
    fn canonical_lane_fulfillment_reports_mapped_and_gap_categories_explicitly() {
        let summary = canonical_non_human_lane_fulfillment();
        assert_eq!(summary.schema_version, "non_human_lane_fulfillment_v1");
        assert_eq!(summary.mapped_category_count, 6);
        assert_eq!(summary.gap_category_count, 2);

        let indexing_bot = summary
            .rows
            .iter()
            .find(|row| row.category_id == "indexing_bot")
            .unwrap();
        assert_eq!(indexing_bot.assignment_status, "mapped");
        assert_eq!(indexing_bot.runtime_lane, "scrapling_traffic");
        assert_eq!(indexing_bot.fulfillment_mode, "scrapling_worker");

        let beneficial_bot = summary
            .rows
            .iter()
            .find(|row| row.category_id == "verified_beneficial_bot")
            .unwrap();
        assert_eq!(beneficial_bot.assignment_status, "gap");
        assert!(beneficial_bot.runtime_lane.is_empty());

        let unknown_non_human = summary
            .rows
            .iter()
            .find(|row| row.category_id == "unknown_non_human")
            .unwrap();
        assert_eq!(unknown_non_human.assignment_status, "gap");
        assert!(unknown_non_human.fulfillment_mode.is_empty());
    }

    #[test]
    fn lane_target_helpers_match_bounded_fulfillment_modes() {
        assert_eq!(scrapling_category_targets(), vec!["indexing_bot"]);
        assert_eq!(
            llm_category_targets_for_mode("browser_mode"),
            vec!["automated_browser", "browser_agent", "agent_on_behalf_of_human"]
        );
        assert_eq!(
            llm_category_targets_for_mode("request_mode"),
            vec!["http_agent", "ai_scraper_bot"]
        );
        assert!(llm_category_targets_for_mode("unknown_mode").is_empty());
    }
}
