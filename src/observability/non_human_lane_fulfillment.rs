#![cfg_attr(not(test), allow(dead_code))]

use serde::{Deserialize, Serialize};

use crate::runtime::non_human_taxonomy::{
    canonical_non_human_taxonomy, NonHumanCategoryId,
};

pub(crate) const NON_HUMAN_LANE_FULFILLMENT_SCHEMA_VERSION: &str =
    "non_human_lane_fulfillment_v1";

const SCRAPLING_OWNED_CATEGORY_TARGETS: [&str; 4] = [
    "indexing_bot",
    "ai_scraper_bot",
    "automated_browser",
    "http_agent",
];
const SCRAPLING_RUNTIME_PROFILE_PREFIX: &str = "scrapling_runtime_lane";
const LLM_RUNTIME_PROFILE_PREFIX: &str = "llm_runtime_lane";
const SCRAPLING_CRAWLER_CATEGORY_TARGETS: [&str; 1] = ["indexing_bot"];
const SCRAPLING_BULK_SCRAPER_CATEGORY_TARGETS: [&str; 1] = ["ai_scraper_bot"];
const SCRAPLING_BROWSER_AUTOMATION_CATEGORY_TARGETS: [&str; 1] = ["automated_browser"];
const SCRAPLING_STEALTH_BROWSER_CATEGORY_TARGETS: [&str; 1] = ["automated_browser"];
const SCRAPLING_HTTP_AGENT_CATEGORY_TARGETS: [&str; 1] = ["http_agent"];
const LLM_BROWSER_CATEGORY_TARGETS: [&str; 2] = ["browser_agent", "agent_on_behalf_of_human"];
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
    SCRAPLING_OWNED_CATEGORY_TARGETS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

pub(crate) fn scrapling_category_targets_for_mode(mode: &str) -> Vec<String> {
    match mode {
        "crawler" => SCRAPLING_CRAWLER_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "bulk_scraper" => SCRAPLING_BULK_SCRAPER_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "browser_automation" => SCRAPLING_BROWSER_AUTOMATION_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "stealth_browser" => SCRAPLING_STEALTH_BROWSER_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        "http_agent" => SCRAPLING_HTTP_AGENT_CATEGORY_TARGETS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        _ => Vec::new(),
    }
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

pub(crate) fn observed_category_targets_for_runtime_profile(
    runtime_lane: &str,
    sim_profile: &str,
    llm_runtime_summary: Option<&crate::admin::adversary_sim::LlmRuntimeRecentRunSummary>,
) -> (String, Vec<String>, Vec<String>) {
    let normalized_profile = sim_profile.trim();
    if runtime_lane == "bot_red_team" {
        return observed_llm_targets_for_runtime_profile(normalized_profile, llm_runtime_summary);
    }
    if runtime_lane != "scrapling_traffic" || normalized_profile.is_empty() {
        return (normalized_profile.to_string(), Vec::new(), Vec::new());
    }

    if normalized_profile == SCRAPLING_RUNTIME_PROFILE_PREFIX {
        return (normalized_profile.to_string(), Vec::new(), Vec::new());
    }

    let Some(modes_raw) = normalized_profile
        .strip_prefix(SCRAPLING_RUNTIME_PROFILE_PREFIX)
        .and_then(|value| value.strip_prefix('.'))
    else {
        return (normalized_profile.to_string(), Vec::new(), Vec::new());
    };

    let mut observed_modes = Vec::new();
    let mut observed_category_ids = Vec::new();
    for raw_mode in modes_raw.split('.') {
        let mode = raw_mode.trim();
        if mode.is_empty() || observed_modes.iter().any(|value| value == mode) {
            continue;
        }
        observed_modes.push(mode.to_string());
        for category_id in scrapling_category_targets_for_mode(mode) {
            if !observed_category_ids.iter().any(|value| value == &category_id) {
                observed_category_ids.push(category_id);
            }
        }
    }

    (
        SCRAPLING_RUNTIME_PROFILE_PREFIX.to_string(),
        observed_modes,
        observed_category_ids,
    )
}

fn observed_llm_targets_for_runtime_profile(
    normalized_profile: &str,
    llm_runtime_summary: Option<&crate::admin::adversary_sim::LlmRuntimeRecentRunSummary>,
) -> (String, Vec<String>, Vec<String>) {
    let mut observed_modes = Vec::new();
    let mut observed_category_ids = Vec::new();

    if let Some(summary) = llm_runtime_summary {
        let mode = summary.fulfillment_mode.trim();
        if !mode.is_empty() {
            observed_modes.push(mode.to_string());
        }
        for category_id in &summary.category_targets {
            let category_id = category_id.trim();
            if !category_id.is_empty()
                && !observed_category_ids.iter().any(|value| value == category_id)
            {
                observed_category_ids.push(category_id.to_string());
            }
        }
    }

    if observed_modes.is_empty() {
        if normalized_profile == LLM_RUNTIME_PROFILE_PREFIX {
            return (
                LLM_RUNTIME_PROFILE_PREFIX.to_string(),
                observed_modes,
                observed_category_ids,
            );
        }
        if let Some(mode) = normalized_profile
            .strip_prefix(LLM_RUNTIME_PROFILE_PREFIX)
            .and_then(|value| value.strip_prefix('.'))
        {
            let mode = mode.trim();
            if !mode.is_empty() {
                observed_modes.push(mode.to_string());
            }
        }
    }

    if observed_category_ids.is_empty() {
        for mode in &observed_modes {
            for category_id in llm_category_targets_for_mode(mode) {
                if !observed_category_ids.iter().any(|value| value == &category_id) {
                    observed_category_ids.push(category_id);
                }
            }
        }
    }

    let normalized = if normalized_profile.starts_with(LLM_RUNTIME_PROFILE_PREFIX) {
        LLM_RUNTIME_PROFILE_PREFIX.to_string()
    } else if normalized_profile.is_empty() {
        LLM_RUNTIME_PROFILE_PREFIX.to_string()
    } else {
        normalized_profile.to_string()
    };

    (normalized, observed_modes, observed_category_ids)
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
            "crawler",
            "Shared-host Scrapling crawler persona is the intended fulfillment lane for indexing and discovery pressure.",
        ),
        NonHumanCategoryId::AiScraperBot => (
            "mapped",
            "scrapling_traffic",
            "bulk_scraper",
            "Shared-host Scrapling bulk-scraper persona is the intended fulfillment lane for request-native retrieval and training-style pressure.",
        ),
        NonHumanCategoryId::AutomatedBrowser => (
            "mapped",
            "scrapling_traffic",
            "browser_automation",
            "Browser-capable Scrapling personas are the intended fulfillment lane for non-agent browser automation pressure.",
        ),
        NonHumanCategoryId::HttpAgent => (
            "mapped",
            "scrapling_traffic",
            "http_agent",
            "Shared-host Scrapling direct-request persona is the intended fulfillment lane for bounded HTTP agent behavior.",
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
        observed_category_targets_for_runtime_profile, scrapling_category_targets,
        scrapling_category_targets_for_mode,
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
        assert_eq!(indexing_bot.fulfillment_mode, "crawler");

        let ai_scraper_bot = summary
            .rows
            .iter()
            .find(|row| row.category_id == "ai_scraper_bot")
            .unwrap();
        assert_eq!(ai_scraper_bot.assignment_status, "mapped");
        assert_eq!(ai_scraper_bot.runtime_lane, "scrapling_traffic");
        assert_eq!(ai_scraper_bot.fulfillment_mode, "bulk_scraper");

        let http_agent = summary
            .rows
            .iter()
            .find(|row| row.category_id == "http_agent")
            .unwrap();
        assert_eq!(http_agent.assignment_status, "mapped");
        assert_eq!(http_agent.runtime_lane, "scrapling_traffic");
        assert_eq!(http_agent.fulfillment_mode, "http_agent");

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
    fn lane_target_helpers_match_full_spectrum_fulfillment_modes() {
        assert_eq!(
            scrapling_category_targets(),
            vec![
                "indexing_bot",
                "ai_scraper_bot",
                "automated_browser",
                "http_agent"
            ]
        );
        assert_eq!(
            scrapling_category_targets_for_mode("crawler"),
            vec!["indexing_bot"]
        );
        assert_eq!(
            scrapling_category_targets_for_mode("bulk_scraper"),
            vec!["ai_scraper_bot"]
        );
        assert_eq!(
            scrapling_category_targets_for_mode("browser_automation"),
            vec!["automated_browser"]
        );
        assert_eq!(
            scrapling_category_targets_for_mode("stealth_browser"),
            vec!["automated_browser"]
        );
        assert_eq!(
            scrapling_category_targets_for_mode("http_agent"),
            vec!["http_agent"]
        );
        assert!(scrapling_category_targets_for_mode("unknown_mode").is_empty());
        assert_eq!(
            llm_category_targets_for_mode("browser_mode"),
            vec!["browser_agent", "agent_on_behalf_of_human"]
        );
        assert_eq!(
            llm_category_targets_for_mode("request_mode"),
            vec!["http_agent", "ai_scraper_bot"]
        );
        assert!(llm_category_targets_for_mode("unknown_mode").is_empty());
    }

    #[test]
    fn runtime_profile_observation_helper_normalizes_scrapling_modes_into_categories() {
        let (profile, modes, categories) = observed_category_targets_for_runtime_profile(
            "scrapling_traffic",
            "scrapling_runtime_lane.crawler.bulk_scraper.browser_automation.stealth_browser.http_agent",
            None,
        );
        assert_eq!(profile, "scrapling_runtime_lane");
        assert_eq!(
            modes,
            vec![
                "crawler".to_string(),
                "bulk_scraper".to_string(),
                "browser_automation".to_string(),
                "stealth_browser".to_string(),
                "http_agent".to_string()
            ]
        );
        assert_eq!(
            categories,
            vec![
                "indexing_bot".to_string(),
                "ai_scraper_bot".to_string(),
                "automated_browser".to_string(),
                "http_agent".to_string()
            ]
        );

        let (unchanged_profile, unchanged_modes, unchanged_categories) =
            observed_category_targets_for_runtime_profile(
                "deterministic_black_box",
                "fast_smoke",
                None,
            );
        assert_eq!(unchanged_profile, "fast_smoke");
        assert!(unchanged_modes.is_empty());
        assert!(unchanged_categories.is_empty());
    }

    #[test]
    fn runtime_profile_observation_helper_projects_llm_modes_into_categories() {
        let summary = crate::admin::adversary_sim::LlmRuntimeRecentRunSummary {
            receipt_count: 1,
            fulfillment_mode: "request_mode".to_string(),
            category_targets: vec!["http_agent".to_string(), "ai_scraper_bot".to_string()],
            backend_kind: "frontier_reference".to_string(),
            backend_state: "configured".to_string(),
            generation_source: "provider_response".to_string(),
            provider: "openai".to_string(),
            model_id: "gpt-5-mini".to_string(),
            fallback_reason: None,
            generated_action_count: 2,
            executed_action_count: 2,
            failed_action_count: 0,
            passed_tick_count: 1,
            failed_tick_count: 0,
            last_response_status: Some(200),
            failure_class: None,
            error: None,
            terminal_failure: None,
            latest_action_receipts: vec![],
        };
        let (profile, modes, categories) = observed_category_targets_for_runtime_profile(
            "bot_red_team",
            "llm_runtime_lane.request_mode",
            Some(&summary),
        );
        assert_eq!(profile, "llm_runtime_lane");
        assert_eq!(modes, vec!["request_mode".to_string()]);
        assert_eq!(
            categories,
            vec!["http_agent".to_string(), "ai_scraper_bot".to_string()]
        );
    }
}
