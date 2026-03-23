use spin_sdk::http::{Request, Response};

#[derive(Clone, Copy)]
pub(crate) struct RequestFlowCapabilityToken(());

impl RequestFlowCapabilityToken {
    fn new() -> Self {
        Self(())
    }
}

fn clean_allow_monitoring_intents(
    traffic_origin: crate::runtime::request_outcome::TrafficOrigin,
    path: &str,
    sample_percent: u8,
    shadow_mode_active: bool,
) -> Vec<crate::runtime::effect_intents::EffectIntent> {
    if matches!(
        traffic_origin,
        crate::runtime::request_outcome::TrafficOrigin::AdversarySim
    ) {
        return Vec::new();
    }

    let mut intents = vec![
        crate::runtime::effect_intents::EffectIntent::RecordPolicyMatch(
            crate::runtime::policy_taxonomy::PolicyTransition::AllowClean,
        ),
        crate::runtime::effect_intents::EffectIntent::RecordLikelyHumanSample {
            sample_percent,
            sample_hint: path.to_string(),
        },
    ];
    if shadow_mode_active {
        intents.push(crate::runtime::effect_intents::EffectIntent::RecordShadowPassThrough);
    }
    intents
}

fn observe_verified_identity_intents(
    default_provenance: crate::bot_identity::contracts::IdentityProvenance,
    result: &crate::bot_identity::verification::IdentityVerificationResult,
) -> Vec<crate::runtime::effect_intents::EffectIntent> {
    let Some(record) = crate::bot_identity::telemetry::IdentityVerificationTelemetryRecord::from_verification_result(
        default_provenance,
        result,
    ) else {
        return Vec::new();
    };

    vec![
        crate::runtime::effect_intents::EffectIntent::RecordVerifiedIdentityTelemetry { record },
    ]
}

fn observe_verified_identity_result(
    store: &dyn crate::challenge::KeyValueStore,
    site_id: &str,
    req: &Request,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
) -> crate::bot_identity::verification::IdentityVerificationResult {
    provider_registry
        .verified_identity_provider()
        .verify_identity(store, site_id, req, cfg)
}

fn verified_identity_default_provenance(
    provider_registry: &crate::providers::registry::ProviderRegistry,
) -> crate::bot_identity::contracts::IdentityProvenance {
    if provider_registry.backend_for(crate::providers::registry::ProviderCapability::VerifiedIdentity)
        == crate::config::ProviderBackend::External
    {
        crate::bot_identity::contracts::IdentityProvenance::Provider
    } else {
        crate::bot_identity::contracts::IdentityProvenance::Native
    }
}

fn bootstrap_failure_handled_response(
    response: Response,
) -> crate::runtime::request_outcome::HandledRequestResponse {
    crate::runtime::request_outcome::HandledRequestResponse {
        branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::BootstrapFailure,
        execution_mode: crate::runtime::effect_intents::ExecutionMode::Enforced,
        rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
            response,
            crate::runtime::request_outcome::ResponseKind::ControlPlaneResponse,
        ),
    }
}

fn finalize_request_outcome<S: crate::challenge::KeyValueStore>(
    store: &S,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
    traffic_origin: crate::runtime::request_outcome::TrafficOrigin,
    handled: crate::runtime::request_outcome::HandledRequestResponse,
    verified_identity_lane: Option<crate::runtime::request_outcome::RequestOutcomeLane>,
    verified_identity_category: Option<crate::runtime::traffic_classification::NonHumanCategoryAssignment>,
) -> Response {
    let outcome = crate::runtime::request_outcome::RenderedRequestOutcome::from_handled_response(
        traffic_origin,
        &handled,
        verified_identity_lane,
        verified_identity_category,
    );
    crate::runtime::effect_intents::execute_request_outcome_intents(vec![
        crate::runtime::effect_intents::EffectIntent::RecordRequestOutcome { outcome },
    ], store, capabilities);
    handled.rendered.response
}

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
    let inherited_sim_metadata = if sim_metadata.is_none() {
        crate::runtime::sim_telemetry::current_metadata()
    } else {
        None
    };
    let _sim_context_guard =
        crate::runtime::sim_telemetry::enter(sim_metadata.or(inherited_sim_metadata));
    let traffic_origin = if crate::runtime::sim_telemetry::current_metadata().is_some() {
        crate::runtime::request_outcome::TrafficOrigin::AdversarySim
    } else {
        crate::runtime::request_outcome::TrafficOrigin::Live
    };
    let capability_token = RequestFlowCapabilityToken::new();
    let request_capabilities =
        crate::runtime::capabilities::RuntimeCapabilities::for_policy_execution_phase(capability_token);

    if crate::config::https_enforced() && !crate::request_is_https(req) {
        return Response::new(403, "HTTPS required");
    }

    if let Some(response) =
        crate::runtime::request_router::maybe_handle_early_route(req, path, &request_capabilities)
    {
        return response;
    }
    let static_bypass = crate::should_bypass_expensive_bot_checks_for_static(req, path);
    let ip = crate::extract_client_ip(req);
    if static_bypass {
        return crate::runtime::upstream_proxy::forward_allow_request(
            crate::runtime::upstream_proxy::ForwardRequestContext { req, ip: &ip },
            "static_asset_bypass",
        )
        .response;
    }

    let site_id = "default";
    let ua = req
        .header("user-agent")
        .map(|v| v.as_str().unwrap_or(""))
        .unwrap_or("");

    let store = match crate::runtime::kv_gate::open_store_or_fail_mode_response() {
        Ok(store) => store,
        Err(response) => return response,
    };
    let store = &store;
    let bootstrap_capabilities =
        crate::runtime::capabilities::RuntimeCapabilities::for_request_bootstrap_phase(capability_token);
    if let Some(sim_tag_failure) = crate::runtime::sim_telemetry::take_last_validation_failure() {
        crate::runtime::effect_intents::execute_metric_intents(
            vec![crate::policy_signal_intent(sim_tag_failure.signal_id())],
            store,
            &bootstrap_capabilities,
        );
        crate::log_line(&format!(
            "[SIM TAG] rejected reason={}",
            sim_tag_failure.as_str()
        ));
    }

    let cfg = match crate::load_runtime_config(store, site_id, path) {
        Ok(cfg) => cfg,
        Err(resp) => {
            return finalize_request_outcome(
                store,
                &request_capabilities,
                traffic_origin,
                bootstrap_failure_handled_response(resp),
                None,
                None,
            )
        }
    };
    let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
    let verified_identity_result =
        observe_verified_identity_result(store, site_id, req, &cfg, &provider_registry);
    let verified_identity_default_provenance =
        verified_identity_default_provenance(&provider_registry);
    let verified_identity = if verified_identity_result.status
        == crate::bot_identity::verification::IdentityVerificationResultStatus::Verified
    {
        verified_identity_result.identity.clone()
    } else {
        None
    };
    let verified_identity_lane = verified_identity.as_ref().map(|identity| {
        crate::runtime::traffic_classification::verified_identity_lane_assignment(identity).into()
    });
    let verified_identity_category = verified_identity.as_ref().map(|identity| {
        crate::runtime::traffic_classification::verified_identity_category_assignment(identity)
    });
    let request_effect_context = crate::runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg: &cfg,
        provider_registry: &provider_registry,
        site_id,
        ip: &ip,
        ua,
        execution_mode: crate::runtime::shadow_mode::effective_execution_mode(&cfg),
    };
    let execute_request_intents = |intents: Vec<crate::runtime::effect_intents::EffectIntent>| {
        crate::runtime::effect_intents::execute_effect_intents(
            intents,
            &request_effect_context,
            &request_capabilities,
            None,
        );
    };
    let finalize_handled_response =
        |handled: crate::runtime::request_outcome::HandledRequestResponse| {
            finalize_request_outcome(
                store,
                &request_capabilities,
                traffic_origin,
                handled,
                verified_identity_lane,
                verified_identity_category,
            )
        };
    let forward_allow_response = |reason: &str| {
        if crate::runtime::shadow_mode::shadow_mode_active(&cfg)
            && !crate::runtime::shadow_mode::shadow_passthrough_available()
        {
            return crate::runtime::request_outcome::RenderedResponseEvidence::synthetic_shadow_allow(
                crate::runtime::shadow_mode::synthetic_shadow_allow_response(),
            );
        }
        crate::runtime::effect_intents::render_forward_allow_response(&request_effect_context, reason)
    };
    let record_allow_clean = || {
        let intents = clean_allow_monitoring_intents(
            traffic_origin,
            path,
            cfg.ip_range_suggestions_likely_human_sample_percent,
            crate::runtime::shadow_mode::shadow_mode_active(&cfg),
        );
        if !intents.is_empty() {
            execute_request_intents(intents);
        }
    };
    execute_request_intents(crate::provider_backend_visibility_intents(&provider_registry));
    execute_request_intents(observe_verified_identity_intents(
        verified_identity_default_provenance,
        &verified_identity_result,
    ));
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
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                provider_registry
                    .fingerprint_signal_provider()
                    .handle_report(store, req),
                crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
            ),
        });
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
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                response,
                crate::runtime::request_outcome::ResponseKind::CheckpointResponse,
            ),
        });
    }

    if path == crate::maze::issue_links_path() {
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                crate::maze::runtime::handle_issue_links(store, &cfg, req, &ip, ua),
                crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
            ),
        });
    }

    if path == crate::tarpit::progress_path() {
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                provider_registry
                    .maze_tarpit_provider()
                    .handle_tarpit_progress(req, store, &cfg, site_id, &ip, ua),
                crate::runtime::request_outcome::ResponseKind::Tarpit,
            ),
        });
    }

    // Maze - route suspicious crawlers into deception space (only if enabled)
    if provider_registry.maze_tarpit_provider().is_maze_path(path) {
        if !cfg.maze_enabled {
            return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
                branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
                execution_mode: request_effect_context.execution_mode,
                rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                    Response::new(404, "Not Found"),
                    crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
                ),
            });
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
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                provider_registry
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
                    ),
                crate::runtime::request_outcome::ResponseKind::Maze,
            ),
        });
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
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::PathAllowlistBypass,
            execution_mode: request_effect_context.execution_mode,
            rendered: forward_allow_response("path_allowlist"),
        });
    }
    // IP/CIDR allowlist
    if cfg.bypass_allowlists_enabled && crate::signals::allowlist::is_allowlisted(&ip, &cfg.allowlist) {
        execute_request_intents(vec![crate::increment_metric_intent(
            crate::observability::metrics::MetricName::AllowlistedTotal,
            None,
        )]);
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::IpAllowlistBypass,
            execution_mode: request_effect_context.execution_mode,
            rendered: forward_allow_response("ip_allowlist"),
        });
    }
    let ip_range_evaluation = crate::signals::ip_range_policy::evaluate(&cfg, &ip);
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
        verified_identity.as_ref(),
        &request_capabilities,
    ) {
        return finalize_handled_response(response);
    }
    // PoW endpoints (public, before JS verification)
    if path == "/pow" {
        if *req.method() != spin_sdk::http::Method::Get {
            return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
                branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
                execution_mode: request_effect_context.execution_mode,
                rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                    Response::new(405, "Method Not Allowed"),
                    crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
                ),
            });
        }
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                provider_registry
                    .challenge_engine_provider()
                    .handle_pow_challenge(
                        &ip,
                        ua,
                        cfg.pow_enabled,
                        cfg.pow_difficulty,
                        cfg.pow_ttl_seconds,
                    ),
                crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
            ),
        });
    }
    if path == "/pow/verify" {
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::DefenceFollowup,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                provider_registry
                    .challenge_engine_provider()
                    .handle_pow_verify(req, &ip, cfg.pow_enabled),
                crate::runtime::request_outcome::ResponseKind::DefenceFollowupResponse,
            ),
        });
    }
    if let Some(response) =
        crate::runtime::policy_pipeline::maybe_handle_policy_graph_verified_identity_tranche(
            req,
            store,
            &cfg,
            &provider_registry,
            site_id,
            &ip,
            ua,
            &geo_assessment,
            &ip_range_evaluation,
            verified_identity.as_ref(),
            &request_capabilities,
        )
    {
        return finalize_handled_response(response);
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
        verified_identity.as_ref(),
        &request_capabilities,
    ) {
        return finalize_handled_response(response);
    }

    if let Some(response) = crate::runtime::sim_public::maybe_handle(req, path, &cfg) {
        if *response.status() == 200u16 {
            record_allow_clean();
        }
        return finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
            branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::SimPublic,
            execution_mode: request_effect_context.execution_mode,
            rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                response,
                crate::runtime::request_outcome::ResponseKind::SimPublicResponse,
            ),
        });
    }

    record_allow_clean();
    finalize_handled_response(crate::runtime::request_outcome::HandledRequestResponse {
        branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::CleanAllow {
            not_a_bot_marker_valid: crate::challenge::has_valid_not_a_bot_marker(req, &ip, ua),
        },
        execution_mode: request_effect_context.execution_mode,
        rendered: forward_allow_response("policy_clean_allow"),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        clean_allow_monitoring_intents, finalize_request_outcome, observe_verified_identity_intents,
        observe_verified_identity_result,
    };

    #[test]
    fn clean_allow_monitoring_intents_skip_live_inference_for_adversary_sim_origin() {
        let intents = clean_allow_monitoring_intents(
            crate::runtime::request_outcome::TrafficOrigin::AdversarySim,
            "/sim/public/landing",
            25,
            true,
        );

        assert!(intents.is_empty());
    }

    #[test]
    fn clean_allow_monitoring_intents_preserve_live_signals() {
        let intents = clean_allow_monitoring_intents(
            crate::runtime::request_outcome::TrafficOrigin::Live,
            "/pricing",
            10,
            true,
        );

        assert!(matches!(
            intents.first(),
            Some(crate::runtime::effect_intents::EffectIntent::RecordPolicyMatch(
                crate::runtime::policy_taxonomy::PolicyTransition::AllowClean
            ))
        ));
        assert!(matches!(
            intents.get(1),
            Some(
                crate::runtime::effect_intents::EffectIntent::RecordLikelyHumanSample {
                    sample_percent: 10,
                    sample_hint
                }
            ) if sample_hint == "/pricing"
        ));
        assert!(matches!(
            intents.get(2),
            Some(crate::runtime::effect_intents::EffectIntent::RecordShadowPassThrough)
        ));
        assert_eq!(intents.len(), 3);
    }

    #[test]
    fn bootstrap_failure_handled_response_uses_control_plane_outcome_contract() {
        let handled = super::bootstrap_failure_handled_response(spin_sdk::http::Response::new(
            500,
            "Configuration unavailable",
        ));

        assert!(matches!(
            handled.branch,
            crate::runtime::traffic_classification::CurrentRuntimeBranch::BootstrapFailure
        ));
        assert_eq!(
            handled.execution_mode,
            crate::runtime::effect_intents::ExecutionMode::Enforced
        );
        assert_eq!(
            handled.rendered.response_kind,
            crate::runtime::request_outcome::ResponseKind::ControlPlaneResponse
        );
        assert!(!handled.rendered.forward_attempted);
        assert!(handled.rendered.forward_failure_class.is_none());
    }

    #[test]
    fn finalize_request_outcome_records_bootstrap_failures_under_control_scope() {
        let store = crate::test_support::InMemoryStore::default();
        let capabilities =
            crate::runtime::capabilities::RuntimeCapabilities::for_test_policy_execution_phase();
        let response = finalize_request_outcome(
            &store,
            &capabilities,
            crate::runtime::request_outcome::TrafficOrigin::Live,
            super::bootstrap_failure_handled_response(spin_sdk::http::Response::new(
                500,
                "Configuration unavailable",
            )),
            None,
            None,
        );

        assert_eq!(*response.status(), 500);

        let summary = crate::observability::monitoring::summarize_with_store(&store, 24, 10);
        let control_scope = summary
            .request_outcomes
            .by_scope
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "bypass_and_control"
                    && row.execution_mode == "enforced"
            })
            .expect("bootstrap failure scope row");
        assert_eq!(control_scope.total_requests, 1);
        assert_eq!(control_scope.control_response_requests, 1);
    }

    #[test]
    fn finalize_request_outcome_surfaces_verified_identity_lane_in_monitoring_context() {
        let store = crate::test_support::InMemoryStore::default();
        let capabilities =
            crate::runtime::capabilities::RuntimeCapabilities::for_test_policy_execution_phase();
        let response = finalize_request_outcome(
            &store,
            &capabilities,
            crate::runtime::request_outcome::TrafficOrigin::Live,
            crate::runtime::request_outcome::HandledRequestResponse {
                branch: crate::runtime::traffic_classification::CurrentRuntimeBranch::PolicyDecision(
                    crate::runtime::policy_graph::PolicyDecision::BotnessChallenge {
                        score: 91,
                        signal_ids: vec![],
                    },
                ),
                execution_mode: crate::runtime::effect_intents::ExecutionMode::Enforced,
                rendered: crate::runtime::request_outcome::RenderedResponseEvidence::local(
                    spin_sdk::http::Response::new(403, "challenge"),
                    crate::runtime::request_outcome::ResponseKind::Challenge,
                ),
            },
            Some(crate::runtime::request_outcome::RequestOutcomeLane {
                lane: crate::runtime::traffic_classification::TrafficLane::SignedAgent,
                exactness: crate::observability::hot_read_contract::TelemetryExactness::Exact,
                basis: crate::observability::hot_read_contract::TelemetryBasis::Observed,
            }),
            Some(crate::runtime::traffic_classification::NonHumanCategoryAssignment {
                category_id: crate::runtime::non_human_taxonomy::NonHumanCategoryId::AgentOnBehalfOfHuman,
                assignment_status: "classified",
            }),
        );

        assert_eq!(*response.status(), 403);

        let summary = crate::observability::monitoring::summarize_with_store(&store, 24, 10);
        let row = summary
            .request_outcomes
            .by_lane
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
                    && row.lane == "signed_agent"
            })
            .expect("signed agent lane row");
        assert_eq!(row.short_circuited_requests, 1);
        let category_row = summary
            .request_outcomes
            .by_non_human_category
            .iter()
            .find(|row| {
                row.traffic_origin == "live"
                    && row.measurement_scope == "ingress_primary"
                    && row.execution_mode == "enforced"
                    && row.category_id == "agent_on_behalf_of_human"
            })
            .expect("signed agent category row");
        assert_eq!(category_row.short_circuited_requests, 1);
    }

    #[test]
    fn observe_verified_identity_intents_skip_non_attempted_requests() {
        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        let registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
        let req = crate::test_support::request_with_headers("/", &[]);
        let store = crate::test_support::InMemoryStore::default();

        let result = observe_verified_identity_result(&store, "default", &req, &cfg, &registry);
        let intents = observe_verified_identity_intents(
            crate::bot_identity::contracts::IdentityProvenance::Native,
            &result,
        );

        assert!(intents.is_empty());
    }

    #[test]
    fn observe_verified_identity_intents_emit_provider_telemetry_for_trusted_assertions() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");

        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        let registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
        let store = crate::test_support::InMemoryStore::default();
        let req = crate::test_support::request_with_headers(
            "/",
            &[
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                ("x-shuma-edge-verified-identity-scheme", "provider_signed_agent"),
                ("x-shuma-edge-verified-identity", "chatgpt-agent"),
                ("x-shuma-edge-verified-identity-operator", "openai"),
                (
                    "x-shuma-edge-verified-identity-category",
                    "user_triggered_agent",
                ),
                (
                    "x-shuma-edge-verified-identity-end-user-controlled",
                    "true",
                ),
            ],
        );

        let result = observe_verified_identity_result(&store, "default", &req, &cfg, &registry);
        let intents = observe_verified_identity_intents(
            crate::bot_identity::contracts::IdentityProvenance::Provider,
            &result,
        );

        assert!(matches!(
            intents.first(),
            Some(
                crate::runtime::effect_intents::EffectIntent::RecordVerifiedIdentityTelemetry {
                    record
                }
            ) if record.provenance
                == crate::bot_identity::contracts::IdentityProvenance::Provider
                && record.operator.as_deref() == Some("openai")
                && record.stable_identity.as_deref() == Some("chatgpt-agent")
        ));
        assert_eq!(intents.len(), 1);
    }

    #[test]
    fn observe_verified_identity_intents_record_monitoring_when_executed() {
        let _lock = crate::test_support::lock_env();
        std::env::set_var("SHUMA_FORWARDED_IP_SECRET", "test-forwarded-secret");

        let mut cfg = crate::config::defaults().clone();
        cfg.verified_identity.enabled = true;
        cfg.edge_integration_mode = crate::config::EdgeIntegrationMode::Additive;
        cfg.provider_backends.fingerprint_signal = crate::config::ProviderBackend::External;
        let registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
        let store = crate::test_support::InMemoryStore::default();
        let req = crate::test_support::request_with_headers(
            "/",
            &[
                ("x-shuma-forwarded-secret", "test-forwarded-secret"),
                (
                    "x-shuma-edge-verified-identity-scheme",
                    "provider_verified_bot",
                ),
                ("x-shuma-edge-verified-identity", "search.example"),
                ("x-shuma-edge-verified-identity-operator", "example"),
                ("x-shuma-edge-verified-identity-category", "search"),
            ],
        );
        let result = observe_verified_identity_result(&store, "default", &req, &cfg, &registry);
        let intents = observe_verified_identity_intents(
            crate::bot_identity::contracts::IdentityProvenance::Provider,
            &result,
        );
        let capabilities =
            crate::runtime::capabilities::RuntimeCapabilities::for_test_policy_execution_phase();

        crate::runtime::effect_intents::execute_request_outcome_intents(
            intents,
            &store,
            &capabilities,
        );

        let summary = crate::observability::monitoring::summarize_with_store(&store, 24, 10);
        assert_eq!(summary.verified_identity.attempts, 1);
        assert_eq!(summary.verified_identity.verified, 1);
        assert_eq!(summary.verified_identity.failed, 0);
    }

    #[test]
    fn observe_verified_identity_intents_preserve_native_provenance_for_failed_results() {
        let intents = observe_verified_identity_intents(
            crate::bot_identity::contracts::IdentityProvenance::Native,
            &crate::bot_identity::verification::IdentityVerificationResult::failed(
                crate::bot_identity::verification::IdentityVerificationFailure::SignatureInvalid,
                crate::bot_identity::verification::IdentityVerificationFreshness::Fresh,
            ),
        );

        assert!(matches!(
            intents.first(),
            Some(
                crate::runtime::effect_intents::EffectIntent::RecordVerifiedIdentityTelemetry {
                    record
                }
            ) if record.provenance
                == crate::bot_identity::contracts::IdentityProvenance::Native
                && record.failure
                    == Some(crate::bot_identity::verification::IdentityVerificationFailure::SignatureInvalid)
        ));
    }
}
