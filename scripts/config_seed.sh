#!/bin/bash
# Seed/backfill or verify local Spin KV config from config/defaults.env.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULTS_FILE="${SHUMA_CONFIG_DEFAULTS_FILE:-${ROOT_DIR}/config/defaults.env}"
DB_PATH="${SHUMA_CONFIG_DB_PATH:-${ROOT_DIR}/.spin/sqlite_key_value.db}"
STORE_NAME="${SHUMA_CONFIG_STORE_NAME:-default}"
CONFIG_KEY="${SHUMA_CONFIG_KEY:-config:default}"
MODE="seed"

usage() {
  cat <<'USAGE'
Usage:
  ./scripts/config_seed.sh [--verify-only|--print-json]

Modes:
  default       Seed missing KV config and backfill/repair explicit persisted config state.
  --verify-only Read-only verification for missing, stale, or invalid persisted KV config.
  --print-json  Emit the canonical merged config JSON to stdout without mutating persisted KV state.

Environment overrides (primarily for tests):
  SHUMA_CONFIG_DEFAULTS_FILE
  SHUMA_CONFIG_DB_PATH
  SHUMA_CONFIG_STORE_NAME
  SHUMA_CONFIG_KEY
USAGE
}

if [[ $# -gt 1 ]]; then
  usage >&2
  exit 1
fi

if [[ $# -eq 1 ]]; then
  case "$1" in
    --verify-only)
      MODE="verify"
      ;;
    --print-json)
      MODE="print"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 1
      ;;
  esac
fi

if [[ ! -f "${DEFAULTS_FILE}" ]]; then
  echo "❌ Missing defaults file: ${DEFAULTS_FILE}" >&2
  exit 1
fi

if [[ "${MODE}" == "seed" ]] && ! command -v sqlite3 >/dev/null 2>&1; then
  echo "❌ sqlite3 is required for config-seed." >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "❌ python3 is required for config-seed." >&2
  exit 1
fi

if [[ "${MODE}" == "seed" ]]; then
  mkdir -p "$(dirname "${DB_PATH}")"
fi

# shellcheck disable=SC1090
set -a
source "${DEFAULTS_FILE}"
set +a

bool_norm() {
  local v
  v="$(printf '%s' "${1:-}" | tr '[:upper:]' '[:lower:]')"
  case "${v}" in
    1|true|yes|on) echo "true" ;;
    0|false|no|off) echo "false" ;;
    *)
      echo "❌ Invalid boolean value: ${1}" >&2
      exit 1
      ;;
  esac
}

make_tmp_file() {
  local prefix="$1"
  local tmp=""
  tmp="$(mktemp "/tmp/${prefix}.XXXXXX" 2>/dev/null)" || \
    tmp="$(mktemp -t "${prefix}" 2>/dev/null)" || {
      echo "❌ Failed to allocate temp file for ${prefix}" >&2
      exit 1
    }
  printf '%s' "${tmp}"
}

if [[ "${MODE}" == "seed" ]]; then
  sqlite3 "${DB_PATH}" <<'SQL'
CREATE TABLE IF NOT EXISTS spin_key_value (
  store TEXT NOT NULL,
  key   TEXT NOT NULL,
  value BLOB NOT NULL,
  PRIMARY KEY (store, key)
);
SQL
fi

tmp_json="$(make_tmp_file "shuma-config-seed")"
tmp_merged="$(make_tmp_file "shuma-config-merged")"
tmp_existing="$(make_tmp_file "shuma-config-existing")"
tmp_report="$(make_tmp_file "shuma-config-report")"
trap 'rm -f "${tmp_json}" "${tmp_merged}" "${tmp_existing}" "${tmp_report}"' EXIT

cat > "${tmp_json}" <<EOF
{
  "ban_duration": ${SHUMA_BAN_DURATION},
  "ban_durations": {
    "honeypot": ${SHUMA_BAN_DURATION_HONEYPOT},
    "rate_limit": ${SHUMA_BAN_DURATION_RATE_LIMIT},
    "admin": ${SHUMA_BAN_DURATION_ADMIN},
    "cdp": ${SHUMA_BAN_DURATION_CDP}
  },
  "rate_limit": ${SHUMA_RATE_LIMIT},
  "honeypot_enabled": $(bool_norm "${SHUMA_HONEYPOT_ENABLED}"),
  "honeypots": ${SHUMA_HONEYPOTS},
  "browser_policy_enabled": $(bool_norm "${SHUMA_BROWSER_POLICY_ENABLED}"),
  "browser_block": ${SHUMA_BROWSER_BLOCK},
  "browser_allowlist": ${SHUMA_BROWSER_ALLOWLIST},
  "geo_risk": ${SHUMA_GEO_RISK_COUNTRIES},
  "geo_allow": ${SHUMA_GEO_ALLOW_COUNTRIES},
  "geo_challenge": ${SHUMA_GEO_CHALLENGE_COUNTRIES},
  "geo_maze": ${SHUMA_GEO_MAZE_COUNTRIES},
  "geo_block": ${SHUMA_GEO_BLOCK_COUNTRIES},
  "geo_edge_headers_enabled": $(bool_norm "${SHUMA_GEO_EDGE_HEADERS_ENABLED}"),
  "bypass_allowlists_enabled": $(bool_norm "${SHUMA_BYPASS_ALLOWLISTS_ENABLED}"),
  "allowlist": ${SHUMA_ALLOWLIST},
  "path_allowlist_enabled": $(bool_norm "${SHUMA_PATH_ALLOWLIST_ENABLED}"),
  "path_allowlist": ${SHUMA_PATH_ALLOWLIST},
  "ip_range_policy_mode": "${SHUMA_IP_RANGE_POLICY_MODE}",
  "ip_range_emergency_allowlist": ${SHUMA_IP_RANGE_EMERGENCY_ALLOWLIST},
  "ip_range_custom_rules": ${SHUMA_IP_RANGE_CUSTOM_RULES},
  "ip_range_suggestions_min_observations": ${SHUMA_IP_RANGE_SUGGESTIONS_MIN_OBSERVATIONS},
  "ip_range_suggestions_min_bot_events": ${SHUMA_IP_RANGE_SUGGESTIONS_MIN_BOT_EVENTS},
  "ip_range_suggestions_min_confidence_percent": ${SHUMA_IP_RANGE_SUGGESTIONS_MIN_CONFIDENCE_PERCENT},
  "ip_range_suggestions_low_collateral_percent": ${SHUMA_IP_RANGE_SUGGESTIONS_LOW_COLLATERAL_PERCENT},
  "ip_range_suggestions_high_collateral_percent": ${SHUMA_IP_RANGE_SUGGESTIONS_HIGH_COLLATERAL_PERCENT},
  "ip_range_suggestions_ipv4_min_prefix_len": ${SHUMA_IP_RANGE_SUGGESTIONS_IPV4_MIN_PREFIX_LEN},
  "ip_range_suggestions_ipv6_min_prefix_len": ${SHUMA_IP_RANGE_SUGGESTIONS_IPV6_MIN_PREFIX_LEN},
  "ip_range_suggestions_likely_human_sample_percent": ${SHUMA_IP_RANGE_SUGGESTIONS_LIKELY_HUMAN_SAMPLE_PERCENT},
  "shadow_mode": $(bool_norm "${SHUMA_SHADOW_MODE}"),
  "adversary_sim_enabled": $(bool_norm "${SHUMA_ADVERSARY_SIM_ENABLED}"),
  "adversary_sim_duration_seconds": ${SHUMA_ADVERSARY_SIM_DURATION_SECONDS},
  "maze_enabled": $(bool_norm "${SHUMA_MAZE_ENABLED}"),
  "tarpit_enabled": $(bool_norm "${SHUMA_TARPIT_ENABLED}"),
  "tarpit_progress_token_ttl_seconds": ${SHUMA_TARPIT_PROGRESS_TOKEN_TTL_SECONDS},
  "tarpit_progress_replay_ttl_seconds": ${SHUMA_TARPIT_PROGRESS_REPLAY_TTL_SECONDS},
  "tarpit_hashcash_min_difficulty": ${SHUMA_TARPIT_HASHCASH_MIN_DIFFICULTY},
  "tarpit_hashcash_max_difficulty": ${SHUMA_TARPIT_HASHCASH_MAX_DIFFICULTY},
  "tarpit_hashcash_base_difficulty": ${SHUMA_TARPIT_HASHCASH_BASE_DIFFICULTY},
  "tarpit_hashcash_adaptive": $(bool_norm "${SHUMA_TARPIT_HASHCASH_ADAPTIVE}"),
  "tarpit_step_chunk_base_bytes": ${SHUMA_TARPIT_STEP_CHUNK_BASE_BYTES},
  "tarpit_step_chunk_max_bytes": ${SHUMA_TARPIT_STEP_CHUNK_MAX_BYTES},
  "tarpit_step_jitter_percent": ${SHUMA_TARPIT_STEP_JITTER_PERCENT},
  "tarpit_shard_rotation_enabled": $(bool_norm "${SHUMA_TARPIT_SHARD_ROTATION_ENABLED}"),
  "tarpit_egress_window_seconds": ${SHUMA_TARPIT_EGRESS_WINDOW_SECONDS},
  "tarpit_egress_global_bytes_per_window": ${SHUMA_TARPIT_EGRESS_GLOBAL_BYTES_PER_WINDOW},
  "tarpit_egress_per_ip_bucket_bytes_per_window": ${SHUMA_TARPIT_EGRESS_PER_IP_BUCKET_BYTES_PER_WINDOW},
  "tarpit_egress_per_flow_max_bytes": ${SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_BYTES},
  "tarpit_egress_per_flow_max_duration_seconds": ${SHUMA_TARPIT_EGRESS_PER_FLOW_MAX_DURATION_SECONDS},
  "tarpit_max_concurrent_global": ${SHUMA_TARPIT_MAX_CONCURRENT_GLOBAL},
  "tarpit_max_concurrent_per_ip_bucket": ${SHUMA_TARPIT_MAX_CONCURRENT_PER_IP_BUCKET},
  "tarpit_fallback_action": "${SHUMA_TARPIT_FALLBACK_ACTION}",
  "maze_auto_ban": $(bool_norm "${SHUMA_MAZE_AUTO_BAN}"),
  "maze_auto_ban_threshold": ${SHUMA_MAZE_AUTO_BAN_THRESHOLD},
  "maze_rollout_phase": "${SHUMA_MAZE_ROLLOUT_PHASE}",
  "maze_token_ttl_seconds": ${SHUMA_MAZE_TOKEN_TTL_SECONDS},
  "maze_token_max_depth": ${SHUMA_MAZE_TOKEN_MAX_DEPTH},
  "maze_token_branch_budget": ${SHUMA_MAZE_TOKEN_BRANCH_BUDGET},
  "maze_replay_ttl_seconds": ${SHUMA_MAZE_REPLAY_TTL_SECONDS},
  "maze_entropy_window_seconds": ${SHUMA_MAZE_ENTROPY_WINDOW_SECONDS},
  "maze_client_expansion_enabled": $(bool_norm "${SHUMA_MAZE_CLIENT_EXPANSION_ENABLED}"),
  "maze_checkpoint_every_nodes": ${SHUMA_MAZE_CHECKPOINT_EVERY_NODES},
  "maze_checkpoint_every_ms": ${SHUMA_MAZE_CHECKPOINT_EVERY_MS},
  "maze_step_ahead_max": ${SHUMA_MAZE_STEP_AHEAD_MAX},
  "maze_no_js_fallback_max_depth": ${SHUMA_MAZE_NO_JS_FALLBACK_MAX_DEPTH},
  "maze_micro_pow_enabled": $(bool_norm "${SHUMA_MAZE_MICRO_POW_ENABLED}"),
  "maze_micro_pow_depth_start": ${SHUMA_MAZE_MICRO_POW_DEPTH_START},
  "maze_micro_pow_base_difficulty": ${SHUMA_MAZE_MICRO_POW_BASE_DIFFICULTY},
  "maze_max_concurrent_global": ${SHUMA_MAZE_MAX_CONCURRENT_GLOBAL},
  "maze_max_concurrent_per_ip_bucket": ${SHUMA_MAZE_MAX_CONCURRENT_PER_IP_BUCKET},
  "maze_max_response_bytes": ${SHUMA_MAZE_MAX_RESPONSE_BYTES},
  "maze_max_response_duration_ms": ${SHUMA_MAZE_MAX_RESPONSE_DURATION_MS},
  "maze_server_visible_links": ${SHUMA_MAZE_SERVER_VISIBLE_LINKS},
  "maze_max_links": ${SHUMA_MAZE_MAX_LINKS},
  "maze_max_paragraphs": ${SHUMA_MAZE_MAX_PARAGRAPHS},
  "maze_path_entropy_segment_len": ${SHUMA_MAZE_PATH_ENTROPY_SEGMENT_LEN},
  "maze_covert_decoys_enabled": $(bool_norm "${SHUMA_MAZE_COVERT_DECOYS_ENABLED}"),
  "maze_seed_provider": "${SHUMA_MAZE_SEED_PROVIDER}",
  "maze_seed_refresh_interval_seconds": ${SHUMA_MAZE_SEED_REFRESH_INTERVAL_SECONDS},
  "maze_seed_refresh_rate_limit_per_hour": ${SHUMA_MAZE_SEED_REFRESH_RATE_LIMIT_PER_HOUR},
  "maze_seed_refresh_max_sources": ${SHUMA_MAZE_SEED_REFRESH_MAX_SOURCES},
  "maze_seed_metadata_only": $(bool_norm "${SHUMA_MAZE_SEED_METADATA_ONLY}"),
  "robots_enabled": $(bool_norm "${SHUMA_ROBOTS_ENABLED}"),
  "robots_block_ai_training": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_TRAINING}"),
  "robots_block_ai_search": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_SEARCH}"),
  "robots_allow_search_engines": $(bool_norm "${SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES}"),
  "ai_policy_block_training": $(bool_norm "${SHUMA_AI_POLICY_BLOCK_TRAINING}"),
  "ai_policy_block_search": $(bool_norm "${SHUMA_AI_POLICY_BLOCK_SEARCH}"),
  "ai_policy_allow_search_engines": $(bool_norm "${SHUMA_AI_POLICY_ALLOW_SEARCH_ENGINES}"),
  "robots_crawl_delay": ${SHUMA_ROBOTS_CRAWL_DELAY},
  "cdp_detection_enabled": $(bool_norm "${SHUMA_CDP_DETECTION_ENABLED}"),
  "cdp_auto_ban": $(bool_norm "${SHUMA_CDP_AUTO_BAN}"),
  "cdp_detection_threshold": ${SHUMA_CDP_DETECTION_THRESHOLD},
  "js_required_enforced": $(bool_norm "${SHUMA_JS_REQUIRED_ENFORCED}"),
  "pow_enabled": $(bool_norm "${SHUMA_POW_ENABLED}"),
  "pow_difficulty": ${SHUMA_POW_DIFFICULTY},
  "pow_ttl_seconds": ${SHUMA_POW_TTL_SECONDS},
  "challenge_puzzle_enabled": $(bool_norm "${SHUMA_CHALLENGE_PUZZLE_ENABLED}"),
  "challenge_puzzle_transform_count": ${SHUMA_CHALLENGE_PUZZLE_TRANSFORM_COUNT},
  "challenge_puzzle_seed_ttl_seconds": ${SHUMA_CHALLENGE_PUZZLE_SEED_TTL_SECONDS},
  "challenge_puzzle_attempt_limit_per_window": ${SHUMA_CHALLENGE_PUZZLE_ATTEMPT_LIMIT_PER_WINDOW},
  "challenge_puzzle_attempt_window_seconds": ${SHUMA_CHALLENGE_PUZZLE_ATTEMPT_WINDOW_SECONDS},
  "challenge_puzzle_risk_threshold": ${SHUMA_CHALLENGE_PUZZLE_RISK_THRESHOLD},
  "not_a_bot_enabled": $(bool_norm "${SHUMA_NOT_A_BOT_ENABLED}"),
  "not_a_bot_risk_threshold": ${SHUMA_NOT_A_BOT_RISK_THRESHOLD},
  "not_a_bot_pass_score": ${SHUMA_NOT_A_BOT_PASS_SCORE},
  "not_a_bot_fail_score": ${SHUMA_NOT_A_BOT_FAIL_SCORE},
  "not_a_bot_nonce_ttl_seconds": ${SHUMA_NOT_A_BOT_NONCE_TTL_SECONDS},
  "not_a_bot_marker_ttl_seconds": ${SHUMA_NOT_A_BOT_MARKER_TTL_SECONDS},
  "not_a_bot_attempt_limit_per_window": ${SHUMA_NOT_A_BOT_ATTEMPT_LIMIT_PER_WINDOW},
  "not_a_bot_attempt_window_seconds": ${SHUMA_NOT_A_BOT_ATTEMPT_WINDOW_SECONDS},
  "botness_maze_threshold": ${SHUMA_BOTNESS_MAZE_THRESHOLD},
  "botness_weights": {
    "js_required": ${SHUMA_BOTNESS_WEIGHT_JS_REQUIRED},
    "geo_risk": ${SHUMA_BOTNESS_WEIGHT_GEO_RISK},
    "rate_medium": ${SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM},
    "rate_high": ${SHUMA_BOTNESS_WEIGHT_RATE_HIGH},
    "maze_behavior": ${SHUMA_BOTNESS_WEIGHT_MAZE_BEHAVIOR}
  },
  "defence_modes": {
    "rate": "${SHUMA_MODE_RATE}",
    "geo": "${SHUMA_MODE_GEO}",
    "js": "${SHUMA_MODE_JS}"
  },
  "provider_backends": {
    "rate_limiter": "${SHUMA_PROVIDER_RATE_LIMITER}",
    "ban_store": "${SHUMA_PROVIDER_BAN_STORE}",
    "challenge_engine": "${SHUMA_PROVIDER_CHALLENGE_ENGINE}",
    "maze_tarpit": "${SHUMA_PROVIDER_MAZE_TARPIT}",
    "fingerprint_signal": "${SHUMA_PROVIDER_FINGERPRINT_SIGNAL}"
  },
  "edge_integration_mode": "${SHUMA_EDGE_INTEGRATION_MODE}"
}
EOF

if [[ "${MODE}" == "seed" ]]; then
  existing_json="$(sqlite3 "${DB_PATH}" "SELECT CAST(value AS TEXT) FROM spin_key_value WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}' LIMIT 1;")"
  printf '%s' "${existing_json}" > "${tmp_existing}"
else
  set +e
  python3 - "${DB_PATH}" "${STORE_NAME}" "${CONFIG_KEY}" "${tmp_existing}" <<'PY'
import pathlib
import sqlite3
import sys

db_path, store_name, config_key, existing_path = sys.argv[1:5]
path = pathlib.Path(db_path)
if not path.exists():
    sys.exit(10)

try:
    conn = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
except sqlite3.Error:
    sys.exit(10)

try:
    cursor = conn.execute(
        "SELECT CAST(value AS TEXT) FROM spin_key_value WHERE store=? AND key=? LIMIT 1",
        (store_name, config_key),
    )
    row = cursor.fetchone()
except sqlite3.Error:
    sys.exit(10)
finally:
    conn.close()

if row is None or row[0] is None:
    sys.exit(10)

pathlib.Path(existing_path).write_text(str(row[0]), encoding="utf-8")
PY
  status=$?
  set -e
  if [[ ${status} -eq 10 ]]; then
    printf '' > "${tmp_existing}"
  elif [[ ${status} -ne 0 ]]; then
    exit "${status}"
  fi
fi

python3 - "${MODE}" "${tmp_json}" "${tmp_merged}" "${tmp_existing}" "${tmp_report}" <<'PY'
import copy
import json
import pathlib
import sys

mode, defaults_path, merged_path, existing_path, report_path = sys.argv[1:6]
with open(defaults_path, "r", encoding="utf-8") as handle:
    defaults = json.load(handle)

existing_raw = pathlib.Path(existing_path).read_text(encoding="utf-8").strip()


def classify_type(value):
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        return "int"
    if isinstance(value, float):
        return "float"
    if isinstance(value, str):
        return "str"
    if isinstance(value, list):
        return "list"
    if isinstance(value, dict):
        return "dict"
    if value is None:
        return "null"
    return type(value).__name__


def repair_value(existing_value, defaults_value, path):
    if isinstance(defaults_value, dict):
        if not isinstance(existing_value, dict):
            return copy.deepcopy(defaults_value), True, [], [f"{path or '<root>'} expected object"]
        merged_map = dict(existing_value)
        changed = False
        missing = []
        invalid = []
        for key, default_child in defaults_value.items():
            child_path = f"{path}.{key}" if path else key
            if key not in merged_map:
                merged_map[key] = copy.deepcopy(default_child)
                changed = True
                missing.append(child_path)
                continue
            repaired_child, child_changed, child_missing, child_invalid = repair_value(
                merged_map[key], default_child, child_path
            )
            merged_map[key] = repaired_child
            changed = changed or child_changed
            missing.extend(child_missing)
            invalid.extend(child_invalid)
        return merged_map, changed, missing, invalid

    if isinstance(defaults_value, list):
        if isinstance(existing_value, list):
            return existing_value, False, [], []
        return copy.deepcopy(defaults_value), True, [], [
            f"{path or '<root>'} expected list, found {classify_type(existing_value)}"
        ]

    if isinstance(defaults_value, bool):
        if isinstance(existing_value, bool):
            return existing_value, False, [], []
        return defaults_value, True, [], [
            f"{path or '<root>'} expected bool, found {classify_type(existing_value)}"
        ]

    if isinstance(defaults_value, int) and not isinstance(defaults_value, bool):
        if isinstance(existing_value, int) and not isinstance(existing_value, bool):
            return existing_value, False, [], []
        return defaults_value, True, [], [
            f"{path or '<root>'} expected int, found {classify_type(existing_value)}"
        ]

    if isinstance(defaults_value, float):
        if isinstance(existing_value, (int, float)) and not isinstance(existing_value, bool):
            return existing_value, False, [], []
        return defaults_value, True, [], [
            f"{path or '<root>'} expected number, found {classify_type(existing_value)}"
        ]

    if isinstance(defaults_value, str):
        if isinstance(existing_value, str):
            return existing_value, False, [], []
        return defaults_value, True, [], [
            f"{path or '<root>'} expected string, found {classify_type(existing_value)}"
        ]

    return copy.deepcopy(defaults_value), existing_value != defaults_value, [], []


if not existing_raw:
    state = "missing"
    merged = copy.deepcopy(defaults)
    changed = True
    missing_paths = []
    invalid_paths = []
else:
    try:
        existing = json.loads(existing_raw)
    except json.JSONDecodeError as exc:
        state = "invalid"
        merged = copy.deepcopy(defaults)
        changed = True
        missing_paths = []
        invalid_paths = [f"json parse error: {exc}"]
    else:
        merged, changed, missing_paths, invalid_paths = repair_value(existing, defaults, "")
        if invalid_paths:
            state = "invalid"
        elif missing_paths:
            state = "stale"
        else:
            state = "ready"

if mode in {"seed", "print"} and isinstance(merged, dict):
    default_shadow_mode = bool(defaults.get("shadow_mode", False))
    if merged.get("shadow_mode") != default_shadow_mode:
        merged["shadow_mode"] = default_shadow_mode
        changed = True
    default_adversary_sim_enabled = bool(defaults.get("adversary_sim_enabled", False))
    if merged.get("adversary_sim_enabled") != default_adversary_sim_enabled:
        merged["adversary_sim_enabled"] = default_adversary_sim_enabled
        changed = True

with open(merged_path, "w", encoding="utf-8") as handle:
    json.dump(merged, handle, separators=(",", ":"))

with open(report_path, "w", encoding="utf-8") as handle:
    json.dump(
        {
            "state": state,
            "changed": changed,
            "missing_paths": missing_paths,
            "invalid_paths": invalid_paths,
        },
        handle,
        separators=(",", ":"),
    )
PY

report_field() {
  local field="$1"
  python3 - "${tmp_report}" "${field}" <<'PY'
import json
import sys

report_path, field = sys.argv[1], sys.argv[2]
with open(report_path, "r", encoding="utf-8") as handle:
    report = json.load(handle)
value = report.get(field)
if isinstance(value, list):
    print(", ".join(str(item) for item in value))
elif isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("")
else:
    print(value)
PY
}

state="$(report_field state)"
missing_paths="$(report_field missing_paths)"
invalid_paths="$(report_field invalid_paths)"

if [[ "${MODE}" == "print" ]]; then
  cat "${tmp_merged}"
  exit 0
fi

if [[ "${MODE}" == "verify" ]]; then
  case "${state}" in
    ready)
      echo "✅ KV config present and schema-complete (${CONFIG_KEY})."
      exit 0
      ;;
    missing)
      echo "❌ Found missing KV config at ${CONFIG_KEY}. Run make setup, make setup-runtime, or make config-seed." >&2
      exit 1
      ;;
    stale)
      echo "❌ Found stale KV config at ${CONFIG_KEY}. Missing keys: ${missing_paths}. Run make config-seed to backfill persisted config explicitly." >&2
      exit 1
      ;;
    invalid)
      echo "❌ Found invalid KV config at ${CONFIG_KEY}. ${invalid_paths}. Run make config-seed to repair the persisted config explicitly." >&2
      exit 1
      ;;
    *)
      echo "❌ Unknown config verification state: ${state}" >&2
      exit 1
      ;;
  esac
fi

case "${state}" in
  missing)
    sqlite3 "${DB_PATH}" "INSERT INTO spin_key_value(store,key,value) VALUES('${STORE_NAME}','${CONFIG_KEY}',readfile('${tmp_merged}'));"
    echo "✅ Seeded KV config from config/defaults.env into ${CONFIG_KEY}"
    ;;
  stale)
    sqlite3 "${DB_PATH}" "UPDATE spin_key_value SET value=readfile('${tmp_merged}') WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}';"
    echo "✅ Backfilled missing KV config keys from config/defaults.env into ${CONFIG_KEY}"
    ;;
  invalid)
    sqlite3 "${DB_PATH}" "UPDATE spin_key_value SET value=readfile('${tmp_merged}') WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}';"
    echo "✅ Repaired invalid KV config from config/defaults.env into ${CONFIG_KEY}"
    ;;
  ready)
    echo "✅ KV config already seeded/backfilled (${CONFIG_KEY}); no missing keys."
    ;;
  *)
    echo "❌ Unknown config seed state: ${state}" >&2
    exit 1
    ;;
esac
