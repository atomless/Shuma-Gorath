const PURE_DECISION_MODULES: &[(&str, &str)] = &[
    ("request_facts.rs", include_str!("request_facts.rs")),
    ("policy_graph.rs", include_str!("policy_graph.rs")),
    ("policy_taxonomy.rs", include_str!("policy_taxonomy.rs")),
    (
        "effect_intents/plan_builder.rs",
        include_str!("effect_intents/plan_builder.rs"),
    ),
];

const FORBIDDEN_RUNTIME_COUPLINGS: &[&str] = &[
    "Store",
    "ProviderRegistry",
    "provider_registry",
    "rate_limiter_provider(",
    "ban_store_provider(",
    "challenge_engine_provider(",
    "maze_tarpit_provider(",
    "fingerprint_signal_provider(",
    "admin::log_event(",
    "observability::metrics::increment(",
    "observability::metrics::record_",
    "observability::monitoring::record_",
    "observability::monitoring::flush_pending_counters(",
];

const FORBIDDEN_MUTABLE_GLOBAL_PATTERNS: &[&str] =
    &["static mut", "thread_local!", "Lazy<Mutex"];

#[test]
fn pure_decision_modules_do_not_depend_on_runtime_side_effect_surfaces() {
    for (module, source) in PURE_DECISION_MODULES {
        for forbidden in FORBIDDEN_RUNTIME_COUPLINGS {
            assert!(
                !source.contains(forbidden),
                "pure decision module {module} must not reference forbidden coupling `{forbidden}`"
            );
        }
    }
}

#[test]
fn pure_decision_modules_do_not_introduce_mutable_global_state_patterns() {
    for (module, source) in PURE_DECISION_MODULES {
        for forbidden in FORBIDDEN_MUTABLE_GLOBAL_PATTERNS {
            assert!(
                !source.contains(forbidden),
                "pure decision module {module} must not contain mutable global pattern `{forbidden}`"
            );
        }
    }
}
