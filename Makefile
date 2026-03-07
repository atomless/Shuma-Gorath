.PHONY: dev dev-prod local run run-prebuilt build build-runtime build-full-dev prod prod-start clean test test-unit unit-test test-integration integration-test test-gateway-harness test-gateway-wasm-tls-harness test-gateway-origin-bypass-probe test-gateway-profile-shared-server test-gateway-profile-edge smoke-gateway-mode test-deploy-linode test-config-lifecycle test-adversarial-python-unit test-adversarial-manifest test-adversarial-preflight test-adversarial-lane-contract test-adversarial-sim-tag-contract test-adversarial-coverage-contract test-adversarial-scenario-review test-adversarial-sim-selftest test-adversarial-fast test-adversarial-smoke test-adversarial-abuse test-adversarial-akamai test-adversarial-coverage test-adversarial-soak test-adversarial-live telemetry-clean adversary-sim-supervisor-build adversary-sim-supervisor test-adversary-sim-runtime-surface test-adversarial-repeatability test-adversarial-promote-candidates test-adversarial-report-diff test-adversarial-container-blackbox test-adversarial-container-isolation test-adversarial-frontier-attempt test-frontier-governance test-frontier-unavailability-policy test-sim2-realtime-bench test-sim2-adr-conformance test-sim2-ci-diagnostics test-sim2-verification-matrix test-sim2-verification-matrix-advisory test-sim2-operational-regressions test-sim2-operational-regressions-strict test-sim2-governance-contract test-sim2-verification-e2e test-ip-range-suggestions test-coverage test-dashboard test-dashboard-svelte-check test-dashboard-unit test-dashboard-budgets test-dashboard-budgets-strict test-dashboard-e2e test-dashboard-e2e-adversary-sim seed-dashboard-data test-maze-benchmark spin-wait-ready smoke-single-host prepare-linode-shared-host remote-use remote-update remote-start remote-stop remote-status remote-logs remote-open-dashboard deploy deploy-profile-baseline deploy-self-hosted-minimal deploy-enterprise-akamai deploy-linode-one-shot logs status stop help setup setup-runtime verify verify-runtime config-seed config-verify dashboard-build env-help api-key-generate gen-admin-api-key api-key-show api-key-rotate api-key-validate deploy-env-validate

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
LINODE_SETUP_RECEIPT ?= .spin/linode-shared-host-setup.json
REMOTE_RECEIPTS_DIR ?= .spin/remotes

# Normalize optional quoted values from .env.local (handles KEY=value and KEY="value")
strip_wrapping_quotes = $(patsubst "%",%,$(patsubst '%',%,$(strip $(1))))
json_receipt_value = $(strip $(shell python3 -c 'import json,pathlib,sys; p=pathlib.Path(sys.argv[1]); keys=sys.argv[2].split("."); cur=json.loads(p.read_text(encoding="utf-8")) if p.exists() else {}; [cur := cur.get(key, "") if isinstance(cur, dict) else "" for key in keys]; print(cur if isinstance(cur, str) else "")' "$(LINODE_SETUP_RECEIPT)" "$(1)" 2>/dev/null))
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
SHUMA_ADMIN_CONFIG_WRITE_ENABLED := $(call strip_wrapping_quotes,$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED))
SHUMA_KV_STORE_FAIL_OPEN := $(call strip_wrapping_quotes,$(SHUMA_KV_STORE_FAIL_OPEN))
SHUMA_ENFORCE_HTTPS := $(call strip_wrapping_quotes,$(SHUMA_ENFORCE_HTTPS))
SHUMA_DEBUG_HEADERS := $(call strip_wrapping_quotes,$(SHUMA_DEBUG_HEADERS))
SHUMA_RUNTIME_ENV := $(call strip_wrapping_quotes,$(SHUMA_RUNTIME_ENV))
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
SHUMA_SPIN_MANIFEST := $(call strip_wrapping_quotes,$(SHUMA_SPIN_MANIFEST))
GATEWAY_SURFACE_CATALOG_PATH := $(call strip_wrapping_quotes,$(GATEWAY_SURFACE_CATALOG_PATH))
SSH_PRIVATE_KEY_FILE := $(call strip_wrapping_quotes,$(SSH_PRIVATE_KEY_FILE))
SSH_PUBLIC_KEY_FILE := $(call strip_wrapping_quotes,$(SSH_PUBLIC_KEY_FILE))
SHUMA_RATE_LIMITER_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_REDIS_URL))
SHUMA_BAN_STORE_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_BAN_STORE_REDIS_URL))
SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN))
SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH))
SSH_PRIVATE_KEY_FILE := $(if $(strip $(SSH_PRIVATE_KEY_FILE)),$(SSH_PRIVATE_KEY_FILE),$(call json_receipt_value,ssh.private_key_path))
SSH_PUBLIC_KEY_FILE := $(if $(strip $(SSH_PUBLIC_KEY_FILE)),$(SSH_PUBLIC_KEY_FILE),$(call json_receipt_value,ssh.public_key_path))
SPIN_UP_MANIFEST := $(if $(strip $(SHUMA_SPIN_MANIFEST)),$(SHUMA_SPIN_MANIFEST),spin.toml)

DEPLOY_ENV_ONLY := \
	SHUMA_API_KEY="$(SHUMA_API_KEY)" \
	SHUMA_JS_SECRET="$(SHUMA_JS_SECRET)" \
	SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" \
	SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
	SHUMA_SIM_TELEMETRY_SECRET="$(SHUMA_SIM_TELEMETRY_SECRET)" \
	SHUMA_DEBUG_HEADERS="$(SHUMA_DEBUG_HEADERS)" \
	SHUMA_ADMIN_IP_ALLOWLIST="$(SHUMA_ADMIN_IP_ALLOWLIST)" \
	SHUMA_ADMIN_CONFIG_WRITE_ENABLED="$(SHUMA_ADMIN_CONFIG_WRITE_ENABLED)" \
	SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED="$(SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED)" \
	SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED="$(SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED)" \
	SHUMA_ENTERPRISE_MULTI_INSTANCE="$(SHUMA_ENTERPRISE_MULTI_INSTANCE)" \
	SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED="$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED)" \
	SHUMA_PROVIDER_RATE_LIMITER="$(SHUMA_PROVIDER_RATE_LIMITER)" \
	SHUMA_PROVIDER_BAN_STORE="$(SHUMA_PROVIDER_BAN_STORE)" \
	SHUMA_RATE_LIMITER_REDIS_URL="$(SHUMA_RATE_LIMITER_REDIS_URL)" \
	SHUMA_BAN_STORE_REDIS_URL="$(SHUMA_BAN_STORE_REDIS_URL)" \
	SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN="$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN)" \
	SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH="$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH)" \
	SHUMA_GATEWAY_UPSTREAM_ORIGIN="$(SHUMA_GATEWAY_UPSTREAM_ORIGIN)" \
	SHUMA_GATEWAY_DEPLOYMENT_PROFILE="$(SHUMA_GATEWAY_DEPLOYMENT_PROFILE)" \
	SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED="$(SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED)" \
	SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED="$(SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED)" \
	SHUMA_GATEWAY_TLS_STRICT="$(SHUMA_GATEWAY_TLS_STRICT)" \
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
	--env SHUMA_KV_STORE_FAIL_OPEN=$(SHUMA_KV_STORE_FAIL_OPEN) \
	--env SHUMA_ENFORCE_HTTPS=$(SHUMA_ENFORCE_HTTPS) \
	--env SHUMA_RUNTIME_ENV=$(SHUMA_RUNTIME_ENV) \
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
SPIN_DEV_OVERRIDES := --env SHUMA_DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_ADMIN_IP_ALLOWLIST=$(DEV_ADMIN_IP_ALLOWLIST) --env SHUMA_RUNTIME_ENV=$(DEV_RUNTIME_ENV) --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) --env SHUMA_LOCAL_PROD_DIRECT_MODE=$(DEV_LOCAL_PROD_DIRECT_MODE)
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
		-s 'pkill -x spin 2>/dev/null || true; $(MAKE) --no-print-directory config-verify && $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1 && RUNTIME_INSTANCE_ID="$$(uuidgen)" && SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) SPIN_ALWAYS_BUILD=0 ./scripts/run_with_adversary_sim_supervisor.sh spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000'

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
		-s 'pkill -x spin 2>/dev/null || true; $(MAKE) --no-print-directory config-verify && $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1 && RUNTIME_INSTANCE_ID="$$(uuidgen)" && SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) SPIN_ALWAYS_BUILD=0 ./scripts/run_with_adversary_sim_supervisor.sh spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env SHUMA_KV_STORE_FAIL_OPEN=false --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000'

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
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) ./scripts/run_with_adversary_sim_supervisor.sh spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000

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
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE) ./scripts/run_with_adversary_sim_supervisor.sh spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 127.0.0.1:3000

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
	@RUNTIME_INSTANCE_ID=$$(uuidgen); SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) ./scripts/run_with_adversary_sim_supervisor.sh spin up --from $(SPIN_UP_MANIFEST) $(SPIN_ENV_ONLY_BASE) $(SPIN_PROD_OVERRIDES) --env RUNTIME_INSTANCE_ID=$$RUNTIME_INSTANCE_ID --listen 0.0.0.0:3000

prod: build-runtime ## Build for production and start server
	@$(MAKE) --no-print-directory prod-start

deploy: build-runtime ## Deploy to Fermyon Cloud
	@$(MAKE) --no-print-directory api-key-validate
	@$(MAKE) --no-print-directory deploy-env-validate
	@echo "$(CYAN)☁️  Deploying to Fermyon Cloud...$(NC)"
	@spin cloud deploy
	@echo "$(GREEN)✅ Deployment complete!$(NC)"

deploy-profile-baseline: ## Profile wrapper baseline: verify seeded config + runtime build
	@echo "$(CYAN)🔧 Running shared deployment baseline...$(NC)"
	@$(MAKE) --no-print-directory config-verify
	@$(MAKE) --no-print-directory build-runtime
	@echo "$(GREEN)✅ Shared deployment baseline complete.$(NC)"

deploy-self-hosted-minimal: deploy-profile-baseline ## Profile wrapper: self_hosted_minimal pre-deploy guardrails
	@echo "$(CYAN)🏠 Validating self_hosted_minimal deployment posture...$(NC)"
	@SHUMA_ENTERPRISE_MULTI_INSTANCE=false $(MAKE) --no-print-directory deploy-env-validate
	@echo "$(GREEN)✅ self_hosted_minimal pre-deploy checks passed.$(NC)"

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
	@$(MAKE) --no-print-directory deploy-env-validate
	@echo "$(GREEN)✅ enterprise_akamai overlay pre-deploy checks passed.$(NC)"

deploy-linode-one-shot: ## Provision Linode VM + deploy Shuma runtime in one command (requires LINODE_TOKEN and SHUMA_ADMIN_IP_ALLOWLIST)
	@ENV_LOCAL="$(ENV_LOCAL)" \
	LINODE_TOKEN="$(LINODE_TOKEN)" \
	REMOTE_RECEIPTS_DIR="$(REMOTE_RECEIPTS_DIR)" \
	SSH_PRIVATE_KEY_FILE="$(SSH_PRIVATE_KEY_FILE)" \
	SSH_PUBLIC_KEY_FILE="$(SSH_PUBLIC_KEY_FILE)" \
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

test: ## Run umbrella tests in series: unit, maze benchmark, integration, adversarial matrix, SIM2 realtime gates, and dashboard e2e
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
	@echo ""
	@echo "$(CYAN)Step 1/8: Rust Unit Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test || exit 1
	@echo ""
	@echo "$(CYAN)Step 2/8: Maze Asymmetry Benchmark Gate$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-maze-benchmark || exit 1
	@echo ""
	@echo "$(CYAN)Step 3/8: Integration Tests (Spin HTTP scenarios)$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
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
	@SIM2_MATRIX_TARGET="test-sim2-verification-matrix"; \
	if ! $(MAKE) --no-print-directory test-adversarial-container-blackbox; then \
		echo "$(YELLOW)Container black-box lane unavailable; running SIM2 matrix in advisory mode.$(NC)"; \
		SIM2_MATRIX_TARGET="test-sim2-verification-matrix-advisory"; \
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

unit-test: test-unit ## Alias for Rust unit tests

test-integration: ## Run integration tests only (21 scenarios, requires running server)
	@echo "$(CYAN)🧪 Running integration tests...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/integration.sh; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev (or make dev-prod / make prod)$(NC)"; \
		exit 1; \
	fi

integration-test: test-integration ## Alias for Spin integration tests

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
	@python3 -m unittest scripts/tests/test_validate_gateway_route_collisions.py
	@python3 -m unittest scripts/tests/test_prepare_linode_shared_host.py
	@python3 -m unittest scripts/tests/test_remote_target.py
	@python3 -m unittest scripts/tests/test_render_gateway_spin_manifest.py
	@python3 -m unittest scripts/tests/test_deploy_linode_one_shot.py
	@python3 -m unittest scripts/tests/test_prod_start_spin_manifest.py
	@python3 -m unittest scripts/tests/test_select_gateway_smoke_path.py
	@python3 -m unittest scripts/tests/test_setup_runtime_spin_install.py
	@python3 -m unittest scripts/tests/test_smoke_single_host.py
	@python3 -m unittest scripts/tests/test_wait_for_spin_ready.py

test-config-lifecycle: ## Validate read-only runtime config lifecycle checks and explicit seed/backfill flows
	@echo "$(CYAN)🧪 Running config lifecycle verification...$(NC)"
	@python3 -m unittest scripts/tests/test_config_lifecycle.py

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
	@python3 -m py_compile scripts/tests/adversarial_simulation_runner.py scripts/tests/adversary_runtime_toggle_surface_gate.py scripts/tests/adversarial_preflight.py scripts/tests/adversarial_live_loop.py scripts/tests/adversarial_repeatability.py scripts/tests/adversarial_promote_candidates.py scripts/tests/adversarial_report_diff.py scripts/tests/adversarial_container_runner.py scripts/tests/adversarial_container/worker.py scripts/tests/frontier_action_contract.py scripts/tests/frontier_capability_envelope.py scripts/tests/frontier_lane_attempt.py scripts/tests/frontier_unavailability_policy.py scripts/tests/check_frontier_payload_artifacts.py scripts/tests/check_adversarial_lane_contract.py scripts/tests/check_adversarial_sim_tag_contract.py scripts/tests/check_adversarial_coverage_contract.py scripts/tests/check_adversarial_scenario_intent_matrix.py scripts/tests/sim2_realtime_bench.py scripts/tests/check_sim2_adr_conformance.py scripts/tests/render_sim2_ci_diagnostics.py scripts/tests/check_sim2_verification_matrix.py scripts/tests/check_sim2_operational_regressions.py scripts/tests/check_sim2_governance_contract.py
	@node --check scripts/tests/adversarial_browser_driver.mjs
	@python3 -m unittest scripts/tests/test_adversary_runtime_toggle_surface_gate.py scripts/tests/test_adversarial_simulation_runner.py scripts/tests/test_adversarial_preflight.py scripts/tests/test_adversarial_live_loop.py scripts/tests/test_adversarial_repeatability.py scripts/tests/test_adversarial_promote_candidates.py scripts/tests/test_adversarial_report_diff.py scripts/tests/test_adversarial_container_runner.py scripts/tests/test_adversarial_container_worker.py scripts/tests/test_frontier_action_contract.py scripts/tests/test_frontier_capability_envelope.py scripts/tests/test_frontier_lane_and_governance.py scripts/tests/test_adversarial_lane_contract.py scripts/tests/test_adversarial_sim_tag_contract.py scripts/tests/test_adversarial_coverage_contract.py scripts/tests/test_adversarial_scenario_intent_matrix.py scripts/tests/test_sim2_realtime_bench.py scripts/tests/test_sim2_adr_conformance.py scripts/tests/test_sim2_ci_diagnostics.py scripts/tests/test_sim2_verification_matrix.py scripts/tests/test_sim2_operational_regressions.py scripts/tests/test_sim2_governance_contract.py

test-adversarial-preflight: ## Validate adversarial required secrets and setup posture before runner execution
	@echo "$(CYAN)🧪 Running adversarial preflight checks...$(NC)"
	@python3 scripts/tests/adversarial_preflight.py --output scripts/tests/adversarial/preflight_report.json

test-adversarial-lane-contract: ## Validate black-box lane capability contract parity across deterministic/container tooling
	@echo "$(CYAN)🧪 Validating adversarial lane capability contract...$(NC)"
	@python3 scripts/tests/check_adversarial_lane_contract.py

test-adversarial-deterministic-corpus: ## Validate shared deterministic attack corpus parity across runtime and CI oracle lanes
	@echo "$(CYAN)🧪 Validating shared deterministic attack corpus parity...$(NC)"
	@python3 scripts/tests/check_adversarial_deterministic_corpus.py

test-adversary-sim-lifecycle: ## Fast adversary-sim lifecycle regression gate (toggle/state/heartbeat contracts)
	@echo "$(CYAN)🧪 Running adversary-sim lifecycle regression gate...$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test adversary_sim_control_start_stop_and_status_round_trip -- --nocapture
	@cargo test adversary_sim_status_reconciles_idle_enabled_state_to_off -- --nocapture
	@cargo test adversary_sim_status_forces_off_when_run_owned_by_previous_process_instance -- --nocapture
	@cargo test adversary_sim_internal_beat_updates_generation_diagnostics_contract -- --nocapture
	@python3 -m unittest scripts/tests/test_adversary_sim_supervisor.py
	@$(MAKE) --no-print-directory test-adversarial-deterministic-corpus

adversary-sim-supervisor-build: ## Build the host-side adversary-sim supervisor worker binary
	@./scripts/adversary_sim_supervisor_launch.sh --build-only

adversary-sim-supervisor: adversary-sim-supervisor-build ## Run host-side adversary-sim supervisor loop (watch mode)
	@SHUMA_ADVERSARY_SIM_SUPERVISOR_BASE_URL=$(ADVERSARY_SIM_SUPERVISOR_BASE_URL) \
		./scripts/adversary_sim_supervisor_launch.sh --watch --base-url $(ADVERSARY_SIM_SUPERVISOR_BASE_URL)

test-adversary-sim-runtime-surface: ## Runtime-toggle integration gate for deterministic defense-surface telemetry coverage (requires running server)
	@echo "$(CYAN)🧪 Running runtime-toggle adversary-sim surface coverage gate...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversary_runtime_toggle_surface_gate.py; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-sim-tag-contract: ## Validate simulation tag signing contract parity across runtime/tooling
	@echo "$(CYAN)🧪 Validating adversarial sim-tag contract...$(NC)"
	@python3 scripts/tests/check_adversarial_sim_tag_contract.py

test-adversarial-coverage-contract: ## Validate full-coverage contract parity across plan, manifest, and runner
	@echo "$(CYAN)🧪 Validating adversarial coverage contract...$(NC)"
	@python3 scripts/tests/check_adversarial_coverage_contract.py

test-adversarial-scenario-review: ## Validate scenario intent matrix parity and review freshness governance
	@echo "$(CYAN)🧪 Validating adversarial scenario intent matrix...$(NC)"
	@python3 scripts/tests/check_adversarial_scenario_intent_matrix.py

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
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-abuse: ## Run replay/stale/ordering abuse regression profile (requires running server)
	@echo "$(CYAN)🧪 Running adversarial abuse regression profile...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile abuse_regression; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-akamai: ## Run Akamai signal fixture smoke profile (requires running server)
	@echo "$(CYAN)🧪 Running adversarial Akamai fixture profile...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=0 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile akamai_smoke; \
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
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" SHUMA_ADVERSARIAL_PRESERVE_STATE=0 SHUMA_ADVERSARIAL_ROTATE_IPS=1 python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile full_coverage || exit 1; \
		$(MAKE) --no-print-directory test-frontier-governance || exit 1; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
			exit 1; \
		fi

test-adversarial-frontier-attempt: ## Attempt frontier provider probes for protected lanes (advisory/non-blocking)
	@echo "$(CYAN)🧪 Attempting protected-lane frontier provider probes (advisory)...$(NC)"
	@python3 scripts/tests/frontier_lane_attempt.py --output scripts/tests/adversarial/frontier_lane_status.json

test-frontier-governance: ## Fail when forbidden frontier fields or secret values appear in report artifacts
	@echo "$(CYAN)🧪 Verifying frontier artifact governance guardrails...$(NC)"
	@python3 scripts/tests/check_frontier_payload_artifacts.py --report scripts/tests/adversarial/latest_report.json --attack-plan scripts/tests/adversarial/attack_plan.json --schema scripts/tests/adversarial/frontier_payload_schema.v1.json

test-frontier-unavailability-policy: ## Evaluate frontier degraded-threshold policy and emit actionability artifact
	@echo "$(CYAN)🧪 Evaluating frontier unavailability policy thresholds...$(NC)"
	@ARGS=""; \
		if [ "$${FRONTIER_POLICY_ENABLE_GITHUB:-0}" = "1" ]; then \
			ARGS="--enable-github"; \
		fi; \
		python3 scripts/tests/frontier_unavailability_policy.py --status scripts/tests/adversarial/frontier_lane_status.json --output scripts/tests/adversarial/frontier_unavailability_policy.json $$ARGS

test-sim2-realtime-bench: ## Run deterministic SIM2 realtime benchmark gate and emit latency/overflow/request-budget artifacts
	@echo "$(CYAN)🧪 Running SIM2 realtime benchmark gate...$(NC)"
	@python3 scripts/tests/sim2_realtime_bench.py --output scripts/tests/adversarial/sim2_realtime_bench_report.json --summary scripts/tests/adversarial/sim2_realtime_bench_summary.md

test-sim2-adr-conformance: ## Verify SIM2 ADR conformance markers for ADR 0007/0008/0009 domains
	@echo "$(CYAN)🧪 Running SIM2 ADR conformance checks...$(NC)"
	@python3 scripts/tests/check_sim2_adr_conformance.py --output scripts/tests/adversarial/sim2_adr_conformance_report.json

test-sim2-ci-diagnostics: ## Render SIM2 CI diagnostics artifact (timeline snapshots, event counts, refresh traces)
	@echo "$(CYAN)🧪 Rendering SIM2 CI diagnostics artifact...$(NC)"
	@python3 scripts/tests/render_sim2_ci_diagnostics.py --report scripts/tests/adversarial/latest_report.json --output scripts/tests/adversarial/sim2_ci_diagnostics.json

test-sim2-verification-matrix: ## Validate SIM2 verification matrix rows and evidence diagnostics
	@echo "$(CYAN)🧪 Validating SIM2 verification matrix...$(NC)"
	@python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report scripts/tests/adversarial/latest_report.json --container-report scripts/tests/adversarial/container_blackbox_report.json --output scripts/tests/adversarial/sim2_verification_matrix_report.json

test-sim2-verification-matrix-advisory: ## Validate SIM2 verification matrix rows (advisory mode allows missing container report for local manifest checks)
	@echo "$(CYAN)🧪 Validating SIM2 verification matrix (advisory)...$(NC)"
	@python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report scripts/tests/adversarial/latest_report.json --container-report scripts/tests/adversarial/container_blackbox_report.json --output scripts/tests/adversarial/sim2_verification_matrix_report.json --allow-missing-container-report

test-sim2-operational-regressions: ## Validate SIM2 operational regression diagnostics for active deterministic profiles (retention/cost/security required; failure/prod evaluated when present)
	@echo "$(CYAN)🧪 Validating SIM2 operational regression diagnostics...$(NC)"
	@python3 scripts/tests/check_sim2_operational_regressions.py --report scripts/tests/adversarial/latest_report.json --output scripts/tests/adversarial/sim2_operational_regressions_report.json --allow-missing-domain failure_injection --allow-missing-domain prod_mode_monitoring --min-large-payload-samples-for-compression-check 2

test-sim2-operational-regressions-strict: ## Validate all SIM2 operational regression domains with strict missing-domain and compression enforcement
	@echo "$(CYAN)🧪 Validating strict SIM2 operational regression diagnostics...$(NC)"
	@python3 scripts/tests/check_sim2_operational_regressions.py --report scripts/tests/adversarial/latest_report.json --output scripts/tests/adversarial/sim2_operational_regressions_report.json

test-sim2-governance-contract: ## Validate SIM2 governance + hybrid lane contract markers and thresholds
	@echo "$(CYAN)🧪 Validating SIM2 governance + hybrid lane contract...$(NC)"
	@python3 scripts/tests/check_sim2_governance_contract.py --contract scripts/tests/adversarial/hybrid_lane_contract.v1.json --promotion-script scripts/tests/adversarial_promote_candidates.py --operator-guide docs/adversarial-operator-guide.md --output scripts/tests/adversarial/sim2_governance_contract_report.json

test-sim2-verification-e2e: ## Run matrix-required SIM2 e2e suite across crawler/scraper/browser/frontier lanes (requires running server + Docker)
	@echo "$(CYAN)🧪 Running SIM2 verification e2e suite...$(NC)"
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		$(MAKE) --no-print-directory test-adversarial-coverage || exit 1; \
		$(MAKE) --no-print-directory test-adversarial-container-blackbox || exit 1; \
		python3 scripts/tests/check_sim2_verification_matrix.py --matrix scripts/tests/adversarial/verification_matrix.v1.json --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report scripts/tests/adversarial/latest_report.json --container-report scripts/tests/adversarial/container_blackbox_report.json --output scripts/tests/adversarial/sim2_verification_matrix_report.json || exit 1; \
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
	REPORT_PATH="$${ADVERSARIAL_REPORT_PATH:-scripts/tests/adversarial/latest_report.json}"; \
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
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_repeatability.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --repeats "$${ADVERSARIAL_REPEATABILITY_REPEATS:-3}" --profiles "$${ADVERSARIAL_REPEATABILITY_PROFILES:-fast_smoke,abuse_regression,full_coverage}" --report scripts/tests/adversarial/repeatability_report.json; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-promote-candidates: ## Run frontier candidate triage + deterministic replay promotion checks (requires running server)
	@echo "$(CYAN)🧪 Running adversarial candidate triage and promotion checks...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		REPORT_PATH="scripts/tests/adversarial/latest_report.json"; \
		ATTACK_PLAN_PATH="scripts/tests/adversarial/attack_plan.json"; \
		if [ ! -f "$$REPORT_PATH" ] || [ ! -f "$$ATTACK_PLAN_PATH" ]; then \
			echo "$(YELLOW)   Missing adversarial report artifacts; generating with test-adversarial-coverage...$(NC)"; \
			$(MAKE) --no-print-directory test-adversarial-coverage || exit 1; \
		fi; \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_promote_candidates.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --report "$$REPORT_PATH" --attack-plan "$$ATTACK_PLAN_PATH" --output scripts/tests/adversarial/promotion_candidates_report.json --fail-on-confirmed-regressions; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

test-adversarial-report-diff: ## Compare baseline/candidate adversarial reports and emit run-delta artifact
	@echo "$(CYAN)🧪 Rendering adversarial report diff artifact...$(NC)"
	@BASELINE="$${ADVERSARIAL_DIFF_BASELINE_PATH:-scripts/tests/adversarial/latest_report.baseline.json}"; \
	CANDIDATE="$${ADVERSARIAL_DIFF_CANDIDATE_PATH:-scripts/tests/adversarial/latest_report.json}"; \
	OUTPUT="$${ADVERSARIAL_DIFF_OUTPUT_PATH:-scripts/tests/adversarial/adversarial_report_diff.json}"; \
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
	@python3 scripts/tests/adversarial_container_runner.py --mode isolation --report scripts/tests/adversarial/container_isolation_report.json

test-adversarial-container-blackbox: ## Run complementary containerized black-box adversary lane (scheduled/manual; non-blocking for release)
	@echo "$(CYAN)🧪 Running adversarial container black-box worker...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-preflight || exit 1
	@if $(MAKE) --no-print-directory spin-wait-ready; then \
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY="$(SHUMA_API_KEY)" SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" \
			python3 scripts/tests/adversarial_container_runner.py --mode blackbox --report scripts/tests/adversarial/container_blackbox_report.json; \
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
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) ./scripts/tests/run_dashboard_e2e.sh --grep "adversary sim (global toggle drives orchestration control lifecycle state|toggle emits fresh telemetry visible in monitoring raw feed)"; \
	else \
		echo "$(RED)❌ Error: Spin server not ready$(NC)"; \
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

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
	@rm -rf .spin
	@rm -rf playwright-report test-results
	@rm -f src/*.wasm
	@echo "$(GREEN)✅ Clean complete$(NC)"

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
	@echo "  SHUMA_ADMIN_CONFIG_WRITE_ENABLED"
	@echo "  SHUMA_KV_STORE_FAIL_OPEN"
	@echo "  SHUMA_ENFORCE_HTTPS"
	@echo "  SHUMA_DEBUG_HEADERS"
	@echo "  SHUMA_RUNTIME_ENV"
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
	@grep -h -E '^(stop|status|clean|logs|env-help|telemetry-clean|adversary-sim-supervisor|api-key-generate|gen-admin-api-key|api-key-show|api-key-rotate|api-key-validate|deploy-env-validate|help|remote-use|remote-update|remote-start|remote-stop|remote-status|remote-logs|remote-open-dashboard):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
