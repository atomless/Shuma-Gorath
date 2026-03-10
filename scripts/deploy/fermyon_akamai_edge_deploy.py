"""Agent-oriented Fermyon / Akamai edge deploy helper."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Sequence

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
from scripts.deploy.local_env import ensure_env_file, read_env_file, read_env_value
from scripts.deploy.setup_common import utc_now_iso, write_json


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
        ],
        env=os.environ.copy(),
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or "Failed to render edge Spin manifest.")


def deploy_env(receipt: dict[str, Any], env_file_values: dict[str, str], setup_receipt_path: Path) -> dict[str, str]:
    gateway = receipt["gateway"]
    env = os.environ.copy()
    env.update(env_file_values)
    env["SHUMA_RUNTIME_ENV"] = gateway.get("runtime_env", DEFAULT_RUNTIME_ENV)
    env["SHUMA_ENTERPRISE_MULTI_INSTANCE"] = "true" if gateway.get("enterprise_multi_instance") else "false"
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


def run_make_target(target: str, env: dict[str, str]) -> None:
    result = run_command(["make", "--no-print-directory", target], env=env, cwd=REPO_ROOT)
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or f"make {target} failed")


def git_head() -> str:
    result = run_command(["git", "rev-parse", "HEAD"], cwd=REPO_ROOT)
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or "Failed to resolve git HEAD.")
    return result.stdout.strip()


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    env_file = Path(args.env_file).expanduser().resolve()
    ensure_env_file(env_file)
    env_file_values = read_env_file(env_file)

    setup_receipt_path = Path(args.setup_receipt).expanduser().resolve()
    receipt = load_receipt(setup_receipt_path)
    rendered_manifest_path = Path(receipt["artifacts"]["rendered_manifest_path"]).expanduser().resolve()
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
    deploy = run_command(deploy_command, env=env)
    if deploy.returncode != 0:
        raise SystemExit(deploy.stderr.strip() or deploy.stdout.strip() or "spin aka deploy failed")

    status_payload = fetch_app_status(
        env=env,
        app_name=app_name,
        app_id=existing_app_id,
        account_id=account_id,
        account_name=account_name,
    )
    app_id = existing_app_id or extract_app_id(status_payload)

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
            "auth_mode": auth_mode,
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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
