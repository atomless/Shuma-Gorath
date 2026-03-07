"""Generic day-2 remote target helpers for normalized ssh_systemd receipts."""

from __future__ import annotations

import argparse
import ipaddress
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence

from scripts.deploy.local_env import ensure_env_file, read_env_value, upsert_env_value

REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_REMOTE_RECEIPTS_DIR = REPO_ROOT / ".spin" / "remotes"
REMOTE_RECEIPT_SCHEMA = "shuma.remote_target.v1"
DEFAULT_BACKEND_KIND = "ssh_systemd"
DEFAULT_APP_DIR = "/opt/shuma-gorath"
DEFAULT_SERVICE_NAME = "shuma-gorath"
DEFAULT_SPIN_MANIFEST_PATH = "/opt/shuma-gorath/spin.gateway.toml"
DEFAULT_SMOKE_PATH = "/health"


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
        receipt = load_remote_receipt(receipts_dir, args.name)
        ensure_env_file(env_file)
        upsert_env_value(env_file, "SHUMA_ACTIVE_REMOTE", receipt["identity"]["name"])
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
