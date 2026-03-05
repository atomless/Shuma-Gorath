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

Options:
  --label <value>                      Linode instance label (default: shuma-<UTC timestamp>)
  --profile <small|medium|large>       Deployment profile (default: small)
  --region <value>                     Linode region slug (default: us-east)
  --type <value>                       Linode type/plan override (profile-derived by default)
  --image <value>                      Linode image slug (default: linode/ubuntu24.04)
  --repo-url <value>                   Git URL cloned on server (default: origin remote URL)
  --repo-ref <value>                   Branch/tag cloned on server (default: main)
  --ssh-public-key-file <path>         SSH public key for first access (default: ~/.ssh/id_ed25519.pub, fallback ~/.ssh/id_rsa.pub)
  --ssh-private-key-file <path>        SSH private key paired with the public key (default: public key without .pub)
  --domain <fqdn>                      Enable Caddy reverse proxy/TLS for this domain
  --enable-caddy <auto|true|false>     Caddy mode (default: auto; auto=true when --domain is set)
  --preflight-only                     Run validations only; do not create infrastructure
  --destroy-on-failure                 Delete created Linode instance if deployment fails
  --help                               Show this help

Notes:
  - Run this from a cloned Shuma-Gorath repository root.
  - When --domain is set, ensure DNS A/AAAA already points to the new Linode before expecting TLS success.
  - Without --domain, service is exposed on http://<linode-ip>:3000 and SHUMA_ENFORCE_HTTPS is set to false.
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

normalize_repo_url_for_remote_clone() {
  local input_url="$1"
  if [[ "$input_url" =~ ^git@github\.com:(.+)$ ]]; then
    printf 'https://github.com/%s' "${BASH_REMATCH[1]}"
    return 0
  fi
  printf '%s' "$input_url"
}

LINODE_API_URL="https://api.linode.com/v4"
LINODE_LABEL="shuma-$(date -u +%Y%m%d%H%M%S)"
LINODE_PROFILE="${LINODE_PROFILE:-small}"
LINODE_REGION="${LINODE_REGION:-us-east}"
LINODE_TYPE="${LINODE_TYPE:-}"
LINODE_IMAGE="${LINODE_IMAGE:-linode/ubuntu24.04}"
REPO_URL="${REPO_URL:-$(git config --get remote.origin.url || true)}"
REPO_REF="${REPO_REF:-main}"
SSH_PUBLIC_KEY_FILE=""
SSH_PRIVATE_KEY_FILE=""
DOMAIN_NAME="${DOMAIN_NAME:-}"
ENABLE_CADDY="${ENABLE_CADDY:-auto}"
PREFLIGHT_ONLY=0
DESTROY_ON_FAILURE=0
TYPE_EXPLICIT=0

LINODE_TOKEN="${LINODE_TOKEN:-}"
SHUMA_ADMIN_IP_ALLOWLIST="${SHUMA_ADMIN_IP_ALLOWLIST:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --label)
      LINODE_LABEL="${2:-}"
      shift 2
      ;;
    --region)
      LINODE_REGION="${2:-}"
      shift 2
      ;;
    --profile)
      LINODE_PROFILE="${2:-}"
      shift 2
      ;;
    --type)
      LINODE_TYPE="${2:-}"
      TYPE_EXPLICIT=1
      shift 2
      ;;
    --image)
      LINODE_IMAGE="${2:-}"
      shift 2
      ;;
    --repo-url)
      REPO_URL="${2:-}"
      shift 2
      ;;
    --repo-ref)
      REPO_REF="${2:-}"
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
if [[ -z "${REPO_URL}" ]]; then
  fail "Could not infer repository URL. Set --repo-url explicitly."
fi
REPO_URL="$(normalize_repo_url_for_remote_clone "${REPO_URL}")"

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

if [[ -z "${SSH_PUBLIC_KEY_FILE}" ]]; then
  for candidate in "$HOME/.ssh/id_ed25519.pub" "$HOME/.ssh/id_rsa.pub"; do
    if [[ -f "$candidate" ]]; then
      SSH_PUBLIC_KEY_FILE="$candidate"
      break
    fi
  done
fi
[[ -f "${SSH_PUBLIC_KEY_FILE}" ]] || fail "SSH public key file not found: ${SSH_PUBLIC_KEY_FILE}"

if [[ -z "${SSH_PRIVATE_KEY_FILE}" ]]; then
  SSH_PRIVATE_KEY_FILE="${SSH_PUBLIC_KEY_FILE%.pub}"
fi
[[ -f "${SSH_PRIVATE_KEY_FILE}" ]] || fail "SSH private key file not found: ${SSH_PRIVATE_KEY_FILE}"

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

for cmd in curl jq ssh scp python3; do
  require_cmd "$cmd"
done

# Keep long-running API operations deterministic by using explicit polling and optional cleanup.
INSTANCE_ID=""
INSTANCE_IPV4=""
TMP_FILES=()

cleanup() {
  local code="$?"
  for tmp in "${TMP_FILES[@]}"; do
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

run_preflight_checks() {
  info "Running preflight checks (no infrastructure changes yet)"
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
  echo "  repo:    ${REPO_URL} @ ${REPO_REF}"
  echo "  caddy:   ${ENABLE_CADDY_NORM}"
}

run_preflight_checks

if [[ "${PREFLIGHT_ONLY}" -eq 1 ]]; then
  success "Preflight-only mode complete. No Linode resources were created."
  exit 0
fi

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

ENFORCE_HTTPS_VALUE="false"
if [[ "${ENABLE_CADDY_NORM}" == "true" ]]; then
  ENFORCE_HTTPS_VALUE="true"
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
SHUMA_ADMIN_CONFIG_WRITE_ENABLED=false
SHUMA_DEBUG_HEADERS=false
SHUMA_RUNTIME_ENV=runtime-prod
SHUMA_ADVERSARY_SIM_AVAILABLE=false
SHUMA_ENFORCE_HTTPS=${ENFORCE_HTTPS_VALUE}
SHUMA_KV_STORE_FAIL_OPEN=false
SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED=false
SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED=false
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
: "${REPO_URL:?missing REPO_URL}"
: "${REPO_REF:?missing REPO_REF}"
: "${ENABLE_CADDY:?missing ENABLE_CADDY}"
DOMAIN_NAME="${DOMAIN_NAME:-}"

sudo apt-get update -y
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y ca-certificates curl git make build-essential pkg-config libssl-dev jq ufw

if [[ "${ENABLE_CADDY}" == "true" ]]; then
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y caddy
fi

sudo mkdir -p "$(dirname "${REMOTE_APP_DIR}")"
sudo chown "${DEPLOY_USER}:${DEPLOY_USER}" "$(dirname "${REMOTE_APP_DIR}")"

if [[ ! -d "${REMOTE_APP_DIR}/.git" ]]; then
  git clone --depth 1 --branch "${REPO_REF}" "${REPO_URL}" "${REMOTE_APP_DIR}"
else
  cd "${REMOTE_APP_DIR}"
  git fetch --depth 1 origin "${REPO_REF}"
  git checkout -f "${REPO_REF}"
  git reset --hard "origin/${REPO_REF}"
fi

cd "${REMOTE_APP_DIR}"
make setup-runtime
cp "${ENV_FILE_PATH}" .env.local
chmod 600 .env.local

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
ExecStart=/usr/bin/make prod
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
sudo ufw --force allow OpenSSH
if [[ "${ENABLE_CADDY}" == "true" ]]; then
  sudo ufw --force allow 80/tcp
  sudo ufw --force allow 443/tcp
else
  sudo ufw --force allow 3000/tcp
fi
sudo ufw --force enable

curl -fsS -H "X-Shuma-Health-Secret: $(grep '^SHUMA_HEALTH_SECRET=' .env.local | cut -d= -f2-)" http://127.0.0.1:3000/health >/dev/null
EOF_REMOTE_BOOTSTRAP
chmod +x "${REMOTE_BOOTSTRAP_SCRIPT}"

info "Uploading deployment artifacts to ${INSTANCE_IPV4}"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${LOCAL_ENV_FILE}" "shuma@${INSTANCE_IPV4}:/tmp/shuma.env"
scp -q -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "${REMOTE_BOOTSTRAP_SCRIPT}" "shuma@${INSTANCE_IPV4}:/tmp/shuma-bootstrap.sh"

REMOTE_CMD="DEPLOY_USER='shuma' REMOTE_APP_DIR='/opt/shuma-gorath' REPO_URL='${REPO_URL}' REPO_REF='${REPO_REF}' ENABLE_CADDY='${ENABLE_CADDY_NORM}' DOMAIN_NAME='${DOMAIN_NAME}' bash /tmp/shuma-bootstrap.sh /tmp/shuma.env"

info "Running remote bootstrap"
ssh -o StrictHostKeyChecking=accept-new -i "${SSH_PRIVATE_KEY_FILE}" "shuma@${INSTANCE_IPV4}" "${REMOTE_CMD}"

success "Deployment completed"

if [[ "${ENABLE_CADDY_NORM}" == "true" ]]; then
  echo ""
  echo "URL:      https://${DOMAIN_NAME}"
  echo "Health:   https://${DOMAIN_NAME}/health (requires X-Shuma-Health-Secret header)"
  echo "Note:     If TLS is not yet active, verify DNS A/AAAA for ${DOMAIN_NAME} points to ${INSTANCE_IPV4} and restart Caddy."
else
  echo ""
  echo "URL:      http://${INSTANCE_IPV4}:3000"
  echo "Health:   http://${INSTANCE_IPV4}:3000/health (requires X-Shuma-Health-Secret header)"
fi

echo "Linode ID: ${INSTANCE_ID}"
echo "Host IP:   ${INSTANCE_IPV4}"
echo ""
echo "Dashboard login key (SHUMA_API_KEY): ${SHUMA_API_KEY_VALUE}"
echo "Health secret (SHUMA_HEALTH_SECRET): ${SHUMA_HEALTH_SECRET_VALUE}"
echo ""
echo "Next commands:"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4}"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4} 'sudo systemctl status shuma-gorath --no-pager'"
echo "  ssh -i ${SSH_PRIVATE_KEY_FILE} shuma@${INSTANCE_IPV4} 'sudo journalctl -u shuma-gorath -n 200 --no-pager'"
echo ""
echo "Cleanup:"
echo "  curl -X DELETE -H 'Authorization: Bearer <LINODE_TOKEN>' ${LINODE_API_URL}/linode/instances/${INSTANCE_ID}"
