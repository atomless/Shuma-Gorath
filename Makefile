.PHONY: dev local run run-prebuilt build build-runtime build-full-dev prod clean test test-unit unit-test test-integration integration-test test-adversarial-manifest test-adversarial-lane-contract test-adversarial-coverage-contract test-adversarial-sim-selftest test-adversarial-fast test-adversarial-smoke test-adversarial-abuse test-adversarial-akamai test-adversarial-coverage test-adversarial-soak test-adversarial-live test-adversarial-repeatability test-adversarial-promote-candidates test-adversarial-container-blackbox test-adversarial-container-isolation test-adversarial-frontier-attempt test-frontier-governance test-frontier-unavailability-policy test-ip-range-suggestions test-coverage test-dashboard test-dashboard-svelte-check test-dashboard-unit test-dashboard-budgets test-dashboard-budgets-strict test-dashboard-e2e seed-dashboard-data test-maze-benchmark spin-wait-ready smoke-single-host deploy deploy-profile-baseline deploy-self-hosted-minimal deploy-enterprise-akamai logs status stop help setup setup-runtime verify verify-runtime config-seed dashboard-build env-help api-key-generate gen-admin-api-key api-key-show api-key-rotate api-key-validate deploy-env-validate

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

# Normalize optional quoted values from .env.local (handles KEY=value and KEY="value")
strip_wrapping_quotes = $(patsubst "%",%,$(patsubst '%',%,$(strip $(1))))
SHUMA_API_KEY := $(call strip_wrapping_quotes,$(SHUMA_API_KEY))
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
SHUMA_ADVERSARY_SIM_AVAILABLE := $(call strip_wrapping_quotes,$(SHUMA_ADVERSARY_SIM_AVAILABLE))
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
SHUMA_RUNTIME_ENV := $(if $(strip $(SHUMA_RUNTIME_ENV)),$(SHUMA_RUNTIME_ENV),runtime-prod)
SHUMA_ADVERSARY_SIM_AVAILABLE := $(if $(strip $(SHUMA_ADVERSARY_SIM_AVAILABLE)),$(SHUMA_ADVERSARY_SIM_AVAILABLE),false)
SHUMA_FRONTIER_OPENAI_MODEL := $(if $(strip $(SHUMA_FRONTIER_OPENAI_MODEL)),$(SHUMA_FRONTIER_OPENAI_MODEL),gpt-5-mini)
SHUMA_FRONTIER_ANTHROPIC_MODEL := $(if $(strip $(SHUMA_FRONTIER_ANTHROPIC_MODEL)),$(SHUMA_FRONTIER_ANTHROPIC_MODEL),claude-3-5-haiku-latest)
SHUMA_FRONTIER_GOOGLE_MODEL := $(if $(strip $(SHUMA_FRONTIER_GOOGLE_MODEL)),$(SHUMA_FRONTIER_GOOGLE_MODEL),gemini-2.0-flash-lite)
SHUMA_FRONTIER_XAI_MODEL := $(if $(strip $(SHUMA_FRONTIER_XAI_MODEL)),$(SHUMA_FRONTIER_XAI_MODEL),grok-3-mini)
SHUMA_ENTERPRISE_MULTI_INSTANCE := $(if $(strip $(SHUMA_ENTERPRISE_MULTI_INSTANCE)),$(SHUMA_ENTERPRISE_MULTI_INSTANCE),false)
SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED := $(if $(strip $(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED)),$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED),false)
SHUMA_RATE_LIMITER_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_REDIS_URL))
SHUMA_BAN_STORE_REDIS_URL := $(call strip_wrapping_quotes,$(SHUMA_BAN_STORE_REDIS_URL))
SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN))
SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH := $(call strip_wrapping_quotes,$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH))

# Inject env-only runtime keys into Spin from .env.local / shell env.
# This list is the operator-facing copy surface for deploy-time env overrides.
SPIN_ENV_ONLY_BASE := --env SHUMA_API_KEY=$(SHUMA_API_KEY) --env SHUMA_ADMIN_READONLY_API_KEY=$(SHUMA_ADMIN_READONLY_API_KEY) --env SHUMA_JS_SECRET=$(SHUMA_JS_SECRET) --env SHUMA_POW_SECRET=$(SHUMA_POW_SECRET) --env SHUMA_CHALLENGE_SECRET=$(SHUMA_CHALLENGE_SECRET) --env SHUMA_MAZE_PREVIEW_SECRET=$(SHUMA_MAZE_PREVIEW_SECRET) --env SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) --env SHUMA_HEALTH_SECRET=$(SHUMA_HEALTH_SECRET) --env SHUMA_ADMIN_IP_ALLOWLIST=$(SHUMA_ADMIN_IP_ALLOWLIST) --env SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=$(SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE) --env SHUMA_EVENT_LOG_RETENTION_HOURS=$(SHUMA_EVENT_LOG_RETENTION_HOURS) --env SHUMA_KV_STORE_FAIL_OPEN=$(SHUMA_KV_STORE_FAIL_OPEN) --env SHUMA_ENFORCE_HTTPS=$(SHUMA_ENFORCE_HTTPS) --env SHUMA_RUNTIME_ENV=$(SHUMA_RUNTIME_ENV) --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(SHUMA_ADVERSARY_SIM_AVAILABLE) --env SHUMA_FRONTIER_OPENAI_API_KEY=$(SHUMA_FRONTIER_OPENAI_API_KEY) --env SHUMA_FRONTIER_ANTHROPIC_API_KEY=$(SHUMA_FRONTIER_ANTHROPIC_API_KEY) --env SHUMA_FRONTIER_GOOGLE_API_KEY=$(SHUMA_FRONTIER_GOOGLE_API_KEY) --env SHUMA_FRONTIER_XAI_API_KEY=$(SHUMA_FRONTIER_XAI_API_KEY) --env SHUMA_FRONTIER_OPENAI_MODEL=$(SHUMA_FRONTIER_OPENAI_MODEL) --env SHUMA_FRONTIER_ANTHROPIC_MODEL=$(SHUMA_FRONTIER_ANTHROPIC_MODEL) --env SHUMA_FRONTIER_GOOGLE_MODEL=$(SHUMA_FRONTIER_GOOGLE_MODEL) --env SHUMA_FRONTIER_XAI_MODEL=$(SHUMA_FRONTIER_XAI_MODEL) --env SHUMA_ENTERPRISE_MULTI_INSTANCE=$(SHUMA_ENTERPRISE_MULTI_INSTANCE) --env SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=$(SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED) --env SHUMA_RATE_LIMITER_REDIS_URL=$(SHUMA_RATE_LIMITER_REDIS_URL) --env SHUMA_BAN_STORE_REDIS_URL=$(SHUMA_BAN_STORE_REDIS_URL) --env SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=$(SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN) --env SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=$(SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH)
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
DEV_ADVERSARY_SIM_AVAILABLE ?= true
SPIN_DEV_OVERRIDES := --env SHUMA_DEBUG_HEADERS=$(DEV_DEBUG_HEADERS) --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) --env SHUMA_ADMIN_IP_ALLOWLIST=$(DEV_ADMIN_IP_ALLOWLIST) --env SHUMA_RUNTIME_ENV=$(DEV_RUNTIME_ENV) --env SHUMA_ADVERSARY_SIM_AVAILABLE=$(DEV_ADVERSARY_SIM_AVAILABLE)
SPIN_PROD_OVERRIDES := --env SHUMA_DEBUG_HEADERS=false --env SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false --env SHUMA_RUNTIME_ENV=runtime-prod --env SHUMA_ADVERSARY_SIM_AVAILABLE=false
SPIN_READY_TIMEOUT_SECONDS ?= 90
SHUMA_DASHBOARD_BUNDLE_MAX_TOTAL_BYTES ?= 352000
SHUMA_DASHBOARD_BUNDLE_MAX_JS_BYTES ?= 330000
SHUMA_DASHBOARD_BUNDLE_MAX_CSS_BYTES ?= 40000
SHUMA_DASHBOARD_BUNDLE_MAX_JS_CHUNK_BYTES ?= 150000
SHUMA_DASHBOARD_BUNDLE_MAX_CSS_ASSET_BYTES ?= 30000
SHUMA_DASHBOARD_BUNDLE_BUDGET_ENFORCE ?= 0
DEV_WATCH_IGNORES := -i '*.wasm' -i 'dist/wasm/shuma_gorath.wasm' -i '.spin/**' -i 'dashboard/.svelte-kit' -i 'dashboard/.svelte-kit/**' -i 'dashboard/.vite' -i 'dashboard/.vite/**'

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

config-seed: ## Seed KV tunable config from config/defaults.env (create + backfill missing keys)
	@./scripts/config_seed.sh

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
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@echo "$(YELLOW)⚡ Startup rebuild override: DEV_FORCE_REBUILD=$${DEV_FORCE_REBUILD:-0}$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@$(MAKE) --no-print-directory config-seed >/dev/null
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
	@pkill -x spin 2>/dev/null || true
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
		-s '$(MAKE) --no-print-directory config-seed >/dev/null 2>&1; $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1; pkill -x spin 2>/dev/null || true; SPIN_ALWAYS_BUILD=0 spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000'

dev-closed: ## Build and run with file watching and SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)
	@echo "$(CYAN)🚨 Starting development server with SHUMA_KV_STORE_FAIL_OPEN=false (fail-closed)...$(NC)"
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@echo "$(YELLOW)⚡ Startup rebuild override: DEV_FORCE_REBUILD=$${DEV_FORCE_REBUILD:-0}$(NC)"
	@echo "$(CYAN)👀 Watching src/*.rs, dashboard/*, and spin.toml for changes... (Ctrl+C to stop)$(NC)"
	@$(MAKE) --no-print-directory config-seed >/dev/null
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
	@pkill -x spin 2>/dev/null || true
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
		-s '$(MAKE) --no-print-directory config-seed >/dev/null 2>&1; $(MAKE) --no-print-directory dashboard-build >/dev/null 2>&1; pkill -x spin 2>/dev/null || true; SPIN_ALWAYS_BUILD=0 spin up --direct-mounts $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --env SHUMA_KV_STORE_FAIL_OPEN=false --listen 127.0.0.1:3000'

local: dev ## Alias for dev

run: ## Build once and run (no file watching)
	@echo "$(CYAN)🚀 Starting development server...$(NC)"
	@echo "$(YELLOW)⚙️  Effective dev flags: WRITE=$(DEV_ADMIN_CONFIG_WRITE_ENABLED) DEBUG_HEADERS=$(DEV_DEBUG_HEADERS)$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@$(MAKE) --no-print-directory config-seed >/dev/null
	@$(MAKE) --no-print-directory dashboard-build >/dev/null
	@pkill -x spin 2>/dev/null || true
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
	@spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000

run-prebuilt: ## Run Spin using prebuilt wasm (CI helper)
	@echo "$(CYAN)🚀 Starting prebuilt server...$(NC)"
	@echo "$(YELLOW)🔐 Local admin allowlist override: DEV_ADMIN_IP_ALLOWLIST='$(DEV_ADMIN_IP_ALLOWLIST)' (empty by default)$(NC)"
	@$(MAKE) --no-print-directory config-seed >/dev/null
	@$(MAKE) --no-print-directory dashboard-build >/dev/null
	@pkill -x spin 2>/dev/null || true
	@echo "$(YELLOW)📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html$(NC)"
	@echo "$(YELLOW)📈 Metrics:   http://127.0.0.1:3000/metrics$(NC)"
	@echo "$(YELLOW)❤️  Health:    http://127.0.0.1:3000/health$(NC)"
	@echo "$(YELLOW)🌀 Maze Preview: http://127.0.0.1:3000/admin/maze/preview (admin auth)$(NC)"
	@spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_DEV_OVERRIDES) --listen 127.0.0.1:3000

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

prod: build-runtime ## Build for production and start server
	@echo "$(CYAN)🚀 Starting production server...$(NC)"
	@$(MAKE) --no-print-directory config-seed >/dev/null
	@pkill -x spin 2>/dev/null || true
	@spin up $(SPIN_ENV_ONLY_BASE) $(SPIN_PROD_OVERRIDES) --listen 0.0.0.0:3000

deploy: build-runtime ## Deploy to Fermyon Cloud
	@$(MAKE) --no-print-directory api-key-validate
	@$(MAKE) --no-print-directory deploy-env-validate
	@echo "$(CYAN)☁️  Deploying to Fermyon Cloud...$(NC)"
	@spin cloud deploy
	@echo "$(GREEN)✅ Deployment complete!$(NC)"

deploy-profile-baseline: ## Profile wrapper baseline: seed config + runtime build
	@echo "$(CYAN)🔧 Running shared deployment baseline...$(NC)"
	@$(MAKE) --no-print-directory config-seed
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

#--------------------------
# Testing
#--------------------------

spin-wait-ready: ## Wait for the existing local Spin server to pass /health
	@SHUMA_FORWARDED_IP_SECRET="$(SHUMA_FORWARDED_IP_SECRET)" SHUMA_HEALTH_SECRET="$(SHUMA_HEALTH_SECRET)" ./scripts/tests/wait_for_spin_ready.sh --timeout-seconds "$(SPIN_READY_TIMEOUT_SECONDS)"

smoke-single-host: ## Run post-deploy single-host smoke checks (health/admin auth/metrics/challenge route)
	@./scripts/tests/smoke_single_host.sh

test: ## Run umbrella tests in series: unit, maze benchmark, integration, mandatory fast adversarial matrix, and dashboard e2e
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
	@echo "$(CYAN)Step 1/6: Rust Unit Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@./scripts/set_crate_type.sh rlib
	@cargo test || exit 1
	@echo ""
	@echo "$(CYAN)Step 2/6: Maze Asymmetry Benchmark Gate$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-maze-benchmark || exit 1
	@echo ""
	@echo "$(CYAN)Step 3/6: Integration Tests (Spin HTTP scenarios)$(NC)"
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
	@echo "$(CYAN)Step 4/6: Adversarial Fast Matrix (smoke + abuse + Akamai)$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-fast || exit 1
	@echo ""
	@echo "$(CYAN)Step 5/6: Dashboard E2E Smoke Tests$(NC)"
	@echo "$(CYAN)--------------------------------------------$(NC)"
	@$(MAKE) --no-print-directory test-dashboard-e2e || exit 1
	@echo ""
	@echo "$(CYAN)Step 6/6: Dashboard Seed Snapshot$(NC)"
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
		echo "$(YELLOW)   Start the server first: make dev$(NC)"; \
		exit 1; \
	fi

integration-test: test-integration ## Alias for Spin integration tests

test-adversarial-manifest: ## Validate adversarial simulation manifest and fixtures (no server required)
	@echo "$(CYAN)🧪 Validating adversarial simulation manifest...$(NC)"
	@python3 -m py_compile scripts/tests/adversarial_simulation_runner.py scripts/tests/adversarial_live_loop.py scripts/tests/adversarial_repeatability.py scripts/tests/adversarial_promote_candidates.py scripts/tests/adversarial_container_runner.py scripts/tests/adversarial_container/worker.py scripts/tests/frontier_lane_attempt.py scripts/tests/frontier_unavailability_policy.py scripts/tests/check_frontier_payload_artifacts.py scripts/tests/check_adversarial_lane_contract.py scripts/tests/check_adversarial_coverage_contract.py
	@python3 -m unittest scripts/tests/test_adversarial_simulation_runner.py scripts/tests/test_adversarial_live_loop.py scripts/tests/test_adversarial_repeatability.py scripts/tests/test_adversarial_promote_candidates.py scripts/tests/test_adversarial_container_runner.py scripts/tests/test_frontier_lane_and_governance.py scripts/tests/test_adversarial_lane_contract.py scripts/tests/test_adversarial_coverage_contract.py
	@$(MAKE) --no-print-directory test-adversarial-lane-contract
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile fast_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile abuse_regression --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile akamai_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v1.json --profile full_coverage --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile fast_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile abuse_regression --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile akamai_smoke --validate-only
	@python3 scripts/tests/adversarial_simulation_runner.py --manifest scripts/tests/adversarial/scenario_manifest.v2.json --profile full_coverage --validate-only

test-adversarial-lane-contract: ## Validate black-box lane capability contract parity across deterministic/container tooling
	@echo "$(CYAN)🧪 Validating adversarial lane capability contract...$(NC)"
	@python3 scripts/tests/check_adversarial_lane_contract.py

test-adversarial-coverage-contract: ## Validate full-coverage contract parity across plan, manifest, and runner
	@echo "$(CYAN)🧪 Validating adversarial coverage contract...$(NC)"
	@python3 scripts/tests/check_adversarial_coverage_contract.py

test-adversarial-sim-selftest: ## Run minimal deterministic simulator self-test harness (no server required)
	@echo "$(CYAN)🧪 Running adversarial simulator self-test harness...$(NC)"
	@python3 scripts/tests/adversarial_sim_selftest.py

test-adversarial-fast: ## Run mandatory fast adversarial matrix (smoke + abuse + Akamai profiles)
	@echo "$(CYAN)🧪 Running mandatory fast adversarial matrix...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-lane-contract || exit 1
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract || exit 1
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
	@$(MAKE) --no-print-directory test-adversarial-coverage-contract || exit 1
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

test-adversarial-soak: ## Run deep adversarial soak gate (full_coverage profile; intended for scheduled/manual CI)
	@echo "$(CYAN)🧪 Running deep adversarial soak profile...$(NC)"
	@$(MAKE) --no-print-directory test-adversarial-coverage

test-adversarial-live: ## Continuously run adversarial simulation profile for live operator monitoring (requires running server)
	@echo "$(CYAN)🧪 Running adversarial live simulation loop...$(NC)"
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

test-adversarial-repeatability: ## Run deterministic repeatability gate across smoke/abuse/coverage profiles (N=3)
	@echo "$(CYAN)🧪 Running adversarial repeatability gate...$(NC)"
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

test-adversarial-container-isolation: ## Validate complementary container black-box isolation contract (scheduled/manual lane; Docker required)
	@echo "$(CYAN)🧪 Running adversarial container isolation conformance...$(NC)"
	@python3 scripts/tests/adversarial_container_runner.py --mode isolation --report scripts/tests/adversarial/container_isolation_report.json

test-adversarial-container-blackbox: ## Run complementary containerized black-box adversary lane (scheduled/manual; non-blocking for release)
	@echo "$(CYAN)🧪 Running adversarial container black-box worker...$(NC)"
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
		SHUMA_BASE_URL=http://127.0.0.1:3000 SHUMA_API_KEY=$(SHUMA_API_KEY) SHUMA_FORWARDED_IP_SECRET=$(SHUMA_FORWARDED_IP_SECRET) corepack pnpm run test:dashboard:e2e; \
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
	@pkill -f "cargo-watch watch --poll -w src -w dashboard -w spin.toml" 2>/dev/null || true
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
	@echo "  SHUMA_ADVERSARY_SIM_AVAILABLE"
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

deploy-env-validate: ## Fail deployment when unsafe debug flags are enabled, admin allowlist is missing, admin edge limits are unconfirmed, API-key rotation is unconfirmed, or enterprise multi-instance state guardrails are unmet
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

#--------------------------
# Help
#--------------------------

help: ## Show this help message
	@echo "$(CYAN)WASM Bot Defence - Available Commands$(NC)"
	@echo ""
	@echo "$(GREEN)First-time Setup:$(NC)"
	@grep -h -E '^(setup|setup-runtime|verify|verify-runtime|config-seed):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Development:$(NC)"
	@grep -h -E '^(dev|local|run|build|build-runtime|build-full-dev):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Production:$(NC)"
	@grep -h -E '^(prod|deploy|deploy-profile-baseline|deploy-self-hosted-minimal|deploy-enterprise-akamai):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Testing:$(NC)"
	@grep -h -E '^(test.*|smoke-single-host):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Utilities:$(NC)"
	@grep -h -E '^(stop|status|clean|logs|env-help|api-key-generate|gen-admin-api-key|api-key-show|api-key-rotate|api-key-validate|deploy-env-validate|help):.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-25s %s\n", $$1, $$2}'
