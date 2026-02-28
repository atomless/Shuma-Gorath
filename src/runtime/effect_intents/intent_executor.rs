use spin_sdk::http::Response;
use spin_sdk::key_value::Store;

use crate::runtime::capabilities::{
    BanWriteCapability, EventLogCapability, MetricsCapability, MonitoringCapability,
    PolicyExecutionCapabilities, PostResponseFlushCapabilities, RequestBootstrapCapabilities,
};

use super::intent_types::{DecisionPlan, EffectExecutionContext, EffectIntent};
use super::response_renderer::execute_response_intent;

pub(super) fn apply_metric_intent(
    _capability: &MetricsCapability,
    store: &Store,
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
        other => Some(other),
    }
}

fn apply_monitoring_intent(
    _capability: &MonitoringCapability,
    store: &Store,
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
    intent: EffectIntent,
) -> Option<EffectIntent> {
    match intent {
        EffectIntent::LogEvent {
            event,
            reason,
            outcome,
        } => {
            crate::admin::log_event(
                store,
                &crate::admin::EventLogEntry {
                    ts: crate::admin::now_ts(),
                    event,
                    ip: Some(ip.to_string()),
                    reason: Some(reason),
                    outcome: Some(outcome),
                    admin: None,
                },
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
        provider_registry.ban_store_provider().ban_ip_with_fingerprint(
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
) -> Option<Response> {
    execute_effect_intents(plan.intents, context, capabilities);
    execute_response_intent(plan.response, facts, context, capabilities)
}

pub(crate) fn execute_effect_intents(
    intents: Vec<EffectIntent>,
    context: &EffectExecutionContext<'_>,
    capabilities: &PolicyExecutionCapabilities,
) {
    for intent in intents {
        let Some(intent) = apply_metric_intent(capabilities.metrics(), context.store, intent) else {
            continue;
        };
        let Some(intent) =
            apply_monitoring_intent(capabilities.monitoring(), context.store, context.ip, intent)
        else {
            continue;
        };
        let Some(intent) =
            apply_event_log_intent(capabilities.event_log(), context.store, context.ip, intent)
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
        let Some(unhandled) = apply_metric_intent(capabilities.metrics(), store, intent) else {
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
