use spin_sdk::http::Response;

use crate::runtime::capabilities::PolicyExecutionCapabilities;

use super::intent_executor::{apply_ban_intent, apply_event_log_intent, apply_metric_intent};
use super::intent_types::{EffectExecutionContext, EffectIntent, ResponseIntent};

pub(super) fn execute_response_intent(
    response_intent: ResponseIntent,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &EffectExecutionContext<'_>,
    capabilities: &PolicyExecutionCapabilities,
) -> Option<Response> {
    let _response_privileged = capabilities.response_privileged();
    match response_intent {
        ResponseIntent::Continue => None,
        ResponseIntent::OkBody(body) => Some(Response::new(200, body)),
        ResponseIntent::BlockPage { status, reason } => {
            Some(Response::new(status, crate::enforcement::block_page::render_block_page(reason)))
        }
        ResponseIntent::PlainTextBlock { body } => Some(
            Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(body)
                .build(),
        ),
        ResponseIntent::DropConnection => Some(
            Response::builder()
                .status(444)
                .header("Connection", "close")
                .body("")
                .build(),
        ),
        ResponseIntent::Redirect { location } => Some(
            Response::builder()
                .status(308)
                .header("Location", location)
                .header("Cache-Control", "no-store")
                .body("")
                .build(),
        ),
        ResponseIntent::Maze {
            entry_path,
            event_reason,
            event_outcome,
            botness_score,
        } => Some(
            context
                .provider_registry
                .maze_tarpit_provider()
                .serve_maze_with_tracking(
                    context.req,
                    context.store,
                    context.cfg,
                    context.ip,
                    context.ua,
                    entry_path.as_str(),
                    event_reason.as_str(),
                    event_outcome.as_str(),
                    botness_score,
                ),
        ),
        ResponseIntent::Challenge => Some(
            context
                .provider_registry
                .challenge_engine_provider()
                .render_challenge(
                    context.req,
                    context.cfg.challenge_puzzle_transform_count as usize,
                    context.cfg.challenge_puzzle_seed_ttl_seconds,
                ),
        ),
        ResponseIntent::NotABot => {
            let not_a_bot_response = context
                .provider_registry
                .challenge_engine_provider()
                .render_not_a_bot(context.req, context.cfg);
            Some(crate::maze::covert_decoy::maybe_inject_non_maze_decoy(
                context.req,
                context.cfg,
                context.ip,
                context.ua,
                not_a_bot_response,
                facts.botness_score,
            ))
        }
        ResponseIntent::JsChallenge => {
            let report_endpoint = context
                .provider_registry
                .fingerprint_signal_provider()
                .report_path();
            Some(crate::signals::js_verification::inject_js_challenge(
                context.ip,
                context.ua,
                report_endpoint,
                context.cfg.pow_enabled,
                context.cfg.pow_difficulty,
                context.cfg.pow_ttl_seconds,
                context.cfg.cdp_probe_family,
                context.cfg.cdp_probe_rollout_percent,
            ))
        }
        ResponseIntent::IpRangeTarpit {
            base_outcome,
            signal_ids,
        } => {
            if let Some(response) = context
                .provider_registry
                .maze_tarpit_provider()
                .maybe_handle_tarpit(
                    context.req,
                    context.store,
                    context.cfg,
                    context.site_id,
                    context.ip,
                )
            {
                let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
                    crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(signal_ids),
                );
                let intent = EffectIntent::LogEvent {
                    event: crate::admin::EventType::Challenge,
                    reason: "ip_range_policy_tarpit".to_string(),
                    outcome: policy_match.annotate_outcome(base_outcome.as_str()),
                };
                if let Some(intent) =
                    apply_event_log_intent(capabilities.event_log(), context.store, context.ip, intent)
                {
                    apply_ban_intent(
                        capabilities.ban_write(),
                        context.store,
                        context.provider_registry,
                        context.site_id,
                        context.ip,
                        intent,
                    );
                }
                return Some(response);
            }

            let transition = crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(
                vec![crate::runtime::policy_taxonomy::SignalId::IpRangeCustom],
            );
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);

            if context.cfg.maze_enabled {
                let event_outcome = policy_match.annotate_outcome(
                    format!("{} tarpit_unavailable fallback=maze", base_outcome).as_str(),
                );
                return Some(
                    context
                        .provider_registry
                        .maze_tarpit_provider()
                        .serve_maze_with_tracking(
                            context.req,
                            context.store,
                            context.cfg,
                            context.ip,
                            context.ua,
                            crate::maze::entry_path("ip-range-tarpit-fallback").as_str(),
                            "ip_range_policy_tarpit_fallback_maze",
                            event_outcome.as_str(),
                            None,
                        ),
                );
            }

            let block_intent = EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::BlocksTotal,
                label: None,
            };
            if let Some(intent) = apply_metric_intent(
                capabilities.metrics(),
                context.store,
                Some(context.cfg),
                block_intent,
            )
            {
                let _ = intent;
            }
            let log_intent = EffectIntent::LogEvent {
                event: crate::admin::EventType::Block,
                reason: "ip_range_policy_tarpit_fallback_block".to_string(),
                outcome: policy_match.annotate_outcome(
                    format!("{} tarpit_unavailable fallback=block", base_outcome).as_str(),
                ),
            };
            if let Some(intent) =
                apply_event_log_intent(capabilities.event_log(), context.store, context.ip, log_intent)
            {
                apply_ban_intent(
                    capabilities.ban_write(),
                    context.store,
                    context.provider_registry,
                    context.site_id,
                    context.ip,
                    intent,
                );
            }

            Some(Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::IpRangePolicy,
                ),
            ))
        }
    }
}
