use std::fs;
use std::path::{Path, PathBuf};

const PURE_DECISION_MODULES: &[(&str, &str)] = &[
    ("request_facts.rs", include_str!("request_facts.rs")),
    ("policy_graph.rs", include_str!("policy_graph.rs")),
    ("policy_taxonomy.rs", include_str!("policy_taxonomy.rs")),
    ("request_outcome.rs", include_str!("request_outcome.rs")),
    (
        "traffic_classification.rs",
        include_str!("traffic_classification.rs"),
    ),
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

const FORBIDDEN_PRIVILEGED_WRITE_PATTERNS: &[&str] = &[
    "admin::log_event(",
    "observability::metrics::increment(",
    "observability::metrics::record_",
    "observability::monitoring::record_",
    "observability::monitoring::flush_pending_counters(",
    "ban_ip_with_fingerprint(",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn collect_sources_under(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read directory {}: {err}", normalize_path(dir));
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!(
                "failed to read directory entry under {}: {err}",
                normalize_path(dir)
            )
        });
        let path = entry.path();
        if path.is_dir() {
            collect_sources_under(&path, root, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let rel = normalize_path(path.strip_prefix(root).expect("path must be under root"));
        if rel == "src/runtime/architecture_guards.rs"
            || rel.ends_with("/tests.rs")
            || rel.contains("/tests/")
        {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read source file {}: {err}", normalize_path(&path));
        });
        out.push((rel, source));
    }
}

fn runtime_and_lib_sources() -> Vec<(String, String)> {
    let root = repo_root();
    let mut out = Vec::new();

    let lib_path = root.join("src/lib.rs");
    let lib_source = fs::read_to_string(&lib_path).unwrap_or_else(|err| {
        panic!(
            "failed to read source file {}: {err}",
            normalize_path(&lib_path)
        )
    });
    out.push(("src/lib.rs".to_string(), lib_source));

    collect_sources_under(&root.join("src/runtime"), &root, &mut out);
    out
}

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

#[test]
fn privileged_write_helpers_are_confined_to_effect_intent_executor_modules() {
    let sources = runtime_and_lib_sources();
    for (module, source) in sources {
        if module.starts_with("src/runtime/effect_intents/") {
            continue;
        }
        for forbidden in FORBIDDEN_PRIVILEGED_WRITE_PATTERNS {
            assert!(
                !source.contains(forbidden),
                "module {module} must route privileged write `{forbidden}` through effect-intent executor APIs"
            );
        }
    }
}

#[test]
fn capability_minting_is_confined_to_trust_boundary_entrypoints() {
    let sources = runtime_and_lib_sources();
    let mut bootstrap_mint_paths = Vec::new();
    let mut policy_mint_paths = Vec::new();
    let mut flush_mint_paths = Vec::new();

    for (module, source) in &sources {
        if source.contains("for_request_bootstrap_phase(") {
            bootstrap_mint_paths.push(module.as_str());
        }
        if source.contains("for_policy_execution_phase(") {
            policy_mint_paths.push(module.as_str());
        }
        if source.contains("for_post_response_flush_phase(") {
            flush_mint_paths.push(module.as_str());
        }
    }

    bootstrap_mint_paths.sort_unstable();
    policy_mint_paths.sort_unstable();
    flush_mint_paths.sort_unstable();

    assert_eq!(
        bootstrap_mint_paths,
        vec!["src/runtime/capabilities.rs", "src/runtime/request_flow.rs"]
    );
    assert_eq!(
        policy_mint_paths,
        vec![
            "src/lib.rs",
            "src/runtime/capabilities.rs",
            "src/runtime/request_flow.rs",
        ]
    );
    assert_eq!(
        flush_mint_paths,
        vec!["src/lib.rs", "src/runtime/capabilities.rs"]
    );
}

#[test]
fn privileged_effect_executors_require_phase_specific_capability_types() {
    let _metrics_executor: fn(
        Vec<crate::runtime::effect_intents::EffectIntent>,
        &spin_sdk::key_value::Store,
        &crate::runtime::capabilities::RequestBootstrapCapabilities,
    ) = crate::runtime::effect_intents::execute_metric_intents;

    let _monitoring_executor: fn(
        Vec<crate::runtime::effect_intents::EffectIntent>,
        &spin_sdk::key_value::Store,
        &crate::runtime::capabilities::PostResponseFlushCapabilities,
    ) = crate::runtime::effect_intents::execute_monitoring_store_intents;

    let _request_outcome_executor: fn(
        Vec<crate::runtime::effect_intents::EffectIntent>,
        &spin_sdk::key_value::Store,
        &crate::runtime::capabilities::PolicyExecutionCapabilities,
    ) = crate::runtime::effect_intents::execute_request_outcome_intents;

    let _effect_executor: for<'a> fn(
        Vec<crate::runtime::effect_intents::EffectIntent>,
        &crate::runtime::effect_intents::EffectExecutionContext<'a>,
        &crate::runtime::capabilities::PolicyExecutionCapabilities,
        Option<crate::runtime::effect_intents::ShadowAction>,
    ) = crate::runtime::effect_intents::execute_effect_intents;
}
