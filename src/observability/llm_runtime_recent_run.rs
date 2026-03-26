use serde::{Deserialize, Serialize};

pub(crate) const LLM_RUNTIME_ACTION_RECEIPT_LIMIT: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmRuntimeActionReceiptSummary {
    pub action_index: u64,
    pub action_type: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub(crate) struct LlmRuntimeActionOutcomeSummary {
    pub allowed_action_count: u64,
    pub intercepted_action_count: u64,
    pub error_action_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct LlmRuntimeRecentRunSummary {
    pub fulfillment_mode: String,
    pub backend_kind: String,
    pub backend_state: String,
    pub generation_source: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub status: String,
    pub generated_action_count: u64,
    pub executed_action_count: u64,
    pub failed_action_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_response_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_failure: Option<String>,
    pub action_outcomes: LlmRuntimeActionOutcomeSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub action_receipts: Vec<LlmRuntimeActionReceiptSummary>,
}

pub(crate) fn llm_runtime_recent_run_status(
    passed: bool,
    backend_state: &str,
    generation_source: &str,
    terminal_failure: Option<&str>,
    error: Option<&str>,
    failed_action_count: u64,
) -> String {
    if !passed
        || failed_action_count > 0
        || terminal_failure
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        || error.map(|value| !value.trim().is_empty()).unwrap_or(false)
    {
        return "failed_closed".to_string();
    }
    if backend_state == "degraded" || generation_source != "provider_response" {
        return "degraded".to_string();
    }
    "provider_backed".to_string()
}

pub(crate) fn summarize_llm_runtime_action_outcomes(
    receipts: &[LlmRuntimeActionReceiptSummary],
) -> LlmRuntimeActionOutcomeSummary {
    let mut summary = LlmRuntimeActionOutcomeSummary::default();
    for receipt in receipts {
        if receipt
            .error
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            summary.error_action_count = summary.error_action_count.saturating_add(1);
            continue;
        }
        match receipt.status {
            Some(status) if (200..300).contains(&status) => {
                summary.allowed_action_count = summary.allowed_action_count.saturating_add(1);
            }
            Some(_) => {
                summary.intercepted_action_count =
                    summary.intercepted_action_count.saturating_add(1);
            }
            None => {
                summary.error_action_count = summary.error_action_count.saturating_add(1);
            }
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::{
        llm_runtime_recent_run_status, summarize_llm_runtime_action_outcomes,
        LlmRuntimeActionReceiptSummary,
    };

    #[test]
    fn llm_recent_run_status_distinguishes_provider_degraded_and_failed_closed_paths() {
        assert_eq!(
            llm_runtime_recent_run_status(
                true,
                "configured",
                "provider_response",
                None,
                None,
                0,
            ),
            "provider_backed"
        );
        assert_eq!(
            llm_runtime_recent_run_status(
                true,
                "degraded",
                "fallback_request_mode",
                None,
                None,
                0,
            ),
            "degraded"
        );
        assert_eq!(
            llm_runtime_recent_run_status(
                false,
                "configured",
                "provider_response",
                Some("transport_failure"),
                None,
                0,
            ),
            "failed_closed"
        );
    }

    #[test]
    fn llm_recent_run_action_outcomes_bucket_allowed_intercepted_and_error_receipts() {
        let summary = summarize_llm_runtime_action_outcomes(&[
            LlmRuntimeActionReceiptSummary {
                action_index: 1,
                action_type: "http_get".to_string(),
                path: "/".to_string(),
                label: None,
                status: Some(200),
                error: None,
            },
            LlmRuntimeActionReceiptSummary {
                action_index: 2,
                action_type: "http_post".to_string(),
                path: "/challenge/puzzle".to_string(),
                label: None,
                status: Some(403),
                error: None,
            },
            LlmRuntimeActionReceiptSummary {
                action_index: 3,
                action_type: "http_get".to_string(),
                path: "/robots.txt".to_string(),
                label: None,
                status: None,
                error: Some("timeout".to_string()),
            },
        ]);

        assert_eq!(summary.allowed_action_count, 1);
        assert_eq!(summary.intercepted_action_count, 1);
        assert_eq!(summary.error_action_count, 1);
    }
}
