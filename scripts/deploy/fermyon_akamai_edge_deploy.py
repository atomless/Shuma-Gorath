"""Agent-oriented Fermyon / Akamai edge deploy helper."""

from __future__ import annotations

import argparse
import http.cookiejar
import json
import os
import pty
import select
import subprocess
import sys
import time
import uuid
from pathlib import Path
from typing import Any, Sequence
from urllib import error as urllib_error
from urllib import parse as urllib_parse
from urllib import request as urllib_request

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.fermyon_akamai_edge_setup import (
    DEFAULT_DEPLOY_RECEIPT_PATH,
    DEFAULT_RECEIPT_PATH,
    DEFAULT_RUNTIME_ENV,
    SPIN_AKA_TOKEN_KEY,
    ensure_aka_plugin,
    fetch_aka_info,
    parse_version_line,
    run_command,
    validate_aka_login,
)
from scripts.deploy.local_env import ensure_env_file, read_env_file, read_env_files, read_env_value
from scripts.deploy.spin_manifest import FERMYON_EDGE_RUNTIME_ENV_KEYS, spin_variable_name
from scripts.deploy.setup_common import utc_now_iso, write_json

DEFAULTS_ENV_FILE = REPO_ROOT / "config" / "defaults.env"
EDGE_CRON_JOB_NAME_PREFIX = "shuma-adversary-sim-beat"
EDGE_CRON_SCHEDULES = (
    "*/5 * * * *",
    "1-59/5 * * * *",
    "2-59/5 * * * *",
    "3-59/5 * * * *",
    "4-59/5 * * * *",
)
EDGE_CRON_SECRET_QUERY_KEY = "edge_cron_secret"
EDGE_ADVERSARY_SIM_SMOKE_TIMEOUT_SECONDS = 185
EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_SECONDS = 90
EDGE_CONTROL_LEASE_RELEASE_TIMEOUT_SECONDS = 45
WORKSPACE_RENDERED_MANIFEST_NAME = "spin.fermyon-akamai-edge.toml"


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Deploy Shuma to Fermyon Wasm Functions on Akamai using a durable setup receipt."
    )
    parser.add_argument("--env-file", default=str(REPO_ROOT / ".env.local"))
    parser.add_argument("--setup-receipt", default=str(DEFAULT_RECEIPT_PATH))
    parser.add_argument("--deploy-receipt-output", default=str(DEFAULT_DEPLOY_RECEIPT_PATH))
    parser.add_argument("--preflight-only", action="store_true")
    return parser.parse_args(argv)


def load_receipt(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise SystemExit(f"Setup receipt does not exist: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Setup receipt is not valid JSON: {path} ({exc})") from exc
    if payload.get("schema") != "shuma.fermyon.akamai_edge_setup.v2":
        raise SystemExit(f"Unexpected setup receipt schema for {path}: {payload.get('schema')!r}")
    if payload.get("status") != "ready":
        progress = payload.get("progress", {})
        blocked_at_step = str(progress.get("blocked_at_step", "")).strip()
        blocked_reason = str(progress.get("blocked_reason", "")).strip()
        next_operator_action = str(progress.get("next_operator_action", "")).strip()
        message = [f"Setup receipt is not ready for deploy: status={payload.get('status')!r}"]
        if blocked_at_step:
            message.append(f"blocked_at_step={blocked_at_step}")
        if blocked_reason:
            message.append(blocked_reason)
        if next_operator_action:
            message.append(next_operator_action)
        raise SystemExit(" ".join(message))
    return payload


def load_previous_deploy_receipt(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    if payload.get("schema") != "shuma.fermyon.akamai_edge_deploy.v1":
        return {}
    return payload


def required_env_value(env_values: dict[str, str], key: str) -> str:
    value = env_values.get(key, "").strip()
    if not value:
        raise SystemExit(f"Required env value is missing: {key}")
    return value


def bool_env_value(env_values: dict[str, str], key: str, default: str = "false") -> str:
    value = env_values.get(key, default).strip().lower()
    return value if value else default


def render_manifest(receipt: dict[str, Any], rendered_manifest_path: Path, env_values: dict[str, str]) -> None:
    manifest_source = REPO_ROOT / "spin.toml"
    upstream_origin = receipt["gateway"]["upstream_origin"]
    result = run_command(
        [
            "python3",
            str(REPO_ROOT / "scripts" / "deploy" / "render_gateway_spin_manifest.py"),
            "--manifest",
            str(manifest_source),
            "--output",
            str(rendered_manifest_path),
            "--upstream-origin",
            upstream_origin,
            "--profile",
            "edge-fermyon",
        ],
        env=os.environ.copy(),
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or "Failed to render edge Spin manifest.")


def resolve_rendered_manifest_path(receipt: dict[str, Any]) -> Path:
    workspace_manifest_path = (REPO_ROOT / WORKSPACE_RENDERED_MANIFEST_NAME).resolve()
    artifacts = receipt.get("artifacts") if isinstance(receipt, dict) else {}
    artifacts_mapping = artifacts if isinstance(artifacts, dict) else {}
    recorded_path = str(artifacts_mapping.get("rendered_manifest_path") or "").strip()
    if not recorded_path:
        return workspace_manifest_path
    candidate = Path(recorded_path).expanduser().resolve()
    if candidate == workspace_manifest_path:
        return candidate
    return workspace_manifest_path


def deploy_env(receipt: dict[str, Any], env_file_values: dict[str, str], setup_receipt_path: Path) -> dict[str, str]:
    gateway = receipt["gateway"]
    env = os.environ.copy()
    env.update(env_file_values)
    env["SHUMA_RUNTIME_ENV"] = gateway.get("runtime_env", DEFAULT_RUNTIME_ENV)
    env["SHUMA_ENTERPRISE_MULTI_INSTANCE"] = "true" if gateway.get("enterprise_multi_instance") else "false"
    env["SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED"] = (
        "true" if gateway.get("enterprise_unsynced_state_exception_confirmed") else "false"
    )
    env["SHUMA_EDGE_INTEGRATION_MODE"] = gateway.get("edge_integration_mode", "additive")
    env["SHUMA_GATEWAY_DEPLOYMENT_PROFILE"] = gateway.get("deployment_profile", "edge-fermyon")
    env["SHUMA_GATEWAY_UPSTREAM_ORIGIN"] = gateway["upstream_origin"]
    env["SHUMA_GATEWAY_TLS_STRICT"] = "true" if gateway.get("tls_strict", True) else "false"
    env["SHUMA_GATEWAY_ORIGIN_AUTH_MODE"] = gateway.get("origin_auth_mode", "signed_header")
    env["SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME"] = gateway["origin_auth_header_name"]
    env["SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED"] = "true" if gateway.get("origin_lock_confirmed") else "false"
    env["SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED"] = (
        "true" if gateway.get("reserved_route_collision_check_passed") else "false"
    )
    env["SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED"] = (
        "true" if gateway.get("admin_edge_rate_limits_confirmed") else "false"
    )
    env["SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED"] = (
        "true" if gateway.get("admin_api_key_rotation_confirmed") else "false"
    )
    env["GATEWAY_SURFACE_CATALOG_PATH"] = gateway["surface_catalog_path"]
    env["FERMYON_AKA_APP_NAME"] = receipt["fermyon"]["app_name"]
    env["FERMYON_AKAMAI_SETUP_RECEIPT"] = str(setup_receipt_path)
    required_env_value(env, SPIN_AKA_TOKEN_KEY)
    required_env_value(env, "SHUMA_API_KEY")
    required_env_value(env, "SHUMA_JS_SECRET")
    required_env_value(env, "SHUMA_FORWARDED_IP_SECRET")
    required_env_value(env, "SHUMA_HEALTH_SECRET")
    required_env_value(env, "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")
    required_env_value(env, "SHUMA_ADMIN_IP_ALLOWLIST")
    required_env_value(env, "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE")
    return env


def ensure_aka_session(env: dict[str, str]) -> tuple[dict[str, Any], str]:
    try:
        return fetch_aka_info(env), "existing_session"
    except SystemExit:
        token = required_env_value(env, SPIN_AKA_TOKEN_KEY)
        info, auth_mode = validate_aka_login(token)
        return info, auth_mode


def extract_app_id(payload: dict[str, Any]) -> str:
    app = payload.get("app")
    app_mapping = app if isinstance(app, dict) else {}
    candidates = (
        payload.get("app_id"),
        payload.get("appId"),
        payload.get("id"),
        app_mapping.get("id"),
        app_mapping.get("app_id"),
        app_mapping.get("appId"),
    )
    for candidate in candidates:
        value = str(candidate or "").strip()
        if value:
            return value
    return ""


def app_status_command(*, app_name: str, app_id: str, account_id: str, account_name: str) -> list[str]:
    command = ["spin", "aka", "app", "status"]
    if app_id:
        command.extend(["--app-id", app_id])
    else:
        command.extend(["--app-name", app_name])
    if account_id:
        command.extend(["--account-id", account_id])
    elif account_name:
        command.extend(["--account-name", account_name])
    command.extend(["--format", "json"])
    return command


def fetch_app_status(*, env: dict[str, str], app_name: str, app_id: str, account_id: str, account_name: str) -> dict[str, Any]:
    status = run_command(app_status_command(app_name=app_name, app_id=app_id, account_id=account_id, account_name=account_name), env=env)
    if status.returncode != 0 or not status.stdout.strip():
        return {}
    try:
        return json.loads(status.stdout)
    except json.JSONDecodeError:
        return {"raw_status_output": status.stdout.strip()}


MAKE_OVERRIDE_KEYS = (
    "SHUMA_ENTERPRISE_MULTI_INSTANCE",
    "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
    "SHUMA_EDGE_INTEGRATION_MODE",
    "SHUMA_GATEWAY_DEPLOYMENT_PROFILE",
    "SHUMA_GATEWAY_UPSTREAM_ORIGIN",
    "SHUMA_GATEWAY_TLS_STRICT",
    "SHUMA_GATEWAY_ORIGIN_AUTH_MODE",
    "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME",
    "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE",
    "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
    "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
    "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED",
    "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED",
    "GATEWAY_SURFACE_CATALOG_PATH",
    "SHUMA_SPIN_MANIFEST",
)


def run_make_target(target: str, env: dict[str, str]) -> None:
    command = ["make", "--no-print-directory"]
    for key in MAKE_OVERRIDE_KEYS:
        value = env.get(key, "")
        if value:
            command.append(f"{key}={value}")
    command.append(target)
    result = run_command(command, env=env, cwd=REPO_ROOT)
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or f"make {target} failed")


def run_interactive_command(
    command: Sequence[str],
    *,
    env: dict[str, str] | None = None,
    cwd: Path | None = None,
) -> subprocess.CompletedProcess[str]:
    master_fd, slave_fd = pty.openpty()
    try:
        process = subprocess.Popen(
            list(command),
            cwd=str(cwd or REPO_ROOT),
            env=env,
            stdin=slave_fd,
            stdout=slave_fd,
            stderr=slave_fd,
            text=False,
            close_fds=True,
        )
    finally:
        os.close(slave_fd)

    output = bytearray()
    try:
        while True:
            ready, _, _ = select.select([master_fd], [], [], 0.25)
            process_exited = process.poll() is not None
            if master_fd in ready:
                try:
                    chunk = os.read(master_fd, 4096)
                except OSError:
                    chunk = b""
                if chunk:
                    output.extend(chunk)
                    sys.stdout.buffer.write(chunk)
                    sys.stdout.buffer.flush()
                elif process_exited:
                    break
            if process_exited and master_fd not in ready:
                break
    finally:
        os.close(master_fd)

    return subprocess.CompletedProcess(
        args=list(command),
        returncode=process.wait(),
        stdout=output.decode("utf-8", errors="replace"),
        stderr="",
    )


def git_head() -> str:
    result = run_command(["git", "rev-parse", "HEAD"], cwd=REPO_ROOT)
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or "Failed to resolve git HEAD.")
    return result.stdout.strip()


def deploy_variable_args(env: dict[str, str]) -> list[str]:
    args: list[str] = []
    for env_key in FERMYON_EDGE_RUNTIME_ENV_KEYS:
        value = env.get(env_key, "").strip()
        if not value:
            continue
        args.extend(["--variable", f"{spin_variable_name(env_key)}={value}"])
    return args


def export_seeded_config_payload(env: dict[str, str]) -> dict[str, Any]:
    result = run_command(
        ["bash", str(REPO_ROOT / "scripts" / "config_seed.sh"), "--print-json"],
        env=env,
        cwd=REPO_ROOT,
    )
    if result.returncode != 0:
        raise SystemExit(
            result.stderr.strip() or result.stdout.strip() or "Failed to export canonical seeded config JSON."
        )
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Canonical seeded config JSON was invalid: {exc}") from exc
    if not isinstance(payload, dict):
        raise SystemExit("Canonical seeded config export must be a JSON object.")
    return payload


def app_urls(status_payload: dict[str, Any]) -> list[str]:
    urls = status_payload.get("urls") if isinstance(status_payload, dict) else None
    if not isinstance(urls, list):
        return []
    return [str(url).strip() for url in urls if str(url or "").strip()]


def http_text_request(
    *,
    method: str,
    url: str,
    headers: dict[str, str] | None = None,
    body: bytes | None = None,
    timeout_seconds: int = 30,
    opener: Any | None = None,
) -> tuple[int, str]:
    request = urllib_request.Request(url, data=body, method=method.upper())
    for key, value in (headers or {}).items():
        request.add_header(key, value)
    try:
        if opener is None:
            response_context = urllib_request.urlopen(request, timeout=timeout_seconds)
        else:
            response_context = opener.open(request, timeout=timeout_seconds)
        with response_context as response:
            return response.getcode(), response.read().decode("utf-8", errors="replace")
    except urllib_error.HTTPError as exc:
        return exc.code, exc.read().decode("utf-8", errors="replace")
    except urllib_error.URLError as exc:
        raise SystemExit(f"HTTP request failed for {url}: {exc}") from exc


def admin_auth_headers(env: dict[str, str], base_url: str) -> dict[str, str]:
    return {
        "Authorization": f"Bearer {required_env_value(env, 'SHUMA_API_KEY')}",
        "Origin": base_url.rstrip("/"),
    }


def edge_cron_path_and_query(env: dict[str, str]) -> str:
    secret = required_env_value(env, "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")
    query = urllib_parse.urlencode({EDGE_CRON_SECRET_QUERY_KEY: secret})
    return f"/shuma/internal/adversary-sim/beat?{query}"


def redacted_edge_cron_path_and_query() -> str:
    return f"/shuma/internal/adversary-sim/beat?{EDGE_CRON_SECRET_QUERY_KEY}=<redacted>"


def cron_scope_args(*, app_id: str, account_id: str, account_name: str) -> list[str]:
    args: list[str] = []
    if app_id:
        args.extend(["--app-id", app_id])
    if account_id:
        args.extend(["--account-id", account_id])
    elif account_name:
        args.extend(["--account-name", account_name])
    return args


def listed_cron_job_names(output: str) -> list[str]:
    names: list[str] = []
    for raw_line in output.splitlines():
        line = raw_line.strip()
        if not line:
            continue
        if line.startswith("+"):
            continue
        if line.startswith("|"):
            columns = [column.strip() for column in line.strip("|").split("|")]
            if not columns:
                continue
            first_column = columns[0]
            if not first_column or first_column.lower() == "name":
                continue
            names.append(first_column)
            continue
        first_token = line.split()[0].strip()
        if first_token and first_token.lower() != "name":
            names.append(first_token)
    return names


def listed_cron_job_names_for_scope(*, env: dict[str, str], app_id: str, account_id: str, account_name: str) -> set[str]:
    list_command = ["spin", "aka", "cron", "list", *cron_scope_args(app_id=app_id, account_id=account_id, account_name=account_name)]
    listed = run_command(list_command, env=env, cwd=REPO_ROOT)
    if listed.returncode != 0:
        raise SystemExit(listed.stderr.strip() or listed.stdout.strip() or "spin aka cron list failed")
    return set(listed_cron_job_names(listed.stdout))


def edge_cron_jobs(env: dict[str, str]) -> list[dict[str, str]]:
    path_and_query = edge_cron_path_and_query(env)
    jobs: list[dict[str, str]] = []
    for index, schedule in enumerate(EDGE_CRON_SCHEDULES):
        jobs.append(
            {
                "name": f"{EDGE_CRON_JOB_NAME_PREFIX}-{index}",
                "schedule": schedule,
                "path_and_query": path_and_query,
            }
        )
    return jobs


def ensure_adversary_sim_edge_cron(*, env: dict[str, str], app_id: str, account_id: str, account_name: str) -> dict[str, str]:
    scope_args = cron_scope_args(app_id=app_id, account_id=account_id, account_name=account_name)
    list_command = ["spin", "aka", "cron", "list", *scope_args]
    existing_names = listed_cron_job_names_for_scope(
        env=env,
        app_id=app_id,
        account_id=account_id,
        account_name=account_name,
    )
    managed_jobs = edge_cron_jobs(env)
    managed_names = {job["name"] for job in managed_jobs}
    managed_names.add(EDGE_CRON_JOB_NAME_PREFIX)

    for existing_name in sorted(existing_names):
        if existing_name not in managed_names and not existing_name.startswith(f"{EDGE_CRON_JOB_NAME_PREFIX}-"):
            continue
        delete_command = ["spin", "aka", "cron", "delete", *scope_args, existing_name]
        deleted = run_command(delete_command, env=env, cwd=REPO_ROOT)
        if deleted.returncode != 0:
            remaining_names = listed_cron_job_names_for_scope(
                env=env,
                app_id=app_id,
                account_id=account_id,
                account_name=account_name,
            )
            if existing_name not in remaining_names:
                continue
            raise SystemExit(
                deleted.stderr.strip() or deleted.stdout.strip() or f"spin aka cron delete failed for {existing_name}"
            )

    for job in managed_jobs:
        create_command = [
            "spin",
            "aka",
            "cron",
            "create",
            *scope_args,
            "--name",
            job["name"],
            "--schedule",
            job["schedule"],
            "--path-and-query",
            job["path_and_query"],
        ]
        created = run_command(create_command, env=env, cwd=REPO_ROOT)
        if created.returncode != 0:
            raise SystemExit(
                created.stderr.strip() or created.stdout.strip() or f"spin aka cron create failed for {job['name']}"
            )

    verified_names = listed_cron_job_names_for_scope(
        env=env,
        app_id=app_id,
        account_id=account_id,
        account_name=account_name,
    )
    if any(job["name"] not in verified_names for job in managed_jobs):
        raise SystemExit("Fermyon edge cron verification failed: expected beat jobs were not present after create")

    return {
        "job_name_prefix": EDGE_CRON_JOB_NAME_PREFIX,
        "job_count": len(managed_jobs),
        "schedules": [job["schedule"] for job in managed_jobs],
        "path_and_query": redacted_edge_cron_path_and_query(),
    }


def admin_session_opener(base_url: str, env: dict[str, str]) -> tuple[Any, str]:
    cookie_jar = http.cookiejar.CookieJar()
    opener = urllib_request.build_opener(urllib_request.HTTPCookieProcessor(cookie_jar))
    login_body = urllib_parse.urlencode(
        {
            "username": "admin",
            "password": required_env_value(env, "SHUMA_API_KEY"),
            "next": "/shuma/dashboard/index.html",
        }
    ).encode("utf-8")
    login_status, login_response_body = http_text_request(
        method="POST",
        url=f"{base_url.rstrip('/')}/shuma/admin/login",
        headers={
            "Content-Type": "application/x-www-form-urlencoded",
            "Origin": base_url.rstrip("/"),
        },
        body=login_body,
        opener=opener,
    )
    if login_status not in {200, 303}:
        raise SystemExit(
            f"Edge admin login smoke failed for {base_url}: status={login_status} body={login_response_body.strip()[:200]}"
        )

    session_status, session_body = http_text_request(
        method="GET",
        url=f"{base_url.rstrip('/')}/shuma/admin/session",
        headers={"Origin": base_url.rstrip("/")},
        opener=opener,
    )
    if session_status != 200:
        raise SystemExit(
            f"Edge admin session smoke failed for {base_url}: status={session_status} body={session_body.strip()[:200]}"
        )
    try:
        payload = json.loads(session_body)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Edge admin session response was invalid JSON: {exc}") from exc
    csrf_token = str(payload.get("csrf_token") or "").strip()
    if not csrf_token:
        raise SystemExit("Edge admin session response did not include csrf_token")
    return opener, csrf_token


def admin_json_request(
    *,
    opener: Any,
    method: str,
    url: str,
    csrf_token: str | None = None,
    origin: str,
    payload: dict[str, Any] | None = None,
    timeout_seconds: int = 30,
) -> tuple[int, dict[str, Any], str]:
    headers = {"Origin": origin}
    body = None
    if csrf_token:
        headers["X-Shuma-CSRF"] = csrf_token
    if payload is not None:
        headers["Content-Type"] = "application/json"
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
    if url.endswith("/shuma/admin/adversary-sim/control"):
        headers["Idempotency-Key"] = str(uuid.uuid4())
    status, text = http_text_request(
        method=method,
        url=url,
        headers=headers,
        body=body,
        opener=opener,
        timeout_seconds=timeout_seconds,
    )
    try:
        parsed = json.loads(text) if text else {}
    except json.JSONDecodeError:
        parsed = {}
    return status, parsed, text


def smoke_adversary_sim_generation(base_url: str, env: dict[str, str]) -> None:
    opener, csrf_token = admin_session_opener(base_url, env)
    origin = base_url.rstrip("/")
    status_url = f"{origin}/shuma/admin/adversary-sim/status"
    control_url = f"{origin}/shuma/admin/adversary-sim/control"
    monitoring_delta_base_url = f"{origin}/shuma/admin/monitoring/delta?hours=24&limit=20"

    def monitoring_delta_contains_simulation_event(payload: dict[str, Any]) -> bool:
        events = payload.get("events") if isinstance(payload, dict) else []
        if not isinstance(events, list):
            return False
        return any(
            isinstance(event, dict) and bool(event.get("is_simulation"))
            for event in events
        )

    def disable_if_running() -> None:
        try:
            _, status_payload, _ = admin_json_request(
                opener=opener,
                method="GET",
                url=status_url,
                origin=origin,
            )
            if bool(status_payload.get("adversary_sim_enabled")):
                admin_json_request(
                    opener=opener,
                    method="POST",
                    url=control_url,
                    csrf_token=csrf_token,
                    origin=origin,
                    payload={"enabled": False, "reason": "fermyon_edge_deploy_smoke"},
                    timeout_seconds=EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_SECONDS,
                )
        except SystemExit:
            raise
        except Exception:
            return

    baseline_monitoring_status, baseline_monitoring_payload, baseline_monitoring_raw = admin_json_request(
        opener=opener,
        method="GET",
        url=monitoring_delta_base_url,
        origin=origin,
    )
    if baseline_monitoring_status != 200:
        raise SystemExit(
            "Edge adversary-sim monitoring baseline smoke failed for "
            f"{base_url}: status={baseline_monitoring_status} body={baseline_monitoring_raw.strip()[:200]}"
        )
    baseline_monitoring_cursor = str(
        baseline_monitoring_payload.get("window_end_cursor")
        or baseline_monitoring_payload.get("next_cursor")
        or ""
    ).strip()

    enable_status, enable_payload, enable_body = admin_json_request(
        opener=opener,
        method="POST",
        url=control_url,
        csrf_token=csrf_token,
        origin=origin,
        payload={"enabled": True, "reason": "fermyon_edge_deploy_smoke"},
        timeout_seconds=EDGE_ADVERSARY_SIM_CONTROL_TIMEOUT_SECONDS,
    )
    if enable_status != 200:
        raise SystemExit(
            f"Edge adversary-sim enable smoke failed for {base_url}: status={enable_status} body={enable_body.strip()[:200]}"
        )

    def lifecycle_supervisor(payload: dict[str, Any]) -> dict[str, Any]:
        lifecycle = payload.get("lifecycle_diagnostics") if isinstance(payload, dict) else {}
        lifecycle_obj = lifecycle if isinstance(lifecycle, dict) else {}
        supervisor = lifecycle_obj.get("supervisor")
        return supervisor if isinstance(supervisor, dict) else {}

    baseline_status, baseline_payload, baseline_raw = admin_json_request(
        opener=opener,
        method="GET",
        url=status_url,
        origin=origin,
    )
    if baseline_status != 200:
        raise SystemExit(
            f"Edge adversary-sim baseline status smoke failed for {base_url}: status={baseline_status} body={baseline_raw.strip()[:200]}"
        )
    baseline_supervisor = lifecycle_supervisor(baseline_payload)
    baseline_tick_count = int(baseline_supervisor.get("generated_tick_count") or 0)
    baseline_request_count = int(baseline_supervisor.get("generated_request_count") or 0)
    baseline_last_beat_at = int(baseline_supervisor.get("last_successful_beat_at") or 0)

    deadline = time.time() + EDGE_ADVERSARY_SIM_SMOKE_TIMEOUT_SECONDS
    last_payload: dict[str, Any] = {}
    last_raw = ""
    monitoring_visible = False
    monitoring_raw = ""
    try:
        while time.time() < deadline:
            status_code, payload, raw = admin_json_request(
                opener=opener,
                method="GET",
                url=status_url,
                origin=origin,
            )
            last_payload = payload
            last_raw = raw
            if status_code == 200:
                supervisor = lifecycle_supervisor(payload)
                tick_count = int(supervisor.get("generated_tick_count") or 0)
                request_count = int(supervisor.get("generated_request_count") or 0)
                last_successful_beat_at = int(supervisor.get("last_successful_beat_at") or 0)
                monitoring_delta_url = monitoring_delta_base_url
                if baseline_monitoring_cursor:
                    monitoring_delta_url = (
                        f"{monitoring_delta_url}&after_cursor={urllib_parse.quote(baseline_monitoring_cursor, safe='')}"
                    )
                monitoring_status, monitoring_payload, monitoring_body = admin_json_request(
                    opener=opener,
                    method="GET",
                    url=monitoring_delta_url,
                    origin=origin,
                )
                monitoring_raw = monitoring_body
                if monitoring_status == 200 and monitoring_delta_contains_simulation_event(monitoring_payload):
                    monitoring_visible = True
                follow_up_tick_observed = (
                    tick_count > baseline_tick_count
                    and request_count > baseline_request_count
                    and last_successful_beat_at > baseline_last_beat_at
                )
                if follow_up_tick_observed and monitoring_visible:
                    return
            time.sleep(5)
    finally:
        disable_if_running()

    raise SystemExit(
        "Edge adversary-sim smoke failed: no generated traffic was observed within "
        f"{EDGE_ADVERSARY_SIM_SMOKE_TIMEOUT_SECONDS}s. Last status: {last_raw[:400]} "
        f"Last monitoring delta: {monitoring_raw[:400]}"
    )


def wait_for_adversary_sim_control_lease_release(base_url: str, env: dict[str, str]) -> None:
    opener, _ = admin_session_opener(base_url, env)
    origin = base_url.rstrip("/")
    status_url = f"{origin}/shuma/admin/adversary-sim/status"
    deadline = time.time() + EDGE_CONTROL_LEASE_RELEASE_TIMEOUT_SECONDS
    last_raw = ""

    while time.time() < deadline:
        status_code, payload, raw = admin_json_request(
            opener=opener,
            method="GET",
            url=status_url,
            origin=origin,
        )
        last_raw = raw
        if status_code == 200:
            lease = payload.get("controller_lease") if isinstance(payload, dict) else None
            lease_obj = lease if isinstance(lease, dict) else {}
            lease_expires_at = int(lease_obj.get("expires_at") or 0)
            enabled = bool(payload.get("adversary_sim_enabled"))
            phase = str(payload.get("phase") or "").strip().lower()
            lease_released = not lease_obj or lease_expires_at <= int(time.time())
            if lease_released and not enabled and phase == "off":
                return
        time.sleep(1)

    raise SystemExit(
        "Edge adversary-sim controller lease did not drain before external dashboard smoke. "
        f"Last status: {last_raw[:400]}"
    )


def smoke_external_dashboard(base_url: str, env: dict[str, str]) -> None:
    command = [
        "make",
        "--no-print-directory",
        f"SHUMA_BASE_URL={base_url}",
        f"SHUMA_API_KEY={required_env_value(env, 'SHUMA_API_KEY')}",
        f"SHUMA_FORWARDED_IP_SECRET={required_env_value(env, 'SHUMA_FORWARDED_IP_SECRET')}",
        "test-dashboard-e2e-external",
    ]
    result = run_interactive_command(command, env=env, cwd=REPO_ROOT)
    if result.returncode != 0:
        raise SystemExit(result.stdout.strip() or "External dashboard live smoke failed.")


def bootstrap_remote_config_if_missing(base_url: str, env: dict[str, str]) -> None:
    config_url = f"{base_url.rstrip('/')}/shuma/admin/config"
    bootstrap_url = f"{base_url.rstrip('/')}/shuma/admin/config/bootstrap"
    headers = admin_auth_headers(env, base_url)
    status, body = http_text_request(method="GET", url=config_url, headers=headers)
    if status == 200:
        return
    if status != 500 or "missing KV config" not in body:
        raise SystemExit(
            f"Edge config bootstrap probe failed for {config_url}: status={status} body={body.strip() or '<empty>'}"
        )

    payload = export_seeded_config_payload(env)
    post_headers = {
        **headers,
        "Content-Type": "application/json",
    }
    post_status, post_body = http_text_request(
        method="POST",
        url=bootstrap_url,
        headers=post_headers,
        body=json.dumps(payload, separators=(",", ":")).encode("utf-8"),
    )
    if post_status != 200:
        raise SystemExit(
            f"Edge config bootstrap failed for {bootstrap_url}: status={post_status} body={post_body.strip() or '<empty>'}"
        )

    verify_status, verify_body = http_text_request(method="GET", url=config_url, headers=headers)
    if verify_status != 200:
        raise SystemExit(
            f"Edge config bootstrap verification failed for {config_url}: status={verify_status} body={verify_body.strip() or '<empty>'}"
        )


def public_route_smoke_acceptable(status: int, body: str) -> bool:
    normalized_body = (body or "").lower()
    if "configuration unavailable" in normalized_body:
        return False
    if status in {200, 403, 429}:
        return True
    if status in {301, 302, 303, 307, 308}:
        return True
    return False


def smoke_deployed_app(base_url: str, env: dict[str, str]) -> None:
    login_status, login_body = http_text_request(
        method="GET",
        url=f"{base_url.rstrip('/')}/shuma/dashboard/login.html",
    )
    if login_status != 200 or "<!doctype html>" not in login_body.lower():
        raise SystemExit(
            f"Edge dashboard login smoke failed for {base_url}: status={login_status}"
        )

    index_status, index_body = http_text_request(
        method="GET",
        url=f"{base_url.rstrip('/')}/index.html",
    )
    if not public_route_smoke_acceptable(index_status, index_body):
        raise SystemExit(
            f"Edge public-route smoke failed for {base_url}: status={index_status} body={index_body.strip()[:200]}"
        )

    config_status, config_body = http_text_request(
        method="GET",
        url=f"{base_url.rstrip('/')}/shuma/admin/config",
        headers=admin_auth_headers(env, base_url),
    )
    if config_status != 200:
        raise SystemExit(
            f"Edge admin-config smoke failed for {base_url}: status={config_status} body={config_body.strip()[:200]}"
        )


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    env_file = Path(args.env_file).expanduser().resolve()
    ensure_env_file(env_file)
    env_file_values = read_env_files(DEFAULTS_ENV_FILE, env_file)

    setup_receipt_path = Path(args.setup_receipt).expanduser().resolve()
    receipt = load_receipt(setup_receipt_path)
    rendered_manifest_path = resolve_rendered_manifest_path(receipt)
    rendered_manifest_path.parent.mkdir(parents=True, exist_ok=True)

    ensure_aka_plugin()
    env = deploy_env(receipt, env_file_values, setup_receipt_path)
    render_manifest(receipt, rendered_manifest_path, env)
    env["SHUMA_SPIN_MANIFEST"] = str(rendered_manifest_path)

    account_info, auth_mode = ensure_aka_session(env)
    run_make_target("deploy-enterprise-akamai", env)
    run_make_target("test-gateway-profile-edge", env)
    run_make_target("smoke-gateway-mode", env)

    account_id = receipt["fermyon"].get("account_id", "").strip()
    account_name = receipt["fermyon"].get("account_name", "").strip()
    app_name = receipt["fermyon"]["app_name"]

    if args.preflight_only:
        print(f"Preflight passed with setup receipt: {setup_receipt_path}")
        return 0

    deploy_receipt_path = Path(args.deploy_receipt_output).expanduser().resolve()
    previous_deploy_receipt = load_previous_deploy_receipt(deploy_receipt_path)
    existing_app_id = str(previous_deploy_receipt.get("fermyon", {}).get("app_id", "")).strip()
    if not existing_app_id:
        existing_app_id = extract_app_id(previous_deploy_receipt.get("fermyon", {}).get("status", {}))
    if not existing_app_id:
        existing_app_id = extract_app_id(
            fetch_app_status(
                env=env,
                app_name=app_name,
                app_id="",
                account_id=account_id,
                account_name=account_name,
            )
        )

    deploy_command = ["spin", "aka", "deploy", "-f", str(rendered_manifest_path), "--no-confirm"]
    if account_id:
        deploy_command.extend(["--account-id", account_id])
    elif account_name:
        deploy_command.extend(["--account-name", account_name])
    if existing_app_id:
        deploy_command.extend(["--app-id", existing_app_id])
    else:
        deploy_command.extend(["--create-name", app_name])
    deploy_command.extend(deploy_variable_args(env))
    deploy = run_interactive_command(deploy_command, env=env)
    if deploy.returncode != 0:
        stderr = (deploy.stderr or "").strip()
        stdout = (deploy.stdout or "").strip()
        raise SystemExit(stderr or stdout or "spin aka deploy failed")

    status_payload = fetch_app_status(
        env=env,
        app_name=app_name,
        app_id=existing_app_id,
        account_id=account_id,
        account_name=account_name,
    )
    app_id = existing_app_id or extract_app_id(status_payload)
    urls = app_urls(status_payload)
    if not urls:
        raise SystemExit("Fermyon deploy succeeded but app status returned no public URLs.")
    primary_url = urls[0]
    cron_job = ensure_adversary_sim_edge_cron(
        env=env,
        app_id=app_id,
        account_id=account_id,
        account_name=account_name,
    )
    print("Smoke step 1/4: bootstrap remote config if missing")
    bootstrap_remote_config_if_missing(primary_url, env)
    print("Smoke step 2/4: verify dashboard/public/admin routes")
    smoke_deployed_app(primary_url, env)
    print("Smoke step 3/4: verify adversary sim generation (enable -> first cron beat -> monitoring visibility)")
    smoke_adversary_sim_generation(primary_url, env)
    wait_for_adversary_sim_control_lease_release(primary_url, env)
    print("Smoke step 4/4: verify external dashboard truthfulness against live edge deployment")
    smoke_external_dashboard(primary_url, env)

    deploy_receipt = {
        "schema": "shuma.fermyon.akamai_edge_deploy.v1",
        "generated_at_utc": utc_now_iso(),
        "setup_receipt_path": str(setup_receipt_path),
        "spin_version": parse_version_line(run_command(["spin", "--version"]).stdout),
        "aka_plugin_version": parse_version_line(run_command(["spin", "aka", "--version"]).stdout),
        "git_head": git_head(),
        "fermyon": {
            "account_id": account_id,
            "account_name": account_name,
            "app_id": app_id,
            "app_name": app_name,
            "primary_url": primary_url,
            "auth_mode": auth_mode,
            "cron": cron_job,
            "info": account_info,
            "status": status_payload,
        },
        "gateway": {
            "upstream_origin": receipt["gateway"]["upstream_origin"],
            "rendered_manifest_path": str(rendered_manifest_path),
            "surface_catalog_path": receipt["gateway"]["surface_catalog_path"],
        },
    }
    write_json(deploy_receipt_path, deploy_receipt)
    print(f"Deploy receipt written: {deploy_receipt_path}")
    print(f"App name: {app_name}")
    for url in urls:
        print(f"App URL: {url}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
