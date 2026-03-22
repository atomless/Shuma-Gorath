use serde::{Deserialize, Serialize};

pub(crate) const NON_HUMAN_TAXONOMY_SCHEMA_VERSION: &str = "non_human_taxonomy_v1";

const POSTURE_SCALE: [&str; 5] = [
    "allowed",
    "tolerated",
    "cost_reduced",
    "restricted",
    "blocked",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NonHumanCategoryId {
    IndexingBot,
    AiScraperBot,
    AutomatedBrowser,
    HttpAgent,
    BrowserAgent,
    AgentOnBehalfOfHuman,
    VerifiedBeneficialBot,
    UnknownNonHuman,
}

#[cfg_attr(not(test), allow(dead_code))]
impl NonHumanCategoryId {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::IndexingBot => "indexing_bot",
            Self::AiScraperBot => "ai_scraper_bot",
            Self::AutomatedBrowser => "automated_browser",
            Self::HttpAgent => "http_agent",
            Self::BrowserAgent => "browser_agent",
            Self::AgentOnBehalfOfHuman => "agent_on_behalf_of_human",
            Self::VerifiedBeneficialBot => "verified_beneficial_bot",
            Self::UnknownNonHuman => "unknown_non_human",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanCategoryDescriptor {
    pub category_id: NonHumanCategoryId,
    pub label: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compatible_postures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NonHumanTaxonomyCatalog {
    pub schema_version: String,
    pub posture_scale: Vec<String>,
    pub categories: Vec<NonHumanCategoryDescriptor>,
}

pub(crate) fn posture_scale() -> Vec<String> {
    POSTURE_SCALE.iter().map(|value| (*value).to_string()).collect()
}

pub(crate) fn canonical_non_human_taxonomy() -> NonHumanTaxonomyCatalog {
    NonHumanTaxonomyCatalog {
        schema_version: NON_HUMAN_TAXONOMY_SCHEMA_VERSION.to_string(),
        posture_scale: posture_scale(),
        categories: vec![
            descriptor(
                NonHumanCategoryId::IndexingBot,
                "Indexing bot",
                "Non-human traffic that primarily discovers and indexes content.",
            ),
            descriptor(
                NonHumanCategoryId::AiScraperBot,
                "AI scraper bot",
                "Non-human traffic that retrieves content in bulk for model training or retrieval.",
            ),
            descriptor(
                NonHumanCategoryId::AutomatedBrowser,
                "Automated browser",
                "Browser-driven automation that executes page flows without a human at the keyboard.",
            ),
            descriptor(
                NonHumanCategoryId::HttpAgent,
                "HTTP agent",
                "Programmatic request traffic that operates directly at the HTTP layer.",
            ),
            descriptor(
                NonHumanCategoryId::BrowserAgent,
                "Browser agent",
                "Agentic automation that drives a browser across multi-step flows.",
            ),
            descriptor(
                NonHumanCategoryId::AgentOnBehalfOfHuman,
                "Agent on behalf of human",
                "Non-human traffic that acts for a human user with delegated intent or explicit user control.",
            ),
            descriptor(
                NonHumanCategoryId::VerifiedBeneficialBot,
                "Verified beneficial bot",
                "Verified non-human traffic intentionally tolerated or allowed by site policy.",
            ),
            descriptor(
                NonHumanCategoryId::UnknownNonHuman,
                "Unknown non-human",
                "Non-human traffic Shuma can distinguish from likely humans but not yet classify more precisely.",
            ),
        ],
    }
}

fn descriptor(
    category_id: NonHumanCategoryId,
    label: &str,
    description: &str,
) -> NonHumanCategoryDescriptor {
    NonHumanCategoryDescriptor {
        category_id,
        label: label.to_string(),
        description: description.to_string(),
        compatible_postures: Vec::new(),
    }
}
