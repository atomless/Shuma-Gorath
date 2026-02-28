use spin_sdk::http::{Request, Response};

/// Main handler logic, testable as a plain Rust function.
pub(crate) fn handle_request(req: &Request) -> Response {
    if let Err(err) = crate::config::validate_env_only_once() {
        crate::log_line(&format!("[ENV ERROR] {}", err));
        return Response::new(500, "Server configuration error");
    }
    let path = req.path();
    let sim_metadata = crate::runtime::sim_telemetry::metadata_from_request(
        req,
        crate::config::runtime_environment(),
        crate::config::adversary_sim_available(),
    );
    let _sim_context_guard = crate::runtime::sim_telemetry::enter(sim_metadata);

    if crate::config::https_enforced() && !crate::request_is_https(req) {
        return Response::new(403, "HTTPS required");
    }

    if let Some(response) = crate::runtime::request_router::maybe_handle_early_route(req, path) {
        return response;
    }

    if crate::should_bypass_expensive_bot_checks_for_static(req, path) {
        return Response::new(200, "OK (passed bot defence)");
    }

    let site_id = "default";
    let ip = crate::extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("");

    let store = match crate::runtime::kv_gate::open_store_or_fail_mode_response() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let store = &store;
    let request_capabilities = crate::runtime::capabilities::RuntimeCapabilities::for_request_path();
    if let Some(sim_tag_failure) = crate::runtime::sim_telemetry::take_last_validation_failure() {
        crate::runtime::effect_intents::execute_metric_intents(
            vec![crate::policy_signal_intent(sim_tag_failure.signal_id())],
            store,
            &request_capabilities,
        );
        crate::log_line(&format!(
            "[SIM TAG] rejected reason={}",
            sim_tag_failure.as_str()
        ));
    }

    let cfg = match crate::load_runtime_config(store, site_id, path) {
        Ok(cfg) => cfg,
        Err(resp) => return resp,
    };
    let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
    let request_effect_context = crate::runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg: &cfg,
        provider_registry: &provider_registry,
        site_id,
        ip: &ip,
        ua,
    };
    let execute_request_intents = |intents: Vec<crate::runtime::effect_intents::EffectIntent>| {
        crate::runtime::effect_intents::execute_effect_intents(
            intents,
            &request_effect_context,
            &request_capabilities,
        );
    };
    execute_request_intents(crate::provider_backend_visibility_intents(&provider_registry));
    execute_request_intents(vec![crate::policy_signal_intent(
        crate::runtime::policy_taxonomy::SignalId::CtxPathClass,
    )]);
    if crate::forwarded_ip_trusted(req) {
        execute_request_intents(vec![crate::policy_signal_intent(
            crate::runtime::policy_taxonomy::SignalId::CtxIpTrusted,
        )]);
    }
    if !ua.is_empty() {
        execute_request_intents(vec![crate::policy_signal_intent(
            crate::runtime::policy_taxonomy::SignalId::CtxUa,
        )]);
    }
    let geo_assessment = crate::assess_geo_request(req, &cfg);

    // CDP Report endpoint - receives automation detection reports from client-side JS
    if path == provider_registry.fingerprint_signal_provider().report_path()
        && *req.method() == spin_sdk::http::Method::Post
    {
        return provider_registry
            .fingerprint_signal_provider()
            .handle_report(store, req);
    }

    if path == crate::maze::checkpoint_path() {
        let response = crate::maze::runtime::handle_checkpoint(store, &cfg, req, &ip, ua);
        let checkpoint_outcome = match *response.status() {
            204 => "accepted",
            405 => "method_not_allowed",
            403 => "binding_mismatch",
            _ => "invalid",
        };
        execute_request_intents(vec![crate::increment_metric_intent(
            crate::observability::metrics::MetricName::MazeCheckpointOutcomes,
            Some(checkpoint_outcome.to_string()),
        )]);
        return response;
    }

    if path == crate::maze::issue_links_path() {
        return crate::maze::runtime::handle_issue_links(store, &cfg, req, &ip, ua);
    }

    if path == crate::tarpit::progress_path() {
        return provider_registry
            .maze_tarpit_provider()
            .handle_tarpit_progress(req, store, &cfg, site_id, &ip, ua);
    }

    // Maze - route suspicious crawlers into deception space (only if enabled)
    if provider_registry.maze_tarpit_provider().is_maze_path(path) {
        if !cfg.maze_enabled {
            return Response::new(404, "Not Found");
        }
        let policy_match = crate::runtime::policy_taxonomy::resolve_policy_match(
            crate::runtime::policy_taxonomy::PolicyTransition::MazeTraversal,
        );
        execute_request_intents(vec![
            crate::runtime::effect_intents::EffectIntent::RecordPolicyMatch(
                crate::runtime::policy_taxonomy::PolicyTransition::MazeTraversal,
            ),
        ]);
        let event_outcome = policy_match.annotate_outcome("maze_page_served");
        return provider_registry
            .maze_tarpit_provider()
            .serve_maze_with_tracking(
                req,
                store,
                &cfg,
                &ip,
                ua,
                path,
                "maze_trap",
                event_outcome.as_str(),
                None,
            );
    }

    execute_request_intents(vec![crate::increment_metric_intent(
        crate::observability::metrics::MetricName::RequestsTotal,
        None,
    )]);

    // Path-based allowlist (for webhooks/integrations)
    if cfg.path_allowlist_enabled && crate::signals::allowlist::is_path_allowlisted(path, &cfg.path_allowlist)
    {
        execute_request_intents(vec![crate::increment_metric_intent(
            crate::observability::metrics::MetricName::AllowlistedTotal,
            None,
        )]);
        return Response::new(200, "OK (path allowlisted)");
    }
    // IP/CIDR allowlist
    if cfg.bypass_allowlists_enabled && crate::signals::allowlist::is_allowlisted(&ip, &cfg.allowlist) {
        execute_request_intents(vec![crate::increment_metric_intent(
            crate::observability::metrics::MetricName::AllowlistedTotal,
            None,
        )]);
        return Response::new(200, "OK (allowlisted)");
    }
    let ip_range_evaluation = crate::signals::ip_range_policy::evaluate(&cfg, &ip);
    if let Some(response) = crate::runtime::test_mode::maybe_handle_test_mode(
        store,
        &cfg,
        site_id,
        &ip,
        path,
        &ip_range_evaluation,
        geo_assessment.route,
        || crate::signals::js_verification::needs_js_verification(req, store, site_id, &ip),
        || {
            execute_request_intents(vec![crate::increment_metric_intent(
                crate::observability::metrics::MetricName::TestModeActions,
                None,
            )])
        },
    ) {
        return response;
    }
    if let Some(response) = crate::runtime::policy_pipeline::maybe_handle_policy_graph_first_tranche(
        req,
        store,
        &cfg,
        &provider_registry,
        site_id,
        &ip,
        path,
        ua,
        &geo_assessment,
        &ip_range_evaluation,
    ) {
        return response;
    }
    // PoW endpoints (public, before JS verification)
    if path == "/pow" {
        if *req.method() != spin_sdk::http::Method::Get {
            return Response::new(405, "Method Not Allowed");
        }
        return provider_registry
            .challenge_engine_provider()
            .handle_pow_challenge(
                &ip,
                ua,
                cfg.pow_enabled,
                cfg.pow_difficulty,
                cfg.pow_ttl_seconds,
            );
    }
    if path == "/pow/verify" {
        return provider_registry
            .challenge_engine_provider()
            .handle_pow_verify(req, &ip, cfg.pow_enabled);
    }
    if let Some(response) = crate::runtime::policy_pipeline::maybe_handle_policy_graph_second_tranche(
        req,
        store,
        &cfg,
        &provider_registry,
        site_id,
        &ip,
        path,
        ua,
        &geo_assessment,
        &ip_range_evaluation,
    ) {
        return response;
    }
    let record_allow_clean = || {
        execute_request_intents(vec![
            crate::runtime::effect_intents::EffectIntent::RecordPolicyMatch(
                crate::runtime::policy_taxonomy::PolicyTransition::AllowClean,
            ),
            crate::runtime::effect_intents::EffectIntent::RecordLikelyHumanSample {
                sample_percent: cfg.ip_range_suggestions_likely_human_sample_percent,
                sample_hint: path.to_string(),
            },
        ]);
    };

    if let Some(response) = crate::runtime::sim_public::maybe_handle(req, path, &cfg) {
        if *response.status() == 200u16 {
            record_allow_clean();
        }
        return response;
    }

    record_allow_clean();

    Response::new(200, "OK (passed bot defence)")
}
