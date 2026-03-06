"""Agent-oriented Linode shared-host setup helper."""

from __future__ import annotations

import argparse
import base64
import getpass
import ipaddress
import json
import os
import secrets
import subprocess
import sys
import time
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Sequence

REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.site_surface_catalog import SUPPORTED_MODES, build_payload

DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_RECEIPT_PATH = REPO_ROOT / ".spin" / "linode-shared-host-setup.json"
DEFAULT_LINODE_API_URL = "https://api.linode.com/v4"
DEFAULT_PUBLIC_IP_URL = "https://api.ipify.org?format=json"
DEFAULT_IMAGE = "linode/ubuntu24.04"
DEFAULT_PROFILE_TYPES = {
    "small": "g6-nanode-1",
    "medium": "g6-standard-1",
    "large": "g6-standard-2",
}


def strip_wrapping_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {'"', "'"}:
        return value[1:-1]
    return value


def ensure_env_file(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        path.write_text("", encoding="utf-8")
    path.chmod(0o600)


def read_env_value(path: Path, key: str) -> str:
    if not path.exists():
        return ""
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if raw_line.startswith(f"{key}="):
            return strip_wrapping_quotes(raw_line.split("=", 1)[1])
    return ""


def upsert_env_value(path: Path, key: str, value: str) -> None:
    ensure_env_file(path)
    new_line = f"{key}={value}"
    lines = path.read_text(encoding="utf-8").splitlines() if path.exists() else []
    replaced = False
    updated: list[str] = []
    for line in lines:
        if line.startswith(f"{key}="):
            updated.append(new_line)
            replaced = True
        else:
            updated.append(line)
    if not replaced:
        updated.append(new_line)
    path.write_text("\n".join(updated).rstrip() + "\n", encoding="utf-8")
    path.chmod(0o600)


def is_interactive_session() -> bool:
    return sys.stdin.isatty() and sys.stdout.isatty()


def prompt_secret(prompt: str) -> str:
    return getpass.getpass(prompt)


def prompt_confirm(prompt: str, *, default_yes: bool = True) -> bool:
    suffix = " [Y/n]: " if default_yes else " [y/N]: "
    answer = input(f"{prompt}{suffix}").strip().lower()
    if not answer:
        return default_yes
    return answer in {"y", "yes"}


def prompt_text(prompt: str) -> str:
    return input(prompt)


def parse_ip_response(body: str) -> str:
    candidate = body.strip()
    if candidate.startswith("{"):
        payload = json.loads(candidate)
        candidate = str(payload.get("ip", "")).strip()
    ipaddress.ip_address(candidate)
    return candidate


def detect_public_ip(url: str = DEFAULT_PUBLIC_IP_URL, timeout_seconds: int = 10) -> str:
    request = urllib.request.Request(url, method="GET")
    with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
        body = response.read().decode("utf-8")
    return parse_ip_response(body)


def default_catalog_output(docroot: Path) -> Path:
    return REPO_ROOT / ".spin" / f"{docroot.name}.surface-catalog.json"


def ensure_ssh_keypair(private_key_path: Path, comment: str = "shuma-linode") -> tuple[Path, Path, str]:
    private_key_path = private_key_path.expanduser().resolve()
    public_key_path = Path(f"{private_key_path}.pub")
    private_key_path.parent.mkdir(parents=True, exist_ok=True)

    if not private_key_path.exists():
        subprocess.run(
            [
                "ssh-keygen",
                "-t",
                "ed25519",
                "-f",
                str(private_key_path),
                "-C",
                comment,
                "-N",
                "",
            ],
            check=True,
            capture_output=True,
            text=True,
        )

    if not public_key_path.exists():
        derived = subprocess.run(
            ["ssh-keygen", "-y", "-f", str(private_key_path)],
            check=True,
            capture_output=True,
            text=True,
        )
        public_key_path.write_text(derived.stdout.strip() + "\n", encoding="utf-8")

    private_key_path.chmod(0o600)
    public_key_path.chmod(0o644)
    public_key = public_key_path.read_text(encoding="utf-8").strip()
    if not public_key:
        raise SystemExit(f"SSH public key file is empty: {public_key_path}")
    return private_key_path, public_key_path, public_key


def build_cloud_init(ssh_public_key: str) -> str:
    return "\n".join(
        [
            "#cloud-config",
            "users:",
            "  - name: shuma",
            "    groups: sudo",
            "    shell: /bin/bash",
            "    sudo: ALL=(ALL) NOPASSWD:ALL",
            "    ssh_authorized_keys:",
            f"      - {ssh_public_key}",
            "disable_root: true",
            "ssh_pwauth: false",
            "package_update: true",
            "",
        ]
    )


class LinodeApiClient:
    def __init__(self, token: str, base_url: str = DEFAULT_LINODE_API_URL) -> None:
        self.token = token
        self.base_url = base_url.rstrip("/")

    def request_json(self, method: str, path: str, payload: dict[str, Any] | None = None) -> dict[str, Any]:
        data = None
        if payload is not None:
            data = json.dumps(payload).encode("utf-8")
        request = urllib.request.Request(
            f"{self.base_url}{path}",
            data=data,
            method=method,
            headers={
                "Authorization": f"Bearer {self.token}",
                "Content-Type": "application/json",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.loads(response.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace")
            try:
                payload = json.loads(body)
                reasons = [entry.get("reason", "") for entry in payload.get("errors", [])]
                reason = "; ".join(part for part in reasons if part) or body
            except json.JSONDecodeError:
                reason = body
            raise SystemExit(f"Linode API {method} {path} failed (HTTP {exc.code}): {reason}") from exc
        except urllib.error.URLError as exc:
            raise SystemExit(f"Linode API {method} {path} failed: {exc.reason}") from exc

    def validate_token(self) -> dict[str, Any]:
        return self.request_json("GET", "/profile")

    def create_instance(
        self,
        *,
        label: str,
        region: str,
        linode_type: str,
        image: str,
        ssh_public_key: str,
    ) -> dict[str, Any]:
        cloud_init = build_cloud_init(ssh_public_key)
        payload = {
            "region": region,
            "type": linode_type,
            "image": image,
            "label": label,
            "root_pass": secrets.token_hex(24),
            "booted": True,
            "metadata": {
                "user_data": base64.b64encode(cloud_init.encode("utf-8")).decode("utf-8")
            },
        }
        return self.request_json("POST", "/linode/instances", payload)

    def get_instance(self, instance_id: int) -> dict[str, Any]:
        return self.request_json("GET", f"/linode/instances/{instance_id}")

    def wait_for_instance_running(
        self, instance_id: int, *, attempts: int = 90, poll_interval_seconds: int = 2
    ) -> dict[str, Any]:
        latest: dict[str, Any] = {}
        for _ in range(attempts):
            latest = self.get_instance(instance_id)
            ipv4 = latest.get("ipv4") or []
            if latest.get("status") == "running" and ipv4:
                return latest
            time.sleep(poll_interval_seconds)
        raise SystemExit(
            f"Linode instance {instance_id} did not reach running state with an IPv4 address in time."
        )


def summarize_instance(details: dict[str, Any]) -> dict[str, Any]:
    ipv4 = details.get("ipv4") or []
    return {
        "instance_id": details.get("id"),
        "label": details.get("label"),
        "status": details.get("status"),
        "public_ipv4": ipv4[0] if ipv4 else "",
        "region": details.get("region"),
        "type": details.get("type"),
        "image": details.get("image"),
    }


def parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prepare a Linode shared host for Shuma deployment and write an agent receipt."
    )
    parser.add_argument("--docroot", required=True, help="Path to the local site docroot")
    parser.add_argument(
        "--site-mode",
        default="auto",
        choices=sorted(SUPPORTED_MODES),
        help="Public-surface catalog mode (default: auto)",
    )
    parser.add_argument(
        "--catalog-output",
        help="Where to write the generated site-surface catalog JSON",
    )
    parser.add_argument(
        "--receipt-output",
        default=str(DEFAULT_RECEIPT_PATH),
        help="Where to write the setup receipt JSON",
    )
    parser.add_argument(
        "--env-file",
        default=str(DEFAULT_ENV_FILE),
        help="gitignored local env file used to persist token and admin allowlist",
    )
    parser.add_argument("--linode-token", help="Linode Personal Access Token override")
    parser.add_argument(
        "--no-store-token",
        action="store_true",
        help="Do not persist LINODE_TOKEN to the env file",
    )
    parser.add_argument(
        "--admin-ip",
        help="Explicit SHUMA_ADMIN_IP_ALLOWLIST value. Defaults to a detected public IP /32 after confirmation.",
    )
    parser.add_argument(
        "--yes",
        action="store_true",
        help="Accept detected defaults without interactive confirmation where safe",
    )
    parser.add_argument(
        "--ssh-private-key-file",
        default=str(Path("~/.ssh/shuma-linode").expanduser()),
        help="Dedicated deploy SSH private key path (default: ~/.ssh/shuma-linode)",
    )
    parser.add_argument(
        "--existing-instance-id",
        type=int,
        help="Reuse an existing prepared Linode instance instead of creating a fresh one",
    )
    parser.add_argument("--label", help="Linode label for a fresh instance")
    parser.add_argument("--profile", choices=sorted(DEFAULT_PROFILE_TYPES), default="small")
    parser.add_argument("--region", default="us-east")
    parser.add_argument("--type", dest="linode_type", help="Linode plan override")
    parser.add_argument("--image", default=DEFAULT_IMAGE)
    return parser.parse_args(argv)


def resolve_token(args: argparse.Namespace, env_file: Path) -> tuple[str, str]:
    if args.linode_token:
        return args.linode_token.strip(), "argument"
    if "LINODE_TOKEN" in os.environ and os.environ["LINODE_TOKEN"].strip():
        return os.environ["LINODE_TOKEN"].strip(), "environment"
    env_value = read_env_value(env_file, "LINODE_TOKEN").strip()
    if env_value:
        return env_value, "env_file"
    if is_interactive_session():
        token = prompt_secret("Linode Personal Access Token (hidden): ").strip()
        if token:
            return token, "prompt"
    raise SystemExit(
        "LINODE_TOKEN is missing. Add it to .env.local, export it in your shell, or rerun interactively to paste it."
    )


def resolve_admin_allowlist(args: argparse.Namespace, env_file: Path) -> str:
    for candidate in (
        args.admin_ip,
        os.environ.get("SHUMA_ADMIN_IP_ALLOWLIST", "").strip(),
        read_env_value(env_file, "SHUMA_ADMIN_IP_ALLOWLIST").strip(),
    ):
        if candidate:
            return candidate

    detected = f"{detect_public_ip()}/32"
    if args.yes:
        return detected
    if not is_interactive_session():
        raise SystemExit(
            "SHUMA_ADMIN_IP_ALLOWLIST is missing. Pass --admin-ip or rerun interactively to confirm the detected public IP."
        )
    if prompt_confirm(f"Use {detected} for SHUMA_ADMIN_IP_ALLOWLIST?", default_yes=True):
        return detected
    entered = prompt_text("Enter SHUMA_ADMIN_IP_ALLOWLIST value: ").strip()
    if not entered:
        raise SystemExit("SHUMA_ADMIN_IP_ALLOWLIST cannot be blank.")
    return entered


def resolve_catalog_output(docroot: Path, requested_output: str | None) -> Path:
    if requested_output:
        return Path(requested_output).expanduser().resolve()
    return default_catalog_output(docroot).resolve()


def resolve_label(requested_label: str | None) -> str:
    if requested_label:
        return requested_label
    return f"shuma-{datetime.now(timezone.utc).strftime('%Y%m%d%H%M%S')}"


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(argv)
    env_file = Path(args.env_file).expanduser().resolve()
    ensure_env_file(env_file)

    docroot = Path(args.docroot).expanduser().resolve()
    if not docroot.is_dir():
        raise SystemExit(f"Docroot does not exist or is not a directory: {docroot}")

    token, token_source = resolve_token(args, env_file)
    if not args.no_store_token and token_source != "env_file":
        upsert_env_value(env_file, "LINODE_TOKEN", token)

    admin_allowlist = resolve_admin_allowlist(args, env_file)
    upsert_env_value(env_file, "SHUMA_ADMIN_IP_ALLOWLIST", admin_allowlist)

    catalog_output = resolve_catalog_output(docroot, args.catalog_output)
    payload = build_payload(docroot, args.site_mode)
    write_json(catalog_output, payload)
    upsert_env_value(env_file, "GATEWAY_SURFACE_CATALOG_PATH", str(catalog_output))

    private_key_path, public_key_path, ssh_public_key = ensure_ssh_keypair(
        Path(args.ssh_private_key_file)
    )

    client = LinodeApiClient(token)
    profile = client.validate_token()

    if args.existing_instance_id:
        mode = "existing-instance"
        instance_details = client.get_instance(args.existing_instance_id)
    else:
        mode = "fresh-instance"
        linode_type = args.linode_type or DEFAULT_PROFILE_TYPES[args.profile]
        created = client.create_instance(
            label=resolve_label(args.label),
            region=args.region,
            linode_type=linode_type,
            image=args.image,
            ssh_public_key=ssh_public_key,
        )
        instance_id = int(created["id"])
        instance_details = client.wait_for_instance_running(instance_id)

    receipt = {
        "schema": "shuma.linode.shared_host_setup.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "mode": mode,
        "docroot": str(docroot),
        "site_mode": args.site_mode if args.site_mode != "auto" else payload["mode"],
        "catalog_path": str(catalog_output),
        "admin_allowlist": admin_allowlist,
        "token_source": token_source,
        "linode_profile_username": profile.get("username", ""),
        "ssh": {
            "private_key_path": str(private_key_path),
            "public_key_path": str(public_key_path),
        },
        "linode": summarize_instance(instance_details),
    }

    receipt_path = Path(args.receipt_output).expanduser().resolve()
    write_json(receipt_path, receipt)

    print(f"Receipt written: {receipt_path}")
    print(f"Linode instance id: {receipt['linode']['instance_id']}")
    print(f"Linode public IPv4: {receipt['linode']['public_ipv4']}")
    print(f"Catalog path: {catalog_output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
