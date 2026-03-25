#!/usr/bin/env python3
"""Live Spin proof for opaque maze traversal across multiple hops."""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any
from urllib.parse import parse_qs, urlparse


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.local_env import read_env_file


DEFAULT_ENV_FILE = REPO_ROOT / ".env.local"
DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "maze_live_traversal.json"
REPORT_SCHEMA_VERSION = "shuma.maze_live_traversal.v1"
DEFAULT_TIMEOUT_SECONDS = 30
DEFAULT_USER_AGENT = "ShumaMazeTraversal/1.0"
DEFAULT_LOCAL_ADMIN_IP = "127.0.0.1"

MAZE_PROFILE = {
    "shadow_mode": False,
    "maze_enabled": True,
    "maze_auto_ban": False,
    "maze_client_expansion_enabled": True,
    "maze_checkpoint_every_nodes": 1,
    "maze_checkpoint_every_ms": 60000,
    "maze_step_ahead_max": 1,
    "maze_no_js_fallback_max_depth": 1,
    "maze_micro_pow_enabled": False,
    "maze_server_visible_links": 1,
    "maze_max_links": 4,
    "pow_enabled": False,
}


class MazeTraversalFailure(RuntimeError):
    """Raised when the live maze traversal contract is violated."""


def extract_first_maze_link(html: str) -> str:
    for fragment in html.split("<a "):
        if 'data-link-kind="maze"' not in fragment:
            continue
        href_match = re.search(r'href="([^"]+)"', fragment, flags=re.IGNORECASE | re.DOTALL)
        if href_match:
            return href_match.group(1)
    raise MazeTraversalFailure("Maze page did not include a tokenized maze link.")


def extract_bootstrap_json(html: str) -> dict[str, Any]:
    match = re.search(
        r'<script id="maze-bootstrap" type="application/json">\s*(\{.*?\})\s*</script>',
        html,
        flags=re.DOTALL,
    )
    if not match:
        raise MazeTraversalFailure("Maze page did not include bootstrap JSON.")
    try:
        payload = json.loads(match.group(1))
    except json.JSONDecodeError as exc:
        raise MazeTraversalFailure(f"Maze bootstrap JSON was invalid: {exc}") from exc
    if not isinstance(payload, dict):
        raise MazeTraversalFailure("Maze bootstrap payload was not an object.")
    return payload


def extract_preview_entry_path(html: str) -> str:
    href_match = re.search(r'<a[^>]+href="([^"]+)"', html, flags=re.IGNORECASE | re.DOTALL)
    if not href_match:
        raise MazeTraversalFailure("Maze preview did not include any preview anchor.")
    parsed = urlparse(href_match.group(1))
    preview_path = parse_qs(parsed.query).get("path", [""])[0]
    if not preview_path.startswith("/"):
        raise MazeTraversalFailure("Maze preview link did not expose a canonical opaque path.")
    return preview_path


def build_issue_links_payload(
    bootstrap: dict[str, Any],
    *,
    requested_hidden_count: int,
) -> dict[str, Any]:
    expansion = bootstrap.get("client_expansion")
    if not isinstance(expansion, dict):
        raise MazeTraversalFailure("Maze bootstrap payload did not include client expansion.")
    return {
        "parent_token": str(bootstrap.get("checkpoint_token") or ""),
        "flow_id": str(bootstrap.get("flow_id") or ""),
        "entropy_nonce": str(bootstrap.get("entropy_nonce") or ""),
        "path_prefix": str(bootstrap.get("path_prefix") or ""),
        "seed": int(expansion.get("seed") or 0),
        "seed_sig": str(expansion.get("seed_sig") or ""),
        "hidden_count": int(expansion.get("hidden_count") or 0),
        "requested_hidden_count": int(requested_hidden_count),
        "segment_len": int(expansion.get("segment_len") or 0),
        "candidates": [],
    }


def find_recent_fallback_event(
    events: list[dict[str, Any]],
    *,
    event_type: str,
    reason_label: str,
    action_label: str,
) -> dict[str, Any]:
    for event in events:
        if str(event.get("event") or "").strip().lower() != event_type.strip().lower():
            continue
        if str(event.get("reason") or "").strip().lower() != "maze_runtime_fallback":
            continue
        outcome = str(event.get("outcome") or "").strip().lower()
        if reason_label.strip().lower() not in outcome:
            continue
        if f"action={action_label.strip().lower()}" not in outcome:
            continue
        return event
    raise MazeTraversalFailure(
        f"Recent events did not include maze fallback event={event_type} "
        f"reason={reason_label} action={action_label}."
    )


def _require_non_empty(value: Any, label: str) -> str:
    rendered = str(value or "").strip()
    if not rendered:
        raise MazeTraversalFailure(f"Missing required {label}.")
    return rendered


def fresh_test_ip() -> str:
    octet = random.SystemRandom().randrange(20, 240)
    host = random.SystemRandom().randrange(10, 240)
    return f"198.51.{octet}.{host}"


class MazeLiveTraversalGate:
    def __init__(
        self,
        *,
        base_url: str,
        api_key: str,
        forwarded_secret: str,
        health_secret: str,
        timeout_seconds: int,
        report_path: Path,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key.strip()
        self.forwarded_secret = forwarded_secret.strip()
        self.health_secret = health_secret.strip()
        self.timeout_seconds = timeout_seconds
        self.report_path = report_path
        self.user_agent = DEFAULT_USER_AGENT
        self.test_ip = fresh_test_ip()
        self.local_admin_ip = DEFAULT_LOCAL_ADMIN_IP
        self.opener = urllib.request.build_opener()

        if not self.api_key:
            raise MazeTraversalFailure("SHUMA_API_KEY must be set for live maze traversal proof.")

    def _headers(
        self,
        *,
        include_json: bool = False,
        include_auth: bool = False,
        include_forwarding: bool = True,
        include_health_secret: bool = False,
        forwarded_ip: str | None = None,
    ) -> dict[str, str]:
        headers = {
            "Accept": "application/json" if include_json else "text/html,application/json",
            "User-Agent": self.user_agent,
            "Cache-Control": "no-store",
            "Pragma": "no-cache",
            "Origin": self.base_url,
            "X-Forwarded-Proto": "https",
        }
        if include_auth:
            headers["Authorization"] = f"Bearer {self.api_key}"
        if include_json:
            headers["Content-Type"] = "application/json"
        if include_forwarding:
            headers["X-Forwarded-For"] = forwarded_ip or self.test_ip
            if self.forwarded_secret:
                headers["X-Shuma-Forwarded-Secret"] = self.forwarded_secret
        if include_health_secret and self.health_secret:
            headers["X-Shuma-Health-Secret"] = self.health_secret
        return headers

    def _request(
        self,
        method: str,
        path: str,
        *,
        payload: dict[str, Any] | None = None,
        include_auth: bool = False,
        include_json: bool = False,
        include_health_secret: bool = False,
        forwarded_ip: str | None = None,
    ) -> dict[str, Any]:
        url = path if path.startswith("http://") or path.startswith("https://") else f"{self.base_url}{path}"
        headers = self._headers(
            include_json=include_json or payload is not None,
            include_auth=include_auth,
            include_health_secret=include_health_secret,
            forwarded_ip=forwarded_ip,
        )
        data = None
        if payload is not None:
            data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(url, data=data, method=method.upper(), headers=headers)
        try:
            with self.opener.open(req, timeout=10) as response:
                raw = response.read().decode("utf-8", errors="replace")
                status = int(getattr(response, "status", 0) or 0)
        except urllib.error.HTTPError as err:
            raw = err.read().decode("utf-8", errors="replace")
            status = int(err.code)
        parsed: Any
        try:
            parsed = json.loads(raw) if raw else {}
        except json.JSONDecodeError:
            parsed = None
        return {"status": status, "raw": raw, "json": parsed}

    def ensure_health(self) -> None:
        health = self._request(
            "GET",
            "/health",
            include_health_secret=True,
            forwarded_ip=self.local_admin_ip,
        )
        if health["status"] != 200:
            raise MazeTraversalFailure(
                f"Health check failed before maze traversal proof: status={health['status']} body={health['raw'][:240]}"
            )

    def fetch_runtime_config(self) -> dict[str, Any]:
        response = self._request(
            "GET",
            "/admin/config",
            include_auth=True,
            include_json=True,
            forwarded_ip=self.local_admin_ip,
        )
        if response["status"] != 200 or not isinstance(response["json"], dict):
            raise MazeTraversalFailure(
                f"Could not read runtime config: status={response['status']} body={response['raw'][:240]}"
            )
        payload = response["json"]
        config = payload.get("config") if isinstance(payload, dict) else None
        if isinstance(config, dict):
            return config
        return payload

    def apply_runtime_config(self, payload: dict[str, Any]) -> None:
        response = self._request(
            "POST",
            "/admin/config",
            payload=payload,
            include_auth=True,
            include_json=True,
            forwarded_ip=self.local_admin_ip,
        )
        if response["status"] != 200:
            raise MazeTraversalFailure(
                f"Could not apply runtime config: status={response['status']} body={response['raw'][:240]}"
            )

    def fetch_recent_events(self) -> list[dict[str, Any]]:
        response = self._request(
            "GET",
            "/admin/monitoring?hours=1&limit=100",
            include_auth=True,
            include_json=True,
            forwarded_ip=self.local_admin_ip,
        )
        if response["status"] != 200 or not isinstance(response["json"], dict):
            raise MazeTraversalFailure(
                f"Could not read recent monitoring events: status={response['status']} body={response['raw'][:240]}"
            )
        details = response["json"].get("details")
        if not isinstance(details, dict):
            return []
        events = details.get("events")
        if not isinstance(events, dict):
            return []
        rows = events.get("recent_events")
        if not isinstance(rows, list):
            return []
        return [row for row in rows if isinstance(row, dict)]

    def wait_for_recent_fallback_event(
        self,
        *,
        event_type: str,
        reason_label: str,
        action_label: str,
        min_ts: int,
    ) -> dict[str, Any]:
        deadline = time.time() + float(self.timeout_seconds)
        last_events: list[dict[str, Any]] = []
        while time.time() < deadline:
            last_events = [
                event
                for event in self.fetch_recent_events()
                if int(event.get("ts") or 0) >= int(min_ts)
            ]
            try:
                return find_recent_fallback_event(
                    last_events,
                    event_type=event_type,
                    reason_label=reason_label,
                    action_label=action_label,
                )
            except MazeTraversalFailure:
                time.sleep(1)
        raise MazeTraversalFailure(
            f"Timed out waiting for maze fallback event={event_type} "
            f"reason={reason_label} action={action_label}. Last events={last_events!r}"
        )

    def run(self) -> dict[str, Any]:
        self.ensure_health()
        baseline_config = self.fetch_runtime_config()
        baseline_restore_payload = {
            key: baseline_config[key]
            for key in MAZE_PROFILE
            if key in baseline_config
        }
        started_ts = int(time.time())
        report: dict[str, Any] = {
            "schema_version": REPORT_SCHEMA_VERSION,
            "base_url": self.base_url,
            "test_ip": self.test_ip,
            "entry_path": None,
            "checkpoint_missing_event": None,
            "token_replay_event": None,
            "replay_attempts": [],
            "checkpoint_status": None,
            "issue_links_count": 0,
            "progressed_hidden_link": None,
        }

        try:
            self.apply_runtime_config(MAZE_PROFILE)

            # The public opaque path is derived server-side; use the canonical preview seed via /admin/maze/preview.
            preview = self._request(
                "GET",
                "/admin/maze/preview",
                include_auth=True,
                forwarded_ip=self.local_admin_ip,
            )
            preview_html = preview["raw"]
            if preview["status"] != 200 or "data-link-kind=\"maze\"" not in preview_html:
                raise MazeTraversalFailure(
                    f"Maze preview did not return an opaque maze page: status={preview['status']}"
                )
            entry_path = extract_preview_entry_path(preview_html)
            report["entry_path"] = entry_path

            entry = self._request("GET", entry_path)
            if entry["status"] != 200:
                raise MazeTraversalFailure(
                    f"Opaque maze entry failed: status={entry['status']} body={entry['raw'][:240]}"
                )
            first_link = extract_first_maze_link(entry["raw"])

            child = self._request("GET", first_link)
            if child["status"] != 200:
                raise MazeTraversalFailure(
                    f"First opaque follow failed: status={child['status']} body={child['raw'][:240]}"
                )
            child_bootstrap = extract_bootstrap_json(child["raw"])
            child_checkpoint_token = _require_non_empty(
                child_bootstrap.get("checkpoint_token"),
                "child checkpoint token",
            )
            child_visible_link = extract_first_maze_link(child["raw"])

            checkpoint_missing = self._request("GET", child_visible_link)
            if checkpoint_missing["status"] != 200 or "document.cookie = 'js_verified=" not in checkpoint_missing["raw"]:
                raise MazeTraversalFailure(
                    "Maze checkpoint-missing fallback did not return the expected JS challenge interstitial."
                )
            report["checkpoint_missing_event"] = self.wait_for_recent_fallback_event(
                event_type="Challenge",
                reason_label="maze_checkpoint_missing",
                action_label="challenge",
                min_ts=started_ts,
            )

            checkpoint_response = self._request(
                "POST",
                _require_non_empty(child_bootstrap.get("path_prefix"), "path prefix").rstrip("/") + "/checkpoint",
                payload={
                    "token": child_checkpoint_token,
                    "flow_id": child_bootstrap.get("flow_id"),
                    "depth": child_bootstrap.get("depth"),
                    "checkpoint_reason": "live_integration",
                },
                include_json=True,
            )
            if checkpoint_response["status"] != 204:
                raise MazeTraversalFailure(
                    f"Maze checkpoint submit was not accepted: status={checkpoint_response['status']} body={checkpoint_response['raw'][:240]}"
                )
            report["checkpoint_status"] = checkpoint_response["status"]

            issue_links_path = _require_non_empty(
                child_bootstrap.get("client_expansion", {}).get("issue_path")
                if isinstance(child_bootstrap.get("client_expansion"), dict)
                else None,
                "issue-links path",
            )
            issue_payload = build_issue_links_payload(child_bootstrap, requested_hidden_count=2)
            issue_response = self._request(
                "POST",
                issue_links_path,
                payload=issue_payload,
                include_json=True,
            )
            if issue_response["status"] != 200 or not isinstance(issue_response["json"], dict):
                raise MazeTraversalFailure(
                    f"Maze issue-links request failed: status={issue_response['status']} body={issue_response['raw'][:240]}"
                )
            issued_links = issue_response["json"].get("links")
            if not isinstance(issued_links, list) or not issued_links:
                raise MazeTraversalFailure("Maze issue-links did not return hidden links.")
            report["issue_links_count"] = len(issued_links)

            hidden_link = _require_non_empty(issued_links[0].get("href"), "issued hidden href")
            progressed = self._request("GET", hidden_link)
            if progressed["status"] != 200 or "data-link-kind=\"maze\"" not in progressed["raw"]:
                raise MazeTraversalFailure(
                    f"Checkpointed issue-link traversal did not continue through maze: status={progressed['status']} body={progressed['raw'][:240]}"
                )
            report["progressed_hidden_link"] = hidden_link

            replay_block_observed = False
            for _ in range(2):
                replay = self._request("GET", first_link)
                report["replay_attempts"].append({"status": replay["status"], "body": replay["raw"][:160]})
                if replay["status"] == 403 and "Access Blocked" in replay["raw"]:
                    replay_block_observed = True
                    break
                if replay["status"] == 200 and "document.cookie = 'js_verified=" in replay["raw"]:
                    continue
                raise MazeTraversalFailure(
                    f"Maze token replay returned an unexpected fallback shape: status={replay['status']} body={replay['raw'][:240]}"
                )
            if not replay_block_observed:
                raise MazeTraversalFailure(
                    f"Maze token replay did not escalate to deterministic block fallback: attempts={report['replay_attempts']!r}"
                )
            report["token_replay_event"] = self.wait_for_recent_fallback_event(
                event_type="Block",
                reason_label="maze_token_replay",
                action_label="block",
                min_ts=started_ts,
            )
        finally:
            try:
                self.apply_runtime_config(baseline_restore_payload)
            except MazeTraversalFailure as exc:
                raise MazeTraversalFailure(
                    f"Maze live traversal proof could not restore the original config: {exc}"
                ) from exc

        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return report


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prove live opaque maze traversal, checkpoint, hidden issuance, and deterministic fallback behavior."
    )
    parser.add_argument("--base-url", default=os.environ.get("SHUMA_BASE_URL", "http://127.0.0.1:3000"))
    parser.add_argument("--env-file", default=str(DEFAULT_ENV_FILE))
    parser.add_argument("--report-path", default=str(DEFAULT_REPORT_PATH))
    parser.add_argument("--timeout-seconds", type=int, default=DEFAULT_TIMEOUT_SECONDS)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    env_values = read_env_file(Path(args.env_file))
    api_key = os.environ.get("SHUMA_API_KEY", env_values.get("SHUMA_API_KEY", ""))
    forwarded_secret = os.environ.get(
        "SHUMA_FORWARDED_IP_SECRET",
        env_values.get("SHUMA_FORWARDED_IP_SECRET", ""),
    )
    health_secret = os.environ.get(
        "SHUMA_HEALTH_SECRET",
        env_values.get("SHUMA_HEALTH_SECRET", ""),
    )
    gate = MazeLiveTraversalGate(
        base_url=args.base_url,
        api_key=api_key,
        forwarded_secret=forwarded_secret,
        health_secret=health_secret,
        timeout_seconds=args.timeout_seconds,
        report_path=Path(args.report_path),
    )
    try:
        report = gate.run()
    except MazeTraversalFailure as exc:
        print(f"maze live traversal gate failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
