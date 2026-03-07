#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RED="\033[0;31m"
NC="\033[0m"

info() {
  echo -e "${CYAN}INFO${NC} $1"
}

warn() {
  echo -e "${YELLOW}WARN${NC} $1"
}

success() {
  echo -e "${GREEN}PASS${NC} $1"
}

fail() {
  echo -e "${RED}FAIL${NC} $1"
  exit 1
}

usage() {
  cat <<'USAGE'
Usage:
  LINODE_TOKEN=<token> SHUMA_ADMIN_IP_ALLOWLIST=<cidr-or-list> \
    ./scripts/deploy_linode_one_shot.sh [options]

Required:
  LINODE_TOKEN                         Linode API token with Linodes read/write scope.
  SHUMA_ADMIN_IP_ALLOWLIST             Trusted admin source IP/CIDR(s), comma-separated.
  SHUMA_GATEWAY_UPSTREAM_ORIGIN        Existing origin in scheme://host[:port] form.
  SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED  Must be true for production cutover.
  SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED Must be true after clean preflight.
  SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED Must be true for production cutover.
  SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED Must be true for production cutover.
  GATEWAY_SURFACE_CATALOG_PATH         Path to reserved-route collision preflight catalog.

Options:
  --label <value>                      Linode instance label (default: shuma-<UTC timestamp>)
  --profile <small|medium|large>       Deployment profile (default: small)
  --region <value>                     Linode region slug (default: us-east)
  --type <value>                       Linode type/plan override (profile-derived by default)
  --image <value>                      Linode image slug (default: linode/ubuntu24.04)
  --existing-instance-id <id>          Use an already prepared Linode instance instead of creating a new VM
  --remote-name <name>                 Day-2 remote target name to write under .spin/remotes/
  --ssh-public-key-file <path>         SSH public key for first access (default: ~/.ssh/id_ed25519.pub, fallback ~/.ssh/id_rsa.pub)
  --ssh-private-key-file <path>        SSH private key paired with the public key (default: public key without .pub)
  --domain <fqdn>                      Required canonical public domain; enables Caddy reverse proxy/TLS
  --open-dashboard                    Open the deployed dashboard URL locally after a successful deploy
  --enable-caddy <auto|true|false>     Caddy mode (default: auto; canonical production path requires true)
  --preflight-only                     Run validations only; do not create infrastructure
  --destroy-on-failure                 Delete created Linode instance if deployment fails
  --help                               Show this help

Notes:
  - Run this from a cloned Shuma-Gorath repository root.
  - This path ships the exact local git HEAD as a release bundle; it does not clone from GitHub on the VM.
  - A dirty local worktree is allowed but warned about; only committed HEAD content is deployed.
  - The script runs local `make deploy-env-validate` before provisioning.
  - When --domain is set, ensure DNS A/AAAA already points to the new Linode before expecting TLS success.
USAGE
}

require_cmd() {
  local cmd="$1"
  command -v "$cmd" >/dev/null 2>&1 || fail "Missing required command: ${cmd}"
}

generate_hex_secret() {
  local bytes="${1:-32}"
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex "$bytes"
  else
    od -An -N"$bytes" -tx1 /dev/urandom | tr -d ' \n'
  fi
}

normalize_bool() {
  local value
  value="$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
  case "${value}" in
    1|true|yes|on) printf 'true' ;;
    0|false|no|off|"") printf 'false' ;;
    *)
      fail "Invalid boolean value: ${1}"
      ;;
  esac
}

require_true_env() {
  local key="$1"
  local value="$2"
  if [[ "$(normalize_bool "${value}")" != "true" ]]; then
    fail "${key} must be true for canonical Linode production deployment."
  fi
}

open_local_url() {
  local url="$1"
  local opener=""
  if command -v open >/dev/null 2>&1; then
    opener="open"
  elif command -v xdg-open >/dev/null 2>&1; then
    opener="xdg-open"
  fi

  if [[ -z "${opener}" ]]; then
    warn "No local URL opener found. Open manually: ${url}"
    return 0
  fi

  if "${opener}" "${url}" >/dev/null 2>&1; then
    success "Opened dashboard locally: ${url}"
  else
    warn "Failed to open dashboard automatically. Open manually: ${url}"
  fi
}

persist_local_env_values() {
  local env_file="$1"
  shift
  python3 - "$env_file" "$@" <<'PY_ENV_LOCAL'
from pathlib import Path
import sys

from scripts.deploy.local_env import upsert_env_value

env_path = Path(sys.argv[1])
pairs = sys.argv[2:]
for pair in pairs:
    key, value = pair.split("=", 1)
    if value:
        upsert_env_value(env_path, key, value)
PY_ENV_LOCAL
}

LINODE_API_URL="https://api.linode.com/v4"
LINODE_LABEL="shuma-$(date -u +%Y%m%d%H%M%S)"
LINODE_PROFILE="${LINODE_PROFILE:-small}"
LINODE_REGION="${LINODE_REGION:-us-east}"
LINODE_TYPE="${LINODE_TYPE:-}"
LINODE_IMAGE="${LINODE_IMAGE:-linode/ubuntu24.04}"
EXISTING_INSTANCE_ID="${EXISTING_INSTANCE_ID:-}"
REMOTE_NAME="${REMOTE_NAME:-}"
REMOTE_RECEIPTS_DIR="${REMOTE_RECEIPTS_DIR:-${REPO_ROOT}/.spin/remotes}"
ENV_LOCAL="${ENV_LOCAL:-${REPO_ROOT}/.env.local}"
SSH_PUBLIC_KEY_FILE="${SSH_PUBLIC_KEY_FILE:-}"
SSH_PRIVATE_KEY_FILE="${SSH_PRIVATE_KEY_FILE:-}"
DOMAIN_NAME="${DOMAIN_NAME:-}"
ENABLE_CADDY="${ENABLE_CADDY:-auto}"
PREFLIGHT_ONLY=0
DESTROY_ON_FAILURE=0
OPEN_DASHBOARD=0
TYPE_EXPLICIT=0
LABEL_EXPLICIT=0
PROFILE_EXPLICIT=0
REGION_EXPLICIT=0
IMAGE_EXPLICIT=0

LINODE_TOKEN="${LINODE_TOKEN:-}"
SHUMA_ADMIN_IP_ALLOWLIST="${SHUMA_ADMIN_IP_ALLOWLIST:-}"
SHUMA_GATEWAY_UPSTREAM_ORIGIN="${SHUMA_GATEWAY_UPSTREAM_ORIGIN:-}"
SHUMA_GATEWAY_DEPLOYMENT_PROFILE="${SHUMA_GATEWAY_DEPLOYMENT_PROFILE:-shared-server}"
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED="${SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED:-}"
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED="${SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED:-}"
SHUMA_GATEWAY_TLS_STRICT="${SHUMA_GATEWAY_TLS_STRICT:-true}"
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED="${SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED:-}"
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED="${SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED:-}"
GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_PATH:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --label)
      LINODE_LABEL="${2:-}"
      LABEL_EXPLICIT=1
      shift 2
      ;;
    --region)
      LINODE_REGION="${2:-}"
      REGION_EXPLICIT=1
      shift 2
      ;;
    --profile)
      LINODE_PROFILE="${2:-}"
      PROFILE_EXPLICIT=1
      shift 2
      ;;
    --type)
      LINODE_TYPE="${2:-}"
      TYPE_EXPLICIT=1
      shift 2
      ;;
    --image)
      LINODE_IMAGE="${2:-}"
      IMAGE_EXPLICIT=1
      shift 2
      ;;
    --existing-instance-id)
      EXISTING_INSTANCE_ID="${2:-}"
      shift 2
      ;;
    --remote-name)
      REMOTE_NAME="${2:-}"
      shift 2
      ;;
    --ssh-public-key-file)
      SSH_PUBLIC_KEY_FILE="${2:-}"
      shift 2
      ;;
    --ssh-private-key-file)
      SSH_PRIVATE_KEY_FILE="${2:-}"
      shift 2
      ;;
    --domain)
      DOMAIN_NAME="${2:-}"
      shift 2
      ;;
    --open-dashboard)
      OPEN_DASHBOARD=1
      shift
      ;;
    --enable-caddy)
      ENABLE_CADDY="${2:-}"
      shift 2
      ;;
    --preflight-only)
      PREFLIGHT_ONLY=1
      shift
      ;;
    --destroy-on-failure)
      DESTROY_ON_FAILURE=1
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "Unknown option: $1"
      ;;
  esac
done

if [[ -z "${LINODE_TOKEN}" ]]; then
  fail "LINODE_TOKEN is required."
fi
if [[ -z "${SHUMA_ADMIN_IP_ALLOWLIST}" ]]; then
  fail "SHUMA_ADMIN_IP_ALLOWLIST is required."
fi
if [[ -z "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}" ]]; then
  fail "SHUMA_GATEWAY_UPSTREAM_ORIGIN is required."
fi
if [[ ! -f "${GATEWAY_SURFACE_CATALOG_PATH}" ]]; then
  fail "GATEWAY_SURFACE_CATALOG_PATH must point to an existing catalog JSON."
fi

if [[ -n "${EXISTING_INSTANCE_ID}" ]]; then
  [[ "${LABEL_EXPLICIT}" -eq 0 ]] || fail "--label must not be used with --existing-instance-id."
  [[ "${PROFILE_EXPLICIT}" -eq 0 ]] || fail "--profile must not be used with --existing-instance-id."
  [[ "${REGION_EXPLICIT}" -eq 0 ]] || fail "--region must not be used with --existing-instance-id."
  [[ "${TYPE_EXPLICIT}" -eq 0 ]] || fail "--type must not be used with --existing-instance-id."
  [[ "${IMAGE_EXPLICIT}" -eq 0 ]] || fail "--image must not be used with --existing-instance-id."
  [[ "${DESTROY_ON_FAILURE}" -eq 0 ]] || fail "--destroy-on-failure must not be used with --existing-instance-id."
fi

LINODE_PROFILE_NORM="$(printf '%s' "${LINODE_PROFILE}" | tr '[:upper:]' '[:lower:]')"
case "${LINODE_PROFILE_NORM}" in
  small)
    if [[ "${TYPE_EXPLICIT}" -eq 0 ]]; then
      LINODE_TYPE="g6-nanode-1"
    fi
    ;;
  medium)
    if [[ "${TYPE_EXPLICIT}" -eq 0 ]]; then
      LINODE_TYPE="g6-standard-1"
    fi
    ;;
  large)
    if [[ "${TYPE_EXPLICIT}" -eq 0 ]]; then
      LINODE_TYPE="g6-standard-2"
    fi
    ;;
  *)
    fail "--profile must be one of small|medium|large"
    ;;
esac
[[ -n "${LINODE_TYPE}" ]] || fail "Linode type cannot be empty."

if [[ -z "${SSH_PUBLIC_KEY_FILE}" && -z "${EXISTING_INSTANCE_ID}" ]]; then
  for candidate in "$HOME/.ssh/id_ed25519.pub" "$HOME/.ssh/id_rsa.pub"; do
    if [[ -f "$candidate" ]]; then
      SSH_PUBLIC_KEY_FILE="$candidate"
      break
    fi
  done
fi

if [[ -z "${SSH_PRIVATE_KEY_FILE}" ]]; then
  if [[ -n "${SSH_PUBLIC_KEY_FILE}" ]]; then
    SSH_PRIVATE_KEY_FILE="${SSH_PUBLIC_KEY_FILE%.pub}"
  else
    for candidate in "$HOME/.ssh/id_ed25519" "$HOME/.ssh/id_rsa"; do
      if [[ -f "$candidate" ]]; then
        SSH_PRIVATE_KEY_FILE="$candidate"
        break
      fi
    done
  fi
fi
[[ -f "${SSH_PRIVATE_KEY_FILE}" ]] || fail "SSH private key file not found: ${SSH_PRIVATE_KEY_FILE}"
if [[ -z "${EXISTING_INSTANCE_ID}" ]]; then
  [[ -f "${SSH_PUBLIC_KEY_FILE}" ]] || fail "SSH public key file not found: ${SSH_PUBLIC_KEY_FILE}"
fi

ENABLE_CADDY_NORM="$(printf '%s' "${ENABLE_CADDY}" | tr '[:upper:]' '[:lower:]')"
case "${ENABLE_CADDY_NORM}" in
  auto)
    if [[ -n "${DOMAIN_NAME}" ]]; then
      ENABLE_CADDY_NORM="true"
    else
      ENABLE_CADDY_NORM="false"
    fi
    ;;
  true|false)
    ;;
  *)
    fail "--enable-caddy must be one of auto|true|false"
    ;;
esac

if [[ "${ENABLE_CADDY_NORM}" == "true" && -z "${DOMAIN_NAME}" ]]; then
  fail "--domain is required when Caddy mode is enabled."
fi
if [[ -z "${DOMAIN_NAME}" ]]; then
  fail "--domain is required for canonical Linode production deployment."
fi
if [[ "${ENABLE_CADDY_NORM}" != "true" ]]; then
  fail "Caddy/TLS must be enabled for canonical Linode production deployment."
fi

GATEWAY_DEPLOYMENT_PROFILE_NORM="$(printf '%s' "${SHUMA_GATEWAY_DEPLOYMENT_PROFILE}" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
if [[ "${GATEWAY_DEPLOYMENT_PROFILE_NORM}" != "shared-server" ]]; then
  fail "SHUMA_GATEWAY_DEPLOYMENT_PROFILE must be shared-server for Linode shared-host deployment."
fi
SHUMA_GATEWAY_DEPLOYMENT_PROFILE="shared-server"
require_true_env "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED" "${SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED}"
require_true_env "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED" "${SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED}"
require_true_env "SHUMA_GATEWAY_TLS_STRICT" "${SHUMA_GATEWAY_TLS_STRICT}"
require_true_env "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED" "${SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED}"
require_true_env "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED" "${SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED}"

for cmd in curl jq ssh scp python3 make git; do
  require_cmd "$cmd"
done

run_local_production_preflight() {
  local rendered_manifest
  rendered_manifest="$(mktemp)"
  TMP_FILES+=("${rendered_manifest}")
  python3 "${REPO_ROOT}/scripts/deploy/render_gateway_spin_manifest.py" \
    --manifest "${REPO_ROOT}/spin.toml" \
    --output "${rendered_manifest}" \
    --upstream-origin "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}" >/dev/null
  info "Running local production preflight (make deploy-env-validate)"
  SHUMA_ADMIN_IP_ALLOWLIST="${SHUMA_ADMIN_IP_ALLOWLIST}" \
  SHUMA_SPIN_MANIFEST="${rendered_manifest}" \
  SHUMA_GATEWAY_UPSTREAM_ORIGIN="${SHUMA_GATEWAY_UPSTREAM_ORIGIN}" \
  SHUMA_GATEWAY_DEPLOYMENT_PROFILE="${SHUMA_GATEWAY_DEPLOYMENT_PROFILE}" \
  SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED="${SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED}" \
  SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED="${SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED}" \
  SHUMA_GATEWAY_TLS_STRICT="${SHUMA_GATEWAY_TLS_STRICT}" \
  SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED="${SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED}" \
  SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED="${SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED}" \
  GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_PATH}" \
  SHUMA_RUNTIME_ENV=runtime-prod \
  SHUMA_DEBUG_HEADERS=false \
  make --no-print-directory deploy-env-validate >/dev/null
  success "Local production preflight passed"
}

# Keep long-running API operations deterministic by using explicit polling and optional cleanup.
INSTANCE_ID=""
INSTANCE_IPV4=""
INSTANCE_LABEL=""
TMP_FILES=()

cleanup() {
  local code="$?"
  for tmp in "${TMP_FILES[@]-}"; do
    [[ -n "$tmp" ]] && rm -f "$tmp" || true
  done

  if [[ "$code" -ne 0 && "$DESTROY_ON_FAILURE" -eq 1 && -n "$INSTANCE_ID" ]]; then
    warn "Deployment failed. Destroying Linode instance ${INSTANCE_ID} (--destroy-on-failure enabled)."
    if curl -sS -X DELETE \
      -H "Authorization: Bearer ${LINODE_TOKEN}" \
      "${LINODE_API_URL}/linode/instances/${INSTANCE_ID}" >/dev/null; then
      warn "Linode instance ${INSTANCE_ID} deleted."
    else
      warn "Failed to delete Linode instance ${INSTANCE_ID}; remove manually from Linode Cloud Manager."
    fi
  fi

  exit "$code"
}
trap cleanup EXIT

linode_api_json() {
  local method="$1"
  local path="$2"
  local payload="${3:-}"
  local tmp
  local status
  tmp="$(mktemp)"
  TMP_FILES+=("$tmp")

  if [[ -n "${payload}" ]]; then
    status="$(curl -sS -o "$tmp" -w "%{http_code}" -X "$method" \
      -H "Authorization: Bearer ${LINODE_TOKEN}" \
      -H "Content-Type: application/json" \
      "${LINODE_API_URL}${path}" \
      --data "$payload")"
  else
    status="$(curl -sS -o "$tmp" -w "%{http_code}" -X "$method" \
      -H "Authorization: Bearer ${LINODE_TOKEN}" \
      -H "Content-Type: application/json" \
      "${LINODE_API_URL}${path}")"
  fi

  if (( status < 200 || status > 299 )); then
    local reason
    reason="$(jq -r '[.errors[]?.reason] | map(select(. != null and . != "")) | join("; ")' "$tmp" 2>/dev/null || true)"
    if [[ -z "$reason" || "$reason" == "null" ]]; then
      reason="$(cat "$tmp")"
    fi
    fail "Linode API ${method} ${path} failed (HTTP ${status}): ${reason}"
  fi

  cat "$tmp"
}

ensure_linode_collection_contains_id() {
  local api_path="$1"
  local wanted_id="$2"
  local display_name="$3"
  local payload
  payload="$(linode_api_json GET "${api_path}")"
  if jq -e --arg wanted "$wanted_id" '.data[]? | select(.id == $wanted)' <<<"$payload" >/dev/null; then
    success "${display_name} validated: ${wanted_id}"
    return 0
  fi
  fail "${display_name} not found: ${wanted_id}"
}

load_existing_instance_details() {
  local details
  local status
  local ip

  details="$(linode_api_json GET "/linode/instances/${EXISTING_INSTANCE_ID}")"
  status="$(jq -r '.status // ""' <<<"${details}")"
  ip="$(jq -r '.ipv4[0] // ""' <<<"${details}")"
  INSTANCE_LABEL="$(jq -r '.label // ""' <<<"${details}")"

  [[ -n "${ip}" ]] || fail "Existing Linode instance ${EXISTING_INSTANCE_ID} does not have an IPv4 address."
  [[ "${status}" == "running" ]] || fail "Existing Linode instance ${EXISTING_INSTANCE_ID} is not running (status=${status})."

  INSTANCE_ID="${EXISTING_INSTANCE_ID}"
  INSTANCE_IPV4="${ip}"
  success "Existing Linode instance validated: id=${INSTANCE_ID} ip=${INSTANCE_IPV4}"
}

run_preflight_checks() {
  info "Running preflight checks (no infrastructure changes yet)"
  if [[ -n "${EXISTING_INSTANCE_ID}" ]]; then
    load_existing_instance_details
    info "Preflight summary"
    echo "  existing instance: ${INSTANCE_ID}"
    echo "  host ip:           ${INSTANCE_IPV4}"
    echo "  domain:            ${DOMAIN_NAME}"
    echo "  gateway:           ${SHUMA_GATEWAY_UPSTREAM_ORIGIN}"
    echo "  caddy:             ${ENABLE_CADDY_NORM}"
    return 0
  fi

  ensure_linode_collection_contains_id "/regions?page_size=500" "${LINODE_REGION}" "Linode region"
  ensure_linode_collection_contains_id "/linode/types?page_size=500" "${LINODE_TYPE}" "Linode type"

  local image_payload
  image_payload="$(linode_api_json GET "/images?page_size=500")"
  if jq -e --arg wanted "${LINODE_IMAGE}" '.data[]? | select(.id == $wanted)' <<<"$image_payload" >/dev/null; then
    success "Linode image validated: ${LINODE_IMAGE}"
  else
    warn "Linode image ${LINODE_IMAGE} not found in first image page; create request may still validate it server-side."
  fi

  info "Preflight summary"
  echo "  profile: ${LINODE_PROFILE_NORM}"
  echo "  region:  ${LINODE_REGION}"
  echo "  type:    ${LINODE_TYPE}"
  echo "  image:   ${LINODE_IMAGE}"
  echo "  domain:  ${DOMAIN_NAME}"
  echo "  gateway: ${SHUMA_GATEWAY_UPSTREAM_ORIGIN}"
  echo "  caddy:   ${ENABLE_CADDY_NORM}"
}

run_local_production_preflight
run_preflight_checks

if [[ "${PREFLIGHT_ONLY}" -eq 1 ]]; then
  success "Preflight-only mode complete. No Linode resources were created."
  exit 0
fi

RELEASE_ARCHIVE_FILE="$(mktemp "/tmp/shuma-release.XXXXXX.tar.gz")"
RELEASE_METADATA_FILE="$(mktemp "/tmp/shuma-release.XXXXXX.json")"
TMP_FILES+=("$RELEASE_ARCHIVE_FILE" "$RELEASE_METADATA_FILE")

info "Building exact local HEAD release bundle"
python3 "${REPO_ROOT}/scripts/deploy/build_linode_release_bundle.py" \
  --repo-root "${REPO_ROOT}" \
  --archive-output "${RELEASE_ARCHIVE_FILE}" \
  --metadata-output "${RELEASE_METADATA_FILE}" >/dev/null
RELEASE_COMMIT_SHA="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1], "r", encoding="utf-8"))["commit"])' "${RELEASE_METADATA_FILE}")"
RELEASE_DIRTY_WORKTREE="$(python3 -c 'import json,sys; print("true" if json.load(open(sys.argv[1], "r", encoding="utf-8"))["dirty_worktree"] else "false")' "${RELEASE_METADATA_FILE}")"
if [[ "${RELEASE_DIRTY_WORKTREE}" == "true" ]]; then
  warn "Local worktree is dirty. Deploying committed HEAD ${RELEASE_COMMIT_SHA} only."
else
  success "Prepared release bundle for commit ${RELEASE_COMMIT_SHA}"
fi

if [[ -z "${EXISTING_INSTANCE_ID}" ]]; then
  SSH_PUBLIC_KEY_CONTENT="$(tr -d '\r\n' < "${SSH_PUBLIC_KEY_FILE}")"
  [[ -n "${SSH_PUBLIC_KEY_CONTENT}" ]] || fail "SSH public key file is empty: ${SSH_PUBLIC_KEY_FILE}"

  ROOT_PASSWORD="$(generate_hex_secret 24)"
  CLOUD_INIT_CONTENT="$(cat <<EOF_CLOUD_INIT
#cloud-config
users:
  - name: shuma
    groups: sudo
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - ${SSH_PUBLIC_KEY_CONTENT}
disable_root: true
ssh_pwauth: false
package_update: true
EOF_CLOUD_INIT
)"

  CLOUD_INIT_B64="$(printf '%s' "${CLOUD_INIT_CONTENT}" | python3 -c 'import base64,sys; print(base64.b64encode(sys.stdin.buffer.read()).decode(), end="")')"

  CREATE_PAYLOAD="$(jq -n \
    --arg region "${LINODE_REGION}" \
    --arg type "${LINODE_TYPE}" \
    --arg image "${LINODE_IMAGE}" \
    --arg label "${LINODE_LABEL}" \
    --arg root_pass "${ROOT_PASSWORD}" \
    --arg user_data "${CLOUD_INIT_B64}" \
    '{region:$region,type:$type,image:$image,label:$label,root_pass:$root_pass,booted:true,metadata:{user_data:$user_data}}'
  )"

  info "Creating Linode instance label=${LINODE_LABEL} region=${LINODE_REGION} type=${LINODE_TYPE} image=${LINODE_IMAGE}"
  CREATE_RESPONSE="$(linode_api_json POST /linode/instances "${CREATE_PAYLOAD}")"
  INSTANCE_ID="$(jq -r '.id // empty' <<<"${CREATE_RESPONSE}")"
  INSTANCE_IPV4="$(jq -r '.ipv4[0] // empty' <<<"${CREATE_RESPONSE}")"
  INSTANCE_LABEL="${LINODE_LABEL}"
  [[ -n "${INSTANCE_ID}" ]] || fail "Linode API did not return an instance id."
  success "Instance created with id=${INSTANCE_ID}"

  poll_ready() {
    local attempt
    local details
    local status
    local ip
    for attempt in $(seq 1 90); do
      details="$(linode_api_json GET "/linode/instances/${INSTANCE_ID}")"
      status="$(jq -r '.status // ""' <<<"${details}")"
      ip="$(jq -r '.ipv4[0] // ""' <<<"${details}")"
      if [[ -n "${ip}" ]]; then
        INSTANCE_IPV4="${ip}"
      fi
      info "Instance status=${status} ip=${INSTANCE_IPV4:-pending} (attempt ${attempt}/90)"
      if [[ "${status}" == "running" && -n "${INSTANCE_IPV4}" ]]; then
        return 0
      fi
      sleep 5
    done
    return 1
  }

  poll_ready || fail "Timed out waiting for Linode instance to become running with IPv4."
  success "Linode instance is running at ${INSTANCE_IPV4}"
else
  info "Using existing Linode instance id=${INSTANCE_ID} ip=${INSTANCE_IPV4}"
fi

wait_for_ssh() {
  local attempt
  for attempt in $(seq 1 60); do
    if ssh -o BatchMode=yes \
      -o ConnectTimeout=5 \
      -o StrictHostKeyChecking=accept-new \
      -i "${SSH_PRIVATE_KEY_FILE}" \
      "shuma@${INSTANCE_IPV4}" true >/dev/null 2>&1; then
      return 0
    fi
    info "Waiting for SSH readiness (attempt ${attempt}/60)..."
    sleep 5
  done
  return 1
}

wait_for_ssh || fail "SSH did not become ready on ${INSTANCE_IPV4}."
success "SSH is reachable"

SHUMA_API_KEY_VALUE="${SHUMA_API_KEY:-$(generate_hex_secret 32)}"
SHUMA_JS_SECRET_VALUE="${SHUMA_JS_SECRET:-$(generate_hex_secret 32)}"
SHUMA_FORWARDED_IP_SECRET_VALUE="${SHUMA_FORWARDED_IP_SECRET:-$(generate_hex_secret 32)}"
SHUMA_HEALTH_SECRET_VALUE="${SHUMA_HEALTH_SECRET:-$(generate_hex_secret 32)}"
SHUMA_SIM_TELEMETRY_SECRET_VALUE="${SHUMA_SIM_TELEMETRY_SECRET:-$(generate_hex_secret 32)}"
SHUMA_ADMIN_CONFIG_WRITE_ENABLED_VALUE="${SHUMA_ADMIN_CONFIG_WRITE_ENABLED:-true}"
SHUMA_ADVERSARY_SIM_AVAILABLE_VALUE="${SHUMA_ADVERSARY_SIM_AVAILABLE:-true}"
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL_VALUE="false"
if [[ "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}" == http://* ]]; then
  SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL_VALUE="true"
fi

LOCAL_ENV_FILE="$(mktemp)"
TMP_FILES+=("$LOCAL_ENV_FILE")
cat >"${LOCAL_ENV_FILE}" <<EOF_ENV
SHUMA_API_KEY=${SHUMA_API_KEY_VALUE}
SHUMA_JS_SECRET=${SHUMA_JS_SECRET_VALUE}
SHUMA_FORWARDED_IP_SECRET=${SHUMA_FORWARDED_IP_SECRET_VALUE}
SHUMA_HEALTH_SECRET=${SHUMA_HEALTH_SECRET_VALUE}
SHUMA_SIM_TELEMETRY_SECRET=${SHUMA_SIM_TELEMETRY_SECRET_VALUE}
SHUMA_ADMIN_IP_ALLOWLIST=${SHUMA_ADMIN_IP_ALLOWLIST}
SHUMA_ADMIN_CONFIG_WRITE_ENABLED=${SHUMA_ADMIN_CONFIG_WRITE_ENABLED_VALUE}
SHUMA_DEBUG_HEADERS=false
SHUMA_RUNTIME_ENV=runtime-prod
SHUMA_ADVERSARY_SIM_AVAILABLE=${SHUMA_ADVERSARY_SIM_AVAILABLE_VALUE}
SHUMA_ENFORCE_HTTPS=true
SHUMA_KV_STORE_FAIL_OPEN=false
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=true
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=true
SHUMA_GATEWAY_UPSTREAM_ORIGIN=${SHUMA_GATEWAY_UPSTREAM_ORIGIN}
SHUMA_GATEWAY_DEPLOYMENT_PROFILE=shared-server
SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL=${SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL_VALUE}
SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED=true
SHUMA_GATEWAY_TLS_STRICT=true
SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED=true
SHUMA_SPIN_MANIFEST=/opt/shuma-gorath/spin.gateway.toml
EOF_ENV

REMOTE_BOOTSTRAP_SCRIPT="$(mktemp)"
TMP_FILES+=("$REMOTE_BOOTSTRAP_SCRIPT")
cat >"${REMOTE_BOOTSTRAP_SCRIPT}" <<'EOF_REMOTE_BOOTSTRAP'
#!/usr/bin/env bash

set -euo pipefail

ENV_FILE_PATH="${1:-}"
if [[ -z "${ENV_FILE_PATH}" || ! -f "${ENV_FILE_PATH}" ]]; then
  echo "Missing env file argument" >&2
  exit 1
fi

: "${DEPLOY_USER:?missing DEPLOY_USER}"
: "${REMOTE_APP_DIR:?missing REMOTE_APP_DIR}"
: "${RELEASE_ARCHIVE_PATH:?missing RELEASE_ARCHIVE_PATH}"
: "${RELEASE_METADATA_PATH:?missing RELEASE_METADATA_PATH}"
: "${GATEWAY_SURFACE_CATALOG_REMOTE_PATH:?missing GATEWAY_SURFACE_CATALOG_REMOTE_PATH}"
: "${ENABLE_CADDY:?missing ENABLE_CADDY}"
DOMAIN_NAME="${DOMAIN_NAME:-}"

sudo apt-get update -y
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y ca-certificates curl git make build-essential pkg-config libssl-dev jq ufw

if [[ "${ENABLE_CADDY}" == "true" ]]; then
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y caddy
fi

sudo mkdir -p "$(dirname "${REMOTE_APP_DIR}")"
sudo chown "${DEPLOY_USER}:${DEPLOY_USER}" "$(dirname "${REMOTE_APP_DIR}")"

NEXT_APP_DIR="${REMOTE_APP_DIR}.next"
rm -rf "${NEXT_APP_DIR}"
mkdir -p "${NEXT_APP_DIR}"
tar -xzf "${RELEASE_ARCHIVE_PATH}" -C "${NEXT_APP_DIR}"
rm -rf "${REMOTE_APP_DIR}"
mv "${NEXT_APP_DIR}" "${REMOTE_APP_DIR}"
cp "${RELEASE_METADATA_PATH}" "${REMOTE_APP_DIR}/.shuma-release.json"

cd "${REMOTE_APP_DIR}"
make setup-runtime
python3 - "${ENV_FILE_PATH}" ".env.local" <<'PY_ENV_MERGE'
from pathlib import Path
import re
import sys

overlay_path = Path(sys.argv[1])
env_path = Path(sys.argv[2])
key_pattern = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")

existing_lines = env_path.read_text(encoding="utf-8").splitlines() if env_path.exists() else []
overlay_lines = overlay_path.read_text(encoding="utf-8").splitlines()

overlay_updates = {}
overlay_order = []
for raw_line in overlay_lines:
    if not key_pattern.match(raw_line):
        continue
    key, value = raw_line.split("=", 1)
    if key not in overlay_updates:
        overlay_order.append(key)
    overlay_updates[key] = value

merged_lines = []
seen_overlay_keys = set()
for raw_line in existing_lines:
    if key_pattern.match(raw_line):
        key = raw_line.split("=", 1)[0]
        if key in overlay_updates:
            if key not in seen_overlay_keys:
                merged_lines.append(f"{key}={overlay_updates[key]}")
                seen_overlay_keys.add(key)
            continue
    merged_lines.append(raw_line)

for key in overlay_order:
    if key not in seen_overlay_keys:
        merged_lines.append(f"{key}={overlay_updates[key]}")

env_path.write_text("\n".join(merged_lines).rstrip("\n") + "\n", encoding="utf-8")
PY_ENV_MERGE
chmod 600 .env.local
set -a
source .env.local
set +a
python3 scripts/deploy/render_gateway_spin_manifest.py \
  --manifest "${REMOTE_APP_DIR}/spin.toml" \
  --output "${REMOTE_APP_DIR}/spin.gateway.toml" \
  --upstream-origin "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}"
GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_REMOTE_PATH}" make deploy-self-hosted-minimal

cat <<UNIT_FILE | sudo tee /etc/systemd/system/shuma-gorath.service >/dev/null
[Unit]
Description=Shuma-Gorath runtime
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${DEPLOY_USER}
Group=${DEPLOY_USER}
WorkingDirectory=${REMOTE_APP_DIR}
Environment=HOME=/home/${DEPLOY_USER}
Environment=PATH=/home/${DEPLOY_USER}/.cargo/bin:/usr/local/bin:/usr/bin:/bin
ExecStart=/usr/bin/make prod-start
ExecStop=/usr/bin/make stop
Restart=always
RestartSec=5
TimeoutStopSec=30

[Install]
WantedBy=multi-user.target
UNIT_FILE

sudo systemctl daemon-reload
sudo systemctl enable shuma-gorath.service
sudo systemctl restart shuma-gorath.service

if [[ "${ENABLE_CADDY}" == "true" ]]; then
  cat <<CADDY_FILE | sudo tee /etc/caddy/Caddyfile >/dev/null
${DOMAIN_NAME} {
  encode zstd gzip
  reverse_proxy 127.0.0.1:3000 {
    header_up X-Forwarded-Proto https
    header_up X-Shuma-Forwarded-Secret ${SHUMA_FORWARDED_IP_SECRET}
  }
}
CADDY_FILE

  sudo systemctl enable caddy
  if ! sudo systemctl restart caddy; then
    echo "WARN: Caddy restart failed. Check DNS for ${DOMAIN_NAME}, then run: sudo systemctl restart caddy" >&2
  fi
fi

sudo ufw --force default deny incoming
sudo ufw --force default allow outgoing
sudo ufw allow OpenSSH
if [[ "${ENABLE_CADDY}" == "true" ]]; then
  sudo ufw allow 80/tcp
  sudo ufw allow 443/tcp
else
  sudo ufw allow 3000/tcp
fi
sudo ufw --force enable

SPIN_READY_TIMEOUT_SECONDS=90 make spin-wait-ready
GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_REMOTE_PATH}" make smoke-single-host
EOF_REMOTE_BOOTSTRAP
chmod +x "${REMOTE_BOOTSTRAP_SCRIPT}"

info "Uploading deployment artifacts to ${INSTANCE_IPV4}"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${LOCAL_ENV_FILE}" "shuma@${INSTANCE_IPV4}:/tmp/shuma.env"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${REMOTE_BOOTSTRAP_SCRIPT}" "shuma@${INSTANCE_IPV4}:/tmp/shuma-bootstrap.sh"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${RELEASE_ARCHIVE_FILE}" "shuma@${INSTANCE_IPV4}:/tmp/shuma-release.tar.gz"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${RELEASE_METADATA_FILE}" "shuma@${INSTANCE_IPV4}:/tmp/shuma-release.json"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${GATEWAY_SURFACE_CATALOG_PATH}" "shuma@${INSTANCE_IPV4}:/tmp/gateway-surface-catalog.json"

REMOTE_CMD="DEPLOY_USER='shuma' REMOTE_APP_DIR='/opt/shuma-gorath' RELEASE_ARCHIVE_PATH='/tmp/shuma-release.tar.gz' RELEASE_METADATA_PATH='/tmp/shuma-release.json' GATEWAY_SURFACE_CATALOG_REMOTE_PATH='/tmp/gateway-surface-catalog.json' ENABLE_CADDY='${ENABLE_CADDY_NORM}' DOMAIN_NAME='${DOMAIN_NAME}' bash /tmp/shuma-bootstrap.sh /tmp/shuma.env"

info "Running remote bootstrap"
ssh -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "shuma@${INSTANCE_IPV4}" "${REMOTE_CMD}"

success "Deployment completed"

if [[ "${ENABLE_CADDY_NORM}" == "true" ]]; then
  BASE_URL="https://${DOMAIN_NAME}"
else
  BASE_URL="http://${INSTANCE_IPV4}:3000"
fi
DASHBOARD_URL="${BASE_URL}/dashboard"
REMOTE_TARGET_NAME="${REMOTE_NAME:-}"
if [[ -z "${REMOTE_TARGET_NAME}" ]]; then
  if [[ -n "${DOMAIN_NAME}" ]]; then
    REMOTE_TARGET_NAME="${DOMAIN_NAME}"
  elif [[ -n "${INSTANCE_LABEL}" ]]; then
    REMOTE_TARGET_NAME="${INSTANCE_LABEL}"
  else
    REMOTE_TARGET_NAME="linode-${INSTANCE_ID}"
  fi
fi
REMOTE_DEPLOYED_AT_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
persist_local_env_values "${ENV_LOCAL}" \
  "SHUMA_API_KEY=${SHUMA_API_KEY_VALUE}" \
  "SHUMA_JS_SECRET=${SHUMA_JS_SECRET_VALUE}" \
  "SHUMA_FORWARDED_IP_SECRET=${SHUMA_FORWARDED_IP_SECRET_VALUE}" \
  "SHUMA_HEALTH_SECRET=${SHUMA_HEALTH_SECRET_VALUE}" \
  "SHUMA_SIM_TELEMETRY_SECRET=${SHUMA_SIM_TELEMETRY_SECRET_VALUE}" \
  "SHUMA_ADMIN_IP_ALLOWLIST=${SHUMA_ADMIN_IP_ALLOWLIST}"

REMOTE_RECEIPT_PATH="$(python3 "${REPO_ROOT}/scripts/manage_remote_target.py" \
  --receipts-dir "${REMOTE_RECEIPTS_DIR}" \
  write-linode-receipt \
  --name "${REMOTE_TARGET_NAME}" \
  --host "${INSTANCE_IPV4}" \
  --private-key-path "${SSH_PRIVATE_KEY_FILE}" \
  --public-base-url "${BASE_URL}" \
  --surface-catalog-path "${GATEWAY_SURFACE_CATALOG_PATH}" \
  --last-deployed-commit "${RELEASE_COMMIT_SHA}" \
  --last-deployed-at-utc "${REMOTE_DEPLOYED_AT_UTC}" \
  --instance-id "${INSTANCE_ID}" \
  --label "${INSTANCE_LABEL}" \
  --region "${LINODE_REGION}" \
  --linode-type "${LINODE_TYPE}" \
  --image "${LINODE_IMAGE}")"

python3 "${REPO_ROOT}/scripts/manage_remote_target.py" \
  --env-file "${ENV_LOCAL}" \
  --receipts-dir "${REMOTE_RECEIPTS_DIR}" \
  use \
  --name "${REMOTE_TARGET_NAME}"

if [[ "${ENABLE_CADDY_NORM}" == "true" ]]; then
  echo ""
  echo "URL:       ${BASE_URL}"
  echo "Dashboard: ${DASHBOARD_URL}"
  echo "Health:    ${BASE_URL}/health (requires X-Shuma-Health-Secret header)"
  echo "Note:     If TLS is not yet active, verify DNS A/AAAA for ${DOMAIN_NAME} points to ${INSTANCE_IPV4} and restart Caddy."
else
  echo ""
  echo "URL:       ${BASE_URL}"
  echo "Dashboard: ${DASHBOARD_URL}"
  echo "Health:    ${BASE_URL}/health (requires X-Shuma-Health-Secret header)"
fi

echo "Linode ID: ${INSTANCE_ID}"
echo "Host IP:   ${INSTANCE_IPV4}"
echo "Commit:    ${RELEASE_COMMIT_SHA}"
echo "Remote:    ${REMOTE_TARGET_NAME}"
echo "Receipt:   ${REMOTE_RECEIPT_PATH}"
echo ""
echo "Dashboard login key (SHUMA_API_KEY, reused from local if already set): ${SHUMA_API_KEY_VALUE}"
echo "Health secret (SHUMA_HEALTH_SECRET): ${SHUMA_HEALTH_SECRET_VALUE}"
echo ""
echo "Next commands:"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4}"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4} 'sudo systemctl status shuma-gorath --no-pager'"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4} 'sudo journalctl -u shuma-gorath -n 200 --no-pager'"
echo ""
echo "Cleanup:"
echo "  curl -X DELETE -H 'Authorization: Bearer <LINODE_TOKEN>' ${LINODE_API_URL}/linode/instances/${INSTANCE_ID}"

if [[ "${OPEN_DASHBOARD}" -eq 1 ]]; then
  open_local_url "${DASHBOARD_URL}"
fi
