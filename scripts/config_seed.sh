#!/bin/bash
# Seed local Spin KV config from config/defaults.env (only when missing).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULTS_FILE="${ROOT_DIR}/config/defaults.env"
DB_PATH="${ROOT_DIR}/.spin/sqlite_key_value.db"
STORE_NAME="default"
CONFIG_KEY="config:default"

if [[ ! -f "${DEFAULTS_FILE}" ]]; then
  echo "❌ Missing defaults file: ${DEFAULTS_FILE}" >&2
  exit 1
fi

if ! command -v sqlite3 >/dev/null 2>&1; then
  echo "❌ sqlite3 is required for config-seed." >&2
  exit 1
fi

mkdir -p "$(dirname "${DB_PATH}")"

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

sqlite3 "${DB_PATH}" <<'SQL'
CREATE TABLE IF NOT EXISTS spin_key_value (
  store TEXT NOT NULL,
  key   TEXT NOT NULL,
  value BLOB NOT NULL,
  PRIMARY KEY (store, key)
);
SQL

exists="$(sqlite3 "${DB_PATH}" "SELECT 1 FROM spin_key_value WHERE store='${STORE_NAME}' AND key='${CONFIG_KEY}' LIMIT 1;")"
if [[ "${exists}" == "1" ]]; then
  echo "✅ KV config already seeded (${CONFIG_KEY}); skipping."
  exit 0
fi

tmp_json="$(mktemp "/tmp/shuma-config-seed.XXXXXX.json")"
trap 'rm -f "${tmp_json}"' EXIT

cat > "${tmp_json}" <<EOF
{
  "ban_duration": ${SHUMA_BAN_DURATION},
  "ban_durations": {
    "honeypot": ${SHUMA_BAN_DURATION_HONEYPOT},
    "rate_limit": ${SHUMA_BAN_DURATION_RATE_LIMIT},
    "browser": ${SHUMA_BAN_DURATION_BROWSER},
    "admin": ${SHUMA_BAN_DURATION_ADMIN},
    "cdp": ${SHUMA_BAN_DURATION_CDP}
  },
  "rate_limit": ${SHUMA_RATE_LIMIT},
  "honeypots": ${SHUMA_HONEYPOTS},
  "browser_block": ${SHUMA_BROWSER_BLOCK},
  "browser_whitelist": ${SHUMA_BROWSER_WHITELIST},
  "geo_risk": ${SHUMA_GEO_RISK_COUNTRIES},
  "geo_allow": ${SHUMA_GEO_ALLOW_COUNTRIES},
  "geo_challenge": ${SHUMA_GEO_CHALLENGE_COUNTRIES},
  "geo_maze": ${SHUMA_GEO_MAZE_COUNTRIES},
  "geo_block": ${SHUMA_GEO_BLOCK_COUNTRIES},
  "whitelist": ${SHUMA_WHITELIST},
  "path_whitelist": ${SHUMA_PATH_WHITELIST},
  "test_mode": $(bool_norm "${SHUMA_TEST_MODE}"),
  "maze_enabled": $(bool_norm "${SHUMA_MAZE_ENABLED}"),
  "maze_auto_ban": $(bool_norm "${SHUMA_MAZE_AUTO_BAN}"),
  "maze_auto_ban_threshold": ${SHUMA_MAZE_AUTO_BAN_THRESHOLD},
  "robots_enabled": $(bool_norm "${SHUMA_ROBOTS_ENABLED}"),
  "robots_block_ai_training": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_TRAINING}"),
  "robots_block_ai_search": $(bool_norm "${SHUMA_ROBOTS_BLOCK_AI_SEARCH}"),
  "robots_allow_search_engines": $(bool_norm "${SHUMA_ROBOTS_ALLOW_SEARCH_ENGINES}"),
  "robots_crawl_delay": ${SHUMA_ROBOTS_CRAWL_DELAY},
  "cdp_detection_enabled": $(bool_norm "${SHUMA_CDP_DETECTION_ENABLED}"),
  "cdp_auto_ban": $(bool_norm "${SHUMA_CDP_AUTO_BAN}"),
  "cdp_detection_threshold": ${SHUMA_CDP_DETECTION_THRESHOLD},
  "js_required_enforced": $(bool_norm "${SHUMA_JS_REQUIRED_ENFORCED}"),
  "pow_enabled": $(bool_norm "${SHUMA_POW_ENABLED}"),
  "pow_difficulty": ${SHUMA_POW_DIFFICULTY},
  "pow_ttl_seconds": ${SHUMA_POW_TTL_SECONDS},
  "challenge_transform_count": ${SHUMA_CHALLENGE_TRANSFORM_COUNT},
  "challenge_risk_threshold": ${SHUMA_CHALLENGE_RISK_THRESHOLD},
  "botness_maze_threshold": ${SHUMA_BOTNESS_MAZE_THRESHOLD},
  "botness_weights": {
    "js_required": ${SHUMA_BOTNESS_WEIGHT_JS_REQUIRED},
    "geo_risk": ${SHUMA_BOTNESS_WEIGHT_GEO_RISK},
    "rate_medium": ${SHUMA_BOTNESS_WEIGHT_RATE_MEDIUM},
    "rate_high": ${SHUMA_BOTNESS_WEIGHT_RATE_HIGH}
  }
}
EOF

sqlite3 "${DB_PATH}" "INSERT INTO spin_key_value(store,key,value) VALUES('${STORE_NAME}','${CONFIG_KEY}',readfile('${tmp_json}'));"
echo "✅ Seeded KV config from config/defaults.env into ${CONFIG_KEY}"
