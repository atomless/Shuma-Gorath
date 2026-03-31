use spin_sdk::http::{Method, Request, Response};
use spin_sdk::key_value::Store;

use crate::http_route_namespace as route_namespace;

const SITE_ID_DEFAULT: &str = "default";

fn request_user_agent(req: &Request) -> &str {
    req.header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("")
}

fn execute_capability_gated_intents(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    ip: &str,
    ua: &str,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
    intents: Vec<crate::runtime::effect_intents::EffectIntent>,
) {
    let context = crate::runtime::effect_intents::EffectExecutionContext {
        req,
        store,
        cfg,
        provider_registry,
        site_id: SITE_ID_DEFAULT,
        ip,
        ua,
        execution_mode: crate::runtime::shadow_mode::effective_execution_mode(cfg),
    };
    crate::runtime::effect_intents::execute_effect_intents(intents, &context, &capabilities, None);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChallengeFailureEnforcement {
    MazeFallback,
    TarpitOrShortBan,
}

fn classify_not_a_bot_failure_enforcement(
    outcome: crate::challenge::NotABotSubmitOutcome,
) -> ChallengeFailureEnforcement {
    match outcome {
        crate::challenge::NotABotSubmitOutcome::FailedScore
        | crate::challenge::NotABotSubmitOutcome::Expired
        | crate::challenge::NotABotSubmitOutcome::MazeOrBlock => {
            ChallengeFailureEnforcement::MazeFallback
        }
        crate::challenge::NotABotSubmitOutcome::Replay
        | crate::challenge::NotABotSubmitOutcome::InvalidSeed
        | crate::challenge::NotABotSubmitOutcome::MissingSeed
        | crate::challenge::NotABotSubmitOutcome::SequenceViolation
        | crate::challenge::NotABotSubmitOutcome::BindingMismatch
        | crate::challenge::NotABotSubmitOutcome::InvalidTelemetry
        | crate::challenge::NotABotSubmitOutcome::AttemptLimitExceeded => {
            ChallengeFailureEnforcement::TarpitOrShortBan
        }
        crate::challenge::NotABotSubmitOutcome::Pass
        | crate::challenge::NotABotSubmitOutcome::EscalatePuzzle => {
            ChallengeFailureEnforcement::MazeFallback
        }
    }
}

fn classify_challenge_failure_enforcement(
    outcome: crate::boundaries::ChallengeSubmitOutcome,
) -> Option<ChallengeFailureEnforcement> {
    match outcome {
        crate::boundaries::ChallengeSubmitOutcome::Solved => None,
        crate::boundaries::ChallengeSubmitOutcome::Incorrect
        | crate::boundaries::ChallengeSubmitOutcome::SequenceOpExpired
        | crate::boundaries::ChallengeSubmitOutcome::SequenceWindowExceeded
        | crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooSlow => {
            Some(ChallengeFailureEnforcement::MazeFallback)
        }
        crate::boundaries::ChallengeSubmitOutcome::AttemptLimitExceeded
        | crate::boundaries::ChallengeSubmitOutcome::SequenceOpMissing
        | crate::boundaries::ChallengeSubmitOutcome::SequenceOpInvalid
        | crate::boundaries::ChallengeSubmitOutcome::SequenceOpReplay
        | crate::boundaries::ChallengeSubmitOutcome::SequenceOrderViolation
        | crate::boundaries::ChallengeSubmitOutcome::SequenceBindingMismatch
        | crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooFast
        | crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooRegular
        | crate::boundaries::ChallengeSubmitOutcome::Forbidden
        | crate::boundaries::ChallengeSubmitOutcome::InvalidOutput => {
            Some(ChallengeFailureEnforcement::TarpitOrShortBan)
        }
    }
}

fn render_maze_or_block_failure(
    req: &Request,
    store: &Store,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    ip: &str,
    ua: &str,
    maze_path_suffix: &str,
    event_reason: &str,
    event_outcome: &str,
) -> Response {
    if cfg.maze_enabled {
        return provider_registry
            .maze_tarpit_provider()
            .serve_maze_with_tracking(
                req,
                store,
                cfg,
                ip,
                ua,
                crate::maze::entry_path(maze_path_suffix).as_str(),
                event_reason,
                event_outcome,
                None,
            );
    }
    Response::new(
        403,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::GeoPolicy,
        ),
    )
}

fn enforce_tarpit_or_short_ban(
    store: &Store,
    req: &Request,
    cfg: &crate::config::Config,
    provider_registry: &crate::providers::registry::ProviderRegistry,
    ip: &str,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
    ban_reason: &str,
    summary: &str,
    signals: &[&str],
) -> Response {
    if let Some(tarpit_response) = provider_registry.maze_tarpit_provider().maybe_handle_tarpit(
        req,
        store,
        cfg,
        SITE_ID_DEFAULT,
        ip,
    ) {
        return tarpit_response;
    }

    execute_capability_gated_intents(
        req,
        store,
        cfg,
        provider_registry,
        ip,
        request_user_agent(req),
        capabilities,
        vec![
            crate::runtime::effect_intents::EffectIntent::Ban(
                crate::runtime::effect_intents::BanIntent {
                    reason: ban_reason.to_string(),
                    duration_seconds: cfg.get_ban_duration(ban_reason),
                    score: None,
                    signals: signals.iter().map(|value| (*value).to_string()).collect(),
                    summary: Some(summary.to_string()),
                },
            ),
            crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::BansTotal,
                label: Some(ban_reason.to_string()),
            },
            crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                metric: crate::observability::metrics::MetricName::BlocksTotal,
                label: None,
            },
            crate::runtime::effect_intents::EffectIntent::LogEvent {
                event: crate::admin::EventType::Ban,
                reason: ban_reason.to_string(),
                outcome: format!("short_ban_{}s", cfg.get_ban_duration(ban_reason)),
            },
        ],
    );

    Response::new(
        403,
        crate::enforcement::block_page::render_block_page(
            crate::enforcement::block_page::BlockReason::GeoPolicy,
        ),
    )
}

fn sequence_violation_intents_for_challenge_submit(
    transition: crate::runtime::policy_taxonomy::PolicyTransition,
    reason: &str,
) -> Vec<crate::runtime::effect_intents::EffectIntent> {
    vec![
        crate::runtime::effect_intents::EffectIntent::RecordPolicyMatch(transition),
        crate::runtime::effect_intents::EffectIntent::LogEvent {
            event: crate::admin::EventType::Challenge,
            reason: reason.to_string(),
            outcome: "challenge_submit_rejected".to_string(),
        },
    ]
}

fn handle_not_a_bot_submit(
    store: &Store,
    req: &Request,
    cfg: &crate::config::Config,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
) -> Response {
    let submit_result = crate::boundaries::handle_not_a_bot_submit_with_outcome(store, req, cfg);
    let provider_registry = crate::providers::registry::ProviderRegistry::from_config(cfg);
    let ip = crate::extract_client_ip(req);
    let ua = req
        .header("user-agent")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let monitoring_outcome = match submit_result.outcome {
        crate::challenge::NotABotSubmitOutcome::Pass => "pass",
        crate::challenge::NotABotSubmitOutcome::EscalatePuzzle => "escalate",
        crate::challenge::NotABotSubmitOutcome::Replay => "replay",
        _ => "fail",
    };
    execute_capability_gated_intents(
        req,
        store,
        cfg,
        &provider_registry,
        ip.as_str(),
        ua,
        capabilities,
        vec![crate::runtime::effect_intents::EffectIntent::RecordNotABotSubmit {
            outcome: monitoring_outcome.to_string(),
            solve_ms: submit_result.solve_ms,
        }],
    );

    match submit_result.decision {
        crate::challenge::NotABotDecision::Pass => {
            execute_capability_gated_intents(
                req,
                store,
                cfg,
                &provider_registry,
                ip.as_str(),
                ua,
                capabilities,
                vec![
                    crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::NotABotPassTotal,
                        label: None,
                    },
                    crate::runtime::effect_intents::EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "not_a_bot_pass".to_string(),
                        outcome: format!(
                            "return_to={} solve_ms={}",
                            submit_result.return_to,
                            submit_result.solve_ms.unwrap_or_default()
                        ),
                    },
                ],
            );
            let mut builder = Response::builder();
            builder.status(303);
            builder.header("Location", submit_result.return_to.as_str());
            builder.header("Cache-Control", "no-store");
            if let Some(marker_cookie) = submit_result.marker_cookie {
                builder.header("Set-Cookie", marker_cookie.as_str());
            }
            builder.body(Vec::new()).build()
        }
        crate::challenge::NotABotDecision::EscalatePuzzle => {
            execute_capability_gated_intents(
                req,
                store,
                cfg,
                &provider_registry,
                ip.as_str(),
                ua,
                capabilities,
                vec![
                    crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::NotABotEscalateTotal,
                        label: None,
                    },
                    crate::runtime::effect_intents::EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "not_a_bot_escalate_puzzle".to_string(),
                        outcome: format!("{:?}", submit_result.outcome),
                    },
                ],
            );
            if cfg.challenge_puzzle_enabled {
                execute_capability_gated_intents(
                    req,
                    store,
                    cfg,
                    &provider_registry,
                    ip.as_str(),
                    ua,
                    capabilities,
                    vec![
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengesTotal,
                            label: None,
                        },
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                            label: None,
                        },
                    ],
                );
                return provider_registry
                    .challenge_engine_provider()
                    .render_challenge(
                        req,
                        cfg.challenge_puzzle_transform_count as usize,
                        cfg.challenge_puzzle_seed_ttl_seconds,
                    );
            }
            if cfg.maze_enabled {
                return provider_registry
                    .maze_tarpit_provider()
                    .serve_maze_with_tracking(
                        req,
                        store,
                        cfg,
                        ip.as_str(),
                        ua,
                        crate::maze::entry_path("not-a-bot-escalate-fallback").as_str(),
                        "not_a_bot_escalate_puzzle_fallback_maze",
                        "not_a_bot_escalate_puzzle challenge_disabled",
                        None,
                    );
            }
            Response::new(
                403,
                crate::enforcement::block_page::render_block_page(
                    crate::enforcement::block_page::BlockReason::GeoPolicy,
                ),
            )
        }
        crate::challenge::NotABotDecision::MazeOrBlock => {
            let mut intents = vec![
                crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                    metric: crate::observability::metrics::MetricName::NotABotFailTotal,
                    label: None,
                },
            ];
            if submit_result.outcome == crate::challenge::NotABotSubmitOutcome::Replay {
                intents.push(crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                    metric: crate::observability::metrics::MetricName::NotABotReplayTotal,
                    label: None,
                });
            }
            intents.push(crate::runtime::effect_intents::EffectIntent::LogEvent {
                event: crate::admin::EventType::Challenge,
                reason: "not_a_bot_fail".to_string(),
                outcome: format!("{:?}", submit_result.outcome),
            });
            execute_capability_gated_intents(
                req,
                store,
                cfg,
                &provider_registry,
                ip.as_str(),
                ua,
                capabilities,
                intents,
            );
            match classify_not_a_bot_failure_enforcement(submit_result.outcome) {
                ChallengeFailureEnforcement::MazeFallback => {
                    let event_outcome = format!("{:?}", submit_result.outcome);
                    render_maze_or_block_failure(
                        req,
                        store,
                        cfg,
                        &provider_registry,
                        ip.as_str(),
                        ua,
                        "not-a-bot-fail",
                        "not_a_bot_submit_fail_maze",
                        event_outcome.as_str(),
                    )
                }
                ChallengeFailureEnforcement::TarpitOrShortBan => {
                    if crate::runtime::shadow_mode::shadow_mode_active(cfg) {
                        let event_outcome = format!("{:?} shadow_mode_no_ban", submit_result.outcome);
                        return render_maze_or_block_failure(
                            req,
                            store,
                            cfg,
                            &provider_registry,
                            ip.as_str(),
                            ua,
                            "not-a-bot-abuse-shadow-mode",
                            "not_a_bot_submit_abuse_shadow_mode_maze",
                            event_outcome.as_str(),
                        );
                    }
                    let summary = format!("outcome={:?}", submit_result.outcome);
                    enforce_tarpit_or_short_ban(
                        store,
                        req,
                        cfg,
                        &provider_registry,
                        ip.as_str(),
                        capabilities,
                        "not_a_bot_abuse",
                        summary.as_str(),
                        &["not_a_bot_abuse"],
                    )
                }
            }
        }
    }
}

pub(crate) fn maybe_handle_early_route(
    req: &Request,
    path: &str,
    capabilities: &crate::runtime::capabilities::PolicyExecutionCapabilities,
) -> Option<Response> {
    if let Some(response) = crate::maze::assets::maybe_handle_asset(path, req.method()) {
        return Some(response);
    }

    if route_namespace::is_shuma_dashboard_root_path(path)
        && (*req.method() == Method::Get || *req.method() == Method::Head)
    {
        return Some(
            Response::builder()
                .status(308)
                .header("Location", route_namespace::SHUMA_DASHBOARD_INDEX_PATH)
                .header("Cache-Control", "no-store, max-age=0, must-revalidate")
                .body(Vec::new())
                .build(),
        );
    }

    // Health check endpoint
    if path == route_namespace::SHUMA_HEALTH_PATH {
        if !crate::health_secret_authorized(req) {
            return Some(Response::new(403, "Forbidden"));
        }
        let allowed = ["127.0.0.1", "::1"];
        let ip = crate::extract_health_client_ip(req);
        if !allowed.contains(&ip.as_str()) {
            return Some(Response::new(403, "Forbidden"));
        }
        let fail_open = crate::shuma_fail_open();
        let mode = crate::fail_mode_label(fail_open);
        if let Ok(store) = Store::open_default() {
            let test_key = "health:test";
            if let Err(e) = store.set(test_key, b"ok") {
                crate::log_line(&format!(
                    "[health] failed to write KV probe key {}: {:?}",
                    test_key, e
                ));
            }
            let ok = store.get(test_key).is_ok();
            if let Err(e) = store.delete(test_key) {
                crate::log_line(&format!(
                    "[health] failed to cleanup KV probe key {}: {:?}",
                    test_key, e
                ));
            }
            if ok {
                return Some(crate::response_with_optional_debug_headers(
                    200,
                    "OK",
                    "available",
                    mode,
                ));
            }
        }
        crate::log_line(&format!(
            "[KV OUTAGE] Key-value store unavailable; SHUMA_KV_STORE_FAIL_OPEN={}",
            fail_open
        ));
        return Some(crate::response_with_optional_debug_headers(
            500,
            "Key-value store error",
            "unavailable",
            mode,
        ));
    }

    if path == crate::boundaries::challenge_not_a_bot_path() && *req.method() == Method::Post {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            return Some(handle_not_a_bot_submit(&store, req, &cfg, capabilities));
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    if path == crate::boundaries::challenge_not_a_bot_path() && *req.method() == Method::Get {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            let ip = crate::extract_client_ip(req);
            let ua = request_user_agent(req);
            let response = crate::boundaries::serve_not_a_bot_page(req, cfg.shadow_mode, &cfg);
            if *response.status() == 200 {
                execute_capability_gated_intents(
                    req,
                    &store,
                    &cfg,
                    &provider_registry,
                    ip.as_str(),
                    ua,
                    capabilities,
                    vec![
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::NotABotServedTotal,
                            label: None,
                        },
                        crate::runtime::effect_intents::EffectIntent::RecordNotABotServed,
                    ],
                );
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // Challenge POST handler
    if path == crate::boundaries::challenge_puzzle_path() && *req.method() == Method::Post {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            let (response, outcome) =
                crate::boundaries::handle_challenge_submit_with_outcome(
                    &store,
                    req,
                    cfg.challenge_puzzle_attempt_window_seconds,
                    cfg.challenge_puzzle_attempt_limit_per_window,
                );
            let challenge_ip = crate::extract_client_ip(req);
            let challenge_ua = request_user_agent(req);
            let outcome_intents = match outcome {
                crate::boundaries::ChallengeSubmitOutcome::Solved => vec![
                    crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeSolvedTotal,
                        label: None,
                    },
                    crate::runtime::effect_intents::EffectIntent::RecordIpRangeChallengeSolved,
                    crate::runtime::effect_intents::EffectIntent::LogEvent {
                        event: crate::admin::EventType::Challenge,
                        reason: "challenge_puzzle_pass".to_string(),
                        outcome: "Solved".to_string(),
                    },
                ],
                crate::boundaries::ChallengeSubmitOutcome::Incorrect => vec![
                    crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeIncorrectTotal,
                        label: None,
                    },
                    crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                        outcome: "incorrect".to_string(),
                    },
                ],
                crate::boundaries::ChallengeSubmitOutcome::AttemptLimitExceeded => vec![
                    crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                        outcome: "attempt_limit".to_string(),
                    },
                ],
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpMissing => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpMissing,
                        "challenge_submit_missing_operation_id",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpInvalid => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpInvalid,
                        "challenge_submit_invalid_operation_envelope",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpExpired => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                            label: None,
                        },
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "expired_replay".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpExpired,
                        "challenge_submit_expired_operation",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOpReplay => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                            label: None,
                        },
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "expired_replay".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOpReplay,
                        "challenge_submit_operation_replay",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceWindowExceeded => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                            metric: crate::observability::metrics::MetricName::ChallengeExpiredReplayTotal,
                            label: None,
                        },
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "expired_replay".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqWindowExceeded,
                        "challenge_submit_sequence_window_exceeded",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceOrderViolation => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqOrderViolation,
                        "challenge_submit_order_violation",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceBindingMismatch => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqBindingMismatch,
                        "challenge_submit_binding_mismatch",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooFast => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooFast,
                        "challenge_submit_timing_too_fast",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooRegular => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooRegular,
                        "challenge_submit_timing_too_regular",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::SequenceTimingTooSlow => {
                    let mut intents = vec![
                        crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                            outcome: "sequence_violation".to_string(),
                        },
                    ];
                    intents.extend(sequence_violation_intents_for_challenge_submit(
                        crate::runtime::policy_taxonomy::PolicyTransition::SeqTimingTooSlow,
                        "challenge_submit_timing_too_slow",
                    ));
                    intents
                }
                crate::boundaries::ChallengeSubmitOutcome::Forbidden => vec![
                    crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                        outcome: "forbidden".to_string(),
                    },
                ],
                crate::boundaries::ChallengeSubmitOutcome::InvalidOutput => vec![
                    crate::runtime::effect_intents::EffectIntent::RecordChallengeFailure {
                        outcome: "invalid_output".to_string(),
                    },
                ],
            };
            execute_capability_gated_intents(
                req,
                &store,
                &cfg,
                &provider_registry,
                challenge_ip.as_str(),
                challenge_ua,
                capabilities,
                outcome_intents,
            );
            if let Some(enforcement) = classify_challenge_failure_enforcement(outcome) {
                match enforcement {
                    ChallengeFailureEnforcement::MazeFallback => {
                        let event_outcome = format!("{:?}", outcome);
                        return Some(render_maze_or_block_failure(
                            req,
                            &store,
                            &cfg,
                            &provider_registry,
                            challenge_ip.as_str(),
                            challenge_ua,
                            "challenge-puzzle-fail",
                            "challenge_puzzle_submit_fail_maze",
                            event_outcome.as_str(),
                        ));
                    }
                    ChallengeFailureEnforcement::TarpitOrShortBan => {
                        if crate::runtime::shadow_mode::shadow_mode_active(&cfg) {
                            let event_outcome = format!("{:?} shadow_mode_no_ban", outcome);
                            return Some(render_maze_or_block_failure(
                                req,
                                &store,
                                &cfg,
                                &provider_registry,
                                challenge_ip.as_str(),
                                challenge_ua,
                                "challenge-puzzle-abuse-shadow-mode",
                                "challenge_puzzle_submit_abuse_shadow_mode_maze",
                                event_outcome.as_str(),
                            ));
                        }
                        let summary = format!("outcome={:?}", outcome);
                        return Some(enforce_tarpit_or_short_ban(
                            &store,
                            req,
                            &cfg,
                            &provider_registry,
                            challenge_ip.as_str(),
                            capabilities,
                            "challenge_puzzle_abuse",
                            summary.as_str(),
                            &["challenge_puzzle_abuse"],
                        ));
                    }
                }
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }
    if path == crate::boundaries::challenge_puzzle_path() && *req.method() == Method::Get {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
            let ip = crate::extract_client_ip(req);
            let ua = request_user_agent(req);
            let response = crate::boundaries::serve_challenge_page(
                req,
                cfg.shadow_mode,
                cfg.challenge_puzzle_transform_count as usize,
                cfg.challenge_puzzle_seed_ttl_seconds,
            );
            if *response.status() == 200 {
                execute_capability_gated_intents(
                    req,
                    &store,
                    &cfg,
                    &provider_registry,
                    ip.as_str(),
                    ua,
                    capabilities,
                    vec![crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::ChallengeServedTotal,
                        label: None,
                    }],
                );
            }
            return Some(response);
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // Prometheus metrics endpoint
    if path == route_namespace::SHUMA_METRICS_PATH {
        if let Ok(store) = Store::open_default() {
            return Some(crate::observability::metrics::handle_metrics(&store));
        }
        return Some(Response::new(500, "Key-value store error"));
    }

    // robots.txt - configurable AI crawler blocking
    if path == route_namespace::PUBLIC_ROBOTS_TXT_PATH {
        if let Ok(store) = Store::open_default() {
            let cfg = match crate::load_runtime_config(&store, "default", path) {
                Ok(cfg) => cfg,
                Err(resp) => return Some(resp),
            };
            if cfg.robots_enabled {
                let provider_registry = crate::providers::registry::ProviderRegistry::from_config(&cfg);
                let ip = crate::extract_client_ip(req);
                execute_capability_gated_intents(
                    req,
                    &store,
                    &cfg,
                    &provider_registry,
                    ip.as_str(),
                    request_user_agent(req),
                    capabilities,
                    vec![crate::runtime::effect_intents::EffectIntent::IncrementMetric {
                        metric: crate::observability::metrics::MetricName::RequestsTotal,
                        label: Some("robots_txt".to_string()),
                    }],
                );
                let public_origin = req.header("host").and_then(|value| value.as_str()).map(|host| {
                    let scheme = if crate::request_is_https(req) {
                        "https"
                    } else {
                        "http"
                    };
                    format!("{scheme}://{}", host.trim())
                });
                let content = crate::crawler_policy::robots::generate_robots_txt_for_public_origin(
                    &cfg,
                    public_origin.as_deref(),
                );
                let content_signal = crate::crawler_policy::robots::get_content_signal_header(&cfg);
                return Some(
                    Response::builder()
                        .status(200)
                        .header("Content-Type", "text/plain; charset=utf-8")
                        .header("Content-Signal", content_signal)
                        .header("Cache-Control", "no-store, no-cache, must-revalidate")
                        .body(content)
                        .build(),
                );
            }
        }
        // If disabled or store error, return 404
        return Some(Response::new(404, "Not Found"));
    }

    // Admin API
    if route_namespace::is_shuma_admin_path(path) {
        if req.method() == &Method::Options {
            return Some(Response::new(403, "Forbidden"));
        }
        return Some(crate::boundaries::handle_admin(req));
    }

    // Internal control-plane endpoints (host-side supervisor/ops tooling only)
    if path.starts_with("/internal/") {
        if req.method() == &Method::Options {
            return Some(Response::new(403, "Forbidden"));
        }
        return Some(crate::boundaries::handle_internal(req));
    }

    None
}

#[cfg(test)]
mod tests;
