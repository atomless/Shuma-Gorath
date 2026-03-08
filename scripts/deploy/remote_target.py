"""Generic day-2 remote target helpers for normalized ssh_systemd receipts."""

from __future__ import annotations

import argparse
import ipaddress
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence

from scripts.deploy.local_env import ensure_env_file, parse_env_text, read_env_file, read_env_value, upsert_env_value

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_DURABLE_STATE_DIR = REPO_ROOT / ".shuma"
DEFAULT_REMOTE_RECEIPTS_DIR = DEFAULT_DURABLE_STATE_DIR / "remotes"
REMOTE_RECEIPT_SCHEMA = "shuma.remote_target.v1"
DEFAULT_BACKEND_KIND = "ssh_systemd"
DEFAULT_APP_DIR = "/opt/shuma-gorath"
DEFAULT_SERVICE_NAME = "shuma-gorath"
DEFAULT_SPIN_MANIFEST_PATH = "/opt/shuma-gorath/spin.gateway.toml"
DEFAULT_SMOKE_PATH = "/health"
RELEASE_BUNDLE_SCRIPT = REPO_ROOT / "scripts" / "deploy" / "build_linode_release_bundle.py"
REMOTE_UPDATE_ARCHIVE_PATH = "/tmp/shuma-remote-update-release.tar.gz"
REMOTE_UPDATE_METADATA_PATH = "/tmp/shuma-remote-update-release.json"
REMOTE_UPDATE_SURFACE_CATALOG_PATH = "/tmp/shuma-remote-update-surface-catalog.json"
REMOTE_UPDATE_SCRIPT_PATH = "/tmp/shuma-remote-update.sh"
REMOTE_SMOKE_ENV_KEYS = (
    "SHUMA_API_KEY",
    "SHUMA_FORWARDED_IP_SECRET",
    "SHUMA_HEALTH_SECRET",
    "SHUMA_ADMIN_IP_ALLOWLIST",
)


def fail(message: str) -> None:
    raise SystemExit(message)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def normalize_remote_name(value: str) -> str:
    normalized = re.sub(r"[^a-z0-9]+", "-", value.strip().lower()).strip("-")
    if not normalized:
        fail("Remote name must contain at least one letter or digit.")
    return normalized


def default_public_base_url(host: str) -> str:
    candidate = host.strip()
    if not candidate:
        fail("Remote host cannot be blank.")
    try:
        ipaddress.ip_address(candidate)
    except ValueError:
        return f"https://{candidate}"
    return f"https://{candidate}.sslip.io"


def build_remote_receipt(
    *,
    name: str,
    provider_kind: str,
    host: str,
    port: int,
    user: str,
    private_key_path: str,
    public_base_url: str,
    surface_catalog_path: str,
    app_dir: str = DEFAULT_APP_DIR,
    service_name: str = DEFAULT_SERVICE_NAME,
    spin_manifest_path: str = DEFAULT_SPIN_MANIFEST_PATH,
    smoke_path: str = DEFAULT_SMOKE_PATH,
    last_deployed_commit: str = "",
    last_deployed_at_utc: str = "",
    provider_extension: dict[str, Any] | None = None,
) -> dict[str, Any]:
    normalized_name = normalize_remote_name(name)
    return {
        "schema": REMOTE_RECEIPT_SCHEMA,
        "identity": {
            "name": normalized_name,
            "backend_kind": DEFAULT_BACKEND_KIND,
            "provider_kind": provider_kind,
        },
        "ssh": {
            "host": host,
            "port": int(port),
            "user": user,
            "private_key_path": private_key_path,
        },
        "runtime": {
            "app_dir": app_dir,
            "service_name": service_name,
            "public_base_url": public_base_url.rstrip("/"),
        },
        "deploy": {
            "spin_manifest_path": spin_manifest_path,
            "surface_catalog_path": surface_catalog_path,
            "smoke_path": smoke_path,
        },
        "metadata": {
            "last_deployed_commit": last_deployed_commit,
            "last_deployed_at_utc": last_deployed_at_utc,
        },
        "provider": provider_extension or {},
    }


def remote_receipt_path(receipts_dir: Path, name: str) -> Path:
    return receipts_dir / f"{normalize_remote_name(name)}.json"


def write_remote_receipt(
    *,
    receipts_dir: Path,
    name: str,
    provider_kind: str,
    host: str,
    port: int = 22,
    user: str = "shuma",
    private_key_path: str,
    public_base_url: str,
    surface_catalog_path: str,
    app_dir: str = DEFAULT_APP_DIR,
    service_name: str = DEFAULT_SERVICE_NAME,
    spin_manifest_path: str = DEFAULT_SPIN_MANIFEST_PATH,
    smoke_path: str = DEFAULT_SMOKE_PATH,
    last_deployed_commit: str = "",
    last_deployed_at_utc: str = "",
    provider_extension: dict[str, Any] | None = None,
) -> Path:
    path = remote_receipt_path(receipts_dir, name)
    receipt = build_remote_receipt(
        name=name,
        provider_kind=provider_kind,
        host=host,
        port=port,
        user=user,
        private_key_path=private_key_path,
        public_base_url=public_base_url,
        surface_catalog_path=surface_catalog_path,
        app_dir=app_dir,
        service_name=service_name,
        spin_manifest_path=spin_manifest_path,
        smoke_path=smoke_path,
        last_deployed_commit=last_deployed_commit,
        last_deployed_at_utc=last_deployed_at_utc,
        provider_extension=provider_extension,
    )
    write_json(path, receipt)
    return path


def _require_mapping(parent: dict[str, Any], key: str) -> dict[str, Any]:
    value = parent.get(key)
    if not isinstance(value, dict):
        fail(f"Invalid remote receipt: {key} must be an object.")
    return value


def _require_string(parent: dict[str, Any], key: str) -> str:
    value = parent.get(key)
    if not isinstance(value, str) or not value.strip():
        fail(f"Invalid remote receipt: {key} must be a non-empty string.")
    return value


def _require_int(parent: dict[str, Any], key: str) -> int:
    value = parent.get(key)
    if not isinstance(value, int):
        fail(f"Invalid remote receipt: {key} must be an integer.")
    return value


def load_remote_receipt(receipts_dir: Path, name: str) -> dict[str, Any]:
    path = remote_receipt_path(receipts_dir, name)
    if not path.exists():
        fail(f"Remote receipt does not exist: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"Remote receipt is not valid JSON: {path} ({exc})")

    if payload.get("schema") != REMOTE_RECEIPT_SCHEMA:
        fail(
            f"Invalid remote receipt schema for {path}: expected {REMOTE_RECEIPT_SCHEMA}, got {payload.get('schema')!r}."
        )

    identity = _require_mapping(payload, "identity")
    ssh = _require_mapping(payload, "ssh")
    runtime = _require_mapping(payload, "runtime")
    deploy = _require_mapping(payload, "deploy")
    metadata = _require_mapping(payload, "metadata")
    provider = _require_mapping(payload, "provider")

    _require_string(identity, "name")
    if _require_string(identity, "backend_kind") != DEFAULT_BACKEND_KIND:
        fail(
            f"Unsupported remote backend: {identity['backend_kind']!r}. "
            f"Only {DEFAULT_BACKEND_KIND!r} is supported in this tranche."
        )
    _require_string(identity, "provider_kind")
    _require_string(ssh, "host")
    _require_int(ssh, "port")
    _require_string(ssh, "user")
    _require_string(ssh, "private_key_path")
    _require_string(runtime, "app_dir")
    _require_string(runtime, "service_name")
    _require_string(runtime, "public_base_url")
    _require_string(deploy, "spin_manifest_path")
    _require_string(deploy, "surface_catalog_path")
    _require_string(deploy, "smoke_path")
    if not isinstance(metadata.get("last_deployed_commit"), str):
        fail("Invalid remote receipt: metadata.last_deployed_commit must be a string.")
    if not isinstance(metadata.get("last_deployed_at_utc"), str):
        fail("Invalid remote receipt: metadata.last_deployed_at_utc must be a string.")
    if not isinstance(provider, dict):
        fail("Invalid remote receipt: provider must be an object.")
    return payload


def resolve_remote_name(explicit_name: str | None, env_file: Path) -> str:
    if explicit_name:
        return normalize_remote_name(explicit_name)
    active = read_env_value(env_file, "SHUMA_ACTIVE_REMOTE")
    if active:
        return normalize_remote_name(active)
    fail("No remote selected. Pass --name or run make remote-use REMOTE=<name> first.")


def activate_remote(env_file: Path, receipts_dir: Path, name: str) -> dict[str, Any]:
    receipt = load_remote_receipt(receipts_dir, name)
    ensure_env_file(env_file)
    upsert_env_value(env_file, "SHUMA_ACTIVE_REMOTE", receipt["identity"]["name"])
    return receipt


def select_remote(explicit_name: str | None, env_file: Path, receipts_dir: Path) -> dict[str, Any]:
    return load_remote_receipt(receipts_dir, resolve_remote_name(explicit_name, env_file))


def ssh_command_for_operation(receipt: dict[str, Any], remote_command: str) -> list[str]:
    ssh = receipt["ssh"]
    return [
        "ssh",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-p",
        str(ssh["port"]),
        "-i",
        ssh["private_key_path"],
        f"{ssh['user']}@{ssh['host']}",
        remote_command,
    ]


def scp_command_for_copy(receipt: dict[str, Any], local_path: Path, remote_path: str) -> list[str]:
    ssh = receipt["ssh"]
    return [
        "scp",
        "-q",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-P",
        str(ssh["port"]),
        "-i",
        ssh["private_key_path"],
        str(local_path),
        f"{ssh['user']}@{ssh['host']}:{remote_path}",
    ]


def run_ssh_operation(receipt: dict[str, Any], remote_command: str) -> int:
    result = subprocess.run(ssh_command_for_operation(receipt, remote_command), check=False)
    return int(result.returncode)


def dashboard_url(receipt: dict[str, Any]) -> str:
    return receipt["runtime"]["public_base_url"].rstrip("/") + "/dashboard"


def open_dashboard(receipt: dict[str, Any]) -> int:
    url = dashboard_url(receipt)
    opener = shutil.which("open") or shutil.which("xdg-open")
    if not opener:
        print(url)
        return 0
    result = subprocess.run([Path(opener).name, url], check=False)
    return int(result.returncode)


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def shell_env_assignments(values: dict[str, str]) -> str:
    return " ".join(f"{key}={shlex.quote(value)}" for key, value in values.items())


def ensure_local_file(path_value: str, description: str) -> Path:
    path = Path(path_value).expanduser()
    if not path.exists():
        fail(f"Missing {description}: {path}")
    return path.resolve()


def first_ip_from_allowlist(raw_allowlist: str) -> str:
    for part in [item.strip() for item in raw_allowlist.split(",") if item.strip()]:
        try:
            if "/" in part:
                network = ipaddress.ip_network(part, strict=False)
                return str(next(iter(network.hosts()), network.network_address))
            return str(ipaddress.ip_address(part))
        except ValueError:
            continue
    return ""


def build_release_bundle(
    *, repo_root: Path, work_dir: Path
) -> tuple[Path, Path, dict[str, Any]]:
    archive_path = work_dir / "shuma-release.tar.gz"
    metadata_path = work_dir / "shuma-release.json"
    result = subprocess.run(
        [
            "python3",
            str(RELEASE_BUNDLE_SCRIPT),
            "--repo-root",
            str(repo_root),
            "--archive-output",
            str(archive_path),
            "--metadata-output",
            str(metadata_path),
        ],
        cwd=str(repo_root),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.stdout.strip():
        print(result.stdout.strip())
    if result.stderr.strip():
        print(result.stderr.strip(), file=sys.stderr)
    if result.returncode != 0:
        fail(result.stderr.strip() or result.stdout.strip() or "Failed to build release bundle.")
    try:
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"Failed to read release bundle metadata: {exc}")
    return archive_path, metadata_path, metadata


def write_remote_update_script(work_dir: Path) -> Path:
    script_path = work_dir / "remote-update.sh"
    script_path.write_text(
        """#!/usr/bin/env bash
set -euo pipefail

ACTION="${1:-install}"
: "${REMOTE_APP_DIR:?missing REMOTE_APP_DIR}"
: "${REMOTE_SERVICE_NAME:?missing REMOTE_SERVICE_NAME}"

NEXT_APP_DIR="${REMOTE_APP_DIR}.next"
PREV_APP_DIR="${REMOTE_APP_DIR}.prev"
FAILED_APP_DIR="${REMOTE_APP_DIR}.failed"

install_release() {
  : "${RELEASE_ARCHIVE_PATH:?missing RELEASE_ARCHIVE_PATH}"
  : "${RELEASE_METADATA_PATH:?missing RELEASE_METADATA_PATH}"
  : "${GATEWAY_SURFACE_CATALOG_REMOTE_PATH:?missing GATEWAY_SURFACE_CATALOG_REMOTE_PATH}"

  if [[ ! -d "${REMOTE_APP_DIR}" ]]; then
    echo "Missing existing remote app dir: ${REMOTE_APP_DIR}" >&2
    exit 1
  fi
  if [[ ! -f "${REMOTE_APP_DIR}/.env.local" ]]; then
    echo "Missing existing remote .env.local: ${REMOTE_APP_DIR}/.env.local" >&2
    exit 1
  fi

  rm -rf "${NEXT_APP_DIR}"
  mkdir -p "${NEXT_APP_DIR}"
  tar -xzf "${RELEASE_ARCHIVE_PATH}" -C "${NEXT_APP_DIR}"
  cp "${REMOTE_APP_DIR}/.env.local" "${NEXT_APP_DIR}/.env.local"
  if [[ -d "${REMOTE_APP_DIR}/.spin" ]]; then
    cp -a "${REMOTE_APP_DIR}/.spin" "${NEXT_APP_DIR}/.spin"
  fi
  cp "${RELEASE_METADATA_PATH}" "${NEXT_APP_DIR}/.shuma-release.json"

  cd "${NEXT_APP_DIR}"
  chmod 600 .env.local
  set -a
  source .env.local
  set +a
  python3 scripts/deploy/render_gateway_spin_manifest.py \
    --manifest "${NEXT_APP_DIR}/spin.toml" \
    --output "${NEXT_APP_DIR}/spin.gateway.toml" \
    --upstream-origin "${SHUMA_GATEWAY_UPSTREAM_ORIGIN}"
  GATEWAY_SURFACE_CATALOG_PATH="${GATEWAY_SURFACE_CATALOG_REMOTE_PATH}" make deploy-self-hosted-minimal

  rm -rf "${PREV_APP_DIR}"
  mv "${REMOTE_APP_DIR}" "${PREV_APP_DIR}"
  mv "${NEXT_APP_DIR}" "${REMOTE_APP_DIR}"
  sudo systemctl daemon-reload
  if ! sudo systemctl restart "${REMOTE_SERVICE_NAME}"; then
    echo "Remote service restart failed; attempting rollback." >&2
    rollback_release || true
    exit 1
  fi
}

rollback_release() {
  if [[ ! -d "${PREV_APP_DIR}" ]]; then
    echo "No previous app dir available for rollback: ${PREV_APP_DIR}" >&2
    exit 1
  fi

  sudo systemctl stop "${REMOTE_SERVICE_NAME}" || true
  rm -rf "${FAILED_APP_DIR}"
  if [[ -d "${REMOTE_APP_DIR}" ]]; then
    mv "${REMOTE_APP_DIR}" "${FAILED_APP_DIR}"
  fi
  mv "${PREV_APP_DIR}" "${REMOTE_APP_DIR}"
  sudo systemctl start "${REMOTE_SERVICE_NAME}"
}

case "${ACTION}" in
  install)
    install_release
    ;;
  rollback)
    rollback_release
    ;;
  *)
    echo "Unknown action: ${ACTION}" >&2
    exit 1
    ;;
esac
""",
        encoding="utf-8",
    )
    script_path.chmod(0o755)
    return script_path


def copy_file_to_remote(receipt: dict[str, Any], local_path: Path, remote_path: str) -> None:
    result = subprocess.run(scp_command_for_copy(receipt, local_path, remote_path), check=False)
    if result.returncode != 0:
        fail(f"Failed to upload {local_path} to {remote_path}.")


def run_remote_update_install(receipt: dict[str, Any]) -> int:
    runtime = receipt["runtime"]
    remote_command = (
        f"{shell_env_assignments({'REMOTE_APP_DIR': runtime['app_dir'], 'REMOTE_SERVICE_NAME': runtime['service_name'], 'RELEASE_ARCHIVE_PATH': REMOTE_UPDATE_ARCHIVE_PATH, 'RELEASE_METADATA_PATH': REMOTE_UPDATE_METADATA_PATH, 'GATEWAY_SURFACE_CATALOG_REMOTE_PATH': REMOTE_UPDATE_SURFACE_CATALOG_PATH})} "
        f"bash {shlex.quote(REMOTE_UPDATE_SCRIPT_PATH)} install"
    )
    return run_ssh_operation(receipt, remote_command)


def rollback_remote_update(receipt: dict[str, Any]) -> int:
    runtime = receipt["runtime"]
    remote_command = (
        f"{shell_env_assignments({'REMOTE_APP_DIR': runtime['app_dir'], 'REMOTE_SERVICE_NAME': runtime['service_name']})} "
        f"bash {shlex.quote(REMOTE_UPDATE_SCRIPT_PATH)} rollback"
    )
    return run_ssh_operation(receipt, remote_command)


def run_remote_loopback_health_check(receipt: dict[str, Any]) -> int:
    runtime = receipt["runtime"]
    shell_script = f"""
set -euo pipefail
cd {shlex.quote(runtime["app_dir"])}
set -a
source .env.local
set +a
headers=(-H "X-Forwarded-For: 127.0.0.1" -H "X-Forwarded-Proto: https")
if [[ -n "${{SHUMA_FORWARDED_IP_SECRET:-}}" ]]; then
  headers+=(-H "X-Shuma-Forwarded-Secret: ${{SHUMA_FORWARDED_IP_SECRET}}")
fi
if [[ -n "${{SHUMA_HEALTH_SECRET:-}}" ]]; then
  headers+=(-H "X-Shuma-Health-Secret: ${{SHUMA_HEALTH_SECRET}}")
fi
response="$(curl -s --max-time 8 -w $'\\n__HTTP_STATUS__:%{{http_code}}' http://127.0.0.1:3000/health "${{headers[@]}}" || true)"
body="${{response%$'\\n'__HTTP_STATUS__:*}}"
status="${{response##*$'\\n'__HTTP_STATUS__:}}"
if [[ "${{status}}" == "200" ]] && grep -q "OK" <<< "${{body}}"; then
  exit 0
fi
printf 'status=%s body=%s\\n' "${{status}}" "${{body}}" >&2
exit 1
"""
    remote_command = f"bash -c {shlex.quote(shell_script)}"
    attempts = 6
    for attempt in range(1, attempts + 1):
        result = subprocess.run(
            ssh_command_for_operation(receipt, remote_command),
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode == 0:
            return 0
        if attempt == attempts:
            stderr = (result.stderr or "").strip()
            stdout = (result.stdout or "").strip()
            message = stderr or stdout or "remote loopback health check failed"
            print(f"Remote loopback health failed: {message}", file=sys.stderr)
            return int(result.returncode)
        print(
            f"Remote loopback health attempt {attempt}/{attempts} failed; retrying in 2s...",
            file=sys.stderr,
        )
        time.sleep(2)
    return 1


def fetch_remote_env_values(receipt: dict[str, Any], keys: Sequence[str]) -> dict[str, str]:
    if not keys:
        return {}
    env_path = f"{receipt['runtime']['app_dir']}/.env.local"
    remote_command = f"cat {shlex.quote(env_path)}"
    result = subprocess.run(
        ssh_command_for_operation(receipt, remote_command),
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        fail(
            "Failed to read remote operator env values required for smoke:\n"
            + (result.stderr.strip() or result.stdout.strip() or "remote command failed")
        )
    parsed = parse_env_text(result.stdout or "")
    return {key: parsed.get(key, "") for key in keys}


def hydrate_missing_local_operator_env(env_file: Path, receipt: dict[str, Any], keys: Sequence[str]) -> dict[str, str]:
    env_values = read_env_file(env_file)
    missing_keys = [key for key in keys if not env_values.get(key, "").strip()]
    if not missing_keys:
        return env_values
    remote_values = fetch_remote_env_values(receipt, missing_keys)
    for key in missing_keys:
        value = remote_values.get(key, "").strip()
        if value:
            upsert_env_value(env_file, key, value)
            env_values[key] = value
    return env_values


def run_remote_smoke(env_file: Path, receipt: dict[str, Any]) -> int:
    smoke_env = os.environ.copy()
    env_values = hydrate_missing_local_operator_env(env_file, receipt, REMOTE_SMOKE_ENV_KEYS)
    smoke_env.update(env_values)
    smoke_env["SHUMA_BASE_URL"] = receipt["runtime"]["public_base_url"]
    smoke_env["SHUMA_SMOKE_SKIP_HEALTH"] = "1"
    smoke_env["GATEWAY_SURFACE_CATALOG_PATH"] = str(
        ensure_local_file(receipt["deploy"]["surface_catalog_path"], "local surface catalog")
    )
    allowlisted_ip = first_ip_from_allowlist(env_values.get("SHUMA_ADMIN_IP_ALLOWLIST", ""))
    if allowlisted_ip:
        smoke_env["SHUMA_SMOKE_FORWARDED_IP"] = allowlisted_ip
        smoke_env["SHUMA_SMOKE_ADMIN_FORWARDED_IP"] = allowlisted_ip

    attempts = 6
    for attempt in range(1, attempts + 1):
        result = subprocess.run(
            ["make", "--no-print-directory", "smoke-single-host"],
            cwd=str(REPO_ROOT),
            env=smoke_env,
            check=False,
        )
        if result.returncode == 0:
            return 0
        if attempt == attempts:
            return int(result.returncode)
        print(
            f"Remote smoke attempt {attempt}/{attempts} failed; retrying in 2s...",
            file=sys.stderr,
        )
        time.sleep(2)
    return 1


def refresh_remote_receipt_metadata(
    receipts_dir: Path,
    name: str,
    *,
    last_deployed_commit: str,
    last_deployed_at_utc: str,
) -> None:
    receipt = load_remote_receipt(receipts_dir, name)
    receipt["metadata"]["last_deployed_commit"] = last_deployed_commit
    receipt["metadata"]["last_deployed_at_utc"] = last_deployed_at_utc
    write_json(remote_receipt_path(receipts_dir, name), receipt)


def perform_remote_update(receipt: dict[str, Any], env_file: Path, receipts_dir: Path) -> int:
    ensure_local_file(receipt["deploy"]["surface_catalog_path"], "local surface catalog")
    with tempfile.TemporaryDirectory(prefix="shuma-remote-update-") as temp_dir:
        work_dir = Path(temp_dir)
        archive_path, metadata_path, metadata = build_release_bundle(repo_root=REPO_ROOT, work_dir=work_dir)
        update_script_path = write_remote_update_script(work_dir)
        copy_file_to_remote(receipt, archive_path, REMOTE_UPDATE_ARCHIVE_PATH)
        copy_file_to_remote(receipt, metadata_path, REMOTE_UPDATE_METADATA_PATH)
        copy_file_to_remote(
            receipt,
            ensure_local_file(receipt["deploy"]["surface_catalog_path"], "local surface catalog"),
            REMOTE_UPDATE_SURFACE_CATALOG_PATH,
        )
        copy_file_to_remote(receipt, update_script_path, REMOTE_UPDATE_SCRIPT_PATH)
        if run_remote_update_install(receipt) != 0:
            fail("Remote update install/restart failed; rollback was attempted on the host.")
        if run_remote_loopback_health_check(receipt) != 0:
            rollback_result = rollback_remote_update(receipt)
            if rollback_result == 0:
                fail("Remote loopback health failed after update; rollback attempted.")
            fail("Remote loopback health failed after update, and rollback also failed.")
        if run_remote_smoke(env_file, receipt) != 0:
            rollback_result = rollback_remote_update(receipt)
            if rollback_result == 0:
                fail("Remote smoke failed after update; rollback attempted.")
            fail("Remote smoke failed after update, and rollback also failed.")
        refresh_remote_receipt_metadata(
            receipts_dir,
            receipt["identity"]["name"],
            last_deployed_commit=str(metadata.get("commit") or ""),
            last_deployed_at_utc=utc_now_iso(),
        )
        print(
            f"Remote updated: {receipt['identity']['name']} -> {receipt['runtime']['public_base_url']} "
            f"(commit={metadata.get('commit', '')})"
        )
    return 0


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Manage normalized Shuma ssh_systemd remote targets.")
    parser.add_argument(
        "--env-file",
        default=str(DEFAULT_ENV_FILE),
        help="gitignored local env file containing SHUMA_ACTIVE_REMOTE",
    )
    parser.add_argument(
        "--receipts-dir",
        default=str(DEFAULT_REMOTE_RECEIPTS_DIR),
        help="Directory containing normalized remote receipts",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    use_parser = subparsers.add_parser("use", help="Select the active remote target in .env.local")
    use_parser.add_argument("--name", required=True, help="Remote target name")

    for command in ("status", "logs", "start", "stop", "open-dashboard"):
        action_parser = subparsers.add_parser(command, help=f"Run {command} against the selected remote")
        action_parser.add_argument("--name", help="Override the active remote target")
    update_parser = subparsers.add_parser(
        "update",
        help="Upload the exact committed HEAD bundle, restart the selected ssh_systemd remote, and run smoke",
    )
    update_parser.add_argument("--name", help="Override the active remote target")

    write_parser = subparsers.add_parser("write-linode-receipt", help="Write a normalized Linode remote receipt")
    write_parser.add_argument("--name", required=True, help="Remote target name")
    write_parser.add_argument("--host", required=True, help="Remote host or IPv4 address")
    write_parser.add_argument("--port", type=int, default=22)
    write_parser.add_argument("--user", default="shuma")
    write_parser.add_argument("--private-key-path", required=True)
    write_parser.add_argument("--public-base-url", required=True)
    write_parser.add_argument("--surface-catalog-path", required=True)
    write_parser.add_argument("--app-dir", default=DEFAULT_APP_DIR)
    write_parser.add_argument("--service-name", default=DEFAULT_SERVICE_NAME)
    write_parser.add_argument("--spin-manifest-path", default=DEFAULT_SPIN_MANIFEST_PATH)
    write_parser.add_argument("--smoke-path", default=DEFAULT_SMOKE_PATH)
    write_parser.add_argument("--last-deployed-commit", default="")
    write_parser.add_argument("--last-deployed-at-utc", default="")
    write_parser.add_argument("--instance-id", type=int)
    write_parser.add_argument("--label", default="")
    write_parser.add_argument("--region", default="")
    write_parser.add_argument("--linode-type", default="")
    write_parser.add_argument("--image", default="")
    return parser.parse_args(argv)


def _linode_provider_extension(args: argparse.Namespace) -> dict[str, Any]:
    values: dict[str, Any] = {}
    if args.instance_id is not None:
        values["instance_id"] = args.instance_id
    if args.label:
        values["label"] = args.label
    if args.region:
        values["region"] = args.region
    if args.linode_type:
        values["type"] = args.linode_type
    if args.image:
        values["image"] = args.image
    return {"linode": values}


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    env_file = Path(args.env_file).expanduser().resolve()
    receipts_dir = Path(args.receipts_dir).expanduser().resolve()

    if args.command == "use":
        receipt = activate_remote(env_file, receipts_dir, args.name)
        print(
            f"Active remote set: {receipt['identity']['name']} -> {receipt['runtime']['public_base_url']}"
        )
        return 0

    if args.command == "write-linode-receipt":
        last_deployed_at_utc = args.last_deployed_at_utc or utc_now_iso()
        path = write_remote_receipt(
            receipts_dir=receipts_dir,
            name=args.name,
            provider_kind="linode",
            host=args.host,
            port=args.port,
            user=args.user,
            private_key_path=args.private_key_path,
            public_base_url=args.public_base_url,
            surface_catalog_path=args.surface_catalog_path,
            app_dir=args.app_dir,
            service_name=args.service_name,
            spin_manifest_path=args.spin_manifest_path,
            smoke_path=args.smoke_path,
            last_deployed_commit=args.last_deployed_commit,
            last_deployed_at_utc=last_deployed_at_utc,
            provider_extension=_linode_provider_extension(args),
        )
        print(path)
        return 0

    receipt = select_remote(getattr(args, "name", None), env_file, receipts_dir)
    if args.command == "update":
        return perform_remote_update(receipt, env_file, receipts_dir)
    service_name = receipt["runtime"]["service_name"]
    command_map = {
        "status": f"sudo systemctl status {service_name} --no-pager",
        "logs": f"sudo journalctl -u {service_name} -n 200 --no-pager",
        "start": f"sudo systemctl start {service_name}",
        "stop": f"sudo systemctl stop {service_name}",
    }
    if args.command == "open-dashboard":
        return open_dashboard(receipt)
    return run_ssh_operation(receipt, command_map[args.command])


if __name__ == "__main__":
    raise SystemExit(main())
