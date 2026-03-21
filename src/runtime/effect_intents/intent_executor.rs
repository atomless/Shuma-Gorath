use spin_sdk::key_value::Store;

use crate::runtime::capabilities::{
    BanWriteCapability, EventLogCapability, MetricsCapability, MonitoringCapability,
    PolicyExecutionCapabilities, PostResponseFlushCapabilities, RequestBootstrapCapabilities,
};
use crate::runtime::request_outcome::RenderedResponseEvidence;

use super::intent_types::{
    DecisionPlan, EffectExecutionContext, EffectIntent, ExecutionMode, ShadowAction,
};
use super::response_renderer::{execute_response_intent, render_shadow_allow_response};

fn shadow_action_for_response(response: &super::intent_types::ResponseIntent) -> Option<ShadowAction> {
    match response {
        super::intent_types::ResponseIntent::Continue
        | super::intent_types::ResponseIntent::ForwardAllow { .. } => None,
        super::intent_types::ResponseIntent::BlockPage { .. }
        | super::intent_types::ResponseIntent::PlainTextBlock { .. } => Some(ShadowAction::Block),
        super::intent_types::ResponseIntent::DropConnection => Some(ShadowAction::DropConnection),
        super::intent_types::ResponseIntent::Redirect { .. } => Some(ShadowAction::Redirect),
        super::intent_types::ResponseIntent::Maze { .. } => Some(ShadowAction::Maze),
        super::intent_types::ResponseIntent::Challenge => Some(ShadowAction::Challenge),
        super::intent_types::ResponseIntent::NotABot => Some(ShadowAction::NotABot),
        super::intent_types::ResponseIntent::JsChallenge => Some(ShadowAction::JsChallenge),
        super::intent_types::ResponseIntent::IpRangeTarpit { .. } => Some(ShadowAction::Tarpit),
    }
}

fn suppress_shadow_metric(metric: crate::observability::metrics::MetricName) -> bool {
    matches!(
        metric,
        crate::observability::metrics::MetricName::BansTotal
            | crate::observability::metrics::MetricName::BlocksTotal
            | crate::observability::metrics::MetricName::ChallengesTotal
    )
}

fn suppress_shadow_monitoring_intent(intent: &EffectIntent) -> bool {
    matches!(
        intent,
        EffectIntent::RecordRateViolation { .. }
            | EffectIntent::RecordGeoViolation { .. }
            | EffectIntent::RecordHoneypotHit { .. }
            | EffectIntent::RecordNotABotServed
            | EffectIntent::RecordNotABotSubmit { .. }
            | EffectIntent::RecordChallengeFailure { .. }
            | EffectIntent::RecordIpRangeChallengeSolved
            | EffectIntent::RecordLikelyHumanSample { .. }
    )
}

fn shadow_event_metadata(
    context: &EffectExecutionContext<'_>,
    action: Option<ShadowAction>,
) -> Option<crate::admin::EventExecutionMetadata> {
    if !matches!(context.execution_mode, ExecutionMode::Shadow) {
        return None;
    }
    let intended_action = action?;
    Some(crate::admin::EventExecutionMetadata {
        execution_mode: Some("shadow".to_string()),
        intended_action: Some(intended_action.as_str().to_string()),
        enforcement_applied: Some(false),
    })
}

fn prepare_intents_for_execution(
    intents: Vec<EffectIntent>,
    execution_mode: ExecutionMode,
    response: &super::intent_types::ResponseIntent,
) -> (Vec<EffectIntent>, Option<ShadowAction>) {
    let shadow_action = match execution_mode {
        ExecutionMode::Enforced => None,
        ExecutionMode::Shadow => shadow_action_for_response(response),
    };
    let Some(shadow_action) = shadow_action else {
        return (intents, None);
    };

    let mut prepared = vec![
        EffectIntent::IncrementMetric {
            metric: crate::observability::metrics::MetricName::ShadowModeActions,
            label: None,
        },
        EffectIntent::RecordShadowAction { action: shadow_action },
    ];

    for intent in intents {
        match &intent {
            EffectIntent::IncrementMetric { metric, .. } if suppress_shadow_metric(*metric) => {}
            EffectIntent::Ban(_) => {}
            _ if suppress_shadow_monitoring_intent(&intent) => {}
            _ => prepared.push(intent),
        }
    }

    (prepared, Some(shadow_action))
}

pub(super) fn apply_metric_intent(
    _capability: &MetricsCapability,
    store: &Store,
    cfg: Option<&crate::config::Config>,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::RecordPolicyMatch(transition) => {
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);
            crate::observability::metrics::record_policy_match(store, &policy_match);
            None
        }
        EffectIntent::IncrementMetric { metric, label } => {
            crate::observability::metrics::increment(store, metric, label.as_deref());
            None
        }
        EffectIntent::RecordBotnessVisibility { assessment } => {
            if let Some(cfg) = cfg {
                crate::observability::metrics::record_botness_visibility(store, cfg, &assessment);
            }
            None
        }
        EffectIntent::RecordShadowAction { .. } | EffectIntent::RecordShadowPassThrough => {
            None
        }
        other => Some(other),
    }
}

fn apply_monitoring_intent(
    _capability: &MonitoringCapability,
    store: &Store,
    ip: &str,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    apply_monitoring_intent_with_store(store, ip, intent)
}

fn apply_monitoring_intent_with_store<S: crate::challenge::KeyValueStore>(
    store: &S,
    ip: &str,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::RecordRateViolation { path, outcome } => {
            crate::observability::monitoring::record_rate_violation_with_path(
                store,
                ip,
                path.as_deref(),
                outcome.as_str(),
            );
            None
        }
        EffectIntent::RecordGeoViolation { country, action } => {
            crate::observability::monitoring::record_geo_violation(store, country.as_deref(), action.as_str());
            None
        }
        EffectIntent::RecordHoneypotHit { path } => {
            crate::observability::monitoring::record_honeypot_hit(store, ip, path.as_str());
            None
        }
        EffectIntent::RecordNotABotServed => {
            crate::observability::monitoring::record_not_a_bot_served(store);
            None
        }
        EffectIntent::RecordNotABotSubmit { outcome, solve_ms } => {
            crate::observability::monitoring::record_not_a_bot_submit(
                store,
                outcome.as_str(),
                solve_ms,
            );
            None
        }
        EffectIntent::RecordChallengeFailure { outcome } => {
            crate::observability::monitoring::record_challenge_failure(store, ip, outcome.as_str());
            None
        }
        EffectIntent::RecordIpRangeChallengeSolved => {
            crate::observability::monitoring::record_ip_range_challenge_solved(store, ip);
            None
        }
        EffectIntent::RecordLikelyHumanSample {
            sample_percent,
            sample_hint,
        } => {
            crate::observability::monitoring::maybe_record_ip_range_likely_human_sample(
                store,
                ip,
                sample_percent,
                sample_hint.as_str(),
            );
            None
        }
        EffectIntent::RecordRequestOutcome { outcome } => {
            crate::observability::monitoring::record_request_outcome(store, &outcome);
            None
        }
        EffectIntent::RecordShadowAction { action } => {
            crate::observability::monitoring::record_shadow_action(store, action);
            None
        }
        EffectIntent::RecordShadowPassThrough => {
            crate::observability::monitoring::record_shadow_pass_through(store);
            None
        }
        EffectIntent::FlushPendingMonitoringCounters => {
            crate::observability::monitoring::flush_pending_counters(store);
            None
        }
        other => Some(other),
    }
}

pub(super) fn apply_event_log_intent(
    _capability: &EventLogCapability,
    store: &Store,
    ip: &str,
    shadow_metadata: Option<&crate::admin::EventExecutionMetadata>,
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::LogEvent {
            event,
            reason,
            outcome,
        } => {
            crate::admin::log_event_with_execution_metadata(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event,
                    ip: Some(ip.to_string()),
                    reason: Some(reason),
                    outcome: Some(outcome),
                    admin: None,
                },
                shadow_metadata.cloned(),
            );
            None
        }
        other => Some(other),
    }
}

pub(super) fn apply_ban_intent(
    _capability: &BanWriteCapability,
    store: &Store,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    intent: EffectIntent,
) {
    if let EffectIntent::Ban(ban) = intent {
        let _ = provider_registry.ban_store_provider().ban_ip_with_fingerprint(
            store,
            site_id,
            ip,
            ban.reason.as_str(),
            ban.duration_seconds,
            Some(crate::enforcement::ban::BanFingerprint {
                score: ban.score,
                signals: ban.signals,
                summary: ban.summary,
            }),
        );
    }
}
pub(crate) fn execute_plan(
    plan: DecisionPlan,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &EffectExecutionContext<'_>,
    capabilities: &PolicyExecutionCapabilities,
) -> Option<RenderedResponseEvidence> {
    let (prepared_intents, shadow_action) =
        prepare_intents_for_execution(plan.intents, context.execution_mode, &plan.response);
    execute_effect_intents(prepared_intents, context, capabilities, shadow_action);
    if let Some(shadow_action) = shadow_action {
        return Some(render_shadow_allow_response(context, shadow_action));
    }
    execute_response_intent(plan.response, facts, context, capabilities)
}

pub(crate) fn execute_effect_intents(
    intents: Vec<EffectIntent>,
    context: &EffectExecutionContext<'_>,
    capabilities: &PolicyExecutionCapabilities,
    shadow_action: Option<ShadowAction>,
) {
    let shadow_metadata = shadow_event_metadata(context, shadow_action);
    for intent in intents {
        let Some(intent) =
            apply_metric_intent(capabilities.metrics(), context.store, Some(context.cfg), intent)
        else {
            continue;
        };
        let Some(intent) =
            apply_monitoring_intent(capabilities.monitoring(), context.store, context.ip, intent)
        else {
            continue;
        };
        let Some(intent) =
            apply_event_log_intent(
                capabilities.event_log(),
                context.store,
                context.ip,
                shadow_metadata.as_ref(),
                intent,
            )
        else {
            continue;
        };
        apply_ban_intent(
            capabilities.ban_write(),
            context.store,
            context.provider_registry,
            context.site_id,
            context.ip,
            intent,
        );
    }
}

pub(crate) fn execute_metric_intents(
    intents: Vec<EffectIntent>,
    store: &Store,
    capabilities: &RequestBootstrapCapabilities,
) {
    for intent in intents {
        let Some(unhandled) = apply_metric_intent(capabilities.metrics(), store, None, intent) else {
            continue;
        };
        let _ = unhandled;
    }
}

pub(crate) fn execute_monitoring_store_intents(
    intents: Vec<EffectIntent>,
    store: &Store,
    capabilities: &PostResponseFlushCapabilities,
) {
    for intent in intents {
        let Some(unhandled) = apply_monitoring_intent(capabilities.monitoring(), store, "", intent)
        else {
            continue;
        };
        let _ = unhandled;
    }
}

pub(crate) fn execute_request_outcome_intents<S: crate::challenge::KeyValueStore>(
    intents: Vec<EffectIntent>,
    store: &S,
    capabilities: &PolicyExecutionCapabilities,
) {
    for intent in intents {
        let Some(unhandled) =
            apply_monitoring_intent_with_store(store, "", intent)
        else {
            continue;
        };
        let _ = capabilities.monitoring();
        let _ = unhandled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_intents_for_shadow_suppresses_enforcement_and_records_shadow_action() {
        let intents = vec![
            EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::BlocksTotal,
                label: None,
            },
            EffectIntent::RecordHoneypotHit {
                path: "/trap".to_string(),
            },
            EffectIntent::LogEvent {
                event: crate::admin::EventType::Block,
                reason: "honeypot".to_string(),
                outcome: "blocked".to_string(),
            },
            EffectIntent::Ban(super::super::intent_types::BanIntent {
                reason: "honeypot".to_string(),
                duration_seconds: 60,
                score: None,
                signals: vec!["honeypot".to_string()],
                summary: None,
            }),
        ];

        let (prepared, shadow_action) = prepare_intents_for_execution(
            intents,
            ExecutionMode::Shadow,
            &super::super::intent_types::ResponseIntent::BlockPage {
                status: 403,
                reason: crate::enforcement::block_page::BlockReason::Honeypot,
            },
        );

        assert_eq!(shadow_action, Some(ShadowAction::Block));
        assert!(matches!(
            prepared[0],
            EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::ShadowModeActions,
                ..
            }
        ));
        assert!(matches!(
            prepared[1],
            EffectIntent::RecordShadowAction {
                action: ShadowAction::Block,
                ..
            }
        ));
        assert!(prepared.iter().any(|intent| matches!(intent, EffectIntent::LogEvent { .. })));
        assert!(!prepared.iter().any(|intent| matches!(intent, EffectIntent::Ban(_))));
        assert!(!prepared.iter().any(|intent| matches!(intent, EffectIntent::RecordHoneypotHit { .. })));
    }

    #[test]
    fn prepare_intents_for_shadow_clean_allow_does_not_inject_shadow_action() {
        let intents = vec![EffectIntent::RecordShadowPassThrough];

        let (prepared, shadow_action) = prepare_intents_for_execution(
            intents,
            ExecutionMode::Shadow,
            &super::super::intent_types::ResponseIntent::ForwardAllow {
                reason: "policy_clean_allow".to_string(),
            },
        );

        assert_eq!(shadow_action, None);
        assert_eq!(prepared.len(), 1);
        assert!(matches!(
            prepared[0],
            EffectIntent::RecordShadowPassThrough
        ));
    }

}
