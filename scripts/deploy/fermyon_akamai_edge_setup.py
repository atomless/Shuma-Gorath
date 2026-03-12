"""Agent-oriented Fermyon / Akamai edge setup helper."""

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

from scripts.deploy.local_env import ensure_env_file, read_env_value, upsert_env_value
from scripts.deploy.setup_common import (
    DEFAULT_ENV_FILE,
    ensure_secret_value,
    is_interactive_session,
    prompt_secret,
    resolve_admin_allowlist,
    resolve_catalog_output,
    utc_now_iso,
    write_json,
)
from scripts.site_surface_catalog import SUPPORTED_MODES, build_payload

DEFAULT_RECEIPT_PATH = REPO_ROOT / ".shuma" / "fermyon-akamai-edge-setup.json"
DEFAULT_DEPLOY_RECEIPT_PATH = REPO_ROOT / ".shuma" / "fermyon-akamai-edge-deploy.json"
DEFAULT_RENDERED_MANIFEST_PATH = REPO_ROOT / "spin.fermyon-akamai-edge.toml"
SETUP_RECEIPT_SCHEMA = "shuma.fermyon.akamai_edge_setup.v2"
DEFAULT_APP_NAME = "shuma-gorath"
DEFAULT_ORIGIN_AUTH_HEADER_NAME = "x-shuma-origin-auth"
DEFAULT_EDGE_INTEGRATION_MODE = "additive"
DEFAULT_RUNTIME_ENV = "runtime-prod"
SPIN_AKA_PLUGIN = "aka"
SPIN_AKA_TOKEN_KEY = "SPIN_AKA_ACCESS_TOKEN"
PLUGIN_PANIC_MARKERS = (
    "thread 'main' panicked",
    "index out of bounds",
    "plugin/src/commands/login.rs",
)


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare a Fermyon / Akamai edge deployment handoff receipt for Shuma."
    )
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE), help="gitignored local env file")
    parser.add_argument(
        "--receipt-output",
        default=str(DEFAULT_RECEIPT_PATH),
        help="Where to write the Fermyon/Akamai setup receipt",
    )
    parser.add_argument(
        "--deploy-receipt-output",
        default=str(DEFAULT_DEPLOY_RECEIPT_PATH),
        help="Where deploy helper should persist the live deploy receipt",
    )
    parser.add_argument(
        "--rendered-manifest-output",
        default=str(DEFAULT_RENDERED_MANIFEST_PATH),
        help="Where deploy helper should render the deployment-specific Spin manifest",
    )
    parser.add_argument("--fermyon-token", help="Fermyon personal access token override")
    parser.add_argument("--no-store-token", action="store_true", help="Do not persist the token to the env file")
    parser.add_argument("--account-id", help="Explicit Fermyon account id for deploy")
    parser.add_argument("--account-name", help="Explicit Fermyon account name for deploy")
    parser.add_argument("--app-name", default=DEFAULT_APP_NAME, help="Desired Fermyon app name")
    parser.add_argument("--edge-hostname", help="Expected public hostname for the Akamai edge app")
    parser.add_argument("--staging-hostname", help="Optional Akamai staging hostname/property host")
    parser.add_argument("--upstream-origin", help="Existing HTTPS origin protected by edge Shuma")
    parser.add_argument("--admin-ip", help="Explicit SHUMA_ADMIN_IP_ALLOWLIST value")
    parser.add_argument(
        "--origin-lock-confirmed",
        choices=("true", "false"),
        help="Explicit SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED attestation",
    )
    parser.add_argument(
        "--reserved-route-collision-check-passed",
        choices=("true", "false"),
        help="Explicit SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED attestation",
    )
    parser.add_argument(
        "--admin-edge-rate-limits-confirmed",
        choices=("true", "false"),
        help="Explicit SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED attestation",
    )
    parser.add_argument(
        "--admin-api-key-rotation-confirmed",
        choices=("true", "false"),
        help="Explicit SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED attestation",
    )
    parser.add_argument(
        "--enterprise-unsynced-state-exception-confirmed",
        choices=("true", "false"),
        help="Explicit SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED attestation for temporary additive/off edge rollout without distributed state",
    )
    parser.add_argument("--docroot", help="Local site docroot for surface-catalog generation")
    parser.add_argument(
        "--site-mode",
        default="auto",
        choices=sorted(SUPPORTED_MODES),
        help="Public-surface catalog mode when --docroot is provided",
    )
    parser.add_argument("--catalog-output", help="Where to write the generated surface catalog")
    parser.add_argument("--surface-catalog-path", help="Use an existing surface-catalog JSON instead of building one")
    parser.add_argument("--yes", action="store_true", help="Accept safe detected defaults without prompting")
    return parser.parse_args(argv)


def resolve_token(args: argparse.Namespace, env_file: Path) -> tuple[str, str]:
    if args.fermyon_token and args.fermyon_token.strip():
        return args.fermyon_token.strip(), "argument"
    if SPIN_AKA_TOKEN_KEY in os.environ and os.environ[SPIN_AKA_TOKEN_KEY].strip():
        return os.environ[SPIN_AKA_TOKEN_KEY].strip(), "environment"
    persisted = read_env_value(env_file, SPIN_AKA_TOKEN_KEY).strip()
    if persisted:
        return persisted, "env_file"
    if is_interactive_session():
        token = prompt_secret("Fermyon Personal Access Token (hidden): ").strip()
        if token:
            return token, "prompt"
    raise SystemExit(
        f"{SPIN_AKA_TOKEN_KEY} is missing. Add it to .env.local, export it, or rerun interactively to paste it."
    )


def ensure_required_https_origin(args: argparse.Namespace, env_file: Path) -> str:
    candidate = (
        args.upstream_origin
        or os.environ.get("SHUMA_GATEWAY_UPSTREAM_ORIGIN", "").strip()
        or read_env_value(env_file, "SHUMA_GATEWAY_UPSTREAM_ORIGIN").strip()
    )
    if not candidate:
        raise SystemExit(
            "SHUMA_GATEWAY_UPSTREAM_ORIGIN is missing. Pass --upstream-origin or set it in .env.local."
        )
    normalized = candidate.strip()
    if not normalized.startswith("https://"):
        raise SystemExit("SHUMA_GATEWAY_UPSTREAM_ORIGIN must use https:// for edge-fermyon posture.")
    return normalized


def upsert_bool_env_value(env_file: Path, key: str, value: bool) -> None:
    upsert_env_value(env_file, key, "true" if value else "false")


def resolve_attestation(
    *,
    explicit_value: str,
    env_file: Path,
    key: str,
) -> bool:
    for candidate in (
        explicit_value,
        os.environ.get(key, "").strip(),
        read_env_value(env_file, key).strip(),
    ):
        normalized = candidate.lower()
        if normalized in {"true", "1", "yes", "on"}:
            return True
        if normalized in {"false", "0", "no", "off"}:
            return False
    return False


def resolve_surface_catalog(args: argparse.Namespace, env_file: Path) -> tuple[str, str]:
    explicit_path = args.surface_catalog_path or read_env_value(env_file, "GATEWAY_SURFACE_CATALOG_PATH").strip()
    if explicit_path:
        path = Path(explicit_path).expanduser().resolve()
        if not path.exists():
            raise SystemExit(f"GATEWAY_SURFACE_CATALOG_PATH does not exist: {path}")
        return str(path), "existing"

    if not args.docroot:
        raise SystemExit(
            "A surface catalog is required. Pass --surface-catalog-path or provide --docroot so setup can build one."
        )
    docroot = Path(args.docroot).expanduser().resolve()
    if not docroot.is_dir():
        raise SystemExit(f"Docroot does not exist or is not a directory: {docroot}")
    output = resolve_catalog_output(docroot, args.catalog_output)
    payload = build_payload(docroot, args.site_mode)
    write_json(output, payload)
    return str(output), "generated"


def resolve_account_target(
    *,
    explicit_account_id: str,
    explicit_account_name: str,
    account_info: dict[str, Any],
) -> tuple[str, str]:
    account_id = explicit_account_id.strip()
    account_name = explicit_account_name.strip()
    auth_info = account_info.get("auth_info")
    accounts = auth_info.get("accounts") if isinstance(auth_info, dict) else None
    normalized_accounts: list[tuple[str, str]] = []
    if isinstance(accounts, list):
        for account in accounts:
            if not isinstance(account, dict):
                continue
            candidate_id = str(account.get("id", "")).strip()
            candidate_name = str(account.get("name", "")).strip()
            if candidate_id or candidate_name:
                normalized_accounts.append((candidate_id, candidate_name))

    if account_id or account_name:
        for candidate_id, candidate_name in normalized_accounts:
            if account_id and candidate_id == account_id:
                return account_id, account_name or candidate_name
            if account_name and candidate_name == account_name:
                return account_id or candidate_id, account_name
        return account_id, account_name

    if not normalized_accounts:
        return "", ""
    if len(normalized_accounts) == 1:
        return normalized_accounts[0]
    raise SystemExit(
        "Multiple Fermyon accounts are available for this identity. "
        "Rerun setup with --account-id or --account-name to choose the deploy target explicitly."
    )


def run_command(
    command: Sequence[str],
    *,
    env: dict[str, str] | None = None,
    cwd: Path | None = None,
    capture_output: bool = True,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(command),
        cwd=str(cwd or REPO_ROOT),
        env=env,
        capture_output=capture_output,
        text=True,
        check=False,
    )


def ensure_aka_plugin() -> str:
    result = run_command(["spin", "plugins", "list", "--installed", "--format", "json"])
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or result.stdout.strip() or "Failed to inspect installed Spin plugins.")
    try:
        installed = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"Installed Spin plugin list was not valid JSON: {exc}") from exc
    for plugin in installed:
        if plugin.get("name") == SPIN_AKA_PLUGIN and plugin.get("installedVersion"):
            info_help = run_command(["spin", "aka", "info", "--help"])
            if info_help.returncode == 0 and "--format <FORMAT>" in info_help.stdout:
                return str(plugin["installedVersion"])
            upgrade = run_command(["spin", "plugins", "upgrade", "-y", SPIN_AKA_PLUGIN])
            if upgrade.returncode != 0:
                raise SystemExit(
                    upgrade.stderr.strip() or upgrade.stdout.strip() or "Failed to upgrade spin aka plugin."
                )
            version = run_command(["spin", "aka", "--version"])
            if version.returncode != 0:
                raise SystemExit(
                    version.stderr.strip()
                    or version.stdout.strip()
                    or "spin aka remained unavailable after upgrade."
                )
            return parse_version_line(version.stdout)

    install = run_command(["spin", "plugins", "install", "-y", SPIN_AKA_PLUGIN])
    if install.returncode != 0:
        raise SystemExit(install.stderr.strip() or install.stdout.strip() or "Failed to install spin aka plugin.")
    version = run_command(["spin", "aka", "--version"])
    if version.returncode != 0:
        raise SystemExit(version.stderr.strip() or version.stdout.strip() or "spin aka remained unavailable after installation.")
    return version.stdout.strip()


def parse_version_line(output: str) -> str:
    return output.strip().splitlines()[0].strip() if output.strip() else ""


def aka_auth_environment(token: str) -> dict[str, str]:
    env = os.environ.copy()
    env[SPIN_AKA_TOKEN_KEY] = token
    return env


def session_auth_environment() -> dict[str, str]:
    env = os.environ.copy()
    env.pop(SPIN_AKA_TOKEN_KEY, None)
    return env


def fetch_aka_info(env: dict[str, str]) -> dict[str, Any]:
    info = run_command(["spin", "aka", "info", "--format", "json"], env=env)
    if info.returncode != 0:
        reason = "\n".join(part for part in (info.stdout.strip(), info.stderr.strip()) if part) or "spin aka info failed"
        raise SystemExit(reason)
    try:
        return json.loads(info.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"spin aka info did not return valid JSON: {exc}") from exc


def validate_aka_login(token: str) -> tuple[dict[str, Any], str]:
    env = aka_auth_environment(token)
    login = run_command(["spin", "aka", "login"], env=env)
    login_stdout = login.stdout.strip()
    login_stderr = login.stderr.strip()
    login_combined = "\n".join(part for part in (login_stdout, login_stderr) if part)
    if login.returncode != 0:
        reason = login_combined or "spin aka login failed"
        if any(marker in reason for marker in PLUGIN_PANIC_MARKERS):
            if is_interactive_session():
                print(
                    "spin aka token login hit an upstream plugin panic; falling back to interactive device login."
                )
                fallback_env = session_auth_environment()
                fallback = run_command(["spin", "aka", "login"], env=fallback_env, capture_output=False)
                if fallback.returncode != 0:
                    raise SystemExit(
                        "spin aka token login panicked and interactive device login also failed."
                    )
                return fetch_aka_info(fallback_env), "device_login"
            raise SystemExit(
                "spin aka login failed due to an upstream plugin panic. "
                "Current plugin/token path is not usable on this machine yet. "
                f"Observed failure: {reason}"
            )
        raise SystemExit(reason)
    return fetch_aka_info(env), "token"


def ensure_env_secret(env_file: Path, key: str, existing: str = "") -> str:
    current = existing or read_env_value(env_file, key)
    value = ensure_secret_value(current)
    upsert_env_value(env_file, key, value)
    return value


def build_setup_receipt(
    *,
    token_source: str,
    spin_version: str,
    aka_plugin_version: str,
    account_id: str,
    account_name: str,
    app_name: str,
    edge_hostname: str,
    staging_hostname: str,
    account_info: dict[str, Any],
    upstream_origin: str,
    admin_allowlist: str,
    origin_auth_header_name: str,
    origin_lock_confirmed: bool,
    reserved_route_collision_check_passed: bool,
    admin_edge_rate_limits_confirmed: bool,
    admin_api_key_rotation_confirmed: bool,
    enterprise_unsynced_state_exception_confirmed: bool,
    surface_catalog_path: str,
    surface_catalog_source: str,
    deploy_receipt_path: Path,
    rendered_manifest_path: Path,
    status: str,
    auth_mode: str,
    last_completed_step: str,
    blocked_at_step: str,
    blocked_reason: str,
    next_operator_action: str,
) -> dict[str, Any]:
    return {
        "schema": SETUP_RECEIPT_SCHEMA,
        "generated_at_utc": utc_now_iso(),
        "mode": "aka",
        "status": status,
        "auth_mode": auth_mode,
        "token_source": token_source,
        "progress": {
            "last_completed_step": last_completed_step,
            "blocked_at_step": blocked_at_step,
            "blocked_reason": blocked_reason,
            "next_operator_action": next_operator_action,
        },
        "spin": {
            "spin_version": spin_version,
            "aka_plugin_version": aka_plugin_version,
        },
        "fermyon": {
            "account_id": account_id,
            "account_name": account_name,
            "app_name": app_name,
            "edge_hostname": edge_hostname,
            "staging_hostname": staging_hostname,
            "info": account_info,
        },
        "gateway": {
            "runtime_env": DEFAULT_RUNTIME_ENV,
            "deployment_profile": "edge-fermyon",
            "enterprise_multi_instance": True,
            "enterprise_unsynced_state_exception_confirmed": enterprise_unsynced_state_exception_confirmed,
            "edge_integration_mode": DEFAULT_EDGE_INTEGRATION_MODE,
            "upstream_origin": upstream_origin,
            "admin_allowlist": admin_allowlist,
            "tls_strict": True,
            "origin_auth_mode": "signed_header",
            "origin_auth_header_name": origin_auth_header_name,
            "origin_lock_confirmed": origin_lock_confirmed,
            "reserved_route_collision_check_passed": reserved_route_collision_check_passed,
            "admin_edge_rate_limits_confirmed": admin_edge_rate_limits_confirmed,
            "admin_api_key_rotation_confirmed": admin_api_key_rotation_confirmed,
            "surface_catalog_path": surface_catalog_path,
            "surface_catalog_source": surface_catalog_source,
        },
        "artifacts": {
            "deploy_receipt_path": str(deploy_receipt_path),
            "rendered_manifest_path": str(rendered_manifest_path),
        },
    }


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    env_file = Path(args.env_file).expanduser().resolve()
    ensure_env_file(env_file)

    token, token_source = resolve_token(args, env_file)
    if not args.no_store_token and token_source != "env_file":
        upsert_env_value(env_file, SPIN_AKA_TOKEN_KEY, token)

    admin_allowlist = resolve_admin_allowlist(
        explicit_value=args.admin_ip or "",
        env_value=os.environ.get("SHUMA_ADMIN_IP_ALLOWLIST", "").strip(),
        persisted_value=read_env_value(env_file, "SHUMA_ADMIN_IP_ALLOWLIST").strip(),
        accept_detected_default=args.yes,
    )
    upsert_env_value(env_file, "SHUMA_ADMIN_IP_ALLOWLIST", admin_allowlist)

    surface_catalog_path, surface_catalog_source = resolve_surface_catalog(args, env_file)
    upsert_env_value(env_file, "GATEWAY_SURFACE_CATALOG_PATH", surface_catalog_path)

    upstream_origin = ensure_required_https_origin(args, env_file)
    upsert_env_value(env_file, "SHUMA_GATEWAY_UPSTREAM_ORIGIN", upstream_origin)
    origin_lock_confirmed = resolve_attestation(
        explicit_value=args.origin_lock_confirmed or "",
        env_file=env_file,
        key="SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED",
    )
    reserved_route_collision_check_passed = resolve_attestation(
        explicit_value=args.reserved_route_collision_check_passed or "",
        env_file=env_file,
        key="SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
    )
    admin_edge_rate_limits_confirmed = resolve_attestation(
        explicit_value=args.admin_edge_rate_limits_confirmed or "",
        env_file=env_file,
        key="SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED",
    )
    admin_api_key_rotation_confirmed = resolve_attestation(
        explicit_value=args.admin_api_key_rotation_confirmed or "",
        env_file=env_file,
        key="SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED",
    )
    enterprise_unsynced_state_exception_confirmed = resolve_attestation(
        explicit_value=args.enterprise_unsynced_state_exception_confirmed or "",
        env_file=env_file,
        key="SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
    )
    upsert_bool_env_value(env_file, "SHUMA_GATEWAY_ORIGIN_LOCK_CONFIRMED", origin_lock_confirmed)
    upsert_bool_env_value(
        env_file,
        "SHUMA_GATEWAY_RESERVED_ROUTE_COLLISION_CHECK_PASSED",
        reserved_route_collision_check_passed,
    )
    upsert_bool_env_value(
        env_file,
        "SHUMA_ADMIN_EDGE_RATE_LIMITS_CONFIRMED",
        admin_edge_rate_limits_confirmed,
    )
    upsert_bool_env_value(
        env_file,
        "SHUMA_ADMIN_API_KEY_ROTATION_CONFIRMED",
        admin_api_key_rotation_confirmed,
    )
    upsert_bool_env_value(
        env_file,
        "SHUMA_ENTERPRISE_UNSYNCED_STATE_EXCEPTION_CONFIRMED",
        enterprise_unsynced_state_exception_confirmed,
    )
    origin_auth_header_name = (
        read_env_value(env_file, "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME").strip()
        or DEFAULT_ORIGIN_AUTH_HEADER_NAME
    )
    upsert_env_value(env_file, "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_NAME", origin_auth_header_name)
    origin_auth_header_value = ensure_env_secret(env_file, "SHUMA_GATEWAY_ORIGIN_AUTH_HEADER_VALUE")
    ensure_env_secret(env_file, "SHUMA_API_KEY")
    ensure_env_secret(env_file, "SHUMA_JS_SECRET")
    ensure_env_secret(env_file, "SHUMA_FORWARDED_IP_SECRET")
    ensure_env_secret(env_file, "SHUMA_HEALTH_SECRET")
    ensure_env_secret(env_file, "SHUMA_ADVERSARY_SIM_EDGE_CRON_SECRET")
    ensure_env_secret(env_file, "SHUMA_SIM_TELEMETRY_SECRET")

    receipt_path = Path(args.receipt_output).expanduser().resolve()
    deploy_receipt_path = Path(args.deploy_receipt_output).expanduser().resolve()
    rendered_manifest_path = Path(args.rendered_manifest_output).expanduser().resolve()
    spin_version = parse_version_line(run_command(["spin", "--version"]).stdout)
    aka_plugin_version = ensure_aka_plugin()
    account_id = args.account_id or ""
    account_name = args.account_name or ""
    app_name = args.app_name.strip() or DEFAULT_APP_NAME
    edge_hostname = (args.edge_hostname or "").strip()
    staging_hostname = (args.staging_hostname or "").strip()

    try:
        try:
            account_info = fetch_aka_info(session_auth_environment())
            auth_mode = "existing_session"
        except SystemExit:
            account_info, auth_mode = validate_aka_login(token)
        account_id, account_name = resolve_account_target(
            explicit_account_id=args.account_id or "",
            explicit_account_name=args.account_name or "",
            account_info=account_info,
        )
    except SystemExit as exc:
        blocked_receipt = build_setup_receipt(
            token_source=token_source,
            spin_version=spin_version,
            aka_plugin_version=aka_plugin_version,
            account_id=account_id,
            account_name=account_name,
            app_name=app_name,
            edge_hostname=edge_hostname,
            staging_hostname=staging_hostname,
            account_info={},
            upstream_origin=upstream_origin,
            admin_allowlist=admin_allowlist,
            origin_auth_header_name=origin_auth_header_name,
            origin_lock_confirmed=origin_lock_confirmed,
            reserved_route_collision_check_passed=reserved_route_collision_check_passed,
            admin_edge_rate_limits_confirmed=admin_edge_rate_limits_confirmed,
            admin_api_key_rotation_confirmed=admin_api_key_rotation_confirmed,
            enterprise_unsynced_state_exception_confirmed=enterprise_unsynced_state_exception_confirmed,
            surface_catalog_path=surface_catalog_path,
            surface_catalog_source=surface_catalog_source,
            deploy_receipt_path=deploy_receipt_path,
            rendered_manifest_path=rendered_manifest_path,
            status="blocked",
            auth_mode="",
            last_completed_step="local_state_prepared",
            blocked_at_step="auth_validation",
            blocked_reason=str(exc),
            next_operator_action=(
                "Resolve the Fermyon authentication/access blocker, then rerun "
                "`make prepare-fermyon-akamai-edge` to refresh the setup receipt."
            ),
        )
        message = str(exc)
        if "Multiple Fermyon accounts are available" in message:
            blocked_receipt["progress"]["blocked_at_step"] = "account_target_resolution"
            blocked_receipt["progress"]["last_completed_step"] = "auth_validated"
            blocked_receipt["progress"]["next_operator_action"] = (
                "Rerun `make prepare-fermyon-akamai-edge` with --account-id or --account-name "
                "to choose the deploy target explicitly."
            )
        write_json(receipt_path, blocked_receipt)
        raise

    receipt = build_setup_receipt(
        token_source=token_source,
        spin_version=spin_version,
        aka_plugin_version=aka_plugin_version,
        account_id=account_id,
        account_name=account_name,
        app_name=app_name,
        edge_hostname=edge_hostname,
        staging_hostname=staging_hostname,
        account_info=account_info,
        upstream_origin=upstream_origin,
        admin_allowlist=admin_allowlist,
        origin_auth_header_name=origin_auth_header_name,
        origin_lock_confirmed=origin_lock_confirmed,
        reserved_route_collision_check_passed=reserved_route_collision_check_passed,
        admin_edge_rate_limits_confirmed=admin_edge_rate_limits_confirmed,
        admin_api_key_rotation_confirmed=admin_api_key_rotation_confirmed,
        enterprise_unsynced_state_exception_confirmed=enterprise_unsynced_state_exception_confirmed,
        surface_catalog_path=surface_catalog_path,
        surface_catalog_source=surface_catalog_source,
        deploy_receipt_path=deploy_receipt_path,
        rendered_manifest_path=rendered_manifest_path,
        status="ready",
        auth_mode=auth_mode,
        last_completed_step="auth_validated",
        blocked_at_step="",
        blocked_reason="",
        next_operator_action="Run `make deploy-fermyon-akamai-edge` to continue.",
    )
    write_json(receipt_path, receipt)
    print(f"Receipt written: {receipt_path}")
    print(f"Aka plugin: {aka_plugin_version}")
    print(f"Surface catalog: {surface_catalog_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
