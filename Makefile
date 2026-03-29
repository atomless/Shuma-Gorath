.PHONY: dev dev-prod local run run-prebuilt build build-runtime build-full-dev prod prod-start clean reset-local-state test test-unit unit-test test-native-build-warning-hygiene test-env-isolation-contract test-ci-workflow-action-versions test-tarpit-observability-contract test-tarpit-collateral-risk-contract test-verified-identity-contracts test-verified-identity-config test-verified-identity-provider test-verified-identity-native test-verified-identity-directory-discovery test-verified-identity-proxy-trust test-verified-identity-policy test-verified-identity-telemetry test-verified-identity-annotations test-verified-identity-calibration-readiness test-verified-identity-taxonomy-crosswalk test-verified-identity-alignment-receipts test-verified-identity-botness-conflicts test-verified-identity-guardrails test-verified-identity-make-target-contract test-host-impact-telemetry test-host-impact-benchmark test-oversight-host-impact test-host-impact-make-target-contract test-integration integration-test test-gateway-harness test-gateway-wasm-tls-harness test-gateway-origin-bypass-probe test-gateway-profile-shared-server test-gateway-profile-edge smoke-gateway-mode test-deploy-linode test-deploy-fermyon test-scrapling-deploy-shared-host test-config-lifecycle test-js-verification-unit test-runtime-preflight-unit test-runtime-preflight test-shadow-mode test-enterprise-ban-store-contract test-telemetry-storage test-telemetry-hot-read-contract test-telemetry-hot-read-projection test-telemetry-hot-read-bootstrap test-telemetry-hot-read-evidence test-telemetry-hot-read-live-evidence test-monitoring-telemetry-contract test-monitoring-telemetry-foundation-unit test-operator-snapshot-foundation test-traffic-taxonomy-contract test-traffic-classification-contract test-operator-objectives-contract test-operator-objectives-category-contract test-benchmark-category-eligibility test-oversight-reconcile test-oversight-agent test-oversight-episode-archive test-oversight-post-sim-trigger test-live-feedback-loop-remote test-live-feedback-loop-remote-unit test-live-feedback-loop-remote-contracts test-remote-target-contract test-setup-runtime-bootstrap test-admin-machine-contracts test-admin-api-routing-contract test-benchmark-suite-contract test-benchmark-results-contract test-benchmark-comparison-contract test-controller-mutability-policy test-controller-action-surface test-controller-action-surface-parity test-controller-hard-boundaries test-rsi-game-contract test-rsi-scorecard-contract test-rsi-score-exploit-progress test-rsi-score-evidence-quality test-rsi-score-urgency-and-homeostasis test-rsi-score-move-selection test-oversight-move-selection-policy telemetry-shared-host-evidence telemetry-fermyon-edge-evidence test-adversarial-python-unit test-adversarial-manifest test-adversarial-preflight test-adversarial-lane-contract test-shared-host-scope-contract test-shared-host-seed-contract build-shared-host-seed-inventory prepare-scrapling-deploy prepare-scrapling-local test-adversarial-sim-tag-contract test-adversarial-coverage-contract test-adversarial-coverage-receipts test-adversarial-scenario-review test-adversarial-scenario-intent-evidence-unit test-adversarial-sim-selftest test-adversarial-fast test-adversarial-smoke test-adversarial-abuse test-adversarial-akamai test-adversarial-coverage test-adversarial-soak test-adversarial-live test-remote-edge-signal-smoke test-fermyon-edge-signal-smoke telemetry-clean adversary-sim-supervisor-build adversary-sim-supervisor test-adversary-sim-supervisor-unit test-adversary-sim-domain-contract test-adversary-sim-make-target-contract test-adversary-sim-runtime-surface test-adversary-sim-runtime-surface-unit test-adversary-sim-scrapling-owned-surface-contract test-adversary-sim-scrapling-category-fit test-adversary-sim-scrapling-malicious-request-native test-adversary-sim-scrapling-coverage-receipts test-scrapling-game-loop-mainline test-adversary-sim-scrapling-worker test-adversary-sim-diagnostics-truth test-adversarial-llm-fit test-adversarial-llm-runtime-dispatch test-adversarial-llm-runtime-projection test-adversarial-repeatability test-adversarial-promote-candidates test-replay-promotion-contract test-protected-tuning-evidence test-adversarial-report-diff test-adversarial-runner-architecture test-adversarial-container-blackbox test-adversarial-container-isolation test-adversarial-frontier-attempt test-frontier-governance test-frontier-unavailability-policy test-frontier-unavailability-policy-unit test-sim2-realtime-bench test-sim2-adr-conformance test-sim2-ci-diagnostics test-sim2-verification-matrix test-sim2-verification-matrix-advisory test-sim2-operational-regressions test-sim2-operational-regressions-strict test-sim2-governance-contract test-sim2-verification-e2e test-ip-range-suggestions test-testing-surface-artifact-path-contract test-make-selector-contract-targets test-coverage test-dashboard test-dashboard-svelte-check test-dashboard-unit test-dashboard-config-surface-contract test-dashboard-adversary-sim-lane-contract test-dashboard-auth-gate test-dashboard-tab-information-architecture test-dashboard-game-loop-accountability test-dashboard-traffic-pane test-dashboard-diagnostics-pane test-dashboard-runtime-unit-contracts test-dashboard-ip-bans-refresh-contract test-dashboard-policy-pane-ownership test-dashboard-verified-identity-pane test-dashboard-red-team-truth-basis test-dashboard-scrapling-evidence test-dashboard-e2e-tab-information-architecture test-dashboard-e2e-policy-pane-ownership test-dashboard-e2e-tab-state-transitions test-dashboard-budgets test-dashboard-budgets-strict test-dashboard-e2e test-dashboard-e2e-adversary-sim test-dashboard-e2e-red-team-frontier-warning test-dashboard-e2e-external seed-dashboard-data test-maze-benchmark test-maze-verification-wiring test-maze-verification-gate test-maze-live-traversal-unit test-maze-live-traversal-contract test-maze-live-browser-unit test-maze-live-browser-contract test-maze-state-concurrency-contract spin-wait-ready smoke-single-host prepare-linode-shared-host prepare-fermyon-akamai-edge remote-use remote-update remote-start remote-stop remote-status remote-logs remote-open-dashboard deploy deploy-profile-baseline deploy-self-hosted-minimal deploy-enterprise-akamai deploy-linode-one-shot deploy-fermyon-akamai-edge logs status stop help setup setup-runtime verify verify-runtime config-seed config-verify dashboard-build dashboard-verify-freshness env-help api-key-generate gen-admin-api-key api-key-show api-key-rotate api-key-validate deploy-env-validate test-rsi-game-mixed-episode-orchestration test-rsi-game-mixed-restriction-score-spine

# Default target
.DEFAULT_GOAL := help

# Colors for output
GREEN := \033[0;32m
YELLOW := \033[1;33m
CYAN := \033[0;36m
RED := \033[0;31m
NC := \033[0m

WASM_BUILD_OUTPUT := target/wasm32-wasip1/release/shuma_gorath.wasm
WASM_ARTIFACT := dist/wasm/shuma_gorath.wasm

# Ensure rustup-installed cargo is available in non-interactive shells
CARGO_HOME ?= $(HOME)/.cargo
PATH := $(CARGO_HOME)/bin:$(PATH)
export PATH

# Load local development overrides (created by make setup)
ENV_LOCAL ?= .env.local
ifneq ("$(wildcard $(ENV_LOCAL))","")
include $(ENV_LOCAL)
endif
SHUMA_LOCAL_STATE_DIR ?= .shuma
LINODE_SETUP_RECEIPT ?= $(SHUMA_LOCAL_STATE_DIR)/linode-shared-host-setup.json
FERMYON_AKAMAI_SETUP_RECEIPT ?= $(SHUMA_LOCAL_STATE_DIR)/fermyon-akamai-edge-setup.json
FERMYON_AKAMAI_DEPLOY_RECEIPT ?= $(SHUMA_LOCAL_STATE_DIR)/fermyon-akamai-edge-deploy.json
FERMYON_AKAMAI_RENDERED_MANIFEST ?= spin.fermyon-akamai-edge.toml
REMOTE_RECEIPTS_DIR ?= $(SHUMA_LOCAL_STATE_DIR)/remotes
TELEMETRY_SHARED_HOST_EVIDENCE_REPORT ?= .spin/telemetry_shared_host_evidence.json
TELEMETRY_FERMYON_EDGE_EVIDENCE_REPORT ?= .spin/telemetry_fermyon_edge_evidence.json
FERMYON_EDGE_SIGNAL_SMOKE_REPORT ?= .spin/fermyon_edge_signal_smoke.json

# Normalize optional quoted values from .env.local (handles KEY=value and KEY="value")
strip_wrapping_quotes = $(patsubst "%",%,$(patsubst '%',%,$(strip $(1))))
json_receipt_value = $(strip $(shell python3 -c 'import json,pathlib,sys; p=pathlib.Path(sys.argv[1]); keys=sys.argv[2].split("."); cur=json.loads(p.read_text(encoding="utf-8")) if p.exists() else {}; [cur := cur.get(key, "") if isinstance(cur, dict) else "" for key in keys]; print(cur if isinstance(cur, str) else "")' "$(LINODE_SETUP_RECEIPT)" "$(1)" 2>/dev/null))
process_env_lookup = $(strip $(shell python3 -c 'import os,sys; name=sys.argv[1]; print("__SET__" + os.environ[name] if name in os.environ else "__MISSING__")' "$(1)"))
prefer_process_env = $(call strip_wrapping_quotes,$(if $(filter __SET__%,$(call process_env_lookup,$(1))),$(patsubst __SET__%,%,$(call process_env_lookup,$(1))),$($(1))))
defaults_env_lookup = $(call strip_wrapping_quotes,$(strip $(shell awk -F= '/^$(1)=/{sub(/^[^=]*=/,""); print; exit}' config/defaults.env 2>/dev/null)))
SHUMA_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_API_KEY))
LINODE_TOKEN := $(call strip_wrapping_quotes,$(LINODE_TOKEN))
SHUMA_ADMIN_READONLY_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_READONLY_API_KEY))
SHUMA_JS_SECRET := $(call strip_wrapping_quotes,$(SHUMA_JS_SECRET))
SHUMA_POW_SECRET := $(call strip_wrapping_quotes,$(SHUMA_POW_SECRET))
SHUMA_CHALLENGE_SECRET := $(call strip_wrapping_quotes,$(SHUMA_CHALLENGE_SECRET))
SHUMA_MAZE_PREVIEW_SECRET := $(call strip_wrapping_quotes,$(SHUMA_MAZE_PREVIEW_SECRET))
SHUMA_FORWARDED_IP_SECRET := $(call strip_wrapping_quotes,$(SHUMA_FORWARDED_IP_SECRET))
SHUMA_HEALTH_SECRET := $(call strip_wrapping_quotes,$(SHUMA_HEALTH_SECRET))
SHUMA_ADMIN_IP_ALLOWLIST := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_IP_ALLOWLIST))
SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE))
SHUMA_EVENT_LOG_RETENTION_HOURS := $(call strip_wrapping_quotes,$(SHUMA_EVENT_LOG_RETENTION_HOURS))
SHUMA_MONITORING_RETENTION_HOURS := $(if $(strip $(SHUMA_MONITORING_RETENTION_HOURS)),$(SHUMA_MONITORING_RETENTION_HOURS),$(call defaults_env_lookup,SHUMA_MONITORING_RETENTION_HOURS))
SHUMA_MONITORING_ROLLUP_RETENTION_HOURS := $(if $(strip $(SHUMA_MONITORING_ROLLUP_RETENTION_HOURS)),$(SHUMA_MONITORING_ROLLUP_RETENTION_HOURS),$(call defaults_env_lookup,SHUMA_MONITORING_ROLLUP_RETENTION_HOURS))
SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED))
SHUMA_KV_STORE_FAIL_OPEN := $(call strip_wrapping_quotes,$(SHUMA_KV_STORE_FAIL_OPEN))
SHUMA_ENFORCE_HTTPS := $(call strip_wrapping_quotes,$(SHUMA_ENFORCE_HTTPS))
SHUMA_DEBUG_HEADERS := $(call strip_wrapping_quotes,$(SHUMA_DEBUG_HEADERS))
SHUMA_RUNTIME_ENV := $(call strip_wrapping_quotes,$(SHUMA_RUNTIME_ENV))
SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS := $(call strip_wrapping_quotes,$(SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS))
SHUMA_LOCAL_PROD_DIRECT_MODE := $(call strip_wrapping_quotes,$(SHUMA_LOCAL_PROD_DIRECT_MODE))
SHUMA_ADVERSARY_SIM_AVAILABLE := $(call strip_wrapping_quotes,$(SHUMA_ADVERSARY_SIM_AVAILABLE))
SHUMA_SIM_TELEMETRY_SECRET := $(call strip_wrapping_quotes,$(SHUMA_SIM_TELEMETRY_SECRET))
SHUMA_FRONTIER_OPENAI_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_OPENAI_API_KEY))
SHUMA_FRONTIER_ANTHROPIC_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_ANTHROPIC_API_KEY))
SHUMA_FRONTIER_GOOGLE_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_GOOGLE_API_KEY))
SHUMA_FRONTIER_XAI_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_XAI_API_KEY))
SHUMA_FRONTIER_OPENAI_MODEL := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_OPENAI_MODEL))
SHUMA_FRONTIER_ANTHROPIC_MODEL := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_ANTHROPIC_MODEL))
SHUMA_FRONTIER_GOOGLE_MODEL := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_GOOGLE_MODEL))
SHUMA_FRONTIER_XAI_MODEL := $(call strip_wrapping_quotes,$(SHUMA_FRONTIER_XAI_MODEL))
SHUMA_ENTERPRISE_MULTI_INSTANCE := $(call strip_wrapping_quotes,$(SHUMA_ENTERPRISE_MULTI_INSTANCE))
SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED := $(call strip_wrapping_quotes,$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED))
SHUMA_GATEWAY_UPSTREAM_ORIGIN := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_UPSTREAM_ORIGIN))
SHUMA_GATEWAY_DEPLOYMENT_PROFILE := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_DEPLOYMENT_PROFILE))
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL))
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS))
SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST))
SHUMA_GATEWAY_PUBLIC_AUTHORITIES := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_PUBLIC_AUTHORITIES))
SHUMA_GATEWAY_LOOP_MAX_HOPS := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_LOOP_MAX_HOPS))
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED))
SHUMA_GATEWAY_ORIGIN_AUTH_MODE := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_AUTH_MODE))
SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME))
SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE))
SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS))
SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS))
SHUMA_GATEWAY_TLS_STRICT := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_TLS_STRICT))
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED := $(call strip_wrapping_quotes,$(SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED))
SHUMA_ACTIVE_REMOTE := $(call strip_wrapping_quotes,$(SHUMA_ACTIVE_REMOTE))
SHUMA_RUNTIME_ENV := $(if $(strip $(SHUMA_RUNTIME_ENV)),$(SHUMA_RUNTIME_ENV),runtime-prod)
SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS := $(if $(strip $(SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS)),$(SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS),$(call defaults_env_lookup,SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS))
SHUMA_LOCAL_PROD_DIRECT_MODE := $(if $(strip $(SHUMA_LOCAL_PROD_DIRECT_MODE)),$(SHUMA_LOCAL_PROD_DIRECT_MODE),false)
SHUMA_ADVERSARY_SIM_AVAILABLE := $(if $(strip $(SHUMA_ADVERSARY_SIM_AVAILABLE)),$(SHUMA_ADVERSARY_SIM_AVAILABLE),true)
SHUMA_FRONTIER_OPENAI_MODEL := $(if $(strip $(SHUMA_FRONTIER_OPENAI_MODEL)),$(SHUMA_FRONTIER_OPENAI_MODEL),gpt-5-mini)
SHUMA_FRONTIER_ANTHROPIC_MODEL := $(if $(strip $(SHUMA_FRONTIER_ANTHROPIC_MODEL)),$(SHUMA_FRONTIER_ANTHROPIC_MODEL),claude-3-5-haiku-latest)
SHUMA_FRONTIER_GOOGLE_MODEL := $(if $(strip $(SHUMA_FRONTIER_GOOGLE_MODEL)),$(SHUMA_FRONTIER_GOOGLE_MODEL),gemini-2.0-flash-lite)
SHUMA_FRONTIER_XAI_MODEL := $(if $(strip $(SHUMA_FRONTIER_XAI_MODEL)),$(SHUMA_FRONTIER_XAI_MODEL),grok-3-mini)
SHUMA_ENTERPRISE_MULTI_INSTANCE := $(if $(strip $(SHUMA_ENTERPRISE_MULTI_INSTANCE)),$(SHUMA_ENTERPRISE_MULTI_INSTANCE),false)
SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED := $(if $(strip $(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED)),$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED),false)
SHUMA_GATEWAY_DEPLOYMENT_PROFILE := $(if $(strip $(SHUMA_GATEWAY_DEPLOYMENT_PROFILE)),$(SHUMA_GATEWAY_DEPLOYMENT_PROFILE),shared-server)
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL := $(if $(strip $(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL)),$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL),false)
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS := $(if $(strip $(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS)),$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS),false)
SHUMA_GATEWAY_LOOP_MAX_HOPS := $(if $(strip $(SHUMA_GATEWAY_LOOP_MAX_HOPS)),$(SHUMA_GATEWAY_LOOP_MAX_HOPS),3)
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED := $(if $(strip $(SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED)),$(SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED),false)
SHUMA_GATEWAY_ORIGIN_AUTH_MODE := $(if $(strip $(SHUMA_GATEWAY_ORIGIN_AUTH_MODE)),$(SHUMA_GATEWAY_ORIGIN_AUTH_MODE),network_only)
SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS := $(if $(strip $(SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS)),$(SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS),90)
SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS := $(if $(strip $(SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS)),$(SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS),7)
SHUMA_GATEWAY_TLS_STRICT := $(if $(strip $(SHUMA_GATEWAY_TLS_STRICT)),$(SHUMA_GATEWAY_TLS_STRICT),true)
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED := $(if $(strip $(SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED)),$(SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED),false)
SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(if $(strip $(SHUMA_ADMIN_CONFIG_WRITE_ENABLED)),$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED),true)
SHUMA_SPIN_MANIFEST := $(call prefer_process_env,SHUMA_SPIN_MANIFEST)
GATEWAY_SURFACE_CATALOG_PATH := $(call strip_wrapping_quotes,$(GATEWAY_SURFACE_CATALOG_PATH))
SSH_PRIVATE_KEY_FILE := $(call strip_wrapping_quotes,$(SSH_PRIVATE_KEY_FILE))
SSH_PUBLIC_KEY_FILE := $(call strip_wrapping_quotes,$(SSH_PUBLIC_KEY_FILE))
SHUMA_RATE_LIMITER_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_REDIS_URL))
SHUMA_BAN_STORE_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_BAN_STORE_REDIS_URL))
SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN))
SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH))
SHUMA_BAN_STORE_OUTAGE_MODE := $(call strip_wrapping_quotes,$(SHUMA_BAN_STORE_OUTAGE_MODE))
SSH_PRIVATE_KEY_FILE := $(if $(strip $(SSH_PRIVATE_KEY_FILE)),$(SSH_PRIVATE_KEY_FILE),$(call json_receipt_value,ssh.private_key_path))
SSH_PUBLIC_KEY_FILE := $(if $(strip $(SSH_PUBLIC_KEY_FILE)),$(SSH_PUBLIC_KEY_FILE),$(call json_receipt_value,ssh.public_key_path))
SPIN_UP_MANIFEST := $(if $(strip $(SHUMA_SPIN_MANIFEST)),$(SHUMA_SPIN_MANIFEST),spin.toml)

DEPLOY_LINODE_TOKEN := $(call prefer_process_env,LINODE_TOKEN)
DEPLOY_REMOTE_RECEIPTS_DIR := $(call prefer_process_env,REMOTE_RECEIPTS_DIR)
DEPLOY_SSH_PRIVATE_KEY_FILE := $(call prefer_process_env,SSH_PRIVATE_KEY_FILE)
DEPLOY_SSH_PUBLIC_KEY_FILE := $(call prefer_process_env,SSH_PUBLIC_KEY_FILE)
DEPLOY_SHUMA_API_KEY := $(call prefer_process_env,SHUMA_API_KEY)
DEPLOY_SHUMA_JS_SECRET := $(call prefer_process_env,SHUMA_JS_SECRET)
DEPLOY_SHUMA_FORWARDED_IP_SECRET := $(call prefer_process_env,SHUMA_FORWARDED_IP_SECRET)
DEPLOY_SHUMA_HEALTH_SECRET := $(call prefer_process_env,SHUMA_HEALTH_SECRET)
DEPLOY_SHUMA_SIM_TELEMETRY_SECRET := $(call prefer_process_env,SHUMA_SIM_TELEMETRY_SECRET)
DEPLOY_SHUMA_DEBUG_HEADERS := $(call prefer_process_env,SHUMA_DEBUG_HEADERS)
DEPLOY_SHUMA_ADMIN_IP_ALLOWLIST := $(call prefer_process_env,SHUMA_ADMIN_IP_ALLOWLIST)
DEPLOY_SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(call prefer_process_env,SHUMA_ADMIN_CONFIG_WRITE_ENABLED)
DEPLOY_SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED := $(call prefer_process_env,SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED)
DEPLOY_SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED := $(call prefer_process_env,SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED)
DEPLOY_SHUMA_MONITORING_RETENTION_HOURS := $(call prefer_process_env,SHUMA_MONITORING_RETENTION_HOURS)
DEPLOY_SHUMA_MONITORING_ROLLUP_RETENTION_HOURS := $(call prefer_process_env,SHUMA_MONITORING_ROLLUP_RETENTION_HOURS)
DEPLOY_SHUMA_ENTERPRISE_MULTI_INSTANCE := $(call prefer_process_env,SHUMA_ENTERPRISE_MULTI_INSTANCE)
DEPLOY_SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED := $(call prefer_process_env,SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED)
DEPLOY_SHUMA_PROVIDER_RATE_LIMITER := $(call prefer_process_env,SHUMA_PROVIDER_RATE_LIMITER)
DEPLOY_SHUMA_PROVIDER_BAN_STORE := $(call prefer_process_env,SHUMA_PROVIDER_BAN_STORE)
DEPLOY_SHUMA_RATE_LIMITER_REDIS_URL := $(call prefer_process_env,SHUMA_RATE_LIMITER_REDIS_URL)
DEPLOY_SHUMA_BAN_STORE_REDIS_URL := $(call prefer_process_env,SHUMA_BAN_STORE_REDIS_URL)
DEPLOY_SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN := $(call prefer_process_env,SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN)
DEPLOY_SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH := $(call prefer_process_env,SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH)
DEPLOY_SHUMA_BAN_STORE_OUTAGE_MODE := $(call prefer_process_env,SHUMA_BAN_STORE_OUTAGE_MODE)
DEPLOY_SHUMA_GATEWAY_UPSTREAM_ORIGIN := $(call prefer_process_env,SHUMA_GATEWAY_UPSTREAM_ORIGIN)
DEPLOY_SHUMA_GATEWAY_DEPLOYMENT_PROFILE := $(call prefer_process_env,SHUMA_GATEWAY_DEPLOYMENT_PROFILE)
DEPLOY_SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED := $(call prefer_process_env,SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED)
DEPLOY_SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED := $(call prefer_process_env,SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED)
DEPLOY_SHUMA_GATEWAY_TLS_STRICT := $(call prefer_process_env,SHUMA_GATEWAY_TLS_STRICT)
DEPLOY_GATEWAY_SURFACE_CATALOG_PATH := $(call prefer_process_env,GATEWAY_SURFACE_CATALOG_PATH)

DEPLOY_ENV_ONLY := \
	SHUMA_API_KEY="$(DEPLOY_SHUMA_API_KEY)" \
	SHUMA_JS_SECRET="$(DEPLOY_SHUMA_JS_SECRET)" \
	SHUMA_FORWARDED_IP_SECRET="$(DEPLOY_SHUMA_FORWARDED_IP_SECRET)" \
	SHUMA_HEALTH_SECRET="$(DEPLOY_SHUMA_HEALTH_SECRET)" \
	SHUMA_SIM_TELEMETRY_SECRET="$(DEPLOY_SHUMA_SIM_TELEMETRY_SECRET)" \
	SHUMA_DEBUG_HEADERS="$(DEPLOY_SHUMA_DEBUG_HEADERS)" \
	SHUMA_ADMIN_IP_ALLOWLIST="$(DEPLOY_SHUMA_ADMIN_IP_ALLOWLIST)" \
	SHUMA_ADMIN_CONFIG_WRITE_ENABLED="$(DEPLOY_SHUMA_ADMIN_CONFIG_WRITE_ENABLED)" \
	SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED="$(DEPLOY_SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED)" \
	SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED="$(DEPLOY_SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED)" \
	SHUMA_MONITORING_RETENTION_HOURS="$(DEPLOY_SHUMA_MONITORING_RETENTION_HOURS)" \
	SHUMA_MONITORING_ROLLUP_RETENTION_HOURS="$(DEPLOY_SHUMA_MONITORING_ROLLUP_RETENTION_HOURS)" \
	SHUMA_ENTERPRISE_MULTI_INSTANCE="$(DEPLOY_SHUMA_ENTERPRISE_MULTI_INSTANCE)" \
	SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED="$(DEPLOY_SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED)" \
	SHUMA_PROVIDER_RATE_LIMITER="$(DEPLOY_SHUMA_PROVIDER_RATE_LIMITER)" \
	SHUMA_PROVIDER_BAN_STORE="$(DEPLOY_SHUMA_PROVIDER_BAN_STORE)" \
	SHUMA_RATE_LIMITER_REDIS_URL="$(DEPLOY_SHUMA_RATE_LIMITER_REDIS_URL)" \
	SHUMA_BAN_STORE_REDIS_URL="$(DEPLOY_SHUMA_BAN_STORE_REDIS_URL)" \
	SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN="$(DEPLOY_SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN)" \
	SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH="$(DEPLOY_SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH)" \
	SHUMA_BAN_STORE_OUTAGE_MODE="$(DEPLOY_SHUMA_BAN_STORE_OUTAGE_MODE)" \
	SHUMA_GATEWAY_UPSTREAM_ORIGIN="$(DEPLOY_SHUMA_GATEWAY_UPSTREAM_ORIGIN)" \
	SHUMA_GATEWAY_DEPLOYMENT_PROFILE="$(DEPLOY_SHUMA_GATEWAY_DEPLOYMENT_PROFILE)" \
	SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED="$(DEPLOY_SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED)" \
	SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED="$(DEPLOY_SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED)" \
	SHUMA_GATEWAY_TLS_STRICT="$(DEPLOY_SHUMA_GATEWAY_TLS_STRICT)" \
	GATEWAY_SURFACE_CATALOG_PATH="$(DEPLOY_GATEWAY_SURFACE_CATALOG_PATH)"

DEPLOY_VALIDATE_FORWARD_VARS := \
	SHUMA_SPIN_MANIFEST="$(SHUMA_SPIN_MANIFEST)" \
	SHUMA_GATEWAY_UPSTREAM_ORIGIN="$(SHUMA_GATEWAY_UPSTREAM_ORIGIN)" \
	GATEWAY_SURFACE_CATALOG_PATH="$(GATEWAY_SURFACE_CATALOG_PATH)"

# Inject env-only runtime keys into Spin from .env.local / shell env.
# This list is the operator-facing copy surface for deploy-time env overrides.
SPIN_ENV_ONLY_BASE := \
	--env SHUMA_API_KEY=$(SHUMA_API_KEY) \
	--env SHUMA_ADMIN_READONLY_API_KEY=$(SHUMA_ADMIN_READONLY_API_KEY) \
	--env SHUMA_JS_SECRET=$(SHUMA_JS_SECRET) \
	--env SHUMA_POW_SECRET=$(SHUMA_POW_SECRET) \
	--env SHUMA_CHALLENGE_SECRET=$(SHUMA_CHALLENGE_SECRET) \
	--env SHUMA_MAZE_PREVIEW_SECRET=$(SHUMA_MAZE_PREVIEW_SECRET) \
	--env SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) \
	--env SHUMA_HEALTH_SECRET=$(SHUMA_HEALTH_SECRET) \
	--env SHUMA_ADMIN_IP_ALLOWLIST=$(SHUMA_ADMIN_IP_ALLOWLIST) \
	--env SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=$(SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE) \
	--env SHUMA_EVENT_LOG_RETENTION_HOURS=$(SHUMA_EVENT_LOG_RETENTION_HOURS) \
	--env SHUMA_MONITORING_RETENTION_HOURS=$(SHUMA_MONITORING_RETENTION_HOURS) \
	--env SHUMA_MONITORING_ROLLUP_RETENTION_HOURS=$(SHUMA_MONITORING_ROLLUP_RETENTION_HOURS) \
	--env SHUMA_KV_STORE_FAIL_OPEN=$(SHUMA_KV_STORE_FAIL_OPEN) \
	--env SHUMA_ENFORCE_HTTPS=$(SHUMA_ENFORCE_HTTPS) \
	--env SHUMA_RUNTIME_ENV=$(SHUMA_RUNTIME_ENV) \
	--env SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS=$(SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS) \
	--env SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) \
	--env SHUMA_SIM_TELEMETRY_SECRET=$(SHUMA_SIM_TELEMETRY_SECRET) \
	--env SHUMA_FRONTIER_OPENAI_API_KEY=$(SHUMA_FRONTIER_OPENAI_API_KEY) \
	--env SHUMA_FRONTIER_ANTHROPIC_API_KEY=$(SHUMA_FRONTIER_ANTHROPIC_API_KEY) \
	--env SHUMA_FRONTIER_GOOGLE_API_KEY=$(SHUMA_FRONTIER_GOOGLE_API_KEY) \
	--env SHUMA_FRONTIER_XAI_API_KEY=$(SHUMA_FRONTIER_XAI_API_KEY) \
	--env SHUMA_FRONTIER_OPENAI_MODEL=$(SHUMA_FRONTIER_OPENAI_MODEL) \
	--env SHUMA_FRONTIER_ANTHROPIC_MODEL=$(SHUMA_FRONTIER_ANTHROPIC_MODEL) \
	--env SHUMA_FRONTIER_GOOGLE_MODEL=$(SHUMA_FRONTIER_GOOGLE_MODEL) \
	--env SHUMA_FRONTIER_XAI_MODEL=$(SHUMA_FRONTIER_XAI_MODEL) \
	--env SHUMA_ENTERPRISE_MULTI_INSTANCE=$(SHUMA_ENTERPRISE_MULTI_INSTANCE) \
	--env SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED) \
	--env SHUMA_RATE_LIMITER_REDIS_URL=$(SHUMA_RATE_LIMITER_REDIS_URL) \
	--env SHUMA_BAN_STORE_REDIS_URL=$(SHUMA_BAN_STORE_REDIS_URL) \
	--env SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN) \
	--env SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH) \
	--env SHUMA_BAN_STORE_OUTAGE_MODE=$(SHUMA_BAN_STORE_OUTAGE_MODE) \
	--env SHUMA_GATEWAY_UPSTREAM_ORIGIN=$(SHUMA_GATEWAY_UPSTREAM_ORIGIN) \
	--env SHUMA_GATEWAY_DEPLOYMENT_PROFILE=$(SHUMA_GATEWAY_DEPLOYMENT_PROFILE) \
	--env SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL) \
	--env SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS=$(SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS) \
	--env SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST=$(SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST) \
	--env SHUMA_GATEWAY_PUBLIC_AUTHORITIES=$(SHUMA_GATEWAY_PUBLIC_AUTHORITIES) \
	--env SHUMA_GATEWAY_LOOP_MAX_HOPS=$(SHUMA_GATEWAY_LOOP_MAX_HOPS) \
	--env SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=$(SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED) \
	--env SHUMA_GATEWAY_ORIGIN_AUTH_MODE=$(SHUMA_GATEWAY_ORIGIN_AUTH_MODE) \
	--env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME=$(SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME) \
	--env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=$(SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE) \
	--env SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS=$(SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS) \
	--env SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS=$(SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS) \
	--env SHUMA_GATEWAY_TLS_STRICT=$(SHUMA_GATEWAY_TLS_STRICT) \
	--env SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=$(SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED)
SPIN_RUNTIME_CONTROL_ENV := --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_DEBUG_HEADERS=$(SHUMA_DEBUG_HEADERS)
SPIN_ENV_ONLY := $(SPIN_ENV_ONLY_BASE) $(SPIN_RUNTIME_CONTROL_ENV)

# Optional forwarded-IP trust header for local health/test requests.
FORWARDED_SECRET_HEADER := $(if $(SHUMA_FORWARDED_IP_SECRET),-H "X-Shuma-Forwarded-Secret: $(SHUMA_FORWARDED_IP_SECRET)",)
# Optional health secret header for local health/test requests.
HEALTH_SECRET_HEADER := $(if $(SHUMA_HEALTH_SECRET),-H "X-Shuma-Health-Secret: $(SHUMA_HEALTH_SECRET)",)
DEV_ADMIN_CONFIG_WRITE_ENABLED ?= true
DEV_DEBUG_HEADERS ?= true
DEV_ADMIN_IP_ALLOWLIST ?=
DEV_RUNTIME_ENV ?= runtime-dev
DEV_LOCAL_PROD_DIRECT_MODE ?= $(SHUMA_LOCAL_PROD_DIRECT_MODE)
DEV_ADVERSARY_SIM_AVAILABLE ?= true
SPIN_DEV_OVERRIDES := --env SHUMA_DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_ADMIN_IP_ALLOWLIST=$(DEV_ADMIN_IP_ALLOWLIST) --env SHUMA_RUNTIME_ENV=$(DEV_RUNTIME_ENV) --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) --env SHUMA_LOCAL_PROD_DIRECT_MODE=$(DEV_LOCAL_PROD_DIRECT_MODE) --env SHUMA_GATEWAY_ORIGIN_AUTH_MODE=network_only --env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME= --env SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=
SPIN_PROD_OVERRIDES := --env SHUMA_DEBUG_HEADERS=false --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_RUNTIME_ENV=runtime-prod --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE)
SPIN_READY_TIMEOUT_SECONDS ?= 90
SHUMA_DASHBOARD_BUNDLE_MAX_TOTAL_BYTES ?= 352000
SHUMA_DASHBOARD_BUNDLE_MAX_JS_BYTES ?= 330000
SHUMA_DASHBOARD_BUNDLE_MAX_CSS_BYTES ?= 40000
SHUMA_DASHBOARD_BUNDLE_MAX_JS_CHUNK_BYTES ?= 150000
SHUMA_DASHBOARD_BUNDLE_MAX_CSS_ASSET_BYTES ?= 30000
SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE ?= 0
DEPLOY_LINODE_ARGS ?=
REMOTE ?=
REMOTE_NAME_ARG := $(if $(strip $(REMOTE)),--name "$(REMOTE)",)
DEV_WATCH_IGNORES := -i '*.wasm' -i 'dist/wasm/shuma_gorath.wasm' -i '.spin/**' -i 'dashboard/.svelte-kit' -i 'dashboard/.svelte-kit/**' -i 'dashboard/.vite' -i 'dashboard/.vite/**'
ADVERSARY_SIM_SUPERVISOR_BASE_URL ?= http://127.0.0.1:3000
GATEWAY_TLS_WASM_REPORT ?= scripts/tests/adversarial/gateway_tls_wasm_harness_report.json
GATEWAY_PROBE_PATH ?= /
GATEWAY_PROBE_FAIL_ON_INCONCLUSIVE ?= 0
GATEWAY_PROBE_JSON_OUTPUT ?= scripts/tests/adversarial/gateway_origin_bypass_probe_report.json
ADVERSARIAL_ARTIFACT_DIR ?= .spin/adversarial
ADVERSARIAL_PREFLIGHT_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/preflight_report.json
ADVERSARIAL_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/latest_report.json
ADVERSARIAL_ATTACK_PLAN_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/attack_plan.json
FRONTIER_LANE_STATUS_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/frontier_lane_status.json
FRONTIER_UNAVAILABILITY_POLICY_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/frontier_unavailability_policy.json
SIM2_REALTIME_BENCH_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_realtime_bench_report.json
SIM2_REALTIME_BENCH_SUMMARY_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_realtime_bench_summary.md
SIM2_ADR_CONFORMANCE_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_adr_conformance_report.json
SIM2_CI_DIAGNOSTICS_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_ci_diagnostics.json
SIM2_VERIFICATION_MATRIX_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_verification_matrix_report.json
SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_operational_regressions_report.json
SIM2_GOVERNANCE_CONTRACT_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/sim2_governance_contract_report.json
ADVERSARIAL_REPEATABILITY_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/repeatability_report.json
ADVERSARIAL_PROMOTION_CANDIDATES_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/promotion_candidates_report.json
ADVERSARIAL_DIFF_BASELINE_PATH ?= scripts/tests/adversarial/latest_report.baseline.json
ADVERSARIAL_DIFF_CANDIDATE_PATH ?= $(ADVERSARIAL_REPORT_PATH)
ADVERSARIAL_DIFF_OUTPUT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/adversarial_report_diff.json
ADVERSARIAL_CONTAINER_ISOLATION_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/container_isolation_report.json
ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH ?= $(ADVERSARIAL_ARTIFACT_DIR)/container_blackbox_report.json
SCRAPLING_VENV_PYTHON ?= .venv-scrapling/bin/python3
SCRAPLING_LOCAL_PUBLIC_BASE_URL ?= http://localhost:3000/
SCRAPLING_LOCAL_RECEIPT_PATH ?= $(SHUMA_LOCAL_STATE_DIR)/scrapling/local-dev.deploy-prep.json
SCRAPLING_LOCAL_SCOPE_PATH ?= $(SHUMA_LOCAL_STATE_DIR)/scrapling/local-dev.scope.json
SCRAPLING_LOCAL_SEED_PATH ?= $(SHUMA_LOCAL_STATE_DIR)/scrapling/local-dev.seed.json
SCRAPLING_LOCAL_CRAWLDIR ?= $(SHUMA_LOCAL_STATE_DIR)/adversary-sim/scrapling-crawldir
SCRAPLING_LOCAL_RUNTIME_ENV := SHUMA_SIM_TELEMETRY_SECRET=$(SHUMA_SIM_TELEMETRY_SECRET) ADVERSARY_SIM_SCRAPLING_SCOPE_DESCRIPTOR_PATH=$(SCRAPLING_LOCAL_SCOPE_PATH) ADVERSARY_SIM_SCRAPLING_SEED_INVENTORY_PATH=$(SCRAPLING_LOCAL_SEED_PATH) ADVERSARY_SIM_SCRAPLING_CRAWLDIR=$(SCRAPLING_LOCAL_CRAWLDIR)

#--------------------------
# Setup (first-time)
#--------------------------

setup: ## Install all dependencies (Rust, Spin, cargo-watch, Node toolchain, pnpm deps, Playwright Chromium)
	@./scripts/bootstrap/setup.sh

setup-runtime: ## Install runtime-only dependencies (Rust, wasm target, Spin, env bootstrap, KV seed)
	@./scripts/bootstrap/setup-runtime.sh

verify: ## Verify all dependencies are installed correctly
	@./scripts/bootstrap/verify-setup.sh

verify-runtime: ## Verify runtime-only dependencies and build path (no Node/pnpm/Playwright checks)
	@./scripts/bootstrap/verify-runtime.sh

config-seed: ## Seed KV tunable config from config/defaults.env (create + explicit backfill/repair)
	@./scripts/config_seed.sh

config-verify: ## Verify KV tunable config is present and schema-complete (read-only)
	@./scripts/config_seed.sh --verify-only

dashboard-build: ## Build SvelteKit dashboard static assets to dist/dashboard
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@rm -rf dist/dashboard
	@corepack pnpm run build:dashboard

dashboard-verify-freshness: ## Fail if dashboard source is newer than the current dist/dashboard build
	@DASHBOARD_STAMP="dist/dashboard/_app/version.json"; \
	if [ ! -f "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/style.css -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/svelte.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/vite.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ package.json -nt "$$DASHBOARD_STAMP" ] || \
	   [ pnpm-lock.yaml -nt "$$DASHBOARD_STAMP" ] || \
	   find dashboard/src dashboard/static -type f -newer "$$DASHBOARD_STAMP" -print -quit | grep -q .; then \
		echo "Dashboard build freshness verification failed: dist/dashboard is older than dashboard sources."; \
		echo "Run: make dashboard-build"; \
		echo "Then restart Spin (for example, make dev, make run, or make run-prebuilt) before rerunning dashboard e2e checks."; \
		exit 1; \
	fi

#--------------------------
# Development
#--------------------------

dev: ## Build and run with file watching (auto-rebuild on save)
	@echo "$(CYAN)🚀 Starting development server with file watching...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) RUNTIME=$(DEV_RUNTIME_ENV) SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@echo "$(YELLOW)⚡ Startup rebuild override: DEV_FORCE_REBUILD=$${DEV_FORCE_REBUILD:-0}$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@pkill -x spin 2>/dev/null || true
	@$(MAKE) --no-print-directory config-verify
	@DASHBOARD_STAMP="dist/dashboard/_app/version.json"; \
	if [ "$${DEV_FORCE_REBUILD:-0}" = "1" ] || [ ! -f "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/style.css -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/svelte.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/vite.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ package.json -nt "$$DASHBOARD_STAMP" ] || \
	   [ pnpm-lock.yaml -nt "$$DASHBOARD_STAMP" ] || \
	   find dashboard/src dashboard/static -type f -newer "$$DASHBOARD_STAMP" -print -quit | grep -q .; then \
		$(MAKE) --no-print-directory dashboard-build >/dev/null; \
	else \
		echo "Dashboard assets unchanged; skipping dashboard-build."; \
	fi
	@if [ "$${DEV_FORCE_REBUILD:-0}" = "1" ] || [ ! -f $(WASM_BUILD_OUTPUT) ] || [ ! -f $(WASM_ARTIFACT) ] || \
	   [ Cargo.toml -nt $(WASM_BUILD_OUTPUT) ] || [ Cargo.lock -nt $(WASM_BUILD_OUTPUT) ] || \
	   { [ -f build.rs ] && [ build.rs -nt $(WASM_BUILD_OUTPUT) ]; } || \
	   find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then \
		./scripts/set_crate_type.sh cdylib; \
		cargo build --target wasm32-wasip1 --release; \
		mkdir -p $(dir $(WASM_ARTIFACT)); \
		cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT); \
		./scripts/set_crate_type.sh rlib; \
	else \
		echo "Rust/WASM artifacts unchanged; skipping initial release build."; \
	fi
	@./scripts/dev_watch_lock.sh cargo watch --poll -w src -w dashboard -w spin.toml $(DEV_WATCH_IGNORES) \
		-s 'if [ ! -f $(WASM_BUILD_OUTPUT) ] || find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then ./scripts/set_crate_type.sh cdylib && cargo build --target wasm32-wasip1 --release && mkdir -p $(dir $(WASM_ARTIFACT)) && cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT) && ./scripts/set_crate_type.sh rlib; else echo "No Rust changes detected; skipping WASM rebuild."; fi' \
		-s 'pkill -x spin 2>/dev/null || true; $(MAKE) --no-print-directory prepare-scrapling-local >/dev/null 2>&1 && $(MAKE) --no-print-directory config-verify && $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1 && RUNTIME_INSTANCE_ID="$$(uuidgen)" && SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) $(SCRAPLING_LOCAL_RUNTIME_ENV) SPIN_ALWAYS_BUILD=0 ./scripts/run_with_oversight_supervisor.sh spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000'

dev-prod: ## Build and run with file watching using runtime-prod local-direct posture (admin writes remain enabled for local tuning)
	@$(MAKE) --no-print-directory dev DEV_RUNTIME_ENV=runtime-prod DEV_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) DEV_DEBUG_HEADERS=false DEV_ADMIN_CONFIG_WRITE_ENABLED=true DEV_LOCAL_PROD_DIRECT_MODE=true

dev-closed: ## Build and run with file watching and SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)
	@echo "$(CYAN)🚨 Starting development server with SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) RUNTIME=$(DEV_RUNTIME_ENV) SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@echo "$(YELLOW)⚡ Startup rebuild override: DEV_FORCE_REBUILD=$${DEV_FORCE_REBUILD:-0}$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@pkill -x spin 2>/dev/null || true
	@$(MAKE) --no-print-directory config-verify
	@DASHBOARD_STAMP="dist/dashboard/_app/version.json"; \
	if [ "$${DEV_FORCE_REBUILD:-0}" = "1" ] || [ ! -f "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/style.css -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/svelte.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ dashboard/vite.config.js -nt "$$DASHBOARD_STAMP" ] || \
	   [ package.json -nt "$$DASHBOARD_STAMP" ] || \
	   [ pnpm-lock.yaml -nt "$$DASHBOARD_STAMP" ] || \
	   find dashboard/src dashboard/static -type f -newer "$$DASHBOARD_STAMP" -print -quit | grep -q .; then \
		$(MAKE) --no-print-directory dashboard-build >/dev/null; \
	else \
		echo "Dashboard assets unchanged; skipping dashboard-build."; \
	fi
	@if [ "$${DEV_FORCE_REBUILD:-0}" = "1" ] || [ ! -f $(WASM_BUILD_OUTPUT) ] || [ ! -f $(WASM_ARTIFACT) ] || \
	   [ Cargo.toml -nt $(WASM_BUILD_OUTPUT) ] || [ Cargo.lock -nt $(WASM_BUILD_OUTPUT) ] || \
	   { [ -f build.rs ] && [ build.rs -nt $(WASM_BUILD_OUTPUT) ]; } || \
	   find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then \
		./scripts/set_crate_type.sh cdylib; \
		cargo build --target wasm32-wasip1 --release; \
		mkdir -p $(dir $(WASM_ARTIFACT)); \
		cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT); \
		./scripts/set_crate_type.sh rlib; \
	else \
		echo "Rust/WASM artifacts unchanged; skipping initial release build."; \
	fi
	@./scripts/dev_watch_lock.sh cargo watch --poll -w src -w dashboard -w spin.toml $(DEV_WATCH_IGNORES) \
		-s 'if [ ! -f $(WASM_BUILD_OUTPUT) ] || find src -name "*.rs" -newer $(WASM_BUILD_OUTPUT) -print -quit | grep -q .; then ./scripts/set_crate_type.sh cdylib && cargo build --target wasm32-wasip1 --release && mkdir -p $(dir $(WASM_ARTIFACT)) && cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT) && ./scripts/set_crate_type.sh rlib; else echo "No Rust changes detected; skipping WASM rebuild."; fi' \
		-s 'pkill -x spin 2>/dev/null || true; $(MAKE) --no-print-directory prepare-scrapling-local >/dev/null 2>&1 && $(MAKE) --no-print-directory config-verify && $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1 && RUNTIME_INSTANCE_ID="$$(uuidgen)" && SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) $(SCRAPLING_LOCAL_RUNTIME_ENV) SPIN_ALWAYS_BUILD=0 ./scripts/run_with_oversight_supervisor.sh spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env SHUMA_KV_STORE_FAIL_OPEN=false --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000'

local: dev ## Alias for dev

run: ## Build once and run (no file watching)
	@echo "$(CYAN)🚀 Starting development server...$(NC)"
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) RUNTIME=$(DEV_RUNTIME_ENV) SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@pkill -x spin 2>/dev/null || true
	@$(MAKE) --no-print-directory config-verify
	@$(MAKE) --no-print-directory dashboard-build >/dev/null
	@sleep 1
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@./scripts/set_crate_type.sh rlib
	@echo "$(GREEN)✅ Build complete. Starting Spin...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@$(MAKE) --no-print-directory prepare-scrapling-local >/dev/null
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) $(SCRAPLING_LOCAL_RUNTIME_ENV) ./scripts/run_with_oversight_supervisor.sh spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000

run-prebuilt: ## Run Spin using prebuilt wasm (CI helper)
	@echo "$(CYAN)🚀 Starting prebuilt server...$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@$(MAKE) --no-print-directory config-verify
	@$(MAKE) --no-print-directory dashboard-build >/dev/null
	@pkill -x spin 2>/dev/null || true
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@$(MAKE) --no-print-directory prepare-scrapling-local >/dev/null
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) $(SCRAPLING_LOCAL_RUNTIME_ENV) ./scripts/run_with_oversight_supervisor.sh spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000

#--------------------------
# Production
#--------------------------

build-runtime: ## Build release wasm artifact for runtime/deploy (no dashboard bundle-budget gate)
	@echo "$(CYAN)🔨 Building release binary...$(NC)"
	@./scripts/set_crate_type.sh cdylib
	@cargo build --target wasm32-wasip1 --release
	@mkdir -p $(dir $(WASM_ARTIFACT))
	@cp $(WASM_BUILD_OUTPUT) $(WASM_ARTIFACT)
	@echo "$(GREEN)✅ Build complete: $(WASM_ARTIFACT)$(NC)"
	@./scripts/set_crate_type.sh rlib

build-full-dev: ## Build release wasm artifact with dashboard bundle-budget reporting (strict via SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=1)
	@$(MAKE) --no-print-directory test-dashboard-budgets >/dev/null
	@$(MAKE) --no-print-directory build-runtime

build: build-runtime ## Alias for runtime/deploy release build

prod-start: ## Start production server using existing build artifacts and env (no build/config mutation)
	@echo "$(CYAN)🚀 Starting production server...$(NC)"
	@pkill -x spin 2>/dev/null || true
	@$(MAKE) --no-print-directory config-verify
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) ./scripts/run_with_oversight_supervisor.sh spin up --from $(SPIN_UP_MANIFEST) $(SPIN_ENV_ONLY_BASE) $(SPIN_PROD_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 0.0.0.0:3000

prod: build-runtime ## Build for production and start server
	@$(MAKE) --no-print-directory prod-start

deploy: build-runtime ## Deploy to Fermyon Cloud
	@$(MAKE) --no-print-directory api-key-validate
	@$(DEPLOY_VALIDATE_FORWARD_VARS) $(MAKE) --no-print-directory deploy-env-validate
	@echo "$(CYAN)☁️  Deploying to Fermyon Cloud...$(NC)"
	@spin cloud deploy
	@echo "$(GREEN)✅ Deployment complete!$(NC)"

deploy-profile-baseline: ## Profile wrapper baseline: verify seeded config + dashboard/runtime build
	@echo "$(CYAN)🔧 Running shared deployment baseline...$(NC)"
	@$(MAKE) --no-print-directory config-verify
	@$(MAKE) --no-print-directory dashboard-build >/dev/null
	@$(MAKE) --no-print-directory build-runtime
	@echo "$(GREEN)✅ Shared deployment baseline complete.$(NC)"

deploy-profile-baseline-prebuilt: ## Profile wrapper baseline for prebuilt release bundles: verify seeded config + shipped dashboard/runtime artifacts
	@echo "$(CYAN)🔧 Running prebuilt deployment baseline...$(NC)"
	@$(MAKE) --no-print-directory config-verify
	@test -f "$(WASM_ARTIFACT)" || (echo "$(RED)❌ Missing prebuilt runtime artifact: $(WASM_ARTIFACT).$(NC)" && exit 1)
	@test -f "dist/dashboard/index.html" || (echo "$(RED)❌ Missing prebuilt dashboard artifact: dist/dashboard/index.html.$(NC)" && exit 1)
	@echo "$(GREEN)✅ Prebuilt deployment baseline complete.$(NC)"

deploy-self-hosted-minimal: deploy-profile-baseline ## Profile wrapper: self_hosted_minimal pre-deploy guardrails
	@echo "$(CYAN)🏠 Validating self_hosted_minimal deployment posture...$(NC)"
	@$(DEPLOY_VALIDATE_FORWARD_VARS) SHUMA_ENTERPRISE_MULTI_INSTANCE=false $(MAKE) --no-print-directory deploy-env-validate
	@echo "$(GREEN)✅ self_hosted_minimal pre-deploy checks passed.$(NC)"

deploy-self-hosted-minimal-prebuilt: deploy-profile-baseline-prebuilt ## Profile wrapper: self_hosted_minimal pre-deploy guardrails for prebuilt release bundles
	@echo "$(CYAN)🏠 Validating self_hosted_minimal deployment posture (prebuilt)...$(NC)"
	@$(DEPLOY_VALIDATE_FORWARD_VARS) SHUMA_ENTERPRISE_MULTI_INSTANCE=false $(MAKE) --no-print-directory deploy-env-validate
	@echo "$(GREEN)✅ self_hosted_minimal prebuilt pre-deploy checks passed.$(NC)"

deploy-enterprise-akamai: deploy-profile-baseline ## Profile wrapper: enterprise_akamai overlay checks on top of baseline
	@echo "$(CYAN)🏢 Validating enterprise_akamai overlay posture...$(NC)"
	@ENTERPRISE_MULTI_INSTANCE_RAW="$${SHUMA_ENTERPRISE_MULTI_INSTANCE:-false}"; \
	ENTERPRISE_MULTI_INSTANCE_NORM="$$(printf '%s' "$$ENTERPRISE_MULTI_INSTANCE_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$ENTERPRISE_MULTI_INSTANCE_NORM" in \
		1|true|yes|on) ;; \
		*) \
			echo "$(RED)❌ SHUMA_ENTERPRISE_MULTI_INSTANCE must be true for deploy-enterprise-akamai.$(NC)"; \
			exit 1 ;; \
	esac; \
	EDGE_MODE_RAW="$${SHUMA_EDGE_INTEGRATION_MODE:-off}"; \
	EDGE_MODE_NORM="$$(printf '%s' "$$EDGE_MODE_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$EDGE_MODE_NORM" in \
		additive|authoritative) ;; \
		*) \
			echo "$(RED)❌ SHUMA_EDGE_INTEGRATION_MODE must be additive or authoritative for deploy-enterprise-akamai.$(NC)"; \
			exit 1 ;; \
	esac
	@$(DEPLOY_VALIDATE_FORWARD_VARS) $(MAKE) --no-print-directory deploy-env-validate
	@echo "$(GREEN)✅ enterprise_akamai overlay pre-deploy checks passed.$(NC)"

deploy-linode-one-shot: ## Provision Linode VM + deploy Shuma runtime in one command (requires LINODE_TOKEN and SHUMA_ADMIN_IP_ALLOWLIST)
	@ENV_LOCAL="$(ENV_LOCAL)" \
	LINODE_TOKEN="$(DEPLOY_LINODE_TOKEN)" \
	REMOTE_RECEIPTS_DIR="$(DEPLOY_REMOTE_RECEIPTS_DIR)" \
	SSH_PRIVATE_KEY_FILE="$(DEPLOY_SSH_PRIVATE_KEY_FILE)" \
	SSH_PUBLIC_KEY_FILE="$(DEPLOY_SSH_PUBLIC_KEY_FILE)" \
	$(DEPLOY_ENV_ONLY) \
	./scripts/deploy_linode_one_shot.sh $(DEPLOY_LINODE_ARGS)

#--------------------------
# Testing
#--------------------------

spin-wait-ready: ## Wait for the existing local Spin server to pass /health
	@SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/wait_for_spin_ready.sh --timeout-seconds "$(SPIN_READY_TIMEOUT_SECONDS)"

smoke-single-host: ## Run post-deploy single-host smoke checks (health/admin auth/metrics/challenge route)
	@./scripts/tests/smoke_single_host.sh

prepare-linode-shared-host: ## Agent-oriented Linode shared-host setup (persist token/admin allowlist, create or inspect instance, build catalog, write receipt)
	@python3 ./scripts/prepare_linode_shared_host.py $(PREPARE_LINODE_ARGS)

prepare-fermyon-akamai-edge: ## Agent-oriented Fermyon/Akamai edge setup (persist PAT, validate spin aka, build handoff receipt)
	@python3 ./scripts/prepare_fermyon_akamai_edge.py --env-file "$(ENV_LOCAL)" --receipt-output "$(FERMYON_AKAMAI_SETUP_RECEIPT)" --deploy-receipt-output "$(FERMYON_AKAMAI_DEPLOY_RECEIPT)" --rendered-manifest-output "$(FERMYON_AKAMAI_RENDERED_MANIFEST)" $(PREPARE_FERMYON_ARGS)

deploy-fermyon-akamai-edge: ## Deploy to Fermyon Wasm Functions on Akamai using the durable setup receipt
	@python3 ./scripts/deploy_fermyon_akamai_edge.py --env-file "$(ENV_LOCAL)" --setup-receipt "$(FERMYON_AKAMAI_SETUP_RECEIPT)" --deploy-receipt-output "$(FERMYON_AKAMAI_DEPLOY_RECEIPT)" $(DEPLOY_FERMYON_ARGS)

remote-use: ## Select the active normalized ssh_systemd remote target (REMOTE=<name>)
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" use --name "$(REMOTE)"

remote-update: ## Upload the exact committed HEAD bundle to the selected ssh_systemd remote, restart, smoke, and refresh receipt metadata
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" update $(REMOTE_NAME_ARG)

remote-status: ## Show systemd status for the active normalized ssh_systemd remote
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" status $(REMOTE_NAME_ARG)

remote-logs: ## Show recent journal logs for the active normalized ssh_systemd remote
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" logs $(REMOTE_NAME_ARG)

remote-start: ## Start the systemd service on the active normalized ssh_systemd remote
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" start $(REMOTE_NAME_ARG)

remote-stop: ## Stop the systemd service on the active normalized ssh_systemd remote
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" stop $(REMOTE_NAME_ARG)

remote-open-dashboard: ## Open the hosted dashboard for the active normalized ssh_systemd remote
	@python3 ./scripts/manage_remote_target.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" open-dashboard $(REMOTE_NAME_ARG)

test-remote-edge-signal-smoke: ## Run live trusted-edge signal smoke against the active ssh_systemd remote (Akamai fingerprint fixtures + GEO headers)
	@python3 ./scripts/tests/remote_edge_signal_smoke.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" $(REMOTE_NAME_ARG)

test-live-feedback-loop-remote: ## Run live shared-host operational proof against the active ssh_systemd remote (separate from make test)
	@python3 ./scripts/tests/live_feedback_loop_remote.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" $(REMOTE_NAME_ARG)

test-live-feedback-loop-remote-unit: ## Validate live shared-host feedback-loop verifier behavior locally
	@echo "$(CYAN)🧪 Running live feedback-loop verifier unit checks...$(NC)"
	@python3 -m unittest scripts.tests.test_live_feedback_loop_remote.LiveFeedbackLoopRemoteBehaviorTests

test-live-feedback-loop-remote-contracts: ## Validate live shared-host feedback-loop wrapper and remote wiring contracts locally
	@echo "$(CYAN)🧪 Running live feedback-loop contract checks...$(NC)"
	@python3 -m unittest scripts.tests.test_live_feedback_loop_remote.LiveFeedbackLoopRemoteContractTests

test-fermyon-edge-signal-smoke: ## Run live trusted-edge signal smoke against the current Fermyon/Akamai deploy receipt
	@python3 ./scripts/tests/fermyon_edge_signal_smoke.py --env-file "$(ENV_LOCAL)" --deploy-receipt "$(FERMYON_AKAMAI_DEPLOY_RECEIPT)" --report-path "$(FERMYON_EDGE_SIGNAL_SMOKE_REPORT)"

test: ## Run the canonical local/CI pre-merge suite: unit, maze verification gate, Spin integration, adversarial matrix, SIM2 gates, and dashboard e2e (excludes live remote proofs)
	@echo "$(CYAN)============================================$(NC)"
	@echo "$(CYAN)  RUNNING ALL TESTS$(NC)"
	@echo "$(CYAN)============================================$(NC)"
	@echo ""
	@echo "$(CYAN)Preflight: waiting up to $(SPIN_READY_TIMEOUT_SECONDS)s for existing Spin server readiness...$(NC)"
	@if ! $(MAKE) --no-print-directory spin-wait-ready; then \
		echo "$(RED)❌ Error: Spin server not ready. Integration tests must run and may not be skipped.$(NC)"; \
		echo "$(YELLOW)   Required flow: 1) make dev  2) make test$(NC)"; \
		exit 1; \
	fi
	@echo "$(GREEN)✅ Preflight: Spin server is ready; integration, adversarial, and dashboard e2e tests will be executed.$(NC)"
	@$(MAKE) --no-print-directory test-runtime-preflight || exit 1
	@echo ""
	@echo "$(CYAN)Step 1/8: Rust Unit Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test || exit 1
	@echo ""
	@echo "$(CYAN)Step 2/8: Maze Verification Gate$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-maze-verification-gate || exit 1
	@echo ""
	@echo "$(CYAN)Step 3/8: Integration Tests (Spin HTTP scenarios)$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-integration-cleanup-contract || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/integration.sh || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not ready. Integration tests must run and may not be skipped.$(NC)"; \
		echo "$(YELLOW)   Start server first: make dev$(NC)"; \
		echo "$(YELLOW)   Then run tests:     make test$(NC)"; \
			exit 1; \
		fi
	@echo ""
	@echo "$(CYAN)Step 4/8: Runtime Toggle Surface Gate$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-adversary-sim-runtime-surface || exit 1
	@echo ""
	@echo "$(CYAN)Step 5/8: Adversarial Fast Matrix (smoke + abuse + Akamai)$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-fast || exit 1
	@echo ""
	@echo "$(CYAN)Step 6/8: SIM2 Realtime Verification Gates$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-sim2-realtime-bench || exit 1
	@$(MAKE) --no-print-directory test-sim2-adr-conformance || exit 1
	@$(MAKE) --no-print-directory test-sim2-ci-diagnostics || exit 1
	@SIM2_MATRIX_TARGET="test-sim2-verification-matrix-advisory"; \
	if ! $(MAKE) --no-print-directory test-adversarial-container-blackbox; then \
		echo "$(YELLOW)Container black-box lane unavailable; continuing with advisory SIM2 matrix validation.$(NC)"; \
	fi; \
	$(MAKE) --no-print-directory $$SIM2_MATRIX_TARGET || exit 1
	@$(MAKE) --no-print-directory test-sim2-operational-regressions || exit 1
	@$(MAKE) --no-print-directory test-sim2-governance-contract || exit 1
	@echo ""
	@echo "$(CYAN)Step 7/8: Dashboard E2E Smoke Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-dashboard-e2e || exit 1
	@echo ""
	@echo "$(CYAN)Step 8/8: Dashboard Seed Snapshot$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory seed-dashboard-data || exit 1
	@echo ""
	@echo "$(GREEN)============================================$(NC)"
	@echo "$(GREEN)  ALL TESTS COMPLETE$(NC)"
	@echo "$(GREEN)============================================$(NC)"

test-unit: ## Run Rust unit tests only (34 tests)
	@echo "$(CYAN)🧪 Running Rust unit tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test

test-native-build-warning-hygiene: ## Fail if focused native Rust test builds emit compiler warnings
	@echo "$(CYAN)🧪 Running focused native build warning-hygiene check...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo clean -p shuma_gorath >/dev/null 2>&1
	@RUSTFLAGS="-D warnings" cargo test --no-run
	@cargo test runtime_var_uses_spin_variable_when_env_is_missing --lib
	@echo "$(GREEN)✅ Focused native Rust test build is warning-free.$(NC)"

test-env-isolation-contract: ## Fail if Rust tests mutate process env without lock_env()
	@echo "$(CYAN)🧪 Running Rust test env-isolation contract...$(NC)"
	@python3 -m unittest scripts/tests/test_rust_env_isolation_contract.py
	@echo "$(GREEN)✅ Rust test env-isolation contract holds.$(NC)"

test-ci-workflow-action-versions: ## Fail if workflow files still pin Node20-backed official GitHub Action majors
	@echo "$(CYAN)🧪 Running CI workflow action-version contract...$(NC)"
	@python3 -m unittest scripts/tests/test_ci_workflow_action_versions.py
	@echo "$(GREEN)✅ CI workflow action-version contract holds.$(NC)"

test-verified-identity-contracts: ## Run focused verified-identity domain contract tests
	@echo "$(CYAN)🧪 Running verified-identity contract tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::tests:: -- --nocapture

test-verified-identity-config: ## Run focused verified-identity config and admin parity tests
	@echo "$(CYAN)🧪 Running verified-identity config parity tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::verified_identity_ -- --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_updates_verified_identity_nested_object -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_rejects_invalid_verified_identity_patch -- --exact --nocapture
	@cargo test admin::api::tests::admin_config_export_returns_non_secret_runtime_values -- --exact --nocapture
	@$(MAKE) --no-print-directory test-config-lifecycle
	@$(MAKE) --no-print-directory test-dashboard-config-surface-contract

test-verified-identity-provider: ## Run focused verified-identity provider seam tests
	@echo "$(CYAN)🧪 Running verified-identity provider seam tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test providers::internal::tests::verified_identity_ -- --nocapture
	@cargo test providers::registry::tests::verified_identity_ -- --nocapture
	@cargo test providers::external::tests::verified_identity_ -- --nocapture

test-verified-identity-native: ## Run focused native verified-identity HTTP Message Signatures tests
	@echo "$(CYAN)🧪 Running native verified-identity tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::native_http_message_signatures::tests:: -- --nocapture
	@cargo test runtime::request_flow::tests::observe_verified_identity_intents_preserve_native_provenance_for_failed_results -- --exact --nocapture

test-verified-identity-directory-discovery: ## Run focused native verified-identity directory discovery/cache tests
	@echo "$(CYAN)🧪 Running verified-identity directory discovery tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::native_http_message_signatures::tests::external_directory_ -- --nocapture

test-verified-identity-proxy-trust: ## Run focused verified-identity proxy and edge trust semantics tests
	@echo "$(CYAN)🧪 Running verified-identity proxy and edge trust tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::native_http_message_signatures::tests::verify_request_accepts_trusted_forwarded_https_for_signed_scheme -- --exact --nocapture
	@cargo test bot_identity::native_http_message_signatures::tests::verify_request_rejects_untrusted_forwarded_https_for_signed_scheme -- --exact --nocapture
	@cargo test bot_identity::native_http_message_signatures::tests::verify_request_accepts_edge_spin_full_url_https_for_signed_scheme -- --exact --nocapture
	@cargo test runtime::upstream_proxy::tests::gateway_forward_preserves_signature_headers_and_strips_shuma_trust_headers -- --exact --nocapture

test-verified-identity-policy: ## Run focused verified-identity policy-registry tests
	@echo "$(CYAN)🧪 Running verified-identity policy tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::policy::tests:: -- --nocapture
	@cargo test runtime::policy_graph::tests:: -- --nocapture
	@cargo test runtime::effect_intents::plan_builder::tests:: -- --nocapture
	@cargo test runtime::traffic_classification::tests:: -- --nocapture

test-verified-identity-telemetry: ## Run focused verified-identity observe-only telemetry tests
	@echo "$(CYAN)🧪 Running verified-identity telemetry tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test bot_identity::tests::verification_telemetry_ -- --nocapture
	@cargo test observability::metrics::tests::render_metrics_includes_verified_identity_monitoring_families -- --exact --nocapture
	@cargo test observability::monitoring::tests::summarize_returns_seeded_maps_when_empty -- --exact --nocapture
	@cargo test observability::monitoring::tests::summarize_aggregates_verified_identity_attempts_and_identities -- --exact --nocapture
	@cargo test observability::hot_read_documents::tests::supporting_summary_contracts_are_narrower_than_bootstrap_document -- --exact --nocapture
	@cargo test runtime::request_flow::tests::observe_verified_identity_ -- --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_returns_structured_summary_shape -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_reports_verified_identity_summary_counts -- --exact --nocapture

test-verified-identity-annotations: ## Run focused verified-identity request-path annotation tests
	@echo "$(CYAN)🧪 Running verified-identity annotation tests...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::traffic_classification::tests::verified_identity_ -- --nocapture
	@cargo test runtime::request_facts::tests::builder_is_side_effect_free_projection_of_inputs -- --exact --nocapture
	@cargo test runtime::request_outcome::tests::verified_identity_ -- --nocapture
	@cargo test runtime::request_flow::tests::observe_verified_identity_ -- --nocapture
	@cargo test runtime::request_flow::tests::finalize_request_outcome_surfaces_verified_identity_lane_in_monitoring_context -- --exact --nocapture

test-verified-identity-make-target-contract: ## Run verified-identity make-target selector contract checks
	@echo "$(CYAN)🧪 Running verified-identity make-target contract checks...$(NC)"
	@python3 -m unittest scripts/tests/test_verified_identity_make_targets.py

test-verified-identity-calibration-readiness: ## Run focused verified-identity calibration-readiness seam checks
	@echo "$(CYAN)🧪 Running verified-identity calibration-readiness checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::traffic_classification::tests::verified_identity_ -- --nocapture
	@cargo test observability::operator_snapshot_verified_identity::tests:: -- --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_materialize_supported_adversary_and_beneficial_non_human_families -- --exact --nocapture
	@cargo test admin::oversight_api::tests::manual_reconcile_route_records_observe_longer_when_classification_is_not_ready -- --exact --nocapture

test-verified-identity-taxonomy-crosswalk: ## Run focused verified-identity taxonomy crosswalk checks
	@echo "$(CYAN)🧪 Running verified-identity taxonomy crosswalk checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::traffic_classification::tests::verified_identity_ -- --nocapture
	@cargo test runtime::request_flow::tests::finalize_request_outcome_surfaces_verified_identity_lane_in_monitoring_context -- --exact --nocapture
	@cargo test observability::monitoring::tests::record_request_outcome_records_non_human_category_counters_for_verified_crosswalks -- --exact --nocapture
	@cargo test observability::non_human_classification::tests::classification_summary_projects_live_verified_category_crosswalk_receipts -- --exact --nocapture
	@cargo test observability::operator_snapshot_non_human::tests::non_human_snapshot_summary_projects_live_verified_search_into_indexing_bot -- --exact --nocapture
	@cargo test observability::operator_snapshot_verified_identity::tests::verified_identity_summary_reports_configured_typed_snapshot_section -- --exact --nocapture

test-verified-identity-alignment-receipts: ## Run focused verified-identity taxonomy alignment receipt checks
	@echo "$(CYAN)🧪 Running verified-identity alignment receipt checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::non_human_classification::tests::verified_identity_alignment_ -- --nocapture
	@cargo test observability::operator_snapshot_verified_identity::tests::verified_identity_summary_projects_taxonomy_alignment_ -- --nocapture

test-verified-identity-botness-conflicts: ## Run focused verified-identity conflict-metric checks
	@echo "$(CYAN)🧪 Running verified-identity conflict metric checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_beneficial_non_human::tests:: -- --nocapture

test-verified-identity-guardrails: ## Run focused verified-identity guardrail checks
	@echo "$(CYAN)🧪 Running verified-identity guardrail checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_results::tests::verified_identity_guardrails_block_tuning_when_conflicts_are_outside_budget -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::observe_longer_when_verified_identity_guardrail_blocks_candidate -- --exact --nocapture

test-host-impact-telemetry: ## Run focused forwarded-latency telemetry and hot-read projection checks
	@echo "$(CYAN)🧪 Running host-impact telemetry checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::request_outcome::tests::forwarded_outcome_preserves_forwarded_upstream_latency -- --exact --nocapture
	@cargo test observability::monitoring::tests::record_request_outcome_records_origin_scope_outcome_and_lane_counters -- --exact --nocapture
	@cargo test observability::monitoring::tests::record_request_outcome_records_non_human_category_counters_for_verified_crosswalks -- --exact --nocapture
	@cargo test observability::monitoring::tests::record_request_outcome_does_not_increment_latency_for_non_forwarded_outcomes -- --exact --nocapture
	@cargo test observability::hot_read_projection::tests::counter_flush_refresh_preserves_request_outcome_summary_rows_in_summary_and_bootstrap -- --exact --nocapture

test-host-impact-make-target-contract: ## Run host-impact make-target selector contract checks
	@echo "$(CYAN)🧪 Running host-impact make-target contract checks...$(NC)"
	@python3 -m unittest scripts/tests/test_host_impact_make_targets.py

test-host-impact-benchmark: ## Run focused host-impact snapshot and benchmark checks
	@echo "$(CYAN)🧪 Running host-impact benchmark checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::operator_snapshot::tests::snapshot_payload_projects_suspicious_forwarded_latency_budget_row -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_materialize_host_impact_metrics_in_suspicious_origin_cost_family -- --exact --nocapture
	@cargo test observability::benchmark_comparison::tests::prior_window_comparison_marks_host_impact_metrics_as_lower_is_better -- --exact --nocapture
	@cargo test observability::benchmark_suite::tests::benchmark_suite_v1_exposes_small_machine_first_family_registry -- --exact --nocapture

test-oversight-host-impact: ## Run focused host-impact reconcile checks
	@echo "$(CYAN)🧪 Running host-impact oversight checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::oversight_reconcile::tests::primary_problem_class_treats_latency_share_budget_miss_as_latency_overspend -- --exact --nocapture

test-ip-range-suggestions: ## Run focused IP-range suggestion regression checks
	@echo "$(CYAN)🧪 Running focused IP-range suggestion regression checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test signals::ip_range_suggestions::tests:: -- --test-threads=1
	@cargo test admin::api::admin_config_tests::admin_ip_range_suggestions_returns_structured_payload -- --test-threads=1
	@$(MAKE) --no-print-directory test-dashboard-unit

test-maze-benchmark: ## Run deterministic maze asymmetry benchmark gate
	@echo "$(CYAN)🧪 Running maze asymmetry benchmark gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test maze::benchmark::tests::maze_asymmetry_benchmark_guardrails_hold -- --nocapture

test-maze-verification-wiring: ## Run focused Makefile/CI wiring proof for the canonical maze verification gate
	@echo "$(CYAN)🧪 Running maze verification wiring checks...$(NC)"
	@python3 -m unittest scripts/tests/test_maze_verification_wiring.py

test-maze-verification-gate: ## Run the canonical maze verification gate: benchmark, live traversal, live browser, and native concurrency proof
	@echo "$(CYAN)🧪 Running canonical maze verification gate...$(NC)"
	@$(MAKE) --no-print-directory test-maze-verification-wiring
	@$(MAKE) --no-print-directory test-maze-benchmark
	@$(MAKE) --no-print-directory test-maze-live-traversal-contract
	@$(MAKE) --no-print-directory test-maze-live-browser-contract
	@$(MAKE) --no-print-directory test-maze-state-concurrency-contract

test-maze-live-traversal-unit: ## Focused unit checks for the live opaque maze traversal gate
	@echo "$(CYAN)🧪 Running live maze traversal gate unit checks...$(NC)"
	@python3 -m unittest scripts/tests/test_maze_live_traversal.py

test-maze-live-traversal-contract: ## Live Spin gate for opaque maze traversal, checkpoint, hidden issuance, and deterministic fallback behavior (requires running server)
	@echo "$(CYAN)🧪 Running live maze traversal integration gate...$(NC)"
	@$(MAKE) --no-print-directory test-maze-live-traversal-unit
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/maze_live_traversal.py; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-maze-live-browser-unit: ## Focused unit checks for the live maze browser gate and browser-driver helpers
	@echo "$(CYAN)🧪 Running live maze browser gate unit checks...$(NC)"
	@node --check scripts/tests/adversarial_browser_driver.mjs
	@node scripts/tests/test_adversarial_browser_driver.mjs
	@python3 -m unittest scripts/tests/test_maze_live_browser.py

test-maze-live-browser-contract: ## Live Spin + Chromium gate for JS/no-JS maze traversal, micro-PoW, replay, and high-confidence escalation (requires running server)
	@echo "$(CYAN)🧪 Running live maze browser integration gate...$(NC)"
	@$(MAKE) --no-print-directory test-maze-live-browser-unit
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/maze_live_browser.py; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-maze-state-concurrency-contract: ## Focused native burst/concurrency proof for maze budget, replay, and checkpoint state primitives
	@echo "$(CYAN)🧪 Running maze state concurrency contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test deception::primitives::tests::shared_budget_parallel_acquire_stays_bounded -- --exact --nocapture
	@cargo test maze::runtime::tests::concurrent_replay_claim_allows_single_winner -- --exact --nocapture
	@cargo test maze::runtime::tests::concurrent_checkpoint_writes_reuse_single_state_key -- --exact --nocapture

test-runtime-preflight-unit: ## Verify full-suite runtime preflight helper behavior (no server required)
	@echo "$(CYAN)🧪 Running full-suite runtime preflight helper tests...$(NC)"
	@python3 -m unittest scripts/tests/test_verify_test_runtime_environment.py

test-runtime-preflight: ## Verify existing server runtime matches the full-suite contract (requires running server)
	@echo "$(CYAN)🧪 Verifying full-suite runtime contract...$(NC)"
	@bash ./scripts/tests/verify_test_runtime_environment.sh --expected-runtime-environment runtime-dev

unit-test: test-unit ## Alias for Rust unit tests

test-integration: ## Run local Spin integration tests only (28 scenarios, requires running server)
	@echo "$(CYAN)🧪 Running integration tests...$(NC)"
	@$(MAKE) --no-print-directory test-integration-cleanup-contract
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/integration.sh; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev (or make dev-prod / make prod)$(NC)"; \
		exit 1; \
	fi

integration-test: test-integration ## Alias for Spin integration tests

test-integration-cleanup-contract: ## Verify integration shell cleanup and restore contract (no server required)
	@echo "$(CYAN)🧪 Running integration shell cleanup contract checks...$(NC)"
	@python3 -m unittest scripts/tests/test_integration_cleanup.py

test-gateway-harness: ## Run deterministic gateway upstream fixture + failure harness checks (no Spin server required)
	@echo "$(CYAN)🧪 Running gateway fixture/failure harness checks...$(NC)"
	@python3 -m unittest scripts/tests/test_validate_gateway_contract.py
	@python3 -m unittest scripts/tests/test_validate_gateway_route_collisions.py
	@python3 -m unittest scripts/tests/test_gateway_failure_harness.py
	@python3 -m unittest scripts/tests/test_gateway_tls_wasm_harness.py
	@python3 -m unittest scripts/tests/test_probe_gateway_origin_bypass.py
	@python3 scripts/tests/gateway_failure_harness.py

test-gateway-wasm-tls-harness: ## Run wasm32 gateway TLS failure matrix (expired/self-signed/hostname-mismatch; external egress required)
	@echo "$(CYAN)🧪 Running wasm32 gateway TLS failure matrix harness...$(NC)"
	@python3 scripts/tests/gateway_tls_wasm_harness.py --json-output $(GATEWAY_TLS_WASM_REPORT)

test-gateway-origin-bypass-probe: ## Optional active direct-origin bypass probe (requires GATEWAY_PROBE_GATEWAY_URL + GATEWAY_PROBE_ORIGIN_URL)
	@echo "$(CYAN)🧪 Running optional gateway origin-bypass active probe...$(NC)"
	@if [ -z "$(GATEWAY_PROBE_GATEWAY_URL)" ] || [ -z "$(GATEWAY_PROBE_ORIGIN_URL)" ]; then \
		echo "$(RED)❌ Missing required URLs. Set GATEWAY_PROBE_GATEWAY_URL and GATEWAY_PROBE_ORIGIN_URL.$(NC)"; \
		exit 1; \
	fi
	@PROBE_EXTRA_ARGS=""; \
	if [ "$(GATEWAY_PROBE_FAIL_ON_INCONCLUSIVE)" = "1" ]; then \
		PROBE_EXTRA_ARGS="--fail-on-inconclusive"; \
	fi; \
	python3 scripts/deploy/probe_gateway_origin_bypass.py \
		--gateway-url "$(GATEWAY_PROBE_GATEWAY_URL)" \
		--origin-url "$(GATEWAY_PROBE_ORIGIN_URL)" \
		--probe-path "$(GATEWAY_PROBE_PATH)" \
		--json-output "$(GATEWAY_PROBE_JSON_OUTPUT)" \
		$$PROBE_EXTRA_ARGS

test-gateway-profile-shared-server: ## Verify shared-server gateway contract + forwarding behavior
	@echo "$(CYAN)🧪 Running gateway shared-server profile verification...$(NC)"
	@python3 -m unittest scripts/tests/test_validate_gateway_contract.py
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::upstream_canonicalization::tests::canonicalize_forward_path_strips_absolute_uri_authority -- --nocapture
	@cargo test --test routing_order_integration -- --nocapture

test-gateway-profile-edge: ## Verify edge/Fermyon gateway contract + signed-header origin-auth behavior
	@echo "$(CYAN)🧪 Running gateway edge/Fermyon profile verification...$(NC)"
	@python3 -m unittest scripts/tests/test_validate_gateway_contract.py
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::validate_env_accepts_edge_profile_with_signed_header_origin_auth_contract -- --nocapture
	@cargo test config::tests::validate_env_rejects_edge_profile_without_signed_header_origin_auth -- --nocapture
	@cargo test runtime::upstream_proxy::tests::signed_header_origin_auth_is_proxy_owned_and_overrides_client_value -- --nocapture

smoke-gateway-mode: ## Fast gateway smoke: origin reachability, allow-forwarding, enforcement-local behavior, and fail-closed outage handling
	@echo "$(CYAN)🧪 Running gateway mode smoke checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test --test routing_order_integration allow_path_forwards_fidelity_and_regenerates_trusted_forwarded_headers -- --nocapture
	@cargo test --test routing_order_integration enforcement_paths_remain_local_and_do_not_require_upstream -- --nocapture
	@cargo test --test routing_order_integration allow_paths_fail_closed_when_upstream_forwarding_is_unavailable -- --nocapture

test-deploy-linode: ## Validate Linode deploy-path helpers and production input gates
	@echo "$(CYAN)🧪 Running Linode deploy-path verification...$(NC)"
	@python3 -m unittest scripts/tests/test_build_linode_release_bundle.py
	@python3 -m unittest scripts/tests/test_build_site_surface_catalog.py
	@python3 -m unittest scripts/tests/test_local_state_contract.py
	@python3 -m unittest scripts/tests/test_validate_gateway_route_collisions.py
	@python3 -m unittest scripts/tests/test_prepare_linode_shared_host.py
	@python3 -m unittest scripts/tests/test_merge_env_overlay.py
	@python3 -m unittest scripts/tests/test_remote_target.py
	@python3 -m unittest scripts/tests/test_remote_edge_signal_smoke.py
	@python3 -m unittest scripts/tests/test_render_gateway_spin_manifest.py
	@python3 -m unittest scripts/tests/test_deploy_linode_one_shot.py
	@python3 -m unittest scripts/tests/test_prod_start_spin_manifest.py
	@python3 -m unittest scripts/tests/test_select_gateway_smoke_path.py
	@python3 -m unittest scripts/tests/test_setup_runtime_spin_install.py
	@python3 -m unittest scripts/tests/test_smoke_single_host.py
	@python3 -m unittest scripts/tests/test_wait_for_spin_ready.py

test-setup-runtime-bootstrap: ## Validate runtime bootstrap installers, including Scrapling venv prerequisites
	@echo "$(CYAN)🧪 Running setup-runtime bootstrap verification...$(NC)"
	@python3 -m unittest scripts/tests/test_setup_runtime_spin_install.py

test-remote-target-contract: ## Validate focused ssh-managed remote target helper contracts
	@echo "$(CYAN)🧪 Running remote target helper contract verification...$(NC)"
	@python3 -m unittest scripts/tests/test_remote_target.py

test-scrapling-deploy-shared-host: ## Validate shared-host Scrapling deploy prep, Linode wiring, and day-2 remote-update receipt contract
	@echo "$(CYAN)🧪 Running shared-host Scrapling deploy verification...$(NC)"
	@python3 -m unittest scripts/tests/test_scrapling_deploy_prep.py
	@python3 -m unittest scripts/tests/test_remote_target.py
	@python3 -m unittest scripts/tests/test_deploy_linode_one_shot.py

test-deploy-fermyon: ## Validate Fermyon/Akamai edge setup and deploy helpers
	@echo "$(CYAN)🧪 Running Fermyon/Akamai edge deploy-path verification...$(NC)"
	@python3 -m unittest scripts/tests/test_deploy_profile_baseline.py
	@python3 -m unittest scripts/tests/test_render_gateway_spin_manifest.py
	@python3 -m unittest scripts/tests/test_prepare_fermyon_akamai_edge.py
	@python3 -m unittest scripts/tests/test_deploy_fermyon_akamai_edge.py
	@python3 -m unittest scripts/tests/test_fermyon_edge_signal_smoke.py
	@python3 -m unittest scripts/tests/test_config_lifecycle.py
	@cargo test validate_env_only_accepts_spin_variables_in_tests --lib
	@cargo test env_string_required_uses_spin_variable_when_env_missing --lib
	@cargo test runtime_var_uses_spin_variable_when_env_is_missing --lib
	@cargo test default_seeded_config_matches_defaults_snapshot --lib
	@cargo test admin_config_bootstraps_missing_config_from_defaults_on_write --lib
	@cargo test validate_env_rejects_edge_profile_without_adversary_sim_edge_cron_secret --lib
	@cargo test edge_fermyon_uses_true_client_ip_for_client_ip_extraction --lib
	@cargo test edge_fermyon_treats_spin_full_url_https_as_https --lib
	@cargo test admin_ip_allowlist_uses_true_client_ip_on_edge_fermyon --lib
	@cargo test internal_adversary_sim_edge_cron_request_requires_edge_profile_https_and_secret --lib
	@cargo test adversary_sim_edge_cron_bypass_is_scoped_to_beat_path_only --lib
	@cargo test adversary_sim_control_enable_reports_edge_cron_warming_before_first_tick --lib
	@cargo test supervisor_status_payload_reports_edge_cron_truthfully_before_first_tick --lib
	@cargo test generation_diagnostics_waits_full_edge_interval_before_no_traffic --lib
	@cargo test edge_fermyon_generated_request_target_stays_within_bounded_budget --lib
	@cargo test edge_fermyon_supplemental_lane_rotation_covers_full_contract --lib

test-config-lifecycle: ## Validate read-only runtime config lifecycle checks and explicit seed/backfill flows
	@echo "$(CYAN)🧪 Running config lifecycle verification...$(NC)"
	@python3 -m unittest scripts/tests/test_config_lifecycle.py
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::adversary_sim_duration_defaults_to_30_and_clamps_loaded_values -- --exact --nocapture

test-js-verification-unit: ## Run focused JS verification interstitial unit checks
	@echo "$(CYAN)🧪 Running JS verification interstitial checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test signals::js_verification::tests:: -- --nocapture

test-shadow-mode: ## Run focused shadow-mode backend truthfulness checks
	@echo "$(CYAN)🧪 Running shadow-mode backend checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::shadow_mode::tests:: -- --nocapture
	@cargo test runtime::effect_intents::intent_executor::tests::prepare_intents_for_shadow_suppresses_enforcement_and_records_shadow_action -- --exact --nocapture
	@cargo test observability::monitoring::tests::summarize_shadow_metrics_uses_bucket_indexes_without_full_keyspace_scan -- --exact --nocapture
	@cargo test admin::api::tests::log_event_with_execution_metadata_persists_shadow_fields_without_source_field -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_returns_structured_summary_shape -- --exact --nocapture

test-enterprise-ban-store-contract: ## Run focused enterprise ban-store outage-mode and guardrail contract checks
	@echo "$(CYAN)🧪 Running enterprise ban-store contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::parse_ban_store_outage_mode_accepts_expected_values -- --exact --nocapture
	@cargo test config::tests::enterprise_state_guardrail_requires_fail_closed_ban_store_outage_mode_for_authoritative_enterprise -- --exact --nocapture
	@cargo test config::tests::enterprise_state_guardrail_is_clear_for_synced_multi_instance_posture -- --exact --nocapture
	@cargo test config::tests::validate_env_rejects_invalid_optional_ban_store_outage_mode -- --exact --nocapture
	@cargo test runtime::policy_pipeline::tests::fail_closed_ban_store_outage_maps_unavailable_lookup_to_existing_ban -- --exact --nocapture
	@cargo test runtime::policy_pipeline::tests::non_strict_ban_store_outage_does_not_map_unavailable_lookup_to_existing_ban -- --exact --nocapture
	@cargo test providers::external::tests::distributed_ban_ -- --nocapture
	@cargo test providers::external::tests::distributed_unban_ -- --nocapture
	@cargo test providers::registry::tests::registry_reports_active_provider_implementation_labels -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_ip_bans_delta_marks_active_bans_unavailable_when_strict_backend_is_unavailable -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::monitoring_details_payload_marks_ban_state_unavailable_when_strict_backend_is_unavailable -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::manual_ban_write_result_returns_503_without_logging_success_when_sync_fails -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::manual_unban_write_result_returns_503_without_logging_success_when_sync_fails -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::active_ban_list_result_returns_503_when_backend_is_unavailable -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_export_returns_non_secret_runtime_values -- --exact --nocapture
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard API adapters normalize sparse payloads safely|dashboard API client exposes cursor-delta and stream URL helpers for realtime tabs|dashboard refresh runtime preserves unavailable ban-state markers instead of coercing them to zero' \
		e2e/dashboard.modules.unit.test.js

test-telemetry-storage: ## Run focused telemetry storage/query verification for indexed reads, retention tiers, rollups, and shared-host evidence tooling
	@echo "$(CYAN)🧪 Running telemetry storage/query verification...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::effect_intents::plan_builder::tests::botness_challenge_plan_avoids_verbose_blended_outcome_strings -- --exact --nocapture
	@cargo test admin::api::tests::log_event_omits_absent_optional_fields_in_persisted_row -- --exact --nocapture
	@cargo test admin::api::tests::log_event_persists_structured_taxonomy_separately_from_outcome_text -- --exact --nocapture
	@cargo test admin::api::tests::log_event_persists_compact_botness_outcome_fields_without_verbose_challenge_payload -- --exact --nocapture
	@cargo test admin::api::tests::log_event_persists_sparse_js_verification_taxonomy_and_omits_default_simulation_flag -- --exact --nocapture
	@cargo test admin::api::tests::log_event_persists_sparse_botness_taxonomy_without_redundant_action_or_detection -- --exact --nocapture
	@cargo test observability::monitoring::tests::summarize_uses_bucket_indexes_without_full_keyspace_scan -- --exact --nocapture
	@cargo test observability::monitoring::tests::summarize_builds_and_reuses_day_rollups_for_complete_prior_days -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_delta_reads_bucket_indexes_without_keyspace_scan -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_cost_governance_accounts_for_bucket_density -- --exact --nocapture
	@cargo test observability::retention::tests::eventlog_retention_is_capped_while_monitoring_retention_tracks_config -- --exact --nocapture
	@cargo test observability::retention::tests::worker_purges_expired_bucket_and_updates_watermark -- --exact --nocapture

test-telemetry-hot-read-contract: ## Run focused telemetry hot-read exactness/projection contract checks
	@echo "$(CYAN)🧪 Running telemetry hot-read contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::hot_read_contract::tests:: -- --nocapture
	@cargo test observability::hot_read_documents::tests:: -- --nocapture
	@python3 -m unittest scripts/tests/test_telemetry_shared_host_evidence.py

test-telemetry-hot-read-projection: ## Run focused telemetry hot-read projection maintenance checks
	@echo "$(CYAN)🧪 Running telemetry hot-read projection checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::hot_read_projection::tests:: -- --nocapture
	@cargo test admin::api::tests::log_event_refreshes_hot_read_recent_events_tail_projection -- --exact --nocapture
	@cargo test admin::api::tests::log_event_refreshes_recent_sim_run_history_without_event_tail_eviction -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_post_refreshes_hot_read_bootstrap_projection -- --exact --nocapture
	@cargo test observability::retention::tests::run_worker_if_due_refreshes_hot_read_retention_projection -- --exact --nocapture
	@cargo test enforcement::ban::tests::ban_and_unban_refresh_hot_read_bootstrap_projection -- --exact --nocapture

test-telemetry-hot-read-bootstrap: ## Run focused telemetry bootstrap hot-read consumption checks
	@echo "$(CYAN)🧪 Running telemetry hot-read bootstrap checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::admin_config_tests::admin_monitoring_returns_compact_recent_event_shape_in_default_and_forensic_modes -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_bootstrap_prefers_materialized_hot_read_documents_without_keyspace_scan -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_edge_profile_bootstrap_prefers_hot_read_even_with_oversized_limit -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_delta_bootstrap_prefers_hot_read_tail_and_security_summary -- --exact --nocapture

test-monitoring-telemetry-contract: ## Run focused external-only monitoring telemetry checks
	@echo "$(CYAN)🧪 Running focused monitoring telemetry contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::admin_config_tests::admin_monitoring_excludes_admin_originated_rows_from_external_telemetry -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_defaults_to_pseudonymized_view_and_supports_forensic_mode -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_returns_compact_recent_event_shape_in_default_and_forensic_modes -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_delta_pseudonymizes_without_forensic_ack -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_delta_bootstrap_prefers_hot_read_tail_and_security_summary -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_includes_simulation_and_baseline_events -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_keeps_live_summary_truth_separate_from_simulation_details -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_keeps_simulation_event_parity_for_equivalent_outcomes -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::adversary_sim_auto_off_preserves_historical_monitoring_visibility -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::adversary_sim_history_cleanup_endpoint_clears_retained_telemetry -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::adversary_sim_history_cleanup_allows_runtime_prod_with_ack_header -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_uses_bounded_details_for_edge_profiles -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_monitoring_delta_reads_bucket_indexes_without_keyspace_scan -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_monitoring_delta_includes_freshness_and_load_contracts -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_monitoring_snapshot_includes_freshness_and_load_contracts -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_monitoring_snapshot_exposes_extended_operator_summary_contract -- --exact --nocapture

test-monitoring-telemetry-foundation-unit: ## Run focused unit checks for monitoring telemetry foundation contracts and runtime classification scaffolding
	@echo "$(CYAN)🧪 Running monitoring telemetry foundation unit checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::monitoring::tests:: -- --nocapture
	@cargo test observability::hot_read_contract::tests:: -- --nocapture
	@cargo test observability::hot_read_documents::tests:: -- --nocapture
	@cargo test observability::hot_read_projection::tests:: -- --nocapture
	@cargo test runtime::traffic_classification::tests:: -- --nocapture
	@cargo test runtime::request_outcome::tests:: -- --nocapture
	@cargo test runtime::request_flow::tests:: -- --nocapture
	@cargo test runtime::effect_intents::intent_executor::tests:: -- --nocapture
	@cargo test runtime::architecture_guards::pure_decision_modules_do_not_depend_on_runtime_side_effect_surfaces -- --exact --nocapture

test-operator-snapshot-foundation: ## Run focused operator snapshot foundation checks
	@echo "$(CYAN)🧪 Running operator snapshot foundation checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::allowed_actions_v1_exposes_conservative_controller_write_surface -- --exact --nocapture
	@cargo test config::tests::controller_config_family_for_patch_key_reuses_allowed_action_catalog -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests:: -- --nocapture
	@cargo test observability::hot_read_documents::tests::operator_snapshot_ -- --nocapture
	@cargo test observability::hot_read_projection::tests::counter_flush_refresh_operator_snapshot -- --nocapture
	@cargo test admin::api::tests::operator_snapshot_recent_changes_ledger_tracks_changed_config_families -- --exact --nocapture
	@cargo test admin::api::tests::operator_snapshot_recent_changes_ledger_ignores_requested_families_without_diff -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_updates_materialize_recent_changes_in_operator_snapshot -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_operator_snapshot_returns_machine_first_snapshot_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_operator_snapshot_returns_503_without_materializing_on_read -- --exact --nocapture

test-traffic-taxonomy-contract: ## Run focused non-human taxonomy and snapshot taxonomy-contract checks
	@echo "$(CYAN)🧪 Running non-human taxonomy contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::traffic_classification::tests::canonical_non_human_taxonomy_exposes_stable_machine_and_operator_facing_metadata -- --exact --nocapture
	@cargo test observability::non_human_lane_fulfillment::tests:: -- --nocapture
	@cargo test observability::operator_snapshot_non_human::tests:: -- --nocapture
	@cargo test observability::hot_read_contract::tests::operator_snapshot_contracts_include_budget_distance_and_runtime_posture -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_uses_persisted_objective_profile_and_typed_verified_identity_summary -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_keeps_live_and_adversary_sim_sections_separate -- --exact --nocapture

test-traffic-classification-contract: ## Run focused non-human classification receipt and gating checks
	@echo "$(CYAN)🧪 Running non-human classification contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime::traffic_classification::tests::non_human_lane_assignments_follow_seeded_taxonomy_contract -- --exact --nocapture
	@cargo test observability::non_human_classification::tests:: -- --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_fail_closed_when_non_human_classification_is_not_ready -- --exact --nocapture
	@cargo test observability::operator_snapshot_non_human::tests:: -- --nocapture

test-adversarial-coverage-receipts: ## Run focused canonical category-coverage receipt checks across adversarial contracts and snapshot/benchmark gates
	@echo "$(CYAN)🧪 Running adversarial category-coverage receipt checks...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract
	@$(MAKE) --no-print-directory test-adversarial-scenario-review
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::tests::recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories -- --exact --nocapture
	@cargo test observability::non_human_coverage::tests:: -- --nocapture
	@cargo test observability::operator_snapshot_non_human::tests:: -- --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_fail_closed_when_non_human_coverage_is_not_ready -- --exact --nocapture

test-protected-tuning-evidence: ## Run focused protected-evidence eligibility checks across replay promotion, snapshot, benchmark, and oversight gates
	@echo "$(CYAN)🧪 Running protected tuning evidence checks...$(NC)"
	@$(MAKE) --no-print-directory test-replay-promotion-contract
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_results::tests::benchmark_results_fail_closed_when_protected_tuning_evidence_is_not_ready -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_allow_strong_live_scrapling_runtime_without_replay_lineage -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::refuse_stale_replay_metadata_when_replay_promoted_lineage_is_current_basis -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::allow_stale_replay_metadata_when_live_runtime_protected_basis_is_current -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::advisory_only_replay_promotion_requires_review_verification -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_surfaces_materialized_replay_promotion_summary -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_returns_bounded_current_instance_contract -- --exact --nocapture

test-admin-machine-contracts: ## Run focused admin read-contract checks for recent changes, operator snapshot, benchmark endpoints, and durable oversight observer history
	@echo "$(CYAN)🧪 Running focused admin machine-contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::tests::recent_sim_run_history_prefers_explicit_scrapling_category_targets_when_profile_is_generic -- --exact --nocapture
	@cargo test admin::oversight_observer_round_archive::tests:: -- --nocapture
	@cargo test admin::api::admin_config_tests::post_sim_oversight_history_and_status_preserve_judged_mixed_attacker_lane_basis -- --exact --nocapture
	@cargo test admin::api::tests::operator_snapshot_recent_changes_ledger_tracks_changed_config_families -- --exact --nocapture
	@cargo test admin::api::tests::operator_snapshot_recent_changes_ledger_ignores_requested_families_without_diff -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_updates_materialize_recent_changes_in_operator_snapshot -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_operator_snapshot_returns_machine_first_snapshot_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_operator_snapshot_returns_503_without_materializing_on_read -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_suite_returns_machine_first_benchmark_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_suite_is_get_only -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_returns_bounded_current_instance_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_returns_503_without_materialized_snapshot -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_is_get_only -- --exact --nocapture
	@cargo test admin::replay_promotion_api::tests:: -- --nocapture

test-admin-api-routing-contract: ## Run focused admin route-family contract checks for structural API refactors
	@echo "$(CYAN)🧪 Running admin route-family contract checks...$(NC)"
	@$(MAKE) --no-print-directory test-admin-machine-contracts
	@$(MAKE) --no-print-directory test-monitoring-telemetry-contract
	@$(MAKE) --no-print-directory test-monitoring-telemetry-foundation-unit
	@$(MAKE) --no-print-directory test-adversary-sim-domain-contract

test-tarpit-observability-contract: ## Run focused tarpit observability/admin contract checks
	@echo "$(CYAN)🧪 Running tarpit observability contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::metrics::tests::tarpit_observability_render_metrics_includes_extended_tarpit_families -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::tarpit_observability_monitoring_payload_projects_extended_tarpit_metrics -- --exact --nocapture
	@cargo test observability::key_catalog::tests::register_key_capped_refuses_new_entries_past_cap -- --exact --nocapture
	@cargo test providers::internal::tests::tarpit_entry_budget_exhaustion_reason_distinguishes_cap_sources -- --exact --nocapture
	@cargo test tarpit::types::tests::progress_reject_reason_chain_violation_labels_are_stable -- --exact --nocapture

test-tarpit-collateral-risk-contract: ## Run focused tarpit collateral-risk guardrail checks
	@echo "$(CYAN)🧪 Running tarpit collateral-risk contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test tarpit::runtime::tests::exact_principal_persistence_counts_do_not_share_same_bucket_state -- --exact --nocapture
	@cargo test tarpit::runtime::tests::exact_principal_persistence_tracking_fails_open_when_catalog_is_full -- --exact --nocapture
	@cargo test providers::internal::tests::tarpit_persistence_escalation_does_not_cross_contaminate_same_bucket_ips -- --exact --nocapture

test-controller-mutability-policy: ## Run focused controller mutability-ring policy checks
	@echo "$(CYAN)🧪 Running controller mutability policy checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::controller_mutability_policy_classifies_operator_and_admin_surfaces -- --exact --nocapture
	@cargo test config::tests::controller_mutability_policy_marks_hard_never_paths_as_permanently_forbidden -- --exact --nocapture

test-controller-action-surface: ## Run focused allowed-actions and controller-family mapping checks
	@echo "$(CYAN)🧪 Running controller action-surface checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::allowed_actions_v1_exposes_conservative_controller_write_surface -- --exact --nocapture
	@cargo test config::tests::controller_config_family_for_patch_key_reuses_allowed_action_catalog -- --exact --nocapture

test-controller-action-surface-parity: ## Run focused parity checks across mutability policy, allowed-actions, benchmark escalation, and patch shaping
	@echo "$(CYAN)🧪 Running controller action-surface parity checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::allowed_actions_v1_exposes_conservative_controller_write_surface -- --exact --nocapture
	@cargo test config::tests::controller_config_family_for_patch_key_reuses_allowed_action_catalog -- --exact --nocapture
	@cargo test config::tests::verified_identity_allowed_actions_surface_is_forbidden -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_proposes_config_tuning_for_addressable_budget_breach -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_filters_out_controller_forbidden_families -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::challenge_threshold_patch_matches_challenge_family_metadata -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::mixed_family_with_allowed_group_remains_proposable -- --exact --nocapture

test-controller-hard-boundaries: ## Run focused hard-boundary rejection checks for controller-forbidden surfaces
	@echo "$(CYAN)🧪 Running controller hard-boundary checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::controller_mutability_policy_marks_hard_never_paths_as_permanently_forbidden -- --exact --nocapture
	@cargo test config::tests::verified_identity_allowed_actions_surface_is_forbidden -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::forbidden_verified_identity_family_is_rejected -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::forbidden_provider_selection_family_is_rejected -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::forbidden_robots_policy_family_is_rejected -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::forbidden_allowlists_family_is_rejected -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::forbidden_tarpit_family_is_rejected -- --exact --nocapture
	@cargo test admin::oversight_apply::tests::apply_refuses_non_tunable_proposals_even_if_one_is_present -- --exact --nocapture

test-rsi-game-contract: ## Run focused recursive-improvement game-contract checks
	@echo "$(CYAN)🧪 Running recursive-improvement game-contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_names_rules_evaluator_moves_gates_and_anchors -- --exact --nocapture
	@cargo test observability::hot_read_contract::tests::operator_snapshot_contracts_include_budget_distance_and_runtime_posture -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_operator_snapshot_returns_machine_first_snapshot_contract -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_uses_persisted_objective_profile_and_typed_verified_identity_summary -- --exact --nocapture
	@cargo test admin::oversight_api::tests::manual_reconcile_route_records_observe_longer_when_classification_is_not_ready -- --exact --nocapture

test-rsi-scorecard-contract: ## Run focused recursive-improvement judge-scorecard checks
	@echo "$(CYAN)🧪 Running recursive-improvement judge-scorecard checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_names_rules_evaluator_moves_gates_and_anchors -- --exact --nocapture
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_partitions_metric_ids_without_collapsing_to_scalar -- --exact --nocapture

test-rsi-score-exploit-progress: ## Run focused exploit-progress judge and comparison checks
	@echo "$(CYAN)🧪 Running exploit-progress judge and comparison checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_suite::tests::benchmark_suite_v1_exposes_small_machine_first_family_registry -- --exact --nocapture
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_names_rules_evaluator_moves_gates_and_anchors -- --exact --nocapture
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_partitions_metric_ids_without_collapsing_to_scalar -- --exact --nocapture
	@cargo test observability::benchmark_comparison::tests::prior_window_comparison_marks_new_exploit_loci_as_regressed_even_when_metrics_are_flat -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_marks_scrapling_exploit_progress_gap_as_code_evolution_only -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_accept_when_latest_scrapling_surface_contract_is_covered -- --exact --nocapture
	@$(MAKE) test-adversary-sim-scrapling-coverage-receipts

test-rsi-score-evidence-quality: ## Run focused exploit-evidence quality and diagnosis-confidence checks
	@echo "$(CYAN)🧪 Running exploit-evidence quality checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_results::tests::benchmark_results_block_tuning_when_exploit_progress_evidence_is_low_confidence -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_mark_exploit_progress_evidence_high_confidence_when_reproduced_and_localized -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::observe_longer_when_exploit_progress_evidence_is_low_confidence -- --exact --nocapture
	@$(MAKE) test-benchmark-results-contract
	@$(MAKE) test-oversight-reconcile

test-rsi-score-urgency-and-homeostasis: ## Run focused urgency scoring and homeostasis-break checks
	@echo "$(CYAN)🧪 Running urgency and homeostasis checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_urgency::tests:: -- --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_materialize_critical_urgency_when_exploit_progress_regresses -- --exact --nocapture
	@cargo test observability::benchmark_comparison::tests::homeostasis_breaks_immediately_when_latest_cycle_reports_exploit_regression -- --exact --nocapture
	@$(MAKE) test-benchmark-results-contract
	@$(MAKE) test-oversight-episode-archive

test-rsi-score-move-selection: ## Run focused move-selection and config-exhaustion checks
	@echo "$(CYAN)🧪 Running move-selection and config-exhaustion checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::oversight_patch_policy::tests::rank_patch_candidates_prefers_smallest_low_friction_candidate_first -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::reconcile_surfaces_selected_move_lineage_for_localized_gap -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::reconcile_promotes_code_evolution_candidate_to_first_class_referral -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::reconcile_emits_config_ring_exhausted_after_repeated_failed_bounded_moves -- --exact --nocapture
	@$(MAKE) test-oversight-reconcile
	@$(MAKE) test-controller-action-surface

test-oversight-move-selection-policy: ## Run focused RSI-GAME-1B move-selection policy checks
	@echo "$(CYAN)🧪 Running oversight move-selection policy checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_names_problem_class_trigger_metrics_and_guidance -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_treats_category_posture_gap_as_recognition_side_quest -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::latency_problem_class_prefers_signal_families_before_higher_friction_moves -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::recommend_patch_when_outside_budget_maps_to_bounded_candidate_family -- --exact --nocapture

test-benchmark-suite-contract: ## Run focused machine-first benchmark suite contract checks
	@echo "$(CYAN)🧪 Running benchmark suite contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_suite::tests:: -- --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_suite_returns_machine_first_benchmark_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_suite_is_get_only -- --exact --nocapture

test-benchmark-results-contract: ## Run focused machine-first benchmark results contract checks
	@echo "$(CYAN)🧪 Running benchmark results contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::non_human_classification::tests:: -- --nocapture
	@cargo test observability::benchmark_non_human_categories::tests:: -- --nocapture
	@cargo test observability::benchmark_results::tests:: -- --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_returns_bounded_current_instance_contract -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_returns_503_without_materialized_snapshot -- --exact --nocapture
	@cargo test admin::api::tests::handle_admin_benchmark_results_is_get_only -- --exact --nocapture

test-benchmark-comparison-contract: ## Run focused benchmark comparison helper contract checks
	@echo "$(CYAN)🧪 Running benchmark comparison contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_results_comparison::tests:: -- --nocapture

test-benchmark-category-eligibility: ## Run focused category-aware benchmark eligibility and comparison checks
	@echo "$(CYAN)🧪 Running category-aware benchmark eligibility checks...$(NC)"
	@$(MAKE) --no-print-directory test-benchmark-suite-contract
	@$(MAKE) --no-print-directory test-benchmark-results-contract
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_comparison::tests:: -- --nocapture

test-operator-objectives-contract: ## Run focused operator-objectives profile and snapshot wiring checks
	@echo "$(CYAN)🧪 Running operator objectives contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::operator_snapshot_objectives::tests:: -- --nocapture
	@cargo test observability::operator_objectives_store::tests:: -- --nocapture
	@cargo test observability::decision_ledger::tests:: -- --nocapture
	@cargo test admin::operator_objectives_api::tests:: -- --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_uses_persisted_objective_profile_and_typed_verified_identity_summary -- --exact --nocapture

test-operator-objectives-category-contract: ## Run focused category-aware operator-objectives contract checks
	@echo "$(CYAN)🧪 Running category-aware operator objectives contract checks...$(NC)"
	@$(MAKE) --no-print-directory test-operator-objectives-contract

test-oversight-reconcile: ## Run focused recommend-only oversight reconcile checks
	@echo "$(CYAN)🧪 Running oversight reconcile checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::oversight_patch_policy::tests:: -- --nocapture
	@cargo test admin::oversight_reconcile::tests:: -- --nocapture
	@cargo test admin::oversight_decision_ledger::tests:: -- --nocapture
	@cargo test admin::oversight_api::tests:: -- --nocapture

test-oversight-agent: ## Run focused shared-host oversight agent contract checks
	@echo "$(CYAN)🧪 Running oversight agent checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test internal_oversight_supervisor_request_requires_marker_bearer_secret_https_and_loopback -- --nocapture
	@cargo test admin::oversight_follow_on_runs::tests:: -- --nocapture
	@cargo test admin::oversight_agent::tests:: -- --nocapture
	@cargo test internal_agent_route_records_periodic_run_and_status_surface -- --nocapture

test-oversight-episode-archive: ## Run focused oversight episode-archive and homeostasis checks
	@echo "$(CYAN)🧪 Running oversight episode-archive checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_comparison::tests::homeostasis_requires_ten_completed_cycle_judgments_before_classifying -- --exact --nocapture
	@cargo test observability::benchmark_comparison::tests::homeostasis_distinguishes_improving_mixed_and_flat_recent_cycles -- --exact --nocapture
	@cargo test admin::oversight_api::tests::manual_reconcile_route_records_observe_longer_when_classification_is_not_ready -- --exact --nocapture
	@cargo test admin::oversight_agent::tests::agent_cycle_rolls_back_canary_when_candidate_window_regresses -- --exact --nocapture
	@cargo test admin::oversight_agent::tests::agent_cycle_keeps_canary_when_candidate_window_improves -- --exact --nocapture

test-rsi-game-mainline: ## Run focused first-working-loop mainline proof checks
	@echo "$(CYAN)🧪 Running first-working-loop mainline proof checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_completion_triggers_post_sim_oversight_agent_once -- --nocapture
	@cargo test adversary_sim_internal_beat_auto_starts_pending_loop_continuation_run_once_after_terminal_improved -- --nocapture
	@cargo test post_sim_oversight_route_can_apply_improve_and_archive_first_working_game_loop -- --nocapture
	@cargo test post_sim_oversight_route_records_repeated_retained_and_rolled_back_episodes_against_changed_configs -- --nocapture
	@python3 -m unittest scripts.tests.test_live_feedback_loop_remote.LiveFeedbackLoopRemoteBehaviorTests.test_run_records_terminal_follow_on_judgment_and_episode_archive

test-rsi-game-mixed-episode-orchestration: ## Run focused mixed-attacker candidate-window and continuation sequencing checks
	@echo "$(CYAN)🧪 Running mixed-attacker episode orchestration checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_candidate_window_sequences_required_scrapling_then_bot_red_team_lanes -- --nocapture
	@cargo test adversary_sim_loop_continuation_waits_for_all_required_lanes_before_post_sim_judgment -- --nocapture

.PHONY: test-rsi-game-mixed-proof-projection
test-rsi-game-mixed-proof-projection: ## Run focused mixed-attacker judged-episode projection checks
	@echo "$(CYAN)🧪 Running mixed-attacker proof projection checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test post_sim_oversight_history_and_status_preserve_judged_mixed_attacker_lane_basis -- --nocapture

test-rsi-game-mixed-restriction-score-spine: ## Run focused mixed-attacker restriction score-spine checks
	@echo "$(CYAN)🧪 Running mixed-attacker restriction score-spine checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::benchmark_suite::tests::benchmark_suite_v1_exposes_small_machine_first_family_registry -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_materialize_mixed_attacker_restriction_family_from_scrapling_and_llm_receipts -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_block_tuning_when_exploit_progress_evidence_is_low_confidence -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_mark_exploit_progress_evidence_high_confidence_when_reproduced_and_localized -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_marks_mixed_attacker_restriction_gap_as_code_evolution_only_without_repair_families -- --exact --nocapture
	@cargo test observability::benchmark_results_comparison::tests::escalation_hint_uses_mixed_attacker_restriction_loci_for_localized_config_tuning -- --exact --nocapture
	@cargo test observability::benchmark_urgency::tests::urgency_raises_confidence_when_restriction_is_ready_and_surface_native -- --exact --nocapture
	@cargo test observability::benchmark_urgency::tests::urgency_uses_suspicious_origin_cost_as_abuse_backstop -- --exact --nocapture
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_names_rules_evaluator_moves_gates_and_anchors -- --exact --nocapture
	@cargo test observability::operator_snapshot_objectives::tests::recursive_improvement_game_contract_partitions_metric_ids_without_collapsing_to_scalar -- --exact --nocapture
	@cargo test admin::oversight_patch_policy::tests::exploit_progress_policy_preserves_localized_candidate_order -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::observe_longer_when_exploit_progress_evidence_is_low_confidence -- --exact --nocapture
	@cargo test admin::oversight_reconcile::tests::reconcile_surfaces_selected_move_lineage_for_localized_gap -- --exact --nocapture

test-rsi-game-human-only-strict: ## Run focused strict human-only stance proof checks for the current machine-first loop
	@echo "$(CYAN)🧪 Running strict human-only loop stance proof checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test post_sim_oversight_route_archives_strict_human_only_profile_context -- --nocapture
	@cargo test runtime::non_human_policy::tests::strict_human_only_profiles_suppress_verified_identity_overrides -- --exact --nocapture
	@cargo test bot_identity::policy::tests::resolve_identity_policy_strict_profiles_suppress_named_overrides -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_projects_strict_human_only_budgets_from_adversary_sim_scope -- --exact --nocapture
	@cargo test observability::benchmark_results::tests::benchmark_results_materialize_strict_human_only_suspicious_origin_metrics_from_adversary_sim -- --exact --nocapture
	@$(MAKE) --no-print-directory test-adversary-sim-runtime-surface-unit
	@$(MAKE) --no-print-directory test-adversary-sim-runtime-surface

test-rsi-game-human-only-proof: ## Run focused repeated strict human-only loop proof checks with measured retained improvement
	@echo "$(CYAN)🧪 Running repeated strict human-only loop proof checks...$(NC)"
	@$(MAKE) --no-print-directory test-rsi-game-human-only-strict
	@./scripts/set_crate_type.sh rlib
	@cargo test execute_oversight_cycle_at_records_ten_retained_improving_cycles_toward_strict_zero_leakage -- --nocapture
	@$(MAKE) --no-print-directory test-oversight-episode-archive

test-scrapling-game-loop-mainline: ## Run the current active mainline proof bundle (attacker-faithful Scrapling plus first working game loop)
	@echo "$(CYAN)🧪 Running active Scrapling -> game-loop mainline proof bundle...$(NC)"
	@$(MAKE) --no-print-directory test-adversary-sim-scrapling-owned-surface-contract
	@$(MAKE) --no-print-directory test-adversary-sim-scrapling-malicious-request-native
	@$(MAKE) --no-print-directory test-adversary-sim-scrapling-coverage-receipts
	@$(MAKE) --no-print-directory test-rsi-game-mainline

test-oversight-apply: ## Run focused closed-loop oversight canary apply and rollback checks
	@echo "$(CYAN)🧪 Running oversight apply-loop checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test manual_reconcile_route_exposes_apply_eligibility_without_mutating_config -- --nocapture
	@cargo test agent_cycle_refuses_canary_apply_when_rollout_guardrail_is_manual_only -- --nocapture
	@cargo test agent_cycle_can_apply_one_canary_when_rollout_guardrail_is_canary_only -- --nocapture
	@cargo test agent_cycle_can_apply_one_canary_with_live_runtime_protected_evidence_even_if_replay_metadata_is_stale -- --nocapture
	@cargo test admin::oversight_agent::tests::agent_cycle_uses_runtime_dev_effective_watch_window_override_for_canary_apply -- --exact --nocapture
	@cargo test agent_cycle_reports_watch_window_open_before_candidate_window_ends -- --nocapture
	@cargo test agent_cycle_rolls_back_canary_when_candidate_window_regresses -- --nocapture
	@cargo test agent_cycle_keeps_canary_when_candidate_window_improves -- --nocapture
	@cargo test admin::api::tests::operator_snapshot_recent_changes_ledger_tracks_changed_config_families -- --exact --nocapture

test-oversight-post-sim-trigger: ## Run focused post-sim oversight agent trigger and wrapper checks
	@echo "$(CYAN)🧪 Running oversight post-sim trigger checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test post_sim_trigger_accepts_generation_evidence_from_previous_running_state -- --nocapture
	@cargo test post_sim_agent_cycle_accepts_persisted_event_evidence_when_terminal_state_is_zeroed -- --nocapture
	@cargo test adversary_sim_completion_triggers_post_sim_oversight_agent_once -- --nocapture
	@python3 -m unittest scripts/tests/test_oversight_supervisor.py

test-telemetry-hot-read-evidence: ## Run focused telemetry hot-read live-evidence tooling checks
	@echo "$(CYAN)🧪 Running telemetry hot-read evidence checks...$(NC)"
	@python3 -m unittest scripts/tests/test_telemetry_shared_host_evidence.py
	@python3 -m unittest scripts/tests/test_telemetry_fermyon_edge_evidence.py

telemetry-shared-host-evidence: ## Capture live shared-host telemetry storage/query evidence for the active ssh_systemd remote
	@python3 ./scripts/tests/telemetry_shared_host_evidence.py --env-file "$(ENV_LOCAL)" --receipts-dir "$(REMOTE_RECEIPTS_DIR)" --report-path "$(TELEMETRY_SHARED_HOST_EVIDENCE_REPORT)" $(REMOTE_NAME_ARG)

telemetry-fermyon-edge-evidence: ## Capture live Fermyon edge telemetry-read evidence for the current edge deploy receipt
	@python3 ./scripts/tests/telemetry_fermyon_edge_evidence.py --env-file "$(ENV_LOCAL)" --receipt-path "$(FERMYON_AKAMAI_DEPLOY_RECEIPT)" --report-path "$(TELEMETRY_FERMYON_EDGE_EVIDENCE_REPORT)"

test-telemetry-hot-read-live-evidence: ## Capture and validate live telemetry hot-read budgets on shared-host and Fermyon deploys
	@$(MAKE) --no-print-directory telemetry-shared-host-evidence
	@$(MAKE) --no-print-directory telemetry-fermyon-edge-evidence

test-adversarial-manifest: ## Validate adversarial simulation manifest and fixtures (no server required)
	@echo "$(CYAN)🧪 Validating adversarial simulation manifest...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-python-unit
	@$(MAKE) --no-print-directory test-adversarial-preflight
	@$(MAKE) --no-print-directory test-adversarial-lane-contract
	@$(MAKE) --no-print-directory test-adversarial-sim-tag-contract
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract
	@$(MAKE) --no-print-directory test-adversarial-scenario-review
	@$(MAKE) --no-print-directory test-sim2-realtime-bench
	@$(MAKE) --no-print-directory test-sim2-adr-conformance
	@$(MAKE) --no-print-directory test-sim2-ci-diagnostics
	@$(MAKE) --no-print-directory test-sim2-verification-matrix-advisory
	@$(MAKE) --no-print-directory test-sim2-operational-regressions
	@$(MAKE) --no-print-directory test-sim2-governance-contract
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile fast_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile abuse_regression --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile akamai_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile full_coverage --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile abuse_regression --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile akamai_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile full_coverage --validate-only

test-adversarial-python-unit: ## Run adversarial python/js unit and syntax checks (no server required)
	@echo "$(CYAN)🧪 Running adversarial python/js unit checks...$(NC)"
	@python3 -m py_compile scripts/tests/adversarial_simulation_runner.py scripts/tests/adversary_runtime_toggle_surface_gate.py scripts/tests/adversarial_preflight.py scripts/tests/adversarial_live_loop.py scripts/tests/adversarial_repeatability.py scripts/tests/adversarial_promote_candidates.py scripts/tests/adversarial_report_diff.py scripts/tests/adversarial_container_runner.py scripts/tests/adversarial_container/worker.py scripts/tests/frontier_action_contract.py scripts/tests/frontier_capability_envelope.py scripts/tests/frontier_lane_attempt.py scripts/tests/frontier_unavailability_policy.py scripts/tests/check_frontier_payload_artifacts.py scripts/tests/check_adversarial_lane_contract.py scripts/tests/check_shared_host_scope_contract.py scripts/tests/check_shared_host_seed_contract.py scripts/tests/check_adversarial_sim_tag_contract.py scripts/tests/check_adversarial_coverage_contract.py scripts/tests/check_adversarial_scenario_intent_matrix.py scripts/tests/live_feedback_loop_remote.py scripts/tests/playwright_runtime.py scripts/tests/shared_host_scope.py scripts/tests/shared_host_seed_inventory.py scripts/tests/sim_tag_helpers.py scripts/tests/sim2_realtime_bench.py scripts/tests/check_sim2_adr_conformance.py scripts/tests/render_sim2_ci_diagnostics.py scripts/tests/check_sim2_verification_matrix.py scripts/tests/check_sim2_operational_regressions.py scripts/tests/check_sim2_governance_contract.py scripts/supervisor/scrapling_worker.py scripts/supervisor/llm_runtime_worker.py scripts/deploy/scrapling_deploy_prep.py scripts/prepare_scrapling_deploy.py scripts/tests/adversarial_runner/llm_fulfillment.py
	@node --check scripts/tests/adversarial_browser_driver.mjs
	@node scripts/tests/test_adversarial_browser_driver.mjs
	@python3 -m unittest scripts/tests/test_adversary_runtime_toggle_surface_gate.py scripts/tests/test_adversarial_simulation_runner.py scripts/tests/test_adversarial_preflight.py scripts/tests/test_adversarial_live_loop.py scripts/tests/test_adversarial_repeatability.py scripts/tests/test_adversarial_promote_candidates.py scripts/tests/test_adversarial_report_diff.py scripts/tests/test_adversarial_container_runner.py scripts/tests/test_adversarial_container_worker.py scripts/tests/test_frontier_action_contract.py scripts/tests/test_frontier_capability_envelope.py scripts/tests/test_frontier_lane_and_governance.py scripts/tests/test_adversarial_lane_contract.py scripts/tests/test_live_feedback_loop_remote.py scripts/tests/test_shared_host_scope.py scripts/tests/test_shared_host_seed_inventory.py scripts/tests/test_scrapling_deploy_prep.py scripts/tests/test_adversarial_sim_tag_contract.py scripts/tests/test_adversarial_coverage_contract.py scripts/tests/test_adversarial_scenario_intent_matrix.py scripts/tests/test_playwright_runtime.py scripts/tests/test_sim2_realtime_bench.py scripts/tests/test_sim2_adr_conformance.py scripts/tests/test_sim2_ci_diagnostics.py scripts/tests/test_sim2_verification_matrix.py scripts/tests/test_sim2_operational_regressions.py scripts/tests/test_sim2_governance_contract.py scripts/tests/test_llm_fulfillment.py scripts/tests/test_llm_runtime_worker.py
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts/tests/test_scrapling_worker.py

test-shared-host-scope-contract: ## Validate shared-host scope contract parity and fail-closed tooling behavior
	@echo "$(CYAN)🧪 Validating shared-host scope contract...$(NC)"
	@python3 scripts/tests/check_shared_host_scope_contract.py
	@python3 -m unittest scripts/tests/test_shared_host_scope.py

test-shared-host-seed-contract: ## Validate shared-host seed contract parity and minimal seed inventory behavior
	@echo "$(CYAN)🧪 Validating shared-host seed contract...$(NC)"
	@python3 scripts/tests/check_shared_host_seed_contract.py
	@python3 -m unittest scripts/tests/test_shared_host_seed_inventory.py

build-shared-host-seed-inventory: ## Build minimal shared-host seed inventory from operator inputs under the shared-host scope contract
	@python3 scripts/tests/shared_host_seed_inventory.py $(SHARED_HOST_SEED_ARGS)

prepare-scrapling-deploy: ## Build deploy-time Scrapling scope/seed/runtime receipt from deploy context
	@python3 ./scripts/prepare_scrapling_deploy.py $(PREPARE_SCRAPLING_ARGS)

prepare-scrapling-local: ## Build local Scrapling scope/seed/runtime artifacts for loopback-hosted dev and test runs
	@mkdir -p "$(dir $(SCRAPLING_LOCAL_RECEIPT_PATH))" "$(SCRAPLING_LOCAL_CRAWLDIR)"
	@python3 ./scripts/prepare_scrapling_deploy.py \
		--public-base-url "$(SCRAPLING_LOCAL_PUBLIC_BASE_URL)" \
		--runtime-mode ssh_systemd \
		--allow-http \
		--receipt-output "$(SCRAPLING_LOCAL_RECEIPT_PATH)" \
		--scope-output "$(SCRAPLING_LOCAL_SCOPE_PATH)" \
		--seed-output "$(SCRAPLING_LOCAL_SEED_PATH)"

test-adversarial-preflight: ## Validate adversarial required secrets and setup posture before runner execution
	@echo "$(CYAN)🧪 Running adversarial preflight checks...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/adversarial_preflight.py --output $(ADVERSARIAL_PREFLIGHT_REPORT_PATH)

test-testing-surface-artifact-path-contract: ## Validate routine adversarial/SIM2 make targets write generated artifacts under .spin state
	@echo "$(CYAN)🧪 Validating testing-surface artifact-path contract...$(NC)"
	@python3 -m unittest scripts/tests/test_testing_surface_artifact_paths.py

test-make-selector-contract-targets: ## Validate feature selector microtests live only in explicit make-target contract lanes
	@echo "$(CYAN)🧪 Validating make-selector contract target split...$(NC)"
	@$(MAKE) --no-print-directory test-adversary-sim-make-target-contract
	@$(MAKE) --no-print-directory test-verified-identity-make-target-contract
	@$(MAKE) --no-print-directory test-host-impact-make-target-contract
	@python3 -m unittest scripts/tests/test_make_selector_contract_targets.py

test-adversarial-lane-contract: ## Validate black-box lane capability contract parity across deterministic/container tooling
	@echo "$(CYAN)🧪 Validating adversarial lane capability contract...$(NC)"
	@python3 scripts/tests/check_adversarial_lane_contract.py
	@python3 -m unittest scripts/tests/test_adversarial_lane_contract.py

test-adversarial-runner-architecture: ## Run focused adversarial runner CLI, unit, and validate-only contract checks
	@echo "$(CYAN)🧪 Running adversarial runner architecture checks...$(NC)"
	@if [ -d "scripts/tests/adversarial_runner" ]; then python3 -m compileall scripts/tests/adversarial_runner >/dev/null; fi
	@python3 -m py_compile scripts/tests/adversarial_simulation_runner.py scripts/tests/adversarial_promote_candidates.py scripts/tests/adversarial_report_diff.py
	@python3 -m unittest scripts/tests/test_adversarial_simulation_runner.py scripts/tests/test_adversarial_promote_candidates.py scripts/tests/test_adversarial_report_diff.py scripts/tests/test_llm_fulfillment.py
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile abuse_regression --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile akamai_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile full_coverage --validate-only

test-adversarial-deterministic-corpus: ## Validate shared deterministic attack corpus parity across runtime and CI oracle lanes
	@echo "$(CYAN)🧪 Validating shared deterministic attack corpus parity...$(NC)"
	@python3 scripts/tests/check_adversarial_deterministic_corpus.py

test-adversary-sim-lifecycle: ## Fast adversary-sim lifecycle regression gate (toggle/state/heartbeat contracts)
	@echo "$(CYAN)🧪 Running adversary-sim lifecycle regression gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test runtime_adversary_sim_enablement_uses_persisted_seeded_state_once_config_exists -- --nocapture
	@cargo test status_payload_surfaces_explicit_production_operating_envelope -- --nocapture
	@cargo test effective_desired_enabled_prefers_persisted_lifecycle_state_after_control_write -- --nocapture
	@cargo test admin_config_runtime_projects_adversary_sim_control_state_across_runtime_cache_reset -- --nocapture
	@cargo test adversary_sim_control_start_stop_and_status_round_trip -- --nocapture
	@cargo test adversary_sim_control_enable_recovers_from_stale_expired_running_state -- --nocapture
	@cargo test adversary_sim_status_reports_reconciliation_required_for_stale_running_state_when_disabled -- --nocapture
	@cargo test adversary_sim_status_reports_previous_process_ownership_without_mutating -- --nocapture
	@cargo test adversary_sim_status_reports_auto_window_expiry_without_second_enabled_authority -- --nocapture
	@cargo test autonomous_supervisor_runs_initial_tick_when_running_without_history -- --nocapture
	@cargo test adversary_sim_internal_beat_updates_generation_diagnostics_contract -- --nocapture
	@python3 -m unittest scripts/tests/test_adversary_sim_supervisor.py
	@$(MAKE) --no-print-directory test-adversarial-deterministic-corpus

test-adversary-sim-make-target-contract: ## Run adversary-sim make-target selector contract checks
	@echo "$(CYAN)🧪 Running adversary-sim make-target contract checks...$(NC)"
	@python3 -m unittest scripts/tests/test_adversary_sim_make_targets.py

test-adversary-sim-lane-contract: ## Focused backend lane-migration contract gate (additive desired/active lane scaffolding)
	@echo "$(CYAN)🧪 Running adversary-sim lane-contract gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test start_and_stop_transitions_track_additive_lane_contract -- --nocapture
	@cargo test status_payload_exposes_additive_lane_migration_contract -- --nocapture
	@cargo test adversary_sim_control_status_exposes_additive_lane_migration_contract -- --nocapture

test-adversary-sim-lane-selection: ## Focused control-path lane-selection gate (validation, persistence, idempotency, desired-vs-active semantics)
	@echo "$(CYAN)🧪 Running adversary-sim lane-selection gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test canonical_payload_hash_changes_when_payload_changes -- --nocapture
	@cargo test adversary_sim_control_accepts_lane_selection_while_off_and_persists_desired_lane -- --nocapture
	@cargo test adversary_sim_control_rejects_invalid_lane_value -- --nocapture
	@cargo test adversary_sim_control_rejects_lane_only_idempotency_payload_mismatch -- --nocapture
	@cargo test adversary_sim_running_lane_selection_updates_desired_lane_without_switching_active_lane -- --nocapture

test-adversary-sim-domain-contract: ## Run focused adversary-sim lifecycle and lane-domain checks without live runtime-surface traffic
	@echo "$(CYAN)🧪 Running adversary-sim domain-contract checks...$(NC)"
	@$(MAKE) --no-print-directory test-adversary-sim-lifecycle
	@$(MAKE) --no-print-directory test-adversary-sim-lane-contract
	@$(MAKE) --no-print-directory test-adversary-sim-lane-selection
	@$(MAKE) --no-print-directory test-adversary-sim-diagnostics-truth

test-adversary-sim-scrapling-owned-surface-contract: ## Focused Scrapling owned-surface matrix and success-contract gate
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling owned-surface contract gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::scrapling_owned_surface::tests:: -- --nocapture

test-adversary-sim-scrapling-category-fit: ## Focused Scrapling category-fit gate (lane ownership contract plus bounded worker-plan targets)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling category-fit gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::non_human_lane_fulfillment::tests:: -- --nocapture
	@cargo test admin::adversary_sim_lane_runtime::tests::scrapling_fulfillment_modes_cycle_across_full_spectrum_personas -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane -- --exact --nocapture

test-adversary-sim-scrapling-browser-capability: ## Focused Scrapling browser-capability gate (browser persona worker execution plus browser owned-surface receipts)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling browser-capability gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::non_human_lane_fulfillment::tests:: -- --nocapture
	@cargo test observability::scrapling_owned_surface::tests:: -- --nocapture
	@cargo test admin::api::tests::recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories -- --exact --nocapture
	@cargo test observability::operator_snapshot_non_human::tests::non_human_snapshot_summary_projects_scrapling_full_spectrum_coverage -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_projects_recent_run_owned_surface_coverage -- --exact --nocapture
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_browser_automation_attempts_browser_owned_surfaces
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_stealth_browser_attempts_browser_owned_surfaces

test-adversary-sim-scrapling-proxy-capability: ## Focused Scrapling proxy-capability gate (beat-plan proxy plumbing plus worker proxy kwargs contracts)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling proxy-capability gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::admin_config_tests::adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane -- --exact --nocapture
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_request_native_session_kwargs_accept_optional_proxy_contract
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_browser_session_kwargs_accept_optional_proxy_contract

test-adversary-sim-scrapling-malicious-request-native: ## Focused Scrapling malicious request-native gate (owned route contract plus malicious bulk/http persona submits)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling malicious request-native gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test admin::api::admin_config_tests::adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane -- --exact --nocapture
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_http_agent_attempts_owned_request_native_abuse_surfaces

test-adversary-sim-scrapling-coverage-receipts: ## Focused Scrapling owned-surface receipt gate (worker receipts plus recent-run coverage aggregation)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling coverage-receipt gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::scrapling_owned_surface::tests:: -- --nocapture
	@cargo test admin::api::tests::recent_sim_run_history_normalizes_scrapling_profiles_and_aggregates_observed_categories -- --exact --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_projects_recent_run_owned_surface_coverage -- --exact --nocapture
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_emits_signed_real_scrapling_requests_and_blocks_out_of_scope_targets
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_bulk_scraper_attempts_owned_challenge_surfaces
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts.tests.test_scrapling_worker.ScraplingWorkerUnitTests.test_execute_worker_plan_http_agent_attempts_owned_request_native_abuse_surfaces

test-adversary-sim-scrapling-worker: ## Focused Scrapling lane worker gate (beat plan/result contract plus real worker/supervisor coverage)
	@echo "$(CYAN)🧪 Running adversary-sim Scrapling worker gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_internal_beat_returns_scrapling_worker_plan_and_switches_active_lane -- --nocapture
	@cargo test adversary_sim_worker_result_updates_scrapling_generation_and_lane_diagnostics -- --nocapture
	@cargo test scrapling_worker_results_without_surface_receipts_still_materialize_recent_run_categories -- --nocapture
	@cargo test adversary_sim_worker_result_is_rejected_after_manual_off_and_does_not_restore_running_state -- --nocapture
	@$(MAKE) --no-print-directory test-adversary-sim-supervisor-unit
	@if [ ! -x "$(SCRAPLING_VENV_PYTHON)" ]; then \
		echo "$(RED)❌ Error: $(SCRAPLING_VENV_PYTHON) not found.$(NC)"; \
		echo "$(YELLOW)   Run make setup or make setup-runtime to provision the repo-owned Scrapling worker runtime.$(NC)"; \
		exit 1; \
	fi
	@$(SCRAPLING_VENV_PYTHON) -m unittest scripts/tests/test_scrapling_worker.py
	@python3 -m unittest scripts/tests/test_adversary_sim_supervisor.py

test-adversary-sim-diagnostics-truth: ## Focused adversary-sim status truth gate (persisted sim event evidence must recover stale status counters)
	@echo "$(CYAN)🧪 Running adversary-sim diagnostics-truth gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_status_recovers_generation_truth_from_persisted_sim_event_evidence -- --nocapture

test-adversarial-llm-fit: ## Focused bounded LLM fulfillment-plan gate (browser/request modes + backend contract)
	@echo "$(CYAN)🧪 Running bounded adversarial LLM fulfillment gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test llm_fulfillment_modes_alternate_between_browser_and_request_contracts -- --nocapture
	@cargo test llm_fulfillment_plan_uses_frontier_reference_when_provider_keys_exist -- --nocapture
	@cargo test llm_fulfillment_plan_reports_unavailable_frontier_backend_without_provider_keys -- --nocapture
	@cargo test adversary_sim_internal_beat_returns_llm_fulfillment_plan_for_bot_red_team_lane -- --nocapture
	@python3 -m unittest scripts/tests/test_llm_fulfillment.py

test-adversarial-llm-runtime-dispatch: ## Focused bounded LLM runtime dispatch gate (supervisor worker dispatch + typed result ingest)
	@echo "$(CYAN)🧪 Running bounded adversarial LLM runtime dispatch gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_worker_result_updates_llm_runtime_generation_and_lane_diagnostics -- --nocapture
	@$(MAKE) --no-print-directory test-adversary-sim-supervisor-unit
	@python3 -m py_compile scripts/supervisor/llm_runtime_worker.py
	@python3 -m unittest scripts/tests/test_adversary_sim_supervisor.py scripts/tests/test_llm_runtime_worker.py

test-adversarial-llm-runtime-projection: ## Focused bounded LLM runtime recent-run and operator projection gate
	@echo "$(CYAN)🧪 Running bounded adversarial LLM runtime projection gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test recent_sim_run_history_projects_llm_runtime_receipts_and_categories -- --nocapture
	@cargo test snapshot_payload_projects_recent_run_llm_runtime_summary -- --nocapture
	@$(MAKE) --no-print-directory test-dashboard-red-team-truth-basis

adversary-sim-supervisor-build: ## Build the host-side adversary-sim supervisor worker binary
	@./scripts/adversary_sim_supervisor_launch.sh --build-only

adversary-sim-supervisor: adversary-sim-supervisor-build ## Run host-side adversary-sim supervisor loop (watch mode)
	@SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) \
		./scripts/adversary_sim_supervisor_launch.sh --watch --base-url $(ADVERSARY_SIM_SUPERVISOR_BASE_URL)

test-adversary-sim-supervisor-unit: ## Focused host-side adversary-sim supervisor transport/parser unit checks
	@echo "$(CYAN)🧪 Running adversary-sim supervisor unit checks...$(NC)"
	@mkdir -p target/tools
	@rustc --edition=2021 --test scripts/supervisor/adversary_sim_supervisor.rs -o target/tools/adversary_sim_supervisor_tests
	@target/tools/adversary_sim_supervisor_tests --nocapture

test-adversary-sim-runtime-surface: ## Runtime-toggle integration gate for deterministic defense-surface coverage plus live-summary no-impact proof (requires running server)
	@echo "$(CYAN)🧪 Running runtime-toggle adversary-sim surface/no-impact gate...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversary_runtime_toggle_surface_gate.py; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversary-sim-runtime-surface-unit: ## Focused unit checks for the runtime-toggle live-summary no-impact gate
	@echo "$(CYAN)🧪 Running runtime-toggle adversary-sim gate unit checks...$(NC)"
	@python3 -m unittest scripts/tests/test_adversary_runtime_toggle_surface_gate.py

test-adversarial-sim-tag-contract: ## Validate simulation tag signing contract parity across runtime/tooling
	@echo "$(CYAN)🧪 Validating adversarial sim-tag contract...$(NC)"
	@python3 scripts/tests/check_adversarial_sim_tag_contract.py

test-adversarial-coverage-contract: ## Validate full-coverage contract parity across plan, manifest, and runner
	@echo "$(CYAN)🧪 Validating adversarial coverage contract...$(NC)"
	@python3 scripts/tests/check_adversarial_coverage_contract.py
	@python3 -m unittest scripts/tests/test_adversarial_coverage_contract.py

test-adversarial-scenario-review: ## Validate scenario intent matrix parity and review freshness governance
	@echo "$(CYAN)🧪 Validating adversarial scenario intent matrix...$(NC)"
	@python3 scripts/tests/check_adversarial_scenario_intent_matrix.py

test-adversarial-scenario-intent-evidence-unit: ## Focused adversarial runner evidence checks for scenario-intent fallback coverage mapping
	@echo "$(CYAN)🧪 Running adversarial scenario-intent evidence unit checks...$(NC)"
	@python3 -m unittest \
		scripts.tests.test_adversarial_simulation_runner.AdversarialRunnerUnitTests.test_derive_coverage_deltas_from_simulation_event_reasons_maps_geo_policy_actions \
		scripts.tests.test_adversarial_simulation_runner.AdversarialRunnerUnitTests.test_derive_coverage_deltas_from_simulation_event_reasons_maps_rate_enforcement \
		scripts.tests.test_adversarial_simulation_runner.AdversarialRunnerUnitTests.test_build_scenario_execution_evidence_supplements_geo_coverage_from_sim_reason_delta \
		scripts.tests.test_adversarial_simulation_runner.AdversarialRunnerUnitTests.test_build_scenario_execution_evidence_supplements_rate_coverage_from_sim_reason_delta

test-adversarial-sim-selftest: ## Run minimal deterministic simulator self-test harness (no server required)
	@echo "$(CYAN)🧪 Running adversarial simulator self-test harness...$(NC)"
	@python3 scripts/tests/adversarial_sim_selftest.py

test-adversarial-fast: ## Run mandatory fast adversarial matrix (smoke + abuse + Akamai profiles)
	@echo "$(CYAN)🧪 Running mandatory fast adversarial matrix...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@$(MAKE) --no-print-directory test-adversarial-lane-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-deterministic-corpus || exit 1
	@$(MAKE) --no-print-directory test-adversarial-sim-tag-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-scenario-review || exit 1
	@$(MAKE) --no-print-directory test-adversarial-sim-selftest || exit 1
	@$(MAKE) --no-print-directory test-adversarial-smoke || exit 1
	@$(MAKE) --no-print-directory test-adversarial-abuse || exit 1
	@$(MAKE) --no-print-directory test-adversarial-akamai || exit 1
	@$(MAKE) --no-print-directory test-frontier-governance || exit 1

test-adversarial-smoke: ## Run adversarial fast smoke simulation profile (requires running server)
	@echo "$(CYAN)🧪 Running adversarial fast smoke simulation...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke --report "$(ADVERSARIAL_REPORT_PATH)"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-abuse: ## Run replay/stale/ordering abuse regression profile (requires running server)
	@echo "$(CYAN)🧪 Running adversarial abuse regression profile...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile abuse_regression --report "$(ADVERSARIAL_REPORT_PATH)"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-akamai: ## Run Akamai signal fixture smoke profile (requires running server)
	@echo "$(CYAN)🧪 Running adversarial Akamai fixture profile...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile akamai_smoke --report "$(ADVERSARIAL_REPORT_PATH)"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-coverage: ## Run deterministic full-coverage oracle profile (protected-lane/release blocker; requires running server)
	@echo "$(CYAN)🧪 Running adversarial coverage profile...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@$(MAKE) --no-print-directory test-adversarial-deterministic-corpus || exit 1
	@$(MAKE) --no-print-directory test-adversarial-sim-tag-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-scenario-review || exit 1
	@$(MAKE) --no-print-directory test-adversarial-sim-selftest || exit 1
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=1 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile full_coverage --report "$(ADVERSARIAL_REPORT_PATH)" || exit 1; \
		$(MAKE) --no-print-directory test-frontier-governance || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
			exit 1; \
		fi

test-adversarial-frontier-attempt: ## Attempt frontier provider probes for protected lanes (advisory/non-blocking)
	@echo "$(CYAN)🧪 Attempting protected-lane frontier provider probes (advisory)...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/frontier_lane_attempt.py --output $(FRONTIER_LANE_STATUS_PATH)

test-frontier-governance: ## Fail when forbidden frontier fields or secret values appear in report artifacts
	@echo "$(CYAN)🧪 Verifying frontier artifact governance guardrails...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_frontier_payload_artifacts.py --report $(ADVERSARIAL_REPORT_PATH) --attack-plan $(ADVERSARIAL_ATTACK_PLAN_PATH) --schema scripts/tests/adversarial/frontier_payload_schema.v1.json

test-frontier-unavailability-policy: ## Evaluate frontier degraded-threshold policy and emit actionability artifact
	@echo "$(CYAN)🧪 Evaluating frontier unavailability policy thresholds...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@ARGS=""; \
		if [ "$${FRONTIER_POLICY_ENABLE_GITHUB:-0}" = "1" ]; then \
			ARGS="--enable-github"; \
		fi; \
		python3 scripts/tests/frontier_unavailability_policy.py --status $(FRONTIER_LANE_STATUS_PATH) --output $(FRONTIER_UNAVAILABILITY_POLICY_PATH) $$ARGS

test-frontier-unavailability-policy-unit: ## Run focused frontier unavailability policy unit coverage (no server required)
	@echo "$(CYAN)🧪 Running frontier unavailability policy unit tests...$(NC)"
	@python3 -m unittest scripts/tests/test_frontier_lane_and_governance.py

test-sim2-realtime-bench: ## Run deterministic SIM2 realtime benchmark gate and emit latency/overflow/request-budget artifacts
	@echo "$(CYAN)🧪 Running SIM2 realtime benchmark gate...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/sim2_realtime_bench.py --output $(SIM2_REALTIME_BENCH_REPORT_PATH) --summary $(SIM2_REALTIME_BENCH_SUMMARY_PATH)

test-sim2-adr-conformance: ## Verify SIM2 ADR conformance markers for ADR 0007/0008/0009 domains
	@echo "$(CYAN)🧪 Running SIM2 ADR conformance checks...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_adr_conformance.py --output $(SIM2_ADR_CONFORMANCE_REPORT_PATH)

test-sim2-ci-diagnostics: ## Render SIM2 CI diagnostics artifact (timeline snapshots, event counts, refresh traces)
	@echo "$(CYAN)🧪 Rendering SIM2 CI diagnostics artifact...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/render_sim2_ci_diagnostics.py --report $(ADVERSARIAL_REPORT_PATH) --output $(SIM2_CI_DIAGNOSTICS_REPORT_PATH)

test-sim2-verification-matrix: ## Validate SIM2 verification matrix rows and evidence diagnostics
	@echo "$(CYAN)🧪 Validating SIM2 verification matrix...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report $(ADVERSARIAL_REPORT_PATH) --container-report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH) --output $(SIM2_VERIFICATION_MATRIX_REPORT_PATH)

test-sim2-verification-matrix-advisory: ## Validate SIM2 verification matrix rows (advisory mode allows missing container report for local manifest checks)
	@echo "$(CYAN)🧪 Validating SIM2 verification matrix (advisory)...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report $(ADVERSARIAL_REPORT_PATH) --container-report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH) --output $(SIM2_VERIFICATION_MATRIX_REPORT_PATH) --allow-missing-container-report --allow-missing-report-scenarios

test-sim2-operational-regressions: ## Validate SIM2 operational regression diagnostics for active deterministic profiles (retention/cost/security required; failure/prod evaluated when present)
	@echo "$(CYAN)🧪 Validating SIM2 operational regression diagnostics...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_operational_regressions.py --report $(ADVERSARIAL_REPORT_PATH) --output $(SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH) --allow-missing-domain failure_injection --allow-missing-domain prod_mode_monitoring --min-large-payload-samples-for-compression-check 2

test-sim2-operational-regressions-strict: ## Validate all SIM2 operational regression domains with strict missing-domain and compression enforcement
	@echo "$(CYAN)🧪 Validating strict SIM2 operational regression diagnostics...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_operational_regressions.py --report $(ADVERSARIAL_REPORT_PATH) --output $(SIM2_OPERATIONAL_REGRESSIONS_REPORT_PATH)

test-sim2-governance-contract: ## Validate SIM2 governance + hybrid lane contract markers and thresholds
	@echo "$(CYAN)🧪 Validating SIM2 governance + hybrid lane contract...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/check_sim2_governance_contract.py --contract scripts/tests/adversarial/hybrid_lane_contract.v1.json --promotion-script scripts/tests/adversarial_promote_candidates.py --operator-guide docs/adversarial-operator-guide.md --output $(SIM2_GOVERNANCE_CONTRACT_REPORT_PATH)

test-replay-promotion-contract: ## Run focused replay-promotion lineage and governance contract checks
	@echo "$(CYAN)🧪 Running replay-promotion contract checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test observability::replay_promotion::tests:: -- --nocapture
	@cargo test admin::replay_promotion_api::tests:: -- --nocapture
	@cargo test observability::operator_snapshot::tests::snapshot_payload_surfaces_materialized_replay_promotion_summary -- --exact --nocapture
	@python3 -m unittest scripts/tests/test_adversarial_promote_candidates.py
	@$(MAKE) --no-print-directory test-sim2-governance-contract

test-sim2-verification-e2e: ## Run matrix-required SIM2 e2e suite across crawler/scraper/browser/frontier lanes (requires running server + Docker)
	@echo "$(CYAN)🧪 Running SIM2 verification e2e suite...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
		$(MAKE) --no-print-directory test-adversarial-coverage || exit 1; \
		$(MAKE) --no-print-directory test-adversarial-container-blackbox || exit 1; \
		python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report $(ADVERSARIAL_REPORT_PATH) --container-report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH) --output $(SIM2_VERIFICATION_MATRIX_REPORT_PATH) || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-soak: ## Run deep adversarial soak gate (full_coverage profile; intended for scheduled/manual CI)
	@echo "$(CYAN)🧪 Running deep adversarial soak profile...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-coverage

test-adversarial-live: ## Continuously run adversarial simulation profile for live operator monitoring (requires running server)
	@echo "$(CYAN)🧪 Running adversarial live simulation loop...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@PROFILE="$${ADVERSARIAL_PROFILE:-fast_smoke}"; \
	RUNS="$${ADVERSARIAL_RUNS:-0}"; \
	PAUSE="$${ADVERSARIAL_PAUSE_SECONDS:-2}"; \
	REPORT_PATH="$${ADVERSARIAL_REPORT_PATH:-$(ADVERSARIAL_REPORT_PATH)}"; \
	CLEANUP_MODE="$${ADVERSARIAL_CLEANUP_MODE:-0}"; \
	case "$$RUNS" in ''|*[!0-9]*) \
		echo "$(RED)❌ ADVERSARIAL_RUNS must be an integer (0 means run until Ctrl+C).$(NC)"; \
		exit 1 ;; \
	esac; \
	case "$$PAUSE" in ''|*[!0-9]*) \
		echo "$(RED)❌ ADVERSARIAL_PAUSE_SECONDS must be an integer >= 0.$(NC)"; \
		exit 1 ;; \
	esac; \
	case "$$CLEANUP_MODE" in 0|1) ;; \
	*) \
		echo "$(RED)❌ ADVERSARIAL_CLEANUP_MODE must be 0 (preserve state) or 1 (cleanup each cycle).$(NC)"; \
		exit 1 ;; \
	esac; \
	echo "$(YELLOW)   profile=$$PROFILE runs=$$RUNS pause_seconds=$$PAUSE cleanup_mode=$$CLEANUP_MODE report=$$REPORT_PATH$(NC)"; \
	echo "$(YELLOW)   Press Ctrl+C to stop.$(NC)"; \
	if $(MAKE) --no-print-directory spin-wait-ready; then \
		mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_live_loop.py \
				--manifest scripts/tests/adversarial/scenario_manifest.v2.json \
				--profile "$$PROFILE" \
				--runs "$$RUNS" \
				--pause-seconds "$$PAUSE" \
				--report "$$REPORT_PATH" \
				--cleanup-mode "$$CLEANUP_MODE" \
				--fatal-cycle-limit "$${ADVERSARIAL_FATAL_CYCLE_LIMIT:-3}" \
				--transient-retry-limit "$${ADVERSARIAL_TRANSIENT_RETRY_LIMIT:-4}" \
				--backoff-base-seconds "$${ADVERSARIAL_BACKOFF_BASE_SECONDS:-2}" \
				--backoff-max-seconds "$${ADVERSARIAL_BACKOFF_MAX_SECONDS:-30}" \
				--preserve-state "$${ADVERSARIAL_PRESERVE_STATE:-1}" \
				--rotate-ips "$${ADVERSARIAL_ROTATE_IPS:-1}" || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

telemetry-clean: ## Clear retained telemetry history from admin monitoring/event surfaces (shared local keyspace; destructive; requires running server)
	@echo "$(RED)🚨 Clearing retained telemetry history (shared local keyspace; destructive).$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		RESPONSE="$$(curl -fsS -X POST \
			-H "Authorization: Bearer $(SHUMA_API_KEY)" \
			-H "X-Shuma-Telemetry-Cleanup-Ack: I_UNDERSTAND_TELEMETRY_CLEANUP" \
			-H "X-Forwarded-For: 127.0.0.1" \
			$(FORWARDED_SECRET_HEADER) \
			http://127.0.0.1:3000/admin/adversary-sim/history/cleanup)" || { \
			echo "$(RED)❌ Failed to clear retained telemetry history.$(NC)"; \
			echo "$(YELLOW)   Endpoint requires admin write access; in runtime-prod it also requires explicit cleanup acknowledgement (sent by this target).$(NC)"; \
			exit 1; \
		}; \
		echo "$(GREEN)✅ Retained telemetry history cleared.$(NC)"; \
		echo "$$RESPONSE"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-repeatability: ## Run deterministic repeatability gate across smoke/abuse/coverage profiles (N=3)
	@echo "$(CYAN)🧪 Running adversarial repeatability gate...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_repeatability.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --repeats "$${ADVERSARIAL_REPEATABILITY_REPEATS:-3}" --profiles "$${ADVERSARIAL_REPEATABILITY_PROFILES:-fast_smoke,abuse_regression,full_coverage}" --report $(ADVERSARIAL_REPEATABILITY_REPORT_PATH); \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-promote-candidates: ## Run frontier candidate triage + deterministic replay promotion checks (requires running server)
	@echo "$(CYAN)🧪 Running adversarial candidate triage and promotion checks...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
		REPORT_PATH="$(ADVERSARIAL_REPORT_PATH)"; \
		ATTACK_PLAN_PATH="$(ADVERSARIAL_ATTACK_PLAN_PATH)"; \
		if [ ! -f "$$REPORT_PATH" ] || [ ! -f "$$ATTACK_PLAN_PATH" ]; then \
			echo "$(YELLOW)   Missing adversarial report artifacts; generating with test-adversarial-coverage...$(NC)"; \
			$(MAKE) --no-print-directory test-adversarial-coverage || exit 1; \
		fi; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_promote_candidates.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report "$$REPORT_PATH" --attack-plan "$$ATTACK_PLAN_PATH" --output $(ADVERSARIAL_PROMOTION_CANDIDATES_REPORT_PATH) --fail-on-confirmed-regressions; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-report-diff: ## Compare baseline/candidate adversarial reports and emit run-delta artifact
	@echo "$(CYAN)🧪 Rendering adversarial report diff artifact...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
	BASELINE="$${ADVERSARIAL_DIFF_BASELINE_PATH:-$(ADVERSARIAL_DIFF_BASELINE_PATH)}"; \
	CANDIDATE="$${ADVERSARIAL_DIFF_CANDIDATE_PATH:-$(ADVERSARIAL_DIFF_CANDIDATE_PATH)}"; \
	OUTPUT="$${ADVERSARIAL_DIFF_OUTPUT_PATH:-$(ADVERSARIAL_DIFF_OUTPUT_PATH)}"; \
	if [ ! -f "$$BASELINE" ]; then \
		echo "$(YELLOW)   Baseline report missing ($$BASELINE); skipping diff generation.$(NC)"; \
		exit 0; \
	fi; \
	if [ ! -f "$$CANDIDATE" ]; then \
		echo "$(YELLOW)   Candidate report missing ($$CANDIDATE); skipping diff generation.$(NC)"; \
		exit 0; \
	fi; \
	python3 scripts/tests/adversarial_report_diff.py --baseline "$$BASELINE" --candidate "$$CANDIDATE" --output "$$OUTPUT"

test-adversarial-container-isolation: ## Validate complementary container black-box isolation contract (scheduled/manual lane; Docker required)
	@echo "$(CYAN)🧪 Running adversarial container isolation conformance...$(NC)"
	@mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"
	@python3 scripts/tests/adversarial_container_runner.py --mode isolation --report $(ADVERSARIAL_CONTAINER_ISOLATION_REPORT_PATH)

test-adversarial-container-blackbox: ## Run complementary containerized black-box adversary lane (scheduled/manual; non-blocking for release)
	@echo "$(CYAN)🧪 Running adversarial container black-box worker...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		mkdir -p "$(ADVERSARIAL_ARTIFACT_DIR)"; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_container_runner.py --mode blackbox --report $(ADVERSARIAL_CONTAINER_BLACKBOX_REPORT_PATH); \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-coverage: ## Run unit test coverage (requires cargo-llvm-cov)
	@echo "$(CYAN)🧪 Running Rust unit test coverage...$(NC)"
	@if ! command -v cargo-llvm-cov >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: cargo-llvm-cov not found$(NC)"; \
		echo "$(YELLOW)   Install with: cargo install cargo-llvm-cov --locked$(NC)"; \
		exit 1; \
	fi
	@./scripts/set_crate_type.sh rlib
	@cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
	@echo "$(GREEN)✅ Coverage report written to lcov.info$(NC)"

test-dashboard: ## Dashboard testing instructions (manual)
	@echo "$(CYAN)🧪 Dashboard testing (manual):$(NC)"
	@echo "1. Ensure Spin is running: make dev"
	@echo "2. Open: http://127.0.0.1:3000/dashboard/index.html"
	@echo "3. Follow checklist in docs/testing.md"

test-dashboard-svelte-check: ## Run Svelte static diagnostics for dashboard sources
	@echo "$(CYAN)🧪 Running dashboard svelte-check diagnostics...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@corepack pnpm run test:dashboard:svelte-check

test-dashboard-unit: ## Run dashboard module unit tests (Node + dashboard JS contracts)
	@echo "$(CYAN)🧪 Running dashboard module unit tests...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@corepack pnpm run test:dashboard:unit

test-dashboard-config-surface-contract: ## Run focused dashboard config-surface parity tests
	@echo "$(CYAN)🧪 Running focused dashboard config-surface contract checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@node --test \
		--test-name-pattern='config form utils and JSON object helpers preserve parser contracts|advanced config template paths match writable admin config patch keys|runtime variable inventory meanings match writable and read-only admin config payload paths' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-adversary-sim-lane-contract: ## Run focused dashboard lane-contract checks for adversary-sim controls
	@echo "$(CYAN)🧪 Running focused dashboard adversary-sim lane-contract checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard API client preserves adversary-sim lane status and diagnostics fields|dashboard API client sends optional adversary-sim lane selection in control writes|dashboard adversary-sim runtime normalizes orchestration status|dashboard red team controller can replace backend status after lane-only control writes|red team auto-refresh rehydrates missing config runtime write truth for lane controls|red team tab reuses verification-style config panel primitives for its adversary sim pane|dashboard route lazily loads heavy tabs and keeps orchestration local' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-auth-gate: ## Run focused dashboard auth-gate checks for logged-out dashboard entry
	@echo "$(CYAN)🧪 Running focused dashboard auth-gate checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard route keeps a neutral auth gate mounted until session bootstrap authenticates' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "logged-out dashboard navigation keeps the auth gate visible until redirect"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-tab-information-architecture: ## Run focused dashboard tab information-architecture contract checks
	@echo "$(CYAN)🧪 Running focused dashboard tab information-architecture checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard state and store contracts remain immutable and bounded with heartbeat-owned connection telemetry|dashboard smoke spec keeps the tab information architecture aligned with the canonical registry|game loop, traffic, and diagnostics tabs make ownership boundaries explicit' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "game loop, traffic, and diagnostics tabs expose their ownership split|traffic, game loop, red team, and ip-bans share the refresh bar while diagnostics remains manual-refresh only"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-game-loop-accountability: ## Run focused dashboard Game Loop machine-contract accountability checks
	@echo "$(CYAN)🧪 Running focused dashboard Game Loop accountability checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard game loop accountability adapters normalize benchmark and oversight payloads safely|dashboard game loop policy truth stops presenting legacy verified-identity stance as the strict target|dashboard game loop accountability source distinguishes judge planes and localized move outcome|dashboard game loop accountability refresh populates machine snapshots through behavior' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory dashboard-verify-freshness || exit 1; \
		./scripts/tests/verify_served_dashboard_assets.sh http://127.0.0.1:3000 || exit 1; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "game loop projects benchmark and oversight accountability from machine-first contracts|game loop tab corroborates latest scrapling evidence readiness|game loop distinguishes category posture achievement from scrapling surface contract truth|game loop tab separates judge planes, breach loci, and config exhaustion state|game loop top casts prefer the freshest exact recent sim run over stale judged history"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-traffic-pane: ## Run focused dashboard Traffic tab ownership and refresh checks
	@echo "$(CYAN)🧪 Running focused dashboard Traffic tab checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard smoke spec keeps the tab information architecture aligned with the canonical registry|dashboard route exposes live traffic refresh controls without changing other tab semantics|game loop, traffic, and diagnostics tabs make ownership boundaries explicit' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "game loop, traffic, and diagnostics tabs expose their ownership split|traffic, game loop, red team, and ip-bans share the refresh bar while diagnostics remains manual-refresh only"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-diagnostics-pane: ## Run focused dashboard Diagnostics defense-breakdown and ownership checks
	@echo "$(CYAN)🧪 Running focused dashboard Diagnostics tab checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='game loop, traffic, and diagnostics tabs make ownership boundaries explicit|monitoring view model and status module remain pure snapshot transforms' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "diagnostics defense breakdown summarizes full defense furniture, not just recent event classes|game loop, traffic, and diagnostics tabs expose their ownership split"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-runtime-unit-contracts: ## Run focused dashboard native/refresh runtime behavior checks
	@echo "$(CYAN)🧪 Running focused dashboard runtime unit-contract checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard native runtime restores session, normalizes tabs, and invalidates config mutations through behavior|dashboard refresh runtime clears caches and resets freshness snapshots through behavior' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-ip-bans-refresh-contract: ## Run focused dashboard IP-bans refresh resilience checks
	@echo "$(CYAN)🧪 Running focused dashboard IP-bans refresh resilience checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='ip-bans refresh keeps the tab usable when range suggestions fail after delta bans succeed' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-policy-pane-ownership: ## Run focused dashboard policy/tuning pane-ownership contract checks
	@echo "$(CYAN)🧪 Running focused dashboard policy/tuning pane-ownership checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard config tabs reuse shared panels, save flows, and owned controls' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-verified-identity-pane: ## Run focused Verification-tab verified-identity surfacing checks
	@echo "$(CYAN)🧪 Running focused dashboard verified-identity pane checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard verification tab wires verified identity operator snapshot and store state|dashboard config tabs reuse shared panels, save flows, and owned controls' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "verification tab surfaces verified identity controls and health summary"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-red-team-truth-basis: ## Run focused Red Team truth-basis surfacing checks
	@echo "$(CYAN)🧪 Running focused dashboard Red Team truth-basis checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='dashboard API client preserves adversary-sim lane status and diagnostics fields|dashboard adversary-sim runtime normalizes orchestration status|red team tab renders the recent adversary runs panel with red-team-specific copy' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "red team tab surfaces recovered adversary-sim truth basis and persisted event evidence"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-scrapling-evidence: ## Run focused dashboard Scrapling evidence surfacing checks
	@echo "$(CYAN)🧪 Running focused dashboard Scrapling evidence checks...$(NC)"
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='monitoring view model and status module remain pure snapshot transforms' \
		e2e/dashboard.modules.unit.test.js
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "red team tab surfaces receipt-backed scrapling attack evidence from recent sim summaries|game loop tab corroborates latest scrapling evidence readiness"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-ban-duration-family-truth: ## Run focused ban-duration family parity checks across config, runtime, and Policy tab
	@echo "$(CYAN)🧪 Running focused ban-duration family truthfulness checks...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test config::tests::ban_duration_lookup_covers_shipped_families_and_legacy_fallback -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::admin_config_updates_lists_and_full_ban_duration_family_set -- --exact --nocapture
	@cargo test admin::api::admin_config_tests::manual_ban_uses_configured_admin_default_duration_when_duration_is_omitted -- --exact --nocapture
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@$(MAKE) --no-print-directory test-dashboard-svelte-check
	@node --test \
		--test-name-pattern='ban duration families remain aligned across runtime, config, and policy surfaces' \
		e2e/dashboard.modules.unit.test.js

test-dashboard-e2e-ban-duration-family-truth: ## Run focused Playwright Policy-tab ban-duration truth smoke
	@echo "$(CYAN)🧪 Running focused dashboard ban-duration truth smoke...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "policy tab save flows cover robots serving, durations, browser policy, and path allowlist controls"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-budgets: ## Report /dashboard/_app bundle size ceilings (non-blocking by default)
	@echo "$(CYAN)🧪 Checking dashboard bundle-size budgets...$(NC)"
	@if [ "$${SHUMA_DASHBOARD_BUDGET_SKIP_BUILD:-0}" != "1" ]; then \
		$(MAKE) --no-print-directory dashboard-build >/dev/null; \
	fi
	@SHUMA_DASHBOARD_BUNDLE_MAX_TOTAL_BYTES=$(SHUMA_DASHBOARD_BUNDLE_MAX_TOTAL_BYTES) \
	SHUMA_DASHBOARD_BUNDLE_MAX_JS_BYTES=$(SHUMA_DASHBOARD_BUNDLE_MAX_JS_BYTES) \
	SHUMA_DASHBOARD_BUNDLE_MAX_CSS_BYTES=$(SHUMA_DASHBOARD_BUNDLE_MAX_CSS_BYTES) \
	SHUMA_DASHBOARD_BUNDLE_MAX_JS_CHUNK_BYTES=$(SHUMA_DASHBOARD_BUNDLE_MAX_JS_CHUNK_BYTES) \
	SHUMA_DASHBOARD_BUNDLE_MAX_CSS_ASSET_BYTES=$(SHUMA_DASHBOARD_BUNDLE_MAX_CSS_ASSET_BYTES) \
	SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=$(SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE) \
	node scripts/tests/check_dashboard_bundle_budget.js

test-dashboard-budgets-strict: ## Enforce hard-fail dashboard bundle-size ceilings
	@SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE=1 $(MAKE) --no-print-directory test-dashboard-budgets

test-dashboard-e2e: ## Run Playwright dashboard smoke tests (waits for existing server readiness)
	@echo "$(CYAN)🧪 Running dashboard e2e smoke tests...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory test-dashboard-unit || exit 1; \
		SHUMA_DASHBOARD_BUDGET_SKIP_BUILD=1 $(MAKE) --no-print-directory test-dashboard-budgets || exit 1; \
		$(MAKE) --no-print-directory dashboard-verify-freshness || exit 1; \
		./scripts/tests/verify_served_dashboard_assets.sh http://127.0.0.1:3000 || exit 1; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh $(PLAYWRIGHT_ARGS); \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-adversary-sim: ## Run focused Playwright adversary-sim dashboard smoke checks
	@echo "$(CYAN)🧪 Running focused dashboard adversary-sim e2e checks...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "adversary sim (global toggle drives orchestration control lifecycle state|toggle emits fresh telemetry visible in monitoring raw feed|lane selector keeps off-state desired versus active truth and disables bot red team)"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-red-team-frontier-warning: ## Run focused Playwright Red Team missing-frontier continue-path smoke
	@echo "$(CYAN)🧪 Running focused Red Team missing-frontier smoke...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "adversary sim toggle continue path omits the no-frontier warning after confirmation"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-tab-information-architecture: ## Run focused Playwright dashboard tab information-architecture smoke
	@echo "$(CYAN)🧪 Running focused dashboard tab information-architecture smoke...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "tab bar reflects the canonical information architecture labels and order|tab keyboard navigation updates hash and selected state"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-policy-pane-ownership: ## Run focused Playwright policy/tuning pane-ownership smoke
	@echo "$(CYAN)🧪 Running focused dashboard policy/tuning pane-ownership smoke...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "geo and tuning save flows cover GEO lists and botness controls|policy tab save flows cover robots serving, durations, browser policy, and path allowlist controls"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-tab-state-transitions: ## Run focused Playwright dashboard tab-state transition smoke
	@echo "$(CYAN)🧪 Running focused dashboard tab-state transition smoke...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
			corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
		fi; \
		$(MAKE) --no-print-directory seed-dashboard-data || exit 1; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "tab states surface loading and data-ready transitions across all tabs"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-dashboard-e2e-external: ## Run focused live hosted-dashboard smoke against an already-hosted deployment (separate from make test)
	@echo "$(CYAN)🧪 Running external dashboard live smoke against $(SHUMA_BASE_URL)...$(NC)"
	@if [ -z "$(SHUMA_BASE_URL)" ]; then \
		echo "$(RED)❌ Error: SHUMA_BASE_URL must be set for test-dashboard-e2e-external.$(NC)"; \
		exit 1; \
	fi
	@if [ -z "$(SHUMA_API_KEY)" ]; then \
		echo "$(RED)❌ Error: SHUMA_API_KEY must be set for test-dashboard-e2e-external.$(NC)"; \
		exit 1; \
	fi
	@if [ -z "$(SHUMA_FORWARDED_IP_SECRET)" ]; then \
		echo "$(RED)❌ Error: SHUMA_FORWARDED_IP_SECRET must be set for test-dashboard-e2e-external.$(NC)"; \
		exit 1; \
	fi
	@if ! command -v corepack >/dev/null 2>&1; then \
		echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
		exit 1; \
	fi
	@corepack enable > /dev/null 2>&1 || true
	@if [ ! -d node_modules/.pnpm ] || [ ! -x node_modules/.bin/vite ] || [ ! -x node_modules/.bin/svelte-check ] || [ ! -d node_modules/svelte ] || [ ! -d node_modules/@sveltejs/kit ] || [ ! -d node_modules/@playwright/test ]; then \
		corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile; \
	fi
	@corepack pnpm exec node ./scripts/tests/dashboard_external_live_smoke.mjs $(PLAYWRIGHT_ARGS)

seed-dashboard-data: ## Seed dashboard sample records for local monitoring UI validation (requires running server)
	@echo "$(CYAN)🧪 Seeding dashboard sample data...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		if ! command -v corepack >/dev/null 2>&1; then \
			echo "$(RED)❌ Error: corepack not found (install Node.js 18+).$(NC)"; \
			exit 1; \
		fi; \
		corepack enable > /dev/null 2>&1 || true; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) node e2e/seed-dashboard-data.js; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

#--------------------------
# Utilities
#--------------------------

stop: ## Stop running Spin server
	@echo "$(CYAN)🛑 Stopping Spin server...$(NC)"
	@pkill -f "scripts/dev_watch_lock.sh cargo watch --poll -w src -w dashboard -w spin.toml" 2>/dev/null || true
	@pkill -f "cargo watch --poll -w src -w dashboard -w spin.toml" 2>/dev/null || true
	@pkill -f "cargo-watch watch --poll -w src -w dashboard -w spin.toml" 2>/dev/null || true
	@pkill -f "scripts/run_with_oversight_supervisor.sh spin up" 2>/dev/null || true
	@pkill -f "scripts/run_with_adversary_sim_supervisor.sh spin up" 2>/dev/null || true
	@pkill -f "target/tools/adversary_sim_supervisor" 2>/dev/null || true
	@rm -rf .spin/dev-watch.lock
	@pkill -x spin 2>/dev/null && echo "$(GREEN)✅ Stopped$(NC)" || echo "$(YELLOW)No server running$(NC)"

status: ## Check if Spin server is running
	@if curl -sf -H "X-Forwarded-For: 127.0.0.1" $(FORWARDED_SECRET_HEADER) $(HEALTH_SECRET_HEADER) http://127.0.0.1:3000/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Spin server is running$(NC)"; \
		echo "   Dashboard: http://127.0.0.1:3000/dashboard/index.html"; \
		echo "   Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)"; \
	else \
		echo "$(YELLOW)⚠️  Spin server is not running$(NC)"; \
	fi

clean: ## Clean build artifacts
	@echo "$(CYAN)🧹 Cleaning build artifacts...$(NC)"
	@cargo clean
	@rm -rf dist/wasm
	@rm -f .spin/dev-watch.lock
	@rm -rf playwright-report test-results
	@rm -f src/*.wasm
	@echo "$(GREEN)✅ Clean complete$(NC)"

reset-local-state: ## Destructively remove local runtime/test state under .spin while preserving durable operator state
	@echo "$(CYAN)🧨 Resetting local runtime/test state under .spin...$(NC)"
	@rm -rf .spin
	@echo "$(GREEN)✅ Local runtime/test state reset complete$(NC)"

logs: ## View Spin component logs
	@echo "$(CYAN)📜 Spin logs:$(NC)"
	@cat .spin/logs/* 2>/dev/null || echo "No logs found. Run 'make dev' first."

api-key-generate: ## Generate a high-entropy SHUMA_API_KEY using system CSPRNG tools
	@echo "$(CYAN)🔐 Generating SHUMA_API_KEY...$(NC)"
	@KEY="$$(if command -v openssl >/dev/null 2>&1; then openssl rand -hex 32; else od -An -N32 -tx1 /dev/urandom | tr -d ' \n'; fi)"; \
	echo ""; \
	echo "$$KEY"; \
	echo ""; \
	echo "$(YELLOW)Set in your secret store as: SHUMA_API_KEY=$$KEY$(NC)"

gen-admin-api-key: api-key-generate ## Alias for generating a strong SHUMA_API_KEY

api-key-show: ## Show SHUMA_API_KEY from .env.local (dashboard login key for local dev)
	@KEY="$$(grep -E '^SHUMA_API_KEY=' .env.local 2>/dev/null | tail -1 | cut -d= -f2- | sed -e 's/^"//' -e 's/"$$//')"; \
	if [ -z "$$KEY" ]; then \
		echo "$(RED)❌ No SHUMA_API_KEY found in .env.local.$(NC)"; \
		echo "$(YELLOW)Run: make setup$(NC)"; \
		exit 1; \
	fi; \
	echo "$(CYAN)Local dashboard login key (SHUMA_API_KEY):$(NC)"; \
	echo "$$KEY"

env-help: ## Show supported env-only runtime overrides
	@echo "$(CYAN)Supported env-only overrides (tunables are KV-backed):$(NC)"
	@echo "  SHUMA_API_KEY"
	@echo "  SHUMA_ADMIN_READONLY_API_KEY"
	@echo "  SHUMA_JS_SECRET"
	@echo "  SHUMA_POW_SECRET"
	@echo "  SHUMA_CHALLENGE_SECRET"
	@echo "  SHUMA_MAZE_PREVIEW_SECRET"
	@echo "  SHUMA_FORWARDED_IP_SECRET"
	@echo "  SHUMA_HEALTH_SECRET"
	@echo "  SHUMA_ADMIN_IP_ALLOWLIST"
	@echo "  SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"
	@echo "  SHUMA_EVENT_LOG_RETENTION_HOURS"
	@echo "  SHUMA_MONITORING_RETENTION_HOURS"
	@echo "  SHUMA_MONITORING_ROLLUP_RETENTION_HOURS"
	@echo "  SHUMA_ADMIN_CONFIG_WRITE_ENABLED"
	@echo "  SHUMA_KV_STORE_FAIL_OPEN"
	@echo "  SHUMA_ENFORCE_HTTPS"
	@echo "  SHUMA_DEBUG_HEADERS"
	@echo "  SHUMA_RUNTIME_ENV"
	@echo "  SHUMA_RUNTIME_DEV_OVERSIGHT_WATCH_WINDOW_SECONDS"
	@echo "  SHUMA_LOCAL_PROD_DIRECT_MODE"
	@echo "  SHUMA_ADVERSARY_SIM_AVAILABLE"
	@echo "  SHUMA_SIM_TELEMETRY_SECRET"
	@echo "  SHUMA_FRONTIER_OPENAI_API_KEY"
	@echo "  SHUMA_FRONTIER_ANTHROPIC_API_KEY"
	@echo "  SHUMA_FRONTIER_GOOGLE_API_KEY"
	@echo "  SHUMA_FRONTIER_XAI_API_KEY"
	@echo "  SHUMA_FRONTIER_OPENAI_MODEL"
	@echo "  SHUMA_FRONTIER_ANTHROPIC_MODEL"
	@echo "  SHUMA_FRONTIER_GOOGLE_MODEL"
	@echo "  SHUMA_FRONTIER_XAI_MODEL"
	@echo "  SHUMA_ENTERPRISE_MULTI_INSTANCE"
	@echo "  SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED"
	@echo "  SHUMA_RATE_LIMITER_REDIS_URL"
	@echo "  SHUMA_BAN_STORE_REDIS_URL"
	@echo "  SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"
	@echo "  SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH"
	@echo "  SHUMA_BAN_STORE_OUTAGE_MODE"
	@echo "  SHUMA_GATEWAY_UPSTREAM_ORIGIN"
	@echo "  SHUMA_GATEWAY_DEPLOYMENT_PROFILE"
	@echo "  SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL"
	@echo "  SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS"
	@echo "  SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST"
	@echo "  SHUMA_GATEWAY_PUBLIC_AUTHORITIES"
	@echo "  SHUMA_GATEWAY_LOOP_MAX_HOPS"
	@echo "  SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"
	@echo "  SHUMA_GATEWAY_ORIGIN_AUTH_MODE"
	@echo "  SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME"
	@echo "  SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"
	@echo "  SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"
	@echo "  SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS"
	@echo "  SHUMA_GATEWAY_TLS_STRICT"
	@echo "  SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"
	@echo ""

api-key-rotate: ## Generate a replacement SHUMA_API_KEY and print rotation guidance
	@$(MAKE) --no-print-directory api-key-generate
	@echo "$(YELLOW)Next steps: update deployment secret, redeploy/restart, then update dashboard login key.$(NC)"

api-key-validate: ## Validate SHUMA_API_KEY for deployment (must be 64-char hex and non-placeholder)
	@KEY="$(SHUMA_API_KEY)"; \
	if [ -z "$$KEY" ]; then \
		echo "$(RED)❌ SHUMA_API_KEY is empty.$(NC)"; \
		echo "$(YELLOW)Set SHUMA_API_KEY before deployment (or export it from your secret manager).$(NC)"; \
		exit 1; \
	fi; \
	case "$$KEY" in \
		changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key) \
			echo "$(RED)❌ SHUMA_API_KEY is a placeholder value. Generate a real key first.$(NC)"; \
			exit 1 ;; \
	esac; \
	if ! printf '%s' "$$KEY" | grep -Eq '^[0-9A-Fa-f]{64}$$'; then \
		echo "$(RED)❌ SHUMA_API_KEY must be exactly 64 hexadecimal characters.$(NC)"; \
		echo "$(YELLOW)Generate one with: make api-key-generate$(NC)"; \
		exit 1; \
	fi; \
	echo "$(GREEN)✅ SHUMA_API_KEY format is valid for deployment.$(NC)"

deploy-env-validate: ## Fail deployment when unsafe debug flags are enabled, admin allowlist is missing, admin edge limits are unconfirmed, API-key rotation is unconfirmed, enterprise state guardrails fail, gateway outbound contract is misaligned, or reserved-route preflight is missing/failed
	@DEBUG_VAL="$${SHUMA_DEBUG_HEADERS:-false}"; \
	DEBUG_NORM="$$(printf '%s' "$$DEBUG_VAL" | tr '[:upper:]' '[:lower:]')"; \
	case "$$DEBUG_NORM" in \
		1|true|yes|on) \
			echo "$(RED)❌ Refusing deployment: SHUMA_DEBUG_HEADERS=true exposes internal headers.$(NC)"; \
			echo "$(YELLOW)Set SHUMA_DEBUG_HEADERS=false for production deploys.$(NC)"; \
			exit 1 ;; \
	esac; \
	ALLOWLIST_RAW="$${SHUMA_ADMIN_IP_ALLOWLIST:-}"; \
	ALLOWLIST_NORM="$$(printf '%s' "$$ALLOWLIST_RAW" | tr -d '[:space:]')"; \
	if [ -z "$$ALLOWLIST_NORM" ]; then \
		echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_IP_ALLOWLIST is required for production admin hardening.$(NC)"; \
		echo "$(YELLOW)Set SHUMA_ADMIN_IP_ALLOWLIST to one or more trusted IP/CIDR entries (comma-separated).$(NC)"; \
		exit 1; \
	fi; \
	case "$$ALLOWLIST_NORM" in \
		*0.0.0.0/0*|*::/0*|*\**) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_IP_ALLOWLIST is overbroad (contains wildcard/global range).$(NC)"; \
			echo "$(YELLOW)Use explicit trusted operator/VPN IPs or CIDRs only.$(NC)"; \
			exit 1 ;; \
	esac; \
	EDGE_LIMITS_CONFIRMED_RAW="$${SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED:-false}"; \
	EDGE_LIMITS_CONFIRMED_NORM="$$(printf '%s' "$$EDGE_LIMITS_CONFIRMED_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$EDGE_LIMITS_CONFIRMED_NORM" in \
		1|true|yes|on) ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED is not true.$(NC)"; \
			echo "$(YELLOW)Before deploy, configure CDN/WAF limits for POST /admin/login and /admin/*, then set SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true.$(NC)"; \
			exit 1 ;; \
	esac; \
	API_KEY_ROTATION_CONFIRMED_RAW="$${SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED:-false}"; \
	API_KEY_ROTATION_CONFIRMED_NORM="$$(printf '%s' "$$API_KEY_ROTATION_CONFIRMED_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$API_KEY_ROTATION_CONFIRMED_NORM" in \
		1|true|yes|on) ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED is not true.$(NC)"; \
			echo "$(YELLOW)Rotate SHUMA_API_KEY on your cadence (recommended 90 days) with make gen-admin-api-key / make api-key-rotate, then set SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true.$(NC)"; \
			exit 1 ;; \
	esac; \
	ENTERPRISE_MULTI_INSTANCE_RAW="$${SHUMA_ENTERPRISE_MULTI_INSTANCE:-false}"; \
	ENTERPRISE_MULTI_INSTANCE_NORM="$$(printf '%s' "$$ENTERPRISE_MULTI_INSTANCE_RAW" | tr '[:upper:]' '[:lower:]')"; \
	case "$$ENTERPRISE_MULTI_INSTANCE_NORM" in \
		1|true|yes|on) \
			EDGE_MODE_RAW="$${SHUMA_EDGE_INTEGRATION_MODE:-off}"; \
			EDGE_MODE_NORM="$$(printf '%s' "$$EDGE_MODE_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$EDGE_MODE_NORM" in \
				off|additive|authoritative) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_EDGE_INTEGRATION_MODE must be one of off|additive|authoritative when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			RATE_BACKEND_RAW="$${SHUMA_PROVIDER_RATE_LIMITER:-internal}"; \
			RATE_BACKEND_NORM="$$(printf '%s' "$$RATE_BACKEND_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$RATE_BACKEND_NORM" in \
				internal|external) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_PROVIDER_RATE_LIMITER must be internal|external when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			if [ "$$RATE_BACKEND_NORM" = "external" ]; then \
				RATE_REDIS_URL_RAW="$${SHUMA_RATE_LIMITER_REDIS_URL:-}"; \
				RATE_REDIS_URL_NORM="$$(printf '%s' "$$RATE_REDIS_URL_RAW" | tr -d '[:space:]')"; \
				if [ -z "$$RATE_REDIS_URL_NORM" ]; then \
					echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_REDIS_URL is required when SHUMA_ENTERPRISE_MULTI_INSTANCE=true and SHUMA_PROVIDER_RATE_LIMITER=external.$(NC)"; \
					exit 1; \
				fi; \
				case "$$RATE_REDIS_URL_NORM" in \
					redis://*|rediss://*) ;; \
					*) \
						echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_REDIS_URL must start with redis:// or rediss://.$(NC)"; \
						exit 1 ;; \
				esac; \
			fi; \
			BAN_BACKEND_RAW="$${SHUMA_PROVIDER_BAN_STORE:-internal}"; \
			BAN_BACKEND_NORM="$$(printf '%s' "$$BAN_BACKEND_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$BAN_BACKEND_NORM" in \
				internal|external) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_PROVIDER_BAN_STORE must be internal|external when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			if [ "$$BAN_BACKEND_NORM" = "external" ]; then \
				BAN_REDIS_URL_RAW="$${SHUMA_BAN_STORE_REDIS_URL:-}"; \
				BAN_REDIS_URL_NORM="$$(printf '%s' "$$BAN_REDIS_URL_RAW" | tr -d '[:space:]')"; \
				if [ -z "$$BAN_REDIS_URL_NORM" ]; then \
					echo "$(RED)❌ Refusing deployment: SHUMA_BAN_STORE_REDIS_URL is required when SHUMA_ENTERPRISE_MULTI_INSTANCE=true and SHUMA_PROVIDER_BAN_STORE=external.$(NC)"; \
					exit 1; \
				fi; \
				case "$$BAN_REDIS_URL_NORM" in \
					redis://*|rediss://*) ;; \
					*) \
						echo "$(RED)❌ Refusing deployment: SHUMA_BAN_STORE_REDIS_URL must start with redis:// or rediss://.$(NC)"; \
						exit 1 ;; \
				esac; \
			fi; \
			RATE_OUTAGE_MAIN_RAW="$${SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN:-fallback_internal}"; \
			RATE_OUTAGE_MAIN_NORM="$$(printf '%s' "$$RATE_OUTAGE_MAIN_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$RATE_OUTAGE_MAIN_NORM" in \
				fallback_internal|fail_open|fail_closed) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN must be fallback_internal|fail_open|fail_closed when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			RATE_OUTAGE_ADMIN_RAW="$${SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH:-fail_closed}"; \
			RATE_OUTAGE_ADMIN_NORM="$$(printf '%s' "$$RATE_OUTAGE_ADMIN_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$RATE_OUTAGE_ADMIN_NORM" in \
				fallback_internal|fail_open|fail_closed) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH must be fallback_internal|fail_open|fail_closed when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			BAN_OUTAGE_RAW="$${SHUMA_BAN_STORE_OUTAGE_MODE:-fallback_internal}"; \
			BAN_OUTAGE_NORM="$$(printf '%s' "$$BAN_OUTAGE_RAW" | tr '[:upper:]' '[:lower:]')"; \
			case "$$BAN_OUTAGE_NORM" in \
				fallback_internal|fail_open|fail_closed) ;; \
				*) \
					echo "$(RED)❌ Refusing deployment: SHUMA_BAN_STORE_OUTAGE_MODE must be fallback_internal|fail_open|fail_closed when SHUMA_ENTERPRISE_MULTI_INSTANCE=true.$(NC)"; \
					exit 1 ;; \
			esac; \
			UNSYNCED_LOCAL_STATE=0; \
			if [ "$$RATE_BACKEND_NORM" != "external" ] || [ "$$BAN_BACKEND_NORM" != "external" ]; then \
				UNSYNCED_LOCAL_STATE=1; \
			fi; \
			if [ "$$UNSYNCED_LOCAL_STATE" -eq 1 ]; then \
				if [ "$$EDGE_MODE_NORM" = "authoritative" ]; then \
					echo "$(RED)❌ Refusing deployment: enterprise multi-instance rollout cannot run with local-only rate/ban state in authoritative edge mode.$(NC)"; \
					echo "$(YELLOW)Use distributed state backends first, or move to advisory mode for a temporary exception window.$(NC)"; \
					exit 1; \
				fi; \
				UNSYNCED_EXCEPTION_RAW="$${SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED:-false}"; \
				UNSYNCED_EXCEPTION_NORM="$$(printf '%s' "$$UNSYNCED_EXCEPTION_RAW" | tr '[:upper:]' '[:lower:]')"; \
				case "$$UNSYNCED_EXCEPTION_NORM" in \
					1|true|yes|on) ;; \
					*) \
						echo "$(RED)❌ Refusing deployment: enterprise multi-instance rollout is using local-only rate/ban state without explicit exception attestation.$(NC)"; \
						echo "$(YELLOW)Set distributed state backends (SHUMA_PROVIDER_RATE_LIMITER=external and SHUMA_PROVIDER_BAN_STORE=external), or set SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=true for temporary advisory-only operation.$(NC)"; \
						exit 1 ;; \
				esac; \
			elif [ "$$EDGE_MODE_NORM" = "authoritative" ] && [ "$$BAN_OUTAGE_NORM" != "fail_closed" ]; then \
				echo "$(RED)❌ Refusing deployment: authoritative enterprise ban sync requires SHUMA_BAN_STORE_OUTAGE_MODE=fail_closed.$(NC)"; \
				exit 1; \
			fi ;; \
		0|false|no|off|"") ;; \
		*) \
			echo "$(RED)❌ Refusing deployment: SHUMA_ENTERPRISE_MULTI_INSTANCE must be a boolean value (true/false).$(NC)"; \
			exit 1 ;; \
	esac; \
	echo "$(GREEN)✅ Deployment env guardrails passed (SHUMA_DEBUG_HEADERS, SHUMA_ADMIN_IP_ALLOWLIST, SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED, SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED, enterprise multi-instance state guardrails).$(NC)"
	@python3 scripts/deploy/validate_gateway_contract.py
	@python3 scripts/deploy/validate_gateway_route_collisions.py

#--------------------------
# Help
#--------------------------

help: ## Show this help message
	@echo "$(CYAN)WASM Bot Defence - Available Commands$(NC)"
	@echo ""
	@echo "$(GREEN)First-time Setup:$(NC)"
	@grep -h -E '^(setup|setup-runtime|verify|verify-runtime|config-seed|config-verify):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@grep -h -E '^(dev|dev-prod|local|run|build|build-runtime|build-full-dev|adversary-sim-supervisor-build|adversary-sim-supervisor):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Production:$(NC)"
	@grep -h -E '^(prod|deploy|deploy-profile-baseline|deploy-self-hosted-minimal|deploy-enterprise-akamai|deploy-linode-one-shot):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@grep -h -E '^(test.*|smoke-single-host):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Utilities:$(NC)"
	@grep -h -E '^(stop|status|clean|reset-local-state|logs|env-help|telemetry-clean|adversary-sim-supervisor|api-key-generate|gen-admin-api-key|api-key-show|api-key-rotate|api-key-validate|deploy-env-validate|help|remote-use|remote-update|remote-start|remote-stop|remote-status|remote-logs|remote-open-dashboard):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
