"""Helpers for deployment-specific Spin manifest rendering."""

from __future__ import annotations

import ast
import re
from urllib.parse import urlsplit

ALLOWED_OUTBOUND_HOSTS_PATTERN = re.compile(
    r"^\s*allowed_outbound_hosts\s*=\s*(\[[^\n]*\])\s*$",
    re.MULTILINE,
)
BOT_DEFENCE_SECTION_PATTERN = re.compile(
    r"(?ms)(^\[component\.bot-defence\]\n)(?P<body>.*?)(?=^\[|\Z)",
)
BOT_DEFENCE_VARIABLES_SECTION_PATTERN = re.compile(
    r"(?ms)(^\[component\.bot-defence\.variables\]\n)(?P<body>.*?)(?=^\[|\Z)",
)
VARIABLES_SECTION_PATTERN = re.compile(
    r"(?ms)(^\[variables\]\n)(?P<body>.*?)(?=^\[|\Z)",
)
TOML_ASSIGNMENT_NAME_PATTERN = re.compile(r"^\s*([A-Za-z0-9_]+)\s*=", re.MULTILINE)
BOT_DEFENCE_ENVIRONMENT_PATTERN = re.compile(r"^\s*environment\s*=\s*\{[^\n]*\}\s*$", re.MULTILINE)

FERMYON_EDGE_RUNTIME_ENV_KEYS = (
    "SHUMA_API_KEY",
    "SHUMA_ADMIN_READONLY_API_KEY",
    "SHUMA_JS_SECRET",
    "SHUMA_POW_SECRET",
    "SHUMA_CHALLENGE_SECRET",
    "SHUMA_MAZE_PREVIEW_SECRET",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_ADMIN_IP_ALLOWLIST",
    "SHUMA_ADMIN_AUTH_FAILURE_LIMIT_PER_MINUTE",
    "SHUMA_EVENT_LOG_RETENTION_HOURS",
    "SHUMA_MONITORING_RETENTION_HOURS",
    "SHUMA_MONITORING_ROLLUP_RETENTION_HOURS",
    "SHUMA_KV_STORE_FAIL_OPEN",
    "SHUMA_ENFORCE_HTTPS",
    "SHUMA_RUNTIME_ENV",
    "SHUMA_ADVERSARY_SIM_AVAILABLE",
    "SHUMA_SIM_TELEMETRY_SECRET",
    "SHUMA_FRONTIER_OPENAI_API_KEY",
    "SHUMA_FRONTIER_ANTHROPIC_API_KEY",
    "SHUMA_FRONTIER_GOOGLE_API_KEY",
    "SHUMA_FRONTIER_XAI_API_KEY",
    "SHUMA_FRONTIER_OPENAI_MODEL",
    "SHUMA_FRONTIER_ANTHROPIC_MODEL",
    "SHUMA_FRONTIER_GOOGLE_MODEL",
    "SHUMA_FRONTIER_XAI_MODEL",
    "SHUMA_ENTERPRISE_MULTI_INSTANCE",
    "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
    "SHUMA_RATE_LIMITER_REDIS_URL",
    "SHUMA_BAN_STORE_REDIS_URL",
    "SHUMA_RATE_LIMITER_OUTAGE_MODE_MAIN",
    "SHUMA_RATE_LIMITER_OUTAGE_MODE_ADMIN_AUTH",
    "SHUMA_GATEWAY_UPSTREAM_ORIGIN",
    "SHUMA_GATEWAY_DEPLOYMENT_PROFILE",
    "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_LOCAL",
    "SHUMA_GATEWAY_ALLOW_INSECURE_HTTP_SPECIAL_USE_IPS",
    "SHUMA_GATEWAY_INSECURE_HTTP_SPECIAL_USE_IP_ALLOWLIST",
    "SHUMA_GATEWAY_PUBLIC_AUTHORITIES",
    "SHUMA_GATEWAY_LOOP_MAX_HOPS",
    "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
    "SHUMA_GATEWAY_ORIGIN_AUTH_MODE",
    "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME",
    "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE",
    "SHUMA_GATEWAY_ORIGIN_AUTH_MAX_AGE_DAYS",
    "SHUMA_GATEWAY_ORIGIN_AUTH_ROTATION_OVERLAP_DAYS",
    "SHUMA_GATEWAY_TLS_STRICT",
    "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
    "SHUMA_ADMIN_CONFIG_WRITE_ENABLED",
    "SHUMA_DEBUG_HEADERS",
)


def normalize_origin(raw: str) -> tuple[str, str]:
    value = (raw or "").strip()
    if not value:
        raise ValueError("origin is empty")
    parsed = urlsplit(value)
    if parsed.scheme not in {"http", "https"}:
        raise ValueError("scheme must be http or https")
    if not parsed.hostname:
        raise ValueError("hostname is missing")
    if parsed.path not in {"", "/"} or parsed.query or parsed.fragment or parsed.username or parsed.password:
        raise ValueError("must not include path, query, fragment, or userinfo")
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    return f"{parsed.scheme}://{parsed.hostname.lower()}:{port}", parsed.scheme


def extract_allowed_outbound_hosts(manifest_text: str) -> list[str]:
    match = ALLOWED_OUTBOUND_HOSTS_PATTERN.search(manifest_text)
    if match is None:
        raise ValueError("spin manifest must define component.bot-defence allowed_outbound_hosts")
    try:
        hosts = ast.literal_eval(match.group(1))
    except (ValueError, SyntaxError) as exc:
        raise ValueError(
            "component.bot-defence.allowed_outbound_hosts must be a string list literal"
        ) from exc
    if not isinstance(hosts, list):
        raise ValueError("component.bot-defence.allowed_outbound_hosts must be a list")
    return [str(raw or "").strip() for raw in hosts if str(raw or "").strip()]


def build_manifest_with_allowed_outbound_hosts(manifest_text: str, allowed_hosts: list[str]) -> str:
    serialized_hosts = ", ".join(f'"{host}"' for host in allowed_hosts)
    replacement = f"allowed_outbound_hosts = [{serialized_hosts}]"
    rewritten, count = ALLOWED_OUTBOUND_HOSTS_PATTERN.subn(replacement, manifest_text, count=1)
    if count != 1:
        raise ValueError("spin manifest must define component.bot-defence allowed_outbound_hosts")
    return rewritten


def spin_variable_name(env_key: str) -> str:
    return env_key.strip().lower()


def _ensure_fermyon_variables_table(manifest_text: str) -> str:
    entries = [
        f'{spin_variable_name(env_key)} = {{ default = "" }}'
        for env_key in FERMYON_EDGE_RUNTIME_ENV_KEYS
    ]
    match = VARIABLES_SECTION_PATTERN.search(manifest_text)
    if match is None:
        manifest = manifest_text.rstrip("\n")
        return f"{manifest}\n\n[variables]\n" + "\n".join(entries) + "\n"

    body = match.group("body")
    existing = {name for name in TOML_ASSIGNMENT_NAME_PATTERN.findall(body)}
    missing = [entry for entry in entries if entry.split("=", 1)[0].strip() not in existing]
    if not missing:
        return manifest_text
    updated_body = body.rstrip("\n")
    if updated_body:
        updated_body += "\n"
    updated_body += "\n".join(missing) + "\n"
    return manifest_text[: match.start("body")] + updated_body + manifest_text[match.end("body") :]


def _ensure_fermyon_bot_defence_component_variables(manifest_text: str) -> str:
    match = BOT_DEFENCE_SECTION_PATTERN.search(manifest_text)
    if match is None:
        raise ValueError("spin manifest must define [component.bot-defence]")
    body = BOT_DEFENCE_ENVIRONMENT_PATTERN.sub("", match.group("body")).rstrip("\n")
    base_manifest = manifest_text[: match.start("body")] + (body + "\n" if body else "") + manifest_text[match.end("body") :]

    entries = [f'{spin_variable_name(env_key)} = "{{{{ {spin_variable_name(env_key)} }}}}"' for env_key in FERMYON_EDGE_RUNTIME_ENV_KEYS]
    match = BOT_DEFENCE_VARIABLES_SECTION_PATTERN.search(base_manifest)
    if match is None:
        section = "[component.bot-defence.variables]\n" + "\n".join(entries) + "\n"
        if not base_manifest.endswith("\n"):
            base_manifest += "\n"
        return base_manifest + "\n" + section

    existing = {name for name in TOML_ASSIGNMENT_NAME_PATTERN.findall(match.group("body"))}
    missing = [entry for entry in entries if entry.split("=", 1)[0].strip() not in existing]
    if not missing:
        return base_manifest
    updated_body = match.group("body").rstrip("\n")
    if updated_body:
        updated_body += "\n"
    updated_body += "\n".join(missing) + "\n"
    return base_manifest[: match.start("body")] + updated_body + base_manifest[match.end("body") :]


def render_gateway_manifest(manifest_text: str, upstream_origin: str) -> str:
    normalized_upstream, _ = normalize_origin(upstream_origin)
    ordered_hosts: list[str] = []
    seen_hosts: set[str] = set()

    for host in [*extract_allowed_outbound_hosts(manifest_text), normalized_upstream]:
        normalized_host, _ = normalize_origin(host)
        if normalized_host in seen_hosts:
            continue
        seen_hosts.add(normalized_host)
        ordered_hosts.append(normalized_host)

    return build_manifest_with_allowed_outbound_hosts(manifest_text, ordered_hosts)


def render_fermyon_edge_manifest(manifest_text: str, upstream_origin: str) -> str:
    rendered = render_gateway_manifest(manifest_text, upstream_origin)
    rendered = _ensure_fermyon_bot_defence_component_variables(rendered)
    return _ensure_fermyon_variables_table(rendered)
