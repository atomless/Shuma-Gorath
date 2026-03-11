#!/bin/bash
# setup-runtime.sh - Runtime-only setup for single-host production operators
#
# Usage: make setup-runtime
#
# Installs only runtime prerequisites:
#   - Rust/Cargo via rustup
#   - wasm32-wasip1 target
#   - Fermyon Spin CLI
#   - local env bootstrap + KV seed prerequisites
#
# This intentionally skips contributor/dev-only tooling (Node/pnpm/Playwright).

set -euo pipefail

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

info() { echo -e "${CYAN}INFO${NC} $1"; }
success() { echo -e "${GREEN}PASS${NC} $1"; }
warn() { echo -e "${YELLOW}WARN${NC} $1"; }
error() { echo -e "${RED}FAIL${NC} $1"; exit 1; }

ENV_LOCAL_FILE=".env.local"
DEFAULTS_FILE="config/defaults.env"

generate_hex_secret() {
    local bytes="${1:-32}"
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -hex "$bytes"
    else
        od -An -N"$bytes" -tx1 /dev/urandom | tr -d ' \n'
    fi
}

read_env_local_value() {
    local key="$1"
    local raw=""
    if [[ -f "$ENV_LOCAL_FILE" ]]; then
        raw="$(grep -E "^${key}=" "$ENV_LOCAL_FILE" | tail -1 || true)"
    fi
    raw="${raw#*=}"
    if [[ ${#raw} -ge 2 ]]; then
        if [[ "${raw:0:1}" == "\"" && "${raw: -1}" == "\"" ]]; then
            raw="${raw:1:${#raw}-2}"
        elif [[ "${raw:0:1}" == "'" && "${raw: -1}" == "'" ]]; then
            raw="${raw:1:${#raw}-2}"
        fi
    fi
    printf '%s' "$raw"
}

upsert_env_local_value() {
    local key="$1"
    local value="$2"
    local tmp_file
    tmp_file="$(mktemp)"
    if [[ -f "$ENV_LOCAL_FILE" ]] && grep -q -E "^${key}=" "$ENV_LOCAL_FILE"; then
        awk -v target_key="$key" -v target_value="$value" '
            $0 ~ ("^" target_key "=") { print target_key "=" target_value; next }
            { print }
        ' "$ENV_LOCAL_FILE" > "$tmp_file"
    else
        if [[ -f "$ENV_LOCAL_FILE" ]]; then
            cat "$ENV_LOCAL_FILE" > "$tmp_file"
        fi
        printf '%s=%s\n' "$key" "$value" >> "$tmp_file"
    fi
    mv "$tmp_file" "$ENV_LOCAL_FILE"
}

normalize_env_local_unquoted_style() {
    local tmp_file
    tmp_file="$(mktemp)"
    awk '
        BEGIN { single_quote = sprintf("%c", 39) }
        /^[A-Za-z_][A-Za-z0-9_]*=/ {
            key = substr($0, 1, index($0, "=") - 1)
            value = substr($0, index($0, "=") + 1)
            if (length(value) >= 2) {
                if (substr(value, 1, 1) == "\"" && substr(value, length(value), 1) == "\"") {
                    value = substr(value, 2, length(value) - 2)
                } else if (substr(value, 1, 1) == single_quote && substr(value, length(value), 1) == single_quote) {
                    value = substr(value, 2, length(value) - 2)
                }
            }
            print key "=" value
            next
        }
        { print }
    ' "$ENV_LOCAL_FILE" > "$tmp_file"
    mv "$tmp_file" "$ENV_LOCAL_FILE"
}

install_sqlite3_with_apt() {
    if command -v sudo >/dev/null 2>&1; then
        if sudo -n true >/dev/null 2>&1; then
            sudo -n apt-get update -y
            sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3
        else
            if [[ ! -t 0 ]]; then
                error "sqlite3 is required for config-seed. Re-run make setup-runtime in an interactive terminal so sudo can install it."
            fi
            sudo apt-get update -y
            sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3
        fi
    else
        apt-get update -y
        env DEBIAN_FRONTEND=noninteractive apt-get install -y sqlite3
    fi
}

sqlite3_is_usable() {
    command -v sqlite3 >/dev/null 2>&1 && sqlite3 --version >/dev/null 2>&1
}

ensure_sqlite3_available() {
    if sqlite3_is_usable; then
        success "sqlite3 available: $(sqlite3 --version | head -1)"
        return
    fi

    if command -v apt-get >/dev/null 2>&1; then
        info "Installing sqlite3..."
        install_sqlite3_with_apt
    elif command -v brew >/dev/null 2>&1; then
        info "Installing sqlite3 via Homebrew..."
        brew install sqlite
    else
        error "sqlite3 is required for config-seed and no supported installer (apt-get or brew) is available."
    fi

    if ! sqlite3_is_usable; then
        error "sqlite3 is still unavailable after install attempt."
    fi
    success "sqlite3 available: $(sqlite3 --version | head -1)"
}

ensure_env_local_file() {
    if [[ ! -f "$ENV_LOCAL_FILE" ]]; then
        info "Creating $ENV_LOCAL_FILE for runtime overrides..."
        cat > "$ENV_LOCAL_FILE" <<EOF
# Local runtime overrides (gitignored)
# Created by \`make setup-runtime\`. Edit values for your environment.
SHUMA_API_KEY=${SHUMA_API_KEY:-}
SHUMA_ADMIN_READONLY_API_KEY=${SHUMA_ADMIN_READONLY_API_KEY:-}
SHUMA_JS_SECRET=${SHUMA_JS_SECRET:-}
SHUMA_POW_SECRET=${SHUMA_POW_SECRET:-}
SHUMA_CHALLENGE_SECRET=${SHUMA_CHALLENGE_SECRET:-}
SHUMA_MAZE_PREVIEW_SECRET=${SHUMA_MAZE_PREVIEW_SECRET:-}
SHUMA_FORWARDED_IP_SECRET=${SHUMA_FORWARDED_IP_SECRET:-}
SHUMA_HEALTH_SECRET=${SHUMA_HEALTH_SECRET:-}
SHUMA_ADMIN_IP_ALLOWLIST=${SHUMA_ADMIN_IP_ALLOWLIST:-}
SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE=${SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE:-}
SHUMA_EVENT_LOG_RETENTION_HOURS=${SHUMA_EVENT_LOG_RETENTION_HOURS:-}
SHUMA_ADMIN_CONFIG_WRITE_ENABLED=${SHUMA_ADMIN_CONFIG_WRITE_ENABLED:-}
SHUMA_KV_STORE_FAIL_OPEN=${SHUMA_KV_STORE_FAIL_OPEN:-}
SHUMA_ENFORCE_HTTPS=${SHUMA_ENFORCE_HTTPS:-}
SHUMA_DEBUG_HEADERS=${SHUMA_DEBUG_HEADERS:-}
SHUMA_RUNTIME_ENV=${SHUMA_RUNTIME_ENV:-}
SHUMA_ADVERSARY_SIM_AVAILABLE=${SHUMA_ADVERSARY_SIM_AVAILABLE:-}
SHUMA_SIM_TELEMETRY_SECRET=${SHUMA_SIM_TELEMETRY_SECRET:-}
SHUMA_FRONTIER_OPENAI_API_KEY=${SHUMA_FRONTIER_OPENAI_API_KEY:-}
SHUMA_FRONTIER_ANTHROPIC_API_KEY=${SHUMA_FRONTIER_ANTHROPIC_API_KEY:-}
SHUMA_FRONTIER_GOOGLE_API_KEY=${SHUMA_FRONTIER_GOOGLE_API_KEY:-}
SHUMA_FRONTIER_XAI_API_KEY=${SHUMA_FRONTIER_XAI_API_KEY:-}
SHUMA_FRONTIER_OPENAI_MODEL=${SHUMA_FRONTIER_OPENAI_MODEL:-}
SHUMA_FRONTIER_ANTHROPIC_MODEL=${SHUMA_FRONTIER_ANTHROPIC_MODEL:-}
SHUMA_FRONTIER_GOOGLE_MODEL=${SHUMA_FRONTIER_GOOGLE_MODEL:-}
SHUMA_FRONTIER_XAI_MODEL=${SHUMA_FRONTIER_XAI_MODEL:-}
SHUMA_ENTERPRISE_MULTI_INSTANCE=${SHUMA_ENTERPRISE_MULTI_INSTANCE:-}
SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED=${SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED:-}
SHUMA_RATE_LIMITER_REDIS_URL=${SHUMA_RATE_LIMITER_REDIS_URL:-}
SHUMA_BAN_STORE_REDIS_URL=${SHUMA_BAN_STORE_REDIS_URL:-}
SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN=${SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN:-}
SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH=${SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH:-}
SHUMA_GATEWAY_UPSTREAM_ORIGIN=${SHUMA_GATEWAY_UPSTREAM_ORIGIN:-}
SHUMA_GATEWAY_DEPLOYMENT_PROFILE=${SHUMA_GATEWAY_DEPLOYMENT_PROFILE:-}
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=${SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL:-}
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS=${SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS:-}
SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST=${SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST:-}
SHUMA_GATEWAY_PUBLIC_AUTHORITIES=${SHUMA_GATEWAY_PUBLIC_AUTHORITIES:-}
SHUMA_GATEWAY_LOOP_MAX_HOPS=${SHUMA_GATEWAY_LOOP_MAX_HOPS:-}
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=${SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED:-}
SHUMA_GATEWAY_ORIGIN_AUTH_MODE=${SHUMA_GATEWAY_ORIGIN_AUTH_MODE:-}
SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME=${SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME:-}
SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE=${SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE:-}
SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS=${SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS:-}
SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS=${SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS:-}
SHUMA_GATEWAY_TLS_STRICT=${SHUMA_GATEWAY_TLS_STRICT:-}
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=${SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED:-}
EOF
    fi
    chmod 600 "$ENV_LOCAL_FILE" 2>/dev/null || true
}

ensure_local_dev_secret() {
    local key="$1"
    local bytes="$2"
    local current_value=""
    local should_generate=0

    current_value="$(read_env_local_value "$key")"
    case "$key" in
        SHUMA_API_KEY)
            case "$current_value" in
                ""|changeme-dev-only-api-key|changeme-supersecret|changeme-prod-api-key)
                    should_generate=1
                    ;;
            esac
            ;;
        SHUMA_JS_SECRET)
            case "$current_value" in
                ""|changeme-dev-only-js-secret|changeme-js-secret|changeme-prod-js-secret)
                    should_generate=1
                    ;;
            esac
            ;;
        SHUMA_FORWARDED_IP_SECRET)
            case "$current_value" in
                ""|changeme-dev-only-ip-secret|changeme-prod-forwarded-ip-secret)
                    should_generate=1
                    ;;
            esac
            ;;
        SHUMA_SIM_TELEMETRY_SECRET)
            case "$current_value" in
                ""|changeme-dev-only-sim-telemetry-secret)
                    should_generate=1
                    ;;
            esac
            ;;
    esac

    if [[ "$should_generate" -eq 1 ]]; then
        upsert_env_local_value "$key" "$(generate_hex_secret "$bytes")"
    fi
}

ensure_env_local_default_from_defaults() {
    local key="$1"
    local current_value=""
    local default_value=""

    current_value="$(read_env_local_value "$key")"
    default_value="${!key:-}"
    if [[ -z "$current_value" ]]; then
        upsert_env_local_value "$key" "$default_value"
    fi
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║    WASM Bot Defence - Runtime Setup              ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"

if [[ ! -f "$DEFAULTS_FILE" ]]; then
    error "Missing ${DEFAULTS_FILE}. Cannot initialize runtime defaults."
fi
# shellcheck disable=SC1090
set -a
source "$DEFAULTS_FILE"
set +a

if [[ "$(uname)" != "Darwin" ]]; then
    warn "This script was tuned on macOS; adapt package/bootstrap tooling as needed."
fi

if command -v rustc >/dev/null 2>&1; then
    success "Rust installed: $(rustc --version)"
else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    success "Rust installed via rustup"
fi

if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
fi

if [[ -f "$HOME/.cargo/env" ]]; then
    CARGO_ENV_LINE='source "$HOME/.cargo/env"'
    PROFILE_FILES=(
        "$HOME/.zprofile"
        "$HOME/.zshrc"
        "$HOME/.bash_profile"
        "$HOME/.bashrc"
        "$HOME/.profile"
    )
    FOUND_PROFILE=0
    for PROFILE in "${PROFILE_FILES[@]}"; do
        if [[ -f "$PROFILE" ]]; then
            FOUND_PROFILE=1
            if ! grep -Fq "$CARGO_ENV_LINE" "$PROFILE"; then
                echo "$CARGO_ENV_LINE" >> "$PROFILE"
            fi
        fi
    done
    if [[ "$FOUND_PROFILE" -eq 0 ]]; then
        echo "$CARGO_ENV_LINE" >> "$HOME/.profile"
    fi
fi

if rustup target list --installed | grep -q "wasm32-wasip1"; then
    success "wasm32-wasip1 target already installed"
else
    info "Adding wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
    success "wasm32-wasip1 target installed"
fi

if command -v spin >/dev/null 2>&1; then
    success "Spin installed: $(spin --version | head -1)"
else
    info "Installing Fermyon Spin..."
    SPIN_INSTALL_DIR="/usr/local/bin"
    TMP_SPIN_DIR="$(mktemp -d /tmp/shuma-gorath-spin.XXXXXX)"
    cleanup_spin_tmp() { rm -rf "$TMP_SPIN_DIR"; }
    trap cleanup_spin_tmp EXIT

    (
        cd "$TMP_SPIN_DIR"
        curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

        if [[ ! -f "spin" ]]; then
            error "Spin installer did not produce a 'spin' binary in $TMP_SPIN_DIR"
        fi

        if [[ -w "$SPIN_INSTALL_DIR" ]]; then
            mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"
        else
            if ! command -v sudo >/dev/null 2>&1; then
                error "sudo not available; cannot move spin into $SPIN_INSTALL_DIR"
            fi

            if sudo -n true >/dev/null 2>&1; then
                if ! sudo -n /bin/mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"; then
                    error "Failed to move spin into $SPIN_INSTALL_DIR with passwordless sudo."
                fi
            else
                if [[ ! -t 0 ]]; then
                    error "This step needs sudo to move spin into $SPIN_INSTALL_DIR. Re-run make setup-runtime in an interactive terminal."
                fi

                if ! sudo /bin/mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"; then
                    error "Failed to move spin into $SPIN_INSTALL_DIR. Re-run make setup-runtime and authorize sudo."
                fi
            fi
        fi
    )
    success "Spin installed"
fi

ensure_env_local_file
ensure_local_dev_secret "SHUMA_API_KEY" 32
ensure_env_local_default_from_defaults "SHUMA_ADMIN_READONLY_API_KEY"
ensure_local_dev_secret "SHUMA_JS_SECRET" 32
ensure_local_dev_secret "SHUMA_FORWARDED_IP_SECRET" 32
ensure_local_dev_secret "SHUMA_SIM_TELEMETRY_SECRET" 32
ensure_env_local_default_from_defaults "SHUMA_POW_SECRET"
ensure_env_local_default_from_defaults "SHUMA_CHALLENGE_SECRET"
ensure_env_local_default_from_defaults "SHUMA_MAZE_PREVIEW_SECRET"
ensure_env_local_default_from_defaults "SHUMA_HEALTH_SECRET"
ensure_env_local_default_from_defaults "SHUMA_ADMIN_IP_ALLOWLIST"
ensure_env_local_default_from_defaults "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE"
ensure_env_local_default_from_defaults "SHUMA_EVENT_LOG_RETENTION_HOURS"
ensure_env_local_default_from_defaults "SHUMA_MONITORING_RETENTION_HOURS"
ensure_env_local_default_from_defaults "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS"
ensure_env_local_default_from_defaults "SHUMA_ADMIN_CONFIG_WRITE_ENABLED"
ensure_env_local_default_from_defaults "SHUMA_KV_STORE_FAIL_OPEN"
ensure_env_local_default_from_defaults "SHUMA_ENFORCE_HTTPS"
ensure_env_local_default_from_defaults "SHUMA_DEBUG_HEADERS"
ensure_env_local_default_from_defaults "SHUMA_RUNTIME_ENV"
ensure_env_local_default_from_defaults "SHUMA_LOCAL_PROD_DIRECT_MODE"
ensure_env_local_default_from_defaults "SHUMA_ADVERSARY_SIM_AVAILABLE"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_OPENAI_API_KEY"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_ANTHROPIC_API_KEY"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_GOOGLE_API_KEY"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_XAI_API_KEY"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_OPENAI_MODEL"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_ANTHROPIC_MODEL"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_GOOGLE_MODEL"
ensure_env_local_default_from_defaults "SHUMA_FRONTIER_XAI_MODEL"
ensure_env_local_default_from_defaults "SHUMA_ENTERPRISE_MULTI_INSTANCE"
ensure_env_local_default_from_defaults "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_REDIS_URL"
ensure_env_local_default_from_defaults "SHUMA_BAN_STORE_REDIS_URL"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN"
ensure_env_local_default_from_defaults "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_UPSTREAM_ORIGIN"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_DEPLOYMENT_PROFILE"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_PUBLIC_AUTHORITIES"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_LOOP_MAX_HOPS"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_AUTH_MODE"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_TLS_STRICT"
ensure_env_local_default_from_defaults "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"
normalize_env_local_unquoted_style
ensure_sqlite3_available
sim_secret_value="$(read_env_local_value "SHUMA_SIM_TELEMETRY_SECRET")"
if [[ -z "$sim_secret_value" ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET is empty after setup-runtime. Re-run make setup-runtime."
fi
if [[ "$sim_secret_value" == "changeme-dev-only-sim-telemetry-secret" ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET is still a placeholder after setup-runtime."
fi
if [[ ! "$sim_secret_value" =~ ^[0-9a-fA-F]{64,}$ ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET must be hex and at least 64 chars after setup-runtime."
fi
success "Runtime env overrides are ready in $ENV_LOCAL_FILE"
success "Adversarial sim telemetry secret is configured and non-placeholder."

info "Seeding/backfilling KV tunables from config/defaults.env..."
make --no-print-directory config-seed
success "KV tunables are seeded"

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Runtime setup complete${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""
echo -n "  Rust:        "
rustc --version 2>/dev/null || echo "not found"
echo -n "  Cargo:       "
cargo --version 2>/dev/null || echo "not found"
echo -n "  WASM target: "
if rustup target list --installed | grep -q "wasm32-wasip1"; then
    echo "wasm32-wasip1"
else
    echo "missing"
fi
echo -n "  Spin:        "
spin --version 2>/dev/null | head -1 || echo "not found"
echo ""
echo "Next commands:"
echo "  make verify-runtime"
echo "  make build-runtime"
echo "  make smoke-single-host"
