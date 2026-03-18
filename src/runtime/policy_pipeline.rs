use spin_sdk::http::Request;
use spin_sdk::key_value::Store;

use crate::runtime::request_outcome::HandledRequestResponse;

fn active_botness_signal_ids(
    assessment: &crate::BotnessAssessment,
) -> Vec<crate::runtime::policy_taxonomy::SignalId> {
    assessment
        .contributions
        .iter()
        .filter(|contribution| contribution.active)
        .filter_map(|contribution| {
            crate::runtime::policy_taxonomy::signal_id_for_botness_key(contribution.key)
        })
        .collect()
}

pub(crate) fn compute_needs_js(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    site_id: &str,
    path: &str,
    ip: &str,
) -> bool {
    if !cfg.js_signal_enabled() && !cfg.js_action_enabled() {
        return false;
    }

    let browser_allowlist = cfg.browser_allowlist.as_slice();
    let js_missing_verification = path != "/health"
        && crate::signals::js_verification::needs_js_verification_with_allowlist(
            req,
            store,
            site_id,
            ip,
            browser_allowlist,
        );
    js_missing_verification
}

fn execute_decision_sequence(
    decisions: Vec<crate::runtime::policy_graph::PolicyDecision>,
    facts: &crate::runtime::request_facts::RequestFacts,
    context: &crate::runtime::effect_intents::EffectExecutionContext<'_>,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
) -> Option<HandledRequestResponse> {
    for decision in decisions {
        let plan = crate::runtime::effect_intents::plan_for_decision(&decision, facts, context.cfg);
        if let Some(rendered) =
            crate::runtime::effect_intents::execute_plan(plan, facts, context, capabilities)
        {
            return Some(HandledRequestResponse {
                branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::PolicyDecision(
                    decision,
                ),
                execution_mode: context.execution_mode,
                rendered,
            });
        }
    }
    None
}

pub(crate) fn maybe_handle_policy_graph_first_tranche(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    path: &str,
    ua: &str,
    geo_assessment: &crate::GeoAssessment,
    ip_range_evaluation: &crate::signals::ip_range_policy::Evaluation,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
) -> Option<HandledRequestResponse> {
    let execution_mode = crate::runtime::shadow_mode::effective_execution_mode(cfg);
    let context = crate::runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg,
        provider_registry,
        site_id,
        ip,
        ua,
        execution_mode,
    };

    let honeypot_hit =
        cfg.honeypot_enabled && crate::enforcement::honeypot::is_honeypot(path, &cfg.honeypots);
    let rate_limit_exceeded = if cfg.rate_action_enabled() {
        if crate::runtime::shadow_mode::shadow_mode_active(cfg) {
            provider_registry
                .rate_limiter_provider()
                .current_rate_usage(store, site_id, ip)
                >= cfg.rate_limit
        } else {
            provider_registry
                .rate_limiter_provider()
                .check_rate_limit(store, site_id, ip, cfg.rate_limit)
                != crate::providers::contracts::RateLimitDecision::Allowed
        }
    } else {
        false
    };
    let existing_ban = provider_registry
        .ban_store_provider()
        .is_banned(store, site_id, ip);

    let pre_facts = crate::runtime::request_facts::build_request_facts(
        req,
        crate::runtime::request_facts::RequestFactInputs {
            site_id: site_id.to_string(),
            ip: ip.to_string(),
            user_agent: ua.to_string(),
            ip_range_evaluation: ip_range_evaluation.clone(),
            honeypot_hit,
            rate_limit_exceeded,
            existing_ban,
            geo_route: geo_assessment.route,
            geo_country: geo_assessment.country.clone(),
            needs_js: false,
            botness_score: 0,
            botness_signal_ids: vec![],
            botness_summary: "none".to_string(),
            botness_state_summary: "none".to_string(),
            runtime_metadata_summary: crate::defence_runtime_metadata_summary(cfg),
            provider_summary: crate::provider_implementations_summary(provider_registry),
            not_a_bot_marker_valid: false,
        },
    );

    let decisions = crate::runtime::policy_graph::evaluate_first_tranche(&pre_facts, cfg);
    execute_decision_sequence(decisions, &pre_facts, &context, &capabilities)
}

pub(crate) fn maybe_handle_policy_graph_second_tranche(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    site_id: &str,
    ip: &str,
    path: &str,
    ua: &str,
    geo_assessment: &crate::GeoAssessment,
    ip_range_evaluation: &crate::signals::ip_range_policy::Evaluation,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
) -> Option<HandledRequestResponse> {
    let execution_mode = crate::runtime::shadow_mode::effective_execution_mode(cfg);
    let context = crate::runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg,
        provider_registry,
        site_id,
        ip,
        ua,
        execution_mode,
    };

    let needs_js = compute_needs_js(req, store, cfg, site_id, path, ip);
    let browser_outdated = cfg.browser_policy_enabled
        && crate::signals::browser_user_agent::is_outdated_browser(ua, &cfg.browser_block);
    let geo_risk = geo_assessment.scored_risk;
    let geo_signal_available = geo_assessment.headers_trusted && geo_assessment.country.is_some();
    let rate_usage = provider_registry
        .rate_limiter_provider()
        .current_rate_usage(store, site_id, ip);
    let maze_behavior_score = crate::maze::runtime::current_behavior_score(store, ip);
    let fingerprint_signals = crate::signals::fingerprint::collect_bot_signals(
        store,
        req,
        cfg,
        ip,
        geo_assessment.headers_trusted,
    );
    let botness = crate::compute_botness_assessment(
        crate::BotnessSignalContext {
            js_needed: needs_js,
            browser_outdated,
            geo_signal_available,
            geo_risk,
            rate_count: rate_usage,
            rate_limit: cfg.rate_limit,
            maze_behavior_score,
            fingerprint_signals,
        },
        cfg,
    );
    crate::runtime::effect_intents::execute_effect_intents(
        vec![crate::runtime::effect_intents::EffectIntent::RecordBotnessVisibility {
            assessment: botness.clone(),
        }],
        &context,
        capabilities,
        None,
    );

    let facts = crate::runtime::request_facts::build_request_facts(
        req,
        crate::runtime::request_facts::RequestFactInputs {
            site_id: site_id.to_string(),
            ip: ip.to_string(),
            user_agent: ua.to_string(),
            ip_range_evaluation: ip_range_evaluation.clone(),
            honeypot_hit: false, // first tranche only
            rate_limit_exceeded: false, // first tranche only
            existing_ban: false, // first tranche only
            geo_route: geo_assessment.route,
            geo_country: geo_assessment.country.clone(),
            needs_js,
            botness_score: botness.score,
            botness_signal_ids: active_botness_signal_ids(&botness),
            botness_summary: crate::botness_signals_summary(&botness),
            botness_state_summary: crate::botness_signal_states_summary(&botness),
            runtime_metadata_summary: crate::defence_runtime_metadata_summary(cfg),
            provider_summary: crate::provider_implementations_summary(provider_registry),
            not_a_bot_marker_valid: crate::challenge::has_valid_not_a_bot_marker(req, ip, ua),
        },
    );

    let decisions = crate::runtime::policy_graph::evaluate_second_tranche(&facts, cfg);
    execute_decision_sequence(decisions, &facts, &context, &capabilities)
}
