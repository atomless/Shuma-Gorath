#!/usr/bin/env python3
"""Live Chromium proof for real maze browser/session behavior."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.deploy.local_env import read_env_file
from scripts.tests.maze_live_traversal import (
    DEFAULT_ENV_FILE,
    DEFAULT_TIMEOUT_SECONDS,
    MAZE_PROFILE,
    MazeLiveTraversalGate,
    MazeTraversalFailure,
    extract_bootstrap_json,
    extract_preview_entry_path,
    fresh_test_ip,
)
from scripts.tests.playwright_runtime import build_playwright_env, ensure_playwright_chromium


DEFAULT_REPORT_PATH = REPO_ROOT / ".spin" / "maze_live_browser.json"
REPORT_SCHEMA_VERSION = "shuma.maze_live_browser.v1"
BROWSER_DRIVER_SCRIPT = REPO_ROOT / "scripts" / "tests" / "adversarial_browser_driver.mjs"
DEFAULT_BROWSER_TIMEOUT_MS = 20_000
DEFAULT_BROWSER_SETTLE_MS = 250
DEFAULT_REPLAY_ATTEMPTS = 2

MAZE_BROWSER_JS_PROFILE = {
    **MAZE_PROFILE,
    "maze_client_expansion_enabled": True,
    "maze_checkpoint_every_nodes": 1,
    "maze_no_js_fallback_max_depth": 1,
    "maze_micro_pow_enabled": False,
    "maze_server_visible_links": 1,
    "pow_enabled": False,
}

MAZE_BROWSER_POW_PROFILE = {
    **MAZE_BROWSER_JS_PROFILE,
    "maze_micro_pow_enabled": True,
    "maze_micro_pow_depth_start": 1,
    "maze_micro_pow_base_difficulty": 1,
}


def build_opaque_entry_path(path_prefix: str, label: str, attempt: int) -> str:
    normalized_prefix = str(path_prefix or "").strip()
    if not normalized_prefix.startswith("/_/") or not normalized_prefix.endswith("/"):
        raise MazeTraversalFailure(f"Invalid opaque maze path prefix: {normalized_prefix!r}")
    safe_label = re.sub(r"[^a-z0-9-]+", "-", str(label or "").strip().lower()).strip("-")
    if not safe_label:
        safe_label = "maze-browser"
    return f"{normalized_prefix}{safe_label}-{max(1, int(attempt))}"


def browser_request_path_seen(browser_evidence: dict[str, Any], target_path: str) -> bool:
    lineage = browser_evidence.get("request_lineage")
    if not isinstance(lineage, list):
        return False
    for row in lineage:
        if isinstance(row, dict) and str(row.get("path") or "") == target_path:
            return True
    return False


class MazeLiveBrowserGate(MazeLiveTraversalGate):
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
        super().__init__(
            base_url=base_url,
            api_key=api_key,
            forwarded_secret=forwarded_secret,
            health_secret=health_secret,
            timeout_seconds=timeout_seconds,
            report_path=report_path,
        )
        if not BROWSER_DRIVER_SCRIPT.exists():
            raise MazeTraversalFailure(f"Missing browser driver script: {BROWSER_DRIVER_SCRIPT}")
        playwright_status = ensure_playwright_chromium()
        self.browser_driver_env = build_playwright_env(
            base_env=os.environ,
            browser_cache=Path(playwright_status.browser_cache),
        )
        self.browser_driver_command = [
            "corepack",
            "pnpm",
            "exec",
            "node",
            str(BROWSER_DRIVER_SCRIPT),
        ]

    def discover_public_entry(self, *, forwarded_ip: str) -> dict[str, str]:
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
        entry = self._request("GET", entry_path, forwarded_ip=forwarded_ip)
        if entry["status"] != 200:
            raise MazeTraversalFailure(
                f"Opaque maze entry failed: status={entry['status']} body={entry['raw'][:240]}"
            )
        bootstrap = extract_bootstrap_json(entry["raw"])
        path_prefix = str(bootstrap.get("path_prefix") or "").strip()
        if not path_prefix:
            raise MazeTraversalFailure("Maze entry bootstrap did not expose a path prefix.")
        return {"entry_path": entry_path, "path_prefix": path_prefix}

    def run_browser_action(
        self,
        *,
        action: str,
        forwarded_ip: str,
        user_agent: str,
        **extra_payload: Any,
    ) -> dict[str, Any]:
        payload = {
            "action": action,
            "base_url": self.base_url,
            "headers": {
                "X-Forwarded-For": forwarded_ip,
            },
            "user_agent": user_agent,
            "timeout_ms": DEFAULT_BROWSER_TIMEOUT_MS,
            "settle_ms": DEFAULT_BROWSER_SETTLE_MS,
            "storage_mode": "stateful_cookie_jar",
            "trusted_forwarded_secret": self.forwarded_secret,
        }
        payload.update(extra_payload)
        proc = subprocess.run(
            self.browser_driver_command,
            input=json.dumps(payload, separators=(",", ":")),
            text=True,
            capture_output=True,
            timeout=max(float(self.timeout_seconds) + 5.0, 35.0),
            check=False,
            env=self.browser_driver_env,
        )
        raw_stdout = str(proc.stdout or "").strip()
        raw_stderr = str(proc.stderr or "").strip()
        try:
            parsed = json.loads(raw_stdout) if raw_stdout else {}
        except json.JSONDecodeError as exc:
            raise MazeTraversalFailure(
                f"Browser driver returned invalid JSON: {exc}; stderr={raw_stderr[:240]}"
            ) from exc
        if proc.returncode != 0 or not isinstance(parsed, dict) or not bool(parsed.get("ok")):
            detail = str(parsed.get("detail") or raw_stderr or raw_stdout or "browser_driver_failed")
            raise MazeTraversalFailure(
                f"Browser driver action={action} failed: {detail[:320]}"
            )
        return parsed

    def run_js_enabled_flow(self) -> dict[str, Any]:
        forwarded_ip = fresh_test_ip()
        public_entry = self.discover_public_entry(forwarded_ip=forwarded_ip)
        started_ts = int(time.time())
        result = self.run_browser_action(
            action="maze_live_js_flow",
            forwarded_ip=forwarded_ip,
            user_agent="ShumaMazeBrowserJS/1.0",
            maze_entry_path=public_entry["entry_path"],
            maze_hidden_link_min=1,
            maze_replay_attempts=DEFAULT_REPLAY_ATTEMPTS,
            javascript_enabled=True,
        )
        evidence = result.get("browser_evidence")
        if not isinstance(evidence, dict):
            raise MazeTraversalFailure("Browser driver did not return browser evidence.")
        if not bool(evidence.get("maze_checkpoint_path_seen")):
            raise MazeTraversalFailure("JS-enabled maze flow did not observe checkpoint POST.")
        if not bool(evidence.get("maze_issue_links_path_seen")):
            raise MazeTraversalFailure("JS-enabled maze flow did not observe issue-links POST.")
        if int(evidence.get("maze_hidden_link_count") or 0) < 1:
            raise MazeTraversalFailure("JS-enabled maze flow did not surface hidden links.")
        replay_outcomes = evidence.get("maze_replay_outcomes")
        if not isinstance(replay_outcomes, list) or not any(
            isinstance(row, dict) and str(row.get("outcome") or "") == "block"
            for row in replay_outcomes
        ):
            raise MazeTraversalFailure("JS-enabled maze replay path did not end in block.")
        replay_event = self.wait_for_recent_fallback_event(
            event_type="Block",
            reason_label="maze_token_replay",
            action_label="block",
            min_ts=started_ts,
        )
        return {
            "entry_path": public_entry["entry_path"],
            "path_prefix": public_entry["path_prefix"],
            "checkpoint_seen": bool(evidence.get("maze_checkpoint_path_seen")),
            "issue_links_seen": bool(evidence.get("maze_issue_links_path_seen")),
            "hidden_link_count": int(evidence.get("maze_hidden_link_count") or 0),
            "replay_outcomes": replay_outcomes,
            "token_replay_event": replay_event,
        }

    def run_js_disabled_fallback(self, *, forwarded_ip: str, entry_path: str, expected: str) -> dict[str, Any]:
        started_ts = int(time.time())
        result = self.run_browser_action(
            action="maze_live_no_js_fallback",
            forwarded_ip=forwarded_ip,
            user_agent="ShumaMazeNoJS/1.0",
            maze_entry_path=entry_path,
            maze_expected_fallback=expected,
            javascript_enabled=False,
        )
        evidence = result.get("browser_evidence")
        if not isinstance(evidence, dict):
            raise MazeTraversalFailure("No-JS maze flow did not return browser evidence.")
        if bool(evidence.get("maze_checkpoint_path_seen")):
            raise MazeTraversalFailure("No-JS maze flow should not submit checkpoint.")
        if bool(evidence.get("maze_issue_links_path_seen")):
            raise MazeTraversalFailure("No-JS maze flow should not issue hidden links.")
        event = self.wait_for_recent_fallback_event(
            event_type="Challenge" if expected == "challenge" else "Block",
            reason_label="maze_checkpoint_missing",
            action_label=expected,
            min_ts=started_ts,
        )
        return {
            "entry_path": entry_path,
            "expected": expected,
            "observed_outcome": str(result.get("observed_outcome") or ""),
            "checkpoint_seen": bool(evidence.get("maze_checkpoint_path_seen")),
            "issue_links_seen": bool(evidence.get("maze_issue_links_path_seen")),
            "fallback_event": event,
        }

    def run_micro_pow_flow(self) -> dict[str, Any]:
        forwarded_ip = fresh_test_ip()
        public_entry = self.discover_public_entry(forwarded_ip=forwarded_ip)
        result = self.run_browser_action(
            action="maze_live_js_flow",
            forwarded_ip=forwarded_ip,
            user_agent="ShumaMazePow/1.0",
            maze_entry_path=public_entry["entry_path"],
            maze_hidden_link_min=1,
            maze_replay_attempts=0,
            maze_expect_pow=True,
            javascript_enabled=True,
        )
        evidence = result.get("browser_evidence")
        if not isinstance(evidence, dict):
            raise MazeTraversalFailure("Micro-PoW maze flow did not return browser evidence.")
        if not bool(evidence.get("maze_first_link_pow_required")):
            raise MazeTraversalFailure("Micro-PoW flow did not expose a PoW-protected maze link.")
        if not browser_request_path_seen(
            evidence,
            f"{public_entry['path_prefix'].rstrip('/')}/checkpoint",
        ):
            raise MazeTraversalFailure("Micro-PoW flow did not retain JS-managed checkpoint behavior.")
        return {
            "entry_path": public_entry["entry_path"],
            "path_prefix": public_entry["path_prefix"],
            "pow_required": bool(evidence.get("maze_first_link_pow_required")),
            "pow_difficulty": int(evidence.get("maze_first_link_pow_difficulty") or 0),
        }

    def run_high_confidence_escalation(self) -> dict[str, Any]:
        forwarded_ip = fresh_test_ip()
        public_entry = self.discover_public_entry(forwarded_ip=forwarded_ip)
        attempts = []
        for attempt in range(1, 4):
            expected = "challenge" if attempt < 3 else "block"
            entry_path = build_opaque_entry_path(
                public_entry["path_prefix"],
                "maze-high-confidence",
                attempt,
            )
            attempts.append(
                self.run_js_disabled_fallback(
                    forwarded_ip=forwarded_ip,
                    entry_path=entry_path,
                    expected=expected,
                )
            )
        return {
            "path_prefix": public_entry["path_prefix"],
            "attempts": attempts,
        }

    def run(self) -> dict[str, Any]:
        self.ensure_health()
        baseline_config = self.fetch_runtime_config()
        baseline_restore_payload = {
            key: baseline_config[key]
            for key in set(MAZE_BROWSER_JS_PROFILE) | set(MAZE_BROWSER_POW_PROFILE)
            if key in baseline_config
        }
        report: dict[str, Any] = {
            "schema_version": REPORT_SCHEMA_VERSION,
            "base_url": self.base_url,
            "js_enabled_flow": {},
            "js_disabled_flow": {},
            "micro_pow_flow": {},
            "high_confidence_escalation": {},
        }
        try:
            self.apply_runtime_config(MAZE_BROWSER_JS_PROFILE)
            report["js_enabled_flow"] = self.run_js_enabled_flow()

            js_disabled_ip = fresh_test_ip()
            public_entry = self.discover_public_entry(forwarded_ip=js_disabled_ip)
            js_disabled_entry = build_opaque_entry_path(
                public_entry["path_prefix"],
                "maze-no-js",
                1,
            )
            report["js_disabled_flow"] = self.run_js_disabled_fallback(
                forwarded_ip=js_disabled_ip,
                entry_path=js_disabled_entry,
                expected="challenge",
            )

            self.apply_runtime_config(MAZE_BROWSER_POW_PROFILE)
            report["micro_pow_flow"] = self.run_micro_pow_flow()

            self.apply_runtime_config(MAZE_BROWSER_JS_PROFILE)
            report["high_confidence_escalation"] = self.run_high_confidence_escalation()
        finally:
            try:
                self.apply_runtime_config(baseline_restore_payload)
            except MazeTraversalFailure as exc:
                raise MazeTraversalFailure(
                    f"Maze live browser proof could not restore the original config: {exc}"
                ) from exc

        self.report_path.parent.mkdir(parents=True, exist_ok=True)
        self.report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return report


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Prove live maze browser behavior over JS/no-JS progression, micro-PoW, replay, and high-confidence escalation."
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
    gate = MazeLiveBrowserGate(
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
        print(f"maze live browser gate failed: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
