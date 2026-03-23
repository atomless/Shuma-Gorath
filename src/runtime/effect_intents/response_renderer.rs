use crate::runtime::capabilities::PolicyExecutionCapabilities;
use crate::runtime::request_outcome::{RenderedResponseEvidence, ResponseKind};

use super::intent_executor::{apply_ban_intent, apply_event_log_intent, apply_metric_intent};
use super::intent_types::{EffectExecutionContext, EffectIntent, ResponseIntent, ShadowAction};

pub(crate) fn render_forward_allow_response(
    context: &EffectExecutionContext<'_>,
    reason: &str,
) -> RenderedResponseEvidence {
    let started = std::time::Instant::now();
    crate::observability::metrics::record_forward_attempt(context.store);
    let forward = crate::runtime::upstream_proxy::forward_allow_request(
        crate::runtime::upstream_proxy::ForwardRequestContext {
            req: context.req,
            ip: context.ip,
        },
        reason,
    );
    let latency_ms = started.elapsed().as_millis() as u64;
    if let Some(class) = forward.failure_class {
        crate::observability::metrics::record_forward_failure(context.store, class);
    } else {
        crate::observability::metrics::record_forward_success(context.store, latency_ms);
    }
    let mut evidence =
        RenderedResponseEvidence::forwarded(forward.response, forward.failure_class, None);
    evidence.forward_latency_ms = Some(latency_ms);
    evidence
}

pub(crate) fn render_shadow_allow_response(
    context: &EffectExecutionContext<'_>,
    action: ShadowAction,
) -> RenderedResponseEvidence {
    if crate::runtime::shadow_mode::shadow_passthrough_available() {
        let reason = format!("shadow_mode_shadow_{}", action.as_str());
        let mut evidence = render_forward_allow_response(context, reason.as_str());
        evidence.intended_action = Some(action);
        return evidence;
    }
    RenderedResponseEvidence::synthetic_shadow_action(
        crate::runtime::shadow_mode::synthetic_shadow_response(action),
        action,
    )
}

pub(super) fn execute_response_intent(
    response_intent: ResponseIntent,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &EffectExecutionContext<'_>,
    capabilities: &PolicyExecutionCapabilities,
) -> Option<RenderedResponseEvidence> {
    let _response_privileged = capabilities.response_privileged();
    match response_intent {
        ResponseIntent::Continue => None,
        ResponseIntent::ForwardAllow { reason } => {
            Some(render_forward_allow_response(context, reason.as_str()))
        }
        ResponseIntent::BlockPage { status, reason } => Some(RenderedResponseEvidence::local(
            spin_sdk::http::Response::new(
                status,
                crate::enforcement::block_page::render_block_page(reason),
            ),
            ResponseKind::BlockPage,
        )),
        ResponseIntent::PlainTextBlock { body } => Some(RenderedResponseEvidence::local(
            spin_sdk::http::Response::builder()
                .status(403)
                .header("Content-Type", "text/plain; charset=utf-8")
                .header("Cache-Control", "no-store")
                .body(body)
                .build(),
            ResponseKind::PlainTextBlock,
        )),
        ResponseIntent::DropConnection => Some(RenderedResponseEvidence::local(
            spin_sdk::http::Response::builder()
                .status(444)
                .body("")
                .build(),
            ResponseKind::DropConnection,
        )),
        ResponseIntent::Redirect { location } => Some(RenderedResponseEvidence::local(
            spin_sdk::http::Response::builder()
                .status(308)
                .header("Location", location)
                .header("Cache-Control", "no-store")
                .body("")
                .build(),
            ResponseKind::Redirect,
        )),
        ResponseIntent::Maze {
            entry_path,
            event_reason,
            event_outcome,
            botness_score,
        } => Some(RenderedResponseEvidence::local(
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
            ResponseKind::Maze,
        )),
        ResponseIntent::Challenge => Some(RenderedResponseEvidence::local(
            context
                .provider_registry
                .challenge_engine_provider()
                .render_challenge(
                    context.req,
                    context.cfg.challenge_puzzle_transform_count as usize,
                    context.cfg.challenge_puzzle_seed_ttl_seconds,
                ),
            ResponseKind::Challenge,
        )),
        ResponseIntent::NotABot => {
            let not_a_bot_response = context
                .provider_registry
                .challenge_engine_provider()
                .render_not_a_bot(context.req, context.cfg);
            Some(RenderedResponseEvidence::local(
                crate::maze::covert_decoy::maybe_inject_non_maze_decoy(
                    context.req,
                    context.cfg,
                    context.ip,
                    context.ua,
                    not_a_bot_response,
                    facts.botness_score,
                ),
                ResponseKind::NotABot,
            ))
        }
        ResponseIntent::JsChallenge => {
            let report_endpoint = context
                .provider_registry
                .fingerprint_signal_provider()
                .report_path();
            Some(RenderedResponseEvidence::local(
                crate::signals::js_verification::inject_js_challenge(
                    context.ip,
                    context.ua,
                    report_endpoint,
                    context.cfg.pow_enabled,
                    context.cfg.pow_difficulty,
                    context.cfg.pow_ttl_seconds,
                    context.cfg.cdp_probe_family,
                    context.cfg.cdp_probe_rollout_percent,
                ),
                ResponseKind::JsChallenge,
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
                    apply_event_log_intent(
                        capabilities.event_log(),
                        context.store,
                        context.ip,
                        None,
                        intent,
                    )
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
                return Some(RenderedResponseEvidence::local(response, ResponseKind::Tarpit));
            }

            let transition = crate::runtime::policy_taxonomy::PolicyTransition::IpRangeTarpit(
                vec![crate::runtime::policy_taxonomy::SignalId::IpRangeCustom],
            );
            let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(transition);

            if context.cfg.maze_enabled {
                let event_outcome = policy_match.annotate_outcome(
                    format!("{} tarpit_unavailable fallback=maze", base_outcome).as_str(),
                );
                return Some(RenderedResponseEvidence::local(
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
                    ResponseKind::Maze,
                ));
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
                apply_event_log_intent(
                    capabilities.event_log(),
                    context.store,
                    context.ip,
                    None,
                    log_intent,
                )
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

            Some(RenderedResponseEvidence::local(
                spin_sdk::http::Response::new(
                    403,
                    crate::enforcement::block_page::render_block_page(
                        crate::enforcement::block_page::BlockReason::IpRangePolicy,
                    ),
                ),
                ResponseKind::BlockPage,
            ))
        }
    }
}
