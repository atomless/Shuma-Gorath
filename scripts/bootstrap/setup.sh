#!/bin/bash
# setup.sh - One-command setup for WASM Bot Defence development
#
# Usage: make setup
#
# This script installs all required dependencies for macOS:
#   - Homebrew (if missing)
#   - Rust/Cargo (via rustup)
#   - wasm32-wasip1 target
#   - Fermyon Spin CLI
#   - cargo-watch (for file watching)
#   - Node.js + corepack (for dashboard toolchain)
#   - pnpm dependencies from lockfile
#   - Playwright Chromium runtime (for dashboard e2e)
#
# After setup, run: make dev

set -e

# Colors
GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

info() { echo -e "${CYAN}ℹ️  $1${NC}"; }
success() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
error() { echo -e "${RED}❌ $1${NC}"; exit 1; }

# shellcheck disable=SC1091
source "./scripts/bootstrap/scrapling_runtime.sh"

ENV_LOCAL_FILE=".env.local"
DEFAULTS_FILE="config/defaults.env"

generate_hex_secret() {
    local bytes="${1:-32}"
    if command -v openssl &> /dev/null; then
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
                error "sqlite3 is required for config-seed. Please run make setup in an interactive terminal so sudo can install it."
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
        info "Creating $ENV_LOCAL_FILE for local development overrides..."
        cat > "$ENV_LOCAL_FILE" <<EOF
# Local development overrides (gitignored)
# Created by `make setup`. Edit values for local development only.
SHUMA_API_KEY=${SHUMA_API_KEY:-}
LINODE_TOKEN=${LINODE_TOKEN:-}
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
SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET=${SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET:-}
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
GATEWAY_SURFACE_CATALOG_PATH=${GATEWAY_SURFACE_CATALOG_PATH:-}
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

is_interactive_terminal() {
    [[ -t 0 && -t 1 ]]
}

frontier_provider_label() {
    local provider="$1"
    case "$provider" in
        openai) echo "OpenAI" ;;
        anthropic) echo "Anthropic" ;;
        google) echo "Google" ;;
        xai) echo "xAI" ;;
        *) echo "$provider" ;;
    esac
}

frontier_provider_model_env_key() {
    local provider="$1"
    case "$provider" in
        openai) echo "SHUMA_FRONTIER_OPENAI_MODEL" ;;
        anthropic) echo "SHUMA_FRONTIER_ANTHROPIC_MODEL" ;;
        google) echo "SHUMA_FRONTIER_GOOGLE_MODEL" ;;
        xai) echo "SHUMA_FRONTIER_XAI_MODEL" ;;
        *) echo "" ;;
    esac
}

frontier_provider_api_key_env_key() {
    local provider="$1"
    case "$provider" in
        openai) echo "SHUMA_FRONTIER_OPENAI_API_KEY" ;;
        anthropic) echo "SHUMA_FRONTIER_ANTHROPIC_API_KEY" ;;
        google) echo "SHUMA_FRONTIER_GOOGLE_API_KEY" ;;
        xai) echo "SHUMA_FRONTIER_XAI_API_KEY" ;;
        *) echo "" ;;
    esac
}

frontier_provider_default_model() {
    local provider="$1"
    case "$provider" in
        openai) echo "${SHUMA_FRONTIER_OPENAI_MODEL:-gpt-5-mini}" ;;
        anthropic) echo "${SHUMA_FRONTIER_ANTHROPIC_MODEL:-claude-3-5-haiku-latest}" ;;
        google) echo "${SHUMA_FRONTIER_GOOGLE_MODEL:-gemini-2.0-flash-lite}" ;;
        xai) echo "${SHUMA_FRONTIER_XAI_MODEL:-grok-3-mini}" ;;
        *) echo "" ;;
    esac
}

validate_frontier_provider_key() {
    local provider="$1"
    local api_key="$2"
    local timeout_seconds="${3:-8}"
    local response_file
    response_file="$(mktemp)"
    local curl_exit=0
    local http_code=""

    case "$provider" in
        openai)
            http_code="$(curl -sS -m "$timeout_seconds" -o "$response_file" -w "%{http_code}" \
                -H "Authorization: Bearer ${api_key}" \
                "https://api.openai.com/v1/models?limit=1")" || curl_exit=$?
            ;;
        anthropic)
            http_code="$(curl -sS -m "$timeout_seconds" -o "$response_file" -w "%{http_code}" \
                -H "x-api-key: ${api_key}" \
                -H "anthropic-version: 2023-06-01" \
                "https://api.anthropic.com/v1/models")" || curl_exit=$?
            ;;
        google)
            http_code="$(curl -sS -m "$timeout_seconds" -o "$response_file" -w "%{http_code}" \
                "https://generativelanguage.googleapis.com/v1beta/models?key=${api_key}")" || curl_exit=$?
            ;;
        xai)
            http_code="$(curl -sS -m "$timeout_seconds" -o "$response_file" -w "%{http_code}" \
                -H "Authorization: Bearer ${api_key}" \
                "https://api.x.ai/v1/models?limit=1")" || curl_exit=$?
            ;;
        *)
            rm -f "$response_file"
            echo "unsupported_provider"
            return 0
            ;;
    esac

    rm -f "$response_file"

    if [[ "$curl_exit" -ne 0 ]]; then
        if [[ "$curl_exit" -eq 28 ]]; then
            echo "timeout"
        else
            echo "network_error"
        fi
        return 0
    fi

    if [[ "$http_code" =~ ^2[0-9][0-9]$ ]]; then
        echo "ok"
        return 0
    fi
    if [[ "$http_code" == "401" || "$http_code" == "403" ]]; then
        echo "auth_error"
        return 0
    fi
    echo "http_${http_code}"
}

configure_frontier_providers_optional() {
    if ! is_interactive_terminal; then
        info "Skipping optional frontier provider setup (non-interactive shell)."
        return 0
    fi

    local configure_choice=""
    printf "Configure frontier adversary providers now? [y/N]: "
    read -r configure_choice || true
    case "$(printf '%s' "$configure_choice" | tr '[:upper:]' '[:lower:]')" in
        y|yes) ;;
        *)
            info "Skipping optional frontier provider setup."
            return 0
            ;;
    esac

    if ! command -v curl >/dev/null 2>&1; then
        warn "curl is required for live frontier key validation; skipping optional frontier setup."
        return 0
    fi

    echo ""
    echo "Select frontier providers (comma-separated):"
    echo "  1) OpenAI"
    echo "  2) Anthropic"
    echo "  3) Google"
    echo "  4) xAI"
    local provider_selection=""
    printf "Provider selection [blank to skip]: "
    read -r provider_selection || true
    if [[ -z "$provider_selection" ]]; then
        warn "No frontier providers selected; continuing setup without frontier keys."
        return 0
    fi

    local selected_providers=()
    local token provider
    IFS=',' read -r -a provider_tokens <<< "$provider_selection"
    for token in "${provider_tokens[@]}"; do
        token="${token//[[:space:]]/}"
        provider=""
        case "$(printf '%s' "$token" | tr '[:upper:]' '[:lower:]')" in
            1|openai) provider="openai" ;;
            2|anthropic) provider="anthropic" ;;
            3|google) provider="google" ;;
            4|xai) provider="xai" ;;
        esac
        if [[ -n "$provider" ]]; then
            if [[ " ${selected_providers[*]} " != *" ${provider} "* ]]; then
                selected_providers+=("$provider")
            fi
        fi
    done

    if [[ "${#selected_providers[@]}" -eq 0 ]]; then
        warn "No valid provider selection was parsed; continuing without frontier keys."
        return 0
    fi

    local valid_provider_count=0
    local model_env_key api_env_key label
    local model_default model_input model_value
    local api_key_input validation_result retry_choice saved_for_provider

    for provider in "${selected_providers[@]}"; do
        label="$(frontier_provider_label "$provider")"
        model_env_key="$(frontier_provider_model_env_key "$provider")"
        api_env_key="$(frontier_provider_api_key_env_key "$provider")"
        model_default="$(read_env_local_value "$model_env_key")"
        if [[ -z "$model_default" ]]; then
            model_default="$(frontier_provider_default_model "$provider")"
        fi

        printf "%s model id [%s]: " "$label" "$model_default"
        read -r model_input || true
        model_value="$model_input"
        if [[ -z "$model_value" ]]; then
            model_value="$model_default"
        fi
        if [[ -n "$model_value" ]]; then
            upsert_env_local_value "$model_env_key" "$model_value"
        fi

        saved_for_provider=0
        while true; do
            printf "%s API key (hidden; leave blank to skip): " "$label"
            read -rs api_key_input || true
            echo ""
            if [[ -z "$api_key_input" ]]; then
                warn "$label key skipped."
                break
            fi

            info "Validating $label key with a bounded live provider probe..."
            validation_result="$(validate_frontier_provider_key "$provider" "$api_key_input" "8")"
            case "$validation_result" in
                ok)
                    upsert_env_local_value "$api_env_key" "$api_key_input"
                    success "$label key validated and saved to $ENV_LOCAL_FILE"
                    valid_provider_count=$((valid_provider_count + 1))
                    saved_for_provider=1
                    break
                    ;;
                auth_error)
                    warn "$label key was rejected by the provider (authentication failed)."
                    ;;
                timeout)
                    warn "$label key probe timed out (network/provider latency)."
                    ;;
                network_error)
                    warn "$label key probe failed due to a network error."
                    ;;
                *)
                    warn "$label key probe returned HTTP status ${validation_result#http_}."
                    ;;
            esac

            printf "Re-enter %s key? [y/N]: " "$label"
            read -r retry_choice || true
            case "$(printf '%s' "$retry_choice" | tr '[:upper:]' '[:lower:]')" in
                y|yes) ;;
                *) break ;;
            esac
        done

        if [[ "$saved_for_provider" -eq 0 ]]; then
            warn "$label is not configured for frontier calls."
        fi
    done

    if [[ "$valid_provider_count" -eq 0 ]]; then
        warn "No valid frontier provider keys were saved. Runs can continue without frontier calls."
    else
        success "Frontier provider setup complete with $valid_provider_count valid provider key(s)."
    fi
}

echo -e "${CYAN}"
echo "╔═══════════════════════════════════════════════════╗"
echo "║     WASM Bot Defence - Development Setup             ║"
echo "╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"
info "If setup needs sudo (for example, to install Spin), run this in an interactive terminal so you can authorize prompts."

if [[ ! -f "$DEFAULTS_FILE" ]]; then
    error "Missing ${DEFAULTS_FILE}. Cannot initialize local defaults."
fi
# shellcheck disable=SC1090
set -a
source "$DEFAULTS_FILE"
set +a

#--------------------------
# Check macOS
#--------------------------
if [[ "$(uname)" != "Darwin" ]]; then
    warn "This script is designed for macOS. You may need to adapt for your OS."
    warn "Linux users: Replace Homebrew commands with your package manager."
fi

#--------------------------
# Homebrew
#--------------------------
if command -v brew &> /dev/null; then
    success "Homebrew already installed"
else
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Add to PATH for Apple Silicon Macs
    if [[ -f "/opt/homebrew/bin/brew" ]]; then
        eval "$(/opt/homebrew/bin/brew shellenv)"
        echo 'eval "$(/opt/homebrew/bin/brew shellenv)"' >> ~/.zprofile
    fi
    success "Homebrew installed"
fi

#--------------------------
# Node.js / corepack
#--------------------------
if command -v node &> /dev/null; then
    NODE_VERSION="$(node --version)"
    success "Node.js already installed (${NODE_VERSION})"
else
    if command -v brew &> /dev/null; then
        info "Installing Node.js via Homebrew..."
        brew install node
        success "Node.js installed ($(node --version 2>/dev/null || echo unknown))"
    else
        error "Node.js not found and Homebrew is unavailable. Install Node.js 18+ and re-run make setup."
    fi
fi

if command -v corepack &> /dev/null; then
    success "corepack already available"
else
    if command -v npm &> /dev/null; then
        info "Installing corepack via npm..."
        npm install -g corepack
        success "corepack installed"
    else
        error "npm is unavailable; cannot install corepack. Install Node.js 18+ and re-run make setup."
    fi
fi
corepack enable > /dev/null 2>&1 || true

#--------------------------
# Rust / Cargo
#--------------------------
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    success "Rust already installed (v$RUST_VERSION)"
else
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed"
fi

# Ensure cargo is in PATH for this session
if [[ -f "$HOME/.cargo/env" ]]; then
    source "$HOME/.cargo/env"
fi

# Ensure future shells load Cargo (so make targets can find cargo)
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

#--------------------------
# WASM target
#--------------------------
if rustup target list --installed | grep -q "wasm32-wasip1"; then
    success "wasm32-wasip1 target already installed"
else
    info "Adding wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
    success "wasm32-wasip1 target installed"
fi

#--------------------------
# Fermyon Spin
#--------------------------
if command -v spin &> /dev/null; then
    SPIN_VERSION=$(spin --version | head -1)
    success "Spin already installed ($SPIN_VERSION)"
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
            if ! command -v sudo &> /dev/null; then
                error "sudo not available; cannot move spin into $SPIN_INSTALL_DIR"
            fi

            if sudo -n true >/dev/null 2>&1; then
                if ! sudo -n /bin/mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"; then
                    error "Failed to move spin into $SPIN_INSTALL_DIR with passwordless sudo."
                fi
            else
                if [[ ! -t 0 ]]; then
                    error "This step needs sudo to move spin into $SPIN_INSTALL_DIR. Please run make setup in an interactive terminal where you can authorize sudo."
                fi

                if ! sudo /bin/mv "$TMP_SPIN_DIR/spin" "$SPIN_INSTALL_DIR/spin"; then
                    error "Failed to move spin into $SPIN_INSTALL_DIR. Please re-run make setup in an interactive terminal and authorize sudo."
                fi
            fi
        fi
    )
    success "Spin installed"
fi

#--------------------------
# Scrapling worker runtime
#--------------------------
if scrapling_runtime_ready; then
    success "Scrapling worker runtime ready: $(scrapling_runtime_summary)"
else
    SCRAPLING_RUNTIME_PYTHON="$(scrapling_runtime_select_python || true)"
    if [[ -z "$SCRAPLING_RUNTIME_PYTHON" ]]; then
        if [[ "$(uname)" == "Darwin" ]] && command -v brew >/dev/null 2>&1; then
            info "Installing Homebrew ${SCRAPLING_RUNTIME_BREW_FORMULA} for the Scrapling worker runtime..."
            SCRAPLING_RUNTIME_PYTHON="$(scrapling_runtime_install_brew_python || true)"
        fi
    fi
    if [[ -z "$SCRAPLING_RUNTIME_PYTHON" ]]; then
        error "Scrapling worker runtime requires Python ${SCRAPLING_RUNTIME_MIN_MAJOR}.${SCRAPLING_RUNTIME_MIN_MINOR}+ and the repo-local ${SCRAPLING_RUNTIME_VENV_DIR}. Install a compatible python, then re-run make setup."
    fi

    info "Creating/updating ${SCRAPLING_RUNTIME_VENV_DIR} with ${SCRAPLING_RUNTIME_PACKAGE_SPEC}..."
    scrapling_runtime_install "$SCRAPLING_RUNTIME_PYTHON"
    if scrapling_runtime_ready; then
        success "Scrapling worker runtime ready: $(scrapling_runtime_summary)"
    else
        error "Scrapling worker runtime verification failed after install."
    fi
fi

#--------------------------
# cargo-watch
#--------------------------
if command -v cargo-watch &> /dev/null; then
    success "cargo-watch already installed"
else
    info "Installing cargo-watch (for file watching)..."
    cargo install cargo-watch
    success "cargo-watch installed"
fi

#--------------------------
# Dashboard JS dependencies + Playwright browser runtime
#--------------------------
dashboard_deps_ready() {
    [[ -d "node_modules/.pnpm" ]] && \
    [[ -x "node_modules/.bin/vite" ]] && \
    [[ -x "node_modules/.bin/svelte-check" ]] && \
    [[ -d "node_modules/svelte" ]] && \
    [[ -d "node_modules/@sveltejs/kit" ]] && \
    [[ -d "node_modules/@playwright/test" ]]
}

if dashboard_deps_ready; then
    success "Dashboard dependencies already installed"
else
    info "Refreshing dashboard dependencies from lockfile..."
    corepack pnpm install --offline --frozen-lockfile || corepack pnpm install --frozen-lockfile
    if dashboard_deps_ready; then
        success "Dashboard dependencies installed from lockfile"
    else
        error "Dashboard dependencies are incomplete after pnpm install."
    fi
fi

PLAYWRIGHT_BROWSER_CACHE="${PLAYWRIGHT_BROWSERS_PATH:-$(pwd)/.cache/ms-playwright}"
mkdir -p "$PLAYWRIGHT_BROWSER_CACHE"
PLAYWRIGHT_CHROMIUM_PATH="$(
    PLAYWRIGHT_BROWSERS_PATH="$PLAYWRIGHT_BROWSER_CACHE" \
    corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
)"
if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
    success "Playwright Chromium already installed ($PLAYWRIGHT_CHROMIUM_PATH)"
else
    info "Installing Playwright Chromium runtime into $PLAYWRIGHT_BROWSER_CACHE..."
    PLAYWRIGHT_BROWSERS_PATH="$PLAYWRIGHT_BROWSER_CACHE" corepack pnpm exec playwright install chromium
    PLAYWRIGHT_CHROMIUM_PATH="$(
        PLAYWRIGHT_BROWSERS_PATH="$PLAYWRIGHT_BROWSER_CACHE" \
        corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
    )"
    if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
        success "Playwright Chromium installed ($PLAYWRIGHT_CHROMIUM_PATH)"
    else
        error "Playwright Chromium install did not produce an executable browser runtime."
    fi
fi

#--------------------------
# Local dev secrets
#--------------------------
ensure_env_local_file
ensure_local_dev_secret "SHUMA_API_KEY" 32
ensure_env_local_default_from_defaults "SHUMA_ADMIN_READONLY_API_KEY"
ensure_local_dev_secret "SHUMA_JS_SECRET" 32
ensure_local_dev_secret "SHUMA_FORWARDED_IP_SECRET" 32
ensure_local_dev_secret "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET" 32
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
configure_frontier_providers_optional
normalize_env_local_unquoted_style
ensure_sqlite3_available
sim_secret_value="$(read_env_local_value "SHUMA_SIM_TELEMETRY_SECRET")"
if [[ -z "$sim_secret_value" ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET is empty after setup. Re-run make setup."
fi
if [[ "$sim_secret_value" == "changeme-dev-only-sim-telemetry-secret" ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET is still a placeholder after setup."
fi
if [[ ! "$sim_secret_value" =~ ^[0-9a-fA-F]{64,}$ ]]; then
    error "SHUMA_SIM_TELEMETRY_SECRET must be hex and at least 64 chars after setup."
fi
success "Local dev secrets are ready in $ENV_LOCAL_FILE"
success "Adversarial sim telemetry secret is configured and non-placeholder."

info "Seeding/backfilling KV tunables from config/defaults.env..."
make --no-print-directory config-seed
success "KV tunables are seeded"

#--------------------------
# Makefile sanity (dev target)
#--------------------------
if grep -q "cargo watch .* -x './scripts/set_crate_type.sh" Makefile 2>/dev/null; then
    warn "Makefile dev target uses cargo watch -x with a shell script; make dev will fail until updated."
fi

#--------------------------
# Verify installation
#--------------------------
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🎉 Setup complete! Installed versions:${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""

echo -n "  Rust:         "
rustc --version 2>/dev/null || echo "not found"

echo -n "  Cargo:        "
cargo --version 2>/dev/null || echo "not found"

echo -n "  WASM target:  "
if rustup target list --installed | grep -q "wasm32-wasip1"; then
    echo "wasm32-wasip1 ✓"
else
    echo "not installed"
fi

echo -n "  Spin:         "
spin --version 2>/dev/null | head -1 || echo "not found"

echo -n "  Scrapling:    "
if scrapling_runtime_ready; then
    scrapling_runtime_summary
else
    echo "not found"
fi

echo -n "  cargo-watch:  "
cargo-watch --version 2>/dev/null || echo "not found"

echo -n "  Node.js:      "
node --version 2>/dev/null || echo "not found"

echo -n "  corepack:     "
corepack --version 2>/dev/null || echo "not found"

echo -n "  pnpm:         "
corepack pnpm --version 2>/dev/null || echo "not found"

echo -n "  Browser cache:"
echo " ${PLAYWRIGHT_BROWSER_CACHE}"

echo -n "  Chromium:     "
PLAYWRIGHT_CHROMIUM_PATH="$(
    PLAYWRIGHT_BROWSERS_PATH="$PLAYWRIGHT_BROWSER_CACHE" \
    corepack pnpm exec node -e "const { chromium } = require('@playwright/test'); process.stdout.write(chromium.executablePath() || '');" 2>/dev/null || true
)"
if [[ -n "$PLAYWRIGHT_CHROMIUM_PATH" && -x "$PLAYWRIGHT_CHROMIUM_PATH" ]]; then
    echo "$PLAYWRIGHT_CHROMIUM_PATH"
else
    echo "not found"
fi

echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo -e "${GREEN}🚀 Ready to go! Run these commands:${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════${NC}"
echo ""
echo "  If commands are missing, open a new terminal or run: source ~/.zshrc"
echo ""
echo "  make dev      # Start dev server with file watching"
echo "  make run      # Build once and run (no watching)"
echo "  make test     # Run tests"
echo "  make api-key-show # Show local dashboard login key (SHUMA_API_KEY)"
echo "  make help     # Show all commands"
echo ""
echo -e "${YELLOW}📊 Dashboard: http://127.0.0.1:3000/dashboard/index.html${NC}"
echo ""
